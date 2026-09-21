import { describe, expect, it } from "vitest";

import { Crash, FaultSchedule, type Fault } from "./faults.js";
import { runScenario, searchForDivergence } from "./replicated_flag.js";
import { ddmin } from "./shrink.js";

function seedRange(n: number): number[] {
  return Array.from({ length: n }, (_, i) => i);
}

// --- The three shrink tests mirrored from the Python suite. ---

describe("shrink", () => {
  it("search finds a diverging schedule", () => {
    const found = searchForDivergence(seedRange(500));
    expect(found).not.toBeNull();
    expect(
      runScenario(found!.seed, FaultSchedule.of(found!.faults)).diverged,
    ).toBe(true);
  });

  it("ddmin strips the irrelevant faults", () => {
    const found = searchForDivergence(seedRange(500));
    expect(found).not.toBeNull();
    const seed = found!.seed;

    // Pad the real reproducer with two faults that fire long after everything is
    // over - they can't possibly matter.
    const noise: Fault[] = [
      new Crash("r1", 900, 950),
      new Crash("r2", 800, 850),
    ];
    const padded = [...found!.faults, ...noise];

    const stillFails = (subset: Fault[]): boolean =>
      runScenario(seed, FaultSchedule.of(subset)).diverged;

    const minimal = ddmin(padded, stillFails);

    expect(stillFails(minimal)).toBe(true);
    expect(minimal.length).toBeLessThan(padded.length);
    for (const n of noise) {
      expect(minimal).not.toContain(n);
    }
  });

  it("ddmin requires a failing input", () => {
    expect(() => ddmin([1, 2, 3], () => false)).toThrow();
  });
});

// --- Extra coverage of the shrinker on plain data. ---

describe("ddmin on plain data", () => {
  it("reduces to the single culprit", () => {
    const items = [1, 2, 3, 4, 5, 6, 7, 8];
    const stillFails = (xs: number[]): boolean => xs.includes(7);
    expect(ddmin(items, stillFails)).toEqual([7]);
  });

  it("keeps all when every item matters", () => {
    const items = [1, 2, 3, 4];
    const stillFails = (xs: number[]): boolean => xs.length === 4;
    expect(ddmin(items, stillFails)).toEqual(items);
  });

  it("handles a single failing element", () => {
    expect(ddmin([42], (xs: number[]) => xs.length === 1)).toEqual([42]);
  });
});
