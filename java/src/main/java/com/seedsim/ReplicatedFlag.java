// A deliberately buggy workload, used to show deterministic-sim-testing finding +
// shrinking a bug.
//
// A coordinator sets a flag on two replicas with a single fire-and-forget message
// each - and, crucially, never retries. That missing retry is the bug: if the message
// to one replica is lost, the replicas diverge forever and nobody notices.

package com.seedsim;

import com.seedsim.Simulator.TraceEntry;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public final class ReplicatedFlag {

    private ReplicatedFlag() {}

    public static final String[] NODES = {"coord", "r1", "r2"};
    public static final int HORIZON = 1000;

    static final class Coordinator extends Node {
        private final List<String> replicas;

        Coordinator(List<String> replicas) {
            super("coord");
            this.replicas = replicas;
        }

        void start() {
            // The bug in one line: send once, never confirm, never retry.
            for (String r : replicas) {
                Map<String, Object> msg = new LinkedHashMap<>();
                msg.put("kind", "set");
                msg.put("value", 1);
                send(r, msg);
            }
        }
    }

    static final class Replica extends Node {
        int value = 0;

        Replica(String nodeId) {
            super(nodeId);
        }

        @Override
        public void onMessage(String src, Map<String, Object> msg) {
            if ("set".equals(msg.get("kind"))) {
                value = (int) msg.get("value");
            }
        }
    }

    public record Result(List<TraceEntry> trace, boolean diverged, Map<String, Integer> values) {}

    public record Found(int seed, List<Fault> faults) {}

    /** Run the workload once. Returns (trace, diverged, replica values). */
    public static Result runScenario(int seed, FaultSchedule faults) {
        Simulator sim = new Simulator(seed, faults);
        Replica r1 = (Replica) sim.add(new Replica("r1"));
        Replica r2 = (Replica) sim.add(new Replica("r2"));
        List<String> ids = List.of(r1.nodeId, r2.nodeId);
        Coordinator coord = (Coordinator) sim.add(new Coordinator(ids));
        sim.at(0, coord::start);
        sim.run(HORIZON);

        Map<String, Integer> values = new LinkedHashMap<>();
        values.put("r1", r1.value);
        values.put("r2", r2.value);
        boolean diverged = r1.value != r2.value;
        return new Result(sim.trace, diverged, values);
    }

    /** Generate a small random fault schedule from a seed. */
    public static List<Fault> randomFaults(Rng rng, String[] nodes) {
        List<Fault> out = new ArrayList<>();
        long count = rng.between(1, 4);
        for (long i = 0; i < count; i++) {
            int start = (int) rng.below(5);
            int end = start + (int) rng.between(20, 200);
            if (rng.chance(0.6)) {
                String a = nodes[(int) rng.below(nodes.length)];
                String b = nodes[(int) rng.below(nodes.length)];
                if (!a.equals(b)) out.add(new Partition(a, b, start, end));
            } else {
                String n = nodes[(int) rng.below(nodes.length)];
                out.add(new Crash(n, start, end));
            }
        }
        return out;
    }

    public static List<Fault> randomFaults(Rng rng) {
        return randomFaults(rng, NODES);
    }

    /** Return the first (seed, faults) whose run diverges, or null. */
    public static Found searchForDivergence(Iterable<Integer> seeds) {
        for (int seed : seeds) {
            List<Fault> faults = randomFaults(new Rng(seed));
            if (runScenario(seed, FaultSchedule.of(faults)).diverged()) {
                return new Found(seed, faults);
            }
        }
        return null;
    }
}
