use crate::{BlindKind, HandKind, ShopOfferKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    BlindStarted {
        ante: u8,
        blind: BlindKind,
        target: i64,
        hands: u8,
        discards: u8,
    },
    HandDealt { count: usize },
    HandScored {
        hand: HandKind,
        chips: i64,
        mult: f64,
        total: i64,
    },
    ShopEntered {
        offers: usize,
        reroll_cost: i64,
        reentered: bool,
    },
    ShopRerolled {
        offers: usize,
        reroll_cost: i64,
        cost: i64,
        money: i64,
    },
    ShopBought {
        offer: ShopOfferKind,
        cost: i64,
        money: i64,
    },
    PackOpened {
        kind: ShopOfferKind,
        options: usize,
        picks: u8,
    },
    PackChosen { picks: usize },
    JokerSold {
        id: String,
        sell_value: i64,
        money: i64,
    },
    BlindCleared { score: i64, reward: i64, money: i64 },
    BlindFailed { score: i64 },
}

#[derive(Debug, Default)]
pub struct EventBus {
    queue: Vec<Event>,
}

impl EventBus {
    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.queue.drain(..)
    }
}
