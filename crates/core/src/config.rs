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
    /// Initial number of joker slots. Defaults to 5.
    #[serde(default = "default_joker_slots")]
    pub initial_joker_slots: usize,
    /// Initial number of consumable slots. Defaults to 2.
    #[serde(default = "default_consumable_slots")]
    pub initial_consumable_slots: usize,
    /// Divisor applied to a joker's buy price when calculating its sell value.
    /// Defaults to 2 (sell for half the buy price).
    #[serde(default = "default_joker_sell_divisor")]
    pub joker_sell_divisor: i64,
}

fn default_hand_size() -> usize {
    8
}
fn default_joker_slots() -> usize {
    5
}
fn default_consumable_slots() -> usize {
    2
}
fn default_joker_sell_divisor() -> i64 {
    2
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
    /// Look up an enhancement by key, falling back to built-in canonical defaults.
    pub fn enhancement(&self, key: &str) -> EnhancementDef {
        self.enhancements
            .get(key)
            .cloned()
            .unwrap_or_else(|| builtin_enhancement(key))
    }
    /// Look up an edition by key, falling back to built-in canonical defaults.
    pub fn edition(&self, key: &str) -> EditionDef {
        self.editions
            .get(key)
            .cloned()
            .unwrap_or_else(|| builtin_edition(key))
    }
    /// Look up a seal by key, falling back to built-in canonical defaults.
    pub fn seal(&self, key: &str) -> SealDef {
        self.seals
            .get(key)
            .cloned()
            .unwrap_or_else(|| builtin_seal(key))
    }

    /// Resolve a dot-notation lookup key like `"enhancement.chips"` for modifier `id`.
    ///
    /// The namespace (`enhancement`, `edition`, `seal`) selects which stat block to use;
    /// the field selects the specific value within that block.
    /// `id` is the modifier's id (e.g. `"bonus"`, `"foil"`, `"gold"`).
    pub fn resolve_lookup(&self, key: &str, id: &str) -> f64 {
        let Some(dot) = key.find('.') else {
            return 0.0;
        };
        let namespace = &key[..dot];
        let field = &key[dot + 1..];
        match namespace {
            "enhancement" => {
                let def = self.enhancement(id);
                match field {
                    "chips" => def.chips as f64,
                    "mult" => def.mult_add,
                    "x_mult" => def.mult_mul,
                    "x_mult_held" => def.mult_mul_held,
                    "destroy_odds" => def.destroy_odds as f64,
                    "lucky_mult_odds" => def.prob_mult_odds as f64,
                    "lucky_mult" => def.prob_mult_add,
                    "lucky_money_odds" => def.prob_money_odds as f64,
                    "lucky_money" => def.prob_money_add as f64,
                    _ => 0.0,
                }
            }
            "edition" => {
                let def = self.edition(id);
                match field {
                    "chips" => def.chips as f64,
                    "mult" => def.mult_add,
                    "x_mult" => def.mult_mul,
                    _ => 0.0,
                }
            }
            "seal" => {
                let def = self.seal(id);
                match field {
                    "money_scored" => def.money_scored as f64,
                    "money_held" => def.money_held as f64,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        }
    }
}

fn builtin_enhancement(key: &str) -> EnhancementDef {
    match key {
        "bonus" => EnhancementDef {
            chips: 30,
            ..Default::default()
        },
        "mult" => EnhancementDef {
            mult_add: 4.0,
            ..Default::default()
        },
        "glass" => EnhancementDef {
            mult_mul: 2.0,
            destroy_odds: 4,
            ..Default::default()
        },
        "steel" => EnhancementDef {
            mult_mul_held: 1.5,
            ..Default::default()
        },
        "stone" => EnhancementDef {
            chips: 50,
            ..Default::default()
        },
        "lucky" => EnhancementDef {
            prob_mult_odds: 5,
            prob_mult_add: 20.0,
            prob_money_odds: 15,
            prob_money_add: 20,
            ..Default::default()
        },
        _ => EnhancementDef::default(),
    }
}

fn builtin_edition(key: &str) -> EditionDef {
    match key {
        "foil" => EditionDef {
            chips: 50,
            ..Default::default()
        },
        "holographic" => EditionDef {
            mult_add: 10.0,
            ..Default::default()
        },
        "polychrome" => EditionDef {
            mult_mul: 1.5,
            ..Default::default()
        },
        _ => EditionDef::default(),
    }
}

fn builtin_seal(key: &str) -> SealDef {
    match key {
        "gold" => SealDef {
            money_scored: 3,
            money_held: 3,
            ..Default::default()
        },
        "blue" => SealDef {
            grant_planet: Some(true),
            ..Default::default()
        },
        "purple" => SealDef {
            grant_tarot_discard: Some(true),
            ..Default::default()
        },
        _ => SealDef::default(),
    }
}

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
