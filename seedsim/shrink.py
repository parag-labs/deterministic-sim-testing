"""Trace shrinking via delta debugging (ddmin).

Finding a seed that breaks an invariant is only half the job; a 40-fault
schedule that triggers a bug is nearly useless to a human. ddmin repeatedly
throws away subsets of the faults and keeps whichever smaller schedule still
fails, converging on a minimal reproducer - usually the one or two faults that
actually matter. This is the classic Zeller/Hildebrandt algorithm, kept small.
"""

from __future__ import annotations

from typing import Callable, Sequence, TypeVar

T = TypeVar("T")


def _split(xs: Sequence[T], n: int) -> list[list[T]]:
    k, r = divmod(len(xs), n)
    out: list[list[T]] = []
    i = 0
    for idx in range(n):
        size = k + (1 if idx < r else 0)
        if size:
            out.append(list(xs[i : i + size]))
        i += size
    return out


def ddmin(items: Sequence[T], still_fails: Callable[[list[T]], bool]) -> list[T]:
    """Return a minimal sublist of `items` for which `still_fails` holds.

    `still_fails(items)` must be True to begin with - you only shrink a known
    failure.
    """
    cur = list(items)
    if not still_fails(cur):
        raise ValueError("ddmin requires the full input to already fail")

    n = 2
    while len(cur) >= 2:
        chunks = _split(cur, n)
        reduced = False
        for i in range(len(chunks)):
            complement = [x for j, ch in enumerate(chunks) if j != i for x in ch]
            if complement and still_fails(complement):
                cur = complement
                n = max(n - 1, 2)
                reduced = True
                break
        if not reduced:
            if n >= len(cur):
                break
            n = min(len(cur), 2 * n)
    return cur
