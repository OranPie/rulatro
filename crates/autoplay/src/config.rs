#[derive(Debug, Clone)]
pub struct AutoplayConfig {
    pub seed: u64,
    pub max_steps: u32,
    pub per_step_time_ms: u64,
    pub per_step_max_simulations: u32,
    pub exploration_c: f64,
    pub max_play_candidates: usize,
    pub max_discard_candidates: usize,
    pub max_shop_candidates: usize,
    pub rollout_depth: u32,
}

impl Default for AutoplayConfig {
    fn default() -> Self {
        Self {
            seed: 0xC0FFEE,
            max_steps: 500,
            per_step_time_ms: 120,
            per_step_max_simulations: 800,
            exploration_c: 1.414,
            max_play_candidates: 24,
            max_discard_candidates: 16,
            max_shop_candidates: 16,
            rollout_depth: 24,
        }
    }
}
