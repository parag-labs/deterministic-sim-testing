// A small, fully deterministic PRNG (splitmix64).
//
// The whole point of deterministic-sim-testing is that a run is a pure function of
// one 64-bit seed, so the randomness has to be reproducible and portable. splitmix64
// is tiny, well distributed, and identical here to the Python, C# and Java ports.

package deterministicsimtesting

import "math"

const golden = 0x9E3779B97F4A7C15

// Rng is a splitmix64 pseudo-random generator seeded from a single 64-bit value.
type Rng struct {
	state uint64
}

// NewRng creates an Rng seeded with the given value. The seed is reinterpreted as
// an unsigned 64-bit integer, matching the reference ports.
func NewRng(seed int64) *Rng {
	return &Rng{state: uint64(seed)}
}

// NextU64 returns the next 64-bit value in the stream.
func (r *Rng) NextU64() uint64 {
	r.state += golden
	z := r.state
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
	z = (z ^ (z >> 27)) * 0x94D049BB133111EB
	return z ^ (z >> 31)
}

// Below returns a uniform value in [0, n). It is rejection-sampled so it is unbiased.
// It panics if n <= 0.
func (r *Rng) Below(n int64) int64 {
	if n <= 0 {
		panic("Below(n) needs n > 0")
	}
	un := uint64(n)
	// Largest multiple of n that fits in 64 bits; reject above it.
	limit := (math.MaxUint64 / un) * un
	for {
		x := r.NextU64()
		if x < limit {
			return int64(x % un)
		}
	}
}

// Between returns a uniform integer in [lo, hi] inclusive. It panics if hi < lo.
func (r *Rng) Between(lo, hi int64) int64 {
	if hi < lo {
		panic("Between(lo, hi) needs hi >= lo")
	}
	return lo + r.Below(hi-lo+1)
}

// Chance returns true with probability p. p <= 0 is always false and p >= 1 is
// always true; neither consumes a draw.
func (r *Rng) Chance(p float64) bool {
	if p <= 0.0 {
		return false
	}
	if p >= 1.0 {
		return true
	}
	return float64(r.NextU64())/18446744073709551616.0 < p
}
