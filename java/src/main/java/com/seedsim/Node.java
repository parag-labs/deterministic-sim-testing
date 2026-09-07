// Base class for a participant. Subclasses react to messages and timers; they never
// touch time or the network directly except through these helpers.

package com.seedsim;

import java.util.Map;

public abstract class Node {

    public final String nodeId;
    Simulator sim;

    protected Node(String nodeId) {
        this.nodeId = nodeId;
    }

    public void send(String dst, Map<String, Object> msg) {
        sim.net.send(nodeId, dst, msg);
    }

    public void timer(int delay, String name, Object payload) {
        Simulator s = sim;
        s.at(delay, () -> {
            if (s.faults.isCrashed(nodeId, s.now())) return;
            s.record("timer", "node", nodeId, "name", name);
            onTimer(name, payload);
        });
    }

    // Override in subclasses.
    public void onMessage(String src, Map<String, Object> msg) {}

    public void onTimer(String name, Object payload) {}
}
