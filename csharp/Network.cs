// The message-passing layer, where latency and loss are decided.
//
// Delay and (optional) random drop are drawn from the simulator's seeded Rng, and
// partitions/crashes are consulted from the fault schedule - both at send time and
// again at delivery time, because a node can crash while a message is in flight.
// Keeping this in one place is what lets the whole run stay deterministic.

using System.Collections.Generic;

namespace SeedSim;

public sealed class Network
{
    private readonly Simulator _sim;
    public int MinDelay { get; }
    public int MaxDelay { get; }
    public double DropProb { get; }

    public Network(Simulator sim, int minDelay = 1, int maxDelay = 10, double dropProb = 0.0)
    {
        _sim = sim;
        MinDelay = minDelay;
        MaxDelay = maxDelay;
        DropProb = dropProb;
    }

    public void Send(string src, string dst, Dictionary<string, object?> msg)
    {
        var sim = _sim;
        var now = sim.Now;
        msg.TryGetValue("kind", out var kind);

        // Blocked before it ever leaves: partition, or either end down.
        if (sim.Faults.IsPartitioned(src, dst, now)
            || sim.Faults.IsCrashed(src, now)
            || sim.Faults.IsCrashed(dst, now))
        {
            sim.Record("drop", ("src", src), ("dst", dst), ("kind", kind));
            return;
        }

        // Only consume an rng draw for loss when loss is actually configured, so
        // enabling DropProb doesn't silently shift the delay stream.
        if (DropProb > 0.0 && sim.Rng.Chance(DropProb))
        {
            sim.Record("drop", ("src", src), ("dst", dst), ("kind", kind));
            return;
        }

        var delay = (int)sim.Rng.Between(MinDelay, MaxDelay);

        void Deliver()
        {
            // Re-check at arrival: the link may have partitioned or the destination
            // may have crashed while the message was in flight.
            if (sim.Faults.IsPartitioned(src, dst, sim.Now) || sim.Faults.IsCrashed(dst, sim.Now))
            {
                sim.Record("drop", ("src", src), ("dst", dst), ("kind", kind));
                return;
            }
            sim.Record("deliver", ("src", src), ("dst", dst), ("kind", kind));
            sim.Nodes[dst].OnMessage(src, msg);
        }

        sim.At(delay, Deliver);
    }
}
