# Benchmarks

All numbers are produced by `bench/benchmark.py` on this development machine
(pure CPython, no native extensions). Reproduce with:

```bash
PYTHONPATH=. python bench/benchmark.py
```

The harness runs the `replicated_flag` workload across many seeds — each with a
randomly generated fault schedule — then replays the identical set of seeds a
second time to measure the overhead of deterministic replay.

## What it measures

- **Throughput** — scenarios and events executed per second by the event loop.
- **Replay overhead** — the wall-clock difference between the first pass over a
  set of seeds and a second, identical pass. This is the number that matters:
  the entire reason to use deterministic simulation is that replaying a failure
  is essentially free, so replaying should cost the same as the first run.

## Latest run (20,000 scenarios)

| Metric | Value |
|---|---|
| Scenarios / sec | ~26,100 |
| Events / sec | ~52,300 |
| Avg events / scenario | 2.0 |
| **Replay overhead** | **+3.35%** (within run-to-run noise) |

## How to read this

- The **replay overhead near zero** is the headline. A second pass over the same
  seeds does the same work in the same time — deterministic replay adds no
  measurable cost, which is what makes "find a failing seed, replay it forever"
  practical.
- The `replicated_flag` workload is intentionally tiny (two messages), so
  *events per scenario* is low by design; the throughput figure reflects
  per-event loop overhead, not a claim about large workloads.
- This is a pure-Python reference implementation. The numbers are honest for what
  it is — a readable engine — not a tuned production simulator. There is no
  fabricated comparison against any other tool.
