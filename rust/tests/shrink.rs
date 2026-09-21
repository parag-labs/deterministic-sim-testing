//! Shrinking tests, mirrored from the Python suite plus extra coverage on plain
//! data.

use deterministic_sim_testing::{
    ddmin, run_scenario, search_for_divergence, Crash, Fault, FaultSchedule,
};

// --- The three shrink tests mirrored from the Python suite. ---

#[test]
fn search_finds_a_diverging_schedule() {
    let found = search_for_divergence(0i64..500).expect("expected some seed in 0..499 to diverge");
    assert!(
        run_scenario(found.seed, FaultSchedule::of(&found.faults)).diverged,
        "the reported schedule should actually diverge"
    );
}

#[test]
fn ddmin_strips_the_irrelevant_faults() {
    let found = search_for_divergence(0i64..500).expect("expected a diverging seed");
    let seed = found.seed;

    // Pad the real reproducer with two faults that fire long after everything is
    // over - they can't possibly matter.
    let noise = vec![
        Fault::Crash(Crash {
            node: "r1".to_string(),
            start: 900,
            end: 950,
        }),
        Fault::Crash(Crash {
            node: "r2".to_string(),
            start: 800,
            end: 850,
        }),
    ];
    let mut padded = found.faults.clone();
    padded.extend(noise.iter().cloned());

    let still_fails = |subset: &[Fault]| run_scenario(seed, FaultSchedule::of(subset)).diverged;

    let minimal = ddmin(&padded, still_fails);

    assert!(
        still_fails(&minimal),
        "the minimal schedule must still fail"
    );
    assert!(
        minimal.len() < padded.len(),
        "ddmin should shrink: minimal={} padded={}",
        minimal.len(),
        padded.len()
    );
    for n in &noise {
        assert!(
            !minimal.contains(n),
            "late-firing noise {n:?} should be gone"
        );
    }
}

#[test]
#[should_panic(expected = "already fail")]
fn ddmin_requires_a_failing_input() {
    ddmin(&[1, 2, 3], |_| false);
}

// --- Extra coverage of the shrinker on plain data. ---

#[test]
fn ddmin_reduces_to_the_single_culprit() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8];
    // Only the presence of 7 matters.
    let still_fails = |xs: &[i32]| xs.contains(&7);
    let minimal = ddmin(&items, still_fails);
    assert_eq!(minimal, vec![7]);
}

#[test]
fn ddmin_keeps_all_when_every_item_matters() {
    let items = vec![1, 2, 3, 4];
    // Removing anything drops the length below 4, so the whole set is required.
    let still_fails = |xs: &[i32]| xs.len() == 4;
    let minimal = ddmin(&items, still_fails);
    assert_eq!(minimal, items);
}

#[test]
fn ddmin_single_failing_element() {
    let minimal = ddmin(&[42], |xs: &[i32]| xs.len() == 1);
    assert_eq!(minimal, vec![42]);
}
