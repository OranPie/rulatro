use super::*;
use crate::*;
use super::helpers::*;
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
                self.queue_joker_removal(target);
            }
        }
    }

    pub(super) fn queue_destroy_random(&mut self, exclude: Option<usize>) {
        let mut indices = self.joker_indices_in_order();
        if let Some(exclude) = exclude {
            indices.retain(|idx| *idx != exclude);
        }
        if indices.is_empty() {
            return;
        }
        let pick = (self.rng.next_u64() % indices.len() as u64) as usize;
        self.queue_joker_removal(indices[pick]);
    }

    pub(super) fn flush_pending_joker_changes(&mut self) -> usize {
        if !self.pending_joker_removals.is_empty() {
            self.pending_joker_removals.sort_unstable();
            self.pending_joker_removals.dedup();
            for index in self.pending_joker_removals.iter().rev() {
                if *index < self.inventory.jokers.len() {
                    self.inventory.jokers.remove(*index);
                }
            }
        }
        self.pending_joker_removals.clear();

        let mut added = 0;
        if !self.pending_joker_additions.is_empty() {
            for joker in self.pending_joker_additions.drain(..) {
                if self.inventory.jokers.len() >= self.inventory.joker_slots {
                    break;
                }
                self.inventory.jokers.push(joker);
                added += 1;
            }
        }
        added
    }

    pub(super) fn trigger_on_acquire(&mut self) {
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let ctx = EvalContext::independent(
            hand_kind,
            self.state.blind,
            &[],
            &[],
            &[],
            self.state.hands_left,
            self.state.discards_left,
            self.inventory.jokers.len(),
        );
        let mut dummy_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        self.apply_joker_effects(
            ActivationType::OnAcquire,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        self.state.money = money;
    }

    pub(super) fn add_joker_var_by_id(
        &mut self,
        id: &str,
        key: &str,
        delta: f64,
        default: f64,
    ) {
        let key = normalize(key);
        for joker in &mut self.inventory.jokers {
            if joker.id != id {
                continue;
            }
            let entry = joker.vars.entry(key.clone()).or_insert(default);
            *entry += delta;
        }
    }

    pub(super) fn add_joker_with_trigger(
        &mut self,
        id: String,
        rarity: crate::JokerRarity,
        buy_price: i64,
    ) -> Result<(), RunError> {
        self.inventory.add_joker(id, rarity, buy_price)?;
        self.trigger_on_acquire();
        Ok(())
    }

    pub(super) fn spawn_joker_from_target(&mut self, target: &str) -> Option<crate::JokerInstance> {
        let norm = normalize(target);
        let (id, rarity) = match norm.as_str() {
            "common" => {
                let def = self.content.pick_joker(crate::JokerRarity::Common, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Common)
            }
            "uncommon" => {
                let def = self
                    .content
                    .pick_joker(crate::JokerRarity::Uncommon, &mut self.rng)?;
                (def.id.clone(), crate::JokerRarity::Uncommon)
            }
            "rare" => {
                let def = self.content.pick_joker(crate::JokerRarity::Rare, &mut self.rng)?;
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

    pub(super) fn apply_joker_edition_before(&self, joker: &crate::JokerInstance, score: &mut Score) {
        match joker.edition {
            Some(Edition::Foil) => score.apply(&crate::RuleEffect::AddChips(50)),
            Some(Edition::Holographic) => score.apply(&crate::RuleEffect::AddMult(10.0)),
            _ => {}
        }
    }

    pub(super) fn apply_joker_edition_after(&self, joker: &crate::JokerInstance, score: &mut Score) {
        match joker.edition {
            Some(Edition::Polychrome) => score.apply(&crate::RuleEffect::MultiplyMult(1.5)),
            _ => {}
        }
    }

    pub(super) fn apply_joker_effects(
        &mut self,
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
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
            let ctx = ctx
                .with_joker_vars(&joker.vars)
                .with_joker_index(snap.index);
            self.apply_joker_effects_for_joker(&mut joker, trigger, &ctx, score, money, results);
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
    }

    pub(super) fn apply_joker_effects_for_joker(
        &mut self,
        joker: &mut crate::JokerInstance,
        trigger: ActivationType,
        ctx: &EvalContext<'_>,
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
                self.apply_actions(joker, &effect.actions, ctx, score, money, results);
            }
        }
    }

    pub(super) fn apply_actions(
        &mut self,
        joker: &mut crate::JokerInstance,
        actions: &[crate::Action],
        ctx: &EvalContext<'_>,
        score: &mut Score,
        money: &mut i64,
        results: &mut TriggerResults,
    ) {
        for action in actions {
            let evaluated = self.eval_expr(&action.value, ctx);
            let value = evaluated.as_number();
            match action.op {
                ActionOp::AddChips => {
                    if let Some(value) = value {
                        score.apply(&crate::RuleEffect::AddChips(value.floor() as i64));
                    }
                }
                ActionOp::AddMult => {
                    if let Some(value) = value {
                        score.apply(&crate::RuleEffect::AddMult(value));
                    }
                }
                ActionOp::MultiplyMult => {
                    if let Some(value) = value {
                        score.apply(&crate::RuleEffect::MultiplyMult(value));
                    }
                }
                ActionOp::MultiplyChips => {
                    if let Some(value) = value {
                        score.apply(&crate::RuleEffect::MultiplyChips(value));
                    }
                }
                ActionOp::AddMoney => {
                    if let Some(value) = value {
                        *money += value.floor() as i64;
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
                        self.deck.draw.push(card);
                    }
                    self.deck.shuffle(&mut self.rng);
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
                            let _ = self
                                .inventory
                                .add_consumable(card.id.clone(), crate::ConsumableKind::Spectral);
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
                        let count = value.map(|v| v.floor() as i64).unwrap_or(1).max(0) as usize;
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
                        if self.inventory.jokers.len() >= self.inventory.joker_slots {
                            break;
                        }
                        if self.inventory.jokers.is_empty() {
                            break;
                        }
                        let idx = (self.rng.next_u64() % self.inventory.jokers.len() as u64)
                            as usize;
                        let mut copy = self.inventory.jokers[idx].clone();
                        if copy.edition == Some(Edition::Negative) {
                            copy.edition = None;
                        }
                        self.inventory.jokers.push(copy);
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
            }
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

}
