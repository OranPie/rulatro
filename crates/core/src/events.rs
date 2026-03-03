use crate::{BlindKind, ConsumableKind, HandKind, JokerRarity, ShopOfferKind};
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
    BlindSkipped {
        ante: u8,
        blind: BlindKind,
        tag: Option<String>,
    },
    HandDealt {
        count: usize,
    },
    HandScored {
        hand: HandKind,
        chips: i64,
        mult: f64,
        total: i64,
    },
    RoundEnded {
        hands_used: u32,
        discards_used: u32,
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
    JokerAcquired {
        id: String,
        name: String,
        rarity: JokerRarity,
        cost: i64,
        money: i64,
    },
    ConsumableGained {
        id: String,
        name: String,
        kind: ConsumableKind,
    },
    ConsumableUsed {
        id: String,
        name: String,
        kind: ConsumableKind,
    },
    VoucherBought {
        id: String,
        cost: i64,
        money: i64,
    },
    TagApplied {
        tag_id: String,
    },
    PackOpened {
        kind: ShopOfferKind,
        options: usize,
        picks: u8,
    },
    PackChosen {
        picks: usize,
    },
    JokerSold {
        id: String,
        name: String,
        sell_value: i64,
        money: i64,
    },
    BlindCleared {
        score: i64,
        reward: i64,
        money: i64,
    },
    BlindFailed {
        score: i64,
    },
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
