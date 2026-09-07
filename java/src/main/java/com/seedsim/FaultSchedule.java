package com.seedsim;

import java.util.ArrayList;
import java.util.List;

public final class FaultSchedule {

    public final List<Partition> partitions;
    public final List<Crash> crashes;

    public FaultSchedule() {
        this(new ArrayList<>(), new ArrayList<>());
    }

    public FaultSchedule(List<Partition> partitions, List<Crash> crashes) {
        this.partitions = new ArrayList<>(partitions);
        this.crashes = new ArrayList<>(crashes);
    }

    public boolean isPartitioned(String a, String b, int t) {
        for (Partition p : partitions) {
            if (p.start() <= t && t < p.end()
                    && ((p.a().equals(a) && p.b().equals(b)) || (p.a().equals(b) && p.b().equals(a)))) {
                return true;
            }
        }
        return false;
    }

    public boolean isCrashed(String node, int t) {
        for (Crash c : crashes) {
            if (c.node().equals(node) && c.start() <= t && t < c.end()) return true;
        }
        return false;
    }

    public List<Fault> items() {
        List<Fault> out = new ArrayList<>();
        out.addAll(partitions);
        out.addAll(crashes);
        return out;
    }

    public static FaultSchedule of(List<? extends Fault> items) {
        List<Partition> parts = new ArrayList<>();
        List<Crash> crs = new ArrayList<>();
        for (Fault f : items) {
            if (f instanceof Partition p) parts.add(p);
            else if (f instanceof Crash c) crs.add(c);
        }
        return new FaultSchedule(parts, crs);
    }
}
