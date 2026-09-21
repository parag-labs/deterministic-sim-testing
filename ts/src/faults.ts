// Fault schedules: the injected adversity a run is subjected to.
//
// A fault is a plain, comparable value with a time window. The schedule is part
// of the reproducer (seed + faults fully determine a run), and because faults are
// value types they can be fed straight into the shrinker as opaque items.

/** The link between two nodes is down for `[start, end)`. */
export class Partition {
  readonly tag = "partition" as const;
  constructor(
    readonly a: string,
    readonly b: string,
    readonly start: number,
    readonly end: number,
  ) {}
}

/** `node` is down for `[start, end)`: it sends nothing and receives nothing. */
export class Crash {
  readonly tag = "crash" as const;
  constructor(
    readonly node: string,
    readonly start: number,
    readonly end: number,
  ) {}
}

/** A fault a run can be subjected to (a partition or a crash). */
export type Fault = Partition | Crash;

/** The full set of partitions and crashes applied to a run. */
export class FaultSchedule {
  constructor(
    public partitions: Partition[] = [],
    public crashes: Crash[] = [],
  ) {}

  /** Reports whether the (unordered) link between `a` and `b` is down at `t`. */
  isPartitioned(a: string, b: string, t: number): boolean {
    return this.partitions.some(
      (p) =>
        p.start <= t &&
        t < p.end &&
        ((p.a === a && p.b === b) || (p.a === b && p.b === a)),
    );
  }

  /** Reports whether `node` is down at `t`. */
  isCrashed(node: string, t: number): boolean {
    return this.crashes.some((c) => c.node === node && c.start <= t && t < c.end);
  }

  /** Returns every fault in the schedule (partitions first, then crashes). */
  items(): Fault[] {
    return [...this.partitions, ...this.crashes];
  }

  /** Partitions a flat list of faults back into a schedule. */
  static of(items: Fault[]): FaultSchedule {
    const partitions: Partition[] = [];
    const crashes: Crash[] = [];
    for (const item of items) {
      if (item.tag === "partition") {
        partitions.push(item);
      } else {
        crashes.push(item);
      }
    }
    return new FaultSchedule(partitions, crashes);
  }
}
