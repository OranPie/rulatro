use crate::{BlindKind, Card};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(super) struct EvalContext<'a> {
    pub(super) hand_kind: crate::HandKind,
    pub(super) blind: BlindKind,
    pub(super) card: Option<Card>,
    pub(super) consumable_kind: Option<crate::ConsumableKind>,
    pub(super) consumable_id: Option<&'a str>,
    pub(super) sold_value: Option<i64>,
    pub(super) is_scoring: bool,
    pub(super) is_held: bool,
    pub(super) is_played: bool,
    pub(super) played_count: usize,
    pub(super) scoring_count: usize,
    pub(super) hands_left: u8,
    pub(super) discards_left: u8,
    pub(super) joker_count: usize,
    pub(super) played_cards: &'a [Card],
    pub(super) scoring_cards: &'a [Card],
    pub(super) held_cards: &'a [Card],
    pub(super) discarded_cards: &'a [Card],
    pub(super) joker_vars: Option<HashMap<String, f64>>,
    pub(super) joker_index: Option<usize>,
}

impl<'a> EvalContext<'a> {
    pub(super) fn played(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        played_cards: &'a [Card],
        scoring_cards: &'a [Card],
        held_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: None,
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: false,
            is_held: false,
            is_played: true,
            played_count: played_cards.len(),
            scoring_count: scoring_cards.len(),
            hands_left,
            discards_left,
            joker_count,
            played_cards,
            scoring_cards,
            held_cards,
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn independent(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        played_cards: &'a [Card],
        scoring_cards: &'a [Card],
        held_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: None,
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: false,
            is_held: false,
            is_played: false,
            played_count: played_cards.len(),
            scoring_count: scoring_cards.len(),
            hands_left,
            discards_left,
            joker_count,
            played_cards,
            scoring_cards,
            held_cards,
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn scoring(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        played_cards: &'a [Card],
        scoring_cards: &'a [Card],
        held_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: Some(card),
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: true,
            is_held: false,
            is_played: true,
            played_count: played_cards.len(),
            scoring_count: scoring_cards.len(),
            hands_left,
            discards_left,
            joker_count,
            played_cards,
            scoring_cards,
            held_cards,
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn held(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        played_cards: &'a [Card],
        scoring_cards: &'a [Card],
        held_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: Some(card),
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: false,
            is_held: true,
            is_played: false,
            played_count: played_cards.len(),
            scoring_count: scoring_cards.len(),
            hands_left,
            discards_left,
            joker_count,
            played_cards,
            scoring_cards,
            held_cards,
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn discard(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        held_cards: &'a [Card],
        discarded_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: Some(card),
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: false,
            is_held: false,
            is_played: false,
            played_count: 0,
            scoring_count: 0,
            hands_left,
            discards_left,
            joker_count,
            played_cards: &[],
            scoring_cards: &[],
            held_cards,
            discarded_cards,
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn discard_batch(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        held_cards: &'a [Card],
        discarded_cards: &'a [Card],
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: None,
            consumable_kind: None,
            consumable_id: None,
            sold_value: None,
            is_scoring: false,
            is_held: false,
            is_played: false,
            played_count: 0,
            scoring_count: 0,
            hands_left,
            discards_left,
            joker_count,
            played_cards: &[],
            scoring_cards: &[],
            held_cards,
            discarded_cards,
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn sell(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        sold_value: i64,
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: None,
            consumable_kind: None,
            consumable_id: None,
            sold_value: Some(sold_value),
            is_scoring: false,
            is_held: false,
            is_played: false,
            played_count: 0,
            scoring_count: 0,
            hands_left,
            discards_left,
            joker_count,
            played_cards: &[],
            scoring_cards: &[],
            held_cards: &[],
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }

    pub(super) fn with_joker_vars(&self, vars: &HashMap<String, f64>) -> Self {
        let mut next = self.clone();
        next.joker_vars = Some(vars.clone());
        next
    }

    pub(super) fn with_joker_index(&self, index: usize) -> Self {
        let mut next = self.clone();
        next.joker_index = Some(index);
        next
    }

    pub(super) fn consumable(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        consumable_kind: crate::ConsumableKind,
        consumable_id: &'a str,
        hands_left: u8,
        discards_left: u8,
        joker_count: usize,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            card: None,
            consumable_kind: Some(consumable_kind),
            consumable_id: Some(consumable_id),
            sold_value: None,
            is_scoring: false,
            is_held: false,
            is_played: false,
            played_count: 0,
            scoring_count: 0,
            hands_left,
            discards_left,
            joker_count,
            played_cards: &[],
            scoring_cards: &[],
            held_cards: &[],
            discarded_cards: &[],
            joker_vars: None,
            joker_index: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum EvalValue {
    Bool(bool),
    Num(f64),
    Str(String),
    None,
}

impl EvalValue {
    pub(super) fn truthy(&self) -> bool {
        match self {
            EvalValue::Bool(value) => *value,
            EvalValue::Num(value) => *value != 0.0,
            EvalValue::Str(value) => !value.is_empty(),
            EvalValue::None => false,
        }
    }

    pub(super) fn as_number(&self) -> Option<f64> {
        match self {
            EvalValue::Num(value) => Some(*value),
            EvalValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
            EvalValue::Str(value) => value.parse::<f64>().ok(),
            EvalValue::None => None,
        }
    }

    pub(super) fn as_string(&self) -> Option<&str> {
        match self {
            EvalValue::Str(value) => Some(value.as_str()),
            _ => None,
        }
    }
}
