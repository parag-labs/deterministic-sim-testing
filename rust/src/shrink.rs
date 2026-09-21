//! Trace shrinking via delta debugging (ddmin).
//!
//! Finding a seed that breaks an invariant is only half the job; a 40-fault
//! schedule that triggers a bug is nearly useless to a human. Ddmin repeatedly
//! throws away subsets of the faults and keeps whichever smaller schedule still
//! fails, converging on a minimal reproducer. This is the classic
//! Zeller/Hildebrandt algorithm.

fn split<T: Clone>(xs: &[T], n: usize) -> Vec<Vec<T>> {
    let k = xs.len() / n;
    let r = xs.len() % n;
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    for idx in 0..n {
        let mut size = k;
        if idx < r {
            size += 1;
        }
        if size > 0 {
            out.push(xs[i..i + size].to_vec());
            i += size;
        }
    }
    out
}

/// Returns a minimal sublist of `items` for which `still_fails` holds.
///
/// `still_fails(items)` must be true to begin with - you only shrink a known
/// failure - otherwise this panics.
pub fn ddmin<T, F>(items: &[T], still_fails: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&[T]) -> bool,
{
    let mut cur = items.to_vec();
    assert!(
        still_fails(&cur),
        "ddmin requires the full input to already fail"
    );

    let mut n = 2;
    while cur.len() >= 2 {
        let chunks = split(&cur, n);
        let mut reduced = false;
        for i in 0..chunks.len() {
            let complement: Vec<T> = chunks
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, chunk)| chunk.iter().cloned())
                .collect();
            if !complement.is_empty() && still_fails(&complement) {
                cur = complement;
                n = if n - 1 > 2 { n - 1 } else { 2 };
                reduced = true;
                break;
            }
        }
        if !reduced {
            if n >= cur.len() {
                break;
            }
            n = if 2 * n < cur.len() { 2 * n } else { cur.len() };
        }
    }
    cur
}
