// Trace shrinking via delta debugging (ddmin).
//
// Finding a seed that breaks an invariant is only half the job; a 40-fault schedule
// that triggers a bug is nearly useless to a human. Ddmin repeatedly throws away
// subsets of the faults and keeps whichever smaller schedule still fails, converging
// on a minimal reproducer. This is the classic Zeller/Hildebrandt algorithm.

package deterministicsimtesting

func split[T any](xs []T, n int) [][]T {
	k := len(xs) / n
	r := len(xs) % n
	out := make([][]T, 0, n)
	i := 0
	for idx := 0; idx < n; idx++ {
		size := k
		if idx < r {
			size++
		}
		if size > 0 {
			chunk := make([]T, size)
			copy(chunk, xs[i:i+size])
			out = append(out, chunk)
		}
		i += size
	}
	return out
}

// Ddmin returns a minimal sublist of items for which stillFails holds.
// stillFails(items) must be true to begin with - you only shrink a known failure -
// otherwise Ddmin panics.
func Ddmin[T any](items []T, stillFails func([]T) bool) []T {
	cur := make([]T, len(items))
	copy(cur, items)
	if !stillFails(cur) {
		panic("ddmin requires the full input to already fail")
	}

	n := 2
	for len(cur) >= 2 {
		chunks := split(cur, n)
		reduced := false
		for i := range chunks {
			complement := make([]T, 0, len(cur))
			for j := range chunks {
				if j != i {
					complement = append(complement, chunks[j]...)
				}
			}
			if len(complement) > 0 && stillFails(complement) {
				cur = complement
				if n-1 > 2 {
					n = n - 1
				} else {
					n = 2
				}
				reduced = true
				break
			}
		}
		if !reduced {
			if n >= len(cur) {
				break
			}
			if 2*n < len(cur) {
				n = 2 * n
			} else {
				n = len(cur)
			}
		}
	}
	return cur
}
