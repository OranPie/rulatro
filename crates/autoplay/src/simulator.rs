use crate::{AnteTargetRecord, AutoAction, AutoplayConfig, AutoplayError, EvalMetrics};
use rulatro_core::{
    score_hand, voucher_by_id, BlindKind, BlindOutcome, Card, ConsumableKind, EffectOp, EventBus,
    HandKind, PackOpen, PackOption, Phase, Rank, RuleEffect, RunState, ShopCardKind, ShopOfferRef,
};

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
        phase_label(self.run.state.phase).to_string()
    }

    pub fn blind_name(&self) -> String {
        blind_label(self.run.state.blind).to_string()
    }

    pub fn collect_ante_targets(&self) -> Vec<AnteTargetRecord> {
        let mut antes: Vec<u8> = self.run.config.antes.iter().map(|rule| rule.ante).collect();
        antes.sort_unstable();
        antes.dedup();
        let mut out = Vec::new();
        for ante in antes {
            let Some(small) = self.run.config.target_for(ante, BlindKind::Small) else {
                continue;
            };
            let Some(big) = self.run.config.target_for(ante, BlindKind::Big) else {
                continue;
            };
            let Some(boss) = self.run.config.target_for(ante, BlindKind::Boss) else {
                continue;
            };
            out.push(AnteTargetRecord {
                ante,
                small,
                big,
                boss,
            });
        }
        out
    }

    pub fn describe_action(&self, action: &AutoAction) -> Option<String> {
        match action {
            AutoAction::Play { indices } => {
                let cards = cards_from_hand(&self.run.hand, indices);
                if cards.is_empty() {
                    return Some("play cards/出牌: (none/无)".to_string());
                }
                let breakdown = score_hand(&cards, &self.run.tables);
                let scoring_cards = breakdown
                    .scoring_indices
                    .iter()
                    .filter_map(|idx| cards.get(*idx))
                    .map(format_card)
                    .collect::<Vec<_>>();
                let selected_cards = cards.iter().map(format_card).collect::<Vec<_>>().join(", ");
                Some(format!(
                    "play cards/出牌: [{selected_cards}]\nestimate/估算: hand/牌型={:?} base/基础={}x{:.2} rank_chips/牌面筹码={} scoring/计分牌=[{}] est_total/预计总分={}",
                    breakdown.hand,
                    breakdown.base.chips,
                    breakdown.base.mult,
                    breakdown.rank_chips,
                    scoring_cards.join(", "),
                    breakdown.total.total()
                ))
            }
            AutoAction::Discard { indices } => {
                let cards = cards_from_hand(&self.run.hand, indices);
                let selected_cards = cards.iter().map(format_card).collect::<Vec<_>>().join(", ");
                Some(format!("discard cards/弃牌: [{selected_cards}]"))
            }
            AutoAction::BuyCard { index } => {
                let shop = self.run.shop.as_ref()?;
                let offer = shop.cards.get(*index)?;
                Some(describe_shop_card(&self.run, offer))
            }
            AutoAction::BuyPack { index } => {
                let shop = self.run.shop.as_ref()?;
                let offer = shop.packs.get(*index)?;
                Some(format!(
                    "buy pack/购买卡包: {:?} {:?} price/价格={} options/选项={} picks/可选={}",
                    offer.kind, offer.size, offer.price, offer.options, offer.picks
                ))
            }
            AutoAction::BuyVoucher { index } => {
                let shop = self.run.shop.as_ref()?;
                let voucher = shop.voucher_offers.get(*index)?;
                Some(describe_voucher(voucher.id.as_str()))
            }
            AutoAction::PickPack { indices } => {
                let open = self.open_pack.as_ref()?;
                let selected = indices
                    .iter()
                    .filter_map(|idx| open.options.get(*idx))
                    .map(|opt| format_pack_option(&self.run, opt))
                    .collect::<Vec<_>>();
                Some(format!(
                    "pick pack options/选择卡包项: [{}]\npack/卡包: {:?} {:?} picks/可选={}",
                    selected.join(", "),
                    open.offer.kind,
                    open.offer.size,
                    open.offer.picks
                ))
            }
            AutoAction::SkipPack => {
                let open = self.open_pack.as_ref()?;
                Some(format!(
                    "skip pack/跳过卡包: {:?} {:?} picks/可选={} options/选项={}",
                    open.offer.kind,
                    open.offer.size,
                    open.offer.picks,
                    open.options.len()
                ))
            }
            AutoAction::UseConsumable { index, selected } => {
                let item = self.run.inventory.consumables.get(*index)?;
                let name = find_consumable_name(&self.run, item.kind, item.id.as_str());
                let selected_cards = cards_from_hand(&self.run.hand, selected)
                    .iter()
                    .map(format_card)
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!(
                    "use consumable/使用消耗牌: {} {:?}\nselected cards/选中卡牌: [{}]",
                    name, item.kind, selected_cards
                ))
            }
            AutoAction::SellJoker { index } => {
                let joker = self.run.inventory.jokers.get(*index)?;
                let name = find_joker_name(&self.run, joker.id.as_str());
                let sell_value = self.run.joker_sell_value(*index).unwrap_or(0);
                Some(format!(
                    "sell joker/出售小丑: {} ({:?}) for/售价 {}",
                    name, joker.rarity, sell_value
                ))
            }
            AutoAction::EnterShop => {
                if let Some(shop) = self.run.shop.as_ref() {
                    Some(format!(
                        "enter existing shop/进入已有商店: cards/卡牌={} packs/卡包={} vouchers/优惠券={} reroll/刷新价={}",
                        shop.cards.len(),
                        shop.packs.len(),
                        shop.vouchers,
                        shop.reroll_cost
                    ))
                } else {
                    Some("enter shop/进入商店: generate new offers/生成新商品".to_string())
                }
            }
            AutoAction::LeaveShop => {
                Some("leave shop/离开商店 and continue run/继续流程".to_string())
            }
            AutoAction::RerollShop => {
                let cost = self
                    .run
                    .shop
                    .as_ref()
                    .map(|shop| shop.reroll_cost)
                    .unwrap_or(0);
                Some(format!("reroll shop offers/刷新商店商品: cost/费用={cost}"))
            }
            AutoAction::Deal => Some(format!(
                "deal/发牌 to hand size/手牌上限 {} (current hand/当前手牌={})",
                self.run.state.hand_size,
                self.run.hand.len()
            )),
            AutoAction::SkipBlind => Some(format!(
                "skip blind/跳过盲注 {} at ante/底注 {}",
                blind_label(self.run.state.blind),
                self.run.state.ante
            )),
            AutoAction::NextBlind => Some(format!(
                "advance blind/推进盲注: ante/底注 {} {} target/目标 {}",
                self.run.state.ante,
                blind_label(self.run.state.blind),
                self.run.state.target
            )),
        }
    }

    pub fn describe_score_detail(
        &self,
        action: &AutoAction,
        before_score: i64,
        before_target: i64,
        after_score: i64,
    ) -> Option<String> {
        let delta = after_score - before_score;
        if !matches!(action, AutoAction::Play { .. }) && delta == 0 {
            return None;
        }
        let mut lines = vec![format!(
            "blind score/盲注分数: {before_score} -> {after_score} ({:+}), target/目标 {} -> {}",
            delta, before_target, self.run.state.target
        )];
        if matches!(action, AutoAction::Play { .. }) {
            let trace = &self.run.last_score_trace;
            if trace.is_empty() {
                lines.push("score trace/得分追踪: (empty/无)".to_string());
            } else {
                lines.push(format!("score trace steps/得分追踪步骤: {}", trace.len()));
                for (idx, item) in trace.iter().take(12).enumerate() {
                    lines.push(format!(
                        "  {}. {} | {} | {}x{:.2} -> {}x{:.2} | total {} -> {}",
                        idx + 1,
                        item.source,
                        format_rule_effect(&item.effect),
                        item.before.chips,
                        item.before.mult,
                        item.after.chips,
                        item.after.mult,
                        item.before.total(),
                        item.after.total()
                    ));
                }
                if trace.len() > 12 {
                    lines.push(format!("  ... {} more/更多", trace.len() - 12));
                }
            }
        }
        Some(lines.join("\n"))
    }

    pub fn describe_ante_detail(
        &self,
        before_ante: u8,
        before_blind: BlindKind,
        before_target: i64,
    ) -> Option<String> {
        let after = &self.run.state;
        if before_ante == after.ante && before_blind == after.blind && before_target == after.target
        {
            return None;
        }
        let mut lines = vec![format!(
            "blind transition/盲注切换: ante/底注 {} {} target/目标 {} -> ante/底注 {} {} target/目标 {}",
            before_ante,
            blind_label(before_blind),
            before_target,
            after.ante,
            blind_label(after.blind),
            after.target
        )];
        if let Some(targets) = format_ante_targets(&self.run, after.ante) {
            if before_ante != after.ante {
                lines.push(format!(
                    "entered ante/进入底注 {} targets/目标: {targets}",
                    after.ante
                ));
            } else {
                lines.push(format!("ante/底注 {} targets/目标: {targets}", after.ante));
            }
        }
        Some(lines.join("\n"))
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
            let joker_free = self
                .run
                .inventory
                .joker_capacity()
                .saturating_sub(self.run.inventory.jokers.len());
            let consumable_free = self
                .run
                .inventory
                .consumable_slots
                .saturating_sub(self.run.inventory.consumable_count());
            return legal_pack_actions(open, cfg.max_shop_candidates, joker_free, consumable_free);
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
                actions.push(AutoAction::LeaveShop);
                if let Some(shop) = self.run.shop.as_ref() {
                    if self.run.state.money >= shop.reroll_cost.max(0) {
                        actions.push(AutoAction::RerollShop);
                    }
                }
                if let Some(shop) = self.run.shop.as_ref() {
                    for idx in 0..shop.cards.len().min(cfg.max_shop_candidates) {
                        let offer = &shop.cards[idx];
                        if !can_afford(self.run.state.money, offer.price) {
                            continue;
                        }
                        if can_buy_shop_card(&self.run, offer) {
                            actions.push(AutoAction::BuyCard { index: idx });
                        }
                    }
                    for idx in 0..shop.packs.len().min(cfg.max_shop_candidates) {
                        let offer = &shop.packs[idx];
                        if can_afford(self.run.state.money, offer.price) {
                            actions.push(AutoAction::BuyPack { index: idx });
                        }
                    }
                    for idx in 0..shop.vouchers.min(cfg.max_shop_candidates) {
                        if can_afford(self.run.state.money, self.run.config.shop.prices.voucher) {
                            actions.push(AutoAction::BuyVoucher { index: idx });
                        }
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

fn legal_pack_actions(
    open: &PackOpen,
    cap: usize,
    joker_free_slots: usize,
    consumable_free_slots: usize,
) -> Vec<AutoAction> {
    let mut actions = vec![AutoAction::SkipPack];
    let max_pick = usize::from(open.offer.picks.max(1));
    let mut combos = Vec::new();
    for pick_count in 1..=max_pick.min(open.options.len()) {
        enumerate_combinations(open.options.len(), pick_count, &mut combos);
    }
    combos.sort();
    for indices in combos {
        if !can_pick_pack_indices(open, &indices, joker_free_slots, consumable_free_slots) {
            continue;
        }
        actions.push(AutoAction::PickPack { indices });
        if actions.len() >= cap.max(1).saturating_add(1) {
            break;
        }
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
    let blind_deficit = (run.state.target - run.state.blind_score).max(0);
    let mut scored: Vec<PlayCandidate> = combos
        .into_iter()
        .map(|indices| {
            let cards: Vec<rulatro_core::Card> = indices.iter().map(|idx| run.hand[*idx]).collect();
            let breakdown = score_hand(&cards, &run.tables);
            let base_total = breakdown.total.total();
            let card_value = indices
                .iter()
                .map(|idx| card_eval_value(&run.hand[*idx], &run.tables))
                .sum::<i64>();
            let finisher_bonus = if base_total >= blind_deficit {
                100_000
            } else {
                0
            };
            let hand_bonus = hand_kind_weight(breakdown.hand) * 10;
            let score = finisher_bonus + base_total * 20 + hand_bonus + card_value;
            PlayCandidate {
                score,
                expected_total: base_total,
                hand: breakdown.hand,
                indices,
            }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.expected_total.cmp(&a.expected_total))
            .then_with(|| hand_kind_weight(b.hand).cmp(&hand_kind_weight(a.hand)))
            .then_with(|| a.indices.cmp(&b.indices))
    });

    let mut selected = Vec::new();
    let mut seen_hands = std::collections::HashSet::new();
    for item in &scored {
        if selected.len() >= cap.max(1) {
            break;
        }
        if seen_hands.insert(item.hand) {
            selected.push(AutoAction::Play {
                indices: item.indices.clone(),
            });
        }
    }
    for item in scored {
        if selected.len() >= cap.max(1) {
            break;
        }
        if selected.iter().any(|action| match action {
            AutoAction::Play { indices } => indices == &item.indices,
            _ => false,
        }) {
            continue;
        }
        selected.push(AutoAction::Play {
            indices: item.indices,
        });
    }
    selected
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
    let mut scored: Vec<(i64, usize, Vec<usize>)> = combos
        .into_iter()
        .map(|indices| {
            let mut kept = Vec::new();
            for (idx, card) in run.hand.iter().enumerate() {
                if !indices.contains(&idx) {
                    kept.push(*card);
                }
            }
            let keep_potential = estimate_keep_potential(run, &kept);
            let discard_value = indices
                .iter()
                .map(|idx| card_eval_value(&run.hand[*idx], &run.tables))
                .sum::<i64>();
            (keep_potential * 10 - discard_value, indices.len(), indices)
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    scored
        .into_iter()
        .take(cap.max(1))
        .map(|(_, _, indices)| AutoAction::Discard { indices })
        .collect()
}

#[derive(Debug, Clone)]
struct PlayCandidate {
    score: i64,
    expected_total: i64,
    hand: HandKind,
    indices: Vec<usize>,
}

fn hand_kind_weight(kind: HandKind) -> i64 {
    match kind {
        HandKind::HighCard => 1,
        HandKind::Pair => 2,
        HandKind::TwoPair => 3,
        HandKind::Trips => 4,
        HandKind::Straight => 5,
        HandKind::Flush => 6,
        HandKind::FullHouse => 7,
        HandKind::Quads => 8,
        HandKind::StraightFlush => 9,
        HandKind::RoyalFlush => 10,
        HandKind::FiveOfAKind => 11,
        HandKind::FlushHouse => 12,
        HandKind::FlushFive => 13,
    }
}

fn estimate_keep_potential(run: &RunState, kept: &[rulatro_core::Card]) -> i64 {
    if kept.is_empty() {
        return 0;
    }
    let mut best_total = 0i64;
    let max_pick = kept.len().min(5);
    for pick in 1..=max_pick {
        let mut combos = Vec::new();
        enumerate_combinations(kept.len(), pick, &mut combos);
        for combo in combos {
            let cards: Vec<rulatro_core::Card> = combo.iter().map(|idx| kept[*idx]).collect();
            let breakdown = score_hand(&cards, &run.tables);
            best_total = best_total.max(breakdown.total.total());
        }
    }

    let mut top_values: Vec<i64> = kept
        .iter()
        .map(|card| card_eval_value(card, &run.tables))
        .collect();
    top_values.sort_by(|a, b| b.cmp(a));
    let raw_sum: i64 = top_values.into_iter().take(5).sum();
    best_total * 3 + raw_sum
}

fn legal_consumable_actions(run: &RunState, cap: usize) -> Vec<AutoAction> {
    let mut actions = Vec::new();
    let free_consumables = run
        .inventory
        .consumable_slots
        .saturating_sub(run.inventory.consumable_count());
    let free_jokers = run
        .inventory
        .joker_capacity()
        .saturating_sub(run.inventory.jokers.len());
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
        if !consumable_effects_fit_slots(&def.effects, free_consumables, free_jokers) {
            continue;
        }
        let required = required_selected_count(&def.effects);
        if required == 0 {
            actions.push(AutoAction::UseConsumable {
                index: idx,
                selected: Vec::new(),
            });
            continue;
        }
        if required > run.hand.len() {
            continue;
        }
        let mut combos = Vec::new();
        enumerate_combinations(run.hand.len(), required, &mut combos);
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

fn can_afford(money: i64, price: i64) -> bool {
    price <= 0 || money >= price
}

fn can_buy_shop_card(run: &RunState, offer: &rulatro_core::CardOffer) -> bool {
    match offer.kind {
        ShopCardKind::Joker => run.inventory.jokers.len() < run.inventory.joker_capacity(),
        ShopCardKind::Tarot | ShopCardKind::Planet => {
            run.inventory.consumable_count() < run.inventory.consumable_slots
        }
    }
}

fn can_pick_pack_indices(
    open: &PackOpen,
    indices: &[usize],
    joker_free_slots: usize,
    consumable_free_slots: usize,
) -> bool {
    let mut jokers_needed = 0usize;
    let mut consumables_needed = 0usize;
    for idx in indices {
        let Some(option) = open.options.get(*idx) else {
            return false;
        };
        match option {
            PackOption::Joker(_) => jokers_needed = jokers_needed.saturating_add(1),
            PackOption::Consumable(_, _) => {
                consumables_needed = consumables_needed.saturating_add(1);
            }
            PackOption::PlayingCard(_) => {}
        }
    }
    jokers_needed <= joker_free_slots && consumables_needed <= consumable_free_slots
}

fn consumable_effects_fit_slots(
    blocks: &[rulatro_core::EffectBlock],
    free_consumables: usize,
    free_jokers: usize,
) -> bool {
    let need_consumables = required_free_consumable_slots(blocks);
    let need_jokers = required_free_joker_slots(blocks);
    need_consumables <= free_consumables && need_jokers <= free_jokers
}

fn required_free_consumable_slots(blocks: &[rulatro_core::EffectBlock]) -> usize {
    let mut required = 0usize;
    for block in blocks {
        for effect in &block.effects {
            match effect {
                EffectOp::AddRandomConsumable { count, .. } => {
                    required = required.max(*count as usize);
                }
                EffectOp::CreateLastConsumable { .. } => {
                    required = required.max(1);
                }
                _ => {}
            }
        }
    }
    required
}

fn required_free_joker_slots(blocks: &[rulatro_core::EffectBlock]) -> usize {
    let mut required = 0usize;
    for block in blocks {
        for effect in &block.effects {
            match effect {
                EffectOp::AddJoker { count, .. } | EffectOp::AddRandomJoker { count } => {
                    required = required.max(*count as usize);
                }
                _ => {}
            }
        }
    }
    required
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

fn cards_from_hand(hand: &[Card], indices: &[usize]) -> Vec<Card> {
    indices
        .iter()
        .filter_map(|idx| hand.get(*idx).copied())
        .collect()
}

fn format_card(card: &Card) -> String {
    if card.face_down {
        return "??".to_string();
    }
    let mut out = format!("{}{}", rank_short(card.rank), suit_short(card.suit));
    let mut tags = Vec::new();
    if let Some(enhancement) = card.enhancement {
        tags.push(format!("{enhancement:?}"));
    }
    if let Some(edition) = card.edition {
        tags.push(format!("{edition:?}"));
    }
    if let Some(seal) = card.seal {
        tags.push(format!("{seal:?}"));
    }
    if card.bonus_chips != 0 {
        tags.push(format!("bonus={}", card.bonus_chips));
    }
    if !tags.is_empty() {
        out.push_str(" [");
        out.push_str(&tags.join(","));
        out.push(']');
    }
    out
}

fn rank_short(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::King => "K",
        Rank::Queen => "Q",
        Rank::Jack => "J",
        Rank::Ten => "T",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::Six => "6",
        Rank::Five => "5",
        Rank::Four => "4",
        Rank::Three => "3",
        Rank::Two => "2",
        Rank::Joker => "Jk",
    }
}

fn suit_short(suit: rulatro_core::Suit) -> &'static str {
    match suit {
        rulatro_core::Suit::Spades => "S",
        rulatro_core::Suit::Hearts => "H",
        rulatro_core::Suit::Clubs => "C",
        rulatro_core::Suit::Diamonds => "D",
        rulatro_core::Suit::Wild => "W",
    }
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Setup => "Setup/准备",
        Phase::Deal => "Deal/发牌",
        Phase::Play => "Play/出牌",
        Phase::Score => "Score/计分",
        Phase::Cleanup => "Cleanup/结算",
        Phase::Shop => "Shop/商店",
    }
}

fn blind_label(blind: BlindKind) -> &'static str {
    match blind {
        BlindKind::Small => "Small/小盲",
        BlindKind::Big => "Big/大盲",
        BlindKind::Boss => "Boss/Boss盲注",
    }
}

fn describe_shop_card(run: &RunState, offer: &rulatro_core::CardOffer) -> String {
    match offer.kind {
        ShopCardKind::Joker => {
            let name = find_joker_name(run, offer.item_id.as_str());
            let rarity = offer
                .rarity
                .map(|item| format!("{item:?}"))
                .unwrap_or_default();
            let edition = offer
                .edition
                .map(|item| format!("{item:?}"))
                .unwrap_or_else(|| "None".to_string());
            format!(
                "buy card/购买卡牌: Joker/小丑 {name} id={} rarity/稀有度={} edition/版本={} price/价格={}",
                offer.item_id, rarity, edition, offer.price
            )
        }
        ShopCardKind::Tarot | ShopCardKind::Planet => {
            let kind = if offer.kind == ShopCardKind::Tarot {
                ConsumableKind::Tarot
            } else {
                ConsumableKind::Planet
            };
            let name = find_consumable_name(run, kind, offer.item_id.as_str());
            let extra = find_consumable_extra(run, kind, offer.item_id.as_str());
            format!(
                "buy card/购买卡牌: {:?} {} id={} price/价格={} {}",
                offer.kind, name, offer.item_id, offer.price, extra
            )
        }
    }
}

fn find_joker_name(run: &RunState, id: &str) -> String {
    run.content
        .jokers
        .iter()
        .find(|item| item.id == id)
        .map(|item| item.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn find_consumable_name(run: &RunState, kind: ConsumableKind, id: &str) -> String {
    let list = match kind {
        ConsumableKind::Tarot => &run.content.tarots,
        ConsumableKind::Planet => &run.content.planets,
        ConsumableKind::Spectral => &run.content.spectrals,
    };
    list.iter()
        .find(|item| item.id == id)
        .map(|item| item.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn find_consumable_extra(run: &RunState, kind: ConsumableKind, id: &str) -> String {
    let list = match kind {
        ConsumableKind::Tarot => &run.content.tarots,
        ConsumableKind::Planet => &run.content.planets,
        ConsumableKind::Spectral => &run.content.spectrals,
    };
    let Some(item) = list.iter().find(|entry| entry.id == id) else {
        return String::new();
    };
    if let Some(hand) = item.hand {
        format!("hand/牌型={hand:?} effects/效果数={}", item.effects.len())
    } else {
        format!("effects/效果数={}", item.effects.len())
    }
}

fn describe_voucher(id: &str) -> String {
    if let Some(voucher) = voucher_by_id(id) {
        format!(
            "buy voucher/购买优惠券: {} id={} effect/效果={}",
            voucher.name(false),
            id,
            voucher.effect_text(false)
        )
    } else {
        format!("buy voucher/购买优惠券: id={id}")
    }
}

fn format_pack_option(run: &RunState, option: &PackOption) -> String {
    match option {
        PackOption::Joker(id) => format!("Joker/小丑 {}", find_joker_name(run, id.as_str())),
        PackOption::Consumable(kind, id) => {
            let name = find_consumable_name(run, *kind, id.as_str());
            format!("{kind:?}/消耗牌 {name}")
        }
        PackOption::PlayingCard(card) => format!("Card/扑克牌 {}", format_card(card)),
    }
}

fn format_ante_targets(run: &RunState, ante: u8) -> Option<String> {
    let small = run.config.target_for(ante, BlindKind::Small)?;
    let big = run.config.target_for(ante, BlindKind::Big)?;
    let boss = run.config.target_for(ante, BlindKind::Boss)?;
    Some(format!(
        "small/小盲={small} big/大盲={big} boss/Boss={boss}"
    ))
}

fn format_rule_effect(effect: &RuleEffect) -> String {
    match effect {
        RuleEffect::AddChips(value) => format!("AddChips({value})"),
        RuleEffect::AddMult(value) => format!("AddMult({value:.2})"),
        RuleEffect::MultiplyMult(value) => format!("MultiplyMult({value:.2})"),
        RuleEffect::MultiplyChips(value) => format!("MultiplyChips({value:.2})"),
    }
}
