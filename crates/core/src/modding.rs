use crate::{ActivationType, BlindKind, Card, ConsumableKind, EffectBlock, GameState, HandKind};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// FLOW KERNEL — unified mod interception architecture
// ═══════════════════════════════════════════════════════════════════════════
//
// Every place in the engine that mods can influence is called a FlowPoint.
// A mod registers handlers with register_flow(point, mode, fn, opts).
//
// Three handler modes:
//   Patch   — accumulate parameter overrides; handlers chain, each sees prior
//             accumulated patch; merge policies are per-field (Max, BoolOr, …)
//   Replace — replace core logic entirely; only the highest-priority handler
//             that returns Some(…) wins; others auto-skip.
//   Around  — middleware wrap; receives `next` to call (or skip) core logic.
//
// Execution order within a mode: priority desc → mod_id asc (stable/deterministic).
// ═══════════════════════════════════════════════════════════════════════════

/// All named interception points in the game engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlowPoint {
    /// Hand type evaluation — max_gap, four_fingers, smear, splash, min_len.
    HandEval,
    /// Per-card debuff check — force_debuff or force_shield.
    CardDebuff,
    /// Score base parameters — chips_mult, mult_mult, level_delta.
    ScoreBase,
    /// Shop configuration — allow_duplicates, price deltas, free_rerolls.
    ShopParams,
    /// Named DSL effect action dispatched from a Joker / Boss / Tag.
    JokerEffect,
    /// Named consumable (Tarot / Planet / Spectral) custom EffectOp.
    ConsumableEffect,
    /// Custom hand-type recognition (Replace returns HandTypeOutput).
    HandType,
    /// General lifecycle trigger hook — fires on ActivationType events.
    Lifecycle(ActivationType),
}

/// How a handler interacts with its FlowPoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlowMode {
    /// Accumulate a typed patch; each handler sees the prior merged state.
    Patch,
    /// Replace the core implementation; only the highest-priority winner runs.
    Replace,
    /// Wrap core (or winner); receives `next` so it can call or bypass it.
    Around,
}

// ─── Per-FlowPoint patch structs ────────────────────────────────────────────

/// Accumulated patch for `FlowPoint::HandEval`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandEvalPatch {
    /// Max rank gap allowed in a straight (None → rule_vars fallback, default 1).
    /// Merge policy: **Max** (most permissive wins across all mods).
    #[serde(default)]
    pub max_gap: Option<u32>,
    /// Minimum non-stone card count for flush/straight (None → 5).
    /// Merge policy: **Min** (smallest requirement wins).
    #[serde(default)]
    pub min_len: Option<u32>,
    /// Allow 4-card flush/straight.  Merge: **BoolOr**.
    #[serde(default)]
    pub four_fingers: Option<bool>,
    /// Red suits share one bucket, black suits share another.  Merge: **BoolOr**.
    #[serde(default)]
    pub smear_suits: Option<bool>,
    /// All played cards score regardless of hand type.  Merge: **BoolOr**.
    #[serde(default)]
    pub splash: Option<bool>,
}

impl HandEvalPatch {
    /// Merge `other` into `self` using per-field policies.
    pub fn merge_from(&mut self, other: &HandEvalPatch) {
        self.max_gap     = merge_max_opt(self.max_gap,     other.max_gap);
        self.min_len     = merge_min_opt(self.min_len,     other.min_len);
        self.four_fingers = bool_or_opt(self.four_fingers, other.four_fingers);
        self.smear_suits  = bool_or_opt(self.smear_suits,  other.smear_suits);
        self.splash       = bool_or_opt(self.splash,       other.splash);
    }
}

/// Accumulated patch for `FlowPoint::CardDebuff`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardDebuffPatch {
    /// Unconditionally debuff this card.  Merge: **BoolOr**.
    #[serde(default)]
    pub force_debuff: bool,
    /// Shield this card from debuff (stronger than force_debuff).  Merge: **BoolOr**.
    #[serde(default)]
    pub force_shield: bool,
}

impl CardDebuffPatch {
    pub fn merge_from(&mut self, other: &CardDebuffPatch) {
        self.force_debuff |= other.force_debuff;
        self.force_shield |= other.force_shield;
    }

    /// Final resolution: shield wins over debuff.
    #[inline]
    pub fn resolve(&self, base_debuffed: bool) -> bool {
        if self.force_shield { return false; }
        if self.force_debuff { return true; }
        base_debuffed
    }
}

/// Accumulated patch for `FlowPoint::ScoreBase`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBasePatch {
    /// Multiplicative factor on base chips (identity = 1.0).  Merge: **Mul**.
    #[serde(default = "one_f64")]
    pub chips_mult: f64,
    /// Multiplicative factor on base mult (identity = 1.0).  Merge: **Mul**.
    #[serde(default = "one_f64")]
    pub mult_mult: f64,
    /// Additive hand-level offset before base-stat lookup.  Merge: **Add**.
    #[serde(default)]
    pub level_delta: i64,
}

impl Default for ScoreBasePatch {
    fn default() -> Self {
        Self { chips_mult: 1.0, mult_mult: 1.0, level_delta: 0 }
    }
}

impl ScoreBasePatch {
    pub fn merge_from(&mut self, other: &ScoreBasePatch) {
        self.chips_mult  *= other.chips_mult;
        self.mult_mult   *= other.mult_mult;
        self.level_delta += other.level_delta;
    }
}

/// Accumulated patch for `FlowPoint::ShopParams`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShopParamsPatch {
    /// Allow purchasing duplicate Jokers.  Merge: **BoolOr**.
    #[serde(default)]
    pub allow_duplicates: Option<bool>,
    /// Flat Joker price modifier (negative = cheaper).  Merge: **Add**.
    #[serde(default)]
    pub joker_price_delta: i64,
    /// Free rerolls granted on shop entry.  Merge: **Add**.
    #[serde(default)]
    pub free_rerolls: u32,
}

impl ShopParamsPatch {
    pub fn merge_from(&mut self, other: &ShopParamsPatch) {
        self.allow_duplicates  = bool_or_opt(self.allow_duplicates, other.allow_duplicates);
        self.joker_price_delta += other.joker_price_delta;
        self.free_rerolls      += other.free_rerolls;
    }
}

// ─── Replace / Around output types ──────────────────────────────────────────

/// Output from a Replace / Around handler on `FlowPoint::HandType`.
/// Supersedes `ModHandResult` for new code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandTypeOutput {
    /// Stable string ID for this hand kind (e.g. `"flush_six"`).
    pub hand_id: String,
    /// Indices into the played cards that score for this hand.
    pub scoring_indices: Vec<usize>,
    #[serde(default)]
    pub base_chips: Option<i64>,
    #[serde(default)]
    pub base_mult: Option<f64>,
    #[serde(default)]
    pub level_chips: Option<i64>,
    #[serde(default)]
    pub level_mult: Option<f64>,
}

/// Unified result returned by Replace / Around handlers on effect flow points
/// (`JokerEffect`, `ConsumableEffect`, `Lifecycle`).
/// Supersedes both `ModActionResult` and `ModHookResult` for new code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectOutput {
    /// Whether this handler actually handled the effect.
    /// `false` ⇒ engine prints "unhandled" warning for Custom ops.
    #[serde(default)]
    pub handled: bool,
    // ── Inline score mutations ────────────────────────────────────
    #[serde(default)]
    pub add_chips: i64,
    #[serde(default)]
    pub add_mult: f64,
    /// Applied as: mult *= mul_mult  (0.0 = no-op, same as 1.0).
    #[serde(default)]
    pub mul_mult: f64,
    #[serde(default)]
    pub mul_chips: f64,
    #[serde(default)]
    pub add_money: i64,
    // ── State mutations ───────────────────────────────────────────
    #[serde(default)]
    pub set_rules: Vec<(String, f64)>,
    #[serde(default)]
    pub add_rules: Vec<(String, f64)>,
    #[serde(default)]
    pub set_vars: Vec<(String, f64)>,
    #[serde(default)]
    pub add_vars: Vec<(String, f64)>,
    // ── Pipeline control ──────────────────────────────────────────
    /// Extra effect blocks to execute after this handler returns.
    #[serde(default)]
    pub extra_effects: Vec<ModEffectBlock>,
    /// Stop further handlers on this flow point (do not continue pipeline).
    #[serde(default)]
    pub stop: bool,
    /// Cancel the core implementation (skip core logic after all handlers).
    #[serde(default)]
    pub cancel_core: bool,
}

impl EffectOutput {
    /// Accumulate `other` into `self`.  Addition for numerics, OR for bools.
    pub fn merge_from(&mut self, other: EffectOutput) {
        self.handled   |= other.handled;
        self.add_chips += other.add_chips;
        self.add_mult  += other.add_mult;
        if other.mul_mult  != 0.0 { self.mul_mult  *= other.mul_mult; }
        if other.mul_chips != 0.0 { self.mul_chips *= other.mul_chips; }
        self.add_money += other.add_money;
        self.set_rules.extend(other.set_rules);
        self.add_rules.extend(other.add_rules);
        self.set_vars.extend(other.set_vars);
        self.add_vars.extend(other.add_vars);
        self.extra_effects.extend(other.extra_effects);
        self.stop        |= other.stop;
        self.cancel_core |= other.cancel_core;
    }
}

// Convert legacy types → EffectOutput for migration paths.
impl From<ModActionResult> for EffectOutput {
    fn from(r: ModActionResult) -> Self {
        Self {
            handled: r.add_chips != 0 || r.add_mult != 0.0 || !r.set_rules.is_empty(),
            add_chips: r.add_chips,
            add_mult:  r.add_mult,
            mul_mult:  r.mul_mult,
            mul_chips: r.mul_chips,
            add_money: r.add_money,
            set_rules: r.set_rules,
            add_rules: r.add_rules,
            set_vars:  r.set_vars,
            add_vars:  r.add_vars,
            ..Default::default()
        }
    }
}

impl From<ModHookResult> for EffectOutput {
    fn from(r: ModHookResult) -> Self {
        Self {
            handled:      !r.effects.is_empty(),
            extra_effects: r.effects,
            stop:          r.stop,
            cancel_core:   r.cancel_core,
            ..Default::default()
        }
    }
}

// ─── Unified context ─────────────────────────────────────────────────────────

/// Context passed to every flow handler, regardless of FlowPoint.
/// Fields are populated based on the active point; unused fields are None / empty slice.
#[derive(Debug, Serialize)]
pub struct FlowCtx<'a> {
    /// Which flow point is active.
    pub point: FlowPoint,
    /// Always present.
    pub state: &'a GameState,

    // ── Scoring / hand context ─────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_kind:  Option<HandKind>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub played:   &'a [Card],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub scoring:  &'a [Card],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub held:     &'a [Card],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub discarded: &'a [Card],

    // ── Per-card context ────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<Card>,

    // ── Effect / action context ─────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joker_id:     Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_name:  Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_value: Option<f64>,

    // ── Lifecycle / trigger context ─────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<ActivationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blind:   Option<BlindKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_lucky_triggers: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sold_value:      Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumable_kind: Option<ConsumableKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumable_id:   Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joker_count: Option<usize>,
}

fn slice_is_empty<T>(s: &&[T]) -> bool { s.is_empty() }

impl<'a> FlowCtx<'a> {
    /// Construct a minimal context for pure-patch flow points.
    pub fn patch(point: FlowPoint, state: &'a GameState) -> Self {
        Self {
            point, state,
            hand_kind: None, played: &[], scoring: &[], held: &[], discarded: &[],
            card: None, joker_id: None, effect_name: None, effect_value: None,
            trigger: None, blind: None, card_lucky_triggers: None, sold_value: None,
            consumable_kind: None, consumable_id: None, joker_count: None,
        }
    }

    /// Construct a context for per-card debuff checks.
    pub fn card_debuff(state: &'a GameState, card: Card) -> Self {
        let mut ctx = Self::patch(FlowPoint::CardDebuff, state);
        ctx.card = Some(card);
        ctx
    }

    /// Construct a context for Joker / custom effect actions.
    pub fn joker_effect(
        state: &'a GameState,
        hand_kind: Option<HandKind>,
        card: Option<Card>,
        joker_id: Option<&'a str>,
        effect_name: &'a str,
        effect_value: f64,
    ) -> Self {
        let mut ctx = Self::patch(FlowPoint::JokerEffect, state);
        ctx.hand_kind    = hand_kind;
        ctx.card         = card;
        ctx.joker_id     = joker_id;
        ctx.effect_name  = Some(effect_name);
        ctx.effect_value = Some(effect_value);
        ctx
    }

    /// Construct a context for consumable custom EffectOp dispatch.
    pub fn consumable_effect(
        state: &'a GameState,
        effect_name: &'a str,
        effect_value: f64,
    ) -> Self {
        let mut ctx = Self::patch(FlowPoint::ConsumableEffect, state);
        ctx.effect_name  = Some(effect_name);
        ctx.effect_value = Some(effect_value);
        ctx
    }

    /// Construct a context for custom hand-type evaluation.
    pub fn hand_type(state: &'a GameState, played: &'a [Card]) -> Self {
        let mut ctx = Self::patch(FlowPoint::HandType, state);
        ctx.played = played;
        ctx
    }

    /// Build from a legacy `ModEffectContext` for migration paths.
    pub fn from_legacy_effect(legacy: &'a ModEffectContext<'a>, name: &'a str, value: f64) -> Self {
        Self::joker_effect(
            legacy.state,
            legacy.hand_kind,
            legacy.card,
            legacy.joker_id,
            name,
            value,
        )
    }
}

// ─── Internal merge helpers ──────────────────────────────────────────────────

fn merge_max_opt(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, y) => x.or(y),
    }
}

fn merge_min_opt(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, y) => x.or(y),
    }
}

fn bool_or_opt(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        (x, y) => x.or(y),
    }
}

fn one_f64() -> f64 { 1.0 }

// ═══════════════════════════════════════════════════════════════════════════
// LEGACY TYPES — kept for backward compatibility; new code should use Flow Kernel
// ═══════════════════════════════════════════════════════════════════════════

/// Hook phase (Pre / Post).  Legacy; new code uses `FlowMode` on `FlowPoint::Lifecycle`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModHookPhase {
    Pre,
    Post,
}

/// An effect block + selection vector queued by a mod hook result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEffectBlock {
    pub block: EffectBlock,
    #[serde(default)]
    pub selected: Vec<usize>,
}

/// Result returned by `ModRuntime::on_hook`.  Legacy; new code returns `EffectOutput`.
#[derive(Debug, Clone, Default)]
pub struct ModHookResult {
    pub stop: bool,
    pub cancel_core: bool,
    pub effects: Vec<ModEffectBlock>,
}

impl ModHookResult {
    pub fn merge(&mut self, other: ModHookResult) {
        self.effects.extend(other.effects);
        self.stop        |= other.stop;
        self.cancel_core |= other.cancel_core;
    }
}

/// Score / state mutations returned by a DSL custom action (`register_effect`).
/// Legacy; new code returns `EffectOutput`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModActionResult {
    #[serde(default)] pub add_chips: i64,
    #[serde(default)] pub add_mult: f64,
    #[serde(default)] pub mul_mult: f64,
    #[serde(default)] pub mul_chips: f64,
    #[serde(default)] pub add_money: i64,
    #[serde(default)] pub set_rules: Vec<(String, f64)>,
    #[serde(default)] pub add_rules: Vec<(String, f64)>,
    #[serde(default)] pub set_vars: Vec<(String, f64)>,
    #[serde(default)] pub add_vars: Vec<(String, f64)>,
}

/// Context passed to `invoke_effect` / `invoke_effect_op`.  Legacy; new code uses `FlowCtx`.
#[derive(Debug, Serialize)]
pub struct ModEffectContext<'a> {
    pub state:    &'a GameState,
    pub hand_kind: Option<HandKind>,
    pub card:      Option<Card>,
    pub joker_id:  Option<&'a str>,
}

/// Context passed to `evaluate_hand`.  Legacy; new code uses `FlowCtx`.
#[derive(Debug, Serialize)]
pub struct ModHandEvalContext<'a> {
    pub state:         &'a GameState,
    pub cards:         &'a [Card],
    pub smeared_suits: bool,
    pub four_fingers:  bool,
    pub shortcut:      bool,
}

/// Result of a custom hand evaluation.  Legacy; new code uses `HandTypeOutput`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModHandResult {
    pub hand_id:         String,
    pub scoring_indices: Vec<usize>,
    #[serde(default)] pub base_chips:  Option<i64>,
    #[serde(default)] pub base_mult:   Option<f64>,
    #[serde(default)] pub level_chips: Option<i64>,
    #[serde(default)] pub level_mult:  Option<f64>,
}

impl From<ModHandResult> for HandTypeOutput {
    fn from(r: ModHandResult) -> Self {
        Self {
            hand_id: r.hand_id,
            scoring_indices: r.scoring_indices,
            base_chips:  r.base_chips,
            base_mult:   r.base_mult,
            level_chips: r.level_chips,
            level_mult:  r.level_mult,
        }
    }
}

impl From<HandTypeOutput> for ModHandResult {
    fn from(o: HandTypeOutput) -> Self {
        Self {
            hand_id: o.hand_id,
            scoring_indices: o.scoring_indices,
            base_chips:  o.base_chips,
            base_mult:   o.base_mult,
            level_chips: o.level_chips,
            level_mult:  o.level_mult,
        }
    }
}

/// Definition for a custom hand stored in `RunState`.
#[derive(Debug, Clone)]
pub struct CustomHandDef {
    pub id:          String,
    pub base_chips:  i64,
    pub base_mult:   f64,
    pub level_chips: i64,
    pub level_mult:  f64,
}

/// Full context for `on_hook`.  Legacy; new code uses `FlowCtx`.
#[derive(Debug, Serialize)]
pub struct ModHookContext<'a> {
    pub phase:               ModHookPhase,
    pub trigger:             ActivationType,
    pub state:               &'a GameState,
    pub hand_kind:           HandKind,
    pub blind:               BlindKind,
    pub played:              &'a [Card],
    pub scoring:             &'a [Card],
    pub held:                &'a [Card],
    pub discarded:           &'a [Card],
    pub card:                Option<Card>,
    pub card_lucky_triggers: i64,
    pub sold_value:          Option<i64>,
    pub consumable_kind:     Option<ConsumableKind>,
    pub consumable_id:       Option<&'a str>,
    pub joker_count:         usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// ModRuntime TRAIT
// ═══════════════════════════════════════════════════════════════════════════

pub trait ModRuntime {
    // ── Legacy methods (kept for backward compat) ────────────────────────────

    fn on_hook(&mut self, ctx: &ModHookContext<'_>) -> ModHookResult;

    fn invoke_effect(
        &mut self,
        name: &str,
        target: Option<&str>,
        value: f64,
        ctx: &ModEffectContext<'_>,
    ) -> ModActionResult {
        let _ = (name, target, value, ctx);
        ModActionResult::default()
    }

    fn evaluate_hand(&mut self, ctx: &ModHandEvalContext<'_>) -> Option<ModHandResult> {
        let _ = ctx;
        None
    }

    fn invoke_effect_op(&mut self, name: &str, value: f64, ctx: &ModEffectContext<'_>) -> bool {
        let _ = (name, value, ctx);
        false
    }

    // ── Flow Kernel methods (new, all have no-op defaults) ───────────────────

    /// Run all registered `Patch` handlers for `FlowPoint::HandEval`.
    /// Each handler receives the accumulated patch so far and returns a modified copy.
    fn flow_hand_eval_patch(&mut self, base: HandEvalPatch, ctx: &FlowCtx<'_>) -> HandEvalPatch {
        let _ = ctx;
        base
    }

    /// Run all registered `Patch` handlers for `FlowPoint::CardDebuff`.
    fn flow_card_debuff_patch(
        &mut self,
        base: CardDebuffPatch,
        ctx: &FlowCtx<'_>,
    ) -> CardDebuffPatch {
        let _ = ctx;
        base
    }

    /// Run all registered `Patch` handlers for `FlowPoint::ScoreBase`.
    fn flow_score_base_patch(&mut self, base: ScoreBasePatch, ctx: &FlowCtx<'_>) -> ScoreBasePatch {
        let _ = ctx;
        base
    }

    /// Run all registered `Patch` handlers for `FlowPoint::ShopParams`.
    fn flow_shop_params_patch(
        &mut self,
        base: ShopParamsPatch,
        ctx: &FlowCtx<'_>,
    ) -> ShopParamsPatch {
        let _ = ctx;
        base
    }

    /// Run the winning `Replace` handler (if any) for `FlowPoint::HandType`.
    /// Returns `None` to fall through to the standard evaluator.
    fn flow_hand_type_replace(&mut self, ctx: &FlowCtx<'_>) -> Option<HandTypeOutput> {
        let _ = ctx;
        None
    }

    /// Run all handlers for a `Lifecycle` flow point (replaces `on_hook` for new code).
    fn flow_lifecycle(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        let _ = ctx;
        EffectOutput::default()
    }

    /// Run all handlers for `FlowPoint::JokerEffect` (replaces `invoke_effect`).
    fn flow_joker_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        let _ = ctx;
        EffectOutput::default()
    }

    /// Run all handlers for `FlowPoint::ConsumableEffect` (replaces `invoke_effect_op`).
    fn flow_consumable_effect(&mut self, ctx: &FlowCtx<'_>) -> EffectOutput {
        let _ = ctx;
        EffectOutput::default()
    }
}

