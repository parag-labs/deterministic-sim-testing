package com.seedsim;

import static org.junit.jupiter.api.Assertions.*;

import com.seedsim.Simulator.TraceEntry;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;

class DeterminismTest {

    @Test
    void sameSeedSameTrace() {
        FaultSchedule faults = new FaultSchedule(List.of(new Partition("coord", "r2", 0, 50)), List.of());
        List<TraceEntry> t1 = ReplicatedFlag.runScenario(12345, faults).trace();
        List<TraceEntry> t2 = ReplicatedFlag.runScenario(12345, faults).trace();
        assertEquals(t1, t2, "a run must be a pure function of (seed, faults)");
    }

    @Test
    void differentSeedsCanDivergeInTrace() {
        // With random network latency, distinct seeds should not all collapse to the
        // same trace - otherwise the seed isn't actually driving anything.
        Set<List<TraceEntry>> traces = new HashSet<>();
        for (int s = 0; s < 20; s++) {
            traces.add(ReplicatedFlag.runScenario(s, new FaultSchedule()).trace());
        }
        assertTrue(traces.size() >= 2);
    }

    @Test
    void isolatingAReplicaDiverges() {
        FaultSchedule faults = new FaultSchedule(List.of(new Partition("coord", "r2", 0, 50)), List.of());
        ReplicatedFlag.Result result = ReplicatedFlag.runScenario(1, faults);
        assertTrue(result.diverged());
        assertNotEquals(result.values().get("r1"), result.values().get("r2"));
    }

    @Test
    void noFaultsStaysConsistent() {
        ReplicatedFlag.Result result = ReplicatedFlag.runScenario(1, new FaultSchedule());
        assertFalse(result.diverged());
        assertEquals(Map.of("r1", 1, "r2", 1), result.values());
    }

    @Test
    void crashingTheCoordinatorDoesNotDiverge() {
        // Both sends are dropped, so the replicas agree (both still 0) - a real but
        // different outcome from isolating a single replica.
        FaultSchedule faults = new FaultSchedule(List.of(), List.of(new Crash("coord", 0, 50)));
        ReplicatedFlag.Result result = ReplicatedFlag.runScenario(1, faults);
        assertFalse(result.diverged());
        assertEquals(Map.of("r1", 0, "r2", 0), result.values());
    }
}
