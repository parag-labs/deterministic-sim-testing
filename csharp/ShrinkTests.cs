using System;
using System.Collections.Generic;
using System.Linq;
using Xunit;

namespace SeedSim.Tests;

public class ShrinkTests
{
    [Fact]
    public void SearchFindsADivergingSchedule()
    {
        var found = ReplicatedFlag.SearchForDivergence(Enumerable.Range(0, 500));
        Assert.NotNull(found);
        var (seed, faults) = found!.Value;
        var result = ReplicatedFlag.RunScenario(seed, FaultSchedule.Of(faults));
        Assert.True(result.Diverged);
    }

    [Fact]
    public void DdminStripsTheIrrelevantFaults()
    {
        var found = ReplicatedFlag.SearchForDivergence(Enumerable.Range(0, 500));
        Assert.NotNull(found);
        var (seed, faults) = found!.Value;

        // Pad the real reproducer with two faults that fire long after everything is
        // over - they can't possibly matter.
        var noise = new List<Fault> { new Crash("r1", 900, 950), new Crash("r2", 800, 850) };
        var padded = faults.Concat(noise).ToList();

        bool StillFails(List<Fault> subset) => ReplicatedFlag.RunScenario(seed, FaultSchedule.Of(subset)).Diverged;

        var minimal = Shrink.Ddmin(padded, StillFails);

        Assert.True(StillFails(minimal));
        Assert.True(minimal.Count < padded.Count);
        // The late-firing noise must be gone from the minimal reproducer.
        foreach (var n in noise) Assert.DoesNotContain(n, minimal);
    }

    [Fact]
    public void DdminRequiresAFailingInput()
    {
        Assert.Throws<ArgumentException>(
            () => Shrink.Ddmin(new List<int> { 1, 2, 3 }, _ => false));
    }
}
