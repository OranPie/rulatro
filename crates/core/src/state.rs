use crate::{ConsumableKind, EventBus, HandKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Deal,
    Play,
    Score,
    Cleanup,
    Shop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlindKind {
    Small,
    Big,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub ante: u8,
    pub blind: BlindKind,
    pub phase: Phase,
    pub target: i64,
    pub blind_score: i64,
    pub hands_left: u8,
    pub discards_left: u8,
    #[serde(default)]
    pub hands_max: u8,
    #[serde(default)]
    pub discards_max: u8,
    #[serde(default)]
    pub hand_size_base: usize,
    pub hand_size: usize,
    pub money: i64,
    #[serde(default)]
    pub last_hand: Option<HandKind>,
    #[serde(default)]
    pub hand_play_counts: HashMap<HandKind, u32>,
    #[serde(default)]
    pub round_hand_types: HashSet<HandKind>,
    #[serde(default)]
    pub round_hand_lock: Option<HandKind>,
    #[serde(default)]
    pub played_card_ids_ante: HashSet<u32>,
    #[serde(default)]
    pub hand_levels: HashMap<HandKind, u32>,
    #[serde(default)]
    pub shop_free_rerolls: u8,
    #[serde(default)]
    pub blinds_skipped: u32,
    #[serde(default)]
    pub planets_used: HashSet<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub duplicate_next_tag: bool,
    #[serde(default)]
    pub duplicate_tag_exclude: Option<String>,
    #[serde(default)]
    pub unused_discards: u32,
    #[serde(default)]
    pub last_consumable: Option<LastConsumable>,
    #[serde(default)]
    pub boss_id: Option<String>,
    #[serde(default)]
    pub active_vouchers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastConsumable {
    pub kind: ConsumableKind,
    pub id: String,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            ante: 1,
            blind: BlindKind::Small,
            phase: Phase::Setup,
            target: 0,
            blind_score: 0,
            hands_left: 0,
            discards_left: 0,
            hands_max: 0,
            discards_max: 0,
            hand_size_base: 8,
            hand_size: 8,
            money: 0,
            last_hand: None,
            hand_play_counts: HashMap::new(),
            round_hand_types: HashSet::new(),
            round_hand_lock: None,
            played_card_ids_ante: HashSet::new(),
            hand_levels: HashMap::new(),
            shop_free_rerolls: 0,
            blinds_skipped: 0,
            planets_used: HashSet::new(),
            tags: Vec::new(),
            duplicate_next_tag: false,
            duplicate_tag_exclude: None,
            unused_discards: 0,
            last_consumable: None,
            boss_id: None,
            active_vouchers: Vec::new(),
        }
    }

    pub fn advance_phase(&mut self, _events: &mut EventBus) {
        self.phase = match self.phase {
            Phase::Setup => Phase::Deal,
            Phase::Deal => Phase::Play,
            Phase::Play => Phase::Score,
            Phase::Score => Phase::Cleanup,
            Phase::Cleanup => Phase::Deal,
            Phase::Shop => Phase::Deal,
        };
    }
}
