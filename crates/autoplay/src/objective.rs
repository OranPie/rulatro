use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TargetConfig {
    pub target_score: Option<i64>,
    pub target_ante: Option<u8>,
    pub target_money: Option<i64>,
    pub stop_on_blind_failed: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            target_score: None,
            target_ante: Some(4),
            target_money: None,
            stop_on_blind_failed: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    pub score: f64,
    pub ante: f64,
    pub money: f64,
    pub survival: f64,
    pub steps_penalty: f64,
}

impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            score: 1.0,
            ante: 2.0,
            money: 0.8,
            survival: 5.0,
            steps_penalty: 0.01,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub ante: u8,
    pub money: i64,
    pub blind_score: i64,
    pub blind_target: i64,
    pub blind_failed: bool,
    pub blind_cleared: bool,
}

pub fn target_reached(metrics: EvalMetrics, targets: TargetConfig) -> bool {
    if let Some(target) = targets.target_score {
        if metrics.blind_score < target {
            return false;
        }
    }
    if let Some(target) = targets.target_ante {
        if metrics.ante < target {
            return false;
        }
    }
    if let Some(target) = targets.target_money {
        if metrics.money < target {
            return false;
        }
    }
    targets.target_score.is_some()
        || targets.target_ante.is_some()
        || targets.target_money.is_some()
}

pub fn weighted_score(metrics: EvalMetrics, weights: ObjectiveWeights, total_steps: u32) -> f64 {
    let score_norm = if metrics.blind_target > 0 {
        metrics.blind_score as f64 / metrics.blind_target as f64
    } else {
        0.0
    };
    let ante_norm = metrics.ante as f64;
    let money_norm = metrics.money.max(0) as f64 / 100.0;
    let survival = if metrics.blind_failed {
        -1.0
    } else if metrics.blind_cleared {
        1.0
    } else {
        0.0
    };

    weights.score * score_norm
        + weights.ante * ante_norm
        + weights.money * money_norm
        + weights.survival * survival
        - weights.steps_penalty * total_steps as f64
}
