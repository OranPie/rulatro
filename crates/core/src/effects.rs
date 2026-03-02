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
    DoubleMoney {
        cap: i64,
    },
    AddMoneyFromJokers {
        cap: i64,
    },
    AddHandSize(i64),
    UpgradeHand {
        hand: HandKind,
        amount: u32,
    },
    UpgradeAllHands {
        amount: u32,
    },
    AddRandomConsumable {
        kind: ConsumableKind,
        count: u8,
    },
    AddJoker {
        rarity: JokerRarity,
        count: u8,
    },
    AddRandomJoker {
        count: u8,
    },
    RandomJokerEdition {
        editions: Vec<Edition>,
        chance: f64,
    },
    SetRandomJokerEdition {
        edition: Edition,
    },
    SetRandomJokerEditionDestroyOthers {
        edition: Edition,
    },
    DuplicateRandomJokerDestroyOthers {
        remove_negative: bool,
    },
    EnhanceSelected {
        enhancement: Enhancement,
        count: u8,
    },
    AddEditionToSelected {
        editions: Vec<Edition>,
        count: u8,
    },
    AddSealToSelected {
        seal: Seal,
        count: u8,
    },
    ConvertSelectedSuit {
        suit: Suit,
        count: u8,
    },
    IncreaseSelectedRank {
        count: u8,
        delta: i8,
    },
    DestroySelected {
        count: u8,
    },
    DestroyRandomInHand {
        count: u8,
    },
    CopySelected {
        count: u8,
    },
    ConvertLeftIntoRight,
    ConvertHandToRandomRank,
    ConvertHandToRandomSuit,
    AddRandomEnhancedCards {
        count: u8,
        filter: RankFilter,
    },
    CreateLastConsumable {
        exclude: Option<String>,
    },
    RetriggerScored(i64),
    RetriggerHeld(i64),
    /// A mod-registered consumable effect. `name` is the effect identifier;
    /// `value` is an optional numeric parameter (defaults to 0.0).
    Custom {
        name: String,
        #[serde(default)]
        value: f64,
    },
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

/// The operation in a DSL action: either a built-in op or a mod-registered custom keyword.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionOpKind {
    Builtin(ActionOp),
    /// Keyword registered by a mod via `rulatro.register_effect`.
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Action {
    pub op: ActionOpKind,
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
    /// Dynamic lookup resolved at evaluation time from `CardAttrRules`.
    /// The key uses dot-notation, e.g. `"enhancement.chips"` or `"edition.x_mult"`.
    Lookup(String),
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

pub fn format_joker_effect_compact(effect: &JokerEffect) -> String {
    let mut out = format!("on {}", activation_short(effect.trigger));
    if !matches!(effect.when, Expr::Bool(true)) {
        out.push_str(" when ");
        out.push_str(&format_expr_compact(&effect.when));
    }
    if effect.actions.is_empty() {
        return out;
    }
    out.push_str(" { ");
    let body = effect
        .actions
        .iter()
        .map(format_action_compact)
        .collect::<Vec<_>>()
        .join("; ");
    out.push_str(&body);
    out.push_str(" }");
    out
}

pub fn format_action_compact(action: &Action) -> String {
    let op = match &action.op {
        ActionOpKind::Builtin(op) => action_op_name(*op),
        ActionOpKind::Custom(name) => name.clone(),
    };
    let value = format_expr_compact(&action.value);
    match action.target.as_deref() {
        Some(target) => format!("{op} {target} {value}"),
        None => format!("{op} {value}"),
    }
}

pub fn format_expr_compact(expr: &Expr) -> String {
    match expr {
        Expr::Bool(value) => value.to_string(),
        Expr::Number(value) => format_number(*value),
        Expr::String(value) => format!("\"{value}\""),
        Expr::Ident(value) => value.clone(),
        Expr::Lookup(key) => format!("lookup({key})"),
        Expr::Call { name, args } => {
            let inner = args
                .iter()
                .map(format_expr_compact)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}({inner})")
        }
        Expr::Unary { op, expr } => {
            let symbol = match op {
                UnaryOp::Not => "!",
                UnaryOp::Neg => "-",
            };
            format!("{symbol}{}", format_expr_child(expr))
        }
        Expr::Binary { left, op, right } => {
            let symbol = match op {
                BinaryOp::Or => "||",
                BinaryOp::And => "&&",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
            };
            format!(
                "{} {symbol} {}",
                format_expr_child(left),
                format_expr_child(right)
            )
        }
    }
}

fn activation_short(trigger: ActivationType) -> &'static str {
    match trigger {
        ActivationType::OnPlayed => "played",
        ActivationType::OnScoredPre => "scored_pre",
        ActivationType::OnScored => "scored",
        ActivationType::OnHeld => "held",
        ActivationType::Independent => "independent",
        ActivationType::OnOtherJokers => "other_jokers",
        ActivationType::OnDiscard => "discard",
        ActivationType::OnDiscardBatch => "discard_batch",
        ActivationType::OnCardDestroyed => "card_destroyed",
        ActivationType::OnCardAdded => "card_added",
        ActivationType::OnRoundEnd => "round_end",
        ActivationType::OnHandEnd => "hand_end",
        ActivationType::OnBlindStart => "blind_start",
        ActivationType::OnBlindFailed => "blind_failed",
        ActivationType::OnShopEnter => "shop_enter",
        ActivationType::OnShopReroll => "shop_reroll",
        ActivationType::OnShopExit => "shop_exit",
        ActivationType::OnPackOpened => "pack_opened",
        ActivationType::OnPackSkipped => "pack_skipped",
        ActivationType::OnUse => "use",
        ActivationType::OnSell => "sell",
        ActivationType::OnAnySell => "any_sell",
        ActivationType::OnAcquire => "acquire",
        ActivationType::Passive => "passive",
    }
}

fn action_op_name(op: ActionOp) -> String {
    camel_to_snake(&format!("{op:?}"))
}

fn camel_to_snake(value: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in value.chars().enumerate() {
        if ch.is_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            for item in ch.to_lowercase() {
                out.push(item);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn format_expr_child(expr: &Expr) -> String {
    if matches!(expr, Expr::Binary { .. }) {
        format!("({})", format_expr_compact(expr))
    } else {
        format_expr_compact(expr)
    }
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        let text = format!("{value:.4}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_effect_compact() {
        let effect = JokerEffect {
            trigger: ActivationType::OnBlindStart,
            when: Expr::Bool(true),
            actions: vec![Action {
                op: ActionOpKind::Builtin(ActionOp::MultiplyTarget),
                target: None,
                value: Expr::Number(2.0),
            }],
        };
        assert_eq!(
            format_joker_effect_compact(&effect),
            "on blind_start { multiply_target 2 }"
        );
    }
}
