use super::*;
use super::helpers::is_face;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawReason {
    Deal,
    Discard,
    BonusAfterPlay,
    BonusAfterDiscard,
}

impl RunState {
    pub fn prepare_hand(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Deal {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        if self.state.hands_left == 0 {
            return Err(RunError::NoHandsLeft);
        }
        self.draw_to_hand_with_reason(events, DrawReason::Deal);
        self.state.phase = Phase::Play;
        Ok(())
    }

    pub fn draw_to_hand(&mut self, events: &mut EventBus) {
        self.draw_to_hand_with_reason(events, DrawReason::Deal);
    }

    fn draw_to_hand_with_reason(&mut self, events: &mut EventBus, reason: DrawReason) {
        let needed = self.state.hand_size.saturating_sub(self.hand.len());
        if needed == 0 {
            return;
        }
        self.draw_cards(needed, events, reason);
    }

    fn draw_cards(&mut self, mut count: usize, events: &mut EventBus, reason: DrawReason) {
        if count == 0 {
            return;
        }
        let mut total_drawn = 0;
        while count > 0 {
            if self.deck.draw.is_empty() {
                self.deck.reshuffle_discard(&mut self.rng);
                if self.deck.draw.is_empty() {
                    break;
                }
            }
            let mut drawn = self.deck.draw_cards(count);
            if drawn.is_empty() {
                break;
            }
            for card in &mut drawn {
                self.assign_card_id(card);
                if self.should_draw_face_down(*card, reason) {
                    card.face_down = true;
                }
            }
            count = count.saturating_sub(drawn.len());
            total_drawn += drawn.len();
            self.hand.append(&mut drawn);
        }
        if total_drawn > 0 {
            events.push(Event::HandDealt { count: total_drawn });
        }
    }

    fn should_draw_face_down(&mut self, card: Card, reason: DrawReason) -> bool {
        let first_hand = self.rule_flag("draw_face_down_first_hand")
            && self.state.hands_left == self.state.hands_max;
        let after_hand = self.rule_flag("draw_face_down_after_hand")
            && matches!(reason, DrawReason::Deal | DrawReason::BonusAfterPlay);
        let mut rolled = false;
        let roll_sides = self.rule_value("draw_face_down_roll").floor() as i64;
        if roll_sides > 0 {
            rolled = self.roll(roll_sides as u64);
        }
        let face_rule = self.rule_flag("draw_face_down_face") && is_face(card);
        first_hand || after_hand || rolled || face_rule
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
        let required = self.rule_value("required_play_count").floor() as i64;
        if required > 0 && indices.len() as i64 != required {
            return Err(RunError::InvalidCardCount);
        }

        let mut played = take_cards(&mut self.hand, indices)?;
        self.state.phase = Phase::Score;
        let eval_rules = self.hand_eval_rules();
        let mut eval_cards = played.clone();
        for card in &mut eval_cards {
            if self.is_card_debuffed(*card) && card.enhancement == Some(Enhancement::Stone) {
                card.enhancement = None;
            }
        }
        let mut breakdown =
            score_hand_with_rules(&eval_cards, &self.tables, eval_rules, &self.state.hand_levels);
        if self.splash_active() {
            breakdown.scoring_indices = (0..played.len()).collect();
        }
        if self.rule_flag("single_hand_type") {
            match self.state.round_hand_lock {
                None => self.state.round_hand_lock = Some(breakdown.hand),
                Some(locked) if locked != breakdown.hand => {
                    return Err(RunError::HandNotAllowed);
                }
                _ => {}
            }
        }
        if self.rule_flag("no_repeat_hand") {
            if self.state.round_hand_types.contains(&breakdown.hand) {
                return Err(RunError::HandNotAllowed);
            }
            self.state.round_hand_types.insert(breakdown.hand);
        }
        let level_delta = self.rule_value("hand_level_delta").floor() as i64;
        if level_delta != 0 {
            let base_level = self.hand_level(breakdown.hand) as i64;
            let effective = (base_level + level_delta).max(1) as u32;
            let (base_chips, base_mult) = self.tables.hand_base_for_level(breakdown.hand, effective);
            breakdown.base.chips = base_chips;
            breakdown.base.mult = base_mult;
            breakdown.total.chips = breakdown.base.chips + breakdown.rank_chips;
            breakdown.total.mult = breakdown.base.mult;
        }
        let base_chips_mult = self.rule_value_or("base_chips_mult", 1.0);
        let base_mult_mult = self.rule_value_or("base_mult_mult", 1.0);
        if base_chips_mult != 1.0 || base_mult_mult != 1.0 {
            let scaled = (breakdown.base.chips as f64 * base_chips_mult).floor() as i64;
            breakdown.base.chips = scaled;
            breakdown.base.mult *= base_mult_mult;
            breakdown.total.chips = breakdown.base.chips + breakdown.rank_chips;
            breakdown.total.mult = breakdown.base.mult;
        }
        self.state.last_hand = Some(breakdown.hand);
        *self
            .state
            .hand_play_counts
            .entry(breakdown.hand)
            .or_insert(0) += 1;
        if self.state.blind != BlindKind::Boss {
            for card in &played {
                self.state.played_card_ids_ante.insert(card.id);
            }
        }
        let mut total_score = breakdown.total.clone();
        let mut money = self.state.money;
        let mut scoring_cards: Vec<Card> = breakdown
            .scoring_indices
            .iter()
            .map(|&idx| played[idx])
            .collect();
        let mut held_cards = self.hand.clone();

        let mut results = TriggerResults::default();
        let mut args = HookArgs::played(
            breakdown.hand,
            self.state.blind,
            HookInject::played(&mut played, &mut scoring_cards, &mut held_cards),
            &mut total_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::Played, &mut args, events);

        let scored_outcome =
            self.apply_scored_card_pipeline(
                &mut played,
                &mut scoring_cards,
                &held_cards,
                &breakdown,
                events,
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
            events,
        );
        let mut independent_results = TriggerResults::default();
        let mut independent_args = HookArgs::independent(
            breakdown.hand,
            self.state.blind,
            HookInject::played(&mut played, &mut scoring_cards, &mut held_cards),
            &mut total_score,
            &mut money,
            &mut independent_results,
        );
        self.invoke_hooks(HookPoint::Independent, &mut independent_args, events);
        let mut hand_end_results = TriggerResults::default();
        let mut hand_end_args = HookArgs::independent(
            breakdown.hand,
            self.state.blind,
            HookInject::played(&mut played, &mut scoring_cards, &mut held_cards),
            &mut total_score,
            &mut money,
            &mut hand_end_results,
        );
        self.invoke_hooks(HookPoint::HandEnd, &mut hand_end_args, events);
        self.apply_pending_effects(&mut total_score, &mut money);

        self.state.money = money;
        breakdown.total = total_score;
        let total = breakdown.total.total();

        self.state.blind_score += total;
        self.state.hands_left = self.state.hands_left.saturating_sub(1);
        self.state.phase = Phase::Cleanup;

        let mut destroyed = scored_outcome.destroyed_indices;
        destroyed.sort_unstable();
        destroyed.dedup();
        let mut to_discard = Vec::with_capacity(played.len());
        for (idx, card) in played.into_iter().enumerate() {
            if destroyed.binary_search(&idx).is_err() {
                to_discard.push(card);
            }
        }
        self.deck.discard(to_discard);
        let discard_after = self.rule_value("discard_held_after_hand").floor() as i64;
        if discard_after > 0 {
            self.discard_random_held(discard_after as usize, events);
        }
        let extra_draw = self.rule_value("draw_after_play").floor() as i64;
        if extra_draw > 0 {
            self.draw_cards(extra_draw as usize, events, DrawReason::BonusAfterPlay);
        }
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
        self.apply_discard_effects(&discarded, events);
        self.deck.discard(discarded);
        self.state.discards_left = self.state.discards_left.saturating_sub(1);
        let extra = self.rule_value("draw_after_discard").floor() as i64;
        if extra > 0 {
            self.draw_cards(extra as usize, events, DrawReason::BonusAfterDiscard);
        } else {
            self.draw_to_hand_with_reason(events, DrawReason::Discard);
        }
        Ok(())
    }

    pub(super) fn apply_scored_card_pipeline(
        &mut self,
        played: &mut [Card],
        scoring_cards: &mut Vec<Card>,
        held_cards: &[Card],
        breakdown: &ScoreBreakdown,
        events: &mut EventBus,
        score: &mut Score,
        money: &mut i64,
    ) -> ScoredOutcome {
        let mut destroyed_indices = Vec::new();
        for (scoring_pos, &idx) in breakdown.scoring_indices.iter().enumerate() {
            let debuffed = self.is_card_debuffed(played[idx]);
            let mut remaining = if !debuffed && played[idx].seal == Some(Seal::Red) {
                2
            } else {
                1
            };
            let mut pending = 0i64;
            while remaining > 0 {
                remaining -= 1;
                let mut results = TriggerResults::default();
                let (destroyed_now, lucky_triggers) = if debuffed {
                    (false, 0)
                } else {
                    self.apply_card_enhancement_scored(
                        &mut played[idx],
                        score,
                        money,
                        &mut results,
                        idx,
                        &mut destroyed_indices,
                    )
                };
                if !debuffed {
                    let mut played_view = played.to_vec();
                    let mut scoring_view = scoring_cards.clone();
                    let mut held_view = held_cards.to_vec();
                    let card_snapshot = played[idx];
                    let mut pre_args = HookArgs::scoring(
                        breakdown.hand,
                        self.state.blind,
                        card_snapshot,
                        lucky_triggers,
                        HookInject::cards(
                            Some(&mut played_view),
                            Some(&mut scoring_view),
                            Some(&mut held_view),
                            None,
                        ),
                        Some(&mut played[idx]),
                        score,
                        money,
                        &mut results,
                    );
                    self.invoke_hooks(HookPoint::ScoredPre, &mut pre_args, events);
                    if scoring_pos < scoring_cards.len() {
                        scoring_cards[scoring_pos] = played[idx];
                    }
                }
                let destroyed_before = results.destroyed_current;
                let destroyed_after = if !debuffed {
                    let destroyed_event = destroyed_now || destroyed_before;
                    let card = played[idx];
                    if card.bonus_chips > 0 {
                        score.apply(&crate::RuleEffect::AddChips(card.bonus_chips));
                    }
                    self.apply_card_seal_scored(card, score, money, &mut results);
                    self.apply_card_edition_scored(card, score);
                    let mut played_view = played.to_vec();
                    let mut scoring_view = scoring_cards.clone();
                    let mut held_view = held_cards.to_vec();
                    let mut args = HookArgs::scoring(
                        breakdown.hand,
                        self.state.blind,
                        card,
                        lucky_triggers,
                        HookInject::cards(
                            Some(&mut played_view),
                            Some(&mut scoring_view),
                            Some(&mut held_view),
                            None,
                        ),
                        None,
                        score,
                        money,
                        &mut results,
                    );
                    if destroyed_event {
                        self.invoke_hooks(HookPoint::CardDestroyed, &mut args, events);
                    }
                    self.invoke_hooks(HookPoint::Scored, &mut args, events);
                    drop(args);
                    let destroyed_after = results.destroyed_current;
                    if destroyed_after && !destroyed_event {
                        let mut played_view = played.to_vec();
                        let mut scoring_view = scoring_cards.clone();
                        let mut held_view = held_cards.to_vec();
                        let mut args = HookArgs::scoring(
                            breakdown.hand,
                            self.state.blind,
                            card,
                            lucky_triggers,
                            HookInject::cards(
                                Some(&mut played_view),
                                Some(&mut scoring_view),
                                Some(&mut held_view),
                                None,
                            ),
                            None,
                            score,
                            money,
                            &mut results,
                        );
                        self.invoke_hooks(HookPoint::CardDestroyed, &mut args, events);
                    }
                    destroyed_after
                } else {
                    false
                };
                if destroyed_after
                    && !destroyed_indices.iter().any(|&existing| existing == idx)
                {
                    destroyed_indices.push(idx);
                }
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
        events: &mut EventBus,
    ) {
        for &card in held_cards {
            let debuffed = self.is_card_debuffed(card);
            let mut remaining = if !debuffed && card.seal == Some(Seal::Red) {
                2
            } else {
                1
            };
            let mut pending = 0i64;
            while remaining > 0 {
                remaining -= 1;
                let mut results = TriggerResults::default();
                if !debuffed {
                    self.apply_card_enhancement_held(card, score, money);
                    self.apply_card_seal_held(card, score, money, &mut results);
                    let mut played_view = played_cards.to_vec();
                    let mut scoring_view = scoring_cards.to_vec();
                    let mut held_view = held_cards.to_vec();
                    let mut args = HookArgs::held(
                        hand_kind,
                        self.state.blind,
                        card,
                        HookInject::cards(
                            Some(&mut played_view),
                            Some(&mut scoring_view),
                            Some(&mut held_view),
                            None,
                        ),
                        score,
                        money,
                        &mut results,
                    );
                    self.invoke_hooks(HookPoint::Held, &mut args, events);
                }
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
    ) -> (bool, i64) {
        let mut destroyed_now = false;
        let mut lucky_triggers = 0i64;
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
                    lucky_triggers += 1;
                }
                if self.roll(15) {
                    *money += 20;
                    lucky_triggers += 1;
                }
            }
            _ => {}
        }
        (destroyed_now, lucky_triggers)
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

    pub(super) fn apply_discard_effects(&mut self, discarded: &[Card], events: &mut EventBus) {
        let eval_rules = self.hand_eval_rules();
        let hand_kind = crate::evaluate_hand_with_rules(discarded, eval_rules);
        let held_cards = self.hand.clone();
        let mut scratch_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        let mut held_view = held_cards.clone();
        let mut discarded_view = discarded.to_vec();
        let mut batch_args = HookArgs::discard_batch(
            hand_kind,
            self.state.blind,
            HookInject::discard(&mut held_view, &mut discarded_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::DiscardBatch, &mut batch_args, events);
        for card in discarded {
            let debuffed = self.is_card_debuffed(*card);
            if !debuffed && card.seal == Some(Seal::Purple) {
                let tarot_id = self
                    .content
                    .pick_consumable(crate::ConsumableKind::Tarot, &mut self.rng)
                    .map(|tarot| tarot.id.clone());
                if let Some(id) = tarot_id {
                    let _ = self.inventory.add_consumable(id, crate::ConsumableKind::Tarot);
                }
            }
            if debuffed {
                continue;
            }
            let mut held_view = held_cards.clone();
            let mut discarded_view = discarded.to_vec();
            let mut card_args = HookArgs::discard(
                hand_kind,
                self.state.blind,
                *card,
                HookInject::discard(&mut held_view, &mut discarded_view),
                &mut scratch_score,
                &mut money,
                &mut results,
            );
            self.invoke_hooks(HookPoint::Discard, &mut card_args, events);
        }
        self.state.money = money;
    }

    fn discard_random_held(&mut self, count: usize, events: &mut EventBus) {
        if count == 0 || self.hand.is_empty() {
            return;
        }
        let mut indices: Vec<usize> = (0..self.hand.len()).collect();
        self.rng.shuffle(&mut indices);
        indices.truncate(count.min(indices.len()));
        indices.sort_unstable();
        let discarded = match take_cards(&mut self.hand, &indices) {
            Ok(cards) => cards,
            Err(_) => return,
        };
        self.apply_discard_effects(&discarded, events);
        self.deck.discard(discarded);
    }

    pub(super) fn resolve_round_end_effects(&mut self, events: &mut EventBus) {
        self.state.unused_discards = self
            .state
            .unused_discards
            .saturating_add(self.state.discards_left as u32);
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let hand = self.hand.clone();
        for card in &hand {
            if card.enhancement == Some(Enhancement::Gold) {
                self.state.money += 3;
            }
            if card.seal == Some(Seal::Blue) {
                self.grant_planet_for_hand(hand_kind);
            }
        }
        let mut scratch_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        let mut held_view = hand.clone();
        let mut args = HookArgs::independent(
            hand_kind,
            self.state.blind,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::RoundEnd, &mut args, events);
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
                EffectOp::SetMoney(value) => *money = *value,
                EffectOp::DoubleMoney { cap } => {
                    let gain = (*money).min(*cap);
                    *money = money.saturating_add(gain);
                }
                EffectOp::AddMoneyFromJokers { cap } => {
                    let total = self
                        .inventory
                        .jokers
                        .iter()
                        .map(|joker| self.calc_joker_sell_value(joker))
                        .sum::<i64>();
                    *money = money.saturating_add(total.min(*cap));
                }
                EffectOp::AddHandSize(value) => {
                    let next = (self.state.hand_size as i64 + value).max(0) as usize;
                    self.state.hand_size = next;
                }
                _ => {}
            }
        }
        self.pending_effects.clear();
    }

    pub(super) fn apply_effect_blocks(
        &mut self,
        blocks: &[EffectBlock],
        trigger: ActivationType,
        hand_kind: crate::HandKind,
        card: Option<Card>,
        selected: &[usize],
        score: &mut Score,
        money: &mut i64,
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        for block in blocks {
            if block.trigger != trigger {
                continue;
            }
            if !self.json_conditions_met(&block.conditions, hand_kind, card) {
                continue;
            }
            self.validate_effect_selection(&block.effects, selected)?;
            self.apply_effect_ops(&block.effects, selected, score, money, events)?;
        }
        Ok(())
    }

    pub fn use_consumable(
        &mut self,
        index: usize,
        selected: &[usize],
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        if index >= self.inventory.consumables.len() {
            return Err(RunError::InvalidSelection);
        }
        let instance = self.inventory.consumables[index].clone();
        let def = self
            .content
            .tarots
            .iter()
            .chain(self.content.planets.iter())
            .chain(self.content.spectrals.iter())
            .find(|card| card.id == instance.id)
            .cloned()
            .ok_or(RunError::InvalidSelection)?;
        self.validate_consumable_selection(&def, selected)?;
        let _ = self.inventory.consumables.remove(index);
        self.apply_consumable_effects(&def, selected, events)
    }

    pub(super) fn apply_consumable_effects(
        &mut self,
        def: &crate::ConsumableDef,
        selected: &[usize],
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        self.validate_consumable_selection(def, selected)?;
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        if def.kind == crate::ConsumableKind::Planet {
            self.state.planets_used.insert(def.id.clone());
        }
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::consumable(
            hand_kind,
            self.state.blind,
            def.kind,
            &def.id,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::UseConsumable, &mut args, events);
        for block in &def.effects {
            if block.trigger != ActivationType::OnUse {
                continue;
            }
            if !self.json_conditions_met(&block.conditions, crate::HandKind::HighCard, None) {
                continue;
            }
            self.apply_effect_ops(&block.effects, selected, &mut scratch_score, &mut money, events)?;
        }
        self.state.money = money;
        if matches!(
            def.kind,
            crate::ConsumableKind::Tarot | crate::ConsumableKind::Planet
        ) && !def.id.eq_ignore_ascii_case("the_fool")
        {
            self.state.last_consumable = Some(crate::LastConsumable {
                kind: def.kind,
                id: def.id.clone(),
            });
        }
        Ok(())
    }

    fn apply_effect_ops(
        &mut self,
        effects: &[EffectOp],
        selected: &[usize],
        _score: &mut Score,
        money: &mut i64,
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        for effect in effects {
            match effect {
                EffectOp::Score(rule) => self.pending_effects.push(EffectOp::Score(rule.clone())),
                EffectOp::AddMoney(value) => *money += value,
                EffectOp::SetMoney(value) => *money = *value,
                EffectOp::DoubleMoney { cap } => {
                    let gain = (*money).min(*cap);
                    *money = money.saturating_add(gain);
                }
                EffectOp::AddMoneyFromJokers { cap } => {
                    let total = self
                        .inventory
                        .jokers
                        .iter()
                        .map(|joker| self.calc_joker_sell_value(joker))
                        .sum::<i64>();
                    *money = money.saturating_add(total.min(*cap));
                }
                EffectOp::AddHandSize(value) => {
                    let next = (self.state.hand_size as i64 + value).max(0) as usize;
                    self.state.hand_size = next;
                }
                EffectOp::UpgradeHand { hand, amount } => {
                    self.upgrade_hand_level(*hand, *amount);
                }
                EffectOp::UpgradeAllHands { amount } => {
                    self.upgrade_all_hands(*amount);
                }
                EffectOp::AddRandomConsumable { kind, count } => {
                    for _ in 0..*count {
                        if let Some(card) = self.content.pick_consumable(*kind, &mut self.rng) {
                            let _ = self.inventory.add_consumable(card.id.clone(), *kind);
                        }
                    }
                }
                EffectOp::AddJoker { rarity, count } => {
                    for _ in 0..*count {
                        self.add_joker_from_rarity(*rarity);
                    }
                }
                EffectOp::AddRandomJoker { count } => {
                    for _ in 0..*count {
                        let idx = if self.content.jokers.is_empty() {
                            None
                        } else {
                            Some((self.rng.next_u64() % self.content.jokers.len() as u64) as usize)
                        };
                        if let Some(idx) = idx {
                            if let Some(def) = self.content.jokers.get(idx) {
                                self.add_joker_from_rarity(def.rarity);
                            }
                        }
                    }
                }
                EffectOp::RandomJokerEdition { editions, chance } => {
                    if editions.is_empty() {
                        continue;
                    }
                    let roll = (self.rng.next_u64() % 1000) as f64 / 1000.0;
                    if roll > *chance {
                        continue;
                    }
                    let candidates: Vec<usize> = self
                        .inventory
                        .jokers
                        .iter()
                        .enumerate()
                        .filter(|(_, joker)| joker.edition.is_none())
                        .map(|(idx, _)| idx)
                        .collect();
                    if candidates.is_empty() {
                        continue;
                    }
                    let pick = candidates[(self.rng.next_u64() % candidates.len() as u64) as usize];
                    let edition = editions[(self.rng.next_u64() % editions.len() as u64) as usize];
                    if let Some(joker) = self.inventory.jokers.get_mut(pick) {
                        joker.edition = Some(edition);
                    }
                    self.mark_rules_dirty();
                }
                EffectOp::SetRandomJokerEdition { edition } => {
                    let Some(pick) = random_joker_index(&mut self.rng, self.inventory.jokers.len()) else {
                        continue;
                    };
                    if let Some(joker) = self.inventory.jokers.get_mut(pick) {
                        joker.edition = Some(*edition);
                    }
                    self.mark_rules_dirty();
                }
                EffectOp::SetRandomJokerEditionDestroyOthers { edition } => {
                    if self.inventory.jokers.is_empty() {
                        continue;
                    }
                    let pick = (self.rng.next_u64() % self.inventory.jokers.len() as u64) as usize;
                    if let Some(joker) = self.inventory.jokers.get_mut(pick) {
                        joker.edition = Some(*edition);
                    }
                    let keep = self.inventory.jokers.get(pick).cloned();
                    self.inventory.jokers.clear();
                    if let Some(joker) = keep {
                        self.inventory.jokers.push(joker);
                    }
                    self.mark_rules_dirty();
                }
                EffectOp::DuplicateRandomJokerDestroyOthers { remove_negative } => {
                    if self.inventory.jokers.is_empty() {
                        continue;
                    }
                    let pick = (self.rng.next_u64() % self.inventory.jokers.len() as u64) as usize;
                    let original = self.inventory.jokers.get(pick).cloned();
                    let mut copy = original.clone();
                    if let Some(copy) = copy.as_mut() {
                        if *remove_negative && copy.edition == Some(Edition::Negative) {
                            copy.edition = None;
                        }
                    }
                    self.inventory.jokers.clear();
                    if let Some(joker) = original {
                        self.inventory.jokers.push(joker);
                    }
                    if let Some(copy) = copy {
                        if self.inventory.jokers.len() < self.inventory.joker_capacity() {
                            self.inventory.jokers.push(copy);
                        }
                    }
                    self.mark_rules_dirty();
                }
                EffectOp::EnhanceSelected { enhancement, count } => {
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    for idx in indices {
                        if let Some(card) = self.hand.get_mut(idx) {
                            card.enhancement = Some(*enhancement);
                        }
                    }
                }
                EffectOp::AddEditionToSelected { editions, count } => {
                    if editions.is_empty() {
                        continue;
                    }
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    for idx in indices {
                        if let Some(card) = self.hand.get_mut(idx) {
                            let pick = (self.rng.next_u64() % editions.len() as u64) as usize;
                            card.edition = Some(editions[pick]);
                        }
                    }
                }
                EffectOp::AddSealToSelected { seal, count } => {
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    for idx in indices {
                        if let Some(card) = self.hand.get_mut(idx) {
                            card.seal = Some(*seal);
                        }
                    }
                }
                EffectOp::ConvertSelectedSuit { suit, count } => {
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    for idx in indices {
                        if let Some(card) = self.hand.get_mut(idx) {
                            card.suit = *suit;
                        }
                    }
                }
                EffectOp::IncreaseSelectedRank { count, delta } => {
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    for idx in indices {
                        if let Some(card) = self.hand.get_mut(idx) {
                            card.rank = shift_rank(card.rank, *delta);
                        }
                    }
                }
                EffectOp::DestroySelected { count } => {
                    let indices = self.select_indices(selected, *count as usize, true)?;
                    self.destroy_hand_cards(&indices, events);
                }
                EffectOp::DestroyRandomInHand { count } => {
                    let indices = self.random_indices(*count as usize);
                    self.destroy_hand_cards(&indices, events);
                }
                EffectOp::CopySelected { count } => {
                    if *count == 0 {
                        continue;
                    }
                    let indices = self.select_indices(selected, 1, true)?;
                    let Some(&idx) = indices.first() else {
                        continue;
                    };
                    let Some(card) = self.hand.get(idx).copied() else {
                        continue;
                    };
                    for _ in 0..*count {
                        let mut copy = card;
                        copy.face_down = false;
                        self.assign_card_id(&mut copy);
                        self.hand.push(copy);
                        self.trigger_on_card_added(copy);
                    }
                }
                EffectOp::ConvertLeftIntoRight => {
                    let (left, right) = self.select_pair(selected, true)?;
                    let Some(right_card) = self.hand.get(right).copied() else {
                        continue;
                    };
                    if let Some(left_card) = self.hand.get_mut(left) {
                        left_card.suit = right_card.suit;
                        left_card.rank = right_card.rank;
                        left_card.enhancement = right_card.enhancement;
                        left_card.edition = right_card.edition;
                        left_card.seal = right_card.seal;
                        left_card.bonus_chips = right_card.bonus_chips;
                    }
                }
                EffectOp::ConvertHandToRandomRank => {
                    if self.hand.is_empty() {
                        continue;
                    }
                    let rank = random_standard_rank(&mut self.rng);
                    for card in &mut self.hand {
                        card.rank = rank;
                    }
                }
                EffectOp::ConvertHandToRandomSuit => {
                    if self.hand.is_empty() {
                        continue;
                    }
                    let suit = random_standard_suit(&mut self.rng);
                    for card in &mut self.hand {
                        card.suit = suit;
                    }
                }
                EffectOp::AddRandomEnhancedCards { count, filter } => {
                    for _ in 0..*count {
                        let mut card = crate::Card::standard(
                            random_standard_suit(&mut self.rng),
                            random_rank_filtered(&mut self.rng, *filter),
                        );
                        card.enhancement = Some(random_enhancement(&mut self.rng));
                        self.assign_card_id(&mut card);
                        self.hand.push(card);
                        self.trigger_on_card_added(card);
                    }
                }
                EffectOp::CreateLastConsumable { exclude } => {
                    let Some(last) = self.state.last_consumable.clone() else {
                        continue;
                    };
                    if exclude
                        .as_ref()
                        .map(|value| value.eq_ignore_ascii_case(&last.id))
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    let _ = self.inventory.add_consumable(last.id, last.kind);
                }
                EffectOp::RetriggerScored(_) | EffectOp::RetriggerHeld(_) => {}
            }
        }
        Ok(())
    }

    fn validate_effect_selection(
        &self,
        effects: &[EffectOp],
        selected: &[usize],
    ) -> Result<(), RunError> {
        let mut requires_selection = false;
        for effect in effects {
            match effect {
                EffectOp::EnhanceSelected { count, .. }
                | EffectOp::AddEditionToSelected { count, .. }
                | EffectOp::AddSealToSelected { count, .. }
                | EffectOp::ConvertSelectedSuit { count, .. }
                | EffectOp::IncreaseSelectedRank { count, .. }
                | EffectOp::DestroySelected { count }
                | EffectOp::CopySelected { count } => {
                    if *count > 0 {
                        requires_selection = true;
                        if !selected.is_empty() && selected.len() > *count as usize {
                            return Err(RunError::InvalidCardCount);
                        }
                    }
                }
                EffectOp::ConvertLeftIntoRight => {
                    requires_selection = true;
                    if !selected.is_empty() && selected.len() != 2 {
                        return Err(RunError::InvalidCardCount);
                    }
                }
                _ => {}
            }
        }
        if requires_selection && selected.is_empty() {
            return Err(RunError::InvalidSelection);
        }
        Ok(())
    }

    fn validate_consumable_selection(
        &self,
        def: &crate::ConsumableDef,
        selected: &[usize],
    ) -> Result<(), RunError> {
        for block in &def.effects {
            if block.trigger != ActivationType::OnUse {
                continue;
            }
            self.validate_effect_selection(&block.effects, selected)?;
        }
        Ok(())
    }

    fn select_indices(
        &mut self,
        selected: &[usize],
        max: usize,
        require: bool,
    ) -> Result<Vec<usize>, RunError> {
        if max == 0 || self.hand.is_empty() {
            return Ok(Vec::new());
        }
        if selected.is_empty() {
            if require {
                return Err(RunError::InvalidSelection);
            }
            let mut indices: Vec<usize> = (0..self.hand.len()).collect();
            self.rng.shuffle(&mut indices);
            indices.truncate(max.min(indices.len()));
            return Ok(indices);
        }
        if selected.len() > max {
            return Err(RunError::InvalidCardCount);
        }
        let mut seen = std::collections::HashSet::new();
        for &idx in selected {
            if idx >= self.hand.len() {
                return Err(RunError::InvalidSelection);
            }
            if !seen.insert(idx) {
                return Err(RunError::InvalidSelection);
            }
        }
        Ok(selected.to_vec())
    }

    fn select_pair(&mut self, selected: &[usize], require: bool) -> Result<(usize, usize), RunError> {
        if self.hand.len() < 2 {
            return Err(RunError::InvalidSelection);
        }
        if selected.len() >= 2 {
            if require && selected.len() != 2 {
                return Err(RunError::InvalidCardCount);
            }
            let mut left = selected[0];
            let mut right = selected[1];
            if left >= self.hand.len() || right >= self.hand.len() {
                return Err(RunError::InvalidSelection);
            }
            if left == right {
                return Err(RunError::InvalidSelection);
            }
            if left > right {
                std::mem::swap(&mut left, &mut right);
            }
            return Ok((left, right));
        }
        if require {
            return Err(RunError::InvalidSelection);
        }
        let mut indices: Vec<usize> = (0..self.hand.len()).collect();
        self.rng.shuffle(&mut indices);
        Ok((indices[0], indices[1]))
    }

    fn random_indices(&mut self, count: usize) -> Vec<usize> {
        if count == 0 || self.hand.is_empty() {
            return Vec::new();
        }
        let mut indices: Vec<usize> = (0..self.hand.len()).collect();
        self.rng.shuffle(&mut indices);
        indices.truncate(count.min(indices.len()));
        indices
    }

    fn destroy_hand_cards(&mut self, indices: &[usize], events: &mut EventBus) {
        if indices.is_empty() {
            return;
        }
        let mut unique: Vec<usize> = indices.to_vec();
        unique.sort_unstable();
        unique.dedup();
        if unique.iter().any(|&idx| idx >= self.hand.len()) {
            return;
        }
        unique.sort_unstable_by(|a, b| b.cmp(a));
        for idx in unique {
            let card = self.hand.remove(idx);
            self.trigger_card_destroyed(card, events);
        }
    }

    fn trigger_card_destroyed(&mut self, card: Card, events: &mut EventBus) {
        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut money = self.state.money;
        let mut results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::scoring(
            hand_kind,
            self.state.blind,
            card,
            0,
            HookInject::held(&mut held_view),
            None,
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::CardDestroyed, &mut args, events);
        self.state.money = money;
    }
}

fn random_joker_index(rng: &mut crate::RngState, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some((rng.next_u64() % len as u64) as usize)
    }
}

fn random_standard_suit(rng: &mut crate::RngState) -> Suit {
    match rng.next_u64() % 4 {
        0 => Suit::Spades,
        1 => Suit::Hearts,
        2 => Suit::Clubs,
        _ => Suit::Diamonds,
    }
}

fn random_standard_rank(rng: &mut crate::RngState) -> Rank {
    const RANKS: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];
    let idx = (rng.next_u64() % RANKS.len() as u64) as usize;
    RANKS[idx]
}

fn random_rank_filtered(rng: &mut crate::RngState, filter: RankFilter) -> Rank {
    match filter {
        RankFilter::Any => random_standard_rank(rng),
        RankFilter::Ace => Rank::Ace,
        RankFilter::Face => match rng.next_u64() % 3 {
            0 => Rank::Jack,
            1 => Rank::Queen,
            _ => Rank::King,
        },
        RankFilter::Numbered => {
            const RANKS: [Rank; 9] = [
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
            ];
            let idx = (rng.next_u64() % RANKS.len() as u64) as usize;
            RANKS[idx]
        }
    }
}

fn random_enhancement(rng: &mut crate::RngState) -> Enhancement {
    const ENHANCEMENTS: [Enhancement; 8] = [
        Enhancement::Bonus,
        Enhancement::Mult,
        Enhancement::Wild,
        Enhancement::Glass,
        Enhancement::Steel,
        Enhancement::Stone,
        Enhancement::Lucky,
        Enhancement::Gold,
    ];
    let idx = (rng.next_u64() % ENHANCEMENTS.len() as u64) as usize;
    ENHANCEMENTS[idx]
}

fn shift_rank(rank: Rank, delta: i8) -> Rank {
    const RANKS: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];
    let Some(pos) = RANKS.iter().position(|&r| r == rank) else {
        return rank;
    };
    let len = RANKS.len() as i8;
    let mut next = pos as i8 + delta;
    while next < 0 {
        next += len;
    }
    let idx = (next % len) as usize;
    RANKS[idx]
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
