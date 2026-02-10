use crate::{BlindKind, JokerRarity, Rank};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandRule {
    pub id: String,
    pub display_name: String,
    pub base_chips: i64,
    pub base_mult: f64,
    #[serde(default)]
    pub level_chips: i64,
    #[serde(default)]
    pub level_mult: f64,
    pub priority: u8,
    pub min_cards: u8,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankRule {
    pub rank: Rank,
    pub chips: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindRule {
    pub kind: BlindKind,
    pub target_mult: f32,
    pub hands: u8,
    pub discards: u8,
    pub can_skip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnteRule {
    pub ante: u8,
    pub base_target: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShopCardKind {
    Joker,
    Tarot,
    Planet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardWeight {
    pub kind: ShopCardKind,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JokerRarityWeight {
    pub rarity: JokerRarity,
    pub weight: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackKind {
    Arcana,
    Buffoon,
    Celestial,
    Spectral,
    Standard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackSize {
    Normal,
    Jumbo,
    Mega,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackWeight {
    pub kind: PackKind,
    pub size: PackSize,
    pub weight: u32,
    pub options: u8,
    pub picks: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackPrice {
    pub size: PackSize,
    pub price: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopPrices {
    pub joker_common: PriceRange,
    pub joker_uncommon: PriceRange,
    pub joker_rare: PriceRange,
    pub joker_legendary: i64,
    pub tarot: i64,
    pub planet: i64,
    pub spectral: i64,
    pub playing_card: i64,
    pub voucher: i64,
    pub reroll_base: i64,
    pub reroll_step: i64,
    pub pack_prices: Vec<PackPrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopRule {
    pub card_slots: u8,
    pub booster_slots: u8,
    pub voucher_slots: u8,
    pub card_weights: Vec<CardWeight>,
    pub joker_rarity_weights: Vec<JokerRarityWeight>,
    pub pack_weights: Vec<PackWeight>,
    pub prices: ShopPrices,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyRule {
    pub reward_small: i64,
    pub reward_big: i64,
    pub reward_boss: i64,
    pub per_hand_reward: i64,
    pub interest_step: i64,
    pub interest_per: i64,
    pub interest_cap: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub hands: Vec<HandRule>,
    pub ranks: Vec<RankRule>,
    pub blinds: Vec<BlindRule>,
    pub antes: Vec<AnteRule>,
    pub economy: EconomyRule,
    pub shop: ShopRule,
}

impl GameConfig {
    pub fn blind_rule(&self, kind: BlindKind) -> Option<&BlindRule> {
        self.blinds.iter().find(|rule| rule.kind == kind)
    }

    pub fn ante_rule(&self, ante: u8) -> Option<&AnteRule> {
        self.antes.iter().find(|rule| rule.ante == ante)
    }

    pub fn target_for(&self, ante: u8, kind: BlindKind) -> Option<i64> {
        let base = self.ante_rule(ante)?.base_target;
        let mult = self.blind_rule(kind)?.target_mult;
        Some((base as f32 * mult).round() as i64)
    }

    pub fn max_ante(&self) -> Option<u8> {
        self.antes.iter().map(|rule| rule.ante).max()
    }
}
