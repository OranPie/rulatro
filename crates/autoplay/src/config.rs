#[derive(Debug, Clone)]
pub struct AutoplayConfig {
    pub seed: u64,
    pub max_steps: u32,
    pub per_step_time_ms: u64,
    pub per_step_max_simulations: u32,
    pub min_simulations_per_step: u32,
    pub exploration_c: f64,
    pub action_retry_limit: u32,
    pub max_play_candidates: usize,
    pub max_discard_candidates: usize,
    pub max_shop_candidates: usize,
    pub rollout_depth: u32,
    pub rollout_top_k: usize,
    pub tactical_finish_margin: i64,
    pub tactical_force_min_sims: u32,
    pub tactical_max_step_share: f64,
    pub skip_blind_deficit_penalty: f64,
    pub endgame_exact_lookahead: bool,
    pub desperation_discard_boost: f64,
}

impl Default for AutoplayConfig {
    fn default() -> Self {
        Self {
            seed: 0xC0FFEE,
            max_steps: 500,
            per_step_time_ms: 120,
            per_step_max_simulations: 800,
            min_simulations_per_step: 12,
            exploration_c: 1.414,
            action_retry_limit: 6,
            max_play_candidates: 24,
            max_discard_candidates: 16,
            max_shop_candidates: 16,
            rollout_depth: 24,
            rollout_top_k: 4,
            tactical_finish_margin: 180,
            tactical_force_min_sims: 6,
            tactical_max_step_share: 0.45,
            skip_blind_deficit_penalty: 6000.0,
            endgame_exact_lookahead: true,
            desperation_discard_boost: 0.45,
        }
    }
}
