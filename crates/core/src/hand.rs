use crate::{Card, Rank, Suit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandKind {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

impl HandKind {
    pub const ALL: [HandKind; 13] = [
        HandKind::HighCard,
        HandKind::Pair,
        HandKind::TwoPair,
        HandKind::Trips,
        HandKind::Straight,
        HandKind::Flush,
        HandKind::FullHouse,
        HandKind::Quads,
        HandKind::StraightFlush,
        HandKind::RoyalFlush,
        HandKind::FiveOfAKind,
        HandKind::FlushHouse,
        HandKind::FlushFive,
    ];

    pub fn id(self) -> &'static str {
        match self {
            HandKind::HighCard => "high_card",
            HandKind::Pair => "pair",
            HandKind::TwoPair => "two_pair",
            HandKind::Trips => "trips",
            HandKind::Straight => "straight",
            HandKind::Flush => "flush",
            HandKind::FullHouse => "full_house",
            HandKind::Quads => "quads",
            HandKind::StraightFlush => "straight_flush",
            HandKind::RoyalFlush => "royal_flush",
            HandKind::FiveOfAKind => "five_kind",
            HandKind::FlushHouse => "flush_house",
            HandKind::FlushFive => "flush_five",
        }
    }
}

pub fn level_kind(kind: HandKind) -> HandKind {
    match kind {
        HandKind::RoyalFlush => HandKind::StraightFlush,
        other => other,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HandEvalRules {
    pub smeared_suits: bool,
    pub four_fingers: bool,
    pub shortcut: bool,
}

pub fn evaluate_hand(cards: &[Card]) -> HandKind {
    evaluate_hand_with_rules(cards, HandEvalRules::default())
}

pub fn evaluate_hand_with_rules(cards: &[Card], rules: HandEvalRules) -> HandKind {
    if cards.is_empty() {
        return HandKind::HighCard;
    }

    // TODO: Handle wilds/jokers and advanced modifier rules.
    let eval_cards: Vec<Card> = cards.iter().copied().filter(|c| !c.is_stone()).collect();
    if eval_cards.is_empty() {
        return HandKind::HighCard;
    }

    let len = eval_cards.len();
    let mut rank_counts: HashMap<Rank, usize> = HashMap::new();
    let mut suit_counts: HashMap<u8, usize> = HashMap::new();
    for card in &eval_cards {
        *rank_counts.entry(card.rank).or_insert(0) += 1;
        *suit_counts
            .entry(suit_bucket(card.suit, rules.smeared_suits))
            .or_insert(0) += 1;
    }

    let mut counts: Vec<usize> = rank_counts.values().copied().collect();
    counts.sort_by(|a, b| b.cmp(a));

    let is_flush = (len == 5 && suit_counts.len() == 1)
        || (rules.four_fingers && len == 4 && suit_counts.len() == 1);
    let max_gap = if rules.shortcut { 2 } else { 1 };
    let is_straight = (len == 5 && is_straight_len(&eval_cards, 5, max_gap))
        || (rules.four_fingers && len == 4 && is_straight_len(&eval_cards, 4, max_gap));

    if len == 5 {
        if counts == [5] {
            return if is_flush {
                HandKind::FlushFive
            } else {
                HandKind::FiveOfAKind
            };
        }
        if counts == [4, 1] {
            return HandKind::Quads;
        }
        if counts == [3, 2] {
            return if is_flush {
                HandKind::FlushHouse
            } else {
                HandKind::FullHouse
            };
        }
        if is_flush && is_straight {
            return if is_royal(&eval_cards) {
                HandKind::RoyalFlush
            } else {
                HandKind::StraightFlush
            };
        }
        if is_flush {
            return HandKind::Flush;
        }
        if is_straight {
            return HandKind::Straight;
        }
        if counts == [3, 1, 1] {
            return HandKind::Trips;
        }
        if counts == [2, 2, 1] {
            return HandKind::TwoPair;
        }
        if counts == [2, 1, 1, 1] {
            return HandKind::Pair;
        }
        return HandKind::HighCard;
    }

    if rules.four_fingers && len == 4 {
        if is_flush && is_straight {
            return HandKind::StraightFlush;
        }
        if is_flush {
            return HandKind::Flush;
        }
        if is_straight {
            return HandKind::Straight;
        }
    }

    if counts == [4] {
        return HandKind::Quads;
    }
    if counts == [3] || counts == [3, 1] {
        return HandKind::Trips;
    }
    if counts == [2, 2] {
        return HandKind::TwoPair;
    }
    if counts == [2] || counts == [2, 1] || counts == [2, 1, 1] {
        return HandKind::Pair;
    }
    HandKind::HighCard
}

fn suit_bucket(suit: Suit, smeared: bool) -> u8 {
    if smeared {
        match suit {
            Suit::Spades | Suit::Clubs => 0,
            Suit::Hearts | Suit::Diamonds => 1,
            Suit::Wild => 2,
        }
    } else {
        match suit {
            Suit::Spades => 0,
            Suit::Hearts => 1,
            Suit::Clubs => 2,
            Suit::Diamonds => 3,
            Suit::Wild => 4,
        }
    }
}

pub fn scoring_cards(cards: &[Card], kind: HandKind) -> Vec<usize> {
    if cards.is_empty() {
        return Vec::new();
    }

    let mut rank_counts: HashMap<Rank, usize> = HashMap::new();
    for (_idx, card) in cards.iter().enumerate() {
        if card.is_stone() {
            continue;
        }
        *rank_counts.entry(card.rank).or_insert(0) += 1;
    }

    let mut scoring: Vec<usize> = Vec::new();
    let stone_indices: Vec<usize> = cards
        .iter()
        .enumerate()
        .filter(|(_, card)| card.is_stone())
        .map(|(idx, _)| idx)
        .collect();

    match kind {
        HandKind::HighCard => {
            if let Some(idx) = highest_card_index(cards) {
                scoring.push(idx);
            }
        }
        HandKind::Pair => scoring.extend(pick_indices_by_count(cards, &rank_counts, 2, 1)),
        HandKind::TwoPair => scoring.extend(pick_indices_by_count(cards, &rank_counts, 2, 2)),
        HandKind::Trips => scoring.extend(pick_indices_by_count(cards, &rank_counts, 3, 1)),
        HandKind::Quads => scoring.extend(pick_indices_by_count(cards, &rank_counts, 4, 1)),
        HandKind::FullHouse
        | HandKind::Straight
        | HandKind::Flush
        | HandKind::StraightFlush
        | HandKind::RoyalFlush
        | HandKind::FiveOfAKind
        | HandKind::FlushHouse
        | HandKind::FlushFive => {
            scoring.extend((0..cards.len()).filter(|idx| !cards[*idx].is_stone()));
        }
    }

    scoring.extend(stone_indices);
    scoring.sort_unstable();
    scoring.dedup();
    scoring
}
fn is_straight_len(cards: &[Card], required: usize, max_gap: u8) -> bool {
    let mut values: Vec<u8> = cards.iter().map(|card| rank_value(card.rank)).collect();
    values.sort_unstable();
    values.dedup();
    if values.len() != required {
        return false;
    }
    if required == 5 && values == [2, 3, 4, 5, 14] {
        return true;
    }
    if required == 4 && values == [2, 3, 4, 14] {
        return true;
    }
    values
        .windows(2)
        .all(|w| w[1].saturating_sub(w[0]) <= max_gap)
}

fn is_royal(cards: &[Card]) -> bool {
    let mut values: Vec<u8> = cards.iter().map(|card| rank_value(card.rank)).collect();
    values.sort_unstable();
    values == [10, 11, 12, 13, 14]
}

fn rank_value(rank: Rank) -> u8 {
    match rank {
        Rank::Ace => 14,
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Joker => 0,
    }
}

fn highest_card_index(cards: &[Card]) -> Option<usize> {
    let mut best: Option<(usize, u8)> = None;
    for (idx, card) in cards.iter().enumerate() {
        if card.is_stone() {
            continue;
        }
        let value = rank_value(card.rank);
        if best.map(|(_, v)| value > v).unwrap_or(true) {
            best = Some((idx, value));
        }
    }
    best.map(|(idx, _)| idx)
}

fn pick_indices_by_count(
    cards: &[Card],
    rank_counts: &HashMap<Rank, usize>,
    count: usize,
    max_groups: usize,
) -> Vec<usize> {
    let mut ranks: Vec<Rank> = rank_counts
        .iter()
        .filter(|(_, &c)| c == count)
        .map(|(r, _)| *r)
        .collect();
    ranks.sort_by(|a, b| rank_value(*b).cmp(&rank_value(*a)));
    ranks.truncate(max_groups);

    let mut picked = Vec::new();
    for (idx, card) in cards.iter().enumerate() {
        if card.is_stone() {
            continue;
        }
        if ranks.contains(&card.rank) {
            picked.push(idx);
        }
    }
    picked
}
