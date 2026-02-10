use crate::{
    BlindKind, Content, Deck, EffectOp, GameConfig, GameState, Inventory, InventoryError, Phase,
    RngState, ScoreTables, ShopState,
};
use thiserror::Error;
use std::collections::HashMap;


mod helpers;
mod context;
mod state;
mod blind;
mod shop;
mod hand;
mod joker;
mod eval;

use context::{EvalContext, EvalValue};

#[derive(Debug, Error)]
pub enum RunError {
    #[error("missing config for blind {0:?}")]
    MissingBlindRule(BlindKind),
    #[error("missing config for ante {0}")]
    MissingAnteRule(u8),
    #[error("invalid phase: {0:?}")]
    InvalidPhase(Phase),
    #[error("no hands left")]
    NoHandsLeft,
    #[error("no discards left")]
    NoDiscardsLeft,
    #[error("invalid card selection")]
    InvalidSelection,
    #[error("invalid card count")]
    InvalidCardCount,
    #[error("not enough money")]
    NotEnoughMoney,
    #[error("shop not available")]
    ShopNotAvailable,
    #[error("invalid shop offer index")]
    InvalidOfferIndex,
    #[error("invalid joker index")]
    InvalidJokerIndex,
    #[error("blind not cleared")]
    BlindNotCleared,
    #[error("pack not available")]
    PackNotAvailable,
    #[error("inventory error: {0}")]
    Inventory(#[from] InventoryError),
}

#[derive(Debug)]
pub struct RunState {
    pub config: GameConfig,
    pub tables: ScoreTables,
    pub content: Content,
    pub inventory: Inventory,
    pub rng: RngState,
    pub deck: Deck,
    pub hand: Vec<crate::Card>,
    pub state: GameState,
    pub shop: Option<ShopState>,
    pub pending_effects: Vec<EffectOp>,
    pub current_joker_counts: HashMap<String, usize>,
    current_joker_snapshot: Vec<JokerSnapshot>,
    pending_joker_removals: Vec<usize>,
    pending_joker_additions: Vec<crate::JokerInstance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlindOutcome {
    Cleared,
    Failed,
}

#[derive(Debug, Default)]
struct TriggerResults {
    scored_retriggers: i64,
    held_retriggers: i64,
}

#[derive(Debug, Default)]
struct ScoredOutcome {
    destroyed_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
struct JokerSnapshot {
    index: usize,
}
