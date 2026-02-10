use super::*;
use crate::*;
use super::helpers::*;

impl RunState {
    pub fn prepare_hand(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Deal {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        if self.state.hands_left == 0 {
            return Err(RunError::NoHandsLeft);
        }
        self.draw_to_hand(events);
        self.state.phase = Phase::Play;
        Ok(())
    }

    pub fn draw_to_hand(&mut self, events: &mut EventBus) {
        let mut needed = self.state.hand_size.saturating_sub(self.hand.len());
        if needed == 0 {
            return;
        }

        let mut total_drawn = 0;
        while needed > 0 {
            if self.deck.draw.is_empty() {
                self.deck.reshuffle_discard(&mut self.rng);
                if self.deck.draw.is_empty() {
                    break;
                }
            }
            let mut drawn = self.deck.draw_cards(needed);
            if drawn.is_empty() {
                break;
            }
            total_drawn += drawn.len();
            self.hand.append(&mut drawn);
            needed = self.state.hand_size.saturating_sub(self.hand.len());
        }

        if total_drawn > 0 {
            events.push(Event::HandDealt { count: total_drawn });
        }
    }

    pub fn play_hand(
        &mut self,
        indices: &[usize],
        events: &mut EventBus,
    ) -> Result<ScoreBreakdown, RunError> {
        if self.state.phase != Phase::Play {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        if self.state.hands_left == 0 {
            return Err(RunError::NoHandsLeft);
        }
        if indices.len() > 5 {
            return Err(RunError::InvalidCardCount);
        }

        let mut played = take_cards(&mut self.hand, indices)?;
        self.state.phase = Phase::Score;
        let eval_rules = self.hand_eval_rules();
        let mut breakdown =
            score_hand_with_rules(&played, &self.tables, eval_rules, &self.state.hand_levels);
        if self.splash_active() {
            breakdown.scoring_indices = (0..played.len()).collect();
        }
        self.state.last_hand = Some(breakdown.hand);
        *self
            .state
            .hand_play_counts
            .entry(breakdown.hand)
            .or_insert(0) += 1;
        let mut total_score = breakdown.total.clone();
        let mut money = self.state.money;
        let mut scoring_cards: Vec<Card> = breakdown
            .scoring_indices
            .iter()
            .map(|&idx| played[idx])
            .collect();
        let held_cards = self.hand.clone();
        let joker_count = self.inventory.jokers.len();

        let is_first_hand = self.state.hands_left == self.state.hands_max;
        let mut pre_destroyed = Vec::new();
        if is_first_hand && played.len() == 1 && self.has_joker_id("sixth_sense") {
            if played[0].rank == crate::Rank::Six
                && self.inventory.consumables.len() < self.inventory.consumable_slots
            {
                pre_destroyed.push(0);
                if let Some(card) =
                    self.content
                        .pick_consumable(crate::ConsumableKind::Spectral, &mut self.rng)
                {
                    let _ = self
                        .inventory
                        .add_consumable(card.id.clone(), crate::ConsumableKind::Spectral);
                }
            }
        }
        if is_first_hand && played.len() == 1 && self.has_joker_id("dna") {
            let copy = played[0];
            self.hand.push(copy);
        }

        self.apply_boss_blind_effects(&mut total_score, &mut money, breakdown.hand);

        let mut results = TriggerResults::default();
        let ctx = EvalContext::played(
            breakdown.hand,
            self.state.blind,
            &played,
            &scoring_cards,
            &held_cards,
            self.state.hands_left,
            self.state.discards_left,
            joker_count,
        );
        self.apply_joker_effects(
            ActivationType::OnPlayed,
            &ctx,
            &mut total_score,
            &mut money,
            &mut results,
        );

        let scored_outcome =
            self.apply_scored_card_pipeline(
                &mut played,
                &mut scoring_cards,
                &held_cards,
                &breakdown,
                joker_count,
                &mut total_score,
                &mut money,
            );
        self.apply_held_card_pipeline(
            &mut total_score,
            &mut money,
            breakdown.hand,
            &played,
            &scoring_cards,
            &held_cards,
            joker_count,
        );
        self.apply_joker_editions_and_independent(
            &mut total_score,
            &mut money,
            breakdown.hand,
            &played,
            &scoring_cards,
            &held_cards,
            joker_count,
        );
        self.apply_pending_effects(&mut total_score, &mut money);

        self.state.money = money;
        breakdown.total = total_score;
        let total = breakdown.total.total();

        self.state.blind_score += total;
        self.state.hands_left = self.state.hands_left.saturating_sub(1);
        self.state.phase = Phase::Cleanup;

        let mut destroyed = scored_outcome.destroyed_indices;
        destroyed.extend(pre_destroyed);
        destroyed.sort_unstable();
        destroyed.dedup();
        let mut to_discard = Vec::with_capacity(played.len());
        for (idx, card) in played.into_iter().enumerate() {
            if destroyed.binary_search(&idx).is_err() {
                to_discard.push(card);
            }
        }
        self.deck.discard(to_discard);
        events.push(Event::HandScored {
            hand: breakdown.hand,
            chips: breakdown.total.chips,
            mult: breakdown.total.mult,
            total,
        });

        if let Some(_outcome) = self.check_outcome(events) {
            self.state.phase = Phase::Cleanup;
            return Ok(breakdown);
        }

        self.state.phase = Phase::Deal;
        Ok(breakdown)
    }

    pub fn discard(&mut self, indices: &[usize], events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Play {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        if self.state.discards_left == 0 {
            return Err(RunError::NoDiscardsLeft);
        }
        if indices.len() > 5 {
            return Err(RunError::InvalidCardCount);
        }
        let discarded = take_cards(&mut self.hand, indices)?;
        self.apply_discard_effects(&discarded);
        self.deck.discard(discarded);
        self.state.discards_left = self.state.discards_left.saturating_sub(1);
        self.draw_to_hand(events);
        Ok(())
    }

    pub(super) fn apply_boss_blind_effects(
        &mut self,
        _score: &mut Score,
        _money: &mut i64,
        _hand_kind: crate::HandKind,
    ) {
        // TODO: Boss blind rules and debuffs live here.
    }

    pub(super) fn apply_scored_card_pipeline(
        &mut self,
        played: &mut [Card],
        scoring_cards: &mut Vec<Card>,
        held_cards: &[Card],
        breakdown: &ScoreBreakdown,
        joker_count: usize,
        score: &mut Score,
        money: &mut i64,
    ) -> ScoredOutcome {
        let mut destroyed_indices = Vec::new();
        let hiker_active = self.has_joker_id("hiker");
        let vampire_active = self.has_joker_id("vampire");
        let midas_active = self.has_joker_id("midas_mask");
        for (scoring_pos, &idx) in breakdown.scoring_indices.iter().enumerate() {
            let mut remaining = if played[idx].seal == Some(Seal::Red) { 2 } else { 1 };
            let mut pending = 0i64;
            while remaining > 0 {
                remaining -= 1;
                let was_enhanced = played[idx].enhancement.is_some();
                let mut results = TriggerResults::default();
                let destroyed_now = self.apply_card_enhancement_scored(
                    &mut played[idx],
                    score,
                    money,
                    &mut results,
                    idx,
                    &mut destroyed_indices,
                );
                if hiker_active {
                    played[idx].bonus_chips = played[idx].bonus_chips.saturating_add(5);
                }
                if vampire_active && was_enhanced {
                    played[idx].enhancement = None;
                    self.add_joker_var_by_id("vampire", "mult", 0.1, 1.0);
                }
                if midas_active && is_face(played[idx]) {
                    played[idx].enhancement = Some(Enhancement::Gold);
                }
                if scoring_pos < scoring_cards.len() {
                    scoring_cards[scoring_pos] = played[idx];
                }
                let card = played[idx];
                if card.bonus_chips > 0 {
                    score.apply(&crate::RuleEffect::AddChips(card.bonus_chips));
                }
                self.apply_card_seal_scored(card, score, money, &mut results);
                self.apply_card_edition_scored(card, score);
                let ctx = EvalContext::scoring(
                    breakdown.hand,
                    self.state.blind,
                    card,
                    played,
                    scoring_cards,
                    held_cards,
                    self.state.hands_left,
                    self.state.discards_left,
                    joker_count,
                );
                if destroyed_now {
                    self.apply_joker_effects(
                        ActivationType::OnCardDestroyed,
                        &ctx,
                        score,
                        money,
                        &mut results,
                    );
                }
                self.apply_joker_effects(
                    ActivationType::OnScored,
                    &ctx,
                    score,
                    money,
                    &mut results,
                );
                if results.scored_retriggers > 0 {
                    pending += results.scored_retriggers;
                }
                if remaining == 0 && pending > 0 {
                    remaining = pending as usize;
                    pending = 0;
                }
            }
        }
        ScoredOutcome { destroyed_indices }
    }

    pub(super) fn apply_held_card_pipeline(
        &mut self,
        score: &mut Score,
        money: &mut i64,
        hand_kind: crate::HandKind,
        played_cards: &[Card],
        scoring_cards: &[Card],
        held_cards: &[Card],
        joker_count: usize,
    ) {
        for &card in held_cards {
            let mut remaining = if card.seal == Some(Seal::Red) { 2 } else { 1 };
            let mut pending = 0i64;
            while remaining > 0 {
                remaining -= 1;
                let mut results = TriggerResults::default();
                self.apply_card_enhancement_held(card, score, money);
                self.apply_card_seal_held(card, score, money, &mut results);
                let ctx = EvalContext::held(
                    hand_kind,
                    self.state.blind,
                    card,
                    played_cards,
                    scoring_cards,
                    held_cards,
                    self.state.hands_left,
                    self.state.discards_left,
                    joker_count,
                );
                self.apply_joker_effects(
                    ActivationType::OnHeld,
                    &ctx,
                    score,
                    money,
                    &mut results,
                );
                if results.held_retriggers > 0 {
                    pending += results.held_retriggers;
                }
                if remaining == 0 && pending > 0 {
                    remaining = pending as usize;
                    pending = 0;
                }
            }
        }
    }

    pub(super) fn apply_card_enhancement_scored(
        &mut self,
        card: &mut Card,
        score: &mut Score,
        money: &mut i64,
        _results: &mut TriggerResults,
        idx: usize,
        destroyed: &mut Vec<usize>,
    ) -> bool {
        let mut destroyed_now = false;
        match card.enhancement {
            Some(Enhancement::Bonus) => score.apply(&crate::RuleEffect::AddChips(30)),
            Some(Enhancement::Mult) => score.apply(&crate::RuleEffect::AddMult(4.0)),
            Some(Enhancement::Glass) => {
                score.apply(&crate::RuleEffect::MultiplyMult(2.0));
                if self.roll(4) {
                    if !destroyed.iter().any(|&existing| existing == idx) {
                        destroyed.push(idx);
                        destroyed_now = true;
                    }
                }
            }
            Some(Enhancement::Stone) => score.apply(&crate::RuleEffect::AddChips(50)),
            Some(Enhancement::Lucky) => {
                if self.roll(5) {
                    score.apply(&crate::RuleEffect::AddMult(20.0));
                }
                if self.roll(15) {
                    *money += 20;
                }
            }
            _ => {}
        }
        destroyed_now
    }

    pub(super) fn apply_card_enhancement_held(&mut self, card: Card, score: &mut Score, _money: &mut i64) {
        match card.enhancement {
            Some(Enhancement::Steel) => score.apply(&crate::RuleEffect::MultiplyMult(1.5)),
            _ => {}
        }
    }

    pub(super) fn apply_card_seal_scored(
        &mut self,
        card: Card,
        _score: &mut Score,
        money: &mut i64,
        _results: &mut TriggerResults,
    ) {
        if card.seal == Some(Seal::Gold) {
            *money += 3;
        }
    }

    pub(super) fn apply_card_seal_held(
        &mut self,
        _card: Card,
        _score: &mut Score,
        _money: &mut i64,
        _results: &mut TriggerResults,
    ) {
    }

    pub(super) fn apply_card_edition_scored(&self, card: Card, score: &mut Score) {
        match card.edition {
            Some(Edition::Foil) => score.apply(&crate::RuleEffect::AddChips(50)),
            Some(Edition::Holographic) => score.apply(&crate::RuleEffect::AddMult(10.0)),
            Some(Edition::Polychrome) => score.apply(&crate::RuleEffect::MultiplyMult(1.5)),
            _ => {}
        }
    }

    pub(super) fn apply_discard_effects(&mut self, discarded: &[Card]) {
        let eval_rules = self.hand_eval_rules();
        let hand_kind = crate::evaluate_hand_with_rules(discarded, eval_rules);
        let joker_count = self.inventory.jokers.len();
        let held_cards = self.hand.clone();
        let mut dummy_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        let batch_ctx = EvalContext::discard_batch(
            hand_kind,
            self.state.blind,
            &held_cards,
            discarded,
            self.state.hands_left,
            self.state.discards_left,
            joker_count,
        );
        self.apply_joker_effects(
            ActivationType::OnDiscardBatch,
            &batch_ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        for card in discarded {
            if card.seal == Some(Seal::Purple) {
                let tarot_id = self
                    .content
                    .pick_consumable(crate::ConsumableKind::Tarot, &mut self.rng)
                    .map(|tarot| tarot.id.clone());
                if let Some(id) = tarot_id {
                    let _ = self.inventory.add_consumable(id, crate::ConsumableKind::Tarot);
                }
            }
            let ctx = EvalContext::discard(
                hand_kind,
                self.state.blind,
                *card,
                &held_cards,
                discarded,
                self.state.hands_left,
                self.state.discards_left,
                joker_count,
            );
            self.apply_joker_effects(
                ActivationType::OnDiscard,
                &ctx,
                &mut dummy_score,
                &mut money,
                &mut results,
            );
        }
        self.state.money = money;
    }

    pub(super) fn resolve_round_end_effects(&mut self) {
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let joker_count = self.inventory.jokers.len();
        let hand = self.hand.clone();
        for card in &hand {
            if card.enhancement == Some(Enhancement::Gold) {
                self.state.money += 3;
            }
            if card.seal == Some(Seal::Blue) {
                self.grant_planet_for_hand(hand_kind);
            }
        }
        let mut dummy_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        let ctx = EvalContext::independent(
            hand_kind,
            self.state.blind,
            &[],
            &[],
            &hand,
            self.state.hands_left,
            self.state.discards_left,
            joker_count,
        );
        self.apply_joker_effects(
            ActivationType::OnRoundEnd,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        self.state.money = money;
    }

    pub(super) fn grant_planet_for_hand(&mut self, hand_kind: crate::HandKind) {
        let planet_id = self
            .content
            .planet_for_hand(hand_kind, &mut self.rng)
            .map(|planet| planet.id.clone());
        if let Some(id) = planet_id {
            let _ = self.inventory.add_consumable(id, crate::ConsumableKind::Planet);
        }
    }

    pub(super) fn apply_pending_effects(&mut self, score: &mut Score, money: &mut i64) {
        for effect in &self.pending_effects {
            match effect {
                EffectOp::Score(rule) => score.apply(rule),
                EffectOp::AddMoney(value) => *money += value,
                EffectOp::AddHandSize(value) => {
                    let next = (self.state.hand_size as i64 + value).max(0) as usize;
                    self.state.hand_size = next;
                }
                EffectOp::RetriggerScored(_) | EffectOp::RetriggerHeld(_) => {}
            }
        }
        self.pending_effects.clear();
    }

    pub(super) fn apply_consumable_effects(&mut self, def: &crate::ConsumableDef) -> Result<(), RunError> {
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let joker_count = self.inventory.jokers.len();
        let mut dummy_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        let ctx = EvalContext::consumable(
            hand_kind,
            self.state.blind,
            def.kind,
            &def.id,
            self.state.hands_left,
            self.state.discards_left,
            joker_count,
        );
        self.apply_joker_effects(
            ActivationType::OnUse,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        for block in &def.effects {
            if block.trigger != ActivationType::OnUse {
                continue;
            }
            if !self.json_conditions_met(&block.conditions, crate::HandKind::HighCard, None) {
                continue;
            }
            for effect in &block.effects {
                match effect {
                    EffectOp::Score(rule) => self.pending_effects.push(EffectOp::Score(rule.clone())),
                    EffectOp::AddMoney(value) => money += value,
                    EffectOp::AddHandSize(value) => {
                        let next = (self.state.hand_size as i64 + value).max(0) as usize;
                        self.state.hand_size = next;
                    }
                    EffectOp::RetriggerScored(_) | EffectOp::RetriggerHeld(_) => {}
                }
            }
        }
        self.state.money = money;
        Ok(())
    }
}

fn take_cards(hand: &mut Vec<crate::Card>, indices: &[usize]) -> Result<Vec<crate::Card>, RunError> {
    if indices.is_empty() {
        return Err(RunError::InvalidSelection);
    }
    let mut unique = indices.to_vec();
    unique.sort_unstable();
    unique.dedup();
    if unique.iter().any(|&idx| idx >= hand.len()) {
        return Err(RunError::InvalidSelection);
    }

    unique.sort_unstable_by(|a, b| b.cmp(a));
    let mut picked = Vec::with_capacity(unique.len());
    for idx in unique {
        picked.push(hand.remove(idx));
    }
    Ok(picked)
}
