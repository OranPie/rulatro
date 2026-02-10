use crate::{
    evaluate_hand, evaluate_hand_with_rules, level_kind, scoring_cards, GameConfig, HandEvalRules,
    HandKind, Rank, Score,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScoreTables {
    hand_rules: HashMap<String, (i64, f64)>,
    hand_level_rules: HashMap<String, (i64, f64)>,
    rank_chips: HashMap<Rank, i64>,
}

impl ScoreTables {
    pub fn from_config(config: &GameConfig) -> Self {
        let mut hand_rules = HashMap::new();
        let mut hand_level_rules = HashMap::new();
        for hand in &config.hands {
            hand_rules.insert(hand.id.clone(), (hand.base_chips, hand.base_mult));
            hand_level_rules.insert(hand.id.clone(), (hand.level_chips, hand.level_mult));
        }
        let mut rank_chips = HashMap::new();
        for rank in &config.ranks {
            rank_chips.insert(rank.rank, rank.chips);
        }
        Self {
            hand_rules,
            hand_level_rules,
            rank_chips,
        }
    }

    pub fn hand_base(&self, kind: HandKind) -> (i64, f64) {
        self.hand_rules
            .get(kind.id())
            .copied()
            .unwrap_or_else(|| default_hand_base(kind))
    }

    pub fn hand_base_for_level(&self, kind: HandKind, level: u32) -> (i64, f64) {
        let (base_chips, base_mult) = self.hand_base(kind);
        let (level_chips, level_mult) = self
            .hand_level_rules
            .get(kind.id())
            .copied()
            .unwrap_or((0, 0.0));
        if level <= 1 {
            return (base_chips, base_mult);
        }
        let extra = (level - 1) as i64;
        let chips = base_chips.saturating_add(level_chips.saturating_mul(extra));
        let mult = base_mult + level_mult * extra as f64;
        (chips, mult)
    }

    pub fn rank_chips(&self, rank: Rank) -> i64 {
        *self.rank_chips.get(&rank).unwrap_or(&0)
    }
}

#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub hand: HandKind,
    pub base: Score,
    pub rank_chips: i64,
    pub scoring_indices: Vec<usize>,
    pub total: Score,
}

pub fn score_hand(cards: &[crate::Card], tables: &ScoreTables) -> ScoreBreakdown {
    let hand = evaluate_hand(cards);
    let (base_chips, base_mult) = tables.hand_base_for_level(hand, 1);
    let base = Score {
        chips: base_chips,
        mult: base_mult,
    };

    let scoring = scoring_cards(cards, hand);
    let rank_chips: i64 = scoring
        .iter()
        .map(|idx| {
            if cards[*idx].is_stone() {
                0
            } else {
                tables.rank_chips(cards[*idx].rank)
            }
        })
        .sum();

    let total = Score {
        chips: base.chips + rank_chips,
        mult: base.mult,
    };

    ScoreBreakdown {
        hand,
        base,
        rank_chips,
        scoring_indices: scoring,
        total,
    }
}

pub fn score_hand_with_rules(
    cards: &[crate::Card],
    tables: &ScoreTables,
    rules: HandEvalRules,
    hand_levels: &HashMap<HandKind, u32>,
) -> ScoreBreakdown {
    let hand = evaluate_hand_with_rules(cards, rules);
    let level_key = level_kind(hand);
    let level = hand_levels.get(&level_key).copied().unwrap_or(1);
    let (base_chips, base_mult) = tables.hand_base_for_level(hand, level);
    let base = Score {
        chips: base_chips,
        mult: base_mult,
    };

    let scoring = scoring_cards(cards, hand);
    let rank_chips: i64 = scoring
        .iter()
        .map(|idx| {
            if cards[*idx].is_stone() {
                0
            } else {
                tables.rank_chips(cards[*idx].rank)
            }
        })
        .sum();

    let total = Score {
        chips: base.chips + rank_chips,
        mult: base.mult,
    };

    ScoreBreakdown {
        hand,
        base,
        rank_chips,
        scoring_indices: scoring,
        total,
    }
}

fn default_hand_base(kind: HandKind) -> (i64, f64) {
    match kind {
        HandKind::HighCard => (5, 1.0),
        HandKind::Pair => (10, 2.0),
        HandKind::TwoPair => (20, 2.0),
        HandKind::Trips => (30, 3.0),
        HandKind::Straight => (30, 4.0),
        HandKind::Flush => (35, 4.0),
        HandKind::FullHouse => (40, 4.0),
        HandKind::Quads => (60, 7.0),
        HandKind::StraightFlush | HandKind::RoyalFlush => (100, 8.0),
        HandKind::FiveOfAKind => (120, 12.0),
        HandKind::FlushHouse => (140, 14.0),
        HandKind::FlushFive => (160, 16.0),
    }
}
