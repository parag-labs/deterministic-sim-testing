// A small, fully deterministic PRNG (splitmix64).
//
// The whole point of deterministic-sim-testing is that a run is a pure function of
// one 64-bit seed, so the randomness has to be reproducible and portable. splitmix64
// is tiny, well distributed, and identical here to the Python and Java ports.

using System;

namespace SeedSim;

public sealed class Rng
{
    private const ulong Golden = 0x9E3779B97F4A7C15UL;
    private ulong _state;

    public Rng(long seed) => _state = unchecked((ulong)seed);

    public ulong NextU64()
    {
        unchecked
        {
            _state += Golden;
            var z = _state;
            z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9UL;
            z = (z ^ (z >> 27)) * 0x94D049BB133111EBUL;
            return z ^ (z >> 31);
        }
    }

    /// <summary>Uniform in [0, n). Rejection-sampled so it's unbiased.</summary>
    public long Below(long n)
    {
        if (n <= 0) throw new ArgumentException("Below(n) needs n > 0");
        var un = (ulong)n;
        // Largest multiple of n that fits in 64 bits; reject above it.
        var limit = ulong.MaxValue / un * un;
        while (true)
        {
            var x = NextU64();
            if (x < limit) return (long)(x % un);
        }
    }

    /// <summary>Uniform integer in [lo, hi] inclusive.</summary>
    public long Between(long lo, long hi)
    {
        if (hi < lo) throw new ArgumentException("Between(lo, hi) needs hi >= lo");
        return lo + Below(hi - lo + 1);
    }

    public bool Chance(double p)
    {
        if (p <= 0.0) return false;
        if (p >= 1.0) return true;
        return NextU64() / 18446744073709551616.0 < p;
    }
}
