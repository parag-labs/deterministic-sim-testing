// The message-passing layer, where latency and loss are decided.
//
// Delay and (optional) random drop are drawn from the simulator's seeded Rng, and
// partitions/crashes are consulted from the fault schedule - both at send time and
// again at delivery time, because a node can crash while a message is in flight.

package com.seedsim;

import java.util.Map;

public final class Network {

    private final Simulator sim;
    public final int minDelay;
    public final int maxDelay;
    public final double dropProb;

    public Network(Simulator sim) {
        this(sim, 1, 10, 0.0);
    }

    public Network(Simulator sim, int minDelay, int maxDelay, double dropProb) {
        this.sim = sim;
        this.minDelay = minDelay;
        this.maxDelay = maxDelay;
        this.dropProb = dropProb;
    }

    public void send(String src, String dst, Map<String, Object> msg) {
        int now = sim.now();
        Object kind = msg.get("kind");

        // Blocked before it ever leaves: partition, or either end down.
        if (sim.faults.isPartitioned(src, dst, now)
                || sim.faults.isCrashed(src, now)
                || sim.faults.isCrashed(dst, now)) {
            sim.record("drop", "src", src, "dst", dst, "kind", kind);
            return;
        }

        // Only consume an rng draw for loss when loss is actually configured, so
        // enabling dropProb doesn't silently shift the delay stream.
        if (dropProb > 0.0 && sim.rng.chance(dropProb)) {
            sim.record("drop", "src", src, "dst", dst, "kind", kind);
            return;
        }

        int delay = (int) sim.rng.between(minDelay, maxDelay);

        sim.at(delay, () -> {
            // Re-check at arrival: the link may have partitioned or the destination
            // may have crashed while the message was in flight.
            if (sim.faults.isPartitioned(src, dst, sim.now()) || sim.faults.isCrashed(dst, sim.now())) {
                sim.record("drop", "src", src, "dst", dst, "kind", kind);
                return;
            }
            sim.record("deliver", "src", src, "dst", dst, "kind", kind);
            sim.nodes.get(dst).onMessage(src, msg);
        });
    }
}
