use crate::{BlindKind, Card, ConsumableKind, Edition, Enhancement, HandKind, Rank, Seal, Suit};

pub(super) fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(super) fn hand_name(kind: HandKind) -> &'static str {
    match kind {
        HandKind::HighCard => "HighCard",
        HandKind::Pair => "Pair",
        HandKind::TwoPair => "TwoPair",
        HandKind::Trips => "Trips",
        HandKind::Straight => "Straight",
        HandKind::Flush => "Flush",
        HandKind::FullHouse => "FullHouse",
        HandKind::Quads => "Quads",
        HandKind::StraightFlush => "StraightFlush",
        HandKind::RoyalFlush => "RoyalFlush",
        HandKind::FiveOfAKind => "FiveOfAKind",
        HandKind::FlushHouse => "FlushHouse",
        HandKind::FlushFive => "FlushFive",
    }
}

pub(super) fn blind_name(kind: BlindKind) -> &'static str {
    match kind {
        BlindKind::Small => "Small",
        BlindKind::Big => "Big",
        BlindKind::Boss => "Boss",
    }
}

pub(super) fn rank_name(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "Ace",
        Rank::Two => "Two",
        Rank::Three => "Three",
        Rank::Four => "Four",
        Rank::Five => "Five",
        Rank::Six => "Six",
        Rank::Seven => "Seven",
        Rank::Eight => "Eight",
        Rank::Nine => "Nine",
        Rank::Ten => "Ten",
        Rank::Jack => "Jack",
        Rank::Queen => "Queen",
        Rank::King => "King",
        Rank::Joker => "Joker",
    }
}

pub(super) fn suit_name(suit: Suit) -> &'static str {
    match suit {
        Suit::Spades => "Spades",
        Suit::Hearts => "Hearts",
        Suit::Clubs => "Clubs",
        Suit::Diamonds => "Diamonds",
        Suit::Wild => "Wild",
    }
}

pub(super) fn suit_index(suit: Suit) -> u8 {
    match suit {
        Suit::Spades => 0,
        Suit::Hearts => 1,
        Suit::Clubs => 2,
        Suit::Diamonds => 3,
        Suit::Wild => 4,
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

pub(super) fn consumable_kind_name(kind: ConsumableKind) -> &'static str {
    match kind {
        ConsumableKind::Tarot => "Tarot",
        ConsumableKind::Planet => "Planet",
        ConsumableKind::Spectral => "Spectral",
    }
}

pub(super) fn is_face(card: Card) -> bool {
    matches!(card.rank, Rank::Jack | Rank::Queen | Rank::King)
}

pub(super) fn is_odd(card: Card) -> bool {
    matches!(
        card.rank,
        Rank::Ace | Rank::Three | Rank::Five | Rank::Seven | Rank::Nine
    )
}

pub(super) fn is_even(card: Card) -> bool {
    matches!(
        card.rank,
        Rank::Two | Rank::Four | Rank::Six | Rank::Eight | Rank::Ten
    )
}

pub(super) fn suit_from_str(value: &str) -> Option<Suit> {
    match normalize(value).as_str() {
        "spades" | "spade" => Some(Suit::Spades),
        "hearts" | "heart" => Some(Suit::Hearts),
        "clubs" | "club" => Some(Suit::Clubs),
        "diamonds" | "diamond" => Some(Suit::Diamonds),
        "wild" => Some(Suit::Wild),
        _ => None,
    }
}

pub(super) fn smeared_suit_group(suit: Suit) -> u8 {
    match suit {
        Suit::Spades | Suit::Clubs => 0,
        Suit::Hearts | Suit::Diamonds => 1,
        Suit::Wild => 2,
    }
}

pub(super) fn rank_from_str(value: &str) -> Option<Rank> {
    match normalize(value).as_str() {
        "ace" | "a" => Some(Rank::Ace),
        "two" | "2" => Some(Rank::Two),
        "three" | "3" => Some(Rank::Three),
        "four" | "4" => Some(Rank::Four),
        "five" | "5" => Some(Rank::Five),
        "six" | "6" => Some(Rank::Six),
        "seven" | "7" => Some(Rank::Seven),
        "eight" | "8" => Some(Rank::Eight),
        "nine" | "9" => Some(Rank::Nine),
        "ten" | "10" => Some(Rank::Ten),
        "jack" | "j" => Some(Rank::Jack),
        "queen" | "q" => Some(Rank::Queen),
        "king" | "k" => Some(Rank::King),
        "joker" => Some(Rank::Joker),
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
    card.is_wild() || matches!(card.suit, Suit::Spades | Suit::Clubs)
}

pub(super) fn is_red(card: Card) -> bool {
    card.is_wild() || matches!(card.suit, Suit::Hearts | Suit::Diamonds)
}

pub(super) fn hand_kind_from_str(value: &str) -> Option<HandKind> {
    match normalize(value).as_str() {
        "highcard" | "high_card" => Some(HandKind::HighCard),
        "pair" => Some(HandKind::Pair),
        "twopair" | "two_pair" => Some(HandKind::TwoPair),
        "trips" | "threeofakind" | "three_kind" => Some(HandKind::Trips),
        "straight" => Some(HandKind::Straight),
        "flush" => Some(HandKind::Flush),
        "fullhouse" | "full_house" => Some(HandKind::FullHouse),
        "quads" | "four_kind" | "fourkind" => Some(HandKind::Quads),
        "straightflush" | "straight_flush" => Some(HandKind::StraightFlush),
        "royalflush" | "royal_flush" => Some(HandKind::RoyalFlush),
        "fiveofakind" | "five_kind" | "fivekind" => Some(HandKind::FiveOfAKind),
        "flushhouse" | "flush_house" => Some(HandKind::FlushHouse),
        "flushfive" | "flush_five" => Some(HandKind::FlushFive),
        _ => None,
    }
}

pub(super) fn hand_contains_kind(hand: HandKind, target: HandKind) -> bool {
    use HandKind::*;
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
