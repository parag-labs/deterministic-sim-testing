package deterministicsimtesting

import (
	"reflect"
	"testing"
)

// --- The three shrink tests mirrored from the Python suite. ---

func TestSearchFindsADivergingSchedule(t *testing.T) {
	found := SearchForDivergence(seedRange(500))
	if found == nil {
		t.Fatal("expected some seed in 0..499 to diverge")
	}
	if !RunScenario(found.Seed, FaultScheduleOf(found.Faults)).Diverged {
		t.Fatal("the reported schedule should actually diverge")
	}
}

func TestDdminStripsTheIrrelevantFaults(t *testing.T) {
	found := SearchForDivergence(seedRange(500))
	if found == nil {
		t.Fatal("expected a diverging seed")
	}
	seed := found.Seed

	// Pad the real reproducer with two faults that fire long after everything is
	// over - they can't possibly matter.
	noise := []Fault{Crash{Node: "r1", Start: 900, End: 950}, Crash{Node: "r2", Start: 800, End: 850}}
	padded := append(append([]Fault{}, found.Faults...), noise...)

	stillFails := func(subset []Fault) bool {
		return RunScenario(seed, FaultScheduleOf(subset)).Diverged
	}

	minimal := Ddmin(padded, stillFails)

	if !stillFails(minimal) {
		t.Fatal("the minimal schedule must still fail")
	}
	if len(minimal) >= len(padded) {
		t.Fatalf("ddmin should shrink: len(minimal)=%d len(padded)=%d", len(minimal), len(padded))
	}
	for _, n := range noise {
		for _, m := range minimal {
			if m == n {
				t.Fatalf("late-firing noise %v should be gone from %v", n, minimal)
			}
		}
	}
}

func TestDdminRequiresAFailingInput(t *testing.T) {
	defer func() {
		if recover() == nil {
			t.Fatal("ddmin should panic when the full input does not fail")
		}
	}()
	Ddmin([]int{1, 2, 3}, func([]int) bool { return false })
}

// --- Extra coverage of the shrinker on plain data. ---

func TestDdminReducesToTheSingleCulprit(t *testing.T) {
	items := []int{1, 2, 3, 4, 5, 6, 7, 8}
	// Only the presence of 7 matters.
	stillFails := func(xs []int) bool {
		for _, x := range xs {
			if x == 7 {
				return true
			}
		}
		return false
	}
	minimal := Ddmin(items, stillFails)
	if !reflect.DeepEqual(minimal, []int{7}) {
		t.Fatalf("minimal = %v, want [7]", minimal)
	}
}

func TestDdminKeepsAllWhenEveryItemMatters(t *testing.T) {
	items := []int{1, 2, 3, 4}
	// Removing anything drops the length below 4, so the whole set is required.
	stillFails := func(xs []int) bool { return len(xs) == 4 }
	minimal := Ddmin(items, stillFails)
	if !reflect.DeepEqual(minimal, items) {
		t.Fatalf("minimal = %v, want %v", minimal, items)
	}
}

func TestDdminSingleFailingElement(t *testing.T) {
	minimal := Ddmin([]int{42}, func(xs []int) bool { return len(xs) == 1 })
	if !reflect.DeepEqual(minimal, []int{42}) {
		t.Fatalf("minimal = %v, want [42]", minimal)
	}
}
