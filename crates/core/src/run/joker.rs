use super::helpers::*;
use super::*;
use crate::*;
use std::collections::HashMap;

impl RunState {
    pub(super) fn build_joker_snapshot(&mut self) {
        self.current_joker_snapshot = self
            .inventory
            .jokers
            .iter()
            .enumerate()
            .map(|(index, _)| JokerSnapshot { index })
            .collect();
        self.current_joker_counts = build_joker_counts(&self.inventory.jokers);
        self.pending_joker_removals.clear();
        self.pending_joker_additions.clear();
    }

    pub(super) fn joker_indices_in_order(&self) -> Vec<usize> {
        self.current_joker_snapshot
            .iter()
            .map(|snap| snap.index)
            .filter(|idx| !self.pending_joker_removals.contains(idx))
            .collect()
    }

    pub(super) fn queue_joker_removal(&mut self, index: usize) {
        if index < self.inventory.jokers.len() {
            self.pending_joker_removals.push(index);
        }
    }

    pub(super) fn queue_destroy_neighbor(&mut self, current: usize, direction: i32) {
        let indices = self.joker_indices_in_order();
        let pos = indices.iter().position(|&idx| idx == current);
        if let Some(pos) = pos {
            let target = if direction < 0 {
                pos.checked_sub(1).and_then(|p| indices.get(p).copied())
            } else {
                indices.get(pos + 1).copied()
            };
            if let Some(target) = target {
                self.last_destroyed_sell_value = self
                    .inventory
                    .jokers
                    .get(target)
                    .map(|joker| self.calc_joker_sell_value(joker))
                    .unwrap_or(0);
                self.queue_joker_removal(target);
            } else {
                self.last_destroyed_sell_value = 0;
            }
        } else {
            self.last_destroyed_sell_value = 0;
        }
    }

    pub(super) fn neighbor_index(&self, current: usize, direction: i32) -> Option<usize> {
        let indices = self.joker_indices_in_order();
        let pos = indices.iter().position(|&idx| idx == current)?;
        if direction < 0 {
            pos.checked_sub(1).and_then(|p| indices.get(p).copied())
        } else {
            indices.get(pos + 1).copied()
        }
    }

    pub(super) fn queue_destroy_random(&mut self, exclude: Option<usize>) {
        let mut indices = self.joker_indices_in_order();
        if let Some(exclude) = exclude {
            indices.retain(|idx| *idx != exclude);
        }
        if indices.is_empty() {
            self.last_destroyed_sell_value = 0;
            return;
        }
        let pick = (self.rng.next_u64() % indices.len() as u64) as usize;
        if let Some(joker) = self.inventory.jokers.get(indices[pick]) {
            self.last_destroyed_sell_value = self.calc_joker_sell_value(joker);
        } else {
            self.last_destroyed_sell_value = 0;
        }
        self.queue_joker_removal(indices[pick]);
    }

    pub(super) fn leftmost_joker_index(&self) -> Option<usize> {
        self.joker_indices_in_order().into_iter().next()
    }

    fn apply_joker_copy_from(
        &mut self,
        source_index: usize,
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
        card_mut: Option<&mut Card>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
        if self.copy_depth >= 8 {
            return;
        }
        if self.copy_stack.contains(&source_index) {
            return;
        }
        let Some(existing) = self.inventory.jokers.get(source_index).cloned() else {
            return;
        };
        self.copy_depth = self.copy_depth.saturating_add(1);
        self.copy_stack.push(source_index);

        let mut joker = existing;
        let ctx = ctx
            .with_joker_vars(&joker.vars)
            .with_joker_index(source_index);
        self.apply_joker_effects_for_joker(
            &mut joker, trigger, &ctx, card_mut, score, money, results,
        );
        if !self.pending_joker_removals.contains(&source_index) {
            if let Some(slot) = self.inventory.jokers.get_mut(source_index) {
                *slot = joker;
            }
        }

        self.copy_stack.pop();
        self.copy_depth = self.copy_depth.saturating_sub(1);
    }

    pub(super) fn flush_pending_joker_changes(&mut self) -> usize {
        let mut changed = false;
        if !self.pending_joker_removals.is_empty() {
            self.pending_joker_removals.sort_unstable();
            self.pending_joker_removals.dedup();
            for index in self.pending_joker_removals.iter().rev() {
                if *index < self.inventory.jokers.len() {
                    self.inventory.jokers.remove(*index);
                    changed = true;
                }
            }
        }
        self.pending_joker_removals.clear();

        let mut added = 0;
        if !self.pending_joker_additions.is_empty() {
            for joker in self.pending_joker_additions.drain(..) {
                if self.inventory.jokers.len() >= self.inventory.joker_capacity() {
                    break;
                }
                self.inventory.jokers.push(joker);
                added += 1;
            }
            if added > 0 {
                changed = true;
            }
        }
        if changed {
            self.mark_rules_dirty();
        }
        added
    }

    pub(super) fn trigger_on_acquire(&mut self) {
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut money = self.state.money;
        let mut results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::independent(
            hand_kind,
            self.state.blind,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        let mut events = EventBus::default();
        self.invoke_hooks(HookPoint::Acquire, &mut args, &mut events);
        self.state.money = money;
    }

    pub(super) fn trigger_on_card_added(&mut self, card: crate::Card) {
        if self.joker_effect_depth > 0 {
            self.deferred_card_added.push(card);
            return;
        }
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut money = self.state.money;
        let mut results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::card_added(
            hand_kind,
            self.state.blind,
            card,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        let mut events = EventBus::default();
        self.invoke_hooks(HookPoint::CardAdded, &mut args, &mut events);
        self.state.money = money;
    }

    pub(super) fn other_joker_sell_value(&self, current_index: Option<usize>) -> i64 {
        self.inventory
            .jokers
            .iter()
            .enumerate()
            .filter(|(idx, _)| Some(*idx) != current_index)
            .map(|(_, joker)| self.calc_joker_sell_value(joker))
            .sum()
    }

    pub(super) fn add_joker_from_rarity(&mut self, rarity: crate::JokerRarity) {
        let Some(def) = self.content.pick_joker(rarity, &mut self.rng) else {
            return;
        };
        if self.inventory.jokers.len() >= self.inventory.joker_capacity() {
            return;
        }
        let _ = self
            .inventory
            .add_joker_with_edition(def.id.clone(), rarity, 0, None);
        self.mark_rules_dirty();
        self.trigger_on_acquire();
    }

    pub(super) fn add_joker_with_trigger(
        &mut self,
        id: String,
        rarity: crate::JokerRarity,
        buy_price: i64,
    ) -> Result<(), RunError> {
        self.add_joker_with_trigger_edition(id, rarity, buy_price, None)
    }

    pub(super) fn add_joker_with_trigger_edition(
        &mut self,
        id: String,
        rarity: crate::JokerRarity,
        buy_price: i64,
        edition: Option<Edition>,
    ) -> Result<(), RunError> {
        self.inventory
            .add_joker_with_edition(id, rarity, buy_price, edition)?;
        self.mark_rules_dirty();
        self.trigger_on_acquire();
        Ok(())
    }

    pub(super) fn spawn_joker_from_target(&mut self, target: &str) -> Option<crate::JokerInstance> {
        let norm = normalize(target);
        let (id, rarity) = match norm.as_str() {
            "common" => {
                let def = self
                    .content
                    .pick_joker(crate::JokerRarity::Common, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Common)
            }
            "uncommon" => {
                let def = self
                    .content
                    .pick_joker(crate::JokerRarity::Uncommon, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Uncommon)
            }
            "rare" => {
                let def = self
                    .content
                    .pick_joker(crate::JokerRarity::Rare, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Rare)
            }
            "legendary" => {
                let def = self
                    .content
                    .pick_joker(crate::JokerRarity::Legendary, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Legendary)
            }
            "random" => {
                let idx = (self.rng.next_u64() % self.content.jokers.len() as u64) as usize;
                let def = self.content.jokers.get(idx)?;
                (def.id.clone(), def.rarity)
            }
            _ => {
                let def = self
                    .content
                    .jokers
                    .iter()
                    .find(|j| normalize(&j.id) == norm || normalize(&j.name) == norm)?;
                (def.id.clone(), def.rarity)
            }
        };

        let buy_price = self.default_joker_price(rarity);
        Some(crate::JokerInstance {
            id,
            rarity,
            edition: None,
            stickers: crate::JokerStickers::default(),
            buy_price,
            vars: HashMap::new(),
        })
    }

    pub(super) fn apply_joker_editions_and_independent(
        &mut self,
        score: &mut Score,
        money: &mut i64,
        hand_kind: crate::HandKind,
        played_cards: &[Card],
        scoring_cards: &[Card],
        held_cards: &[Card],
        joker_count: usize,
    ) {
        let base_ctx = EvalContext::independent(
            hand_kind,
            self.state.blind,
            played_cards,
            scoring_cards,
            held_cards,
            self.state.hands_left,
            self.state.discards_left,
            joker_count,
        );
        self.build_joker_snapshot();
        let snapshot = self.current_joker_snapshot.clone();
        for snap in &snapshot {
            if self.pending_joker_removals.contains(&snap.index) {
                continue;
            }
            let Some(existing) = self.inventory.jokers.get(snap.index).cloned() else {
                continue;
            };
            let mut joker = existing;
            self.apply_joker_edition_before(&joker, score);
            let mut results = TriggerResults::default();
            let ctx = base_ctx
                .with_joker_vars(&joker.vars)
                .with_joker_index(snap.index);
            self.apply_joker_effects_for_joker(
                &mut joker,
                ActivationType::Independent,
                &ctx,
                None,
                score,
                money,
                &mut results,
            );
            if !self.pending_joker_removals.contains(&snap.index) {
                self.apply_joker_edition_after(&joker, score);
                if let Some(slot) = self.inventory.jokers.get_mut(snap.index) {
                    *slot = joker;
                }
            }
        }
        let added = self.flush_pending_joker_changes();
        if added > 0 {
            self.trigger_on_acquire();
        }
    }

    pub(super) fn apply_joker_edition_before(
        &mut self,
        joker: &crate::JokerInstance,
        score: &mut Score,
    ) {
        match joker.edition {
            Some(Edition::Foil) => {
                let chips = self.tables.card_attrs.edition("foil").chips;
                self.apply_rule_effect(
                    score,
                    crate::RuleEffect::AddChips(chips),
                    "joker_edition:foil",
                )
            }
            Some(Edition::Holographic) => {
                let mult = self.tables.card_attrs.edition("holographic").mult_add;
                self.apply_rule_effect(
                    score,
                    crate::RuleEffect::AddMult(mult),
                    "joker_edition:holographic",
                )
            }
            _ => {}
        }
    }

    pub(super) fn apply_joker_edition_after(
        &mut self,
        joker: &crate::JokerInstance,
        score: &mut Score,
    ) {
        match joker.edition {
            Some(Edition::Polychrome) => {
                let mul = self.tables.card_attrs.edition("polychrome").mult_mul;
                self.apply_rule_effect(
                    score,
                    crate::RuleEffect::MultiplyMult(mul),
                    "joker_edition:polychrome",
                )
            }
            _ => {}
        }
    }

    pub(super) fn apply_joker_effects(
        &mut self,
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
        card_mut: Option<&mut Card>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
        self.joker_effect_depth = self.joker_effect_depth.saturating_add(1);
        self.last_destroyed_sell_value = 0;
        self.copy_depth = 0;
        self.copy_stack.clear();
        self.build_joker_snapshot();
        let snapshot = self.current_joker_snapshot.clone();
        let mut card_ref = card_mut;
        for snap in &snapshot {
            if self.pending_joker_removals.contains(&snap.index) {
                continue;
            }
            let Some(existing) = self.inventory.jokers.get(snap.index).cloned() else {
                continue;
            };
            let mut joker = existing;
            let ctx = ctx
                .with_joker_vars(&joker.vars)
                .with_joker_index(snap.index);
            let card_mut = card_ref.as_deref_mut();
            self.apply_joker_effects_for_joker(
                &mut joker, trigger, &ctx, card_mut, score, money, results,
            );
            if !self.pending_joker_removals.contains(&snap.index) {
                if let Some(slot) = self.inventory.jokers.get_mut(snap.index) {
                    *slot = joker;
                }
            }
        }
        let added = self.flush_pending_joker_changes();
        if added > 0 {
            self.trigger_on_acquire();
        }
        self.joker_effect_depth = self.joker_effect_depth.saturating_sub(1);
        if self.joker_effect_depth == 0 && !self.deferred_card_added.is_empty() {
            let pending = self.deferred_card_added.drain(..).collect::<Vec<_>>();
            for card in pending {
                self.trigger_on_card_added(card);
            }
        }
    }

    pub(super) fn apply_joker_effects_for_joker(
        &mut self,
        joker: &mut crate::JokerInstance,
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
        mut card_mut: Option<&mut Card>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
        let effects = self
            .content
            .jokers
            .iter()
            .find(|j| j.id == joker.id)
            .map(|def| def.effects.clone());
        if let Some(effects) = effects {
            for effect in &effects {
                if effect.trigger != trigger {
                    continue;
                }
                if !self.eval_bool(&effect.when, ctx) {
                    continue;
                }
                self.apply_actions(
                    joker,
                    &effect.actions,
                    trigger,
                    ctx,
                    card_mut.as_deref_mut(),
                    score,
                    money,
                    results,
                );
            }
        }
    }

    pub(super) fn apply_actions(
        &mut self,
        joker: &mut crate::JokerInstance,
        actions: &[crate::Action],
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
        mut card_mut: Option<&mut Card>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
        for action in actions {
            if trigger == ActivationType::Passive {
                let is_rule_op = matches!(
                    &action.op,
                    crate::ActionOpKind::Builtin(
                        ActionOp::SetRule | ActionOp::AddRule | ActionOp::ClearRule
                    )
                );
                if !is_rule_op {
                    continue;
                }
            }
            let evaluated = self.eval_expr(&action.value, ctx);
            let value = evaluated.as_number();
            match &action.op {
                crate::ActionOpKind::Custom(name) => {
                    if let Some(rt) = self.mod_runtime.as_mut() {
                        let effect_ctx = crate::ModEffectContext {
                            state: &self.state,
                            hand_kind: Some(ctx.hand_kind),
                            card: ctx.card,
                            joker_id: Some(&joker.id),
                        };
                        let mod_val = evaluated.as_number().unwrap_or(0.0);
                        let result =
                            rt.invoke_effect(name, action.target.as_deref(), mod_val, &effect_ctx);
                        self.apply_mod_action_result(&result, joker, score, money);
                    }
                }
                crate::ActionOpKind::Builtin(op) => match op {
                    ActionOp::AddChips => {
                        if let Some(value) = value {
                            let source = format!("joker:{}:add_chips", joker.id);
                            self.apply_rule_effect(
                                score,
                                crate::RuleEffect::AddChips(value.floor() as i64),
                                &source,
                            );
                        }
                    }
                    ActionOp::AddMult => {
                        if let Some(value) = value {
                            let source = format!("joker:{}:add_mult", joker.id);
                            self.apply_rule_effect(
                                score,
                                crate::RuleEffect::AddMult(value),
                                &source,
                            );
                        }
                    }
                    ActionOp::MultiplyMult => {
                        if let Some(value) = value {
                            let source = format!("joker:{}:mul_mult", joker.id);
                            self.apply_rule_effect(
                                score,
                                crate::RuleEffect::MultiplyMult(value),
                                &source,
                            );
                        }
                    }
                    ActionOp::MultiplyChips => {
                        if let Some(value) = value {
                            let source = format!("joker:{}:mul_chips", joker.id);
                            self.apply_rule_effect(
                                score,
                                crate::RuleEffect::MultiplyChips(value),
                                &source,
                            );
                        }
                    }
                    ActionOp::AddMoney => {
                        if let Some(value) = value {
                            *money += value.floor() as i64;
                        }
                    }
                    ActionOp::SetMoney => {
                        if let Some(value) = value {
                            *money = value.floor() as i64;
                        }
                    }
                    ActionOp::AddHandSize => {
                        if let Some(value) = value {
                            let next = (self.state.hand_size as f64 + value).max(0.0) as usize;
                            self.state.hand_size = next;
                        }
                    }
                    ActionOp::AddHands => {
                        if let Some(value) = value {
                            let delta = value.floor() as i64;
                            if delta != 0 {
                                let next = (self.state.hands_left as i64 + delta).max(0) as u8;
                                self.state.hands_left = next;
                            }
                        }
                    }
                    ActionOp::SetHands => {
                        if let Some(value) = value {
                            let next = value.floor().max(0.0) as u8;
                            self.state.hands_left = next;
                        }
                    }
                    ActionOp::AddDiscards => {
                        if let Some(value) = value {
                            let delta = value.floor() as i64;
                            if delta != 0 {
                                let next = (self.state.discards_left as i64 + delta).max(0) as u8;
                                self.state.discards_left = next;
                            }
                        }
                    }
                    ActionOp::SetDiscards => {
                        if let Some(value) = value {
                            let next = value.floor().max(0.0) as u8;
                            self.state.discards_left = next;
                        }
                    }
                    ActionOp::RetriggerScored => {
                        if let Some(value) = value {
                            results.scored_retriggers += value.floor() as i64;
                        }
                    }
                    ActionOp::RetriggerHeld => {
                        if let Some(value) = value {
                            results.held_retriggers += value.floor() as i64;
                        }
                    }
                    ActionOp::AddStoneCard => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        if count == 0 {
                            continue;
                        }
                        for _ in 0..count {
                            let mut card = self.content.random_standard_card(&mut self.rng);
                            card.enhancement = Some(Enhancement::Stone);
                            self.assign_card_id(&mut card);
                            self.deck.draw.push(card);
                            self.trigger_on_card_added(card);
                        }
                        self.deck.shuffle(&mut self.rng);
                    }
                    ActionOp::AddCardBonus => {
                        if let (Some(card), Some(value)) = (card_mut.as_deref_mut(), value) {
                            card.bonus_chips =
                                card.bonus_chips.saturating_add(value.floor() as i64);
                        }
                    }
                    ActionOp::SetCardEnhancement => {
                        let Some(card) = card_mut.as_deref_mut() else {
                            continue;
                        };
                        let Some(target) = action.target.as_deref() else {
                            continue;
                        };
                        let norm = normalize(target);
                        if norm == "none" || norm == "clear" {
                            card.enhancement = None;
                        } else if let Some(kind) = enhancement_from_str(&norm) {
                            card.enhancement = Some(kind);
                        }
                    }
                    ActionOp::ClearCardEnhancement => {
                        if let Some(card) = card_mut.as_deref_mut() {
                            card.enhancement = None;
                        }
                    }
                    ActionOp::DestroyCard => {
                        results.destroyed_current = true;
                    }
                    ActionOp::CopyPlayedCard => {
                        let card = if let Some(card) = ctx.card {
                            Some(card)
                        } else if ctx.played_cards.len() == 1 {
                            ctx.played_cards.first().copied()
                        } else {
                            None
                        };
                        if let Some(card) = card {
                            let mut copy = card;
                            copy.face_down = false;
                            self.assign_card_id(&mut copy);
                            self.hand.push(copy);
                            self.trigger_on_card_added(copy);
                        }
                    }
                    ActionOp::AddTarot => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            if let Some(card) = self
                                .content
                                .pick_consumable(crate::ConsumableKind::Tarot, &mut self.rng)
                            {
                                let _ = self
                                    .inventory
                                    .add_consumable(card.id.clone(), crate::ConsumableKind::Tarot);
                            }
                        }
                    }
                    ActionOp::AddPlanet => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            if let Some(card) = self
                                .content
                                .pick_consumable(crate::ConsumableKind::Planet, &mut self.rng)
                            {
                                let _ = self
                                    .inventory
                                    .add_consumable(card.id.clone(), crate::ConsumableKind::Planet);
                            }
                        }
                    }
                    ActionOp::AddSpectral => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            if let Some(card) = self
                                .content
                                .pick_consumable(crate::ConsumableKind::Spectral, &mut self.rng)
                            {
                                let _ = self.inventory.add_consumable(
                                    card.id.clone(),
                                    crate::ConsumableKind::Spectral,
                                );
                            }
                        }
                    }
                    ActionOp::AddFreeReroll => {
                        if let Some(value) = value {
                            let delta = value.floor() as i64;
                            if delta >= 0 {
                                let added = delta.min(u8::MAX as i64) as u8;
                                self.state.shop_free_rerolls =
                                    self.state.shop_free_rerolls.saturating_add(added);
                            } else {
                                let sub = (-delta).min(self.state.shop_free_rerolls as i64) as u8;
                                self.state.shop_free_rerolls =
                                    self.state.shop_free_rerolls.saturating_sub(sub);
                            }
                        }
                    }
                    ActionOp::SetShopPrice => {
                        if let (Some(target), Some(value)) = (action.target.as_deref(), value) {
                            let price = value.floor().max(0.0) as i64;
                            self.apply_shop_price_override(target, price);
                        }
                    }
                    ActionOp::AddJoker => {
                        if let Some(target) = action.target.as_deref() {
                            let count =
                                value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                            for _ in 0..count {
                                if let Some(joker) = self.spawn_joker_from_target(target) {
                                    self.pending_joker_additions.push(joker);
                                }
                            }
                        }
                    }
                    ActionOp::DestroyRandomJoker => {
                        self.queue_destroy_random(ctx.joker_index);
                    }
                    ActionOp::DestroyJokerRight => {
                        if let Some(index) = ctx.joker_index {
                            self.queue_destroy_neighbor(index, 1);
                        }
                    }
                    ActionOp::DestroyJokerLeft => {
                        if let Some(index) = ctx.joker_index {
                            self.queue_destroy_neighbor(index, -1);
                        }
                    }
                    ActionOp::DestroySelf => {
                        if let Some(index) = ctx.joker_index {
                            self.queue_joker_removal(index);
                        }
                    }
                    ActionOp::UpgradeHand => {
                        if let Some(hand_str) = evaluated.as_string() {
                            let norm = normalize(hand_str);
                            if norm == "all" || norm == "any" {
                                self.upgrade_all_hands(1);
                            } else if let Some(hand) = hand_kind_from_str(&norm) {
                                self.upgrade_hand_level(hand, 1);
                            }
                        } else if let Some(levels) = evaluated.as_number() {
                            let amount = levels.floor().max(0.0) as u32;
                            self.upgrade_hand_level(ctx.hand_kind, amount);
                        }
                    }
                    ActionOp::DuplicateRandomJoker => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            if self.inventory.jokers.len() >= self.inventory.joker_capacity() {
                                break;
                            }
                            if self.inventory.jokers.is_empty() {
                                break;
                            }
                            let idx =
                                (self.rng.next_u64() % self.inventory.jokers.len() as u64) as usize;
                            let mut copy = self.inventory.jokers[idx].clone();
                            if copy.edition == Some(Edition::Negative) {
                                copy.edition = None;
                            }
                            self.inventory.jokers.push(copy);
                            self.mark_rules_dirty();
                        }
                    }
                    ActionOp::DuplicateRandomConsumable => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            if self.inventory.consumables.is_empty() {
                                break;
                            }
                            let idx = (self.rng.next_u64()
                                % self.inventory.consumables.len() as u64)
                                as usize;
                            if let Some(existing) = self.inventory.consumables.get(idx).cloned() {
                                let _ = self.inventory.add_consumable_with_edition(
                                    existing.id,
                                    existing.kind,
                                    Some(Edition::Negative),
                                    existing.sell_bonus,
                                );
                            }
                        }
                    }
                    ActionOp::AddSellBonus => {
                        if let (Some(target), Some(value)) = (action.target.as_deref(), value) {
                            let delta = value;
                            let target = normalize(target);
                            if target == "all" || target == "everything" || target == "jokers" {
                                for joker in &mut self.inventory.jokers {
                                    let entry =
                                        joker.vars.entry("sell_bonus".into()).or_insert(0.0);
                                    *entry += delta;
                                }
                            }
                            if target == "all" || target == "everything" || target == "consumables"
                            {
                                for consumable in &mut self.inventory.consumables {
                                    consumable.sell_bonus += delta;
                                }
                            }
                        }
                    }
                    ActionOp::DisableBoss => {
                        if self.state.phase == Phase::Shop {
                            self.boss_disable_pending = true;
                        } else {
                            self.boss_disabled = true;
                        }
                    }
                    ActionOp::AddRandomHandCard => {
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                        for _ in 0..count {
                            let mut card = self.content.random_standard_card(&mut self.rng);
                            let roll = self.rng.next_u64() % 4;
                            card.seal = match roll {
                                0 => Some(Seal::Red),
                                1 => Some(Seal::Blue),
                                2 => Some(Seal::Gold),
                                _ => Some(Seal::Purple),
                            };
                            self.assign_card_id(&mut card);
                            self.hand.push(card);
                            self.trigger_on_card_added(card);
                        }
                    }
                    ActionOp::CopyJokerRight => {
                        if let Some(index) = ctx.joker_index {
                            if let Some(target) = self.neighbor_index(index, 1) {
                                if target != index {
                                    let card_mut = card_mut.as_deref_mut();
                                    self.apply_joker_copy_from(
                                        target, trigger, ctx, card_mut, score, money, results,
                                    );
                                }
                            }
                        }
                    }
                    ActionOp::CopyJokerLeftmost => {
                        if let Some(target) = self.leftmost_joker_index() {
                            if Some(target) != ctx.joker_index {
                                let card_mut = card_mut.as_deref_mut();
                                self.apply_joker_copy_from(
                                    target, trigger, ctx, card_mut, score, money, results,
                                );
                            }
                        }
                    }
                    ActionOp::PreventDeath => {
                        if value.unwrap_or(1.0) != 0.0 {
                            self.prevent_death = true;
                        }
                    }
                    ActionOp::AddTag => {
                        if let Some(target) = action.target.as_deref() {
                            let count =
                                value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                            for _ in 0..count {
                                let tag_id = normalize(target);
                                let should_duplicate = self.state.duplicate_next_tag
                                    && self
                                        .state
                                        .duplicate_tag_exclude
                                        .as_deref()
                                        .map(|ex| ex != tag_id)
                                        .unwrap_or(true);
                                if should_duplicate {
                                    self.state.tags.push(tag_id.clone());
                                    self.state.tags.push(tag_id.clone());
                                    self.state.duplicate_next_tag = false;
                                    self.state.duplicate_tag_exclude = None;
                                } else {
                                    self.state.tags.push(tag_id);
                                }
                            }
                            self.mark_rules_dirty();
                            self.trigger_on_acquire();
                        }
                    }
                    ActionOp::DuplicateNextTag => {
                        if let Some(target) = action.target.as_deref() {
                            self.state.duplicate_next_tag = true;
                            self.state.duplicate_tag_exclude = Some(normalize(target));
                        } else {
                            self.state.duplicate_next_tag = true;
                            self.state.duplicate_tag_exclude = None;
                        }
                    }
                    ActionOp::AddPack => {
                        if let Some(target) = action.target.as_deref() {
                            let price = value.map(|v| v.floor() as i64);
                            self.add_pack_offer_from_target(target, price);
                        }
                    }
                    ActionOp::AddShopJoker => {
                        if let Some(target) = action.target.as_deref() {
                            let price = value.map(|v| v.floor() as i64);
                            self.add_shop_joker_offer(target, price);
                        }
                    }
                    ActionOp::AddVoucher => {
                        if self.shop.is_some() {
                            let delta =
                                value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
                            for _ in 0..delta {
                                let offer = self.voucher_offer_for_shop();
                                if let Some(shop) = self.shop.as_mut() {
                                    shop.add_voucher_offer(offer);
                                }
                            }
                        }
                    }
                    ActionOp::SetRerollCost => {
                        if let Some(shop) = self.shop.as_mut() {
                            let next = value.map(|v| v.floor() as i64).unwrap_or(0).max(0);
                            shop.reroll_cost = next;
                        }
                    }
                    ActionOp::SetShopJokerEdition => {
                        if let Some(target) = action.target.as_deref() {
                            let price = value.map(|v| v.floor() as i64);
                            self.set_shop_joker_edition(target, price);
                        }
                    }
                    ActionOp::RerollBoss => {
                        self.reroll_boss();
                    }
                    ActionOp::UpgradeRandomHand => {
                        let amount = value.map(|v| v.floor().max(0.0) as u32).unwrap_or(1);
                        self.upgrade_random_hand(amount);
                    }
                    ActionOp::MultiplyTarget => {
                        if let Some(value) = value {
                            let scaled = (self.state.target as f64 * value).floor() as i64;
                            self.state.target = scaled.max(0);
                        }
                    }
                    ActionOp::SetRule => {
                        if let (Some(key), Some(value)) = (action.target.as_deref(), value) {
                            self.set_rule_var(key, value);
                        }
                    }
                    ActionOp::AddRule => {
                        if let (Some(key), Some(value)) = (action.target.as_deref(), value) {
                            self.add_rule_var(key, value);
                        }
                    }
                    ActionOp::ClearRule => {
                        if let Some(key) = action.target.as_deref() {
                            self.set_rule_var(key, 0.0);
                        }
                    }
                    ActionOp::SetVar => {
                        if let (Some(key), Some(value)) = (action.target.as_deref(), value) {
                            joker.vars.insert(normalize(key), value);
                        }
                    }
                    ActionOp::AddVar => {
                        if let (Some(key), Some(value)) = (action.target.as_deref(), value) {
                            let entry = joker.vars.entry(normalize(key)).or_insert(0.0);
                            *entry += value;
                        }
                    }
                }, // end Builtin match
            } // end ActionOpKind match
        }
    }

    /// Apply the lightweight mutations returned by a mod-registered DSL effect.
    pub(super) fn apply_mod_action_result(
        &mut self,
        result: &crate::ModActionResult,
        joker: &mut crate::JokerInstance,
        score: &mut Score,
        money: &mut i64,
    ) {
        if result.add_chips != 0 {
            let source = format!("mod_effect:{}:add_chips", joker.id);
            self.apply_rule_effect(
                score,
                crate::RuleEffect::AddChips(result.add_chips),
                &source,
            );
        }
        if result.add_mult != 0.0 {
            let source = format!("mod_effect:{}:add_mult", joker.id);
            self.apply_rule_effect(score, crate::RuleEffect::AddMult(result.add_mult), &source);
        }
        if result.mul_mult != 0.0 {
            let source = format!("mod_effect:{}:mul_mult", joker.id);
            self.apply_rule_effect(
                score,
                crate::RuleEffect::MultiplyMult(result.mul_mult),
                &source,
            );
        }
        if result.mul_chips != 0.0 {
            let source = format!("mod_effect:{}:mul_chips", joker.id);
            self.apply_rule_effect(
                score,
                crate::RuleEffect::MultiplyChips(result.mul_chips),
                &source,
            );
        }
        *money += result.add_money;
        for (key, val) in &result.set_rules {
            self.set_rule_var(key, *val);
        }
        for (key, delta) in &result.add_rules {
            let current = self.rule_value(key);
            self.set_rule_var(key, current + delta);
        }
        for (key, val) in &result.set_vars {
            joker.vars.insert(normalize(key), *val);
        }
        for (key, delta) in &result.add_vars {
            let entry = joker.vars.entry(normalize(key)).or_insert(0.0);
            *entry += delta;
        }
    }

    pub(super) fn apply_shop_price_override(&mut self, target: &str, price: i64) {
        let Some(shop) = self.shop.as_mut() else {
            return;
        };
        let target = normalize(target);
        match target.as_str() {
            "all" | "everything" => {
                for card in &mut shop.cards {
                    card.price = price;
                }
                for pack in &mut shop.packs {
                    pack.price = price;
                }
            }
            "cards" | "card" => {
                for card in &mut shop.cards {
                    card.price = price;
                }
            }
            "packs" | "pack" => {
                for pack in &mut shop.packs {
                    pack.price = price;
                }
            }
            "joker" | "jokers" => {
                for card in &mut shop.cards {
                    if matches!(card.kind, crate::ShopCardKind::Joker) {
                        card.price = price;
                    }
                }
            }
            "tarot" | "tarots" => {
                for card in &mut shop.cards {
                    if matches!(card.kind, crate::ShopCardKind::Tarot) {
                        card.price = price;
                    }
                }
            }
            "planet" | "planets" => {
                for card in &mut shop.cards {
                    if matches!(card.kind, crate::ShopCardKind::Planet) {
                        card.price = price;
                    }
                }
            }
            "arcana_pack" | "arcana" => {
                for pack in &mut shop.packs {
                    if matches!(pack.kind, crate::PackKind::Arcana) {
                        pack.price = price;
                    }
                }
            }
            "buffoon_pack" | "buffoon" => {
                for pack in &mut shop.packs {
                    if matches!(pack.kind, crate::PackKind::Buffoon) {
                        pack.price = price;
                    }
                }
            }
            "celestial_pack" | "celestial" => {
                for pack in &mut shop.packs {
                    if matches!(pack.kind, crate::PackKind::Celestial) {
                        pack.price = price;
                    }
                }
            }
            "spectral_pack" | "spectral" => {
                for pack in &mut shop.packs {
                    if matches!(pack.kind, crate::PackKind::Spectral) {
                        pack.price = price;
                    }
                }
            }
            "standard_pack" | "standard" => {
                for pack in &mut shop.packs {
                    if matches!(pack.kind, crate::PackKind::Standard) {
                        pack.price = price;
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn add_pack_offer_from_target(&mut self, target: &str, price: Option<i64>) {
        let (kind, size) = match parse_pack_target(target, &self.config.shop.pack_weights) {
            Some(pair) => pair,
            None => return,
        };
        let Some(shop) = self.shop.as_mut() else {
            return;
        };
        let options = self
            .config
            .shop
            .pack_weights
            .iter()
            .find(|pack| pack.kind == kind && pack.size == size)
            .map(|pack| (pack.options, pack.picks));
        let Some((options, picks)) = options else {
            return;
        };
        let default_price = self
            .config
            .shop
            .prices
            .pack_prices
            .iter()
            .find(|price| price.size == size)
            .map(|price| price.price)
            .unwrap_or(0);
        shop.packs.push(crate::PackOffer {
            kind,
            size,
            options,
            picks,
            price: price.unwrap_or(default_price),
        });
    }

    pub(super) fn add_shop_joker_offer(&mut self, target: &str, price: Option<i64>) {
        let rarity = match normalize(target).as_str() {
            "common" => crate::JokerRarity::Common,
            "uncommon" => crate::JokerRarity::Uncommon,
            "rare" => crate::JokerRarity::Rare,
            "legendary" => crate::JokerRarity::Legendary,
            _ => crate::JokerRarity::Common,
        };
        let joker_id = {
            let Some(joker) = self.content.pick_joker(rarity, &mut self.rng) else {
                return;
            };
            joker.id.clone()
        };
        let default_price = self.default_joker_price(rarity);
        let Some(shop) = self.shop.as_mut() else {
            return;
        };
        shop.cards.push(crate::CardOffer {
            kind: crate::ShopCardKind::Joker,
            item_id: joker_id,
            rarity: Some(rarity),
            price: price.unwrap_or(default_price),
            edition: None,
        });
    }

    pub(super) fn set_shop_joker_edition(&mut self, edition: &str, price: Option<i64>) {
        let Some(shop) = self.shop.as_mut() else {
            return;
        };
        let Some(edition) = edition_from_str(edition) else {
            return;
        };
        for card in &mut shop.cards {
            if matches!(card.kind, crate::ShopCardKind::Joker) && card.edition.is_none() {
                card.edition = Some(edition);
                if let Some(price) = price {
                    card.price = price;
                }
                break;
            }
        }
    }

    pub(super) fn reroll_boss(&mut self) {
        if self.state.blind != BlindKind::Boss {
            return;
        }
        if self.boss_disabled() {
            return;
        }
        let current = self.state.boss_id.clone();
        if self.content.bosses.is_empty() {
            return;
        }
        let mut next = current.clone();
        for _ in 0..5 {
            if let Some(boss) = self.content.pick_boss(&mut self.rng) {
                if current.as_deref() != Some(&boss.id) {
                    next = Some(boss.id.clone());
                    break;
                }
                next = Some(boss.id.clone());
            }
        }
        self.state.boss_id = next;
        self.mark_rules_dirty();
    }

    pub(super) fn upgrade_random_hand(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        let mut kinds = crate::HandKind::ALL.to_vec();
        self.rng.shuffle(&mut kinds);
        if let Some(kind) = kinds.first().copied() {
            self.upgrade_hand_level(kind, amount);
        }
    }
}

fn parse_pack_target(
    target: &str,
    pack_weights: &[crate::PackWeight],
) -> Option<(crate::PackKind, crate::PackSize)> {
    let norm = normalize(target);
    let mut parts = norm.split('_');
    let kind_part = parts.next().unwrap_or("");
    let size_part = parts.next().unwrap_or("normal");
    let kind = pack_weights.iter().find_map(|pack| {
        let pack_kind = normalize(&format!("{:?}", pack.kind));
        if pack_kind == kind_part {
            Some(pack.kind)
        } else {
            None
        }
    })?;
    let size = match size_part {
        "normal" | "base" => crate::PackSize::Normal,
        "jumbo" => crate::PackSize::Jumbo,
        "mega" => crate::PackSize::Mega,
        _ => crate::PackSize::Normal,
    };
    Some((kind, size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pack_target_uses_configured_pack_weights_for_kind() {
        let weights = vec![
            crate::PackWeight {
                kind: crate::PackKind::Arcana,
                size: crate::PackSize::Normal,
                weight: 1,
                options: 1,
                picks: 1,
            },
            crate::PackWeight {
                kind: crate::PackKind::Buffoon,
                size: crate::PackSize::Mega,
                weight: 1,
                options: 1,
                picks: 1,
            },
        ];
        assert_eq!(
            parse_pack_target("buffoon_mega", &weights),
            Some((crate::PackKind::Buffoon, crate::PackSize::Mega))
        );
        assert_eq!(parse_pack_target("celestial_mega", &weights), None);
    }
}
