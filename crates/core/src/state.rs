use crate::{EventBus, HandKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub hand_levels: HashMap<HandKind, u32>,
    #[serde(default)]
    pub shop_free_rerolls: u8,
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
            hand_levels: HashMap::new(),
            shop_free_rerolls: 0,
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
