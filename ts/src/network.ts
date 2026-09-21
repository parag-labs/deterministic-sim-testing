// The message-passing layer, where latency and loss are decided.
//
// Delay and (optional) random drop are drawn from the simulator's seeded Rng, and
// partitions/crashes are consulted from the fault schedule - both at send time and
// again at delivery time, because a node can crash while a message is in flight.
// Keeping this in one place is what lets the whole run stay deterministic.

import type { Msg, Simulator } from "./sim.js";

/** The simulated message layer for a Simulator. */
export class Network {
  minDelay = 1;
  maxDelay = 10;
  dropProb = 0.0;

  constructor(private sim: Simulator) {}

  /** Routes a message from `src` to `dst`, applying latency, loss, and faults. */
  send(src: string, dst: string, msg: Msg): void {
    const sim = this.sim;
    const now = sim.now;
    const kind = msg["kind"];

    // Blocked before it ever leaves: partition, or either end down.
    if (
      sim.faults.isPartitioned(src, dst, now) ||
      sim.faults.isCrashed(src, now) ||
      sim.faults.isCrashed(dst, now)
    ) {
      sim.record("drop", { src, dst, kind });
      return;
    }

    // Only consume an rng draw for loss when loss is actually configured, so
    // enabling dropProb doesn't silently shift the delay stream.
    if (this.dropProb > 0.0 && sim.rng.chance(this.dropProb)) {
      sim.record("drop", { src, dst, kind });
      return;
    }

    const delay = sim.rng.between(this.minDelay, this.maxDelay);
    sim.at(delay, () => {
      // Re-check at arrival: the link may have partitioned or the destination
      // may have crashed while the message was in flight.
      if (
        sim.faults.isPartitioned(src, dst, sim.now) ||
        sim.faults.isCrashed(dst, sim.now)
      ) {
        sim.record("drop", { src, dst, kind });
        return;
      }
      sim.record("deliver", { src, dst, kind });
      sim.nodes[dst].onMessage(src, msg);
    });
  }
}
