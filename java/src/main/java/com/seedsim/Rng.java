// A small, fully deterministic PRNG (splitmix64).
//
// The whole point of deterministic-sim-testing is that a run is a pure function of
// one 64-bit seed, so the randomness has to be reproducible and portable. splitmix64
// is tiny, well distributed, and identical here to the Python and C# ports.

package com.seedsim;

public final class Rng {

    private static final long GOLDEN = 0x9E3779B97F4A7C15L;
    private long state;

    public Rng(long seed) {
        this.state = seed;
    }

    public long nextU64() {
        state = state + GOLDEN;
        long z = state;
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }

    /** Uniform in [0, n). Rejection-sampled so it's unbiased. */
    public long below(long n) {
        if (n <= 0) throw new IllegalArgumentException("below(n) needs n > 0");
        // Largest multiple of n that fits in 64 bits (unsigned); reject above it.
        long limit = Long.divideUnsigned(-1L, n) * n;
        while (true) {
            long x = nextU64();
            if (Long.compareUnsigned(x, limit) < 0) {
                return Long.remainderUnsigned(x, n);
            }
        }
    }

    /** Uniform integer in [lo, hi] inclusive. */
    public long between(long lo, long hi) {
        if (hi < lo) throw new IllegalArgumentException("between(lo, hi) needs hi >= lo");
        return lo + below(hi - lo + 1);
    }

    public boolean chance(double p) {
        if (p <= 0.0) return false;
        if (p >= 1.0) return true;
        // Top 53 bits scaled into [0, 1) - deterministic and uniform.
        double d = (nextU64() >>> 11) * 0x1.0p-53;
        return d < p;
    }
}
