// A deliberately buggy workload, used to show deterministic-sim-testing finding +
// shrinking a bug.
//
// A coordinator sets a flag on two replicas with a single fire-and-forget message
// each - and, crucially, never retries. That missing retry is the bug: if the
// message to one replica is lost, the replicas diverge forever and nobody notices.

package deterministicsimtesting

// Nodes are the participants in the worked example.
var Nodes = []string{"coord", "r1", "r2"}

// Horizon is the virtual time the scenario runs until.
const Horizon = 1000

// Coordinator sets a flag on each replica exactly once and never retries.
type Coordinator struct {
	BaseNode
	replicas []string
}

// NewCoordinator builds the coordinator for the given replica ids.
func NewCoordinator(replicas []string) *Coordinator {
	c := &Coordinator{replicas: replicas}
	c.ID = "coord"
	return c
}

// Start fires the coordinator's one-shot writes. The bug in one line: send once,
// never confirm, never retry.
func (c *Coordinator) Start() {
	for _, r := range c.replicas {
		c.Send(r, map[string]any{"kind": "set", "value": 1})
	}
}

// Replica stores a single flag value updated by "set" messages.
type Replica struct {
	BaseNode
	Value int
}

// NewReplica builds a replica with the given id and an initial value of 0.
func NewReplica(id string) *Replica {
	r := &Replica{}
	r.ID = id
	return r
}

// OnMessage applies a "set" message to the replica's value.
func (r *Replica) OnMessage(src string, msg map[string]any) {
	if msg["kind"] == "set" {
		r.Value = msg["value"].(int)
	}
}

// Result is the outcome of a single scenario run.
type Result struct {
	Trace    []TraceEntry
	Diverged bool
	Values   map[string]int
}

// Found is a seed together with the fault schedule that made it diverge.
type Found struct {
	Seed   int
	Faults []Fault
}

// RunScenario runs the workload once and reports the trace, whether the replicas
// diverged, and their final values.
func RunScenario(seed int, faults *FaultSchedule) Result {
	sim := NewSimulator(seed, faults)
	r1 := NewReplica("r1")
	sim.Add(r1)
	r2 := NewReplica("r2")
	sim.Add(r2)
	coord := NewCoordinator([]string{r1.NodeID(), r2.NodeID()})
	sim.Add(coord)
	sim.At(0, coord.Start)
	horizon := Horizon
	sim.Run(&horizon, 1_000_000)

	values := map[string]int{"r1": r1.Value, "r2": r2.Value}
	diverged := r1.Value != r2.Value
	return Result{Trace: sim.Trace, Diverged: diverged, Values: values}
}

// RandomFaults generates a small random fault schedule from a seeded Rng. Passing
// nil for nodes uses the default Nodes. Windows are biased to start early because
// all the interesting activity happens in the first few ticks.
func RandomFaults(rng *Rng, nodes []string) []Fault {
	if nodes == nil {
		nodes = Nodes
	}
	var out []Fault
	count := rng.Between(1, 4)
	for i := int64(0); i < count; i++ {
		start := int(rng.Below(5))
		end := start + int(rng.Between(20, 200))
		if rng.Chance(0.6) {
			a := nodes[rng.Below(int64(len(nodes)))]
			b := nodes[rng.Below(int64(len(nodes)))]
			if a != b {
				out = append(out, Partition{A: a, B: b, Start: start, End: end})
			}
		} else {
			n := nodes[rng.Below(int64(len(nodes)))]
			out = append(out, Crash{Node: n, Start: start, End: end})
		}
	}
	return out
}

// SearchForDivergence returns the first (seed, faults) whose run diverges, or nil.
func SearchForDivergence(seeds []int) *Found {
	for _, seed := range seeds {
		faults := RandomFaults(NewRng(int64(seed)), nil)
		if RunScenario(seed, FaultScheduleOf(faults)).Diverged {
			return &Found{Seed: seed, Faults: faults}
		}
	}
	return nil
}
