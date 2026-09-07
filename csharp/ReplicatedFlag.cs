// A deliberately buggy workload, used to show deterministic-sim-testing finding +
// shrinking a bug.
//
// A coordinator sets a flag on two replicas with a single fire-and-forget message
// each - and, crucially, never retries. That missing retry is the bug: if the message
// to one replica is lost, the replicas diverge forever and nobody notices.

using System;
using System.Collections.Generic;
using System.Linq;

namespace SeedSim;

public sealed class Coordinator : Node
{
    private readonly List<string> _replicas;

    public Coordinator(List<string> replicas) : base("coord") => _replicas = replicas;

    public void Start()
    {
        // The bug in one line: send once, never confirm, never retry.
        foreach (var r in _replicas)
            Send(r, new Dictionary<string, object?> { ["kind"] = "set", ["value"] = 1 });
    }
}

public sealed class Replica : Node
{
    public int Value { get; private set; }

    public Replica(string nodeId) : base(nodeId) { }

    public override void OnMessage(string src, Dictionary<string, object?> msg)
    {
        if (Equals(msg["kind"], "set")) Value = (int)msg["value"]!;
    }
}

public static class ReplicatedFlag
{
    public static readonly string[] Nodes = { "coord", "r1", "r2" };
    public const int Horizon = 1000;

    public sealed record Result(List<TraceEntry> Trace, bool Diverged, Dictionary<string, int> Values);

    /// <summary>Run the workload once. Returns (trace, diverged, replica values).</summary>
    public static Result RunScenario(int seed, FaultSchedule faults)
    {
        var sim = new Simulator(seed, faults);
        var replicas = new List<Replica> { (Replica)sim.Add(new Replica("r1")), (Replica)sim.Add(new Replica("r2")) };
        var coord = (Coordinator)sim.Add(new Coordinator(replicas.Select(r => r.NodeId).ToList()));
        sim.At(0, coord.Start);
        sim.Run(until: Horizon);
        var values = replicas.ToDictionary(r => r.NodeId, r => r.Value);
        var diverged = values.Values.Distinct().Count() > 1;
        return new Result(sim.Trace, diverged, values);
    }

    /// <summary>Generate a small random fault schedule from a seed.</summary>
    public static List<Fault> RandomFaults(Rng rng, string[]? nodes = null)
    {
        nodes ??= Nodes;
        var outFaults = new List<Fault>();
        var count = rng.Between(1, 4);
        for (var i = 0; i < count; i++)
        {
            var start = (int)rng.Below(5);
            var end = start + (int)rng.Between(20, 200);
            if (rng.Chance(0.6))
            {
                var a = nodes[rng.Below(nodes.Length)];
                var b = nodes[rng.Below(nodes.Length)];
                if (a != b) outFaults.Add(new Partition(a, b, start, end));
            }
            else
            {
                var n = nodes[rng.Below(nodes.Length)];
                outFaults.Add(new Crash(n, start, end));
            }
        }
        return outFaults;
    }

    /// <summary>Return the first (seed, faults) whose run diverges, or null.</summary>
    public static (int Seed, List<Fault> Faults)? SearchForDivergence(IEnumerable<int> seeds)
    {
        foreach (var seed in seeds)
        {
            var faults = RandomFaults(new Rng(seed));
            var result = RunScenario(seed, FaultSchedule.Of(faults));
            if (result.Diverged) return (seed, faults);
        }
        return null;
    }
}
