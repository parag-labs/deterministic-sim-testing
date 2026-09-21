package deterministicsimtesting

import (
	"fmt"
	"reflect"
	"strings"
	"testing"
)

func seedRange(n int) []int {
	s := make([]int, n)
	for i := range s {
		s[i] = i
	}
	return s
}

func traceKey(trace []TraceEntry) string {
	var sb strings.Builder
	for _, e := range trace {
		fmt.Fprintf(&sb, "%d:%s:%s|", e.Time, e.Event, e.Fields)
	}
	return sb.String()
}

// --- The five determinism tests mirrored from the Python suite. ---

func TestSameSeedSameTrace(t *testing.T) {
	faults := NewFaultSchedule([]Partition{{A: "coord", B: "r2", Start: 0, End: 50}}, nil)
	t1 := RunScenario(12345, faults).Trace
	t2 := RunScenario(12345, faults).Trace
	if !reflect.DeepEqual(t1, t2) {
		t.Fatalf("a run must be a pure function of (seed, faults): %v != %v", t1, t2)
	}
}

func TestDifferentSeedsCanDivergeInTrace(t *testing.T) {
	traces := map[string]bool{}
	for s := 0; s < 20; s++ {
		traces[traceKey(RunScenario(s, NewFaultSchedule(nil, nil)).Trace)] = true
	}
	if len(traces) < 2 {
		t.Fatalf("expected at least 2 distinct traces across seeds, got %d", len(traces))
	}
}

func TestIsolatingAReplicaDiverges(t *testing.T) {
	res := RunScenario(1, NewFaultSchedule([]Partition{{A: "coord", B: "r2", Start: 0, End: 50}}, nil))
	if !res.Diverged {
		t.Fatal("isolating r2 should diverge")
	}
	if res.Values["r1"] == res.Values["r2"] {
		t.Fatalf("replicas should hold different values, got %v", res.Values)
	}
}

func TestNoFaultsStaysConsistent(t *testing.T) {
	res := RunScenario(1, NewFaultSchedule(nil, nil))
	if res.Diverged {
		t.Fatal("no faults should not diverge")
	}
	if want := map[string]int{"r1": 1, "r2": 1}; !reflect.DeepEqual(res.Values, want) {
		t.Fatalf("want %v, got %v", want, res.Values)
	}
}

func TestCrashingTheCoordinatorDoesNotDiverge(t *testing.T) {
	// Both sends are dropped, so the replicas agree (both still 0).
	res := RunScenario(1, NewFaultSchedule(nil, []Crash{{Node: "coord", Start: 0, End: 50}}))
	if res.Diverged {
		t.Fatal("crashing the coordinator should not diverge")
	}
	if want := map[string]int{"r1": 0, "r2": 0}; !reflect.DeepEqual(res.Values, want) {
		t.Fatalf("want %v, got %v", want, res.Values)
	}
}

// --- Cross-language exactness checks against the Python reference stream. ---

func TestRngNextU64MatchesReference(t *testing.T) {
	r := NewRng(12345)
	want := []uint64{
		2454886589211414944,
		3778200017661327597,
		2205171434679333405,
		3248800117070709450,
		9350289611492784363,
	}
	for i, w := range want {
		if got := r.NextU64(); got != w {
			t.Fatalf("NextU64[%d] = %d, want %d", i, got, w)
		}
	}
}

func TestRngBelowMatchesReference(t *testing.T) {
	r := NewRng(1)
	want := []int64{0, 4, 0, 0, 1}
	for i, w := range want {
		if got := r.Below(5); got != w {
			t.Fatalf("Below(5)[%d] = %d, want %d", i, got, w)
		}
	}
}

func TestRngBetweenMatchesReference(t *testing.T) {
	r := NewRng(1)
	want := []int64{84, 161, 197, 150, 86}
	for i, w := range want {
		if got := r.Between(20, 200); got != w {
			t.Fatalf("Between(20,200)[%d] = %d, want %d", i, got, w)
		}
	}
}

func TestRngChanceMatchesReference(t *testing.T) {
	r := NewRng(1)
	want := []bool{true, false, false, true, true, false, false, true}
	for i, w := range want {
		if got := r.Chance(0.6); got != w {
			t.Fatalf("Chance(0.6)[%d] = %v, want %v", i, got, w)
		}
	}
}

func TestBelowPanicsOnNonPositive(t *testing.T) {
	for _, n := range []int64{0, -1} {
		func() {
			defer func() {
				if recover() == nil {
					t.Fatalf("Below(%d) should panic", n)
				}
			}()
			NewRng(1).Below(n)
		}()
	}
}

func TestBetweenPanicsWhenHiLessThanLo(t *testing.T) {
	defer func() {
		if recover() == nil {
			t.Fatal("Between(5,4) should panic")
		}
	}()
	NewRng(1).Between(5, 4)
}

func TestBetweenIsInclusiveOnASinglePoint(t *testing.T) {
	r := NewRng(99)
	for i := 0; i < 100; i++ {
		if got := r.Between(7, 7); got != 7 {
			t.Fatalf("Between(7,7) = %d, want 7", got)
		}
	}
}

func TestChanceBoundariesConsumeNoDraw(t *testing.T) {
	r := NewRng(1)
	if r.Chance(0.0) {
		t.Fatal("Chance(0) must be false")
	}
	if !r.Chance(1.0) {
		t.Fatal("Chance(1) must be true")
	}
	if r.Chance(-3.0) {
		t.Fatal("Chance(negative) must be false")
	}
	if !r.Chance(2.0) {
		t.Fatal("Chance(>1) must be true")
	}
	// None of the above should have advanced the stream.
	if got := r.NextU64(); got != NewRng(1).NextU64() {
		t.Fatalf("boundary Chance calls must not consume a draw, got %d", got)
	}
}

func TestScenarioTraceMatchesReference(t *testing.T) {
	faults := NewFaultSchedule([]Partition{{A: "coord", B: "r2", Start: 0, End: 50}}, nil)
	res := RunScenario(12345, faults)
	want := []TraceEntry{
		{Time: 0, Event: "drop", Fields: "dst=r2,kind=set,src=coord"},
		{Time: 5, Event: "deliver", Fields: "dst=r1,kind=set,src=coord"},
	}
	if !reflect.DeepEqual(res.Trace, want) {
		t.Fatalf("trace = %v, want %v", res.Trace, want)
	}
	if !res.Diverged {
		t.Fatal("seed 12345 with the r2 partition should diverge")
	}
	if res.Values["r1"] != 1 || res.Values["r2"] != 0 {
		t.Fatalf("values = %v, want r1=1 r2=0", res.Values)
	}
}

func TestNoFaultScenarioTraceMatchesReference(t *testing.T) {
	res := RunScenario(1, NewFaultSchedule(nil, nil))
	want := []TraceEntry{
		{Time: 6, Event: "deliver", Fields: "dst=r1,kind=set,src=coord"},
		{Time: 10, Event: "deliver", Fields: "dst=r2,kind=set,src=coord"},
	}
	if !reflect.DeepEqual(res.Trace, want) {
		t.Fatalf("trace = %v, want %v", res.Trace, want)
	}
}

func TestRandomFaultsMatchesReference(t *testing.T) {
	got := RandomFaults(NewRng(1), nil)
	want := []Fault{
		Partition{A: "coord", B: "r2", Start: 4, End: 201},
		Partition{A: "r1", B: "coord", Start: 0, End: 191},
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("RandomFaults(Rng(1)) = %v, want %v", got, want)
	}
}

func TestSearchForDivergenceMatchesReference(t *testing.T) {
	found := SearchForDivergence(seedRange(500))
	if found == nil {
		t.Fatal("expected a diverging seed in 0..499")
	}
	if found.Seed != 3 {
		t.Fatalf("first diverging seed = %d, want 3", found.Seed)
	}
	want := []Fault{Partition{A: "coord", B: "r1", Start: 1, End: 90}}
	if !reflect.DeepEqual(found.Faults, want) {
		t.Fatalf("faults = %v, want %v", found.Faults, want)
	}
}
