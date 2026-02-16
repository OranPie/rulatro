use crate::{
    target_reached, weighted_score, AutoAction, AutoplayConfig, AutoplayError, AutoplayResult,
    FinalMetrics, ObjectiveWeights, RunStatus, Simulator, StepRecord, StepSearchStats,
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

    fn gen_index(&mut self, len: usize) -> usize {
        if len <= 1 {
            0
        } else {
            (self.next_u64() as usize) % len
        }
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
            let tactical_choice = if should_try_tactical_finish(&sim, &request.config) {
                select_tactical_action(
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

            let (action, search_stats) = if let Some((choice, score)) = tactical_choice {
                (
                    choice,
                    StepSearchStats {
                        simulations: 0,
                        elapsed_ms: step_started.elapsed().as_millis() as u64,
                        root_children: root_candidates.len(),
                        selected_visits: 0,
                        selected_value: score,
                    },
                )
            } else {
                match select_action_mcts(
                    factory,
                    &history,
                    step,
                    &request.config,
                    request.targets,
                    request.weights,
                    &root_candidates,
                ) {
                    Ok(value) => value,
                    Err(err) if is_recoverable_action_error(&err) => {
                        status = Some(RunStatus::NoLegalAction);
                        break;
                    }
                    Err(err) => return Err(err),
                }
            };
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

fn should_try_tactical_finish(sim: &Simulator, cfg: &AutoplayConfig) -> bool {
    if sim.run.state.target <= 0 {
        return false;
    }
    let phase = sim.run.state.phase;
    if phase != rulatro_core::Phase::Play && phase != rulatro_core::Phase::Deal {
        return false;
    }
    let deficit = (sim.run.state.target - sim.run.state.blind_score).max(0);
    deficit <= cfg.tactical_finish_margin || sim.run.state.hands_left <= 1
}

fn select_tactical_action<F>(
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
    let candidates = dedup_actions(root_actions.to_vec());
    let mut best_action = None;
    let mut best_score = f64::NEG_INFINITY;
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
        }
    }

    Ok(best_action.map(|action| (action, best_score)))
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

fn select_action_mcts<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
    root_candidates: &[AutoAction],
) -> Result<(AutoAction, StepSearchStats), AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let started_at = Instant::now();
    let (_root_sim, root_terminal, root_legal) =
        materialize(factory, history, &[], cfg, targets, step)?;
    if root_terminal {
        return Err(AutoplayError::InvalidAction("root is terminal".to_string()));
    }

    let candidates = if root_candidates.is_empty() {
        root_legal
    } else {
        root_candidates.to_vec()
    };
    let root_actions = validate_actions(factory, history, &[], cfg, targets, step, &candidates)?;
    if root_actions.is_empty() {
        return Err(AutoplayError::InvalidAction(
            "no valid root action".to_string(),
        ));
    }

    let mut nodes = vec![Node::new_root(root_actions.clone(), false)];
    let mut rng = SimpleRng::new(cfg.seed ^ (step as u64).wrapping_mul(0x9E3779B9));
    let mut simulations = 0u32;
    let min_sims = cfg
        .min_simulations_per_step
        .min(cfg.per_step_max_simulations);

    while simulations < cfg.per_step_max_simulations {
        if cfg.per_step_time_ms > 0
            && simulations >= min_sims
            && started_at.elapsed().as_millis() as u64 >= cfg.per_step_time_ms
        {
            break;
        }

        let mut path: Vec<AutoAction> = Vec::new();
        let mut node_idx = 0usize;
        let mut leaf_sim: Option<Simulator> = None;

        loop {
            if nodes[node_idx].terminal {
                match materialize(factory, history, &path, cfg, targets, step) {
                    Ok((sim, _, _)) => leaf_sim = Some(sim),
                    Err(err) if is_recoverable_action_error(&err) => {}
                    Err(err) => return Err(err),
                }
                break;
            }

            let should_expand = should_expand_node(&nodes[node_idx]);
            if should_expand {
                let pick = pick_unexpanded_index(&nodes[node_idx].unexpanded, &mut rng);
                let action = nodes[node_idx].unexpanded.remove(pick);
                path.push(action.clone());
                match materialize(factory, history, &path, cfg, targets, step) {
                    Ok((sim, terminal, legal)) => {
                        let valid_legal = dedup_actions(legal);
                        let child_idx = nodes.len();
                        let prior =
                            action_prior_weight(&action, &sim, weights, step + path.len() as u32);
                        nodes.push(Node::new_child(
                            node_idx,
                            action,
                            valid_legal.clone(),
                            terminal || valid_legal.is_empty(),
                            nodes[node_idx].depth + 1,
                            prior,
                        ));
                        nodes[node_idx].children.push(child_idx);
                        node_idx = child_idx;
                        leaf_sim = Some(sim);
                        break;
                    }
                    Err(err) if is_recoverable_action_error(&err) => {
                        path.pop();
                        if nodes[node_idx].unexpanded.is_empty()
                            && nodes[node_idx].children.is_empty()
                        {
                            nodes[node_idx].terminal = true;
                        }
                        continue;
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
                        * (parent_visits.sqrt() / (1.0 + child.visits as f64));
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
                    path.push(action.clone());
                }
                continue;
            }

            if nodes[node_idx].children.is_empty() {
                match materialize(factory, history, &path, cfg, targets, step) {
                    Ok((sim, terminal, legal)) => {
                        let valid_legal = dedup_actions(legal);
                        nodes[node_idx].unexpanded = valid_legal.clone();
                        nodes[node_idx].terminal = terminal || valid_legal.is_empty();
                        if nodes[node_idx].terminal {
                            leaf_sim = Some(sim);
                            break;
                        }
                        continue;
                    }
                    Err(err) if is_recoverable_action_error(&err) => {
                        nodes[node_idx].terminal = true;
                        break;
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        let Some(mut sim) = leaf_sim else {
            simulations = simulations.saturating_add(1);
            continue;
        };
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
    let max_children = 1.5 * visits.powf(0.55) + 1.0;
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
    let top = ranked.len().min(3);
    let pick = rng.gen_index(top);
    ranked[pick].2
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
        AutoAction::Deal => 82,
        AutoAction::Discard { .. } => 72,
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

fn validate_actions<F>(
    factory: &F,
    history: &[AutoAction],
    path: &[AutoAction],
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    step: u32,
    actions: &[AutoAction],
) -> Result<Vec<AutoAction>, AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let mut unique = actions.to_vec();
    unique.sort_by_key(|action| action.stable_key());
    unique.dedup_by_key(|action| action.stable_key());

    let mut valid = Vec::new();
    for action in unique {
        let mut candidate = path.to_vec();
        candidate.push(action.clone());
        match materialize(factory, history, &candidate, cfg, targets, step) {
            Ok(_) => valid.push(action),
            Err(err) if is_recoverable_action_error(&err) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(valid)
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
    let mut sim = factory()?;
    for action in history {
        sim.apply_action(action)?;
    }
    for action in path {
        sim.apply_action(action)?;
    }
    let metrics = sim.metrics();
    let done = target_reached(metrics, targets)
        || (targets.stop_on_blind_failed && metrics.blind_failed)
        || step.saturating_add(path.len() as u32) >= cfg.max_steps;
    if done {
        return Ok((sim, true, Vec::new()));
    }
    let legal = sim.legal_actions(cfg);
    let terminal = legal.is_empty();
    Ok((sim, terminal, legal))
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
            let action = select_rollout_action(&legal, sim, rng, cfg.rollout_top_k);
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
    top_k: usize,
) -> AutoAction {
    let mut scored: Vec<(f64, String, AutoAction)> = actions
        .iter()
        .cloned()
        .map(|action| {
            let key = action.stable_key();
            let score = rollout_action_score(&action, sim);
            (score, key, action)
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let limit = scored.len().min(top_k.max(1));
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

fn rollout_action_score(action: &AutoAction, sim: &Simulator) -> f64 {
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
            80.0 + estimate_play_strength(sim, indices) + deficit * 0.01 + indices.len() as f64
        }
        AutoAction::Discard { indices } => {
            42.0 + sim.run.state.discards_left as f64 * 4.0 - indices.len() as f64
        }
        AutoAction::Deal => 72.0 + sim.run.state.hands_left as f64 * 1.2,
        AutoAction::SkipBlind => 6.0,
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
        AutoAction::Deal => 0.92,
        AutoAction::RerollShop => 0.82,
        AutoAction::Discard { .. } => 0.75,
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
