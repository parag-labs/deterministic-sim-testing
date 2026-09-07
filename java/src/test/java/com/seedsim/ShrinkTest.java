package com.seedsim;

import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.IntStream;
import org.junit.jupiter.api.Test;

class ShrinkTest {

    private static Iterable<Integer> range(int n) {
        List<Integer> out = new ArrayList<>();
        IntStream.range(0, n).forEach(out::add);
        return out;
    }

    @Test
    void searchFindsADivergingSchedule() {
        ReplicatedFlag.Found found = ReplicatedFlag.searchForDivergence(range(500));
        assertNotNull(found, "expected some seed in 0..499 to diverge");
        assertTrue(ReplicatedFlag.runScenario(found.seed(), FaultSchedule.of(found.faults())).diverged());
    }

    @Test
    void ddminStripsTheIrrelevantFaults() {
        ReplicatedFlag.Found found = ReplicatedFlag.searchForDivergence(range(500));
        assertNotNull(found);

        // Pad the real reproducer with two faults that fire long after everything is
        // over - they can't possibly matter.
        List<Fault> noise = List.of(new Crash("r1", 900, 950), new Crash("r2", 800, 850));
        List<Fault> padded = new ArrayList<>(found.faults());
        padded.addAll(noise);

        var stillFails = (java.util.function.Predicate<List<Fault>>)
            subset -> ReplicatedFlag.runScenario(found.seed(), FaultSchedule.of(subset)).diverged();

        List<Fault> minimal = Shrink.ddmin(padded, stillFails);

        assertTrue(stillFails.test(minimal));
        assertTrue(minimal.size() < padded.size());
        // The late-firing noise must be gone from the minimal reproducer.
        for (Fault n : noise) assertFalse(minimal.contains(n));
    }

    @Test
    void ddminRequiresAFailingInput() {
        assertThrows(IllegalArgumentException.class,
            () -> Shrink.ddmin(List.of(1, 2, 3), subset -> false));
    }
}
