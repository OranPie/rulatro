use super::*;
use crate::*;
use super::helpers::*;

impl RunState {
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
        let bonus = joker
            .vars
            .get("sell_bonus")
            .copied()
            .unwrap_or(0.0)
            .floor() as i64;
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
        let shop = ShopState::generate(&self.config.shop, &self.content, &mut self.rng);
        let offers = shop.cards.len() + shop.packs.len() + shop.vouchers;
        self.shop = Some(shop);
        self.state.phase = Phase::Shop;
        self.state.shop_free_rerolls = 0;

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
            ActivationType::OnShopEnter,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        self.state.money = money;

        events.push(Event::ShopEntered {
            offers,
            reroll_cost: self.shop.as_ref().map(|s| s.reroll_cost).unwrap_or(0),
        });
        Ok(())
    }

    pub fn reroll_shop(&mut self, events: &mut EventBus) -> Result<(), RunError> {
        if self.state.phase != Phase::Shop {
            return Err(RunError::InvalidPhase(self.state.phase));
        }
        let money_floor = self.money_floor();
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
            shop.reroll_cards(&self.config.shop, &self.content, &mut self.rng);
            let offers = shop.cards.len() + shop.packs.len() + shop.vouchers;
            let reroll_cost = shop.reroll_cost;
            (offers, reroll_cost, cost)
        };
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
            ActivationType::OnShopReroll,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
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
                    self.add_joker_with_trigger(card.item_id.clone(), rarity, card.price)?;
                }
                crate::ShopCardKind::Tarot => {
                    self.inventory.add_consumable(
                        card.item_id.clone(),
                        crate::ConsumableKind::Tarot,
                    )?;
                }
                crate::ShopCardKind::Planet => {
                    self.inventory.add_consumable(
                        card.item_id.clone(),
                        crate::ConsumableKind::Planet,
                    )?;
                }
            },
            ShopPurchase::Voucher => {
                // TODO: apply voucher-specific effects.
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
        let sell_value = self.calc_joker_sell_value(&joker);
        self.state.money += sell_value;
        self.current_joker_counts = build_joker_counts(&self.inventory.jokers);

        let hand_kind = self.state.last_hand.unwrap_or(crate::HandKind::HighCard);
        let ctx = EvalContext::sell(
            hand_kind,
            self.state.blind,
            sell_value,
            self.state.hands_left,
            self.state.discards_left,
            self.inventory.jokers.len(),
        );
        let mut dummy_score = Score::default();
        let mut results = TriggerResults::default();
        let mut money = self.state.money;
        self.apply_joker_effects_for_joker(
            &mut joker,
            ActivationType::OnSell,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        self.apply_joker_effects(
            ActivationType::OnAnySell,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
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
        let open = open_pack(pack, &self.content, &self.config.shop.joker_rarity_weights, &mut self.rng);
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
            ActivationType::OnPackOpened,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
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
        let chosen = crate::pick_pack_options(open, indices).map_err(|_| RunError::InvalidSelection)?;
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
                    if let Some(def) = self
                        .content
                        .tarots
                        .iter()
                        .chain(self.content.planets.iter())
                        .chain(self.content.spectrals.iter())
                        .find(|card| card.id == *id)
                    {
                        let def = def.clone();
                        if matches!(open.offer.kind, crate::PackKind::Arcana | crate::PackKind::Celestial | crate::PackKind::Spectral) {
                            self.apply_consumable_effects(&def)?;
                        } else {
                            self.inventory.add_consumable(id.clone(), *kind)?;
                        }
                    }
                }
                crate::PackOption::PlayingCard(card) => {
                    self.deck.discard(vec![*card]);
                }
            }
        }
        events.push(Event::PackChosen { picks: chosen.len() });
        Ok(())
    }

    pub fn skip_pack(&mut self, _open: &PackOpen, events: &mut EventBus) -> Result<(), RunError> {
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
            ActivationType::OnPackSkipped,
            &ctx,
            &mut dummy_score,
            &mut money,
            &mut results,
        );
        self.state.money = money;
        events.push(Event::PackChosen { picks: 0 });
        Ok(())
    }

    pub fn leave_shop(&mut self) {
        self.shop = None;
        self.state.phase = Phase::Deal;
        self.state.shop_free_rerolls = 0;
    }

}
