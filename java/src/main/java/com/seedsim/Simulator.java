// The simulator core: a single-threaded discrete-event loop over virtual time.
//
// Everything that could introduce nondeterminism in a real distributed system - the
// clock, message latency, message loss, the order concurrent events fire in - is
// funnelled through this one loop and the seeded Rng. There are no real threads and
// no wall-clock reads, so a given (seed, workload, fault schedule) always produces
// the exact same event trace. That's the property the tests lean on.

package com.seedsim;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.PriorityQueue;
import java.util.TreeMap;

public final class Simulator {

    /** One ordered, comparable line of the event trace. */
    public record TraceEntry(int time, String event, String fields) {}

    private record Item(int time, int seq, Runnable fn) {}

    public final int seed;
    public final Rng rng;
    public final FaultSchedule faults;
    public final Network net;
    public final Map<String, Node> nodes = new HashMap<>();
    public final List<TraceEntry> trace = new ArrayList<>();

    private int now = 0;
    // Ties break on insertion order (seq), which keeps concurrent events deterministic.
    private final PriorityQueue<Item> heap =
        new PriorityQueue<>(Comparator.comparingInt(Item::time).thenComparingInt(Item::seq));
    private int seq = 0;

    public Simulator(int seed) {
        this(seed, null);
    }

    public Simulator(int seed, FaultSchedule faults) {
        this.seed = seed;
        this.rng = new Rng(seed);
        this.faults = faults != null ? faults : new FaultSchedule();
        this.net = new Network(this);
    }

    public int now() { return now; }

    public Node add(Node node) {
        node.sim = this;
        nodes.put(node.nodeId, node);
        return node;
    }

    /** Schedule fn to run delay ticks from now. */
    public void at(int delay, Runnable fn) {
        if (delay < 0) throw new IllegalArgumentException("delay must be >= 0");
        heap.add(new Item(now + delay, seq, fn));
        seq++;
    }

    public void record(String event, Object... kv) {
        TreeMap<String, String> sorted = new TreeMap<>();
        for (int i = 0; i < kv.length; i += 2) {
            Object v = kv[i + 1];
            sorted.put((String) kv[i], v == null ? "None" : v.toString());
        }
        StringBuilder sb = new StringBuilder();
        for (Map.Entry<String, String> e : sorted.entrySet()) {
            if (sb.length() > 0) sb.append(',');
            sb.append(e.getKey()).append('=').append(e.getValue());
        }
        trace.add(new TraceEntry(now, event, sb.toString()));
    }

    public List<TraceEntry> run(Integer until, int maxSteps) {
        int steps = 0;
        while (!heap.isEmpty() && steps < maxSteps) {
            Item nxt = heap.peek();
            if (until != null && nxt.time() > until) break;
            heap.poll();
            now = nxt.time();
            nxt.fn().run();
            steps++;
        }
        return trace;
    }

    public List<TraceEntry> run(Integer until) {
        return run(until, 1_000_000);
    }
}
