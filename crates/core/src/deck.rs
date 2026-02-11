use crate::{Card, Rank, RngState, Suit};

#[derive(Debug, Default, Clone)]
pub struct Deck {
    pub draw: Vec<Card>,
    pub discard: Vec<Card>,
}

impl Deck {
    pub fn standard52() -> Self {
        let mut draw = Vec::with_capacity(52);
        for suit in [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {
            for rank in [
                Rank::Ace,
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
            ] {
                draw.push(Card::standard(suit, rank));
            }
        }
        Self {
            draw,
            discard: Vec::new(),
        }
    }

    pub fn shuffle(&mut self, rng: &mut RngState) {
        rng.shuffle(&mut self.draw);
    }

    pub fn draw_cards(&mut self, count: usize) -> Vec<Card> {
        let mut cards = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(card) = self.draw.pop() {
                cards.push(card);
            } else {
                break;
            }
        }
        cards
    }

    pub fn discard(&mut self, mut cards: Vec<Card>) {
        for card in &mut cards {
            card.face_down = false;
        }
        self.discard.append(&mut cards);
    }

    pub fn reshuffle_discard(&mut self, rng: &mut RngState) {
        if self.discard.is_empty() {
            return;
        }
        self.draw.append(&mut self.discard);
        rng.shuffle(&mut self.draw);
    }
}
