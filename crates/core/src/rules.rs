use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Score {
    pub chips: i64,
    pub mult: f64,
}

impl Score {
    pub fn total_raw(&self) -> f64 {
        self.chips as f64 * self.mult
    }

    pub fn total(&self) -> i64 {
        self.total_raw().floor() as i64
    }

    pub fn apply(&mut self, effect: &RuleEffect) {
        match effect {
            RuleEffect::AddChips(value) => self.chips += value,
            RuleEffect::AddMult(value) => self.mult += value,
            RuleEffect::MultiplyMult(value) => self.mult *= value,
            RuleEffect::MultiplyChips(value) => {
                let scaled = (self.chips as f64 * value).floor() as i64;
                self.chips = scaled;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Trigger {
    OnHandStart,
    OnCardPlayed,
    OnScoring,
    OnRoundEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleEffect {
    AddChips(i64),
    AddMult(f64),
    MultiplyMult(f64),
    MultiplyChips(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreTraceStep {
    pub source: String,
    pub effect: RuleEffect,
    pub before: Score,
    pub after: Score,
}
