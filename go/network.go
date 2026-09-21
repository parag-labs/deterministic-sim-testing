// The message-passing layer, where latency and loss are decided.
//
// Delay and (optional) random drop are drawn from the simulator's seeded Rng, and
// partitions/crashes are consulted from the fault schedule - both at send time and
// again at delivery time, because a node can crash while a message is in flight.
// Keeping this in one place is what lets the whole run stay deterministic.

package deterministicsimtesting

// Network is the simulated message layer for a Simulator.
type Network struct {
	sim      *Simulator
	MinDelay int
	MaxDelay int
	DropProb float64
}

// NewNetwork creates a network with the default latency window [1, 10] and no loss.
func NewNetwork(sim *Simulator) *Network {
	return &Network{sim: sim, MinDelay: 1, MaxDelay: 10, DropProb: 0.0}
}

// Send routes a message from src to dst, applying latency, loss, and faults.
func (n *Network) Send(src, dst string, msg map[string]any) {
	sim := n.sim
	now := sim.Now()
	kind := msg["kind"]

	// Blocked before it ever leaves: partition, or either end down.
	if sim.Faults.IsPartitioned(src, dst, now) ||
		sim.Faults.IsCrashed(src, now) ||
		sim.Faults.IsCrashed(dst, now) {
		sim.Record("drop", KV{Key: "src", Val: src}, KV{Key: "dst", Val: dst}, KV{Key: "kind", Val: kind})
		return
	}

	// Only consume an rng draw for loss when loss is actually configured, so
	// enabling DropProb doesn't silently shift the delay stream.
	if n.DropProb > 0.0 && sim.Rng.Chance(n.DropProb) {
		sim.Record("drop", KV{Key: "src", Val: src}, KV{Key: "dst", Val: dst}, KV{Key: "kind", Val: kind})
		return
	}

	delay := int(sim.Rng.Between(int64(n.MinDelay), int64(n.MaxDelay)))

	sim.At(delay, func() {
		// Re-check at arrival: the link may have partitioned or the destination
		// may have crashed while the message was in flight.
		if sim.Faults.IsPartitioned(src, dst, sim.Now()) || sim.Faults.IsCrashed(dst, sim.Now()) {
			sim.Record("drop", KV{Key: "src", Val: src}, KV{Key: "dst", Val: dst}, KV{Key: "kind", Val: kind})
			return
		}
		sim.Record("deliver", KV{Key: "src", Val: src}, KV{Key: "dst", Val: dst}, KV{Key: "kind", Val: kind})
		sim.Nodes[dst].OnMessage(src, msg)
	})
}
