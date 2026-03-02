use super::*;
use crate::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HookPoint {
    Played,
    ScoredPre,
    Scored,
    Held,
    Independent,
    Discard,
    DiscardBatch,
    CardDestroyed,
    CardAdded,
    RoundEnd,
    HandEnd,
    BlindStart,
    BlindFailed,
    ShopEnter,
    ShopReroll,
    ShopExit,
    PackOpened,
    PackSkipped,
    UseConsumable,
    Sell,
    AnySell,
    Acquire,
    OtherJokers,
    Passive,
}

impl HookPoint {
    const ALL: [Self; 24] = [
        Self::Played,
        Self::ScoredPre,
        Self::Scored,
        Self::Held,
        Self::Independent,
        Self::Discard,
        Self::DiscardBatch,
        Self::CardDestroyed,
        Self::CardAdded,
        Self::RoundEnd,
        Self::HandEnd,
        Self::BlindStart,
        Self::BlindFailed,
        Self::ShopEnter,
        Self::ShopReroll,
        Self::ShopExit,
        Self::PackOpened,
        Self::PackSkipped,
        Self::UseConsumable,
        Self::Sell,
        Self::AnySell,
        Self::Acquire,
        Self::OtherJokers,
        Self::Passive,
    ];
}

const HOOK_ACTIVATION_BINDINGS: &[(HookPoint, ActivationType)] = &[
    (HookPoint::Played, ActivationType::OnPlayed),
    (HookPoint::ScoredPre, ActivationType::OnScoredPre),
    (HookPoint::Scored, ActivationType::OnScored),
    (HookPoint::Held, ActivationType::OnHeld),
    (HookPoint::Independent, ActivationType::Independent),
    (HookPoint::Discard, ActivationType::OnDiscard),
    (HookPoint::DiscardBatch, ActivationType::OnDiscardBatch),
    (HookPoint::CardDestroyed, ActivationType::OnCardDestroyed),
    (HookPoint::CardAdded, ActivationType::OnCardAdded),
    (HookPoint::RoundEnd, ActivationType::OnRoundEnd),
    (HookPoint::HandEnd, ActivationType::OnHandEnd),
    (HookPoint::BlindStart, ActivationType::OnBlindStart),
    (HookPoint::BlindFailed, ActivationType::OnBlindFailed),
    (HookPoint::ShopEnter, ActivationType::OnShopEnter),
    (HookPoint::ShopReroll, ActivationType::OnShopReroll),
    (HookPoint::ShopExit, ActivationType::OnShopExit),
    (HookPoint::PackOpened, ActivationType::OnPackOpened),
    (HookPoint::PackSkipped, ActivationType::OnPackSkipped),
    (HookPoint::UseConsumable, ActivationType::OnUse),
    (HookPoint::Sell, ActivationType::OnSell),
    (HookPoint::AnySell, ActivationType::OnAnySell),
    (HookPoint::Acquire, ActivationType::OnAcquire),
    (HookPoint::OtherJokers, ActivationType::OnOtherJokers),
    (HookPoint::Passive, ActivationType::Passive),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HookPriority {
    CoreRules = 0,
    Tags = 1,
    Jokers = 2,
    Post = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HookResult {
    Continue,
    CancelCore,
    Stop,
}

pub(super) struct HookArgs<'a> {
    pub hand_kind: crate::HandKind,
    pub blind: BlindKind,
    pub inject: HookInject<'a>,
    pub card: Option<Card>,
    pub card_mut: Option<&'a mut Card>,
    pub card_lucky_triggers: i64,
    pub sold_value: Option<i64>,
    pub consumable_kind: Option<ConsumableKind>,
    pub consumable_id: Option<&'a str>,
    pub score: &'a mut Score,
    pub money: &'a mut i64,
    pub results: &'a mut TriggerResults,
    pub sold_joker: Option<&'a mut JokerInstance>,
}

#[derive(Debug)]
pub(super) struct HookInject<'a> {
    pub played: Option<&'a mut [Card]>,
    pub scoring: Option<&'a mut [Card]>,
    pub held: Option<&'a mut [Card]>,
    pub discarded: Option<&'a mut [Card]>,
}

impl<'a> HookInject<'a> {
    pub(super) fn none() -> Self {
        Self {
            played: None,
            scoring: None,
            held: None,
            discarded: None,
        }
    }

    pub(super) fn cards(
        played: Option<&'a mut [Card]>,
        scoring: Option<&'a mut [Card]>,
        held: Option<&'a mut [Card]>,
        discarded: Option<&'a mut [Card]>,
    ) -> Self {
        Self {
            played,
            scoring,
            held,
            discarded,
        }
    }

    pub(super) fn played(
        played: &'a mut [Card],
        scoring: &'a mut [Card],
        held: &'a mut [Card],
    ) -> Self {
        Self::cards(Some(played), Some(scoring), Some(held), None)
    }

    pub(super) fn held(held: &'a mut [Card]) -> Self {
        Self::cards(None, None, Some(held), None)
    }

    pub(super) fn discard(held: &'a mut [Card], discarded: &'a mut [Card]) -> Self {
        Self::cards(None, None, Some(held), Some(discarded))
    }
}

#[derive(Debug, Clone, Copy)]
struct HookView<'a> {
    hand_kind: crate::HandKind,
    blind: BlindKind,
    played: &'a [Card],
    scoring: &'a [Card],
    held: &'a [Card],
    discarded: &'a [Card],
    card: Option<Card>,
    card_lucky_triggers: i64,
    sold_value: Option<i64>,
    consumable_kind: Option<ConsumableKind>,
    consumable_id: Option<&'a str>,
}

impl<'a> HookView<'a> {
    fn from_parts(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Option<Card>,
        card_lucky_triggers: i64,
        sold_value: Option<i64>,
        consumable_kind: Option<ConsumableKind>,
        consumable_id: Option<&'a str>,
        inject: &'a HookInject<'a>,
    ) -> Self {
        let played = inject.played.as_deref().unwrap_or(&[]);
        let scoring = inject.scoring.as_deref().unwrap_or(&[]);
        let held = inject.held.as_deref().unwrap_or(&[]);
        let discarded = inject.discarded.as_deref().unwrap_or(&[]);
        Self {
            hand_kind,
            blind,
            played,
            scoring,
            held,
            discarded,
            card,
            card_lucky_triggers,
            sold_value,
            consumable_kind,
            consumable_id,
        }
    }
}

impl<'a> HookArgs<'a> {
    pub(super) fn independent(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: None,
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn played(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self::independent(hand_kind, blind, inject, score, money, results)
    }

    pub(super) fn scoring(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        lucky_triggers: i64,
        inject: HookInject<'a>,
        card_mut: Option<&'a mut Card>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: Some(card),
            card_mut,
            card_lucky_triggers: lucky_triggers,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn held(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: Some(card),
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn discard(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: Some(card),
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn discard_batch(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: None,
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn consumable(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        consumable_kind: ConsumableKind,
        consumable_id: &'a str,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: None,
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: Some(consumable_kind),
            consumable_id: Some(consumable_id),
            score,
            money,
            results,
            sold_joker: None,
        }
    }

    pub(super) fn sell(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        sold_value: i64,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
        sold_joker: Option<&'a mut JokerInstance>,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: None,
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: Some(sold_value),
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker,
        }
    }

    pub(super) fn card_added(
        hand_kind: crate::HandKind,
        blind: BlindKind,
        card: Card,
        inject: HookInject<'a>,
        score: &'a mut Score,
        money: &'a mut i64,
        results: &'a mut TriggerResults,
    ) -> Self {
        Self {
            hand_kind,
            blind,
            inject,
            card: Some(card),
            card_mut: None,
            card_lucky_triggers: 0,
            sold_value: None,
            consumable_kind: None,
            consumable_id: None,
            score,
            money,
            results,
            sold_joker: None,
        }
    }
}

pub(super) trait RuleHook {
    fn id(&self) -> &'static str;
    fn priority(&self) -> HookPriority;
    fn on_hook(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) -> HookResult;
}

struct HookEntry {
    priority: HookPriority,
    order: usize,
    hook: Box<dyn RuleHook>,
}

pub(super) struct HookRegistry {
    hooks: Vec<HookEntry>,
    next_order: usize,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    pub(super) fn new() -> Self {
        Self {
            hooks: Vec::new(),
            next_order: 0,
        }
    }

    pub(super) fn with_defaults() -> Self {
        assert!(
            hook_activation_bindings_valid(),
            "invalid HookPoint -> ActivationType bindings"
        );
        let mut registry = Self::new();
        // TODO: expose hook registration to mod runtime (mixin injection).
        registry.register(Box::new(ModRuntimeHook::new(crate::ModHookPhase::Pre)));
        registry.register(Box::new(BossHook::new()));
        registry.register(Box::new(TagHook::new()));
        registry.register(Box::new(JokerDslHook::new()));
        registry.register(Box::new(ModRuntimeHook::new(crate::ModHookPhase::Post)));
        registry
    }

    pub(super) fn register(&mut self, hook: Box<dyn RuleHook>) {
        let priority = hook.priority();
        let order = self.next_order;
        self.next_order = self.next_order.saturating_add(1);
        self.hooks.push(HookEntry {
            priority,
            order,
            hook,
        });
        self.hooks
            .sort_by(|left, right| (left.priority, left.order).cmp(&(right.priority, right.order)));
    }

    pub(super) fn invoke(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) {
        let mut cancel_core = false;
        for entry in self.hooks.iter_mut() {
            if cancel_core && entry.priority != HookPriority::Post {
                continue;
            }
            match entry.hook.on_hook(point, run, events, args) {
                HookResult::Continue => {}
                HookResult::CancelCore => {
                    cancel_core = true;
                }
                HookResult::Stop => break,
            }
        }
    }
}

impl fmt::Debug for HookRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HookRegistry")
            .field("hooks", &self.hooks.len())
            .finish()
    }
}

impl RunState {
    pub(super) fn invoke_hooks(
        &mut self,
        point: HookPoint,
        args: &mut HookArgs<'_>,
        events: &mut EventBus,
    ) {
        let mut hooks = std::mem::take(&mut self.hooks);
        hooks.invoke(point, self, events, args);
        self.hooks = hooks;
    }
}

struct JokerDslHook;

impl JokerDslHook {
    fn new() -> Self {
        Self
    }
}

struct ModRuntimeHook {
    phase: crate::ModHookPhase,
}

impl ModRuntimeHook {
    fn new(phase: crate::ModHookPhase) -> Self {
        Self { phase }
    }
}

impl RuleHook for ModRuntimeHook {
    fn id(&self) -> &'static str {
        "mod_runtime"
    }

    fn priority(&self) -> HookPriority {
        match self.phase {
            crate::ModHookPhase::Pre => HookPriority::CoreRules,
            crate::ModHookPhase::Post => HookPriority::Post,
        }
    }

    fn on_hook(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) -> HookResult {
        let Some(runtime) = run.mod_runtime.as_mut() else {
            return HookResult::Continue;
        };
        let Some(trigger) = hook_trigger_for(point) else {
            return HookResult::Continue;
        };
        let inject = std::mem::replace(&mut args.inject, HookInject::none());
        let view = HookView::from_parts(
            args.hand_kind,
            args.blind,
            args.card,
            args.card_lucky_triggers,
            args.sold_value,
            args.consumable_kind,
            args.consumable_id,
            &inject,
        );
        let ctx = crate::ModHookContext {
            phase: self.phase,
            trigger,
            state: &run.state,
            hand_kind: view.hand_kind,
            blind: view.blind,
            played: view.played,
            scoring: view.scoring,
            held: view.held,
            discarded: view.discarded,
            card: view.card,
            card_lucky_triggers: view.card_lucky_triggers,
            sold_value: view.sold_value,
            consumable_kind: view.consumable_kind,
            consumable_id: view.consumable_id,
            joker_count: run.inventory.jokers.len(),
        };
        let mut result = runtime.on_hook(&ctx);
        if !result.effects.is_empty() {
            for effect in result.effects.drain(..) {
                let _ = run.apply_effect_blocks(
                    std::slice::from_ref(&effect.block),
                    trigger,
                    view.hand_kind,
                    view.card,
                    &effect.selected,
                    args.score,
                    args.money,
                    events,
                );
            }
        }
        args.inject = inject;
        if result.stop {
            HookResult::Stop
        } else if result.cancel_core && self.phase == crate::ModHookPhase::Pre {
            HookResult::CancelCore
        } else {
            HookResult::Continue
        }
    }
}

impl RuleHook for JokerDslHook {
    fn id(&self) -> &'static str {
        "joker_dsl"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::Jokers
    }

    fn on_hook(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        _events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) -> HookResult {
        let inject = std::mem::replace(&mut args.inject, HookInject::none());
        let view = HookView::from_parts(
            args.hand_kind,
            args.blind,
            args.card,
            args.card_lucky_triggers,
            args.sold_value,
            args.consumable_kind,
            args.consumable_id,
            &inject,
        );
        if point == HookPoint::Independent {
            run.apply_joker_editions_and_independent(
                args.score,
                args.money,
                view.hand_kind,
                view.played,
                view.scoring,
                view.held,
                run.inventory.jokers.len(),
            );
            args.inject = inject;
            return HookResult::Continue;
        }
        if point == HookPoint::Sell {
            let Some(joker) = args.sold_joker.as_deref_mut() else {
                args.inject = inject;
                return HookResult::Continue;
            };
            let ctx = build_eval_context(point, run, view);
            run.apply_joker_effects_for_joker(
                joker,
                ActivationType::OnSell,
                &ctx,
                None,
                args.score,
                args.money,
                args.results,
            );
            args.inject = inject;
            return HookResult::Continue;
        }
        let Some(trigger) = hook_trigger_for(point) else {
            args.inject = inject;
            return HookResult::Continue;
        };
        let ctx = build_eval_context(point, run, view);
        let card_mut = args.card_mut.as_deref_mut();
        run.apply_joker_effects(
            trigger,
            &ctx,
            card_mut,
            args.score,
            args.money,
            args.results,
        );
        args.inject = inject;
        HookResult::Continue
    }
}

struct BossHook {
    vars: HashMap<String, HashMap<String, f64>>,
}

impl BossHook {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

impl RuleHook for BossHook {
    fn id(&self) -> &'static str {
        "boss_rules"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::CoreRules
    }

    fn on_hook(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        _events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) -> HookResult {
        if run.state.blind != BlindKind::Boss {
            return HookResult::Continue;
        }
        let Some(boss_id) = run.state.boss_id.clone() else {
            return HookResult::Continue;
        };
        if run.boss_disabled() {
            return HookResult::Continue;
        }
        let Some(trigger) = hook_trigger_for(point) else {
            return HookResult::Continue;
        };
        let inject = std::mem::replace(&mut args.inject, HookInject::none());
        let view = HookView::from_parts(
            args.hand_kind,
            args.blind,
            args.card,
            args.card_lucky_triggers,
            args.sold_value,
            args.consumable_kind,
            args.consumable_id,
            &inject,
        );
        let ctx = build_eval_context(point, run, view);
        let effects = match run.content.boss_by_id(&boss_id) {
            Some(def) => def.effects.clone(),
            None => {
                args.inject = inject;
                return HookResult::Continue;
            }
        };
        let vars = self.vars.entry(boss_id.to_string()).or_default();
        let mut card_mut = args.card_mut.take();
        let card_ref = card_mut.as_deref_mut();
        apply_effect_list(run, &boss_id, &effects, trigger, &ctx, vars, args, card_ref);
        args.card_mut = card_mut;
        args.inject = inject;
        HookResult::Continue
    }
}

struct TagHook {
    vars: HashMap<String, HashMap<String, f64>>,
}

impl TagHook {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

impl RuleHook for TagHook {
    fn id(&self) -> &'static str {
        "tag_rules"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::Tags
    }

    fn on_hook(
        &mut self,
        point: HookPoint,
        run: &mut RunState,
        _events: &mut EventBus,
        args: &mut HookArgs<'_>,
    ) -> HookResult {
        if run.state.tags.is_empty() {
            return HookResult::Continue;
        }
        let Some(trigger) = hook_trigger_for(point) else {
            return HookResult::Continue;
        };
        let inject = std::mem::replace(&mut args.inject, HookInject::none());
        let view = HookView::from_parts(
            args.hand_kind,
            args.blind,
            args.card,
            args.card_lucky_triggers,
            args.sold_value,
            args.consumable_kind,
            args.consumable_id,
            &inject,
        );
        let ctx = build_eval_context(point, run, view);
        let mut consumed = Vec::new();
        let tags_snapshot = run.state.tags.clone();
        for (index, tag_id) in tags_snapshot.iter().enumerate() {
            let effects = match run.content.tag_by_id(tag_id) {
                Some(def) => def.effects.clone(),
                None => continue,
            };
            let vars = self.vars.entry(tag_id.clone()).or_default();
            let mut card_mut = args.card_mut.take();
            let card_ref = card_mut.as_deref_mut();
            let triggered =
                apply_effect_list(run, tag_id, &effects, trigger, &ctx, vars, args, card_ref);
            args.card_mut = card_mut;
            if triggered {
                consumed.push(index);
            }
        }
        if !consumed.is_empty() {
            consumed.sort_unstable();
            consumed.dedup();
            for index in consumed.into_iter().rev() {
                if index < run.state.tags.len() {
                    let removed = run.state.tags.remove(index);
                    self.vars.remove(&removed);
                }
            }
            run.mark_rules_dirty();
        }
        args.inject = inject;
        HookResult::Continue
    }
}

fn hook_activation_bindings_valid() -> bool {
    for (idx, (point, _)) in HOOK_ACTIVATION_BINDINGS.iter().enumerate() {
        if !HookPoint::ALL.contains(point) {
            return false;
        }
        if HOOK_ACTIVATION_BINDINGS
            .iter()
            .skip(idx + 1)
            .any(|(other, _)| other == point)
        {
            return false;
        }
    }
    HookPoint::ALL.iter().all(|point| {
        HOOK_ACTIVATION_BINDINGS
            .iter()
            .any(|(other, _)| other == point)
    })
}

fn hook_trigger_for(point: HookPoint) -> Option<ActivationType> {
    HOOK_ACTIVATION_BINDINGS
        .iter()
        .find(|(hook_point, _)| *hook_point == point)
        .map(|(_, trigger)| *trigger)
}

fn build_eval_context<'a>(point: HookPoint, run: &RunState, view: HookView<'a>) -> EvalContext<'a> {
    let joker_count = run.inventory.jokers.len();
    match point {
        HookPoint::Played => EvalContext::played(
            view.hand_kind,
            view.blind,
            view.played,
            view.scoring,
            view.held,
            run.state.hands_left,
            run.state.discards_left,
            joker_count,
        ),
        HookPoint::Scored | HookPoint::ScoredPre | HookPoint::CardDestroyed => view
            .card
            .map(|card| {
                EvalContext::scoring(
                    view.hand_kind,
                    view.blind,
                    card,
                    view.card_lucky_triggers,
                    view.played,
                    view.scoring,
                    view.held,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            })
            .unwrap_or_else(|| {
                EvalContext::independent(
                    view.hand_kind,
                    view.blind,
                    view.played,
                    view.scoring,
                    view.held,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            }),
        HookPoint::Held => view
            .card
            .map(|card| {
                EvalContext::held(
                    view.hand_kind,
                    view.blind,
                    card,
                    view.played,
                    view.scoring,
                    view.held,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            })
            .unwrap_or_else(|| {
                EvalContext::independent(
                    view.hand_kind,
                    view.blind,
                    view.played,
                    view.scoring,
                    view.held,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            }),
        HookPoint::Discard => view
            .card
            .map(|card| {
                EvalContext::discard(
                    view.hand_kind,
                    view.blind,
                    card,
                    view.held,
                    view.discarded,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            })
            .unwrap_or_else(|| {
                EvalContext::discard_batch(
                    view.hand_kind,
                    view.blind,
                    view.held,
                    view.discarded,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            }),
        HookPoint::DiscardBatch => EvalContext::discard_batch(
            view.hand_kind,
            view.blind,
            view.held,
            view.discarded,
            run.state.hands_left,
            run.state.discards_left,
            joker_count,
        ),
        HookPoint::CardAdded => view
            .card
            .map(|card| {
                EvalContext::card_added(
                    view.hand_kind,
                    view.blind,
                    card,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            })
            .unwrap_or_else(|| {
                EvalContext::independent(
                    view.hand_kind,
                    view.blind,
                    view.played,
                    view.scoring,
                    view.held,
                    run.state.hands_left,
                    run.state.discards_left,
                    joker_count,
                )
            }),
        HookPoint::UseConsumable => match (view.consumable_kind, view.consumable_id) {
            (Some(kind), Some(id)) => EvalContext::consumable(
                view.hand_kind,
                view.blind,
                kind,
                id,
                run.state.hands_left,
                run.state.discards_left,
                joker_count,
            ),
            _ => EvalContext::independent(
                view.hand_kind,
                view.blind,
                view.played,
                view.scoring,
                view.held,
                run.state.hands_left,
                run.state.discards_left,
                joker_count,
            ),
        },
        HookPoint::Sell | HookPoint::AnySell => EvalContext::sell(
            view.hand_kind,
            view.blind,
            view.sold_value.unwrap_or(0),
            run.state.hands_left,
            run.state.discards_left,
            joker_count,
        ),
        _ => EvalContext::independent(
            view.hand_kind,
            view.blind,
            view.played,
            view.scoring,
            view.held,
            run.state.hands_left,
            run.state.discards_left,
            joker_count,
        ),
    }
}

fn apply_effect_list(
    run: &mut RunState,
    id: &str,
    effects: &[JokerEffect],
    trigger: ActivationType,
    ctx: &EvalContext<'_>,
    vars: &mut HashMap<String, f64>,
    args: &mut HookArgs<'_>,
    mut card_mut: Option<&mut Card>,
) -> bool {
    let mut triggered = false;
    let mut dummy = JokerInstance {
        id: id.to_string(),
        rarity: JokerRarity::Common,
        edition: None,
        stickers: JokerStickers::default(),
        buy_price: 0,
        vars: vars.clone(),
    };
    let ctx = ctx.with_joker_vars(&dummy.vars);
    for effect in effects {
        if effect.trigger != trigger {
            continue;
        }
        if !run.eval_bool(&effect.when, &ctx) {
            continue;
        }
        triggered = true;
        run.apply_actions(
            &mut dummy,
            &effect.actions,
            trigger,
            &ctx,
            card_mut.as_deref_mut(),
            args.score,
            args.money,
            args.results,
        );
    }
    *vars = dummy.vars;

    let added = run.flush_pending_joker_changes();
    if added > 0 {
        run.trigger_on_acquire();
    }

    triggered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_activation_bindings_cover_all_points() {
        assert!(hook_activation_bindings_valid());
        for point in HookPoint::ALL {
            assert!(
                hook_trigger_for(point).is_some(),
                "missing trigger for {point:?}"
            );
        }
    }

    #[test]
    fn hook_activation_bindings_keep_sell_semantics() {
        assert_eq!(
            hook_trigger_for(HookPoint::Sell),
            Some(ActivationType::OnSell)
        );
        assert_eq!(
            hook_trigger_for(HookPoint::AnySell),
            Some(ActivationType::OnAnySell)
        );
    }
}
