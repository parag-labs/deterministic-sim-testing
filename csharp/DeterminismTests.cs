using System.Collections.Generic;
using System.Linq;
using Xunit;

namespace SeedSim.Tests;

public class DeterminismTests
{
    [Fact]
    public void SameSeedSameTrace()
    {
        var faults = new FaultSchedule(new[] { new Partition("coord", "r2", 0, 50) });
        var t1 = ReplicatedFlag.RunScenario(12345, faults).Trace;
        var t2 = ReplicatedFlag.RunScenario(12345, faults).Trace;
        Assert.True(t1.SequenceEqual(t2), "a run must be a pure function of (seed, faults)");
    }

    [Fact]
    public void DifferentSeedsCanDivergeInTrace()
    {
        // With random network latency, distinct seeds should not all collapse to the
        // same trace - otherwise the seed isn't actually driving anything.
        var traces = new HashSet<string>();
        for (var s = 0; s < 20; s++)
        {
            var trace = ReplicatedFlag.RunScenario(s, new FaultSchedule()).Trace;
            traces.Add(string.Join("|", trace.Select(e => $"{e.Time}:{e.Event}:{e.Fields}")));
        }
        Assert.True(traces.Count >= 2);
    }

    [Fact]
    public void IsolatingAReplicaDiverges()
    {
        var result = ReplicatedFlag.RunScenario(1, new FaultSchedule(new[] { new Partition("coord", "r2", 0, 50) }));
        Assert.True(result.Diverged);
        Assert.NotEqual(result.Values["r1"], result.Values["r2"]);
    }

    [Fact]
    public void NoFaultsStaysConsistent()
    {
        var result = ReplicatedFlag.RunScenario(1, new FaultSchedule());
        Assert.False(result.Diverged);
        Assert.Equal(new Dictionary<string, int> { ["r1"] = 1, ["r2"] = 1 }, result.Values);
    }

    [Fact]
    public void CrashingTheCoordinatorDoesNotDiverge()
    {
        // Both sends are dropped, so the replicas agree (both still 0) - a real but
        // different outcome from isolating a single replica.
        var result = ReplicatedFlag.RunScenario(1, new FaultSchedule(crashes: new[] { new Crash("coord", 0, 50) }));
        Assert.False(result.Diverged);
        Assert.Equal(new Dictionary<string, int> { ["r1"] = 0, ["r2"] = 0 }, result.Values);
    }
}
