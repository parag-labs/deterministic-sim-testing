"""The message-passing layer, where latency and loss are decided.

Delay and (optional) random drop are drawn from the simulator's seeded Rng, and
partitions/crashes are consulted from the fault schedule - both at *send* time
and again at *delivery* time, because a node can crash while a message is in
flight. Keeping this in one place is what lets the whole run stay deterministic.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from .sim import Simulator


class Network:
    def __init__(
        self,
        sim: "Simulator",
        min_delay: int = 1,
        max_delay: int = 10,
        drop_prob: float = 0.0,
    ) -> None:
        self.sim = sim
        self.min_delay = min_delay
        self.max_delay = max_delay
        self.drop_prob = drop_prob

    def send(self, src: str, dst: str, msg: dict) -> None:
        sim = self.sim
        now = sim.now
        kind = msg.get("kind")

        # Blocked before it ever leaves: partition, or either end down.
        if (
            sim.faults.is_partitioned(src, dst, now)
            or sim.faults.is_crashed(src, now)
            or sim.faults.is_crashed(dst, now)
        ):
            sim.record("drop", src=src, dst=dst, kind=kind)
            return

        # Only consume an rng draw for loss when loss is actually configured,
        # so enabling drop_prob doesn't silently shift the delay stream.
        if self.drop_prob > 0.0 and sim.rng.chance(self.drop_prob):
            sim.record("drop", src=src, dst=dst, kind=kind)
            return

        delay = sim.rng.between(self.min_delay, self.max_delay)

        def deliver() -> None:
            # Re-check at arrival: the link may have partitioned or the
            # destination may have crashed while the message was in flight.
            if sim.faults.is_partitioned(src, dst, sim.now) or sim.faults.is_crashed(dst, sim.now):
                sim.record("drop", src=src, dst=dst, kind=kind)
                return
            sim.record("deliver", src=src, dst=dst, kind=kind)
            sim.nodes[dst].on_message(src, msg)

        sim.at(delay, deliver)
