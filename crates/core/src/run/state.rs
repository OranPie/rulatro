use super::*;
use crate::*;

impl RunState {
    pub fn new(config: GameConfig, content: Content, seed: u64) -> Self {
        let mut rng = RngState::from_seed(seed);
        let mut deck = Deck::standard52();
        deck.shuffle(&mut rng);
        let tables = ScoreTables::from_config(&config);
        Self {
            config,
            tables,
            content,
            inventory: Inventory::new(),
            rng,
            deck,
            hand: Vec::new(),
            state: GameState::new(),
            shop: None,
            pending_effects: Vec::new(),
            current_joker_counts: HashMap::new(),
            current_joker_snapshot: Vec::new(),
            pending_joker_removals: Vec::new(),
            pending_joker_additions: Vec::new(),
        }
    }

    pub(super) fn hand_eval_rules(&self) -> HandEvalRules {
        HandEvalRules {
            smeared_suits: self.smeared_suits_active(),
            four_fingers: self.four_fingers_active(),
            shortcut: self.shortcut_active(),
        }
    }

    pub(super) fn has_joker_id(&self, id: &str) -> bool {
        self.inventory.jokers.iter().any(|joker| joker.id == id)
    }

    pub(super) fn smeared_suits_active(&self) -> bool {
        self.has_joker_id("smeared_joker")
    }

    pub(super) fn four_fingers_active(&self) -> bool {
        self.has_joker_id("four_fingers")
    }

    pub(super) fn pareidolia_active(&self) -> bool {
        self.has_joker_id("pareidolia")
    }

    pub(super) fn splash_active(&self) -> bool {
        self.has_joker_id("splash")
    }

    pub(super) fn money_floor(&self) -> i64 {
        if self.has_joker_id("credit_card") {
            -20
        } else {
            0
        }
    }

    pub(super) fn shortcut_active(&self) -> bool {
        self.has_joker_id("shortcut")
    }

    pub(super) fn most_played_hand(&self) -> crate::HandKind {
        let mut best = self
            .state
            .last_hand
            .unwrap_or(crate::HandKind::HighCard);
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
        self.rng.next_u64() % sides == 0
    }

}
