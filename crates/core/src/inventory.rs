use crate::{ConsumableKind, Edition, JokerRarity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct JokerStickers {
    pub eternal: bool,
    pub perishable: bool,
    pub rental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JokerInstance {
    pub id: String,
    pub rarity: JokerRarity,
    #[serde(default)]
    pub edition: Option<Edition>,
    #[serde(default)]
    pub stickers: JokerStickers,
    #[serde(default)]
    pub buy_price: i64,
    #[serde(default)]
    pub vars: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableInstance {
    pub id: String,
    pub kind: ConsumableKind,
    #[serde(default)]
    pub edition: Option<Edition>,
    #[serde(default)]
    pub sell_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub joker_slots: usize,
    pub consumable_slots: usize,
    pub jokers: Vec<JokerInstance>,
    pub consumables: Vec<ConsumableInstance>,
}

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("no joker slots")]
    NoJokerSlots,
    #[error("no consumable slots")]
    NoConsumableSlots,
}

impl Inventory {
    pub fn joker_capacity(&self) -> usize {
        self.joker_slots + self.negative_joker_bonus()
    }

    pub fn negative_joker_bonus(&self) -> usize {
        self.jokers
            .iter()
            .filter(|joker| joker.edition == Some(Edition::Negative))
            .count()
    }

    pub fn new() -> Self {
        Self::with_slots(5, 2)
    }

    /// Create a new inventory with explicit initial slot counts.
    pub fn with_slots(joker_slots: usize, consumable_slots: usize) -> Self {
        Self {
            joker_slots,
            consumable_slots,
            jokers: Vec::new(),
            consumables: Vec::new(),
        }
    }

    pub fn add_joker(
        &mut self,
        id: String,
        rarity: JokerRarity,
        buy_price: i64,
    ) -> Result<(), InventoryError> {
        self.add_joker_with_edition(id, rarity, buy_price, None)
    }

    pub fn add_joker_with_edition(
        &mut self,
        id: String,
        rarity: JokerRarity,
        buy_price: i64,
        edition: Option<Edition>,
    ) -> Result<(), InventoryError> {
        let mut capacity = self.joker_capacity();
        if edition == Some(Edition::Negative) {
            capacity = capacity.saturating_add(1);
        }
        if self.jokers.len() >= capacity {
            return Err(InventoryError::NoJokerSlots);
        }
        self.jokers.push(JokerInstance {
            id,
            rarity,
            edition,
            stickers: JokerStickers::default(),
            buy_price,
            vars: HashMap::new(),
        });
        Ok(())
    }

    pub fn add_consumable(
        &mut self,
        id: String,
        kind: ConsumableKind,
    ) -> Result<(), InventoryError> {
        self.add_consumable_with_edition(id, kind, None, 0.0)
    }

    pub fn add_consumable_with_edition(
        &mut self,
        id: String,
        kind: ConsumableKind,
        edition: Option<Edition>,
        sell_bonus: f64,
    ) -> Result<(), InventoryError> {
        if edition != Some(Edition::Negative) && self.consumable_count() >= self.consumable_slots {
            return Err(InventoryError::NoConsumableSlots);
        }
        self.consumables.push(ConsumableInstance {
            id,
            kind,
            edition,
            sell_bonus,
        });
        Ok(())
    }

    pub fn consumable_count(&self) -> usize {
        self.consumables
            .iter()
            .filter(|item| item.edition != Some(Edition::Negative))
            .count()
    }
}
