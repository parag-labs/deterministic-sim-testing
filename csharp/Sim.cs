// The simulator core: a single-threaded discrete-event loop over virtual time.
//
// Everything that could introduce nondeterminism in a real distributed system - the
// clock, message latency, message loss, the order concurrent events fire in - is
// funnelled through this one loop and the seeded Rng. There are no real threads and
// no wall-clock reads, so a given (seed, workload, fault schedule) always produces
// the exact same event trace. That's the property the tests lean on.

using System;
using System.Collections.Generic;
using System.Linq;

namespace SeedSim;

/// <summary>One ordered, comparable line of the event trace.</summary>
public sealed record TraceEntry(int Time, string Event, string Fields);

public sealed class Simulator
{
    public int Seed { get; }
    public Rng Rng { get; }
    public int Now { get; private set; }
    public FaultSchedule Faults { get; }
    public Network Net { get; }
    public Dictionary<string, Node> Nodes { get; } = new();
    public List<TraceEntry> Trace { get; } = new();

    // Ties break on insertion order (seq), which keeps concurrent events deterministic.
    private readonly PriorityQueue<Action, (int Time, int Seq)> _heap = new();
    private int _seq;

    public Simulator(int seed, FaultSchedule? faults = null)
    {
        Seed = seed;
        Rng = new Rng(seed);
        Faults = faults ?? new FaultSchedule();
        Net = new Network(this);
    }

    public Node Add(Node node)
    {
        node.Sim = this;
        Nodes[node.NodeId] = node;
        return node;
    }

    /// <summary>Schedule <paramref name="fn"/> to run <paramref name="delay"/> ticks from now.</summary>
    public void At(int delay, Action fn)
    {
        if (delay < 0) throw new ArgumentException("delay must be >= 0");
        _heap.Enqueue(fn, (Now + delay, _seq));
        _seq++;
    }

    public void Record(string ev, params (string Key, object? Val)[] fields)
    {
        var canonical = string.Join(",",
            fields.OrderBy(f => f.Key, StringComparer.Ordinal)
                  .Select(f => $"{f.Key}={f.Val?.ToString() ?? "None"}"));
        Trace.Add(new TraceEntry(Now, ev, canonical));
    }

    public List<TraceEntry> Run(int? until = null, int maxSteps = 1_000_000)
    {
        var steps = 0;
        while (_heap.Count > 0 && steps < maxSteps)
        {
            _heap.TryPeek(out _, out var priority);
            if (until is not null && priority.Time > until) break;
            var fn = _heap.Dequeue();
            Now = priority.Time;
            fn();
            steps++;
        }
        return Trace;
    }
}

/// <summary>
/// Base class for a participant. Subclasses react to messages and timers; they never
/// touch time or the network directly except through these helpers.
/// </summary>
public abstract class Node
{
    public string NodeId { get; }
    public Simulator? Sim { get; set; }

    protected Node(string nodeId) => NodeId = nodeId;

    public void Send(string dst, Dictionary<string, object?> msg)
        => Sim!.Net.Send(NodeId, dst, msg);

    public void Timer(int delay, string name, object? payload = null)
    {
        var sim = Sim!;
        void Fire()
        {
            if (sim.Faults.IsCrashed(NodeId, sim.Now)) return;
            sim.Record("timer", ("node", NodeId), ("name", name));
            OnTimer(name, payload);
        }
        sim.At(delay, Fire);
    }

    public virtual void OnMessage(string src, Dictionary<string, object?> msg) { }
    public virtual void OnTimer(string name, object? payload) { }
}
