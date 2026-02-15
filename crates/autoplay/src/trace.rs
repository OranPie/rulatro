use crate::{AutoAction, AutoplayError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    TargetReached,
    Failed,
    MaxSteps,
    NoLegalAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSearchStats {
    pub simulations: u32,
    pub elapsed_ms: u64,
    pub root_children: usize,
    pub selected_visits: u32,
    pub selected_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step: u32,
    pub phase_before: String,
    pub blind_before: String,
    pub ante_before: u8,
    pub money_before: i64,
    pub score_before: i64,
    pub action: AutoAction,
    pub mcts: StepSearchStats,
    pub phase_after: String,
    pub blind_after: String,
    pub ante_after: u8,
    pub money_after: i64,
    pub score_after: i64,
    pub outcome_after: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalMetrics {
    pub ante: u8,
    pub money: i64,
    pub blind_score: i64,
    pub blind_target: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub steps: u32,
    pub total_simulations: u64,
    pub wall_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoplayResult {
    pub status: RunStatus,
    pub final_metrics: FinalMetrics,
    pub steps: Vec<StepRecord>,
    pub summary: SummaryStats,
}

impl AutoplayResult {
    pub fn to_text_report(&self) -> String {
        let mut lines = vec![
            format!("status: {:?}", self.status),
            format!(
                "final: ante={} money={} blind_score={}/{}",
                self.final_metrics.ante,
                self.final_metrics.money,
                self.final_metrics.blind_score,
                self.final_metrics.blind_target
            ),
            format!(
                "summary: steps={} simulations={} wall_ms={}",
                self.summary.steps, self.summary.total_simulations, self.summary.wall_time_ms
            ),
            String::new(),
            "steps:".to_string(),
        ];
        for step in &self.steps {
            lines.push(format!(
                "  {:>4} | {:?} | {} -> {} | ${} -> ${} | score {} -> {} | sims {}",
                step.step,
                step.action,
                step.phase_before,
                step.phase_after,
                step.money_before,
                step.money_after,
                step.score_before,
                step.score_after,
                step.mcts.simulations
            ));
        }
        lines.join("\n")
    }
}

pub fn write_json(path: &Path, result: &AutoplayResult) -> Result<(), AutoplayError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serde_json::to_string_pretty(result)?;
    fs::write(path, body)?;
    Ok(())
}

pub fn write_text(path: &Path, result: &AutoplayResult) -> Result<(), AutoplayError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, result.to_text_report())?;
    Ok(())
}
