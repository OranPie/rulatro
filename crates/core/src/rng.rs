use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};

#[derive(Debug, Clone)]
pub struct RngState {
    seed: u64,
    rng: StdRng,
}

impl RngState {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        items.shuffle(&mut self.rng);
    }
}
