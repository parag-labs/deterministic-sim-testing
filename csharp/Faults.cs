// Fault schedules: the injected adversity a run is subjected to.
//
// A fault is a plain, hashable value with a time window. The schedule is part of the
// reproducer (seed + faults fully determine a run), and because faults are value
// types they can be fed straight into the shrinker as opaque items.

using System.Collections.Generic;
using System.Linq;

namespace SeedSim;

/// <summary>A fault a run can be subjected to (partition or crash).</summary>
public abstract record Fault;

/// <summary>The link between two nodes is down for [Start, End).</summary>
public sealed record Partition(string A, string B, int Start, int End) : Fault;

/// <summary>Node is down for [Start, End): it sends nothing and receives nothing.</summary>
public sealed record Crash(string Node, int Start, int End) : Fault;

public sealed class FaultSchedule
{
    public List<Partition> Partitions { get; }
    public List<Crash> Crashes { get; }

    public FaultSchedule(
        IEnumerable<Partition>? partitions = null,
        IEnumerable<Crash>? crashes = null)
    {
        Partitions = partitions?.ToList() ?? new List<Partition>();
        Crashes = crashes?.ToList() ?? new List<Crash>();
    }

    public bool IsPartitioned(string a, string b, int t)
    {
        foreach (var p in Partitions)
        {
            if (p.Start <= t && t < p.End &&
                ((p.A == a && p.B == b) || (p.A == b && p.B == a)))
                return true;
        }
        return false;
    }

    public bool IsCrashed(string node, int t)
    {
        foreach (var c in Crashes)
            if (c.Node == node && c.Start <= t && t < c.End) return true;
        return false;
    }

    public List<Fault> Items()
    {
        var items = new List<Fault>();
        items.AddRange(Partitions);
        items.AddRange(Crashes);
        return items;
    }

    public static FaultSchedule Of(IEnumerable<Fault> items)
    {
        var list = items.ToList();
        return new FaultSchedule(
            list.OfType<Partition>(),
            list.OfType<Crash>());
    }
}
