// A deliberately buggy workload, used to show deterministic-sim-testing finding
// and shrinking a bug.
//
// A coordinator sets a flag on two replicas with a single fire-and-forget message
// each - and, crucially, never retries. That missing retry is the bug: if the
// message to one replica is lost, the replicas diverge forever and nobody notices.

import { Crash, Fault, FaultSchedule, Partition } from "./faults.js";
import { Rng } from "./rng.js";
import { Msg, Node, Simulator, TraceEntry } from "./sim.js";

/** The participants in the worked example. */
export const NODES: readonly string[] = ["coord", "r1", "r2"];

/** The virtual time the scenario runs until. */
export const HORIZON = 1000;

/** Sets a flag on each replica exactly once and never retries. */
export class Coordinator extends Node {
  constructor(private replicas: string[]) {
    super("coord");
  }

  /**
   * Fires the coordinator's one-shot writes. The bug in one line: send once,
   * never confirm, never retry.
   */
  start(): void {
    for (const r of this.replicas) {
      this.send(r, { kind: "set", value: 1 });
    }
  }
}

/** Stores a single flag value updated by "set" messages. */
export class Replica extends Node {
  value = 0;

  constructor(id: string) {
    super(id);
  }

  onMessage(_src: string, msg: Msg): void {
    if (msg["kind"] === "set") {
      this.value = msg["value"] as number;
    }
  }
}

/** The outcome of a single scenario run. */
export interface ScenarioResult {
  trace: TraceEntry[];
  diverged: boolean;
  values: Record<string, number>;
}

/** A seed together with the fault schedule that made it diverge. */
export interface Found {
  seed: number;
  faults: Fault[];
}

/**
 * Runs the workload once and reports the trace, whether the replicas diverged,
 * and their final values.
 */
export function runScenario(
  seed: number,
  faults: FaultSchedule | null = null,
): ScenarioResult {
  const sim = new Simulator(seed, faults);
  const r1 = new Replica("r1");
  sim.add(r1);
  const r2 = new Replica("r2");
  sim.add(r2);
  const coord = new Coordinator([r1.id, r2.id]);
  sim.add(coord);
  sim.at(0, () => coord.start());
  sim.run(HORIZON, 1_000_000);

  const values: Record<string, number> = { r1: r1.value, r2: r2.value };
  return { trace: sim.trace, diverged: r1.value !== r2.value, values };
}

/**
 * Generates a small random fault schedule from a seeded Rng. Windows are biased
 * to start early because all the interesting activity happens in the first few
 * ticks.
 */
export function randomFaults(rng: Rng, nodes: readonly string[] = NODES): Fault[] {
  const out: Fault[] = [];
  const count = rng.between(1, 4);
  for (let i = 0; i < count; i++) {
    const start = rng.below(5);
    const end = start + rng.between(20, 200);
    if (rng.chance(0.6)) {
      const a = nodes[rng.below(nodes.length)];
      const b = nodes[rng.below(nodes.length)];
      if (a !== b) {
        out.push(new Partition(a, b, start, end));
      }
    } else {
      const node = nodes[rng.below(nodes.length)];
      out.push(new Crash(node, start, end));
    }
  }
  return out;
}

/** Returns the first (seed, faults) whose run diverges, or `null`. */
export function searchForDivergence(seeds: Iterable<number>): Found | null {
  for (const seed of seeds) {
    const faults = randomFaults(new Rng(seed));
    if (runScenario(seed, FaultSchedule.of(faults)).diverged) {
      return { seed, faults };
    }
  }
  return null;
}
