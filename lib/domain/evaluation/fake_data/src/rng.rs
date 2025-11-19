//! Deterministic RNG helpers shared across generators, CLIs, and tests.
//!
//! See docs referenced in the crate-level module docs for layering guidance.

use once_cell::sync::Lazy;
use rand::{SeedableRng, rngs::StdRng};
use std::sync::Mutex;

/// Default seed (ASCII "DFPS") for deterministic fake data.
pub const DEFAULT_SEED: u64 = 0x4446_5053;

static GLOBAL_RNG: Lazy<Mutex<StdRng>> =
    Lazy::new(|| Mutex::new(StdRng::seed_from_u64(DEFAULT_SEED)));

/// Execute a closure with the shared deterministic RNG.
pub fn with_global_rng<T>(f: impl FnOnce(&mut StdRng) -> T) -> T {
    let mut guard = GLOBAL_RNG
        .lock()
        .expect("dfps_fake_data global RNG lock poisoned");
    f(&mut guard)
}

/// Deterministic RNG seeded from the given value.
pub fn rng_from_seed(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// Helper that hands out unique deterministic seeds (e.g., for batched generators).
#[derive(Debug, Clone)]
pub struct SeedSequence {
    state: u64,
    step: u64,
}

impl SeedSequence {
    /// Start a new sequence from the provided base seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed,
            step: 0x9E37_79B1_85EB_CA87,
        }
    }

    /// Borrow the next RNG in the sequence.
    pub fn next_rng(&mut self) -> StdRng {
        let seed = self.state;
        self.state = self.state.wrapping_add(self.step);
        StdRng::seed_from_u64(seed)
    }
}
