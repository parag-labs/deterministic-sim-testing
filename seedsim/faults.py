"""Fault schedules: the injected adversity a run is subjected to.

A fault is a plain, hashable value with a time window. That matters for two
reasons: the schedule is part of the reproducer (seed + faults fully determine a
run), and because faults are hashable and comparable they can be fed straight
into the shrinker, which needs to add/remove them as opaque items.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable


@dataclass(frozen=True)
class Partition:
    """The link between two nodes is down for [start, end)."""

    a: str
    b: str
    start: int
    end: int


@dataclass(frozen=True)
class Crash:
    """`node` is down for [start, end): it sends nothing and receives nothing."""

    node: str
    start: int
    end: int


class FaultSchedule:
    def __init__(
        self,
        partitions: Iterable[Partition] = (),
        crashes: Iterable[Crash] = (),
    ) -> None:
        self.partitions = list(partitions)
        self.crashes = list(crashes)

    def is_partitioned(self, a: str, b: str, t: int) -> bool:
        pair = {a, b}
        for p in self.partitions:
            if p.start <= t < p.end and pair == {p.a, p.b}:
                return True
        return False

    def is_crashed(self, node: str, t: int) -> bool:
        for c in self.crashes:
            if c.node == node and c.start <= t < c.end:
                return True
        return False

    def items(self) -> list:
        return [*self.partitions, *self.crashes]

    @classmethod
    def of(cls, items: Iterable) -> "FaultSchedule":
        parts = [i for i in items if isinstance(i, Partition)]
        crs = [i for i in items if isinstance(i, Crash)]
        return cls(parts, crs)
