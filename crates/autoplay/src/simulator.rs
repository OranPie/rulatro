use crate::{AutoAction, AutoplayConfig, AutoplayError, EvalMetrics};
use rulatro_core::{
    BlindOutcome, ConsumableKind, EffectOp, EventBus, PackOpen, Phase, Rank, RunState, ShopOfferRef,
};
use std::cmp::Reverse;

#[derive(Debug)]
pub struct Simulator {
    pub run: RunState,
    pub events: EventBus,
    pub open_pack: Option<PackOpen>,
}

impl Simulator {
    pub fn new(run: RunState) -> Self {
        Self {
            run,
            events: EventBus::default(),
            open_pack: None,
        }
    }

    pub fn metrics(&self) -> EvalMetrics {
        EvalMetrics {
            ante: self.run.state.ante,
            money: self.run.state.money,
            blind_score: self.run.state.blind_score,
            blind_target: self.run.state.target,
            blind_failed: self.run.blind_outcome() == Some(BlindOutcome::Failed),
            blind_cleared: self.run.blind_outcome() == Some(BlindOutcome::Cleared),
        }
    }

    pub fn phase_name(&self) -> String {
        format!("{:?}", self.run.state.phase)
    }

    pub fn blind_name(&self) -> String {
        format!("{:?}", self.run.state.blind)
    }

    pub fn apply_action(&mut self, action: &AutoAction) -> Result<usize, AutoplayError> {
        match action {
            AutoAction::Deal => self
                .run
                .prepare_hand(&mut self.events)
                .map_err(|err| AutoplayError::Run(err.to_string()))?,
            AutoAction::Play { indices } => {
                self.run
                    .play_hand(indices, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
            }
            AutoAction::Discard { indices } => self
                .run
                .discard(indices, &mut self.events)
                .map_err(|err| AutoplayError::Run(err.to_string()))?,
            AutoAction::SkipBlind => self
                .run
                .skip_blind(&mut self.events)
                .map_err(|err| AutoplayError::Run(err.to_string()))?,
            AutoAction::EnterShop => self
                .run
                .enter_shop(&mut self.events)
                .map_err(|err| AutoplayError::Run(err.to_string()))?,
            AutoAction::LeaveShop => {
                self.run.leave_shop();
                self.open_pack = None;
            }
            AutoAction::RerollShop => self
                .run
                .reroll_shop(&mut self.events)
                .map_err(|err| AutoplayError::Run(err.to_string()))?,
            AutoAction::BuyCard { index } => {
                let purchase = self
                    .run
                    .buy_shop_offer(ShopOfferRef::Card(*index), &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.run
                    .apply_purchase(&purchase)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
            }
            AutoAction::BuyPack { index } => {
                let purchase = self
                    .run
                    .buy_shop_offer(ShopOfferRef::Pack(*index), &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                let open = self
                    .run
                    .open_pack_purchase(&purchase, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.open_pack = Some(open);
            }
            AutoAction::BuyVoucher { index } => {
                let purchase = self
                    .run
                    .buy_shop_offer(ShopOfferRef::Voucher(*index), &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.run
                    .apply_purchase(&purchase)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
            }
            AutoAction::PickPack { indices } => {
                let open = self
                    .open_pack
                    .clone()
                    .ok_or_else(|| AutoplayError::InvalidAction("no open pack".to_string()))?;
                self.run
                    .choose_pack_options(&open, indices, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.open_pack = None;
            }
            AutoAction::SkipPack => {
                let open = self
                    .open_pack
                    .clone()
                    .ok_or_else(|| AutoplayError::InvalidAction("no open pack".to_string()))?;
                self.run
                    .skip_pack(&open, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.open_pack = None;
            }
            AutoAction::UseConsumable { index, selected } => {
                self.run
                    .use_consumable(*index, selected, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
            }
            AutoAction::SellJoker { index } => {
                self.run
                    .sell_joker(*index, &mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
            }
            AutoAction::NextBlind => {
                self.run
                    .start_next_blind(&mut self.events)
                    .map_err(|err| AutoplayError::Run(err.to_string()))?;
                self.open_pack = None;
            }
        }
        let event_count = self.events.drain().count();
        Ok(event_count)
    }

    pub fn legal_actions(&self, cfg: &AutoplayConfig) -> Vec<AutoAction> {
        if let Some(open) = self.open_pack.as_ref() {
            return legal_pack_actions(open, cfg.max_shop_candidates);
        }

        let mut actions = Vec::new();
        match self.run.state.phase {
            Phase::Deal => {
                actions.push(AutoAction::Deal);
                if self.run.state.blind != rulatro_core::BlindKind::Boss {
                    actions.push(AutoAction::SkipBlind);
                }
            }
            Phase::Play => {
                actions.extend(legal_play_actions(&self.run, cfg.max_play_candidates));
                if self.run.state.discards_left > 0 {
                    actions.extend(legal_discard_actions(&self.run, cfg.max_discard_candidates));
                }
                actions.extend(legal_consumable_actions(&self.run, cfg.max_play_candidates));
            }
            Phase::Shop => {
                actions.push(AutoAction::RerollShop);
                actions.push(AutoAction::LeaveShop);
                if let Some(shop) = self.run.shop.as_ref() {
                    for idx in 0..shop.cards.len().min(cfg.max_shop_candidates) {
                        actions.push(AutoAction::BuyCard { index: idx });
                    }
                    for idx in 0..shop.packs.len().min(cfg.max_shop_candidates) {
                        actions.push(AutoAction::BuyPack { index: idx });
                    }
                    for idx in 0..shop.vouchers.min(cfg.max_shop_candidates) {
                        actions.push(AutoAction::BuyVoucher { index: idx });
                    }
                }
                for idx in 0..self.run.inventory.jokers.len().min(cfg.max_shop_candidates) {
                    actions.push(AutoAction::SellJoker { index: idx });
                }
            }
            Phase::Cleanup | Phase::Setup | Phase::Score => {}
        }

        if self.run.blind_outcome() == Some(BlindOutcome::Cleared) {
            if self.run.state.phase != Phase::Shop {
                actions.push(AutoAction::EnterShop);
            }
            actions.push(AutoAction::NextBlind);
        }

        actions.sort_by_key(|item| item.stable_key());
        actions.dedup_by_key(|item| item.stable_key());
        actions
    }
}

fn legal_pack_actions(open: &PackOpen, cap: usize) -> Vec<AutoAction> {
    let mut actions = vec![AutoAction::SkipPack];
    let max_pick = usize::from(open.offer.picks.max(1));
    let mut combos = Vec::new();
    for pick_count in 1..=max_pick.min(open.options.len()) {
        enumerate_combinations(open.options.len(), pick_count, &mut combos);
    }
    combos.sort();
    for indices in combos.into_iter().take(cap.max(1)) {
        actions.push(AutoAction::PickPack { indices });
    }
    actions
}

fn legal_play_actions(run: &RunState, cap: usize) -> Vec<AutoAction> {
    let max_cards = run.hand.len().min(5);
    if max_cards == 0 {
        return Vec::new();
    }
    let mut combos = Vec::new();
    for count in 1..=max_cards {
        enumerate_combinations(run.hand.len(), count, &mut combos);
    }
    let mut scored: Vec<(i64, Vec<usize>)> = combos
        .into_iter()
        .map(|indices| {
            let value = indices
                .iter()
                .map(|idx| card_eval_value(&run.hand[*idx], &run.tables))
                .sum::<i64>();
            (value, indices)
        })
        .collect();
    scored.sort_by_key(|(value, indices)| (Reverse(*value), indices.clone()));
    scored
        .into_iter()
        .take(cap.max(1))
        .map(|(_, indices)| AutoAction::Play { indices })
        .collect()
}

fn legal_discard_actions(run: &RunState, cap: usize) -> Vec<AutoAction> {
    let max_cards = run.hand.len().min(5);
    if max_cards == 0 {
        return Vec::new();
    }
    let mut combos = Vec::new();
    for count in 1..=max_cards {
        enumerate_combinations(run.hand.len(), count, &mut combos);
    }
    let mut scored: Vec<(i64, Vec<usize>)> = combos
        .into_iter()
        .map(|indices| {
            let value = indices
                .iter()
                .map(|idx| card_eval_value(&run.hand[*idx], &run.tables))
                .sum::<i64>();
            (value, indices)
        })
        .collect();
    scored.sort_by_key(|(value, indices)| (*value, indices.clone()));
    scored
        .into_iter()
        .take(cap.max(1))
        .map(|(_, indices)| AutoAction::Discard { indices })
        .collect()
}

fn legal_consumable_actions(run: &RunState, cap: usize) -> Vec<AutoAction> {
    let mut actions = Vec::new();
    for (idx, item) in run.inventory.consumables.iter().enumerate() {
        let maybe_def = match item.kind {
            ConsumableKind::Tarot => run.content.tarots.iter().find(|card| card.id == item.id),
            ConsumableKind::Planet => run.content.planets.iter().find(|card| card.id == item.id),
            ConsumableKind::Spectral => {
                run.content.spectrals.iter().find(|card| card.id == item.id)
            }
        };
        let Some(def) = maybe_def else {
            continue;
        };
        let required = required_selected_count(&def.effects);
        if required == 0 {
            actions.push(AutoAction::UseConsumable {
                index: idx,
                selected: Vec::new(),
            });
            continue;
        }
        let mut combos = Vec::new();
        enumerate_combinations(run.hand.len(), required.min(run.hand.len()), &mut combos);
        combos.sort();
        for selected in combos.into_iter().take(cap.max(1)) {
            actions.push(AutoAction::UseConsumable {
                index: idx,
                selected,
            });
        }
    }
    actions
}

fn required_selected_count(blocks: &[rulatro_core::EffectBlock]) -> usize {
    let mut required = 0usize;
    for block in blocks {
        for effect in &block.effects {
            match effect {
                EffectOp::EnhanceSelected { count, .. }
                | EffectOp::AddEditionToSelected { count, .. }
                | EffectOp::AddSealToSelected { count, .. }
                | EffectOp::ConvertSelectedSuit { count, .. }
                | EffectOp::DestroySelected { count }
                | EffectOp::CopySelected { count } => {
                    required = required.max(*count as usize);
                }
                EffectOp::IncreaseSelectedRank { count, .. } => {
                    required = required.max(*count as usize);
                }
                EffectOp::ConvertLeftIntoRight => {
                    required = required.max(2);
                }
                _ => {}
            }
        }
    }
    required
}

fn enumerate_combinations(n: usize, k: usize, out: &mut Vec<Vec<usize>>) {
    if n == 0 || k == 0 || k > n {
        return;
    }
    let mut current = Vec::with_capacity(k);
    recurse_combinations(0, n, k, &mut current, out);
}

fn recurse_combinations(
    start: usize,
    n: usize,
    k: usize,
    current: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    if current.len() == k {
        out.push(current.clone());
        return;
    }
    let remaining = k - current.len();
    let max_idx = n - remaining;
    for idx in start..=max_idx {
        current.push(idx);
        recurse_combinations(idx + 1, n, k, current, out);
        current.pop();
    }
}

fn card_eval_value(card: &rulatro_core::Card, tables: &rulatro_core::ScoreTables) -> i64 {
    if card.is_stone() {
        0
    } else {
        tables.rank_chips(card.rank) + card.bonus_chips + rank_heuristic(card.rank)
    }
}

fn rank_heuristic(rank: Rank) -> i64 {
    match rank {
        Rank::Ace => 14,
        Rank::King => 13,
        Rank::Queen => 12,
        Rank::Jack => 11,
        Rank::Ten => 10,
        Rank::Nine => 9,
        Rank::Eight => 8,
        Rank::Seven => 7,
        Rank::Six => 6,
        Rank::Five => 5,
        Rank::Four => 4,
        Rank::Three => 3,
        Rank::Two => 2,
        Rank::Joker => 15,
    }
}
