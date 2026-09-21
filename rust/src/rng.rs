//! A small, fully deterministic PRNG (splitmix64).
//!
//! The whole point of deterministic-sim-testing is that a run is a pure function
//! of one 64-bit seed, so the randomness has to be reproducible and portable.
//! splitmix64 is tiny, well distributed, and identical here to the Python, C# and
//! Java ports.

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// A splitmix64 pseudo-random generator seeded from a single 64-bit value.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates an `Rng` seeded with `seed`. The seed is reinterpreted as an
    /// unsigned 64-bit integer, matching the reference ports.
    pub fn new(seed: i64) -> Self {
        Rng { state: seed as u64 }
    }

    /// Returns the next 64-bit value in the stream.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a uniform value in `[0, n)`. It is rejection-sampled so it is
    /// unbiased. Panics if `n <= 0`.
    pub fn below(&mut self, n: i64) -> i64 {
        assert!(n > 0, "below(n) needs n > 0");
        let un = n as u64;
        // Largest multiple of n that fits in 64 bits; reject above it.
        let limit = (u64::MAX / un) * un;
        loop {
            let x = self.next_u64();
            if x < limit {
                return (x % un) as i64;
            }
        }
    }

    /// Returns a uniform integer in `[lo, hi]` inclusive. Panics if `hi < lo`.
    pub fn between(&mut self, lo: i64, hi: i64) -> i64 {
        assert!(hi >= lo, "between(lo, hi) needs hi >= lo");
        lo + self.below(hi - lo + 1)
    }

    /// Returns true with probability `p`. `p <= 0` is always false and `p >= 1`
    /// is always true; neither consumes a draw.
    pub fn chance(&mut self, p: f64) -> bool {
        if p <= 0.0 {
            return false;
        }
        if p >= 1.0 {
            return true;
        }
        (self.next_u64() as f64) / 2.0_f64.powi(64) < p
    }
}
