//! Determinism and cross-language exactness tests, mirrored from the Python suite.

use std::collections::HashSet;

use deterministic_sim_testing::{
    random_faults, run_scenario, search_for_divergence, Crash, Fault, FaultSchedule, Partition,
    Rng, TraceEntry,
};

fn entry(time: i64, event: &str, fields: &str) -> TraceEntry {
    TraceEntry {
        time,
        event: event.to_string(),
        fields: fields.to_string(),
    }
}

fn trace_key(trace: &[TraceEntry]) -> String {
    trace
        .iter()
        .map(|e| format!("{}:{}:{}|", e.time, e.event, e.fields))
        .collect()
}

// --- The five determinism tests mirrored from the Python suite. ---

#[test]
fn same_seed_same_trace() {
    let faults = || {
        FaultSchedule::new(
            vec![Partition {
                a: "coord".to_string(),
                b: "r2".to_string(),
                start: 0,
                end: 50,
            }],
            vec![],
        )
    };
    let t1 = run_scenario(12345, faults()).trace;
    let t2 = run_scenario(12345, faults()).trace;
    assert_eq!(t1, t2, "a run must be a pure function of (seed, faults)");
}

#[test]
fn different_seeds_can_diverge_in_trace() {
    let mut traces = HashSet::new();
    for s in 0..20 {
        traces.insert(trace_key(&run_scenario(s, FaultSchedule::empty()).trace));
    }
    assert!(
        traces.len() >= 2,
        "expected at least 2 distinct traces across seeds, got {}",
        traces.len()
    );
}

#[test]
fn isolating_a_replica_diverges() {
    let res = run_scenario(
        1,
        FaultSchedule::new(
            vec![Partition {
                a: "coord".to_string(),
                b: "r2".to_string(),
                start: 0,
                end: 50,
            }],
            vec![],
        ),
    );
    assert!(res.diverged, "isolating r2 should diverge");
    assert_ne!(
        res.values["r1"], res.values["r2"],
        "replicas should hold different values"
    );
}

#[test]
fn no_faults_stays_consistent() {
    let res = run_scenario(1, FaultSchedule::empty());
    assert!(!res.diverged, "no faults should not diverge");
    assert_eq!(res.values["r1"], 1);
    assert_eq!(res.values["r2"], 1);
}

#[test]
fn crashing_the_coordinator_does_not_diverge() {
    // Both sends are dropped, so the replicas agree (both still 0).
    let res = run_scenario(
        1,
        FaultSchedule::new(
            vec![],
            vec![Crash {
                node: "coord".to_string(),
                start: 0,
                end: 50,
            }],
        ),
    );
    assert!(!res.diverged, "crashing the coordinator should not diverge");
    assert_eq!(res.values["r1"], 0);
    assert_eq!(res.values["r2"], 0);
}

// --- Cross-language exactness checks against the Python reference stream. ---

#[test]
fn rng_next_u64_matches_reference() {
    let mut r = Rng::new(12345);
    let want: [u64; 5] = [
        2454886589211414944,
        3778200017661327597,
        2205171434679333405,
        3248800117070709450,
        9350289611492784363,
    ];
    for (i, w) in want.iter().enumerate() {
        assert_eq!(r.next_u64(), *w, "next_u64[{i}]");
    }
}

#[test]
fn rng_below_matches_reference() {
    let mut r = Rng::new(1);
    let want: [i64; 5] = [0, 4, 0, 0, 1];
    for (i, w) in want.iter().enumerate() {
        assert_eq!(r.below(5), *w, "below(5)[{i}]");
    }
}

#[test]
fn rng_between_matches_reference() {
    let mut r = Rng::new(1);
    let want: [i64; 5] = [84, 161, 197, 150, 86];
    for (i, w) in want.iter().enumerate() {
        assert_eq!(r.between(20, 200), *w, "between(20,200)[{i}]");
    }
}

#[test]
fn rng_chance_matches_reference() {
    let mut r = Rng::new(1);
    let want: [bool; 8] = [true, false, false, true, true, false, false, true];
    for (i, w) in want.iter().enumerate() {
        assert_eq!(r.chance(0.6), *w, "chance(0.6)[{i}]");
    }
}

#[test]
#[should_panic(expected = "below")]
fn below_panics_on_zero() {
    Rng::new(1).below(0);
}

#[test]
#[should_panic(expected = "below")]
fn below_panics_on_negative() {
    Rng::new(1).below(-1);
}

#[test]
#[should_panic(expected = "between")]
fn between_panics_when_hi_less_than_lo() {
    Rng::new(1).between(5, 4);
}

#[test]
fn between_is_inclusive_on_a_single_point() {
    let mut r = Rng::new(99);
    for _ in 0..100 {
        assert_eq!(r.between(7, 7), 7);
    }
}

#[test]
fn chance_boundaries_consume_no_draw() {
    let mut r = Rng::new(1);
    assert!(!r.chance(0.0), "chance(0) must be false");
    assert!(r.chance(1.0), "chance(1) must be true");
    assert!(!r.chance(-3.0), "chance(negative) must be false");
    assert!(r.chance(2.0), "chance(>1) must be true");
    // None of the above should have advanced the stream.
    assert_eq!(r.next_u64(), Rng::new(1).next_u64());
}

#[test]
fn scenario_trace_matches_reference() {
    let res = run_scenario(
        12345,
        FaultSchedule::new(
            vec![Partition {
                a: "coord".to_string(),
                b: "r2".to_string(),
                start: 0,
                end: 50,
            }],
            vec![],
        ),
    );
    let want = vec![
        entry(0, "drop", "dst=r2,kind=set,src=coord"),
        entry(5, "deliver", "dst=r1,kind=set,src=coord"),
    ];
    assert_eq!(res.trace, want);
    assert!(
        res.diverged,
        "seed 12345 with the r2 partition should diverge"
    );
    assert_eq!(res.values["r1"], 1);
    assert_eq!(res.values["r2"], 0);
}

#[test]
fn no_fault_scenario_trace_matches_reference() {
    let res = run_scenario(1, FaultSchedule::empty());
    let want = vec![
        entry(6, "deliver", "dst=r1,kind=set,src=coord"),
        entry(10, "deliver", "dst=r2,kind=set,src=coord"),
    ];
    assert_eq!(res.trace, want);
}

#[test]
fn random_faults_matches_reference() {
    let got = random_faults(&mut Rng::new(1), None);
    let want = vec![
        Fault::Partition(Partition {
            a: "coord".to_string(),
            b: "r2".to_string(),
            start: 4,
            end: 201,
        }),
        Fault::Partition(Partition {
            a: "r1".to_string(),
            b: "coord".to_string(),
            start: 0,
            end: 191,
        }),
    ];
    assert_eq!(got, want);
}

#[test]
fn search_for_divergence_matches_reference() {
    let found = search_for_divergence(0i64..500).expect("expected a diverging seed in 0..499");
    assert_eq!(found.seed, 3, "first diverging seed");
    let want = vec![Fault::Partition(Partition {
        a: "coord".to_string(),
        b: "r1".to_string(),
        start: 1,
        end: 90,
    })];
    assert_eq!(found.faults, want);
}
