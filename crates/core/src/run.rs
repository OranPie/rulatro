use crate::{
    BlindKind, Content, CustomHandDef, Deck, EffectOp, GameConfig, GameState, Inventory,
    InventoryError, ModRuntime, Phase, RngState, ScoreTables, ScoreTraceStep, ShopState,
};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

mod action_handlers;
mod blind;
mod context;
mod eval;
mod hand;
mod helpers;
mod hooks;
mod joker;
mod shop;
mod state;

use context::{EvalContext, EvalValue};
#[allow(unused_imports)]
use hooks::{HookArgs, HookInject, HookPoint, HookPriority, HookRegistry, HookResult, RuleHook};

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
    #[error("hand type not allowed")]
    HandNotAllowed,
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
    #[error("cannot skip boss blind")]
    CannotSkipBoss,
    #[error("inventory error: {0}")]
    Inventory(#[from] InventoryError),
}

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
    pub last_score_trace: Vec<ScoreTraceStep>,
    pub current_joker_counts: HashMap<String, usize>,
    current_joker_snapshot: Vec<JokerSnapshot>,
    pending_joker_removals: Vec<usize>,
    pending_joker_additions: Vec<crate::JokerInstance>,
    last_destroyed_sell_value: i64,
    boss_disable_pending: bool,
    boss_disabled: bool,
    prevent_death: bool,
    rule_vars: HashMap<String, f64>,
    rule_dirty: bool,
    refreshing_rules: bool,
    next_card_id: u32,
    copy_depth: u8,
    copy_stack: Vec<usize>,
    joker_effect_depth: u8,
    deferred_card_added: Vec<crate::Card>,
    mod_runtime: Option<Box<dyn ModRuntime>>,
    hooks: HookRegistry,
    /// Custom hands registered by mods. The index into this Vec corresponds to
    /// the `u32` in `HandKind::Custom(u32)`.
    pub custom_hand_registry: Vec<CustomHandDef>,
}

impl fmt::Debug for RunState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunState")
            .field("state", &self.state)
            .field("hand", &self.hand.len())
            .field("inventory", &self.inventory)
            .field("mod_runtime", &self.mod_runtime.is_some())
            .finish()
    }
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
    destroyed_current: bool,
}

#[derive(Debug, Default)]
struct ScoredOutcome {
    destroyed_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
struct JokerSnapshot {
    index: usize,
}
