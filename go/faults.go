// Fault schedules: the injected adversity a run is subjected to.
//
// A fault is a plain, comparable value with a time window. The schedule is part of
// the reproducer (seed + faults fully determine a run), and because faults are value
// types they can be fed straight into the shrinker as opaque items.

package deterministicsimtesting

// Fault is a fault a run can be subjected to (a Partition or a Crash).
type Fault interface {
	isFault()
}

// Partition means the link between two nodes is down for [Start, End).
type Partition struct {
	A     string
	B     string
	Start int
	End   int
}

func (Partition) isFault() {}

// Crash means Node is down for [Start, End): it sends nothing and receives nothing.
type Crash struct {
	Node  string
	Start int
	End   int
}

func (Crash) isFault() {}

// FaultSchedule is the full set of partitions and crashes applied to a run.
type FaultSchedule struct {
	Partitions []Partition
	Crashes    []Crash
}

// NewFaultSchedule builds a schedule from the given partitions and crashes. Nil
// slices are treated as empty.
func NewFaultSchedule(partitions []Partition, crashes []Crash) *FaultSchedule {
	fs := &FaultSchedule{
		Partitions: append([]Partition(nil), partitions...),
		Crashes:    append([]Crash(nil), crashes...),
	}
	return fs
}

// IsPartitioned reports whether the (unordered) link between a and b is down at t.
func (f *FaultSchedule) IsPartitioned(a, b string, t int) bool {
	for _, p := range f.Partitions {
		if p.Start <= t && t < p.End &&
			((p.A == a && p.B == b) || (p.A == b && p.B == a)) {
			return true
		}
	}
	return false
}

// IsCrashed reports whether node is down at t.
func (f *FaultSchedule) IsCrashed(node string, t int) bool {
	for _, c := range f.Crashes {
		if c.Node == node && c.Start <= t && t < c.End {
			return true
		}
	}
	return false
}

// Items returns every fault in the schedule (partitions first, then crashes).
func (f *FaultSchedule) Items() []Fault {
	out := make([]Fault, 0, len(f.Partitions)+len(f.Crashes))
	for _, p := range f.Partitions {
		out = append(out, p)
	}
	for _, c := range f.Crashes {
		out = append(out, c)
	}
	return out
}

// FaultScheduleOf partitions a flat list of faults back into a schedule.
func FaultScheduleOf(items []Fault) *FaultSchedule {
	var parts []Partition
	var crs []Crash
	for _, it := range items {
		switch f := it.(type) {
		case Partition:
			parts = append(parts, f)
		case Crash:
			crs = append(crs, f)
		}
	}
	return NewFaultSchedule(parts, crs)
}
