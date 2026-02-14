use crate::{
    all_vouchers, ConsumableKind, Content, Edition, JokerRarity, PackKind, PackPrice, PackSize,
    PackWeight, PriceRange, RngState, ShopCardKind, ShopPrices, ShopRule,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CardOffer {
    pub kind: ShopCardKind,
    pub item_id: String,
    pub rarity: Option<JokerRarity>,
    pub price: i64,
    pub edition: Option<Edition>,
}

#[derive(Debug, Clone)]
pub struct PackOffer {
    pub kind: PackKind,
    pub size: PackSize,
    pub options: u8,
    pub picks: u8,
    pub price: i64,
}

#[derive(Debug, Clone)]
pub struct VoucherOffer {
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopOfferKind {
    Card(ShopCardKind),
    Pack(PackKind, PackSize),
    Voucher,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopOfferRef {
    Card(usize),
    Pack(usize),
    Voucher(usize),
}

#[derive(Debug, Clone)]
pub struct ShopState {
    pub cards: Vec<CardOffer>,
    pub packs: Vec<PackOffer>,
    pub vouchers: usize,
    pub voucher_offers: Vec<VoucherOffer>,
    pub reroll_cost: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ShopRestrictions {
    pub allow_duplicates: bool,
    pub owned_jokers: HashSet<String>,
    pub owned_tarots: HashSet<String>,
    pub owned_planets: HashSet<String>,
    pub owned_spectrals: HashSet<String>,
    pub owned_vouchers: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct PackOpen {
    pub offer: PackOffer,
    pub options: Vec<PackOption>,
}

#[derive(Debug, Clone)]
pub enum PackOption {
    Joker(String),
    Consumable(ConsumableKind, String),
    PlayingCard(crate::Card),
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("invalid selection")]
    InvalidSelection,
    #[error("too many picks")]
    TooManyPicks,
}

impl ShopState {
    pub fn generate(
        rule: &ShopRule,
        content: &Content,
        rng: &mut RngState,
        restrictions: &ShopRestrictions,
    ) -> Self {
        let cards = generate_cards(rule, content, rng, restrictions);
        let packs = generate_packs(rule, rng);
        let voucher_offers = generate_vouchers(rule.voucher_slots as usize, rng, restrictions);
        Self {
            cards,
            packs,
            vouchers: voucher_offers.len(),
            voucher_offers,
            reroll_cost: rule.prices.reroll_base,
        }
    }

    pub fn reroll_cards(
        &mut self,
        rule: &ShopRule,
        content: &Content,
        rng: &mut RngState,
        restrictions: &ShopRestrictions,
    ) {
        self.cards = generate_cards(rule, content, rng, restrictions);
        self.reroll_cost += rule.prices.reroll_step;
    }

    pub fn offer_kind(&self, offer: ShopOfferRef) -> Option<ShopOfferKind> {
        match offer {
            ShopOfferRef::Card(index) => self
                .cards
                .get(index)
                .map(|card| ShopOfferKind::Card(card.kind)),
            ShopOfferRef::Pack(index) => self
                .packs
                .get(index)
                .map(|pack| ShopOfferKind::Pack(pack.kind, pack.size)),
            ShopOfferRef::Voucher(index) => {
                if index < self.vouchers {
                    Some(ShopOfferKind::Voucher)
                } else {
                    None
                }
            }
        }
    }

    pub fn price_for_offer(&self, offer: ShopOfferRef, prices: &ShopPrices) -> Option<i64> {
        match offer {
            ShopOfferRef::Card(index) => self.cards.get(index).map(|card| card.price),
            ShopOfferRef::Pack(index) => self.packs.get(index).map(|pack| pack.price),
            ShopOfferRef::Voucher(index) => {
                if index < self.vouchers {
                    Some(prices.voucher)
                } else {
                    None
                }
            }
        }
    }

    pub fn take_offer(&mut self, offer: ShopOfferRef) -> Option<ShopPurchase> {
        match offer {
            ShopOfferRef::Card(index) => {
                if index < self.cards.len() {
                    Some(ShopPurchase::Card(self.cards.remove(index)))
                } else {
                    None
                }
            }
            ShopOfferRef::Pack(index) => {
                if index < self.packs.len() {
                    Some(ShopPurchase::Pack(self.packs.remove(index)))
                } else {
                    None
                }
            }
            ShopOfferRef::Voucher(index) => {
                if index < self.vouchers {
                    let taken = if index < self.voucher_offers.len() {
                        self.voucher_offers.remove(index)
                    } else {
                        VoucherOffer {
                            id: "blank".to_string(),
                        }
                    };
                    self.vouchers = self.voucher_offers.len();
                    Some(ShopPurchase::Voucher(taken))
                } else {
                    None
                }
            }
        }
    }

    pub fn add_voucher_offer(&mut self, offer: VoucherOffer) {
        self.voucher_offers.push(offer);
        self.vouchers = self.voucher_offers.len();
    }

    pub fn remove_voucher_slots(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let keep = self.voucher_offers.len().saturating_sub(count);
        self.voucher_offers.truncate(keep);
        self.vouchers = self.voucher_offers.len();
    }
}

#[derive(Debug, Clone)]
pub enum ShopPurchase {
    Card(CardOffer),
    Pack(PackOffer),
    Voucher(VoucherOffer),
}

impl ShopPurchase {
    pub fn kind(&self) -> ShopOfferKind {
        match self {
            ShopPurchase::Card(card) => ShopOfferKind::Card(card.kind),
            ShopPurchase::Pack(pack) => ShopOfferKind::Pack(pack.kind, pack.size),
            ShopPurchase::Voucher(_) => ShopOfferKind::Voucher,
        }
    }
}

pub fn open_pack(
    offer: &PackOffer,
    content: &Content,
    rarity_weights: &[crate::JokerRarityWeight],
    rng: &mut RngState,
    restrictions: &ShopRestrictions,
) -> PackOpen {
    let mut options = Vec::with_capacity(offer.options as usize);
    for _ in 0..offer.options {
        match offer.kind {
            PackKind::Arcana => {
                if let Some(card) =
                    pick_consumable_restricted(content, ConsumableKind::Tarot, rng, restrictions)
                {
                    options.push(PackOption::Consumable(
                        ConsumableKind::Tarot,
                        card.id.clone(),
                    ));
                }
            }
            PackKind::Buffoon => {
                if let Some(rarity) = pick_weighted_rarity(rarity_weights, rng) {
                    if let Some(joker) = pick_joker_restricted(content, rarity, rng, restrictions) {
                        options.push(PackOption::Joker(joker.id.clone()));
                    }
                }
            }
            PackKind::Celestial => {
                if let Some(card) =
                    pick_consumable_restricted(content, ConsumableKind::Planet, rng, restrictions)
                {
                    options.push(PackOption::Consumable(
                        ConsumableKind::Planet,
                        card.id.clone(),
                    ));
                }
            }
            PackKind::Spectral => {
                if let Some(card) =
                    pick_consumable_restricted(content, ConsumableKind::Spectral, rng, restrictions)
                {
                    options.push(PackOption::Consumable(
                        ConsumableKind::Spectral,
                        card.id.clone(),
                    ));
                }
            }
            PackKind::Standard => {
                let card = content.random_standard_card(rng);
                options.push(PackOption::PlayingCard(card));
            }
        }
    }

    PackOpen {
        offer: offer.clone(),
        options,
    }
}

pub fn pick_pack_options(open: &PackOpen, indices: &[usize]) -> Result<Vec<PackOption>, PackError> {
    if indices.is_empty() {
        return Err(PackError::InvalidSelection);
    }
    if indices.len() > open.offer.picks as usize {
        return Err(PackError::TooManyPicks);
    }
    let mut unique = indices.to_vec();
    unique.sort_unstable();
    unique.dedup();
    if unique.iter().any(|&idx| idx >= open.options.len()) {
        return Err(PackError::InvalidSelection);
    }
    Ok(unique
        .into_iter()
        .map(|idx| open.options[idx].clone())
        .collect())
}

fn generate_cards(
    rule: &ShopRule,
    content: &Content,
    rng: &mut RngState,
    restrictions: &ShopRestrictions,
) -> Vec<CardOffer> {
    let mut cards = Vec::new();
    for _ in 0..rule.card_slots {
        if let Some(kind) = pick_weighted_card(&rule.card_weights, rng) {
            match kind {
                ShopCardKind::Joker => {
                    if let Some(rarity) = pick_weighted_rarity(&rule.joker_rarity_weights, rng) {
                        if let Some(joker) =
                            pick_joker_restricted(content, rarity, rng, restrictions)
                        {
                            let price = price_for_joker_rarity(rarity, &rule.prices, rng);
                            cards.push(CardOffer {
                                kind,
                                item_id: joker.id.clone(),
                                rarity: Some(rarity),
                                price,
                                edition: None,
                            });
                        }
                    }
                }
                ShopCardKind::Tarot => {
                    if let Some(tarot) = pick_consumable_restricted(
                        content,
                        ConsumableKind::Tarot,
                        rng,
                        restrictions,
                    ) {
                        cards.push(CardOffer {
                            kind,
                            item_id: tarot.id.clone(),
                            rarity: None,
                            price: rule.prices.tarot,
                            edition: None,
                        });
                    }
                }
                ShopCardKind::Planet => {
                    if let Some(planet) = pick_consumable_restricted(
                        content,
                        ConsumableKind::Planet,
                        rng,
                        restrictions,
                    ) {
                        cards.push(CardOffer {
                            kind,
                            item_id: planet.id.clone(),
                            rarity: None,
                            price: rule.prices.planet,
                            edition: None,
                        });
                    }
                }
            }
        }
    }
    cards
}

fn generate_vouchers(
    slots: usize,
    rng: &mut RngState,
    restrictions: &ShopRestrictions,
) -> Vec<VoucherOffer> {
    if slots == 0 {
        return Vec::new();
    }
    let all: Vec<String> = all_vouchers()
        .iter()
        .map(|entry| entry.id.to_string())
        .collect();
    let mut pool: Vec<String> = if restrictions.allow_duplicates {
        all.clone()
    } else {
        all.into_iter()
            .filter(|id| !restrictions.owned_vouchers.contains(id))
            .collect()
    };
    if pool.is_empty() {
        pool = all_vouchers()
            .iter()
            .map(|entry| entry.id.to_string())
            .collect();
    }
    let mut picked = Vec::new();
    for _ in 0..slots {
        if pool.is_empty() {
            break;
        }
        let idx = (rng.next_u64() % pool.len() as u64) as usize;
        let id = pool.remove(idx);
        picked.push(VoucherOffer { id });
        if restrictions.allow_duplicates {
            pool = all_vouchers()
                .iter()
                .map(|entry| entry.id.to_string())
                .collect();
        }
    }
    picked
}

fn generate_packs(rule: &ShopRule, rng: &mut RngState) -> Vec<PackOffer> {
    let mut packs = Vec::new();
    for _ in 0..rule.booster_slots {
        if let Some(pack) = pick_weighted_pack(&rule.pack_weights, &rule.prices.pack_prices, rng) {
            packs.push(pack);
        }
    }
    packs
}

fn pick_joker_restricted<'a>(
    content: &'a Content,
    rarity: JokerRarity,
    rng: &mut RngState,
    restrictions: &ShopRestrictions,
) -> Option<&'a crate::JokerDef> {
    if restrictions.allow_duplicates {
        return content.pick_joker(rarity, rng);
    }
    let indices: Vec<usize> = content
        .jokers
        .iter()
        .enumerate()
        .filter(|(_, joker)| {
            joker.rarity == rarity && !restrictions.owned_jokers.contains(&joker.id)
        })
        .map(|(idx, _)| idx)
        .collect();
    pick_index(&indices, rng).and_then(|idx| content.jokers.get(idx))
}

fn pick_consumable_restricted<'a>(
    content: &'a Content,
    kind: ConsumableKind,
    rng: &mut RngState,
    restrictions: &ShopRestrictions,
) -> Option<&'a crate::ConsumableDef> {
    if restrictions.allow_duplicates {
        return content.pick_consumable(kind, rng);
    }
    let owned = match kind {
        ConsumableKind::Tarot => &restrictions.owned_tarots,
        ConsumableKind::Planet => &restrictions.owned_planets,
        ConsumableKind::Spectral => &restrictions.owned_spectrals,
    };
    let pool = match kind {
        ConsumableKind::Tarot => &content.tarots,
        ConsumableKind::Planet => &content.planets,
        ConsumableKind::Spectral => &content.spectrals,
    };
    let indices: Vec<usize> = pool
        .iter()
        .enumerate()
        .filter(|(_, card)| !owned.contains(&card.id))
        .map(|(idx, _)| idx)
        .collect();
    pick_index(&indices, rng).and_then(|idx| pool.get(idx))
}

fn price_for_joker_rarity(rarity: JokerRarity, prices: &ShopPrices, rng: &mut RngState) -> i64 {
    match rarity {
        JokerRarity::Common => pick_range(prices.joker_common.clone(), rng),
        JokerRarity::Uncommon => pick_range(prices.joker_uncommon.clone(), rng),
        JokerRarity::Rare => pick_range(prices.joker_rare.clone(), rng),
        JokerRarity::Legendary => prices.joker_legendary,
    }
}

fn pick_weighted_card(weights: &[crate::CardWeight], rng: &mut RngState) -> Option<ShopCardKind> {
    pick_weighted(weights.iter().map(|w| (w.kind, w.weight)), rng)
}

fn pick_weighted_rarity(
    weights: &[crate::JokerRarityWeight],
    rng: &mut RngState,
) -> Option<JokerRarity> {
    pick_weighted(weights.iter().map(|w| (w.rarity, w.weight)), rng)
}

fn pick_weighted_pack(
    weights: &[PackWeight],
    prices: &[PackPrice],
    rng: &mut RngState,
) -> Option<PackOffer> {
    let picked = pick_weighted(weights.iter().map(|w| (w.clone(), w.weight)), rng)?;
    let price = prices
        .iter()
        .find(|entry| entry.size == picked.size)
        .map(|entry| entry.price)?;

    Some(PackOffer {
        kind: picked.kind,
        size: picked.size,
        options: picked.options,
        picks: picked.picks,
        price,
    })
}

fn pick_weighted<T: Clone>(items: impl Iterator<Item = (T, u32)>, rng: &mut RngState) -> Option<T> {
    let items: Vec<(T, u32)> = items.filter(|(_, w)| *w > 0).collect();
    let total: u32 = items.iter().map(|(_, w)| *w).sum();
    if total == 0 {
        return None;
    }
    let mut roll = (rng.next_u64() % total as u64) as u32;
    for (item, weight) in items {
        if roll < weight {
            return Some(item);
        }
        roll -= weight;
    }
    None
}

fn pick_index(indices: &[usize], rng: &mut RngState) -> Option<usize> {
    if indices.is_empty() {
        return None;
    }
    let idx = (rng.next_u64() % indices.len() as u64) as usize;
    indices.get(idx).copied()
}

fn pick_range(range: PriceRange, rng: &mut RngState) -> i64 {
    if range.min >= range.max {
        return range.min;
    }
    let span = (range.max - range.min + 1) as u64;
    let roll = rng.next_u64() % span;
    range.min + roll as i64
}
