"""A small, fully deterministic PRNG (splitmix64).

The whole point of seed-sim is that a run is a pure function of one 64-bit seed,
so the randomness has to be reproducible and portable - not `random.random()`,
whose stream can shift between interpreter versions. splitmix64 is tiny, well
distributed, and trivial to reimplement in another language later if the ports
ever need to agree on a stream.
"""

from __future__ import annotations

_MASK = (1 << 64) - 1
_GOLDEN = 0x9E3779B97F4A7C15


class Rng:
    __slots__ = ("_state",)

    def __init__(self, seed: int) -> None:
        self._state = seed & _MASK

    def next_u64(self) -> int:
        self._state = (self._state + _GOLDEN) & _MASK
        z = self._state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & _MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & _MASK
        return z ^ (z >> 31)

    def below(self, n: int) -> int:
        """Uniform in [0, n). Rejection-sampled so it's unbiased."""
        if n <= 0:
            raise ValueError("below(n) needs n > 0")
        # Largest multiple of n that fits in 64 bits; reject above it.
        limit = (_MASK // n) * n
        while True:
            x = self.next_u64()
            if x < limit:
                return x % n

    def between(self, lo: int, hi: int) -> int:
        """Uniform integer in [lo, hi] inclusive."""
        if hi < lo:
            raise ValueError("between(lo, hi) needs hi >= lo")
        return lo + self.below(hi - lo + 1)

    def chance(self, p: float) -> bool:
        if p <= 0.0:
            return False
        if p >= 1.0:
            return True
        return self.next_u64() / 2 ** 64 < p
