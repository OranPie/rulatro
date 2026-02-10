use super::{EvalContext, EvalValue};
use crate::{BlindKind, Card, Edition, Enhancement, Seal};
use std::collections::HashMap;

pub(super) fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(super) fn build_joker_counts(jokers: &[crate::JokerInstance]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for joker in jokers {
        let key = normalize(&joker.id);
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

pub(super) fn values_equal(left: &EvalValue, right: &EvalValue, smeared: bool) -> bool {
    match (left, right) {
        (EvalValue::Bool(a), EvalValue::Bool(b)) => a == b,
        (EvalValue::Num(a), EvalValue::Num(b)) => a == b,
        (EvalValue::Str(a), EvalValue::Str(b)) => {
            if smeared {
                if let (Some(left_suit), Some(right_suit)) =
                    (suit_from_str(a), suit_from_str(b))
                {
                    if matches!(left_suit, crate::Suit::Wild)
                        || matches!(right_suit, crate::Suit::Wild)
                    {
                        return true;
                    }
                    return smeared_suit_group(left_suit) == smeared_suit_group(right_suit);
                }
            } else if let (Some(left_suit), Some(right_suit)) = (suit_from_str(a), suit_from_str(b)) {
                if matches!(left_suit, crate::Suit::Wild)
                    || matches!(right_suit, crate::Suit::Wild)
                {
                    return true;
                }
                return left_suit == right_suit;
            }
            a == b
        }
        _ => match (left.as_number(), right.as_number()) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
    }
}

pub(super) fn compare_numbers<F>(left: &EvalValue, right: &EvalValue, cmp: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    match (left.as_number(), right.as_number()) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

pub(super) fn combine_numbers<F>(left: &EvalValue, right: &EvalValue, op: F) -> EvalValue
where
    F: Fn(f64, f64) -> f64,
{
    match (left.as_number(), right.as_number()) {
        (Some(a), Some(b)) => EvalValue::Num(op(a, b)),
        _ => EvalValue::None,
    }
}

pub(super) fn hand_name(kind: crate::HandKind) -> &'static str {
    match kind {
        crate::HandKind::HighCard => "HighCard",
        crate::HandKind::Pair => "Pair",
        crate::HandKind::TwoPair => "TwoPair",
        crate::HandKind::Trips => "Trips",
        crate::HandKind::Straight => "Straight",
        crate::HandKind::Flush => "Flush",
        crate::HandKind::FullHouse => "FullHouse",
        crate::HandKind::Quads => "Quads",
        crate::HandKind::StraightFlush => "StraightFlush",
        crate::HandKind::RoyalFlush => "RoyalFlush",
        crate::HandKind::FiveOfAKind => "FiveOfAKind",
        crate::HandKind::FlushHouse => "FlushHouse",
        crate::HandKind::FlushFive => "FlushFive",
    }
}

pub(super) fn blind_name(kind: BlindKind) -> &'static str {
    match kind {
        BlindKind::Small => "Small",
        BlindKind::Big => "Big",
        BlindKind::Boss => "Boss",
    }
}

pub(super) fn rank_name(rank: crate::Rank) -> &'static str {
    match rank {
        crate::Rank::Ace => "Ace",
        crate::Rank::Two => "Two",
        crate::Rank::Three => "Three",
        crate::Rank::Four => "Four",
        crate::Rank::Five => "Five",
        crate::Rank::Six => "Six",
        crate::Rank::Seven => "Seven",
        crate::Rank::Eight => "Eight",
        crate::Rank::Nine => "Nine",
        crate::Rank::Ten => "Ten",
        crate::Rank::Jack => "Jack",
        crate::Rank::Queen => "Queen",
        crate::Rank::King => "King",
        crate::Rank::Joker => "Joker",
    }
}

pub(super) fn suit_name(suit: crate::Suit) -> &'static str {
    match suit {
        crate::Suit::Spades => "Spades",
        crate::Suit::Hearts => "Hearts",
        crate::Suit::Clubs => "Clubs",
        crate::Suit::Diamonds => "Diamonds",
        crate::Suit::Wild => "Wild",
    }
}

pub(super) fn suit_index(suit: crate::Suit) -> u8 {
    match suit {
        crate::Suit::Spades => 0,
        crate::Suit::Hearts => 1,
        crate::Suit::Clubs => 2,
        crate::Suit::Diamonds => 3,
        crate::Suit::Wild => 4,
    }
}

pub(super) fn enhancement_name(kind: Enhancement) -> &'static str {
    match kind {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Lucky => "Lucky",
        Enhancement::Gold => "Gold",
    }
}

pub(super) fn edition_name(kind: Edition) -> &'static str {
    match kind {
        Edition::Foil => "Foil",
        Edition::Holographic => "Holographic",
        Edition::Polychrome => "Polychrome",
        Edition::Negative => "Negative",
    }
}

pub(super) fn seal_name(kind: Seal) -> &'static str {
    match kind {
        Seal::Red => "Red",
        Seal::Blue => "Blue",
        Seal::Gold => "Gold",
        Seal::Purple => "Purple",
    }
}

pub(super) fn consumable_kind_name(kind: crate::ConsumableKind) -> &'static str {
    match kind {
        crate::ConsumableKind::Tarot => "Tarot",
        crate::ConsumableKind::Planet => "Planet",
        crate::ConsumableKind::Spectral => "Spectral",
    }
}

pub(super) fn is_face(card: Card) -> bool {
    matches!(card.rank, crate::Rank::Jack | crate::Rank::Queen | crate::Rank::King)
}

pub(super) fn is_odd(card: Card) -> bool {
    matches!(
        card.rank,
        crate::Rank::Ace
            | crate::Rank::Three
            | crate::Rank::Five
            | crate::Rank::Seven
            | crate::Rank::Nine
    )
}

pub(super) fn is_even(card: Card) -> bool {
    matches!(
        card.rank,
        crate::Rank::Two
            | crate::Rank::Four
            | crate::Rank::Six
            | crate::Rank::Eight
            | crate::Rank::Ten
    )
}

pub(super) fn hand_kind_from_str(value: &str) -> Option<crate::HandKind> {
    match normalize(value).as_str() {
        "highcard" | "high_card" => Some(crate::HandKind::HighCard),
        "pair" => Some(crate::HandKind::Pair),
        "twopair" | "two_pair" => Some(crate::HandKind::TwoPair),
        "trips" | "threeofakind" | "three_kind" => Some(crate::HandKind::Trips),
        "straight" => Some(crate::HandKind::Straight),
        "flush" => Some(crate::HandKind::Flush),
        "fullhouse" | "full_house" => Some(crate::HandKind::FullHouse),
        "quads" | "four_kind" | "fourkind" => Some(crate::HandKind::Quads),
        "straightflush" | "straight_flush" => Some(crate::HandKind::StraightFlush),
        "royalflush" | "royal_flush" => Some(crate::HandKind::RoyalFlush),
        "fiveofakind" | "five_kind" | "fivekind" => Some(crate::HandKind::FiveOfAKind),
        "flushhouse" | "flush_house" => Some(crate::HandKind::FlushHouse),
        "flushfive" | "flush_five" => Some(crate::HandKind::FlushFive),
        _ => None,
    }
}

pub(super) fn scope_cards<'a>(ctx: &'a EvalContext<'a>, scope: &str) -> &'a [Card] {
    match normalize(scope).as_str() {
        "played" => ctx.played_cards,
        "scoring" => ctx.scoring_cards,
        "held" => ctx.held_cards,
        "discarded" => ctx.discarded_cards,
        _ => &[],
    }
}

pub(super) fn count_matching(cards: &[Card], target: &str, smeared: bool) -> usize {
    let target_norm = normalize(target);
    match target_norm.as_str() {
        "any" | "all" => cards.len(),
        "face" => cards.iter().filter(|card| !card.is_stone() && is_face(**card)).count(),
        "odd" => cards.iter().filter(|card| !card.is_stone() && is_odd(**card)).count(),
        "even" => cards.iter().filter(|card| !card.is_stone() && is_even(**card)).count(),
        "wild" => cards.iter().filter(|card| card.is_wild()).count(),
        "stone" => cards.iter().filter(|card| card.is_stone()).count(),
        "enhanced" => cards.iter().filter(|card| card.enhancement.is_some()).count(),
        "black" => cards
            .iter()
            .filter(|card| !card.is_stone() && is_black(**card))
            .count(),
        "red" => cards
            .iter()
            .filter(|card| !card.is_stone() && is_red(**card))
            .count(),
        _ => {
            if let Some(suit) = suit_from_str(&target_norm) {
                if smeared {
                    let target_group = smeared_suit_group(suit);
                    return cards
                        .iter()
                        .filter(|card| {
                            !card.is_stone()
                                && (card.is_wild()
                                    || smeared_suit_group(card.suit) == target_group)
                        })
                        .count();
                }
                return cards
                    .iter()
                    .filter(|card| !card.is_stone() && (card.is_wild() || card.suit == suit))
                    .count();
            }
            if let Some(rank) = rank_from_str(&target_norm) {
                return cards
                    .iter()
                    .filter(|card| !card.is_stone() && card.rank == rank)
                    .count();
            }
            if let Some(kind) = enhancement_from_str(&target_norm) {
                return cards
                    .iter()
                    .filter(|card| card.enhancement == Some(kind))
                    .count();
            }
            if let Some(kind) = edition_from_str(&target_norm) {
                return cards.iter().filter(|card| card.edition == Some(kind)).count();
            }
            if let Some(kind) = seal_from_str(&target_norm) {
                return cards.iter().filter(|card| card.seal == Some(kind)).count();
            }
            0
        }
    }
}

pub(super) fn suit_from_str(value: &str) -> Option<crate::Suit> {
    match normalize(value).as_str() {
        "spades" | "spade" => Some(crate::Suit::Spades),
        "hearts" | "heart" => Some(crate::Suit::Hearts),
        "clubs" | "club" => Some(crate::Suit::Clubs),
        "diamonds" | "diamond" => Some(crate::Suit::Diamonds),
        "wild" => Some(crate::Suit::Wild),
        _ => None,
    }
}

pub(super) fn smeared_suit_group(suit: crate::Suit) -> u8 {
    match suit {
        crate::Suit::Spades | crate::Suit::Clubs => 0,
        crate::Suit::Hearts | crate::Suit::Diamonds => 1,
        crate::Suit::Wild => 2,
    }
}

pub(super) fn rank_from_str(value: &str) -> Option<crate::Rank> {
    match normalize(value).as_str() {
        "ace" | "a" => Some(crate::Rank::Ace),
        "two" | "2" => Some(crate::Rank::Two),
        "three" | "3" => Some(crate::Rank::Three),
        "four" | "4" => Some(crate::Rank::Four),
        "five" | "5" => Some(crate::Rank::Five),
        "six" | "6" => Some(crate::Rank::Six),
        "seven" | "7" => Some(crate::Rank::Seven),
        "eight" | "8" => Some(crate::Rank::Eight),
        "nine" | "9" => Some(crate::Rank::Nine),
        "ten" | "10" => Some(crate::Rank::Ten),
        "jack" | "j" => Some(crate::Rank::Jack),
        "queen" | "q" => Some(crate::Rank::Queen),
        "king" | "k" => Some(crate::Rank::King),
        "joker" => Some(crate::Rank::Joker),
        _ => None,
    }
}

pub(super) fn enhancement_from_str(value: &str) -> Option<Enhancement> {
    match normalize(value).as_str() {
        "bonus" => Some(Enhancement::Bonus),
        "mult" => Some(Enhancement::Mult),
        "wild" => Some(Enhancement::Wild),
        "glass" => Some(Enhancement::Glass),
        "steel" => Some(Enhancement::Steel),
        "stone" => Some(Enhancement::Stone),
        "lucky" => Some(Enhancement::Lucky),
        "gold" => Some(Enhancement::Gold),
        _ => None,
    }
}

pub(super) fn edition_from_str(value: &str) -> Option<Edition> {
    match normalize(value).as_str() {
        "foil" => Some(Edition::Foil),
        "holographic" => Some(Edition::Holographic),
        "polychrome" => Some(Edition::Polychrome),
        "negative" => Some(Edition::Negative),
        _ => None,
    }
}

pub(super) fn seal_from_str(value: &str) -> Option<Seal> {
    match normalize(value).as_str() {
        "red" => Some(Seal::Red),
        "blue" => Some(Seal::Blue),
        "gold" => Some(Seal::Gold),
        "purple" => Some(Seal::Purple),
        _ => None,
    }
}

pub(super) fn is_black(card: Card) -> bool {
    card.is_wild() || matches!(card.suit, crate::Suit::Spades | crate::Suit::Clubs)
}

pub(super) fn is_red(card: Card) -> bool {
    card.is_wild() || matches!(card.suit, crate::Suit::Hearts | crate::Suit::Diamonds)
}

pub(super) fn hand_contains_kind(hand: crate::HandKind, target: crate::HandKind) -> bool {
    use crate::HandKind::*;
    if hand == target {
        return true;
    }
    match target {
        HighCard => true,
        Pair => matches!(
            hand,
            Pair | TwoPair | Trips | FullHouse | Quads | FiveOfAKind | FlushHouse | FlushFive
        ),
        TwoPair => matches!(hand, TwoPair),
        Trips => matches!(
            hand,
            Trips | FullHouse | Quads | FiveOfAKind | FlushHouse | FlushFive
        ),
        Straight => matches!(hand, Straight | StraightFlush | RoyalFlush),
        Flush => matches!(hand, Flush | StraightFlush | RoyalFlush | FlushHouse | FlushFive),
        FullHouse => matches!(hand, FullHouse | FlushHouse),
        Quads => matches!(hand, Quads | FiveOfAKind | FlushFive),
        StraightFlush => matches!(hand, StraightFlush | RoyalFlush),
        RoyalFlush => matches!(hand, RoyalFlush),
        FiveOfAKind => matches!(hand, FiveOfAKind | FlushFive),
        FlushHouse => matches!(hand, FlushHouse),
        FlushFive => matches!(hand, FlushFive),
    }
}
