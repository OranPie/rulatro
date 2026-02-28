use super::helpers::{is_face, normalize};
use super::*;
use crate::*;

impl RunState {
    pub fn new(config: GameConfig, content: Content, seed: u64) -> Self {
        let mut rng = RngState::from_seed(seed);
        let mut deck = Deck::standard52();
        let mut next_card_id = 1u32;
        for card in &mut deck.draw {
            card.id = next_card_id;
            next_card_id = next_card_id.saturating_add(1);
        }
        deck.shuffle(&mut rng);
        let tables = ScoreTables::from_config(&config);
        let initial_hand_size = config.economy.initial_hand_size;
        let joker_slots = config.economy.initial_joker_slots;
        let consumable_slots = config.economy.initial_consumable_slots;
        let mut state = GameState::new();
        state.hand_size_base = initial_hand_size;
        state.hand_size = initial_hand_size;
        Self {
            config,
            tables,
            content,
            inventory: Inventory::with_slots(joker_slots, consumable_slots),
            rng,
            deck,
            hand: Vec::new(),
            state,
            shop: None,
            pending_effects: Vec::new(),
            last_score_trace: Vec::new(),
            current_joker_counts: HashMap::new(),
            current_joker_snapshot: Vec::new(),
            pending_joker_removals: Vec::new(),
            pending_joker_additions: Vec::new(),
            last_destroyed_sell_value: 0,
            boss_disable_pending: false,
            boss_disabled: false,
            prevent_death: false,
            rule_vars: HashMap::new(),
            rule_dirty: true,
            refreshing_rules: false,
            next_card_id,
            copy_depth: 0,
            copy_stack: Vec::new(),
            joker_effect_depth: 0,
            deferred_card_added: Vec::new(),
            mod_runtime: None,
            hooks: HookRegistry::with_defaults(),
            custom_hand_registry: Vec::new(),
        }
    }

    pub fn set_mod_runtime(&mut self, runtime: Option<Box<dyn ModRuntime>>) {
        self.mod_runtime = runtime;
    }

    pub fn current_boss(&self) -> Option<&BossDef> {
        self.state
            .boss_id
            .as_deref()
            .and_then(|id| self.content.boss_by_id(id))
    }

    pub fn boss_effects_disabled(&self) -> bool {
        self.boss_disabled
    }

    pub fn current_boss_effect_summaries(&self) -> Vec<String> {
        self.current_boss()
            .map(|boss| {
                boss.effects
                    .iter()
                    .map(format_joker_effect_compact)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn active_voucher_ids(&self) -> &[String] {
        &self.state.active_vouchers
    }

    pub fn active_voucher_summaries(&self, zh_cn: bool) -> Vec<String> {
        self.state
            .active_vouchers
            .iter()
            .map(|id| {
                if let Some(voucher) = voucher_by_id(id) {
                    format!(
                        "{} ({}) - {}",
                        voucher.name(zh_cn),
                        voucher.id,
                        voucher.effect_text(zh_cn)
                    )
                } else {
                    id.clone()
                }
            })
            .collect()
    }

    pub(super) fn hand_eval_rules(&mut self) -> HandEvalRules {
        self.ensure_rule_vars();
        HandEvalRules {
            smeared_suits: self.rule_flag("smeared_suits"),
            four_fingers: self.rule_flag("four_fingers"),
            shortcut: self.rule_flag("shortcut"),
        }
    }

    pub(super) fn mark_rules_dirty(&mut self) {
        self.rule_dirty = true;
    }

    pub(super) fn set_rule_var(&mut self, key: &str, value: f64) {
        let key = normalize(key);
        self.rule_vars.insert(key, value);
    }

    pub(super) fn add_rule_var(&mut self, key: &str, delta: f64) {
        let key = normalize(key);
        let entry = self.rule_vars.entry(key).or_insert(0.0);
        *entry += delta;
    }

    pub(super) fn rule_value_or(&mut self, key: &str, default: f64) -> f64 {
        self.ensure_rule_vars();
        let key = normalize(key);
        self.rule_vars.get(&key).copied().unwrap_or(default)
    }

    pub(super) fn rule_value(&mut self, key: &str) -> f64 {
        self.ensure_rule_vars();
        let key = normalize(key);
        self.rule_vars.get(&key).copied().unwrap_or(0.0)
    }

    pub(super) fn rule_flag(&mut self, key: &str) -> bool {
        self.rule_value(key) != 0.0
    }

    pub(super) fn smeared_suits_active(&mut self) -> bool {
        self.rule_flag("smeared_suits")
    }

    pub(super) fn pareidolia_active(&mut self) -> bool {
        self.rule_flag("pareidolia")
    }

    pub(super) fn splash_active(&mut self) -> bool {
        self.rule_flag("splash")
    }

    pub(super) fn money_floor(&mut self) -> i64 {
        let floor = self.rule_value("money_floor").floor() as i64;
        if floor < 0 {
            floor
        } else {
            0
        }
    }

    pub(super) fn alloc_card_id(&mut self) -> u32 {
        let id = self.next_card_id;
        self.next_card_id = self.next_card_id.saturating_add(1);
        id
    }

    pub(super) fn assign_card_id(&mut self, card: &mut crate::Card) {
        if card.id == 0 {
            card.id = self.alloc_card_id();
        }
    }

    pub(super) fn clear_score_trace(&mut self) {
        self.last_score_trace.clear();
    }

    pub(super) fn apply_rule_effect(
        &mut self,
        score: &mut Score,
        effect: RuleEffect,
        source: &str,
    ) {
        let before = score.clone();
        score.apply(&effect);
        let after = score.clone();
        self.last_score_trace.push(crate::ScoreTraceStep {
            source: source.to_string(),
            effect,
            before,
            after,
        });
    }

    pub(super) fn is_card_debuffed(&mut self, card: crate::Card) -> bool {
        // Compute base debuff from rule_flags.
        let base = self.rule_flag("debuff_face") && is_face(card)
            || self.rule_flag("debuff_suit_spades") && card.suit == crate::Suit::Spades
            || self.rule_flag("debuff_suit_hearts") && card.suit == crate::Suit::Hearts
            || self.rule_flag("debuff_suit_clubs") && card.suit == crate::Suit::Clubs
            || self.rule_flag("debuff_suit_diamonds") && card.suit == crate::Suit::Diamonds
            || (self.rule_flag("debuff_played_ante")
                && self.state.played_card_ids_ante.contains(&card.id));
        // Apply Flow Kernel CardDebuff patch.
        let patch = if let Some(rt) = self.mod_runtime.as_mut() {
            let ctx = FlowCtx::card_debuff(&self.state, card);
            rt.flow_card_debuff_patch(CardDebuffPatch::default(), &ctx)
        } else {
            CardDebuffPatch::default()
        };
        patch.resolve(base)
    }

    pub(super) fn boss_disabled(&self) -> bool {
        self.boss_disabled
    }

    fn ensure_rule_vars(&mut self) {
        if !self.rule_dirty || self.refreshing_rules {
            return;
        }
        self.rule_dirty = false;
        self.refreshing_rules = true;
        self.rule_vars.clear();
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut scratch_money = self.state.money;
        let mut scratch_results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::independent(
            hand_kind,
            self.state.blind,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut scratch_money,
            &mut scratch_results,
        );
        let mut scratch_events = EventBus::default();
        self.invoke_hooks(HookPoint::Passive, &mut args, &mut scratch_events);
        self.refreshing_rules = false;
    }

    pub(super) fn most_played_hand(&self) -> crate::HandKind {
        let mut best = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut best_count = 0u32;
        for kind in crate::HandKind::ALL {
            let count = self.state.hand_play_counts.get(&kind).copied().unwrap_or(0);
            if count > best_count {
                best_count = count;
                best = kind;
            }
        }
        best
    }

    pub(super) fn hand_level(&self, kind: crate::HandKind) -> u32 {
        let key = crate::level_kind(kind);
        self.state.hand_levels.get(&key).copied().unwrap_or(1)
    }

    pub(super) fn upgrade_hand_level(&mut self, kind: crate::HandKind, amount: u32) {
        if amount == 0 {
            return;
        }
        let key = crate::level_kind(kind);
        let entry = self.state.hand_levels.entry(key).or_insert(1);
        *entry = entry.saturating_add(amount);
    }

    pub(super) fn upgrade_all_hands(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        for kind in crate::HandKind::ALL {
            self.upgrade_hand_level(kind, amount);
        }
    }

    pub(super) fn random_range_values(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let span = (max - min) as u64;
        let roll = self.rng.next_u64() % (span + 1);
        min + roll as i64
    }

    pub(super) fn roll(&mut self, sides: u64) -> bool {
        if sides == 0 {
            return false;
        }
        let mut multiplier = 1u64;
        let count = self.rule_value("roll_bonus").floor().max(0.0) as u32;
        for _ in 0..count {
            multiplier = multiplier.saturating_mul(2);
        }
        let successes = sides.min(multiplier);
        self.rng.next_u64() % sides < successes
    }
}
