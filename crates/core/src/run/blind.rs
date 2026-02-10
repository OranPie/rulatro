use super::*;
use crate::*;

impl RunState {
    pub fn start_blind(
        &mut self,
        ante: u8,
        blind: BlindKind,
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        let (hands, discards) = self
            .config
            .blind_rule(blind)
            .map(|rule| (rule.hands, rule.discards))
            .ok_or(RunError::MissingBlindRule(blind))?;
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
        self.hand.clear();
        self.shop = None;

        let ctx = EvalContext::independent(
            crate::HandKind::HighCard,
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
            ActivationType::OnBlindStart,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
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

    pub fn blind_cleared(&self) -> bool {
        self.state.target > 0 && self.state.blind_score >= self.state.target
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
                self.resolve_round_end_effects();
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
                self.resolve_round_end_effects();
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
