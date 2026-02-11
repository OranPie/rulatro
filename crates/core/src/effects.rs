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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActivationType {
    OnPlayed,
    OnScoredPre,
    OnScored,
    OnHeld,
    Independent,
    OnOtherJokers,
    OnDiscard,
    OnDiscardBatch,
    OnCardDestroyed,
    OnCardAdded,
    OnRoundEnd,
    OnHandEnd,
    OnBlindStart,
    OnBlindFailed,
    OnShopEnter,
    OnShopReroll,
    OnShopExit,
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
    SetMoney(i64),
    DoubleMoney { cap: i64 },
    AddMoneyFromJokers { cap: i64 },
    AddHandSize(i64),
    UpgradeHand { hand: HandKind, amount: u32 },
    UpgradeAllHands { amount: u32 },
    AddRandomConsumable { kind: ConsumableKind, count: u8 },
    AddJoker { rarity: JokerRarity, count: u8 },
    AddRandomJoker { count: u8 },
    RandomJokerEdition { editions: Vec<Edition>, chance: f64 },
    SetRandomJokerEdition { edition: Edition },
    SetRandomJokerEditionDestroyOthers { edition: Edition },
    DuplicateRandomJokerDestroyOthers { remove_negative: bool },
    EnhanceSelected { enhancement: Enhancement, count: u8 },
    AddEditionToSelected { editions: Vec<Edition>, count: u8 },
    AddSealToSelected { seal: Seal, count: u8 },
    ConvertSelectedSuit { suit: Suit, count: u8 },
    IncreaseSelectedRank { count: u8, delta: i8 },
    DestroySelected { count: u8 },
    DestroyRandomInHand { count: u8 },
    CopySelected { count: u8 },
    ConvertLeftIntoRight,
    ConvertHandToRandomRank,
    ConvertHandToRandomSuit,
    AddRandomEnhancedCards { count: u8, filter: RankFilter },
    CreateLastConsumable { exclude: Option<String> },
    RetriggerScored(i64),
    RetriggerHeld(i64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RankFilter {
    Any,
    Face,
    Ace,
    Numbered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOp {
    AddChips,
    AddMult,
    MultiplyMult,
    MultiplyChips,
    AddMoney,
    SetMoney,
    AddHandSize,
    RetriggerScored,
    RetriggerHeld,
    AddStoneCard,
    AddCardBonus,
    SetCardEnhancement,
    ClearCardEnhancement,
    DestroyCard,
    CopyPlayedCard,
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
    DuplicateRandomConsumable,
    AddSellBonus,
    DisableBoss,
    AddRandomHandCard,
    CopyJokerRight,
    CopyJokerLeftmost,
    PreventDeath,
    AddTag,
    DuplicateNextTag,
    AddPack,
    AddShopJoker,
    AddVoucher,
    SetRerollCost,
    SetShopJokerEdition,
    RerollBoss,
    UpgradeRandomHand,
    SetHands,
    MultiplyTarget,
    SetRule,
    AddRule,
    ClearRule,
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
            "set_money" | "money_set" => Some(Self::SetMoney),
            "add_hand_size" => Some(Self::AddHandSize),
            "retrigger_scored" => Some(Self::RetriggerScored),
            "retrigger_held" => Some(Self::RetriggerHeld),
            "add_stone_card" => Some(Self::AddStoneCard),
            "add_card_bonus" | "add_card_chips" | "card_bonus" => Some(Self::AddCardBonus),
            "set_card_enhancement" | "set_enhancement" | "card_enhancement" => {
                Some(Self::SetCardEnhancement)
            }
            "clear_card_enhancement" | "remove_card_enhancement" | "clear_enhancement" => {
                Some(Self::ClearCardEnhancement)
            }
            "destroy_card" | "destroy_current_card" => Some(Self::DestroyCard),
            "copy_played_card" | "copy_card" | "copy_scoring_card" => Some(Self::CopyPlayedCard),
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
            "duplicate_random_consumable" | "dup_random_consumable" => {
                Some(Self::DuplicateRandomConsumable)
            }
            "add_sell_bonus" | "sell_bonus" => Some(Self::AddSellBonus),
            "disable_boss" | "boss_disable" => Some(Self::DisableBoss),
            "add_random_hand_card" | "add_hand_card" => Some(Self::AddRandomHandCard),
            "copy_joker_right" | "copy_right_joker" => Some(Self::CopyJokerRight),
            "copy_joker_leftmost" | "copy_leftmost_joker" => Some(Self::CopyJokerLeftmost),
            "prevent_death" | "survive" => Some(Self::PreventDeath),
            "add_tag" | "tag" => Some(Self::AddTag),
            "duplicate_next_tag" | "dup_next_tag" => Some(Self::DuplicateNextTag),
            "add_pack" | "add_booster_pack" => Some(Self::AddPack),
            "add_shop_joker" | "shop_joker" => Some(Self::AddShopJoker),
            "add_voucher" | "voucher_add" => Some(Self::AddVoucher),
            "set_reroll_cost" | "reroll_cost" => Some(Self::SetRerollCost),
            "set_shop_joker_edition" | "shop_joker_edition" => Some(Self::SetShopJokerEdition),
            "reroll_boss" | "boss_reroll" => Some(Self::RerollBoss),
            "upgrade_random_hand" | "upgrade_hand_random" => Some(Self::UpgradeRandomHand),
            "set_hands" | "hands_set" | "set_hands_left" => Some(Self::SetHands),
            "mul_target" | "multiply_target" | "target_mult" => Some(Self::MultiplyTarget),
            "set_rule" | "rule_set" => Some(Self::SetRule),
            "add_rule" | "rule_add" => Some(Self::AddRule),
            "clear_rule" | "rule_clear" => Some(Self::ClearRule),
            "set_var" => Some(Self::SetVar),
            "add_var" => Some(Self::AddVar),
            _ => None,
        }
    }

    pub fn requires_target(self) -> bool {
        matches!(
            self,
            Self::SetVar
                | Self::AddVar
                | Self::SetShopPrice
                | Self::AddJoker
                | Self::AddSellBonus
                | Self::AddTag
                | Self::DuplicateNextTag
                | Self::AddPack
                | Self::AddShopJoker
                | Self::SetShopJokerEdition
                | Self::SetRule
                | Self::AddRule
                | Self::ClearRule
                | Self::SetCardEnhancement
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
