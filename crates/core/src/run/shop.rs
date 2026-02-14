use super::helpers::*;
use super::*;
use crate::*;

impl RunState {
    pub(super) fn shop_restrictions(&mut self) -> crate::ShopRestrictions {
        let mut restrictions = crate::ShopRestrictions::default();
        restrictions.allow_duplicates = self.rule_flag("shop_allow_duplicates");
        restrictions.owned_jokers = self
            .inventory
            .jokers
            .iter()
            .map(|joker| joker.id.clone())
            .collect();
        for consumable in &self.inventory.consumables {
            match consumable.kind {
                crate::ConsumableKind::Tarot => {
                    restrictions.owned_tarots.insert(consumable.id.clone());
                }
                crate::ConsumableKind::Planet => {
                    restrictions.owned_planets.insert(consumable.id.clone());
                }
                crate::ConsumableKind::Spectral => {
                    restrictions.owned_spectrals.insert(consumable.id.clone());
                }
            }
        }
        restrictions.owned_vouchers = self.state.active_vouchers.iter().cloned().collect();
        restrictions
    }

    pub(super) fn default_joker_price(&mut self, rarity: crate::JokerRarity) -> i64 {
        match rarity {
            crate::JokerRarity::Common => {
                let range = &self.config.shop.prices.joker_common;
                self.random_range_values(range.min, range.max)
            }
            crate::JokerRarity::Uncommon => {
                let range = &self.config.shop.prices.joker_uncommon;
                self.random_range_values(range.min, range.max)
            }
            crate::JokerRarity::Rare => {
                let range = &self.config.shop.prices.joker_rare;
                self.random_range_values(range.min, range.max)
            }
            crate::JokerRarity::Legendary => self.config.shop.prices.joker_legendary,
        }
    }

    pub(super) fn calc_joker_sell_value(&self, joker: &crate::JokerInstance) -> i64 {
        let base = joker.buy_price.max(0);
        let bonus = joker.vars.get("sell_bonus").copied().unwrap_or(0.0).floor() as i64;
        let value = base / 2 + bonus;
        value.max(1)
    }

    pub fn joker_sell_value(&self, index: usize) -> Option<i64> {
        self.inventory
            .jokers
            .get(index)
            .map(|joker| self.calc_joker_sell_value(joker))
    }

    pub fn enter_shop(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if !self.blind_cleared() {
            return Err(RunError::BlindNotCleared);
        }
        if let Some(shop) = self.shop.as_ref() {
            let offers = shop.cards.len() + shop.packs.len() + shop.vouchers;
            self.state.phase = Phase::Shop;
            events.push(Event::ShopEntered {
                offers,
                reroll_cost: shop.reroll_cost,
                reentered: true,
            });
            return Ok(());
        }
        let restrictions = self.shop_restrictions();
        let rule = self.effective_shop_rule();
        let shop = ShopState::generate(&rule, &self.content, &mut self.rng, &restrictions);
        let offers = shop.cards.len() + shop.packs.len() + shop.vouchers;
        let reroll_cost = shop.reroll_cost;
        self.shop = Some(shop);
        self.state.phase = Phase::Shop;
        self.state.shop_free_rerolls = 0;

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
        self.invoke_hooks(HookPoint::ShopEnter, &mut args, events);
        self.state.money = money;

        events.push(Event::ShopEntered {
            offers,
            reroll_cost,
            reentered: false,
        });
        Ok(())
    }

    pub fn reroll_shop(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Shop {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        let money_floor = self.money_floor();
        let restrictions = self.shop_restrictions();
        let rule = self.effective_shop_rule();
        let (offers, reroll_cost, cost) = {
            let shop = self.shop.as_mut().ok_or(RunError::ShopNotAvailable)?;
            let mut cost = shop.reroll_cost;
            if self.state.shop_free_rerolls > 0 {
                self.state.shop_free_rerolls -= 1;
                cost = 0;
            }
            if cost > 0 {
                if self.state.money - cost < money_floor {
                    return Err(RunError::NotEnoughMoney);
                }
                self.state.money -= cost;
            }
            shop.reroll_cards(&rule, &self.content, &mut self.rng, &restrictions);
            let offers = shop.cards.len() + shop.packs.len() + shop.vouchers;
            let reroll_cost = shop.reroll_cost;
            (offers, reroll_cost, cost)
        };
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
        self.invoke_hooks(HookPoint::ShopReroll, &mut args, events);
        self.state.money = money;
        events.push(Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money: self.state.money,
        });
        Ok(())
    }

    pub fn buy_shop_offer(
        &mut self,
        offer: ShopOfferRef,
        events: &mut EventBus,
    ) -> Result<ShopPurchase, RunError> {
        if self.state.phase != Phase::Shop {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        let money_floor = self.money_floor();
        let shop = self.shop.as_mut().ok_or(RunError::ShopNotAvailable)?;
        let price = shop
            .price_for_offer(offer, &self.config.shop.prices)
            .ok_or(RunError::InvalidOfferIndex)?;
        if self.state.money - price < money_floor {
            return Err(RunError::NotEnoughMoney);
        }
        let purchase = shop.take_offer(offer).ok_or(RunError::InvalidOfferIndex)?;
        self.state.money -= price;
        events.push(Event::ShopBought {
            offer: purchase.kind(),
            cost: price,
            money: self.state.money,
        });
        Ok(purchase)
    }

    pub fn apply_purchase(&mut self, purchase: &ShopPurchase) -> Result<(), RunError> {
        match purchase {
            ShopPurchase::Card(card) => match card.kind {
                crate::ShopCardKind::Joker => {
                    let rarity = card.rarity.unwrap_or(crate::JokerRarity::Common);
                    self.add_joker_with_trigger_edition(
                        card.item_id.clone(),
                        rarity,
                        card.price,
                        card.edition,
                    )?;
                }
                crate::ShopCardKind::Tarot => {
                    self.inventory
                        .add_consumable(card.item_id.clone(), crate::ConsumableKind::Tarot)?;
                }
                crate::ShopCardKind::Planet => {
                    self.inventory
                        .add_consumable(card.item_id.clone(), crate::ConsumableKind::Planet)?;
                }
            },
            ShopPurchase::Voucher(voucher) => {
                if self
                    .state
                    .active_vouchers
                    .iter()
                    .any(|owned| owned == &voucher.id)
                {
                    return Ok(());
                }
                let old_rule = self.effective_shop_rule();
                self.state.active_vouchers.push(voucher.id.clone());
                self.apply_voucher_state_effect(&voucher.id);
                let new_rule = self.effective_shop_rule();
                self.reprice_open_shop_after_voucher(&old_rule, &new_rule);
            }
            ShopPurchase::Pack(_) => {}
        }
        Ok(())
    }

    pub fn sell_joker(&mut self, index: usize, events: &mut EventBus) -> Result<(), RunError> {
        if index >= self.inventory.jokers.len() {
            return Err(RunError::InvalidJokerIndex);
        }
        let mut joker = self.inventory.jokers.remove(index);
        self.mark_rules_dirty();
        let sell_value = self.calc_joker_sell_value(&joker);
        self.state.money += sell_value;
        self.current_joker_counts = build_joker_counts(&self.inventory.jokers);

        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let mut scratch_score = Score::default();
        let mut money = self.state.money;
        let mut results = TriggerResults::default();
        let mut held_view = self.hand.clone();
        let mut sell_args = HookArgs::sell(
            hand_kind,
            self.state.blind,
            sell_value,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
            Some(&mut joker),
        );
        self.invoke_hooks(HookPoint::Sell, &mut sell_args, events);
        let mut held_view = self.hand.clone();
        let mut any_sell_args = HookArgs::sell(
            hand_kind,
            self.state.blind,
            sell_value,
            HookInject::held(&mut held_view),
            &mut scratch_score,
            &mut money,
            &mut results,
            None,
        );
        self.invoke_hooks(HookPoint::AnySell, &mut any_sell_args, events);
        self.state.money = money;
        events.push(Event::JokerSold {
            id: joker.id,
            sell_value,
            money: self.state.money,
        });
        Ok(())
    }

    pub fn open_pack_purchase(
        &mut self,
        purchase: &ShopPurchase,
        events: &mut EventBus,
    ) -> Result<PackOpen, RunError> {
        let pack = match purchase {
            ShopPurchase::Pack(pack) => pack,
            _ => return Err(RunError::PackNotAvailable),
        };
        let restrictions = self.shop_restrictions();
        let open = open_pack(
            pack,
            &self.content,
            &self.config.shop.joker_rarity_weights,
            &mut self.rng,
            &restrictions,
        );
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
        self.invoke_hooks(HookPoint::PackOpened, &mut args, events);
        self.state.money = money;
        events.push(Event::PackOpened {
            kind: purchase.kind(),
            options: open.options.len(),
            picks: open.offer.picks,
        });
        Ok(open)
    }

    pub fn choose_pack_options(
        &mut self,
        open: &PackOpen,
        indices: &[usize],
        events: &mut EventBus,
    ) -> Result<(), RunError> {
        let chosen =
            crate::pick_pack_options(open, indices).map_err(|_| RunError::InvalidSelection)?;
        for option in &chosen {
            match option {
                crate::PackOption::Joker(id) => {
                    let rarity = self
                        .content
                        .jokers
                        .iter()
                        .find(|j| &j.id == id)
                        .map(|j| j.rarity);
                    if let Some(rarity) = rarity {
                        let price = self.default_joker_price(rarity);
                        self.add_joker_with_trigger(id.clone(), rarity, price)?;
                    }
                }
                crate::PackOption::Consumable(kind, id) => {
                    if let Some(_def) = self
                        .content
                        .tarots
                        .iter()
                        .chain(self.content.planets.iter())
                        .chain(self.content.spectrals.iter())
                        .find(|card| card.id == *id)
                    {
                        self.inventory.add_consumable(id.clone(), *kind)?;
                    }
                }
                crate::PackOption::PlayingCard(card) => {
                    let mut card = *card;
                    self.assign_card_id(&mut card);
                    self.deck.discard(vec![card]);
                    self.trigger_on_card_added(card);
                }
            }
        }
        events.push(Event::PackChosen {
            picks: chosen.len(),
        });
        Ok(())
    }

    pub fn skip_pack(&mut self, _open: &PackOpen, events: &mut EventBus) -> Result<(), RunError> {
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
        self.invoke_hooks(HookPoint::PackSkipped, &mut args, events);
        self.state.money = money;
        events.push(Event::PackChosen { picks: 0 });
        Ok(())
    }

    pub fn leave_shop(&mut self) {
        self.state.phase = Phase::Deal;
        self.state.shop_free_rerolls = 0;
    }

    pub(super) fn voucher_offer_for_shop(&mut self) -> crate::VoucherOffer {
        let mut pool: Vec<String> = crate::all_vouchers()
            .iter()
            .map(|entry| entry.id.to_string())
            .collect();
        if let Some(shop) = self.shop.as_ref() {
            let in_shop: std::collections::HashSet<String> = shop
                .voucher_offers
                .iter()
                .map(|offer| offer.id.clone())
                .collect();
            pool.retain(|id| !in_shop.contains(id));
        }
        pool.retain(|id| !self.state.active_vouchers.iter().any(|owned| owned == id));
        if pool.is_empty() {
            return crate::VoucherOffer {
                id: "blank".to_string(),
            };
        }
        let idx = (self.rng.next_u64() % pool.len() as u64) as usize;
        crate::VoucherOffer {
            id: pool[idx].clone(),
        }
    }

    pub fn active_vouchers(&self) -> &[String] {
        &self.state.active_vouchers
    }

    fn effective_shop_rule(&self) -> crate::ShopRule {
        let mut rule = self.config.shop.clone();
        let mut add_slots = 0u8;
        let mut add_tarot_weight = 0u32;
        let mut add_planet_weight = 0u32;
        let mut reroll_reduce = 0i64;
        let mut discount = 0u8;
        for id in &self.state.active_vouchers {
            if let Some(voucher) = crate::voucher_by_id(id) {
                match voucher.effect {
                    crate::VoucherEffect::AddShopCardSlots(value) => {
                        add_slots = add_slots.saturating_add(value);
                    }
                    crate::VoucherEffect::AddTarotWeight(value) => {
                        add_tarot_weight = add_tarot_weight.saturating_add(value);
                    }
                    crate::VoucherEffect::AddPlanetWeight(value) => {
                        add_planet_weight = add_planet_weight.saturating_add(value);
                    }
                    crate::VoucherEffect::ReduceRerollBase(value) => {
                        reroll_reduce = reroll_reduce.saturating_add(value.max(0));
                    }
                    crate::VoucherEffect::SetShopDiscountPercent(value) => {
                        discount = discount.max(value.min(95));
                    }
                    _ => {}
                }
            }
        }
        rule.card_slots = rule.card_slots.saturating_add(add_slots);
        for entry in &mut rule.card_weights {
            match entry.kind {
                crate::ShopCardKind::Tarot => {
                    entry.weight = entry.weight.saturating_add(add_tarot_weight);
                }
                crate::ShopCardKind::Planet => {
                    entry.weight = entry.weight.saturating_add(add_planet_weight);
                }
                _ => {}
            }
        }
        rule.prices.reroll_base = rule.prices.reroll_base.saturating_sub(reroll_reduce).max(0);
        if discount > 0 {
            apply_discount_to_shop_prices(&mut rule.prices, discount);
        }
        rule
    }

    pub(super) fn voucher_hands_bonus(&self) -> u8 {
        self.state
            .active_vouchers
            .iter()
            .filter_map(|id| crate::voucher_by_id(id))
            .filter_map(|voucher| match voucher.effect {
                crate::VoucherEffect::AddHandsPerRound(value) => Some(value),
                _ => None,
            })
            .fold(0u8, |acc, value| acc.saturating_add(value))
    }

    pub(super) fn voucher_discards_bonus(&self) -> u8 {
        self.state
            .active_vouchers
            .iter()
            .filter_map(|id| crate::voucher_by_id(id))
            .filter_map(|voucher| match voucher.effect {
                crate::VoucherEffect::AddDiscardsPerRound(value) => Some(value),
                _ => None,
            })
            .fold(0u8, |acc, value| acc.saturating_add(value))
    }

    fn apply_voucher_state_effect(&mut self, voucher_id: &str) {
        let Some(voucher) = crate::voucher_by_id(voucher_id) else {
            return;
        };
        match voucher.effect {
            crate::VoucherEffect::AddConsumableSlots(value) => {
                self.inventory.consumable_slots = self
                    .inventory
                    .consumable_slots
                    .saturating_add(value as usize);
            }
            crate::VoucherEffect::AddJokerSlots(value) => {
                self.inventory.joker_slots =
                    self.inventory.joker_slots.saturating_add(value as usize);
            }
            crate::VoucherEffect::AddHandSizeBase(value) => {
                let value = value as usize;
                self.state.hand_size_base = self.state.hand_size_base.saturating_add(value);
                self.state.hand_size = self.state.hand_size.saturating_add(value);
            }
            _ => {}
        }
    }

    fn reprice_open_shop_after_voucher(
        &mut self,
        old_rule: &crate::ShopRule,
        new_rule: &crate::ShopRule,
    ) {
        let Some(shop) = self.shop.as_mut() else {
            return;
        };
        let old_discount = discount_percent_from_prices(&self.config.shop.prices, &old_rule.prices);
        let new_discount = discount_percent_from_prices(&self.config.shop.prices, &new_rule.prices);
        if new_discount > old_discount {
            let old_keep = (100 - old_discount as i64).max(1);
            let new_keep = (100 - new_discount as i64).max(0);
            for card in &mut shop.cards {
                card.price = ((card.price * new_keep) / old_keep).max(0);
            }
            for pack in &mut shop.packs {
                pack.price = ((pack.price * new_keep) / old_keep).max(0);
            }
        }
        let old_base = old_rule.prices.reroll_base.max(0);
        let new_base = new_rule.prices.reroll_base.max(0);
        let step = shop.reroll_cost.saturating_sub(old_base);
        shop.reroll_cost = new_base.saturating_add(step);
    }
}

fn apply_discount_to_shop_prices(prices: &mut crate::ShopPrices, discount_percent: u8) {
    let keep = (100 - discount_percent.min(95) as i64).max(1);
    prices.joker_common.min = (prices.joker_common.min * keep) / 100;
    prices.joker_common.max = (prices.joker_common.max * keep) / 100;
    prices.joker_uncommon.min = (prices.joker_uncommon.min * keep) / 100;
    prices.joker_uncommon.max = (prices.joker_uncommon.max * keep) / 100;
    prices.joker_rare.min = (prices.joker_rare.min * keep) / 100;
    prices.joker_rare.max = (prices.joker_rare.max * keep) / 100;
    prices.joker_legendary = (prices.joker_legendary * keep) / 100;
    prices.tarot = (prices.tarot * keep) / 100;
    prices.planet = (prices.planet * keep) / 100;
    prices.spectral = (prices.spectral * keep) / 100;
    prices.playing_card = (prices.playing_card * keep) / 100;
    for pack in &mut prices.pack_prices {
        pack.price = (pack.price * keep) / 100;
    }
}

fn discount_percent_from_prices(base: &crate::ShopPrices, current: &crate::ShopPrices) -> u8 {
    if base.joker_common.min <= 0 {
        return 0;
    }
    let ratio = (current.joker_common.min as f64 / base.joker_common.min as f64) * 100.0;
    let keep = ratio.round().clamp(0.0, 100.0) as i64;
    (100 - keep as u8).min(95)
}
