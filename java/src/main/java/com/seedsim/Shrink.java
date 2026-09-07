// Trace shrinking via delta debugging (ddmin).
//
// Finding a seed that breaks an invariant is only half the job; a 40-fault schedule
// that triggers a bug is nearly useless to a human. ddmin repeatedly throws away
// subsets of the faults and keeps whichever smaller schedule still fails, converging
// on a minimal reproducer. This is the classic Zeller/Hildebrandt algorithm.

package com.seedsim;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Predicate;

public final class Shrink {

    private Shrink() {}

    private static <T> List<List<T>> split(List<T> xs, int n) {
        int k = xs.size() / n;
        int r = xs.size() % n;
        List<List<T>> out = new ArrayList<>();
        int i = 0;
        for (int idx = 0; idx < n; idx++) {
            int size = k + (idx < r ? 1 : 0);
            if (size > 0) {
                out.add(new ArrayList<>(xs.subList(i, i + size)));
            }
            i += size;
        }
        return out;
    }

    /** Return a minimal sublist of items for which stillFails holds. stillFails(items)
     * must be true to begin with - you only shrink a known failure. */
    public static <T> List<T> ddmin(List<T> items, Predicate<List<T>> stillFails) {
        List<T> cur = new ArrayList<>(items);
        if (!stillFails.test(cur)) {
            throw new IllegalArgumentException("ddmin requires the full input to already fail");
        }

        int n = 2;
        while (cur.size() >= 2) {
            List<List<T>> chunks = split(cur, n);
            boolean reduced = false;
            for (int i = 0; i < chunks.size(); i++) {
                List<T> complement = new ArrayList<>();
                for (int j = 0; j < chunks.size(); j++) {
                    if (j != i) complement.addAll(chunks.get(j));
                }
                if (!complement.isEmpty() && stillFails.test(complement)) {
                    cur = complement;
                    n = Math.max(n - 1, 2);
                    reduced = true;
                    break;
                }
            }
            if (!reduced) {
                if (n >= cur.size()) break;
                n = Math.min(cur.size(), 2 * n);
            }
        }
        return cur;
    }
}
