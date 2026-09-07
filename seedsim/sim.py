"""The simulator core: a single-threaded discrete-event loop over virtual time.

Everything that could introduce nondeterminism in a real distributed system -
the clock, message latency, message loss, the order concurrent events fire in -
is funnelled through this one loop and the seeded Rng. There are no real threads
and no wall-clock reads, so a given (seed, workload, fault schedule) always
produces the exact same event trace. That's the property the tests lean on.
"""

from __future__ import annotations

import heapq
from dataclasses import dataclass, field
from typing import Any, Callable, Optional

from .faults import FaultSchedule
from .network import Network
from .rng import Rng


@dataclass(order=True)
class _Item:
    time: int
    seq: int
    fn: Callable[[], None] = field(compare=False)


class Simulator:
    def __init__(self, seed: int, faults: Optional[FaultSchedule] = None) -> None:
        self.seed = seed
        self.rng = Rng(seed)
        self.now = 0
        self.nodes: dict[str, "Node"] = {}
        self.trace: list[tuple] = []
        self.faults = faults or FaultSchedule()
        self.net = Network(self)
        self._heap: list[_Item] = []
        self._seq = 0

    def add(self, node: "Node") -> "Node":
        node.sim = self
        self.nodes[node.node_id] = node
        return node

    def at(self, delay: int, fn: Callable[[], None]) -> None:
        """Schedule `fn` to run `delay` ticks from now. Ties break on insertion
        order (`seq`), which is what keeps concurrent events deterministic."""
        if delay < 0:
            raise ValueError("delay must be >= 0")
        heapq.heappush(self._heap, _Item(self.now + delay, self._seq, fn))
        self._seq += 1

    def record(self, event: str, **fields: Any) -> None:
        # A trace entry is fully ordered and hashable so two runs can be compared
        # for exact equality.
        self.trace.append((self.now, event, tuple(sorted(fields.items()))))

    def run(self, until: Optional[int] = None, max_steps: int = 1_000_000) -> list[tuple]:
        steps = 0
        while self._heap and steps < max_steps:
            nxt = self._heap[0]
            if until is not None and nxt.time > until:
                break
            heapq.heappop(self._heap)
            self.now = nxt.time
            nxt.fn()
            steps += 1
        return self.trace


class Node:
    """Base class for a participant. Subclasses react to messages and timers;
    they never touch time or the network directly except through these helpers."""

    def __init__(self, node_id: str) -> None:
        self.node_id = node_id
        self.sim: Optional[Simulator] = None

    def send(self, dst: str, msg: dict) -> None:
        assert self.sim is not None
        self.sim.net.send(self.node_id, dst, msg)

    def timer(self, delay: int, name: str, payload: Any = None) -> None:
        assert self.sim is not None
        sim = self.sim

        def fire() -> None:
            if sim.faults.is_crashed(self.node_id, sim.now):
                return
            sim.record("timer", node=self.node_id, name=name)
            self.on_timer(name, payload)

        sim.at(delay, fire)

    # Override in subclasses.
    def on_message(self, src: str, msg: dict) -> None:  # pragma: no cover - stub
        pass

    def on_timer(self, name: str, payload: Any) -> None:  # pragma: no cover - stub
        pass
