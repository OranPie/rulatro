use crate::{
    target_reached, weighted_score, AutoAction, AutoplayConfig, AutoplayError, AutoplayResult,
    EvalMetrics, FinalMetrics, ObjectiveWeights, RunStatus, Simulator, StepRecord, StepSearchStats,
    SummaryStats, TargetConfig,
};
use std::collections::HashSet;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct AutoplayRequest {
    pub config: AutoplayConfig,
    pub targets: TargetConfig,
    pub weights: ObjectiveWeights,
}

impl Default for AutoplayRequest {
    fn default() -> Self {
        Self {
            config: AutoplayConfig::default(),
            targets: TargetConfig::default(),
            weights: ObjectiveWeights::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct Node {
    parent: Option<usize>,
    action: Option<AutoAction>,
    visits: u32,
    value_sum: f64,
    children: Vec<usize>,
    unexpanded: Vec<AutoAction>,
    terminal: bool,
    depth: u32,
    prior: f64,
}

impl Node {
    fn new_root(unexpanded: Vec<AutoAction>, terminal: bool) -> Self {
        Self {
            parent: None,
            action: None,
            visits: 0,
            value_sum: 0.0,
            children: Vec::new(),
            unexpanded,
            terminal,
            depth: 0,
            prior: 1.0,
        }
    }

    fn new_child(
        parent: usize,
        action: AutoAction,
        unexpanded: Vec<AutoAction>,
        terminal: bool,
        depth: u32,
        prior: f64,
    ) -> Self {
        Self {
            parent: Some(parent),
            action: Some(action),
            visits: 0,
            value_sum: 0.0,
            children: Vec::new(),
            unexpanded,
            terminal,
            depth,
            prior,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(
            seed.wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add(0xD1B54A32D192ED03),
        )
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 7;
        self.0 ^= self.0 >> 9;
        self.0 ^= self.0 << 8;
        self.0
    }

    fn gen_unit_f64(&mut self) -> f64 {
        let denom = u64::MAX as f64;
        if denom <= 0.0 {
            0.0
        } else {
            (self.next_u64() as f64) / denom
        }
    }
}

#[derive(Debug, Clone)]
struct TacticalChoice {
    action: AutoAction,
    score: f64,
    immediate_clear: bool,
    skip_blind_penalty: f64,
}

#[derive(Debug, Clone)]
struct SurvivalChoice {
    action: AutoAction,
    score: f64,
    reason: String,
}

pub fn run_autoplay<F>(
    factory: &F,
    request: &AutoplayRequest,
) -> Result<AutoplayResult, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let started_at = Instant::now();
    let mut sim = factory()?;
    let ante_targets = sim.collect_ante_targets();
    let mut history: Vec<AutoAction> = Vec::new();
    let mut records: Vec<StepRecord> = Vec::new();
    let mut total_simulations: u64 = 0;
    let mut tactical_bypass_steps: u32 = 0;
    let mut status = None;

    for step in 0..request.config.max_steps {
        let before = sim.metrics();
        if target_reached(before, request.targets) {
            status = Some(RunStatus::TargetReached);
            break;
        }
        if request.targets.stop_on_blind_failed && before.blind_failed {
            status = Some(RunStatus::Failed);
            break;
        }

        let mut blocked_actions: HashSet<String> = HashSet::new();
        let mut committed = false;
        let retries = request.config.action_retry_limit.max(1);
        for _ in 0..retries {
            let root_candidates = sim
                .legal_actions(&request.config)
                .into_iter()
                .filter(|action| !blocked_actions.contains(&action.stable_key()))
                .collect::<Vec<_>>();
            if root_candidates.is_empty() {
                status = Some(RunStatus::NoLegalAction);
                break;
            }

            let step_started = Instant::now();
            let survival_choice = select_survival_guard_action(
                factory,
                &history,
                step,
                &sim,
                &request.config,
                request.targets,
                request.weights,
                &root_candidates,
            )?;
            let endgame_choice = if request.config.endgame_exact_lookahead {
                select_endgame_exact_action(
                    factory,
                    &history,
                    step,
                    &request.config,
                    request.targets,
                    request.weights,
                    &root_candidates,
                )?
            } else {
                None
            };
            let tactical_trigger = tactical_trigger_reason(&sim, &request.config);
            let tactical_choice = if endgame_choice.is_none() && survival_choice.is_none() {
                if tactical_trigger.is_some() {
                    select_tactical_action(
                        factory,
                        &history,
                        step,
                        before,
                        &request.config,
                        request.targets,
                        request.weights,
                        &root_candidates,
                    )?
                } else {
                    None
                }
            } else {
                None
            };

            let (mut action, mut search_stats) = if let Some(survival) = survival_choice {
                (
                    survival.action.clone(),
                    StepSearchStats {
                        simulations: 0,
                        elapsed_ms: step_started.elapsed().as_millis() as u64,
                        root_children: root_candidates.len(),
                        selected_visits: 0,
                        selected_value: survival.score,
                        decision_source: "survival_guard".to_string(),
                        tactical_trigger: Some(survival.reason),
                        tactical_candidate: Some(survival.action.short_label()),
                        tactical_bypassed_mcts: true,
                        forced_min_sims: 0,
                        skip_blind_penalty: 0.0,
                    },
                )
            } else if let Some((choice, score)) = endgame_choice {
                (
                    choice,
                    StepSearchStats {
                        simulations: 0,
                        elapsed_ms: step_started.elapsed().as_millis() as u64,
                        root_children: root_candidates.len(),
                        selected_visits: 0,
                        selected_value: score,
                        decision_source: "endgame_exact".to_string(),
                        tactical_trigger,
                        tactical_candidate: None,
                        tactical_bypassed_mcts: true,
                        forced_min_sims: 0,
                        skip_blind_penalty: 0.0,
                    },
                )
            } else if let Some(tactical) = tactical_choice {
                let completed_steps = history.len() as u32;
                let bypass_allowed = tactical.immediate_clear
                    || (request.config.tactical_force_min_sims == 0
                        && tactical_bypass_allowed(
                            tactical_bypass_steps,
                            completed_steps,
                            request.config.tactical_max_step_share,
                        ));
                if bypass_allowed {
                    tactical_bypass_steps = tactical_bypass_steps.saturating_add(1);
                    (
                        tactical.action.clone(),
                        StepSearchStats {
                            simulations: 0,
                            elapsed_ms: step_started.elapsed().as_millis() as u64,
                            root_children: root_candidates.len(),
                            selected_visits: 0,
                            selected_value: tactical.score,
                            decision_source: "tactical".to_string(),
                            tactical_trigger,
                            tactical_candidate: Some(tactical.action.short_label()),
                            tactical_bypassed_mcts: true,
                            forced_min_sims: 0,
                            skip_blind_penalty: tactical.skip_blind_penalty,
                        },
                    )
                } else {
                    let forced_min_sims = request
                        .config
                        .min_simulations_per_step
                        .max(request.config.tactical_force_min_sims);
                    match select_action_mcts(
                        factory,
                        &history,
                        step,
                        &request.config,
                        request.targets,
                        request.weights,
                        &root_candidates,
                        Some(forced_min_sims),
                        Some(&tactical.action),
                    ) {
                        Ok((picked, mut stats)) => {
                            stats.decision_source = "mcts+tactical".to_string();
                            stats.tactical_trigger = tactical_trigger;
                            stats.tactical_candidate = Some(tactical.action.short_label());
                            stats.tactical_bypassed_mcts = false;
                            stats.forced_min_sims = forced_min_sims;
                            stats.skip_blind_penalty = tactical.skip_blind_penalty;
                            (picked, stats)
                        }
                        Err(err) if is_recoverable_action_error(&err) => (
                            tactical.action.clone(),
                            StepSearchStats {
                                simulations: 0,
                                elapsed_ms: step_started.elapsed().as_millis() as u64,
                                root_children: root_candidates.len(),
                                selected_visits: 0,
                                selected_value: tactical.score,
                                decision_source: "tactical_fallback".to_string(),
                                tactical_trigger,
                                tactical_candidate: Some(tactical.action.short_label()),
                                tactical_bypassed_mcts: true,
                                forced_min_sims,
                                skip_blind_penalty: tactical.skip_blind_penalty,
                            },
                        ),
                        Err(err) => return Err(err),
                    }
                }
            } else {
                match select_action_mcts(
                    factory,
                    &history,
                    step,
                    &request.config,
                    request.targets,
                    request.weights,
                    &root_candidates,
                    None,
                    None,
                ) {
                    Ok(mut value) => {
                        value.1.decision_source = "mcts".to_string();
                        (value.0, value.1)
                    }
                    Err(err) if is_recoverable_action_error(&err) => {
                        match pick_non_failing_root_action(
                            factory,
                            &history,
                            step,
                            &request.config,
                            request.targets,
                            request.weights,
                            &root_candidates,
                        )? {
                            Some((fallback, score)) => (
                                fallback,
                                StepSearchStats {
                                    simulations: 0,
                                    elapsed_ms: step_started.elapsed().as_millis() as u64,
                                    root_children: root_candidates.len(),
                                    selected_visits: 0,
                                    selected_value: score,
                                    decision_source: "fallback_safe_root".to_string(),
                                    tactical_trigger: Some(
                                        "recoverable_mcts_error_choose_safe_root".to_string(),
                                    ),
                                    tactical_candidate: None,
                                    tactical_bypassed_mcts: true,
                                    forced_min_sims: 0,
                                    skip_blind_penalty: 0.0,
                                },
                            ),
                            None => {
                                status = Some(RunStatus::NoLegalAction);
                                break;
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            };
            if request.targets.stop_on_blind_failed {
                let path = vec![action.clone()];
                if let Ok((probe, _, _)) = materialize(
                    factory,
                    &history,
                    &path,
                    &request.config,
                    request.targets,
                    step,
                ) {
                    let after = probe.metrics();
                    if after.blind_failed {
                        if let Some((fallback, score)) = pick_non_failing_root_action(
                            factory,
                            &history,
                            step,
                            &request.config,
                            request.targets,
                            request.weights,
                            &root_candidates,
                        )? {
                            action = fallback;
                            search_stats.selected_value = score;
                            search_stats.decision_source =
                                format!("{}+avoid_fail", search_stats.decision_source);
                            if search_stats.tactical_trigger.is_none() {
                                search_stats.tactical_trigger =
                                    Some("predicted_blind_failed_swap_action".to_string());
                            }
                        }
                    }
                }
            }
            if let Some((discard_action, discard_score, reason)) = prefer_discard_over_weak_play(
                factory,
                &history,
                step,
                &request.config,
                request.targets,
                request.weights,
                &root_candidates,
                &action,
            )? {
                action = discard_action;
                search_stats.selected_value = discard_score;
                search_stats.decision_source =
                    format!("{}+discard_guard", search_stats.decision_source);
                search_stats.tactical_candidate = Some(action.short_label());
                search_stats.tactical_trigger = Some(reason);
            }
            total_simulations = total_simulations.saturating_add(search_stats.simulations as u64);

            let phase_before = sim.phase_name();
            let blind_before = sim.blind_name();
            let blind_kind_before = sim.run.state.blind;
            let action_detail = sim.describe_action(&action);
            match sim.apply_action(&action) {
                Ok(event_count) => {
                    let after = sim.metrics();
                    let score_detail = sim.describe_score_detail(
                        &action,
                        before.blind_score,
                        before.blind_target,
                        after.blind_score,
                    );
                    let ante_detail = sim.describe_ante_detail(
                        before.ante,
                        blind_kind_before,
                        before.blind_target,
                    );
                    records.push(StepRecord {
                        step,
                        phase_before,
                        blind_before,
                        ante_before: before.ante,
                        money_before: before.money,
                        target_before: before.blind_target,
                        score_before: before.blind_score,
                        action: action.clone(),
                        action_detail,
                        mcts: search_stats,
                        phase_after: sim.phase_name(),
                        blind_after: sim.blind_name(),
                        ante_after: after.ante,
                        money_after: after.money,
                        target_after: after.blind_target,
                        score_after: after.blind_score,
                        score_detail,
                        ante_detail,
                        outcome_after: sim.run.blind_outcome().map(|value| format!("{value:?}")),
                        event_count,
                    });
                    history.push(action);
                    committed = true;
                    break;
                }
                Err(err) => {
                    if !is_recoverable_action_error(&err) {
                        return Err(err);
                    }
                    blocked_actions.insert(action.stable_key());
                    let (recovered, _, _) = materialize(
                        factory,
                        &history,
                        &[],
                        &request.config,
                        request.targets,
                        step,
                    )?;
                    sim = recovered;
                }
            }
        }

        if !committed {
            status.get_or_insert(RunStatus::NoLegalAction);
            break;
        }

        let after = sim.metrics();
        if target_reached(after, request.targets) {
            status = Some(RunStatus::TargetReached);
            break;
        }
        if request.targets.stop_on_blind_failed && after.blind_failed {
            status = Some(RunStatus::Failed);
            break;
        }
    }

    let final_metrics = sim.metrics();
    let status = status.unwrap_or(RunStatus::MaxSteps);
    Ok(AutoplayResult {
        status,
        final_metrics: FinalMetrics {
            ante: final_metrics.ante,
            money: final_metrics.money,
            blind_score: final_metrics.blind_score,
            blind_target: final_metrics.blind_target,
        },
        ante_targets,
        steps: records,
        summary: SummaryStats {
            steps: history.len() as u32,
            total_simulations,
            wall_time_ms: started_at.elapsed().as_millis() as u64,
        },
    })
}

fn tactical_trigger_reason(sim: &Simulator, cfg: &AutoplayConfig) -> Option<String> {
    if sim.run.state.target <= 0 {
        return None;
    }
    let phase = sim.run.state.phase;
    if phase != rulatro_core::Phase::Play && phase != rulatro_core::Phase::Deal {
        return None;
    }
    let deficit = (sim.run.state.target - sim.run.state.blind_score).max(0);
    let pressure_margin = if sim.run.state.hands_left <= 1 {
        cfg.tactical_finish_margin.saturating_mul(2)
    } else {
        cfg.tactical_finish_margin
    };
    if deficit > pressure_margin {
        return None;
    }
    let reason = if sim.run.state.hands_left <= 1 && deficit > cfg.tactical_finish_margin {
        format!(
            "last_hand_pressure deficit={deficit} margin={pressure_margin} hands_left={}",
            sim.run.state.hands_left
        )
    } else {
        format!(
            "near_finish deficit={deficit} margin={} phase={phase:?}",
            cfg.tactical_finish_margin
        )
    };
    Some(reason)
}

fn tactical_bypass_allowed(bypass_steps: u32, completed_steps: u32, max_share: f64) -> bool {
    if max_share <= 0.0 {
        return false;
    }
    if !max_share.is_finite() {
        return true;
    }
    let total = completed_steps.max(1) as f64;
    let share = bypass_steps as f64 / total;
    share < max_share
}

fn select_tactical_action<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    baseline: EvalMetrics,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_actions: &[AutoAction],
) -> Result<Option<TacticalChoice>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let candidates = dedup_actions(root_actions.to_vec());
    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_immediate_clear = false;
    let mut best_skip_penalty = 0.0f64;
    let mut best_key = String::new();
    let mut found_terminal_clear = false;

    for action in candidates {
        let path = vec![action.clone()];
        let (sim_after, terminal, next_legal) =
            match materialize(factory, history, &path, cfg, targets, step) {
                Ok(value) => value,
                Err(err) if is_recoverable_action_error(&err) => continue,
                Err(err) => return Err(err),
            };

        let after = sim_after.metrics();
        let mut score = weighted_score(after, weights, step + 1);
        let mut skip_penalty = 0.0;
        if matches!(action, AutoAction::SkipBlind) {
            skip_penalty = skip_blind_deficit_penalty(baseline, after, cfg);
            score -= skip_penalty;
        }
        let clears = after.blind_cleared || target_reached(after, targets);
        if clears {
            score += 20_000.0;
            found_terminal_clear = true;
        }
        if after.blind_failed {
            score -= 10_000.0;
        }

        if !terminal && !found_terminal_clear {
            let mut lookahead_best = f64::NEG_INFINITY;
            let next_actions =
                prioritize_tactical_next_actions(next_legal, cfg.rollout_top_k.max(1) * 2);
            for next in next_actions {
                let path2 = vec![action.clone(), next];
                let (sim2, _, _) = match materialize(factory, history, &path2, cfg, targets, step) {
                    Ok(value) => value,
                    Err(err) if is_recoverable_action_error(&err) => continue,
                    Err(err) => return Err(err),
                };
                let metric2 = sim2.metrics();
                let mut next_score = weighted_score(metric2, weights, step + 2);
                if metric2.blind_cleared || target_reached(metric2, targets) {
                    next_score += 8_000.0;
                }
                if metric2.blind_failed {
                    next_score -= 8_000.0;
                }
                if next_score > lookahead_best {
                    lookahead_best = next_score;
                }
            }
            if lookahead_best.is_finite() {
                score += lookahead_best * 0.42;
            }
        }

        let key = action.stable_key();
        if score > best_score || (score == best_score && key < best_key) {
            best_score = score;
            best_key = key;
            best_action = Some(action);
            best_immediate_clear = clears;
            best_skip_penalty = skip_penalty;
        }
    }

    Ok(best_action.map(|action| TacticalChoice {
        action,
        score: best_score,
        immediate_clear: best_immediate_clear,
        skip_blind_penalty: best_skip_penalty,
    }))
}

fn prioritize_tactical_next_actions(actions: Vec<AutoAction>, cap: usize) -> Vec<AutoAction> {
    let mut unique = dedup_actions(actions);
    unique.sort_by(|a, b| {
        action_expand_priority(b)
            .cmp(&action_expand_priority(a))
            .then_with(|| a.stable_key().cmp(&b.stable_key()))
    });
    unique.into_iter().take(cap.max(1)).collect()
}

fn skip_blind_deficit_penalty(
    before: EvalMetrics,
    after: EvalMetrics,
    cfg: &AutoplayConfig,
) -> f64 {
    let before_deficit = (before.blind_target - before.blind_score).max(0) as f64;
    let after_deficit = (after.blind_target - after.blind_score).max(0) as f64;
    let pressure = if before.blind_target > 0 {
        (before_deficit / before.blind_target as f64).clamp(0.0, 1.0)
    } else {
        1.0
    };
    if after_deficit > before_deficit {
        let scaled = cfg.skip_blind_deficit_penalty * (1.0 - 0.45 * pressure);
        scaled.max(cfg.skip_blind_deficit_penalty * 0.35)
    } else {
        cfg.skip_blind_deficit_penalty * 0.2
    }
}

fn pick_non_failing_root_action<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_actions: &[AutoAction],
) -> Result<Option<(AutoAction, f64)>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_key = String::new();
    for action in dedup_actions(root_actions.to_vec()) {
        let path = vec![action.clone()];
        let (sim_after, _, _) = match materialize(factory, history, &path, cfg, targets, step) {
            Ok(value) => value,
            Err(err) if is_recoverable_action_error(&err) => continue,
            Err(err) => return Err(err),
        };
        let metrics = sim_after.metrics();
        if metrics.blind_failed {
            continue;
        }
        let mut score = weighted_score(metrics, weights, step + 1);
        if metrics.blind_cleared || target_reached(metrics, targets) {
            score += 12_000.0;
        }
        let key = action.stable_key();
        if score > best_score || (score == best_score && key < best_key) {
            best_score = score;
            best_key = key;
            best_action = Some(action);
        }
    }
    Ok(best_action.map(|action| (action, best_score)))
}

fn prefer_discard_over_weak_play<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_actions: &[AutoAction],
    chosen: &AutoAction,
) -> Result<Option<(AutoAction, f64, String)>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let AutoAction::Play { .. } = chosen else {
        return Ok(None);
    };

    let root_sim = replay_history(factory, history)?;
    if root_sim.run.state.phase != rulatro_core::Phase::Play
        || root_sim.run.state.discards_left == 0
    {
        return Ok(None);
    }
    let before = root_sim.metrics();
    let deficit_before = (before.blind_target - before.blind_score).max(0);
    if deficit_before <= 0 {
        return Ok(None);
    }

    let chosen_path = vec![chosen.clone()];
    let (chosen_after_sim, _, _) =
        match materialize(factory, history, &chosen_path, cfg, targets, step) {
            Ok(value) => value,
            Err(err) if is_recoverable_action_error(&err) => return Ok(None),
            Err(err) => return Err(err),
        };
    let chosen_after = chosen_after_sim.metrics();
    let play_gain = (chosen_after.blind_score - before.blind_score).max(0);
    let mut chosen_score = weighted_score(chosen_after, weights, step + 1);
    if chosen_after.blind_cleared || target_reached(chosen_after, targets) {
        chosen_score += 10_000.0;
    }
    if chosen_after.blind_failed {
        chosen_score -= 10_000.0;
    }

    let gain_ratio = play_gain as f64 / deficit_before as f64;
    let urgent = root_sim.run.state.hands_left <= 2;
    if !chosen_after.blind_failed && !urgent && gain_ratio >= 0.55 {
        return Ok(None);
    }

    let discard_candidates = dedup_actions(root_actions.to_vec())
        .into_iter()
        .filter(|action| matches!(action, AutoAction::Discard { .. }))
        .collect::<Vec<_>>();
    if discard_candidates.is_empty() {
        return Ok(None);
    }

    let mut best_discard = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_key = String::new();
    for discard in discard_candidates {
        let path = vec![discard.clone()];
        let (sim_after, terminal, next_legal) =
            match materialize(factory, history, &path, cfg, targets, step) {
                Ok(value) => value,
                Err(err) if is_recoverable_action_error(&err) => continue,
                Err(err) => return Err(err),
            };
        let after = sim_after.metrics();
        if after.blind_failed {
            continue;
        }
        let mut score = weighted_score(after, weights, step + 1);
        if !terminal {
            let follow_actions =
                prioritize_tactical_next_actions(next_legal, cfg.rollout_top_k.max(1) * 2);
            let mut follow_best = f64::NEG_INFINITY;
            for next in follow_actions
                .into_iter()
                .filter(|next| matches!(next, AutoAction::Play { .. }))
            {
                let path2 = vec![discard.clone(), next];
                let (sim2, _, _) = match materialize(factory, history, &path2, cfg, targets, step) {
                    Ok(value) => value,
                    Err(err) if is_recoverable_action_error(&err) => continue,
                    Err(err) => return Err(err),
                };
                let metrics2 = sim2.metrics();
                let mut s2 = weighted_score(metrics2, weights, step + 2);
                if metrics2.blind_cleared || target_reached(metrics2, targets) {
                    s2 += 8_000.0;
                }
                if metrics2.blind_failed {
                    s2 -= 8_000.0;
                }
                if s2 > follow_best {
                    follow_best = s2;
                }
            }
            if follow_best.is_finite() {
                score += follow_best * 0.45;
            }
        }

        let key = discard.stable_key();
        if score > best_score || (score == best_score && key < best_key) {
            best_score = score;
            best_key = key;
            best_discard = Some(discard);
        }
    }

    let threshold = if chosen_after.blind_failed { 0.0 } else { 0.3 };
    if let Some(discard) = best_discard {
        if best_score > chosen_score + threshold {
            return Ok(Some((
                discard,
                best_score,
                format!(
                    "discard_guard deficit={} play_gain={} ratio={:.2}",
                    deficit_before, play_gain, gain_ratio
                ),
            )));
        }
    }
    Ok(None)
}

fn select_endgame_exact_action<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_actions: &[AutoAction],
) -> Result<Option<(AutoAction, f64)>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let root_sim = replay_history(factory, history)?;
    if root_sim.run.state.phase != rulatro_core::Phase::Play || root_sim.run.state.hands_left > 1 {
        return Ok(None);
    }

    let candidates = dedup_actions(root_actions.to_vec())
        .into_iter()
        .filter(|action| matches!(action, AutoAction::Play { .. } | AutoAction::Discard { .. }))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(None);
    }

    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_key = String::new();

    for action in candidates {
        let path = vec![action.clone()];
        let (sim_after, terminal, next_legal) =
            match materialize(factory, history, &path, cfg, targets, step) {
                Ok(value) => value,
                Err(err) if is_recoverable_action_error(&err) => continue,
                Err(err) => return Err(err),
            };
        let mut score = endgame_action_score(sim_after.metrics(), targets, weights, step + 1);

        if matches!(action, AutoAction::Discard { .. }) && !terminal {
            let followups =
                prioritize_tactical_next_actions(next_legal, cfg.rollout_top_k.max(1) * 2);
            for next in followups
                .into_iter()
                .filter(|next| matches!(next, AutoAction::Play { .. } | AutoAction::Discard { .. }))
            {
                let path2 = vec![action.clone(), next];
                let (sim2, _, _) = match materialize(factory, history, &path2, cfg, targets, step) {
                    Ok(value) => value,
                    Err(err) if is_recoverable_action_error(&err) => continue,
                    Err(err) => return Err(err),
                };
                score = score.max(endgame_action_score(
                    sim2.metrics(),
                    targets,
                    weights,
                    step + 2,
                ));
            }
        }

        let key = action.stable_key();
        if score > best_score || (score == best_score && key < best_key) {
            best_score = score;
            best_key = key;
            best_action = Some(action);
        }
    }

    Ok(best_action.map(|action| (action, best_score)))
}

fn endgame_action_score(
    metrics: EvalMetrics,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    total_steps: u32,
) -> f64 {
    let mut score = weighted_score(metrics, weights, total_steps);
    if metrics.blind_cleared || target_reached(metrics, targets) {
        score += 30_000.0;
    }
    if metrics.blind_failed {
        score -= 20_000.0;
    }
    let deficit = (metrics.blind_target - metrics.blind_score).max(0) as f64;
    score - deficit * 0.35
}

fn select_survival_guard_action<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    sim: &Simulator,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_actions: &[AutoAction],
) -> Result<Option<SurvivalChoice>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    if sim.run.state.phase != rulatro_core::Phase::Play
        || sim.run.state.discards_left == 0
        || sim.run.state.hands_left == 0
    {
        return Ok(None);
    }

    let baseline = sim.metrics();
    let deficit = (baseline.blind_target - baseline.blind_score).max(0);
    if deficit <= 0 {
        return Ok(None);
    }

    let candidates = dedup_actions(root_actions.to_vec());
    let play_actions = candidates
        .iter()
        .filter(|action| matches!(action, AutoAction::Play { .. }))
        .cloned()
        .collect::<Vec<_>>();
    let discard_actions = candidates
        .iter()
        .filter(|action| matches!(action, AutoAction::Discard { .. }))
        .cloned()
        .collect::<Vec<_>>();
    if discard_actions.is_empty() {
        return Ok(None);
    }

    let mut can_clear_now = false;
    let mut best_play_gain = 0i64;
    for play in play_actions {
        let path = vec![play];
        let (sim_after, _, _) = match materialize(factory, history, &path, cfg, targets, step) {
            Ok(value) => value,
            Err(err) if is_recoverable_action_error(&err) => continue,
            Err(err) => return Err(err),
        };
        let after = sim_after.metrics();
        if after.blind_cleared || target_reached(after, targets) {
            can_clear_now = true;
            break;
        }
        best_play_gain = best_play_gain.max((after.blind_score - baseline.blind_score).max(0));
    }
    if can_clear_now {
        return Ok(None);
    }

    let hands_left = sim.run.state.hands_left;
    let desperation =
        hands_left <= 1 || (hands_left == 2 && (best_play_gain as f64) < (deficit as f64 * 0.45));
    if !desperation {
        return Ok(None);
    }

    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_key = String::new();
    for discard in discard_actions {
        let path = vec![discard.clone()];
        let (sim_after, terminal, next_legal) =
            match materialize(factory, history, &path, cfg, targets, step) {
                Ok(value) => value,
                Err(err) if is_recoverable_action_error(&err) => continue,
                Err(err) => return Err(err),
            };
        let after = sim_after.metrics();
        if after.blind_failed {
            continue;
        }

        let mut score = weighted_score(after, weights, step + 1) + 6_000.0;
        if !terminal {
            let next_actions =
                prioritize_tactical_next_actions(next_legal, cfg.rollout_top_k.max(1) * 2);
            let mut follow_best = f64::NEG_INFINITY;
            for next in next_actions {
                let path2 = vec![discard.clone(), next];
                let (sim2, _, _) = match materialize(factory, history, &path2, cfg, targets, step) {
                    Ok(value) => value,
                    Err(err) if is_recoverable_action_error(&err) => continue,
                    Err(err) => return Err(err),
                };
                let metrics2 = sim2.metrics();
                let mut follow_score = weighted_score(metrics2, weights, step + 2);
                if metrics2.blind_cleared || target_reached(metrics2, targets) {
                    follow_score += 15_000.0;
                }
                if metrics2.blind_failed {
                    follow_score -= 15_000.0;
                }
                if follow_score > follow_best {
                    follow_best = follow_score;
                }
            }
            if follow_best.is_finite() {
                score += follow_best * 0.38;
            }
        }

        let key = discard.stable_key();
        if score > best_score || (score == best_score && key < best_key) {
            best_score = score;
            best_key = key;
            best_action = Some(discard);
        }
    }

    Ok(best_action.map(|action| SurvivalChoice {
        action,
        score: best_score,
        reason: format!(
            "survival_guard deficit={} hands_left={} discards_left={}",
            deficit, sim.run.state.hands_left, sim.run.state.discards_left
        ),
    }))
}

fn select_action_mcts<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_candidates: &[AutoAction],
    min_sims_override: Option<u32>,
    preferred_root: Option<&AutoAction>,
) -> Result<(AutoAction, StepSearchStats), AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let started_at = Instant::now();
    let root_sim = replay_history(factory, history)?;
    let (root_terminal, root_legal) = evaluate_node_state(&root_sim, cfg, targets, step);
    if root_terminal {
        return Err(AutoplayError::InvalidAction("root is terminal".to_string()));
    }

    let mut candidates = if root_candidates.is_empty() {
        root_legal
    } else {
        root_candidates.to_vec()
    };
    candidates = dedup_actions(candidates);
    let root_actions = candidates;
    if root_actions.is_empty() {
        return Err(AutoplayError::InvalidAction(
            "no valid root action".to_string(),
        ));
    }

    let mut nodes = vec![Node::new_root(root_actions.clone(), false)];
    let mut rng = SimpleRng::new(cfg.seed ^ (step as u64).wrapping_mul(0x9E3779B9));
    let mut simulations = 0u32;
    let mut min_sims = cfg
        .min_simulations_per_step
        .min(cfg.per_step_max_simulations);
    if let Some(override_value) = min_sims_override {
        min_sims = min_sims
            .max(override_value)
            .min(cfg.per_step_max_simulations);
    }
    let preferred_root_key = preferred_root.map(|action| action.stable_key());

    'simulation: while simulations < cfg.per_step_max_simulations {
        if cfg.per_step_time_ms > 0
            && simulations >= min_sims
            && started_at.elapsed().as_millis() as u64 >= cfg.per_step_time_ms
        {
            break;
        }

        let mut sim = match replay_history(factory, history) {
            Ok(value) => value,
            Err(err) if is_recoverable_action_error(&err) => {
                simulations = simulations.saturating_add(1);
                continue;
            }
            Err(err) => return Err(err),
        };
        let mut path: Vec<AutoAction> = Vec::new();
        let mut node_idx = 0usize;

        loop {
            if nodes[node_idx].terminal {
                break;
            }

            let depth_step = step.saturating_add(path.len() as u32);
            let (state_terminal, legal_now) = evaluate_node_state(&sim, cfg, targets, depth_step);
            if state_terminal {
                nodes[node_idx].terminal = true;
                nodes[node_idx].unexpanded.clear();
                nodes[node_idx].children.clear();
                break;
            }
            reconcile_node_with_legal(&mut nodes, node_idx, legal_now);
            if nodes[node_idx].terminal {
                break;
            }

            let should_expand = should_expand_node(&nodes[node_idx]);
            if should_expand {
                let pick = pick_unexpanded_index(&nodes[node_idx].unexpanded, &mut rng);
                let action = nodes[node_idx].unexpanded.remove(pick);
                match sim.apply_action(&action) {
                    Ok(_) => {
                        path.push(action.clone());
                        let depth_after = step.saturating_add(path.len() as u32);
                        let (child_terminal, child_legal) =
                            evaluate_node_state(&sim, cfg, targets, depth_after);
                        let child_idx = nodes.len();
                        let mut prior = action_prior_weight(&action, &sim, weights, depth_after);
                        if node_idx == 0
                            && preferred_root_key
                                .as_ref()
                                .is_some_and(|key| action.stable_key() == *key)
                        {
                            prior = (prior * 1.25).min(4.0);
                        }
                        nodes.push(Node::new_child(
                            node_idx,
                            action,
                            child_legal,
                            child_terminal,
                            nodes[node_idx].depth + 1,
                            prior,
                        ));
                        nodes[node_idx].children.push(child_idx);
                        node_idx = child_idx;
                        break;
                    }
                    Err(err) if is_recoverable_action_error(&err) => {
                        if nodes[node_idx].unexpanded.is_empty()
                            && nodes[node_idx].children.is_empty()
                        {
                            nodes[node_idx].terminal = true;
                        }
                        continue 'simulation;
                    }
                    Err(err) => return Err(err),
                }
            }

            if !nodes[node_idx].children.is_empty() {
                let parent_visits = nodes[node_idx].visits.max(1) as f64;
                let mut best = nodes[node_idx].children[0];
                let mut best_score = f64::NEG_INFINITY;
                let mut best_key = String::new();
                for child_idx in nodes[node_idx].children.iter().copied() {
                    let child = &nodes[child_idx];
                    let mean = if child.visits == 0 {
                        0.0
                    } else {
                        child.value_sum / child.visits as f64
                    };
                    let explore = cfg.exploration_c
                        * child.prior
                        * (((parent_visits + 1.0).ln() + 1.0).sqrt()
                            / (1.0 + child.visits as f64).sqrt());
                    let score = mean + explore;
                    let key = child
                        .action
                        .as_ref()
                        .map(|item| item.stable_key())
                        .unwrap_or_default();
                    if score > best_score || (score == best_score && key < best_key) {
                        best_score = score;
                        best_key = key;
                        best = child_idx;
                    }
                }
                node_idx = best;
                if let Some(action) = nodes[node_idx].action.as_ref() {
                    match sim.apply_action(action) {
                        Ok(_) => path.push(action.clone()),
                        Err(err) if is_recoverable_action_error(&err) => {
                            let parent_idx = nodes[node_idx].parent.unwrap_or(0);
                            nodes[parent_idx]
                                .children
                                .retain(|value| *value != node_idx);
                            continue 'simulation;
                        }
                        Err(err) => return Err(err),
                    }
                }
                continue;
            }

            nodes[node_idx].terminal = true;
            break;
        }

        let reward = rollout(
            &mut sim,
            step + path.len() as u32,
            cfg,
            targets,
            weights,
            &mut rng,
        )?;

        let mut walk = Some(node_idx);
        while let Some(idx) = walk {
            nodes[idx].visits = nodes[idx].visits.saturating_add(1);
            nodes[idx].value_sum += reward;
            walk = nodes[idx].parent;
        }
        simulations = simulations.saturating_add(1);
    }

    let mut best_action = None;
    let mut best_visits = 0u32;
    let mut best_value = f64::NEG_INFINITY;
    let mut best_prior = f64::NEG_INFINITY;
    let mut best_key = String::new();
    for child_idx in nodes[0].children.iter().copied() {
        let child = &nodes[child_idx];
        let value = if child.visits == 0 {
            f64::NEG_INFINITY
        } else {
            child.value_sum / child.visits as f64
        };
        let key = child
            .action
            .as_ref()
            .map(|item| item.stable_key())
            .unwrap_or_default();
        if child.visits > best_visits
            || (child.visits == best_visits && value > best_value)
            || (child.visits == best_visits && value == best_value && child.prior > best_prior)
            || (child.visits == best_visits
                && value == best_value
                && child.prior == best_prior
                && key < best_key)
        {
            best_visits = child.visits;
            best_value = value;
            best_prior = child.prior;
            best_key = key;
            best_action = child.action.clone();
        }
    }

    let selected = best_action.unwrap_or_else(|| {
        let mut fallback = root_actions;
        fallback.sort_by_key(|item| item.stable_key());
        fallback.into_iter().next().unwrap_or(AutoAction::Deal)
    });
    Ok((
        selected,
        StepSearchStats {
            simulations,
            elapsed_ms: started_at.elapsed().as_millis() as u64,
            root_children: nodes[0].children.len(),
            selected_visits: best_visits,
            selected_value: if best_value.is_finite() {
                best_value
            } else {
                0.0
            },
            decision_source: "mcts".to_string(),
            tactical_trigger: None,
            tactical_candidate: None,
            tactical_bypassed_mcts: false,
            forced_min_sims: min_sims_override.unwrap_or(0),
            skip_blind_penalty: 0.0,
        },
    ))
}

fn should_expand_node(node: &Node) -> bool {
    if node.unexpanded.is_empty() {
        return false;
    }
    if node.children.is_empty() {
        return true;
    }
    let visits = node.visits.max(1) as f64;
    let max_children = if node.depth == 0 {
        4.0 + 2.0 * visits.powf(0.45)
    } else if node.depth <= 2 {
        2.2 + 1.4 * visits.powf(0.50)
    } else {
        1.5 * visits.powf(0.55) + 1.0
    };
    (node.children.len() as f64) < max_children
}

fn pick_unexpanded_index(actions: &[AutoAction], rng: &mut SimpleRng) -> usize {
    if actions.len() <= 1 {
        return 0;
    }
    let mut ranked: Vec<(i32, String, usize)> = actions
        .iter()
        .enumerate()
        .map(|(idx, action)| (action_expand_priority(action), action.stable_key(), idx))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let top = ranked.len().min(6);
    let min_priority = ranked
        .iter()
        .take(top)
        .map(|item| item.0)
        .min()
        .unwrap_or(0);
    let mut total = 0.0f64;
    let mut weights = Vec::with_capacity(top);
    for item in ranked.iter().take(top) {
        let weight = ((item.0 - min_priority + 1).max(1) as f64).powf(1.35);
        weights.push(weight);
        total += weight;
    }
    if total <= 0.0 {
        return ranked[0].2;
    }
    let mut pick = rng.gen_unit_f64() * total;
    for (idx, weight) in weights.into_iter().enumerate() {
        if pick <= weight {
            return ranked[idx].2;
        }
        pick -= weight;
    }
    ranked[0].2
}

fn action_expand_priority(action: &AutoAction) -> i32 {
    match action {
        AutoAction::NextBlind => 120,
        AutoAction::EnterShop => 110,
        AutoAction::Play { .. } => 100,
        AutoAction::BuyPack { .. } => 96,
        AutoAction::BuyCard { .. } => 92,
        AutoAction::BuyVoucher { .. } => 90,
        AutoAction::UseConsumable { .. } => 88,
        AutoAction::Discard { .. } => 86,
        AutoAction::Deal => 82,
        AutoAction::RerollShop => 60,
        AutoAction::PickPack { .. } => 50,
        AutoAction::SkipPack => 38,
        AutoAction::SellJoker { .. } => 30,
        AutoAction::LeaveShop => 20,
        AutoAction::SkipBlind => 10,
    }
}

fn dedup_actions(mut actions: Vec<AutoAction>) -> Vec<AutoAction> {
    actions.sort_by_key(|action| action.stable_key());
    actions.dedup_by_key(|action| action.stable_key());
    actions
}

fn materialize<F>(
    factory: &F,
    history: &[AutoAction],
    path: &[AutoAction],
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    step: u32,
) -> Result<(Simulator, bool, Vec<AutoAction>), AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let mut sim = replay_history(factory, history)?;
    for action in path {
        sim.apply_action(action)?;
    }
    let (terminal, legal) =
        evaluate_node_state(&sim, cfg, targets, step.saturating_add(path.len() as u32));
    Ok((sim, terminal, legal))
}

fn replay_history<F>(factory: &F, history: &[AutoAction]) -> Result<Simulator, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let mut sim = factory()?;
    for action in history {
        sim.apply_action(action)?;
    }
    Ok(sim)
}

fn evaluate_node_state(
    sim: &Simulator,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    step: u32,
) -> (bool, Vec<AutoAction>) {
    let metrics = sim.metrics();
    if target_reached(metrics, targets)
        || (targets.stop_on_blind_failed && metrics.blind_failed)
        || step >= cfg.max_steps
    {
        return (true, Vec::new());
    }
    let legal = dedup_actions(sim.legal_actions(cfg));
    (legal.is_empty(), legal)
}

fn reconcile_node_with_legal(nodes: &mut [Node], node_idx: usize, legal: Vec<AutoAction>) {
    let legal_keys: HashSet<String> = legal.iter().map(|action| action.stable_key()).collect();
    let existing_children = nodes[node_idx].children.clone();
    nodes[node_idx].children.clear();
    for child_idx in existing_children {
        let keep = match nodes[child_idx].action.as_ref() {
            Some(action) => legal_keys.contains(&action.stable_key()),
            None => false,
        };
        if keep {
            nodes[node_idx].children.push(child_idx);
        }
    }
    nodes[node_idx]
        .unexpanded
        .retain(|action| legal_keys.contains(&action.stable_key()));
    if nodes[node_idx].children.is_empty() && nodes[node_idx].unexpanded.is_empty() {
        nodes[node_idx].unexpanded = legal;
    }
    nodes[node_idx].terminal =
        nodes[node_idx].children.is_empty() && nodes[node_idx].unexpanded.is_empty();
}

fn rollout(
    sim: &mut Simulator,
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    rng: &mut SimpleRng,
) -> Result<f64, AutoplayError> {
    let mut depth = 0u32;
    let depth_limit = dynamic_rollout_depth(sim, cfg);
    loop {
        let metrics = sim.metrics();
        if target_reached(metrics, targets)
            || (targets.stop_on_blind_failed && metrics.blind_failed)
            || step.saturating_add(depth) >= cfg.max_steps
        {
            return Ok(weighted_score(metrics, weights, step + depth));
        }

        let mut legal = sim.legal_actions(cfg);
        if legal.is_empty() {
            return Ok(weighted_score(metrics, weights, step + depth));
        }
        legal.sort_by_key(|action| action.stable_key());
        legal.dedup_by_key(|action| action.stable_key());

        let mut applied = false;
        while !legal.is_empty() {
            let action = select_rollout_action(&legal, sim, rng, cfg);
            match sim.apply_action(&action) {
                Ok(_) => {
                    depth = depth.saturating_add(1);
                    applied = true;
                    break;
                }
                Err(err) if is_recoverable_action_error(&err) => {
                    let key = action.stable_key();
                    legal.retain(|item| item.stable_key() != key);
                }
                Err(err) => return Err(err),
            }
        }
        if !applied {
            let metrics = sim.metrics();
            return Ok(weighted_score(metrics, weights, step + depth));
        }
        if depth >= depth_limit {
            let metrics = sim.metrics();
            return Ok(weighted_score(metrics, weights, step + depth));
        }
    }
}

fn dynamic_rollout_depth(sim: &Simulator, cfg: &AutoplayConfig) -> u32 {
    let phase_bonus = match sim.run.state.phase {
        rulatro_core::Phase::Play => 8,
        rulatro_core::Phase::Deal => 4,
        rulatro_core::Phase::Shop => 2,
        _ => 0,
    };
    let resource_bonus = (sim.run.state.hands_left as u32).min(4);
    cfg.rollout_depth
        .saturating_add(phase_bonus)
        .saturating_add(resource_bonus)
}

fn select_rollout_action(
    actions: &[AutoAction],
    sim: &Simulator,
    rng: &mut SimpleRng,
    cfg: &AutoplayConfig,
) -> AutoAction {
    let mut scored: Vec<(f64, String, AutoAction)> = actions
        .iter()
        .cloned()
        .map(|action| {
            let key = action.stable_key();
            let score = rollout_action_score(&action, sim, cfg);
            (score, key, action)
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let limit = scored.len().min(cfg.rollout_top_k.max(1));
    if limit == 1 {
        return scored[0].2.clone();
    }

    let min_score = scored[..limit]
        .iter()
        .map(|item| item.0)
        .fold(f64::INFINITY, f64::min);
    let mut total_weight = 0.0f64;
    let mut weights = Vec::with_capacity(limit);
    for idx in 0..limit {
        let weight = (scored[idx].0 - min_score + 0.15).max(0.01);
        total_weight += weight;
        weights.push(weight);
    }
    if total_weight <= 0.0 {
        return scored[0].2.clone();
    }

    let mut pick = rng.gen_unit_f64() * total_weight;
    for (idx, weight) in weights.into_iter().enumerate() {
        if pick <= weight {
            return scored[idx].2.clone();
        }
        pick -= weight;
    }
    scored[0].2.clone()
}

fn rollout_action_score(action: &AutoAction, sim: &Simulator, cfg: &AutoplayConfig) -> f64 {
    let deficit = (sim.run.state.target - sim.run.state.blind_score).max(0) as f64;
    let blind_cleared = sim.run.blind_outcome() == Some(rulatro_core::BlindOutcome::Cleared);
    let money = sim.run.state.money as f64;
    let free_jokers = sim
        .run
        .inventory
        .joker_capacity()
        .saturating_sub(sim.run.inventory.jokers.len()) as f64;
    let free_consumables = sim
        .run
        .inventory
        .consumable_slots
        .saturating_sub(sim.run.inventory.consumable_count()) as f64;

    match action {
        AutoAction::Play { indices } => {
            let estimate = estimate_play_strength(sim, indices);
            let mut score = 80.0 + estimate + deficit * 0.01 + indices.len() as f64;
            if sim.run.state.hands_left <= 1 && sim.run.state.discards_left > 0 {
                let projected = estimate * 1.25;
                if projected < deficit {
                    score -= ((deficit - projected) * 0.5).min(120.0);
                }
            }
            score
        }
        AutoAction::Discard { indices } => {
            let desperation_bonus = if sim.run.state.hands_left <= 1 {
                (deficit * cfg.desperation_discard_boost).min(180.0)
            } else {
                0.0
            };
            42.0 + sim.run.state.discards_left as f64 * 4.0 - indices.len() as f64
                + desperation_bonus
        }
        AutoAction::Deal => 72.0 + sim.run.state.hands_left as f64 * 1.2,
        AutoAction::SkipBlind => {
            let pressure = if sim.run.state.target > 0 {
                (deficit / sim.run.state.target as f64).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let risk_penalty = if blind_cleared {
                0.0
            } else {
                (cfg.skip_blind_deficit_penalty * 0.02 * (1.0 - 0.55 * pressure)).min(180.0)
            };
            6.0 - risk_penalty
        }
        AutoAction::EnterShop => {
            if blind_cleared {
                125.0
            } else {
                12.0
            }
        }
        AutoAction::LeaveShop => 18.0,
        AutoAction::RerollShop => 58.0 + (money * 0.03).min(10.0),
        AutoAction::BuyCard { .. } => 90.0 + free_jokers * 4.0 + free_consumables * 2.0,
        AutoAction::BuyPack { .. } => 95.0 + free_jokers * 3.0 + free_consumables * 3.0,
        AutoAction::BuyVoucher { .. } => 88.0 + (money * 0.02).min(8.0),
        AutoAction::PickPack { indices } => 55.0 + indices.len() as f64 * 5.0,
        AutoAction::SkipPack => 28.0,
        AutoAction::UseConsumable { selected, .. } => 77.0 + selected.len() as f64 * 2.0,
        AutoAction::SellJoker { .. } => 22.0 + if money <= 1.0 { 8.0 } else { 0.0 },
        AutoAction::NextBlind => {
            if blind_cleared {
                135.0
            } else {
                8.0
            }
        }
    }
}

fn estimate_play_strength(sim: &Simulator, indices: &[usize]) -> f64 {
    let mut total = 0.0f64;
    for idx in indices {
        let Some(card) = sim.run.hand.get(*idx) else {
            continue;
        };
        if card.is_stone() {
            continue;
        }
        total += sim.run.tables.rank_chips(card.rank) as f64;
        total += card.bonus_chips as f64;
        total += match card.rank {
            rulatro_core::Rank::Ace => 14.0,
            rulatro_core::Rank::King => 13.0,
            rulatro_core::Rank::Queen => 12.0,
            rulatro_core::Rank::Jack => 11.0,
            rulatro_core::Rank::Ten => 10.0,
            rulatro_core::Rank::Nine => 9.0,
            rulatro_core::Rank::Eight => 8.0,
            rulatro_core::Rank::Seven => 7.0,
            rulatro_core::Rank::Six => 6.0,
            rulatro_core::Rank::Five => 5.0,
            rulatro_core::Rank::Four => 4.0,
            rulatro_core::Rank::Three => 3.0,
            rulatro_core::Rank::Two => 2.0,
            rulatro_core::Rank::Joker => 15.0,
        };
    }
    total
}

fn action_prior_weight(
    action: &AutoAction,
    sim: &Simulator,
    weights: ObjectiveWeights,
    depth_step: u32,
) -> f64 {
    let score = weighted_score(sim.metrics(), weights, depth_step);
    let normalized = 1.0 + (score / 18.0).tanh();
    let action_bias = match action {
        AutoAction::NextBlind => 1.35,
        AutoAction::EnterShop => 1.20,
        AutoAction::BuyPack { .. } => 1.15,
        AutoAction::BuyCard { .. } => 1.10,
        AutoAction::BuyVoucher { .. } => 1.08,
        AutoAction::Play { .. } => 1.00,
        AutoAction::UseConsumable { .. } => 0.96,
        AutoAction::Discard { .. } => 0.90,
        AutoAction::Deal => 0.92,
        AutoAction::RerollShop => 0.82,
        AutoAction::PickPack { .. } => 0.70,
        AutoAction::SkipPack => 0.55,
        AutoAction::SellJoker { .. } => 0.45,
        AutoAction::LeaveShop => 0.40,
        AutoAction::SkipBlind => 0.20,
    };
    (normalized * action_bias).clamp(0.05, 4.0)
}

fn is_recoverable_action_error(err: &AutoplayError) -> bool {
    matches!(err, AutoplayError::Run(_) | AutoplayError::InvalidAction(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_blind_penalty_is_larger_when_deficit_worsens() {
        let cfg = AutoplayConfig::default();
        let before = EvalMetrics {
            ante: 1,
            money: 0,
            blind_score: 224,
            blind_target: 450,
            blind_failed: false,
            blind_cleared: false,
        };
        let worse = EvalMetrics {
            ante: 1,
            money: 0,
            blind_score: 0,
            blind_target: 600,
            blind_failed: false,
            blind_cleared: false,
        };
        let better = EvalMetrics {
            ante: 1,
            money: 0,
            blind_score: 595,
            blind_target: 600,
            blind_failed: false,
            blind_cleared: false,
        };
        let strong = skip_blind_deficit_penalty(before, worse, &cfg);
        let light = skip_blind_deficit_penalty(before, better, &cfg);
        assert!(strong > light);
        assert!(strong <= cfg.skip_blind_deficit_penalty);
        assert!(strong >= cfg.skip_blind_deficit_penalty * 0.35);
    }

    #[test]
    fn tactical_bypass_respects_share_cap() {
        assert!(tactical_bypass_allowed(1, 8, 0.45));
        assert!(!tactical_bypass_allowed(4, 8, 0.45));
        assert!(!tactical_bypass_allowed(0, 0, 0.0));
    }

    #[test]
    fn endgame_scoring_prefers_clear_over_fail() {
        let targets = TargetConfig::default();
        let weights = ObjectiveWeights::default();
        let clear = EvalMetrics {
            ante: 1,
            money: 0,
            blind_score: 500,
            blind_target: 450,
            blind_failed: false,
            blind_cleared: true,
        };
        let fail = EvalMetrics {
            ante: 1,
            money: 0,
            blind_score: 320,
            blind_target: 450,
            blind_failed: true,
            blind_cleared: false,
        };
        assert!(
            endgame_action_score(clear, targets, weights, 10)
                > endgame_action_score(fail, targets, weights, 10)
        );
    }
}
