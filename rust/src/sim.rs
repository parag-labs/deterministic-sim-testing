//! The simulator core: a single-threaded discrete-event loop over virtual time.
//!
//! Everything that could introduce nondeterminism in a real distributed system -
//! the clock, message latency, message loss, the order concurrent events fire in -
//! is funnelled through this one loop and the seeded `Rng`. There are no real
//! threads and no wall-clock reads, so a given (seed, workload, fault schedule)
//! always produces the exact same event trace. That's the property the tests lean
//! on.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::rc::Rc;

use crate::faults::FaultSchedule;
use crate::network::Network;
use crate::rng::Rng;

/// A single message field value: either a string or an integer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MsgValue {
    Str(String),
    Int(i64),
}

/// A message is an ordered map of string keys to values.
pub type Msg = BTreeMap<String, MsgValue>;

/// One ordered, comparable line of the event trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceEntry {
    pub time: i64,
    pub event: String,
    /// Fields sorted by key and rendered `key=value`, joined with commas.
    pub fields: String,
}

struct Event {
    time: i64,
    seq: u64,
    action: Box<dyn FnOnce(&mut Simulator)>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.seq == other.seq
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed so the max-heap `BinaryHeap` pops the smallest (time, seq)
        // first. Ties break on insertion order, keeping concurrent events
        // deterministic.
        other
            .time
            .cmp(&self.time)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

/// A participant in the simulation. Implementors react to messages and timers;
/// they never touch time or the network directly except through the simulator
/// handed to each callback.
pub trait Node {
    /// The node's identifier.
    fn id(&self) -> &str;

    /// Handles an incoming message. The default is a no-op.
    fn on_message(&mut self, _sim: &mut Simulator, _src: &str, _msg: &Msg) {}

    /// Handles a fired timer. The default is a no-op.
    fn on_timer(&mut self, _sim: &mut Simulator, _name: &str, _payload: Option<i64>) {}
}

/// A deterministic discrete-event loop over integer virtual time.
pub struct Simulator {
    pub seed: i64,
    pub rng: Rng,
    pub faults: FaultSchedule,
    pub net: Network,
    pub nodes: HashMap<String, Rc<RefCell<dyn Node>>>,
    pub trace: Vec<TraceEntry>,
    now: i64,
    heap: BinaryHeap<Event>,
    seq: u64,
}

impl Simulator {
    /// Creates a simulator for the given seed and fault schedule.
    pub fn new(seed: i64, faults: FaultSchedule) -> Self {
        Simulator {
            seed,
            rng: Rng::new(seed),
            faults,
            net: Network::new(),
            nodes: HashMap::new(),
            trace: Vec::new(),
            now: 0,
            heap: BinaryHeap::new(),
            seq: 0,
        }
    }

    /// The current virtual time.
    pub fn now(&self) -> i64 {
        self.now
    }

    /// Registers a node with the simulator.
    pub fn add(&mut self, node: Rc<RefCell<dyn Node>>) {
        let id = node.borrow().id().to_string();
        self.nodes.insert(id, node);
    }

    /// Schedules `action` to run `delay` ticks from now. Ties break on insertion
    /// order, keeping concurrent events deterministic. Panics if `delay < 0`.
    pub fn at(&mut self, delay: i64, action: Box<dyn FnOnce(&mut Simulator)>) {
        assert!(delay >= 0, "delay must be >= 0");
        let time = self.now + delay;
        self.heap.push(Event {
            time,
            seq: self.seq,
            action,
        });
        self.seq += 1;
    }

    /// Schedules a named timer on `node_id` to fire `delay` ticks from now. A
    /// crashed node's timers are silently swallowed when they would fire.
    pub fn timer(&mut self, node_id: &str, delay: i64, name: &str, payload: Option<i64>) {
        let id = node_id.to_string();
        let name = name.to_string();
        self.at(
            delay,
            Box::new(move |sim: &mut Simulator| {
                if sim.faults.is_crashed(&id, sim.now()) {
                    return;
                }
                sim.record(
                    "timer",
                    &[
                        ("node", MsgValue::Str(id.clone())),
                        ("name", MsgValue::Str(name.clone())),
                    ],
                );
                if let Some(node) = sim.nodes.get(&id).cloned() {
                    node.borrow_mut().on_timer(sim, &name, payload);
                }
            }),
        );
    }

    /// Appends a fully ordered, comparable line to the trace. Fields are sorted by
    /// key so two runs can be compared for exact equality.
    pub fn record(&mut self, event: &str, fields: &[(&str, MsgValue)]) {
        let mut sorted: Vec<(&str, String)> =
            fields.iter().map(|(k, v)| (*k, render_value(v))).collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        let rendered = sorted
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(",");
        self.trace.push(TraceEntry {
            time: self.now,
            event: event.to_string(),
            fields: rendered,
        });
    }

    /// Drains the event loop until it empties, until virtual time would pass
    /// `until` (`None` for no limit), or until `max_steps` events have fired.
    pub fn run(&mut self, until: Option<i64>, max_steps: u64) -> &[TraceEntry] {
        let mut steps: u64 = 0;
        while !self.heap.is_empty() && steps < max_steps {
            if let Some(limit) = until {
                if self.heap.peek().unwrap().time > limit {
                    break;
                }
            }
            let ev = self.heap.pop().unwrap();
            self.now = ev.time;
            (ev.action)(self);
            steps += 1;
        }
        &self.trace
    }
}

fn render_value(v: &MsgValue) -> String {
    match v {
        MsgValue::Str(s) => s.clone(),
        MsgValue::Int(i) => i.to_string(),
    }
}
