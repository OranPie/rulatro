use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
    Wild,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Joker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Enhancement {
    Bonus,
    Mult,
    Wild,
    Glass,
    Steel,
    Stone,
    Lucky,
    Gold,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Edition {
    Foil,
    Holographic,
    Polychrome,
    Negative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Seal {
    Red,
    Blue,
    Gold,
    Purple,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
    #[serde(default)]
    pub enhancement: Option<Enhancement>,
    #[serde(default)]
    pub edition: Option<Edition>,
    #[serde(default)]
    pub seal: Option<Seal>,
    #[serde(default)]
    pub bonus_chips: i64,
}

impl Card {
    pub fn standard(suit: Suit, rank: Rank) -> Self {
        Self {
            suit,
            rank,
            enhancement: None,
            edition: None,
            seal: None,
            bonus_chips: 0,
        }
    }

    pub fn is_wild(&self) -> bool {
        matches!(self.enhancement, Some(Enhancement::Wild)) || self.suit == Suit::Wild
    }

    pub fn is_stone(&self) -> bool {
        matches!(self.enhancement, Some(Enhancement::Stone))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CardKind {
    Standard(Card),
    Joker(Card),
    Consumable(String),
}
