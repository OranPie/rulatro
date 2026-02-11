use crate::{ConsumableKind, EffectBlock, HandKind, JokerEffect, JokerRarity, Rank, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JokerDef {
    pub id: String,
    pub name: String,
    pub rarity: JokerRarity,
    pub effects: Vec<JokerEffect>,
}

#[derive(Debug, Clone)]
pub struct BossDef {
    pub id: String,
    pub name: String,
    pub effects: Vec<JokerEffect>,
}

#[derive(Debug, Clone)]
pub struct TagDef {
    pub id: String,
    pub name: String,
    pub effects: Vec<JokerEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableDef {
    pub id: String,
    pub name: String,
    pub kind: ConsumableKind,
    #[serde(default)]
    pub hand: Option<HandKind>,
    pub effects: Vec<EffectBlock>,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub jokers: Vec<JokerDef>,
    pub bosses: Vec<BossDef>,
    pub tags: Vec<TagDef>,
    pub tarots: Vec<ConsumableDef>,
    pub planets: Vec<ConsumableDef>,
    pub spectrals: Vec<ConsumableDef>,
}

impl Content {
    pub fn pick_joker<'a>(
        &'a self,
        rarity: JokerRarity,
        rng: &mut crate::RngState,
    ) -> Option<&'a JokerDef> {
        let indices: Vec<usize> = self
            .jokers
            .iter()
            .enumerate()
            .filter(|(_, joker)| joker.rarity == rarity)
            .map(|(idx, _)| idx)
            .collect();
        pick_index(&indices, rng).map(|idx| &self.jokers[idx])
    }

    pub fn pick_consumable<'a>(
        &'a self,
        kind: ConsumableKind,
        rng: &mut crate::RngState,
    ) -> Option<&'a ConsumableDef> {
        let pool = match kind {
            ConsumableKind::Tarot => &self.tarots,
            ConsumableKind::Planet => &self.planets,
            ConsumableKind::Spectral => &self.spectrals,
        };
        let indices: Vec<usize> = (0..pool.len()).collect();
        pick_index(&indices, rng).map(|idx| &pool[idx])
    }

    pub fn planet_for_hand<'a>(
        &'a self,
        hand: HandKind,
        rng: &mut crate::RngState,
    ) -> Option<&'a ConsumableDef> {
        let indices: Vec<usize> = self
            .planets
            .iter()
            .enumerate()
            .filter(|(_, planet)| planet.hand == Some(hand))
            .map(|(idx, _)| idx)
            .collect();
        if !indices.is_empty() {
            return pick_index(&indices, rng).map(|idx| &self.planets[idx]);
        }
        self.pick_consumable(ConsumableKind::Planet, rng)
    }

    pub fn pick_boss<'a>(&'a self, rng: &mut crate::RngState) -> Option<&'a BossDef> {
        if self.bosses.is_empty() {
            return None;
        }
        let idx = (rng.next_u64() % self.bosses.len() as u64) as usize;
        self.bosses.get(idx)
    }

    pub fn boss_by_id(&self, id: &str) -> Option<&BossDef> {
        self.bosses.iter().find(|boss| boss.id == id)
    }

    pub fn tag_by_id(&self, id: &str) -> Option<&TagDef> {
        self.tags.iter().find(|tag| tag.id == id)
    }

    pub fn random_standard_card(&self, rng: &mut crate::RngState) -> crate::Card {
        let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
        let ranks = [
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

        let suit = suits[(rng.next_u64() % suits.len() as u64) as usize];
        let rank = ranks[(rng.next_u64() % ranks.len() as u64) as usize];
        crate::Card::standard(suit, rank)
    }
}

fn pick_index(items: &[usize], rng: &mut crate::RngState) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let idx = (rng.next_u64() % items.len() as u64) as usize;
    items.get(idx).copied()
}
