use super::helpers::normalize;
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
        let mut content = content;
        if content.card_modifiers.is_empty() {
            content.card_modifiers =
                super::builtin_card_modifiers::build_builtin_card_modifiers(&config.card_attrs);
        }
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

    /// Apply deck startup effects. Call after `RunState::new()` to customise the starting state.
    pub fn apply_deck(&mut self, deck_id: &str) {
        match deck_id {
            "red" => { /* +1 discard per round handled by DeckHook at BlindStart */ }
            "blue" => { /* +1 hand per round handled by DeckHook at BlindStart */ }
            "yellow" => {
                self.state.money += 10;
            }
            "green" => { /* money bonus handled by DeckHook at RoundEnd */ }
            "black" => {
                self.inventory.joker_slots += 1;
                // -1 hand per round handled by DeckHook at BlindStart
            }
            "magic" => {
                self.state
                    .active_vouchers
                    .push("v_crystal_ball".to_string());
                for _ in 0..2 {
                    if self.inventory.consumables.len() < self.inventory.consumable_slots {
                        self.inventory.consumables.push(ConsumableInstance {
                            id: "c_fool".to_string(),
                            kind: ConsumableKind::Tarot,
                            edition: None,
                            sell_bonus: 0.0,
                        });
                    }
                }
            }
            "nebula" => {
                self.state.active_vouchers.push("v_telescope".to_string());
                self.inventory.consumable_slots = self.inventory.consumable_slots.saturating_sub(1);
            }
            "ghost" => {
                self.rule_vars.insert("deck_ghost".to_string(), 1.0);
                if self.inventory.consumables.len() < self.inventory.consumable_slots {
                    self.inventory.consumables.push(ConsumableInstance {
                        id: "c_hex".to_string(),
                        kind: ConsumableKind::Spectral,
                        edition: None,
                        sell_bonus: 0.0,
                    });
                }
            }
            "abandoned" => {
                self.deck
                    .draw
                    .retain(|card| !matches!(card.rank, Rank::Jack | Rank::Queen | Rank::King));
            }
            "checkered" => {
                let mut new_draw: Vec<Card> = Vec::new();
                let mut card_id = self.next_card_id;
                for suit in [Suit::Spades, Suit::Hearts] {
                    for rank in [
                        Rank::Ace,
                        Rank::Two,
                        Rank::Three,
                        Rank::Four,
                        Rank::Five,
                        Rank::Six,
                        Rank::Seven,
                        Rank::Eight,
                        Rank::Nine,
                        Rank::Ten,
                        Rank::Jack,
                        Rank::Queen,
                        Rank::King,
                    ] {
                        let mut c = Card::standard(suit, rank);
                        c.id = card_id;
                        card_id = card_id.saturating_add(1);
                        new_draw.push(c);
                    }
                }
                self.next_card_id = card_id;
                self.deck.draw = new_draw;
                self.deck.discard.clear();
                self.deck.shuffle(&mut self.rng);
            }
            "zodiac" => {
                for v in ["v_tarot_merchant", "v_planet_merchant", "v_overstock"] {
                    self.state.active_vouchers.push(v.to_string());
                }
            }
            "painted" => {
                self.state.hand_size_base += 2;
                self.state.hand_size += 2;
                self.inventory.joker_slots = self.inventory.joker_slots.saturating_sub(1);
            }
            "anaglyph" => {
                self.rule_vars.insert("deck_anaglyph".to_string(), 1.0);
            }
            "plasma" => {
                self.rule_vars.insert("deck_plasma".to_string(), 1.0);
            }
            "erratic" => {
                for card in &mut self.deck.draw {
                    let rank_idx = self.rng.next_u64() % 13;
                    let suit_idx = self.rng.next_u64() % 4;
                    card.rank = [
                        Rank::Ace,
                        Rank::Two,
                        Rank::Three,
                        Rank::Four,
                        Rank::Five,
                        Rank::Six,
                        Rank::Seven,
                        Rank::Eight,
                        Rank::Nine,
                        Rank::Ten,
                        Rank::Jack,
                        Rank::Queen,
                        Rank::King,
                    ][rank_idx as usize];
                    card.suit = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
                        [suit_idx as usize];
                }
            }
            _ => {}
        }
        self.rule_vars.insert(format!("deck_{}", deck_id), 1.0);
    }

    /// Create a new run and immediately apply the given deck's starting effects.
    pub fn new_with_deck(config: GameConfig, content: Content, seed: u64, deck_id: &str) -> Self {
        let mut run = Self::new(config, content, seed);
        run.apply_deck(deck_id);
        run
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
                if let Some(voucher) = self.content.voucher_by_id(id) {
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
        // Snapshot rules to avoid simultaneous borrow of self.content and self (rule_flag is &mut).
        let rules: Vec<crate::CardConditionalRule> = self.content.debuff_rules.clone();
        let played_ids: std::collections::HashSet<u32> = self.state.played_card_ids_ante.clone();
        // Compute base debuff from data-driven debuff_rules table.
        let base = rules
            .iter()
            .any(|rule| self.rule_flag(&rule.key) && rule.condition.matches(card, &played_ids));
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
