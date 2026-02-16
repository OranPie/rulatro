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
    #[serde(default)]
    pub target_before: i64,
    pub score_before: i64,
    pub action: AutoAction,
    #[serde(default)]
    pub action_detail: Option<String>,
    pub mcts: StepSearchStats,
    pub phase_after: String,
    pub blind_after: String,
    pub ante_after: u8,
    pub money_after: i64,
    #[serde(default)]
    pub target_after: i64,
    pub score_after: i64,
    #[serde(default)]
    pub score_detail: Option<String>,
    #[serde(default)]
    pub ante_detail: Option<String>,
    pub outcome_after: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnteTargetRecord {
    pub ante: u8,
    pub small: i64,
    pub big: i64,
    pub boss: i64,
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
    #[serde(default)]
    pub ante_targets: Vec<AnteTargetRecord>,
    pub steps: Vec<StepRecord>,
    pub summary: SummaryStats,
}

impl AutoplayResult {
    pub fn to_text_report(&self) -> String {
        let mut lines = vec![
            format!("status/状态: {}", run_status_label(&self.status)),
            format!(
                "final/结束: ante/底注={} money/金钱={} blind_score/盲注分数={}/{}",
                self.final_metrics.ante,
                self.final_metrics.money,
                self.final_metrics.blind_score,
                self.final_metrics.blind_target
            ),
            format!(
                "summary/汇总: steps/步数={} simulations/模拟={} wall_ms/耗时毫秒={}",
                self.summary.steps, self.summary.total_simulations, self.summary.wall_time_ms
            ),
            String::new(),
            "ante targets/底注目标:".to_string(),
        ];
        if self.ante_targets.is_empty() {
            lines.push("  (none/无)".to_string());
        } else {
            for item in &self.ante_targets {
                lines.push(format!(
                    "  ante/底注 {:>2}: small/小盲={} big/大盲={} boss/Boss={}",
                    item.ante, item.small, item.big, item.boss
                ));
            }
        }
        lines.push(String::new());
        lines.push("steps/步骤:".to_string());
        for step in &self.steps {
            lines.push(format!(
                "  step/步骤 {:>4} | {}",
                step.step,
                action_name(&step.action)
            ));
            lines.push(format!(
                "    state/状态: {} {} ante/底注 {} target/目标 {} score/分数 {} money/金钱 {}",
                step.phase_before,
                step.blind_before,
                step.ante_before,
                step.target_before,
                step.score_before,
                step.money_before
            ));
            lines.push(format!(
                "      -> {} {} ante/底注 {} target/目标 {} score/分数 {} money/金钱 {}",
                step.phase_after,
                step.blind_after,
                step.ante_after,
                step.target_after,
                step.score_after,
                step.money_after
            ));
            lines.push(format!(
                "    search/搜索: sims/模拟={} elapsed/耗时={}ms children/子节点={} pick_visits/选择访问={} pick_value/选择值={:.2}",
                step.mcts.simulations,
                step.mcts.elapsed_ms,
                step.mcts.root_children,
                step.mcts.selected_visits,
                step.mcts.selected_value
            ));
            lines.push(format!("    events/事件: {}", step.event_count));
            if let Some(outcome) = step.outcome_after.as_ref() {
                lines.push(format!("    outcome/结果: {outcome}"));
            }
            if let Some(detail) = step.action_detail.as_ref() {
                push_block(&mut lines, "action/动作", detail);
            }
            if let Some(detail) = step.score_detail.as_ref() {
                push_block(&mut lines, "score/得分", detail);
            }
            if let Some(detail) = step.ante_detail.as_ref() {
                push_block(&mut lines, "ante/底注", detail);
            }
            lines.push(String::new());
        }
        lines.join("\n")
    }
}

fn push_block(lines: &mut Vec<String>, label: &str, text: &str) {
    for row in text.lines() {
        lines.push(format!("    {label}: {row}"));
    }
}

fn action_name(action: &AutoAction) -> &'static str {
    match action {
        AutoAction::Deal => "Deal/发牌",
        AutoAction::Play { .. } => "Play/出牌",
        AutoAction::Discard { .. } => "Discard/弃牌",
        AutoAction::SkipBlind => "SkipBlind/跳过盲注",
        AutoAction::EnterShop => "EnterShop/进入商店",
        AutoAction::LeaveShop => "LeaveShop/离开商店",
        AutoAction::RerollShop => "RerollShop/刷新商店",
        AutoAction::BuyCard { .. } => "BuyCard/购买卡牌",
        AutoAction::BuyPack { .. } => "BuyPack/购买卡包",
        AutoAction::BuyVoucher { .. } => "BuyVoucher/购买优惠券",
        AutoAction::PickPack { .. } => "PickPack/选择卡包",
        AutoAction::SkipPack => "SkipPack/跳过卡包",
        AutoAction::UseConsumable { .. } => "UseConsumable/使用消耗牌",
        AutoAction::SellJoker { .. } => "SellJoker/出售小丑",
        AutoAction::NextBlind => "NextBlind/下一盲注",
    }
}

fn run_status_label(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::TargetReached => "TargetReached/达到目标",
        RunStatus::Failed => "Failed/失败",
        RunStatus::MaxSteps => "MaxSteps/达到最大步数",
        RunStatus::NoLegalAction => "NoLegalAction/无合法动作",
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
