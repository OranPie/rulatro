use serde::{Deserialize, Serialize};

pub use rulatro_core::{
    ActivationType, AnteRule, BlindRule, BossDef, CardAttrRules, Condition, ConsumableDef,
    ConsumableKind, Content, EconomyRule, EffectBlock, EffectOp, GameConfig, HandRule, JokerDef,
    JokerRarity, JokerRarityWeight, PackPrice, PackSize, PriceRange, RankRule, ShopPrices,
    ShopRule, TagDef,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub id: String,
    pub kind: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackJokerDef {
    pub id: String,
    pub display_name: String,
    pub rarity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPack {
    pub cards: Vec<CardDef>,
    pub jokers: Vec<PackJokerDef>,
}
