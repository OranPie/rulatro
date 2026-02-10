use crate::{BlindKind, Edition, Enhancement, HandKind, Rank, RuleEffect, Seal, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JokerRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsumableKind {
    Tarot,
    Planet,
    Spectral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivationType {
    OnPlayed,
    OnScored,
    OnHeld,
    Independent,
    OnOtherJokers,
    OnDiscard,
    OnDiscardBatch,
    OnCardDestroyed,
    OnRoundEnd,
    OnBlindStart,
    OnShopEnter,
    OnShopReroll,
    OnPackOpened,
    OnPackSkipped,
    OnUse,
    OnSell,
    OnAnySell,
    OnAcquire,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectBlock {
    pub trigger: ActivationType,
    pub conditions: Vec<Condition>,
    pub effects: Vec<EffectOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    Always,
    HandKind(HandKind),
    BlindKind(BlindKind),
    CardSuit(Suit),
    CardRank(Rank),
    CardIsFace,
    CardIsOdd,
    CardIsEven,
    CardHasEnhancement(Enhancement),
    CardHasEdition(Edition),
    CardHasSeal(Seal),
    CardIsStone,
    CardIsWild,
    IsBossBlind,
    IsScoringCard,
    IsHeldCard,
    IsPlayedCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectOp {
    Score(RuleEffect),
    AddMoney(i64),
    AddHandSize(i64),
    RetriggerScored(i64),
    RetriggerHeld(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOp {
    AddChips,
    AddMult,
    MultiplyMult,
    MultiplyChips,
    AddMoney,
    AddHandSize,
    RetriggerScored,
    RetriggerHeld,
    AddStoneCard,
    AddHands,
    AddDiscards,
    SetDiscards,
    AddTarot,
    AddPlanet,
    AddSpectral,
    AddFreeReroll,
    SetShopPrice,
    AddJoker,
    DestroyRandomJoker,
    DestroyJokerRight,
    DestroyJokerLeft,
    DestroySelf,
    UpgradeHand,
    DuplicateRandomJoker,
    SetVar,
    AddVar,
}

impl ActionOp {
    pub fn from_keyword(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "add_chips" => Some(Self::AddChips),
            "add_mult" => Some(Self::AddMult),
            "mul_mult" | "multiply_mult" => Some(Self::MultiplyMult),
            "mul_chips" | "multiply_chips" => Some(Self::MultiplyChips),
            "add_money" => Some(Self::AddMoney),
            "add_hand_size" => Some(Self::AddHandSize),
            "retrigger_scored" => Some(Self::RetriggerScored),
            "retrigger_held" => Some(Self::RetriggerHeld),
            "add_stone_card" => Some(Self::AddStoneCard),
            "add_hands" => Some(Self::AddHands),
            "add_discards" => Some(Self::AddDiscards),
            "set_discards" => Some(Self::SetDiscards),
            "add_tarot" => Some(Self::AddTarot),
            "add_planet" => Some(Self::AddPlanet),
            "add_spectral" => Some(Self::AddSpectral),
            "add_free_reroll" => Some(Self::AddFreeReroll),
            "set_shop_price" | "shop_price" => Some(Self::SetShopPrice),
            "add_joker" | "add_random_joker" => Some(Self::AddJoker),
            "destroy_random_joker" | "destroy_joker_random" => Some(Self::DestroyRandomJoker),
            "destroy_joker_right" | "destroy_right_joker" => Some(Self::DestroyJokerRight),
            "destroy_joker_left" | "destroy_left_joker" => Some(Self::DestroyJokerLeft),
            "destroy_self" => Some(Self::DestroySelf),
            "upgrade_hand" => Some(Self::UpgradeHand),
            "duplicate_random_joker" | "dup_random_joker" => Some(Self::DuplicateRandomJoker),
            "set_var" => Some(Self::SetVar),
            "add_var" => Some(Self::AddVar),
            _ => None,
        }
    }

    pub fn requires_target(self) -> bool {
        matches!(
            self,
            Self::SetVar | Self::AddVar | Self::SetShopPrice | Self::AddJoker
        )
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    pub op: ActionOp,
    pub target: Option<String>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct JokerEffect {
    pub trigger: ActivationType,
    pub when: Expr,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool),
    Number(f64),
    String(String),
    Ident(String),
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}
