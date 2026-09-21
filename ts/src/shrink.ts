// Trace shrinking via delta debugging (ddmin).
//
// Finding a seed that breaks an invariant is only half the job; a 40-fault
// schedule that triggers a bug is nearly useless to a human. Ddmin repeatedly
// throws away subsets of the faults and keeps whichever smaller schedule still
// fails, converging on a minimal reproducer. This is the classic
// Zeller/Hildebrandt algorithm.

function split<T>(xs: T[], n: number): T[][] {
  const k = Math.floor(xs.length / n);
  const r = xs.length % n;
  const out: T[][] = [];
  let i = 0;
  for (let idx = 0; idx < n; idx++) {
    let size = k;
    if (idx < r) {
      size += 1;
    }
    if (size > 0) {
      out.push(xs.slice(i, i + size));
      i += size;
    }
  }
  return out;
}

/**
 * Returns a minimal sublist of `items` for which `stillFails` holds.
 *
 * `stillFails(items)` must be true to begin with - you only shrink a known
 * failure - otherwise this throws.
 */
export function ddmin<T>(items: T[], stillFails: (subset: T[]) => boolean): T[] {
  let cur = [...items];
  if (!stillFails(cur)) {
    throw new Error("ddmin requires the full input to already fail");
  }

  let n = 2;
  while (cur.length >= 2) {
    const chunks = split(cur, n);
    let reduced = false;
    for (let i = 0; i < chunks.length; i++) {
      const complement: T[] = [];
      for (let j = 0; j < chunks.length; j++) {
        if (j !== i) {
          complement.push(...chunks[j]);
        }
      }
      if (complement.length > 0 && stillFails(complement)) {
        cur = complement;
        n = n - 1 > 2 ? n - 1 : 2;
        reduced = true;
        break;
      }
    }
    if (!reduced) {
      if (n >= cur.length) {
        break;
      }
      n = 2 * n < cur.length ? 2 * n : cur.length;
    }
  }
  return cur;
}
