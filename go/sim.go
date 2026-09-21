// The simulator core: a single-threaded discrete-event loop over virtual time.
//
// Everything that could introduce nondeterminism in a real distributed system - the
// clock, message latency, message loss, the order concurrent events fire in - is
// funnelled through this one loop and the seeded Rng. There are no real threads and
// no wall-clock reads, so a given (seed, workload, fault schedule) always produces
// the exact same event trace. That's the property the tests lean on.

package deterministicsimtesting

import (
	"container/heap"
	"fmt"
	"sort"
	"strings"
)

// TraceEntry is one ordered, comparable line of the event trace.
type TraceEntry struct {
	Time   int
	Event  string
	Fields string
}

// KV is a single key/value field recorded on a trace entry.
type KV struct {
	Key string
	Val any
}

type eventItem struct {
	time int
	seq  int
	fn   func()
}

type eventHeap []*eventItem

func (h eventHeap) Len() int { return len(h) }

// Less breaks ties on insertion order (seq), which keeps concurrent events deterministic.
func (h eventHeap) Less(i, j int) bool {
	if h[i].time != h[j].time {
		return h[i].time < h[j].time
	}
	return h[i].seq < h[j].seq
}

func (h eventHeap) Swap(i, j int) { h[i], h[j] = h[j], h[i] }

func (h *eventHeap) Push(x any) { *h = append(*h, x.(*eventItem)) }

func (h *eventHeap) Pop() any {
	old := *h
	n := len(old)
	it := old[n-1]
	old[n-1] = nil
	*h = old[:n-1]
	return it
}

// Simulator is a deterministic discrete-event loop over integer virtual time.
type Simulator struct {
	Seed   int
	Rng    *Rng
	Faults *FaultSchedule
	Net    *Network
	Nodes  map[string]Node
	Trace  []TraceEntry

	now  int
	heap eventHeap
	seq  int
}

// NewSimulator creates a simulator for the given seed and fault schedule. A nil
// schedule means no faults.
func NewSimulator(seed int, faults *FaultSchedule) *Simulator {
	if faults == nil {
		faults = NewFaultSchedule(nil, nil)
	}
	s := &Simulator{
		Seed:   seed,
		Rng:    NewRng(int64(seed)),
		Faults: faults,
		Nodes:  make(map[string]Node),
	}
	s.Net = NewNetwork(s)
	return s
}

// Now returns the current virtual time.
func (s *Simulator) Now() int { return s.now }

// Add registers a node with the simulator and returns it.
func (s *Simulator) Add(n Node) Node {
	b := n.base()
	b.sim = s
	b.self = n
	s.Nodes[b.ID] = n
	return n
}

// At schedules fn to run delay ticks from now. Ties break on insertion order (seq),
// which is what keeps concurrent events deterministic. It panics if delay < 0.
func (s *Simulator) At(delay int, fn func()) {
	if delay < 0 {
		panic("delay must be >= 0")
	}
	heap.Push(&s.heap, &eventItem{time: s.now + delay, seq: s.seq, fn: fn})
	s.seq++
}

// Record appends a fully ordered, comparable line to the trace. Fields are sorted
// by key so two runs can be compared for exact equality.
func (s *Simulator) Record(event string, fields ...KV) {
	sorted := make([]KV, len(fields))
	copy(sorted, fields)
	sort.SliceStable(sorted, func(i, j int) bool { return sorted[i].Key < sorted[j].Key })
	var sb strings.Builder
	for i, f := range sorted {
		if i > 0 {
			sb.WriteByte(',')
		}
		sb.WriteString(f.Key)
		sb.WriteByte('=')
		if f.Val == nil {
			sb.WriteString("None")
		} else {
			sb.WriteString(fmt.Sprint(f.Val))
		}
	}
	s.Trace = append(s.Trace, TraceEntry{Time: s.now, Event: event, Fields: sb.String()})
}

// Run drains the event loop until it empties, until virtual time would pass until
// (nil for no limit), or until maxSteps events have fired.
func (s *Simulator) Run(until *int, maxSteps int) []TraceEntry {
	steps := 0
	for s.heap.Len() > 0 && steps < maxSteps {
		if until != nil && s.heap[0].time > *until {
			break
		}
		nxt := heap.Pop(&s.heap).(*eventItem)
		s.now = nxt.time
		nxt.fn()
		steps++
	}
	return s.Trace
}

// Node is a participant in the simulation. Subclasses react to messages and timers;
// they never touch time or the network directly except through the helpers on
// BaseNode.
type Node interface {
	NodeID() string
	OnMessage(src string, msg map[string]any)
	OnTimer(name string, payload any)
	base() *BaseNode
}

// BaseNode provides the machinery every Node shares: its id, its link back to the
// simulator, and the Send/Timer helpers. Embed it in a concrete node type.
type BaseNode struct {
	ID   string
	sim  *Simulator
	self Node
}

// NodeID returns the node's identifier.
func (b *BaseNode) NodeID() string { return b.ID }

func (b *BaseNode) base() *BaseNode { return b }

// OnMessage is the default no-op message handler. Override it in a concrete node.
func (b *BaseNode) OnMessage(src string, msg map[string]any) {}

// OnTimer is the default no-op timer handler. Override it in a concrete node.
func (b *BaseNode) OnTimer(name string, payload any) {}

// Send hands a message to the network from this node.
func (b *BaseNode) Send(dst string, msg map[string]any) {
	b.sim.Net.Send(b.ID, dst, msg)
}

// Timer schedules a named timer to fire delay ticks from now. A crashed node's
// timers are silently swallowed when they would fire.
func (b *BaseNode) Timer(delay int, name string, payload any) {
	sim := b.sim
	self := b.self
	id := b.ID
	sim.At(delay, func() {
		if sim.Faults.IsCrashed(id, sim.Now()) {
			return
		}
		sim.Record("timer", KV{Key: "node", Val: id}, KV{Key: "name", Val: name})
		self.OnTimer(name, payload)
	})
}
