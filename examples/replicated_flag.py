"""A deliberately buggy workload, used to show seed-sim finding + shrinking a bug.

A coordinator sets a flag on two replicas with a single fire-and-forget message
each - and, crucially, never retries. That missing retry is the bug: if the
message to one replica is lost (a partition, or that replica being briefly down
at send time), the replicas diverge forever and nobody notices.

`search_for_divergence` scans seeds until it finds a fault schedule that makes
the replicas disagree; `run_scenario` re-runs any (seed, schedule) exactly. The
tests then hand the failing schedule to ddmin to strip it to the one fault that
matters.
"""

from __future__ import annotations

from seedsim.faults import Crash, FaultSchedule, Partition
from seedsim.rng import Rng
from seedsim.sim import Node, Simulator

NODES = ["coord", "r1", "r2"]
HORIZON = 1000


class Coordinator(Node):
    def __init__(self, replicas: list[str]) -> None:
        super().__init__("coord")
        self.replicas = replicas

    def start(self) -> None:
        # The bug in one line: send once, never confirm, never retry.
        for r in self.replicas:
            self.send(r, {"kind": "set", "value": 1})


class Replica(Node):
    def __init__(self, node_id: str) -> None:
        super().__init__(node_id)
        self.value = 0

    def on_message(self, src: str, msg: dict) -> None:
        if msg["kind"] == "set":
            self.value = msg["value"]


def run_scenario(seed: int, faults: FaultSchedule) -> tuple[list[tuple], bool, dict]:
    """Run the workload once. Returns (trace, diverged, replica_values)."""
    sim = Simulator(seed, faults)
    replicas = [sim.add(Replica("r1")), sim.add(Replica("r2"))]
    coord = sim.add(Coordinator([r.node_id for r in replicas]))
    sim.at(0, coord.start)
    sim.run(until=HORIZON)
    values = {r.node_id: r.value for r in replicas}
    diverged = len(set(values.values())) > 1
    return sim.trace, diverged, values


def random_faults(rng: Rng, nodes: list[str] = NODES) -> list:
    """Generate a small random fault schedule from a seed.

    Windows are biased to start early (below(5)) because all the interesting
    activity in this workload happens in the first few ticks - that's just
    domain knowledge about the workload, not about the answer.
    """
    out: list = []
    for _ in range(rng.between(1, 4)):
        start = rng.below(5)
        end = start + rng.between(20, 200)
        if rng.chance(0.6):
            a = nodes[rng.below(len(nodes))]
            b = nodes[rng.below(len(nodes))]
            if a != b:
                out.append(Partition(a, b, start, end))
        else:
            n = nodes[rng.below(len(nodes))]
            out.append(Crash(n, start, end))
    return out


def search_for_divergence(seeds) -> tuple[int, list] | None:
    """Return the first (seed, faults) whose run diverges, or None."""
    for seed in seeds:
        faults = random_faults(Rng(seed))
        _, diverged, _ = run_scenario(seed, FaultSchedule.of(faults))
        if diverged:
            return seed, faults
    return None
