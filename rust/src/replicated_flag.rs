//! A deliberately buggy workload, used to show deterministic-sim-testing finding
//! and shrinking a bug.
//!
//! A coordinator sets a flag on two replicas with a single fire-and-forget message
//! each - and, crucially, never retries. That missing retry is the bug: if the
//! message to one replica is lost, the replicas diverge forever and nobody notices.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::faults::{Crash, Fault, FaultSchedule, Partition};
use crate::rng::Rng;
use crate::sim::{Msg, MsgValue, Node, Simulator};

/// The participants in the worked example.
pub const NODES: [&str; 3] = ["coord", "r1", "r2"];

/// The virtual time the scenario runs until.
pub const HORIZON: i64 = 1000;

/// Sets a flag on each replica exactly once and never retries.
pub struct Coordinator {
    id: String,
    replicas: Vec<String>,
}

impl Coordinator {
    /// Builds the coordinator for the given replica ids.
    pub fn new(replicas: Vec<String>) -> Self {
        Coordinator {
            id: "coord".to_string(),
            replicas,
        }
    }

    /// Fires the coordinator's one-shot writes. The bug in one line: send once,
    /// never confirm, never retry.
    pub fn start(&self, sim: &mut Simulator) {
        for r in &self.replicas {
            let mut msg = Msg::new();
            msg.insert("kind".to_string(), MsgValue::Str("set".to_string()));
            msg.insert("value".to_string(), MsgValue::Int(1));
            sim.send(&self.id, r, msg);
        }
    }
}

impl Node for Coordinator {
    fn id(&self) -> &str {
        &self.id
    }
}

/// Stores a single flag value updated by "set" messages.
pub struct Replica {
    id: String,
    pub value: i64,
}

impl Replica {
    /// Builds a replica with the given id and an initial value of 0.
    pub fn new(id: &str) -> Self {
        Replica {
            id: id.to_string(),
            value: 0,
        }
    }
}

impl Node for Replica {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_message(&mut self, _sim: &mut Simulator, _src: &str, msg: &Msg) {
        if matches!(msg.get("kind"), Some(MsgValue::Str(k)) if k == "set") {
            if let Some(MsgValue::Int(v)) = msg.get("value") {
                self.value = *v;
            }
        }
    }
}

/// The outcome of a single scenario run.
#[derive(Clone, Debug)]
pub struct ScenarioResult {
    pub trace: Vec<crate::sim::TraceEntry>,
    pub diverged: bool,
    pub values: BTreeMap<String, i64>,
}

/// A seed together with the fault schedule that made it diverge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub seed: i64,
    pub faults: Vec<Fault>,
}

/// Runs the workload once and reports the trace, whether the replicas diverged,
/// and their final values.
pub fn run_scenario(seed: i64, faults: FaultSchedule) -> ScenarioResult {
    let mut sim = Simulator::new(seed, faults);
    let r1 = Rc::new(RefCell::new(Replica::new("r1")));
    let r2 = Rc::new(RefCell::new(Replica::new("r2")));
    sim.add(r1.clone());
    sim.add(r2.clone());
    let coord = Rc::new(RefCell::new(Coordinator::new(vec![
        "r1".to_string(),
        "r2".to_string(),
    ])));
    sim.add(coord.clone());

    let coord_start = coord.clone();
    sim.at(
        0,
        Box::new(move |sim: &mut Simulator| coord_start.borrow().start(sim)),
    );
    sim.run(Some(HORIZON), 1_000_000);

    let v1 = r1.borrow().value;
    let v2 = r2.borrow().value;
    let mut values = BTreeMap::new();
    values.insert("r1".to_string(), v1);
    values.insert("r2".to_string(), v2);
    ScenarioResult {
        trace: sim.trace.clone(),
        diverged: v1 != v2,
        values,
    }
}

/// Generates a small random fault schedule from a seeded `Rng`. Passing `None`
/// for `nodes` uses the default [`NODES`]. Windows are biased to start early
/// because all the interesting activity happens in the first few ticks.
pub fn random_faults(rng: &mut Rng, nodes: Option<&[&str]>) -> Vec<Fault> {
    let nodes = nodes.unwrap_or(&NODES);
    let mut out = Vec::new();
    let count = rng.between(1, 4);
    for _ in 0..count {
        let start = rng.below(5);
        let end = start + rng.between(20, 200);
        if rng.chance(0.6) {
            let a = nodes[rng.below(nodes.len() as i64) as usize];
            let b = nodes[rng.below(nodes.len() as i64) as usize];
            if a != b {
                out.push(Fault::Partition(Partition {
                    a: a.to_string(),
                    b: b.to_string(),
                    start,
                    end,
                }));
            }
        } else {
            let n = nodes[rng.below(nodes.len() as i64) as usize];
            out.push(Fault::Crash(Crash {
                node: n.to_string(),
                start,
                end,
            }));
        }
    }
    out
}

/// Returns the first (seed, faults) whose run diverges, or `None`.
pub fn search_for_divergence(seeds: impl IntoIterator<Item = i64>) -> Option<Found> {
    for seed in seeds {
        let faults = random_faults(&mut Rng::new(seed), None);
        if run_scenario(seed, FaultSchedule::of(&faults)).diverged {
            return Some(Found { seed, faults });
        }
    }
    None
}
