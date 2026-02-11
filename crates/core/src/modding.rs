use crate::{ActivationType, BlindKind, Card, ConsumableKind, EffectBlock, GameState, HandKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEffectBlock {
    pub block: EffectBlock,
    #[serde(default)]
    pub selected: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ModHookResult {
    pub stop: bool,
    pub effects: Vec<ModEffectBlock>,
}

impl ModHookResult {
    pub fn merge(&mut self, other: ModHookResult) {
        if other.effects.is_empty() {
            self.stop |= other.stop;
            return;
        }
        self.effects.extend(other.effects);
        self.stop |= other.stop;
    }
}

#[derive(Debug, Serialize)]
pub struct ModHookContext<'a> {
    pub trigger: ActivationType,
    pub state: &'a GameState,
    pub hand_kind: HandKind,
    pub blind: BlindKind,
    pub played: &'a [Card],
    pub scoring: &'a [Card],
    pub held: &'a [Card],
    pub discarded: &'a [Card],
    pub card: Option<Card>,
    pub card_lucky_triggers: i64,
    pub sold_value: Option<i64>,
    pub consumable_kind: Option<ConsumableKind>,
    pub consumable_id: Option<&'a str>,
    pub joker_count: usize,
}

pub trait ModRuntime {
    fn on_hook(&mut self, ctx: &ModHookContext<'_>) -> ModHookResult;
}
