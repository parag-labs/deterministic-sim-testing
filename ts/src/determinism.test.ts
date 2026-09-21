import { describe, expect, it } from "vitest";

import { Crash, FaultSchedule, Partition, type Fault } from "./faults.js";
import { Rng } from "./rng.js";
import {
  randomFaults,
  runScenario,
  searchForDivergence,
} from "./replicated_flag.js";
import type { TraceEntry } from "./sim.js";

function traceKey(trace: TraceEntry[]): string {
  return trace.map((e) => `${e.time}:${e.event}:${e.fields}|`).join("");
}

function seedRange(n: number): number[] {
  return Array.from({ length: n }, (_, i) => i);
}

// --- The five determinism tests mirrored from the Python suite. ---

describe("determinism", () => {
  it("same seed produces the same trace", () => {
    const faults = () =>
      new FaultSchedule([new Partition("coord", "r2", 0, 50)]);
    const t1 = runScenario(12345, faults()).trace;
    const t2 = runScenario(12345, faults()).trace;
    expect(t1).toEqual(t2);
  });

  it("different seeds can diverge in trace", () => {
    const traces = new Set<string>();
    for (let s = 0; s < 20; s++) {
      traces.add(traceKey(runScenario(s, new FaultSchedule()).trace));
    }
    expect(traces.size).toBeGreaterThanOrEqual(2);
  });

  it("isolating a replica diverges", () => {
    const res = runScenario(
      1,
      new FaultSchedule([new Partition("coord", "r2", 0, 50)]),
    );
    expect(res.diverged).toBe(true);
    expect(res.values["r1"]).not.toBe(res.values["r2"]);
  });

  it("no faults stays consistent", () => {
    const res = runScenario(1, new FaultSchedule());
    expect(res.diverged).toBe(false);
    expect(res.values).toEqual({ r1: 1, r2: 1 });
  });

  it("crashing the coordinator does not diverge", () => {
    // Both sends are dropped, so the replicas agree (both still 0).
    const res = runScenario(
      1,
      new FaultSchedule([], [new Crash("coord", 0, 50)]),
    );
    expect(res.diverged).toBe(false);
    expect(res.values).toEqual({ r1: 0, r2: 0 });
  });
});

// --- Cross-language exactness checks against the Python reference stream. ---

describe("rng exactness", () => {
  it("nextU64 matches the reference stream", () => {
    const r = new Rng(12345);
    const want = [
      2454886589211414944n,
      3778200017661327597n,
      2205171434679333405n,
      3248800117070709450n,
      9350289611492784363n,
    ];
    for (const w of want) {
      expect(r.nextU64()).toBe(w);
    }
  });

  it("below matches the reference stream", () => {
    const r = new Rng(1);
    expect([r.below(5), r.below(5), r.below(5), r.below(5), r.below(5)]).toEqual([
      0, 4, 0, 0, 1,
    ]);
  });

  it("between matches the reference stream", () => {
    const r = new Rng(1);
    const got = [0, 0, 0, 0, 0].map(() => r.between(20, 200));
    expect(got).toEqual([84, 161, 197, 150, 86]);
  });

  it("chance matches the reference stream", () => {
    const r = new Rng(1);
    const got = new Array(8).fill(0).map(() => r.chance(0.6));
    expect(got).toEqual([true, false, false, true, true, false, false, true]);
  });

  it("below throws on non-positive n", () => {
    expect(() => new Rng(1).below(0)).toThrow();
    expect(() => new Rng(1).below(-1)).toThrow();
  });

  it("between throws when hi < lo", () => {
    expect(() => new Rng(1).between(5, 4)).toThrow();
  });

  it("between is inclusive on a single point", () => {
    const r = new Rng(99);
    for (let i = 0; i < 100; i++) {
      expect(r.between(7, 7)).toBe(7);
    }
  });

  it("chance boundaries consume no draw", () => {
    const r = new Rng(1);
    expect(r.chance(0)).toBe(false);
    expect(r.chance(1)).toBe(true);
    expect(r.chance(-3)).toBe(false);
    expect(r.chance(2)).toBe(true);
    // None of the above should have advanced the stream.
    expect(r.nextU64()).toBe(new Rng(1).nextU64());
  });
});

// --- Scenario and fault-generation exactness. ---

describe("scenario exactness", () => {
  it("scenario trace matches the reference", () => {
    const res = runScenario(
      12345,
      new FaultSchedule([new Partition("coord", "r2", 0, 50)]),
    );
    expect(res.trace).toEqual([
      { time: 0, event: "drop", fields: "dst=r2,kind=set,src=coord" },
      { time: 5, event: "deliver", fields: "dst=r1,kind=set,src=coord" },
    ]);
    expect(res.diverged).toBe(true);
    expect(res.values).toEqual({ r1: 1, r2: 0 });
  });

  it("no-fault scenario trace matches the reference", () => {
    const res = runScenario(1, new FaultSchedule());
    expect(res.trace).toEqual([
      { time: 6, event: "deliver", fields: "dst=r1,kind=set,src=coord" },
      { time: 10, event: "deliver", fields: "dst=r2,kind=set,src=coord" },
    ]);
  });

  it("randomFaults matches the reference", () => {
    const got = randomFaults(new Rng(1));
    const want: Fault[] = [
      new Partition("coord", "r2", 4, 201),
      new Partition("r1", "coord", 0, 191),
    ];
    expect(got).toEqual(want);
  });

  it("searchForDivergence matches the reference", () => {
    const found = searchForDivergence(seedRange(500));
    expect(found).not.toBeNull();
    expect(found?.seed).toBe(3);
    expect(found?.faults).toEqual([new Partition("coord", "r1", 1, 90)]);
  });
});
