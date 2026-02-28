use crate::{BlindKind, JokerRarity, Rank};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Starting hand size for each blind. Defaults to 8.
    #[serde(default = "default_hand_size")]
    pub initial_hand_size: usize,
}

fn default_hand_size() -> usize {
    8
}

/// Stat block for a card enhancement (Bonus, Mult, Glass, etc.).
/// All fields default to zero / no-effect.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnhancementDef {
    /// Flat chips added when this card scores.
    #[serde(default)]
    pub chips: i64,
    /// Flat mult added when this card scores.
    #[serde(default)]
    pub mult_add: f64,
    /// Mult multiplier applied when this card scores.
    #[serde(default)]
    pub mult_mul: f64,
    /// Mult multiplier applied when this card is held (Steel-style).
    #[serde(default)]
    pub mult_mul_held: f64,
    /// 1-in-N probability of destroying this card after scoring (0 = never).
    #[serde(default)]
    pub destroy_odds: u32,
    /// For probabilistic mult: 1-in-N chance of adding `prob_mult_add`.
    #[serde(default)]
    pub prob_mult_odds: u32,
    /// Mult added when the prob_mult roll succeeds.
    #[serde(default)]
    pub prob_mult_add: f64,
    /// For probabilistic money: 1-in-N chance of adding `prob_money_add`.
    #[serde(default)]
    pub prob_money_odds: u32,
    /// Money added when the prob_money roll succeeds.
    #[serde(default)]
    pub prob_money_add: i64,
}

/// Stat block for a card edition (Foil, Holographic, Polychrome).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditionDef {
    /// Flat chips added when this card scores.
    #[serde(default)]
    pub chips: i64,
    /// Flat mult added when this card scores.
    #[serde(default)]
    pub mult_add: f64,
    /// Mult multiplier applied when this card scores.
    #[serde(default)]
    pub mult_mul: f64,
}

/// Stat block for a card seal (Gold, Red, Blue, Purple).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SealDef {
    /// Money added each time this card is scored in a hand.
    #[serde(default)]
    pub money_scored: i64,
    /// Money added for held-in-hand cards at round end.
    #[serde(default)]
    pub money_held: i64,
    /// If true, grants a planet card matching the played hand (Blue seal).
    /// If absent, defaults to the standard behavior (true for Blue seal).
    #[serde(default)]
    pub grant_planet: Option<bool>,
    /// If true, grants a random tarot when the card is discarded (Purple seal).
    /// If absent, defaults to the standard behavior (true for Purple seal).
    #[serde(default)]
    pub grant_tarot_discard: Option<bool>,
}

/// All card attribute balance values. Keys are the lowercase variant name
/// (e.g. `"bonus"`, `"foil"`, `"gold"`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardAttrRules {
    #[serde(default)]
    pub enhancements: HashMap<String, EnhancementDef>,
    #[serde(default)]
    pub editions: HashMap<String, EditionDef>,
    #[serde(default)]
    pub seals: HashMap<String, SealDef>,
}

impl CardAttrRules {
    pub fn enhancement(&self, key: &str) -> &EnhancementDef {
        self.enhancements.get(key).map(|d| d).unwrap_or(&DEFAULT_ENHANCEMENT)
    }
    pub fn edition(&self, key: &str) -> &EditionDef {
        self.editions.get(key).map(|d| d).unwrap_or(&DEFAULT_EDITION)
    }
    pub fn seal(&self, key: &str) -> &SealDef {
        self.seals.get(key).map(|d| d).unwrap_or(&DEFAULT_SEAL)
    }
}

static DEFAULT_ENHANCEMENT: EnhancementDef = EnhancementDef {
    chips: 0, mult_add: 0.0, mult_mul: 0.0, mult_mul_held: 0.0,
    destroy_odds: 0, prob_mult_odds: 0, prob_mult_add: 0.0,
    prob_money_odds: 0, prob_money_add: 0,
};
static DEFAULT_EDITION: EditionDef = EditionDef { chips: 0, mult_add: 0.0, mult_mul: 0.0 };
static DEFAULT_SEAL: SealDef = SealDef {
    money_scored: 0, money_held: 0, grant_planet: None, grant_tarot_discard: None,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub hands: Vec<HandRule>,
    pub ranks: Vec<RankRule>,
    pub blinds: Vec<BlindRule>,
    pub antes: Vec<AnteRule>,
    pub economy: EconomyRule,
    pub shop: ShopRule,
    /// Card attribute balance values (enhancement/edition/seal stats).
    /// If absent (old configs), falls back to hardcoded defaults.
    #[serde(default)]
    pub card_attrs: CardAttrRules,
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
