// A small, fully deterministic PRNG (splitmix64).
//
// The whole point of deterministic-sim-testing is that a run is a pure function
// of one 64-bit seed, so the randomness has to be reproducible and portable.
// splitmix64 is tiny, well distributed, and identical here to the Python, C# and
// Java ports. All 64-bit arithmetic is done with BigInt so it matches the other
// languages' unsigned wrapping exactly.

const MASK64 = (1n << 64n) - 1n;
const GOLDEN = 0x9e3779b97f4a7c15n;
const TWO_POW_64 = 2 ** 64;

/** A splitmix64 pseudo-random generator seeded from a single 64-bit value. */
export class Rng {
  private state: bigint;

  /**
   * Creates an `Rng` seeded with `seed`. The seed is reinterpreted as an
   * unsigned 64-bit integer, matching the reference ports.
   */
  constructor(seed: number | bigint) {
    this.state = BigInt(seed) & MASK64;
  }

  /** Returns the next 64-bit value in the stream. */
  nextU64(): bigint {
    this.state = (this.state + GOLDEN) & MASK64;
    let z = this.state;
    z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & MASK64;
    z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & MASK64;
    return z ^ (z >> 31n);
  }

  /**
   * Returns a uniform value in `[0, n)`. It is rejection-sampled so it is
   * unbiased. Throws if `n <= 0`.
   */
  below(n: number): number {
    if (n <= 0) {
      throw new Error("below(n) needs n > 0");
    }
    const un = BigInt(n);
    // Largest multiple of n that fits in 64 bits; reject above it.
    const limit = (MASK64 / un) * un;
    for (;;) {
      const x = this.nextU64();
      if (x < limit) {
        return Number(x % un);
      }
    }
  }

  /** Returns a uniform integer in `[lo, hi]` inclusive. Throws if `hi < lo`. */
  between(lo: number, hi: number): number {
    if (hi < lo) {
      throw new Error("between(lo, hi) needs hi >= lo");
    }
    return lo + this.below(hi - lo + 1);
  }

  /**
   * Returns true with probability `p`. `p <= 0` is always false and `p >= 1` is
   * always true; neither consumes a draw.
   */
  chance(p: number): boolean {
    if (p <= 0) {
      return false;
    }
    if (p >= 1) {
      return true;
    }
    return Number(this.nextU64()) / TWO_POW_64 < p;
  }
}
