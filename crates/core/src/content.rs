use crate::{
    Action, ActivationType, ConsumableKind, EffectBlock, HandKind, JokerEffect, JokerRarity, Rank,
    Suit,
};
use serde::{Deserialize, Serialize};

fn default_weight() -> u32 {
    1
}

/// Identifies whether a card modifier is an Enhancement, Edition, or Seal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardModifierKind {
    Enhancement,
    Edition,
    Seal,
}

/// Data-driven definition for a card modifier's scoring/held effects.
#[derive(Debug, Clone)]
pub struct CardModifierDef {
    pub kind: CardModifierKind,
    /// The lowercase id matching the key used in [`CardAttrRules`](crate::CardAttrRules)
    /// (e.g. `"bonus"`, `"foil"`, `"gold"`).
    pub id: String,
    /// Deterministic effects applied when the trigger fires.
    pub effects: Vec<JokerEffect>,
    /// 1-in-N odds of destroying the card after scoring (0 = never). Used by Glass.
    pub destroy_odds: u32,
    /// 1-in-N odds of a lucky mult proc (0 = never). Used by Lucky.
    pub lucky_mult_odds: u32,
    /// Mult added when the lucky mult roll succeeds.
    pub lucky_mult_add: f64,
    /// 1-in-N odds of a lucky money proc (0 = never). Used by Lucky.
    pub lucky_money_odds: u32,
    /// Money added when the lucky money roll succeeds.
    pub lucky_money_add: i64,
}

impl CardModifierDef {
    /// Convenience constructor for a modifier with only deterministic effects.
    pub fn simple(
        kind: CardModifierKind,
        id: impl Into<String>,
        effects: Vec<JokerEffect>,
    ) -> Self {
        Self {
            kind,
            id: id.into(),
            effects,
            destroy_odds: 0,
            lucky_mult_odds: 0,
            lucky_mult_add: 0.0,
            lucky_money_odds: 0,
            lucky_money_add: 0,
        }
    }

    /// Convenience helper: a single-action scored effect.
    pub fn scored(kind: CardModifierKind, id: impl Into<String>, action: Action) -> Self {
        Self::simple(
            kind,
            id,
            vec![JokerEffect {
                trigger: ActivationType::OnScored,
                when: crate::Expr::Bool(true),
                actions: vec![action],
            }],
        )
    }

    /// Convenience helper: a single-action held effect.
    pub fn held(kind: CardModifierKind, id: impl Into<String>, action: Action) -> Self {
        Self::simple(
            kind,
            id,
            vec![JokerEffect {
                trigger: ActivationType::OnHeld,
                when: crate::Expr::Bool(true),
                actions: vec![action],
            }],
        )
    }
}

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
    /// Relative selection weight. Defaults to 1. Higher = more common.
    pub weight: u32,
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
    /// When true, using this consumable does not update `last_consumable` state.
    /// Defaults to false. Used by The Fool which replays the last consumable.
    #[serde(default)]
    pub skip_last_consumable: bool,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub jokers: Vec<JokerDef>,
    pub bosses: Vec<BossDef>,
    pub tags: Vec<TagDef>,
    pub tarots: Vec<ConsumableDef>,
    pub planets: Vec<ConsumableDef>,
    pub spectrals: Vec<ConsumableDef>,
    /// Data-driven definitions for Enhancement, Edition, and Seal scoring behaviors.
    pub card_modifiers: Vec<CardModifierDef>,
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
        let total_weight: u64 = self.bosses.iter().map(|b| b.weight as u64).sum();
        if total_weight == 0 {
            // All weights zero — fall back to uniform
            let idx = (rng.next_u64() % self.bosses.len() as u64) as usize;
            return self.bosses.get(idx);
        }
        let mut roll = rng.next_u64() % total_weight;
        for boss in &self.bosses {
            if roll < boss.weight as u64 {
                return Some(boss);
            }
            roll -= boss.weight as u64;
        }
        self.bosses.last()
    }

    pub fn boss_by_id(&self, id: &str) -> Option<&BossDef> {
        self.bosses.iter().find(|boss| boss.id == id)
    }

    pub fn tag_by_id(&self, id: &str) -> Option<&TagDef> {
        self.tags.iter().find(|tag| tag.id == id)
    }

    /// Look up a card modifier definition by kind and id.
    pub fn modifier_def(&self, kind: CardModifierKind, id: &str) -> Option<&CardModifierDef> {
        self.card_modifiers
            .iter()
            .find(|d| d.kind == kind && d.id == id)
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
