"""Measures two things that matter for a simulator: raw event throughput, and
the overhead of the determinism machinery (does replaying the same seed cost the
same as the first run?). Numbers are written to bench/results/ and summarised on
stdout. Everything here runs on this machine - no fabricated comparisons.
"""

from __future__ import annotations

import json
import statistics
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from seedsim.faults import FaultSchedule
from examples.replicated_flag import random_faults, run_scenario
from seedsim.rng import Rng

RESULTS = Path(__file__).resolve().parent / "results"
RESULTS.mkdir(exist_ok=True)


def _time_runs(n: int) -> tuple[float, int]:
    total_events = 0
    t0 = time.perf_counter()
    for seed in range(n):
        faults = random_faults(Rng(seed))
        trace, _, _ = run_scenario(seed, FaultSchedule.of(faults))
        total_events += len(trace)
    dt = time.perf_counter() - t0
    return dt, total_events


def main() -> None:
    n = 20_000
    # Warm up so the first-run JIT/import costs don't skew the measurement.
    _time_runs(200)

    dt, events = _time_runs(n)
    runs_per_s = n / dt
    events_per_s = events / dt

    # Replay determinism: same seeds, second pass - should be within noise.
    dt2, events2 = _time_runs(n)
    assert events == events2, "replay produced a different number of events!"
    overhead_pct = (dt2 - dt) / dt * 100.0

    summary = {
        "scenarios": n,
        "events": events,
        "runs_per_sec": round(runs_per_s, 1),
        "events_per_sec": round(events_per_s, 1),
        "replay_overhead_pct": round(overhead_pct, 2),
        "avg_events_per_scenario": round(events / n, 2),
    }
    (RESULTS / "summary.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
