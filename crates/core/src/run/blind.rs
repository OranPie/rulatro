use super::*;
use crate::*;

impl RunState {
    pub fn start_blind(
        &mut self,
        ante: u8,
        blind: BlindKind,
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        let prev_ante = self.state.ante;
        if self.state.phase == Phase::Shop {
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
            self.invoke_hooks(HookPoint::ShopExit, &mut args, events);
            self.state.money = money;
        }
        let (hands, discards) = self
            .config
            .blind_rule(blind)
            .map(|rule| (rule.hands, rule.discards))
            .ok_or(RunError::MissingBlindRule(blind))?;
        let hands = hands.saturating_add(self.voucher_hands_bonus());
        let discards = discards.saturating_add(self.voucher_discards_bonus());
        let target = self
            .config
            .target_for(ante, blind)
            .ok_or(RunError::MissingAnteRule(ante))?;

        self.state.ante = ante;
        self.state.blind = blind;
        self.state.phase = Phase::Deal;
        self.state.target = target;
        self.state.blind_score = 0;
        self.state.hands_left = hands;
        self.state.discards_left = discards;
        self.state.hand_size = self.state.hand_size_base;
        self.state.last_hand = None;
        self.state.round_hand_types.clear();
        self.state.round_hand_lock = None;
        if blind == BlindKind::Small && ante != prev_ante {
            self.state.played_card_ids_ante.clear();
        }
        self.hand.clear();
        self.shop = None;
        self.boss_disabled = false;
        self.state.boss_id = None;
        if blind == BlindKind::Boss && self.boss_disable_pending {
            self.boss_disabled = true;
            self.boss_disable_pending = false;
        }
        if blind == BlindKind::Boss && !self.boss_disabled() {
            if let Some(boss) = self.content.pick_boss(&mut self.rng) {
                self.state.boss_id = Some(boss.id.clone());
            }
        }
        self.mark_rules_dirty();

        let mut scratch_score = Score::default();
        let mut money = self.state.money;
        let mut results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut args = HookArgs::independent(
            crate::HandKind::HighCard,
            self.state.blind,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
        );
        self.invoke_hooks(HookPoint::BlindStart, &mut args, events);
        self.state.money = money;
        self.state.hands_max = self.state.hands_left;
        self.state.discards_max = self.state.discards_left;

        events.push(Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        });
        Ok(())
    }

    pub fn start_current_blind(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        self.start_blind(self.state.ante, self.state.blind, events)
    }

    pub fn advance_blind(&mut self) -> Result<(), RunError> {
        let (next_ante, next_blind) = match self.state.blind {
            BlindKind::Small => (self.state.ante, BlindKind::Big),
            BlindKind::Big => (self.state.ante, BlindKind::Boss),
            BlindKind::Boss => (self.state.ante.saturating_add(1), BlindKind::Small),
        };

        if self.config.ante_rule(next_ante).is_none() {
            return Err(RunError::MissingAnteRule(next_ante));
        }

        self.state.ante = next_ante;
        self.state.blind = next_blind;
        Ok(())
    }

    pub fn start_next_blind(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        self.advance_blind()?;
        self.start_current_blind(events)
    }

    pub fn skip_blind(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Deal {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        if self.state.blind == BlindKind::Boss {
            return Err(RunError::CannotSkipBoss);
        }
        let tag = self.pick_skip_tag();
        if let Some(tag_id) = tag.clone() {
            self.state.tags.push(tag_id);
            self.mark_rules_dirty();
        }
        self.state.blinds_skipped = self.state.blinds_skipped.saturating_add(1);
        events.push(Event::BlindSkipped {
            ante: self.state.ante,
            blind: self.state.blind,
            tag,
        });
        self.advance_blind()?;
        self.start_current_blind(events)
    }

    pub fn blind_cleared(&self) -> bool {
        self.state.target > 0 && self.state.blind_score >= self.state.target
    }

    fn pick_skip_tag(&mut self) -> Option<String> {
        if self.content.tags.is_empty() {
            return None;
        }
        let idx = (self.rng.next_u64() % self.content.tags.len() as u64) as usize;
        self.content.tags.get(idx).map(|tag| tag.id.clone())
    }

    pub fn blind_outcome(&self) -> Option<BlindOutcome> {
        if self.blind_cleared() {
            Some(BlindOutcome::Cleared)
        } else if self.state.hands_left == 0 {
            Some(BlindOutcome::Failed)
        } else {
            None
        }
    }

    pub(super) fn check_outcome(&mut self, events: &mut EventBus) -> Option<BlindOutcome> {
        match self.blind_outcome() {
            Some(BlindOutcome::Cleared) => {
                self.resolve_round_end_effects(events);
                let reward = self.reward_for_clear();
                self.state.money += reward;
                events.push(Event::BlindCleared {
                    score: self.state.blind_score,
                    reward,
                    money: self.state.money,
                });
                Some(BlindOutcome::Cleared)
            }
            Some(BlindOutcome::Failed) => {
                self.prevent_death = false;
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
                self.invoke_hooks(HookPoint::BlindFailed, &mut args, events);
                self.state.money = money;
                if self.prevent_death {
                    self.prevent_death = false;
                    self.state.blind_score = self.state.target;
                    self.resolve_round_end_effects(events);
                    let reward = 0;
                    events.push(Event::BlindCleared {
                        score: self.state.blind_score,
                        reward,
                        money: self.state.money,
                    });
                    return Some(BlindOutcome::Cleared);
                }
                self.resolve_round_end_effects(events);
                events.push(Event::BlindFailed {
                    score: self.state.blind_score,
                });
                Some(BlindOutcome::Failed)
            }
            None => None,
        }
    }

    pub(super) fn reward_for_clear(&self) -> i64 {
        let economy = &self.config.economy;
        let base = match self.state.blind {
            BlindKind::Small => economy.reward_small,
            BlindKind::Big => economy.reward_big,
            BlindKind::Boss => economy.reward_boss,
        };
        let mut reward = base;
        reward += economy.per_hand_reward * self.state.hands_left as i64;
        reward + self.interest_earned()
    }

    pub(super) fn interest_earned(&self) -> i64 {
        let economy = &self.config.economy;
        if economy.interest_step <= 0 || economy.interest_per <= 0 {
            return 0;
        }
        let steps = (self.state.money / economy.interest_step).max(0);
        let cap_steps = if economy.interest_per > 0 {
            economy.interest_cap / economy.interest_per
        } else {
            0
        };
        let capped = steps.min(cap_steps);
        capped * economy.interest_per
    }
}
