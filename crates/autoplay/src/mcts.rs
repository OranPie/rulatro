use crate::{
    target_reached, weighted_score, AutoAction, AutoplayConfig, AutoplayError, AutoplayResult,
    FinalMetrics, ObjectiveWeights, RunStatus, Simulator, StepRecord, StepSearchStats,
    SummaryStats, TargetConfig,
};
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
        }
    }

    fn new_child(
        parent: usize,
        action: AutoAction,
        unexpanded: Vec<AutoAction>,
        terminal: bool,
        depth: u32,
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

        let legal = sim.legal_actions(&request.config);
        if legal.is_empty() {
            status = Some(RunStatus::NoLegalAction);
            break;
        }

        let (action, search_stats) = select_action_mcts(
            factory,
            &history,
            step,
            &request.config,
            request.targets,
            request.weights,
        )?;
        total_simulations = total_simulations.saturating_add(search_stats.simulations as u64);

        let phase_before = sim.phase_name();
        let blind_before = sim.blind_name();
        let event_count = sim.apply_action(&action)?;
        let after = sim.metrics();

        records.push(StepRecord {
            step,
            phase_before,
            blind_before,
            ante_before: before.ante,
            money_before: before.money,
            score_before: before.blind_score,
            action: action.clone(),
            mcts: search_stats,
            phase_after: sim.phase_name(),
            blind_after: sim.blind_name(),
            ante_after: after.ante,
            money_after: after.money,
            score_after: after.blind_score,
            outcome_after: sim.run.blind_outcome().map(|value| format!("{value:?}")),
            event_count,
        });
        history.push(action);

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
        steps: records,
        summary: SummaryStats {
            steps: history.len() as u32,
            total_simulations,
            wall_time_ms: started_at.elapsed().as_millis() as u64,
        },
    })
}

fn select_action_mcts<F>(
    factory: &F,
    history: &[AutoAction],
    step: u32,
    cfg: &AutoplayConfig,
    targets: TargetConfig,
    weights: ObjectiveWeights,
) -> Result<(AutoAction, StepSearchStats), AutoplayError>
where
    F: Fn() -> Result<Simulator, AutoplayError>,
{
    let started_at = Instant::now();
    let (root_sim, root_terminal, root_legal) =
        materialize(factory, history, &[], cfg, targets, step)?;
    let root_actions = if root_legal.is_empty() {
        root_sim.legal_actions(cfg)
    } else {
        root_legal
    };
    if root_actions.is_empty() {
        return Err(AutoplayError::InvalidAction(
            "no legal action at root".to_string(),
        ));
    }

    let mut nodes = vec![Node::new_root(root_actions, root_terminal)];
    let mut rng = SimpleRng::new(cfg.seed ^ (step as u64).wrapping_mul(0x9E3779B9));
    let mut simulations = 0u32;

    while simulations < cfg.per_step_max_simulations {
        if cfg.per_step_time_ms > 0
            && started_at.elapsed().as_millis() as u64 >= cfg.per_step_time_ms
        {
            break;
        }

        let mut path: Vec<AutoAction> = Vec::new();
        let mut node_idx = 0usize;
        let leaf_sim: Simulator;

        loop {
            if nodes[node_idx].terminal {
                let (sim, _, _) = materialize(factory, history, &path, cfg, targets, step)?;
                leaf_sim = sim;
                break;
            }
            if !nodes[node_idx].unexpanded.is_empty() {
                let pick = rng.gen_index(nodes[node_idx].unexpanded.len());
                let action = nodes[node_idx].unexpanded.remove(pick);
                path.push(action.clone());
                let (sim, terminal, legal) =
                    materialize(factory, history, &path, cfg, targets, step)?;
                let child_idx = nodes.len();
                nodes.push(Node::new_child(
                    node_idx,
                    action,
                    legal,
                    terminal,
                    nodes[node_idx].depth + 1,
                ));
                nodes[node_idx].children.push(child_idx);
                node_idx = child_idx;
                leaf_sim = sim;
                break;
            }
            if nodes[node_idx].children.is_empty() {
                let (sim, terminal, legal) =
                    materialize(factory, history, &path, cfg, targets, step)?;
                nodes[node_idx].terminal = terminal;
                if nodes[node_idx].unexpanded.is_empty() {
                    nodes[node_idx].unexpanded = legal;
                }
                leaf_sim = sim;
                break;
            }

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
                let explore =
                    cfg.exploration_c * ((parent_visits.ln()) / child.visits.max(1) as f64).sqrt();
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
        }

        let mut sim = leaf_sim;
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
            || (child.visits == best_visits && value == best_value && key < best_key)
        {
            best_visits = child.visits;
            best_value = value;
            best_key = key;
            best_action = child.action.clone();
        }
    }

    let selected = best_action.unwrap_or_else(|| {
        let mut fallback = nodes[0].unexpanded.clone();
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
    loop {
        let metrics = sim.metrics();
        if target_reached(metrics, targets)
            || (targets.stop_on_blind_failed && metrics.blind_failed)
            || step.saturating_add(depth) >= cfg.max_steps
        {
            return Ok(weighted_score(metrics, weights, step + depth));
        }

        let legal = sim.legal_actions(cfg);
        if legal.is_empty() {
            return Ok(weighted_score(metrics, weights, step + depth));
        }

        let action = select_rollout_action(&legal, rng);
        sim.apply_action(&action)?;
        depth = depth.saturating_add(1);
        if depth >= cfg.rollout_depth {
            let metrics = sim.metrics();
            return Ok(weighted_score(metrics, weights, step + depth));
        }
    }
}

fn select_rollout_action(actions: &[AutoAction], rng: &mut SimpleRng) -> AutoAction {
    let mut ordered = actions.to_vec();
    ordered.sort_by(|a, b| {
        let pa = rollout_priority(a);
        let pb = rollout_priority(b);
        pb.cmp(&pa)
            .then_with(|| a.stable_key().cmp(&b.stable_key()))
    });
    let limit = ordered.len().min(4);
    let pick = rng.gen_index(limit);
    ordered[pick].clone()
}

fn rollout_priority(action: &AutoAction) -> i32 {
    match action {
        AutoAction::NextBlind => 120,
        AutoAction::EnterShop => 110,
        AutoAction::BuyPack { .. } => 100,
        AutoAction::BuyCard { .. } => 90,
        AutoAction::BuyVoucher { .. } => 85,
        AutoAction::Play { .. } => 80,
        AutoAction::UseConsumable { .. } => 75,
        AutoAction::Deal => 70,
        AutoAction::RerollShop => 60,
        AutoAction::Discard { .. } => 50,
        AutoAction::PickPack { .. } => 40,
        AutoAction::SkipPack => 30,
        AutoAction::SellJoker { .. } => 20,
        AutoAction::LeaveShop => 10,
        AutoAction::SkipBlind => 5,
    }
}
