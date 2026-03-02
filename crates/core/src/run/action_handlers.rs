use super::context::{EvalContext, EvalValue};
use super::helpers::*;
use super::TriggerResults;
use crate::*;

pub(super) struct ActionContext<'a> {
    pub action: &'a Action,
    pub value: Option<f64>,
    pub evaluated: &'a EvalValue,
    pub trigger: ActivationType,
    pub eval_ctx: &'a EvalContext<'a>,
}

pub(super) fn dispatch_action(
    op: ActionOp,
    run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    score: &mut Score,
    money: &mut i64,
    results: &mut TriggerResults,
) {
    match op {
        ActionOp::AddChips => handle_add_chips(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddMult => handle_add_mult(run, joker, ctx, card_mut, score, money, results),
        ActionOp::MultiplyMult => {
            handle_multiply_mult(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::MultiplyChips => {
            handle_multiply_chips(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddMoney => handle_add_money(run, joker, ctx, card_mut, score, money, results),
        ActionOp::SetMoney => handle_set_money(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddHandSize => {
            handle_add_hand_size(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddHands => handle_add_hands(run, joker, ctx, card_mut, score, money, results),
        ActionOp::SetHands => handle_set_hands(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddDiscards => {
            handle_add_discards(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetDiscards => {
            handle_set_discards(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::RetriggerScored => {
            handle_retrigger_scored(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::RetriggerHeld => {
            handle_retrigger_held(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddStoneCard => {
            handle_add_stone_card(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddCardBonus => {
            handle_add_card_bonus(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetCardEnhancement => {
            handle_set_card_enhancement(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::ClearCardEnhancement => {
            handle_clear_card_enhancement(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DestroyCard => {
            handle_destroy_card(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::CopyPlayedCard => {
            handle_copy_played_card(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddTarot => handle_add_tarot(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddPlanet => {
            handle_add_planet(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddSpectral => {
            handle_add_spectral(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddFreeReroll => {
            handle_add_free_reroll(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetShopPrice => {
            handle_set_shop_price(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddJoker => handle_add_joker(run, joker, ctx, card_mut, score, money, results),
        ActionOp::DestroyRandomJoker => {
            handle_destroy_random_joker(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DestroyJokerRight => {
            handle_destroy_joker_right(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DestroyJokerLeft => {
            handle_destroy_joker_left(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DestroySelf => {
            handle_destroy_self(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::UpgradeHand => {
            handle_upgrade_hand(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DuplicateRandomJoker => {
            handle_duplicate_random_joker(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DuplicateRandomConsumable => {
            handle_duplicate_random_consumable(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddSellBonus => {
            handle_add_sell_bonus(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::DisableBoss => {
            handle_disable_boss(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddRandomHandCard => {
            handle_add_random_hand_card(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::CopyJokerRight => {
            handle_copy_joker_right(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::CopyJokerLeftmost => {
            handle_copy_joker_leftmost(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::PreventDeath => {
            handle_prevent_death(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddTag => handle_add_tag(run, joker, ctx, card_mut, score, money, results),
        ActionOp::DuplicateNextTag => {
            handle_duplicate_next_tag(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddPack => handle_add_pack(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddShopJoker => {
            handle_add_shop_joker(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::AddVoucher => {
            handle_add_voucher(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetRerollCost => {
            handle_set_reroll_cost(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetShopJokerEdition => {
            handle_set_shop_joker_edition(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::RerollBoss => {
            handle_reroll_boss(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::UpgradeRandomHand => {
            handle_upgrade_random_hand(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::MultiplyTarget => {
            handle_multiply_target(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetRule => handle_set_rule(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddRule => handle_add_rule(run, joker, ctx, card_mut, score, money, results),
        ActionOp::ClearRule => {
            handle_clear_rule(run, joker, ctx, card_mut, score, money, results)
        }
        ActionOp::SetVar => handle_set_var(run, joker, ctx, card_mut, score, money, results),
        ActionOp::AddVar => handle_add_var(run, joker, ctx, card_mut, score, money, results),
    }
}

pub(super) fn handle_add_chips(
    run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let source = format!("joker:{}:add_chips", joker.id);
        run.apply_rule_effect(score, RuleEffect::AddChips(value.floor() as i64), &source);
    }
}

pub(super) fn handle_add_mult(
    run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let source = format!("joker:{}:add_mult", joker.id);
        run.apply_rule_effect(score, RuleEffect::AddMult(value), &source);
    }
}

pub(super) fn handle_multiply_mult(
    run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let source = format!("joker:{}:mul_mult", joker.id);
        run.apply_rule_effect(score, RuleEffect::MultiplyMult(value), &source);
    }
}

pub(super) fn handle_multiply_chips(
    run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let source = format!("joker:{}:mul_chips", joker.id);
        run.apply_rule_effect(score, RuleEffect::MultiplyChips(value), &source);
    }
}

pub(super) fn handle_add_money(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        *money += value.floor() as i64;
    }
}

pub(super) fn handle_set_money(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        *money = value.floor() as i64;
    }
}

pub(super) fn handle_add_hand_size(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let next = (run.state.hand_size as f64 + value).max(0.0) as usize;
        run.state.hand_size = next;
    }
}

pub(super) fn handle_add_hands(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let delta = value.floor() as i64;
        if delta != 0 {
            let next = (run.state.hands_left as i64 + delta).max(0) as u8;
            run.state.hands_left = next;
        }
    }
}

pub(super) fn handle_set_hands(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let next = value.floor().max(0.0) as u8;
        run.state.hands_left = next;
    }
}

pub(super) fn handle_add_discards(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let delta = value.floor() as i64;
        if delta != 0 {
            let next = (run.state.discards_left as i64 + delta).max(0) as u8;
            run.state.discards_left = next;
        }
    }
}

pub(super) fn handle_set_discards(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let next = value.floor().max(0.0) as u8;
        run.state.discards_left = next;
    }
}

pub(super) fn handle_retrigger_scored(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        results.scored_retriggers += value.floor() as i64;
    }
}

pub(super) fn handle_retrigger_held(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        results.held_retriggers += value.floor() as i64;
    }
}

pub(super) fn handle_add_stone_card(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    if count == 0 {
        return;
    }
    for _ in 0..count {
        let mut card = run.content.random_standard_card(&mut run.rng);
        card.enhancement = Some(Enhancement::Stone);
        run.assign_card_id(&mut card);
        run.deck.draw.push(card);
        run.trigger_on_card_added(card);
    }
    run.deck.shuffle(&mut run.rng);
}

pub(super) fn handle_add_card_bonus(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(card), Some(value)) = (card_mut, ctx.value) {
        card.bonus_chips = card.bonus_chips.saturating_add(value.floor() as i64);
    }
}

pub(super) fn handle_set_card_enhancement(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let Some(card) = card_mut else {
        return;
    };
    let Some(target) = ctx.action.target.as_deref() else {
        return;
    };
    let norm = normalize(target);
    if norm == "none" || norm == "clear" {
        card.enhancement = None;
    } else if let Some(kind) = enhancement_from_str(&norm) {
        card.enhancement = Some(kind);
    }
}

pub(super) fn handle_clear_card_enhancement(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    _ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(card) = card_mut {
        card.enhancement = None;
    }
}

pub(super) fn handle_destroy_card(
    _run: &mut super::RunState,
    _joker: &mut JokerInstance,
    _ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    results: &mut TriggerResults,
) {
    results.destroyed_current = true;
}

pub(super) fn handle_copy_played_card(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let card = if let Some(card) = ctx.eval_ctx.card {
        Some(card)
    } else if ctx.eval_ctx.played_cards.len() == 1 {
        ctx.eval_ctx.played_cards.first().copied()
    } else {
        None
    };
    if let Some(card) = card {
        let mut copy = card;
        copy.face_down = false;
        run.assign_card_id(&mut copy);
        run.hand.push(copy);
        run.trigger_on_card_added(copy);
    }
}

pub(super) fn handle_add_tarot(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        if let Some(card) = run
            .content
            .pick_consumable(ConsumableKind::Tarot, &mut run.rng)
        {
            let _ = run
                .inventory
                .add_consumable(card.id.clone(), ConsumableKind::Tarot);
        }
    }
}

pub(super) fn handle_add_planet(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        if let Some(card) = run
            .content
            .pick_consumable(ConsumableKind::Planet, &mut run.rng)
        {
            let _ = run
                .inventory
                .add_consumable(card.id.clone(), ConsumableKind::Planet);
        }
    }
}

pub(super) fn handle_add_spectral(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        if let Some(card) = run
            .content
            .pick_consumable(ConsumableKind::Spectral, &mut run.rng)
        {
            let _ = run
                .inventory
                .add_consumable(card.id.clone(), ConsumableKind::Spectral);
        }
    }
}

pub(super) fn handle_add_free_reroll(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let delta = value.floor() as i64;
        if delta >= 0 {
            let added = delta.min(u8::MAX as i64) as u8;
            run.state.shop_free_rerolls = run.state.shop_free_rerolls.saturating_add(added);
        } else {
            let sub = (-delta).min(run.state.shop_free_rerolls as i64) as u8;
            run.state.shop_free_rerolls = run.state.shop_free_rerolls.saturating_sub(sub);
        }
    }
}

pub(super) fn handle_set_shop_price(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(target), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        let price = value.floor().max(0.0) as i64;
        run.apply_shop_price_override(target, price);
    }
}

pub(super) fn handle_add_joker(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
        for _ in 0..count {
            if let Some(new_joker) = run.spawn_joker_from_target(target) {
                run.pending_joker_additions.push(new_joker);
            }
        }
    }
}

pub(super) fn handle_destroy_random_joker(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    run.queue_destroy_random(ctx.eval_ctx.joker_index);
}

pub(super) fn handle_destroy_joker_right(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(index) = ctx.eval_ctx.joker_index {
        run.queue_destroy_neighbor(index, 1);
    }
}

pub(super) fn handle_destroy_joker_left(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(index) = ctx.eval_ctx.joker_index {
        run.queue_destroy_neighbor(index, -1);
    }
}

pub(super) fn handle_destroy_self(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(index) = ctx.eval_ctx.joker_index {
        run.queue_joker_removal(index);
    }
}

pub(super) fn handle_upgrade_hand(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(hand_str) = ctx.evaluated.as_string() {
        let norm = normalize(hand_str);
        if norm == "all" || norm == "any" {
            run.upgrade_all_hands(1);
        } else if let Some(hand) = hand_kind_from_str(&norm) {
            run.upgrade_hand_level(hand, 1);
        }
    } else if let Some(levels) = ctx.evaluated.as_number() {
        let amount = levels.floor().max(0.0) as u32;
        run.upgrade_hand_level(ctx.eval_ctx.hand_kind, amount);
    }
}

pub(super) fn handle_duplicate_random_joker(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        if run.inventory.jokers.len() >= run.inventory.joker_capacity() {
            break;
        }
        if run.inventory.jokers.is_empty() {
            break;
        }
        let idx = (run.rng.next_u64() % run.inventory.jokers.len() as u64) as usize;
        let mut copy = run.inventory.jokers[idx].clone();
        if copy.edition == Some(Edition::Negative) {
            copy.edition = None;
        }
        run.inventory.jokers.push(copy);
        run.mark_rules_dirty();
    }
}

pub(super) fn handle_duplicate_random_consumable(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        if run.inventory.consumables.is_empty() {
            break;
        }
        let idx = (run.rng.next_u64() % run.inventory.consumables.len() as u64) as usize;
        if let Some(existing) = run.inventory.consumables.get(idx).cloned() {
            let _ = run.inventory.add_consumable_with_edition(
                existing.id,
                existing.kind,
                Some(Edition::Negative),
                existing.sell_bonus,
            );
        }
    }
}

pub(super) fn handle_add_sell_bonus(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(target), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        let delta = value;
        let target = normalize(target);
        if target == "all" || target == "everything" || target == "jokers" {
            for j in &mut run.inventory.jokers {
                let entry = j.vars.entry("sell_bonus".into()).or_insert(0.0);
                *entry += delta;
            }
        }
        if target == "all" || target == "everything" || target == "consumables" {
            for consumable in &mut run.inventory.consumables {
                consumable.sell_bonus += delta;
            }
        }
    }
}

pub(super) fn handle_disable_boss(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    _ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if run.state.phase == Phase::Shop {
        run.boss_disable_pending = true;
    } else {
        run.boss_disabled = true;
    }
}

pub(super) fn handle_add_random_hand_card(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
    for _ in 0..count {
        let mut card = run.content.random_standard_card(&mut run.rng);
        let roll = run.rng.next_u64() % 4;
        card.seal = match roll {
            0 => Some(Seal::Red),
            1 => Some(Seal::Blue),
            2 => Some(Seal::Gold),
            _ => Some(Seal::Purple),
        };
        run.assign_card_id(&mut card);
        run.hand.push(card);
        run.trigger_on_card_added(card);
    }
}

pub(super) fn handle_copy_joker_right(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    score: &mut Score,
    money: &mut i64,
    results: &mut TriggerResults,
) {
    if let Some(index) = ctx.eval_ctx.joker_index {
        if let Some(target) = run.neighbor_index(index, 1) {
            if target != index {
                run.apply_joker_copy_from(
                    target,
                    ctx.trigger,
                    ctx.eval_ctx,
                    card_mut,
                    score,
                    money,
                    results,
                );
            }
        }
    }
}

pub(super) fn handle_copy_joker_leftmost(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    card_mut: Option<&mut Card>,
    score: &mut Score,
    money: &mut i64,
    results: &mut TriggerResults,
) {
    if let Some(target) = run.leftmost_joker_index() {
        if Some(target) != ctx.eval_ctx.joker_index {
            run.apply_joker_copy_from(
                target,
                ctx.trigger,
                ctx.eval_ctx,
                card_mut,
                score,
                money,
                results,
            );
        }
    }
}

pub(super) fn handle_prevent_death(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if ctx.value.unwrap_or(1.0) != 0.0 {
        run.prevent_death = true;
    }
}

pub(super) fn handle_add_tag(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        let count = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
        for _ in 0..count {
            let tag_id = normalize(target);
            let should_duplicate = run.state.duplicate_next_tag
                && run
                    .state
                    .duplicate_tag_exclude
                    .as_deref()
                    .map(|ex| ex != tag_id)
                    .unwrap_or(true);
            if should_duplicate {
                run.state.tags.push(tag_id.clone());
                run.state.tags.push(tag_id.clone());
                run.state.duplicate_next_tag = false;
                run.state.duplicate_tag_exclude = None;
            } else {
                run.state.tags.push(tag_id);
            }
        }
        run.mark_rules_dirty();
        run.trigger_on_acquire();
    }
}

pub(super) fn handle_duplicate_next_tag(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        run.state.duplicate_next_tag = true;
        run.state.duplicate_tag_exclude = Some(normalize(target));
    } else {
        run.state.duplicate_next_tag = true;
        run.state.duplicate_tag_exclude = None;
    }
}

pub(super) fn handle_add_pack(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        let price = ctx.value.map(|v| v.floor() as i64);
        run.add_pack_offer_from_target(target, price);
    }
}

pub(super) fn handle_add_shop_joker(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        let price = ctx.value.map(|v| v.floor() as i64);
        run.add_shop_joker_offer(target, price);
    }
}

pub(super) fn handle_add_voucher(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if run.shop.is_some() {
        let delta = ctx.value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
        for _ in 0..delta {
            let offer = run.voucher_offer_for_shop();
            if let Some(shop) = run.shop.as_mut() {
                shop.add_voucher_offer(offer);
            }
        }
    }
}

pub(super) fn handle_set_reroll_cost(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(shop) = run.shop.as_mut() {
        let next = ctx.value.map(|v| v.floor() as i64).unwrap_or(0).max(0);
        shop.reroll_cost = next;
    }
}

pub(super) fn handle_set_shop_joker_edition(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(target) = ctx.action.target.as_deref() {
        let price = ctx.value.map(|v| v.floor() as i64);
        run.set_shop_joker_edition(target, price);
    }
}

pub(super) fn handle_reroll_boss(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    _ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    run.reroll_boss();
}

pub(super) fn handle_upgrade_random_hand(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    let amount = ctx.value.map(|v| v.floor().max(0.0) as u32).unwrap_or(1);
    run.upgrade_random_hand(amount);
}

pub(super) fn handle_multiply_target(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(value) = ctx.value {
        let scaled = (run.state.target as f64 * value).floor() as i64;
        run.state.target = scaled.max(0);
    }
}

pub(super) fn handle_set_rule(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(key), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        run.set_rule_var(key, value);
    }
}

pub(super) fn handle_add_rule(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(key), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        run.add_rule_var(key, value);
    }
}

pub(super) fn handle_clear_rule(
    run: &mut super::RunState,
    _joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let Some(key) = ctx.action.target.as_deref() {
        run.set_rule_var(key, 0.0);
    }
}

pub(super) fn handle_set_var(
    _run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(key), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        joker.vars.insert(normalize(key), value);
    }
}

pub(super) fn handle_add_var(
    _run: &mut super::RunState,
    joker: &mut JokerInstance,
    ctx: &ActionContext<'_>,
    _card_mut: Option<&mut Card>,
    _score: &mut Score,
    _money: &mut i64,
    _results: &mut TriggerResults,
) {
    if let (Some(key), Some(value)) = (ctx.action.target.as_deref(), ctx.value) {
        let entry = joker.vars.entry(normalize(key)).or_insert(0.0);
        *entry += value;
    }
}
