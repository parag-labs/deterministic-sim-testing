// The simulator core: a single-threaded discrete-event loop over virtual time.
//
// Everything that could introduce nondeterminism in a real distributed system -
// the clock, message latency, message loss, the order concurrent events fire in -
// is funnelled through this one loop and the seeded Rng. There are no real threads
// and no wall-clock reads, so a given (seed, workload, fault schedule) always
// produces the exact same event trace. That's the property the tests lean on.

import { FaultSchedule } from "./faults.js";
import { Network } from "./network.js";
import { Rng } from "./rng.js";

/** A single message field value: either a string or a number. */
export type MsgValue = string | number;

/** A message is a map of string keys to values. */
export type Msg = Record<string, MsgValue>;

/** One ordered, comparable line of the event trace. */
export interface TraceEntry {
  time: number;
  event: string;
  /** Fields sorted by key and rendered `key=value`, joined with commas. */
  fields: string;
}

interface HeapItem {
  time: number;
  seq: number;
  fn: () => void;
}

function itemLess(a: HeapItem, b: HeapItem): boolean {
  if (a.time !== b.time) {
    return a.time < b.time;
  }
  return a.seq < b.seq;
}

// A binary min-heap keyed on (time, seq). Ties break on insertion order, which
// keeps concurrent events deterministic.
class EventQueue {
  private items: HeapItem[] = [];

  get size(): number {
    return this.items.length;
  }

  peek(): HeapItem {
    return this.items[0];
  }

  push(item: HeapItem): void {
    const items = this.items;
    items.push(item);
    let i = items.length - 1;
    while (i > 0) {
      const parent = (i - 1) >> 1;
      if (itemLess(items[i], items[parent])) {
        [items[i], items[parent]] = [items[parent], items[i]];
        i = parent;
      } else {
        break;
      }
    }
  }

  pop(): HeapItem {
    const items = this.items;
    const top = items[0];
    const last = items.pop() as HeapItem;
    if (items.length > 0) {
      items[0] = last;
      let i = 0;
      const n = items.length;
      for (;;) {
        const l = 2 * i + 1;
        const r = 2 * i + 2;
        let smallest = i;
        if (l < n && itemLess(items[l], items[smallest])) {
          smallest = l;
        }
        if (r < n && itemLess(items[r], items[smallest])) {
          smallest = r;
        }
        if (smallest === i) {
          break;
        }
        [items[i], items[smallest]] = [items[smallest], items[i]];
        i = smallest;
      }
    }
    return top;
  }
}

/**
 * Base class for a participant. Subclasses react to messages and timers; they
 * never touch time or the network directly except through these helpers.
 */
export abstract class Node {
  sim: Simulator | null = null;

  constructor(readonly id: string) {}

  /** Hands a message to the network from this node. */
  send(dst: string, msg: Msg): void {
    if (this.sim === null) {
      throw new Error("node is not attached to a simulator");
    }
    this.sim.net.send(this.id, dst, msg);
  }

  /**
   * Schedules a named timer to fire `delay` ticks from now. A crashed node's
   * timers are silently swallowed when they would fire.
   */
  timer(delay: number, name: string, payload: unknown = null): void {
    const sim = this.sim;
    if (sim === null) {
      throw new Error("node is not attached to a simulator");
    }
    sim.at(delay, () => {
      if (sim.faults.isCrashed(this.id, sim.now)) {
        return;
      }
      sim.record("timer", { node: this.id, name });
      this.onTimer(name, payload);
    });
  }

  /** Handles an incoming message. The default is a no-op. */
  onMessage(_src: string, _msg: Msg): void {}

  /** Handles a fired timer. The default is a no-op. */
  onTimer(_name: string, _payload: unknown): void {}
}

/** A deterministic discrete-event loop over integer virtual time. */
export class Simulator {
  seed: number;
  rng: Rng;
  now = 0;
  nodes: Record<string, Node> = {};
  trace: TraceEntry[] = [];
  faults: FaultSchedule;
  net: Network;
  private heap = new EventQueue();
  private seq = 0;

  /** Creates a simulator for the given seed and fault schedule. */
  constructor(seed: number, faults: FaultSchedule | null = null) {
    this.seed = seed;
    this.rng = new Rng(seed);
    this.faults = faults ?? new FaultSchedule();
    this.net = new Network(this);
  }

  /** Registers a node with the simulator and returns it. */
  add(node: Node): Node {
    node.sim = this;
    this.nodes[node.id] = node;
    return node;
  }

  /**
   * Schedules `fn` to run `delay` ticks from now. Ties break on insertion order,
   * keeping concurrent events deterministic. Throws if `delay < 0`.
   */
  at(delay: number, fn: () => void): void {
    if (delay < 0) {
      throw new Error("delay must be >= 0");
    }
    this.heap.push({ time: this.now + delay, seq: this.seq, fn });
    this.seq += 1;
  }

  /**
   * Appends a fully ordered, comparable line to the trace. Fields are sorted by
   * key so two runs can be compared for exact equality.
   */
  record(event: string, fields: Record<string, MsgValue | null | undefined>): void {
    const rendered = Object.keys(fields)
      .sort()
      .map((k) => `${k}=${renderValue(fields[k])}`)
      .join(",");
    this.trace.push({ time: this.now, event, fields: rendered });
  }

  /**
   * Drains the event loop until it empties, until virtual time would pass
   * `until` (`null` for no limit), or until `maxSteps` events have fired.
   */
  run(until: number | null = null, maxSteps = 1_000_000): TraceEntry[] {
    let steps = 0;
    while (this.heap.size > 0 && steps < maxSteps) {
      const nxt = this.heap.peek();
      if (until !== null && nxt.time > until) {
        break;
      }
      this.heap.pop();
      this.now = nxt.time;
      nxt.fn();
      steps += 1;
    }
    return this.trace;
  }
}

function renderValue(v: MsgValue | null | undefined): string {
  if (v === null || v === undefined) {
    return "None";
  }
  return String(v);
}
