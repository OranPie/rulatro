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
    pub fn new() -> Self {
        Self {
            joker_slots: 5,
            consumable_slots: 2,
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
        if self.jokers.len() >= self.joker_slots {
            return Err(InventoryError::NoJokerSlots);
        }
        self.jokers.push(JokerInstance {
            id,
            rarity,
            edition: None,
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
        if self.consumables.len() >= self.consumable_slots {
            return Err(InventoryError::NoConsumableSlots);
        }
        self.consumables.push(ConsumableInstance { id, kind });
        Ok(())
    }
}
