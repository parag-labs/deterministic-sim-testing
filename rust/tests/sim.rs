//! Direct simulator tests: virtual-time ordering and the timer path.

use std::cell::RefCell;
use std::rc::Rc;

use deterministic_sim_testing::{Crash, FaultSchedule, Node, Simulator};

struct Recorder {
    id: String,
    fired: Vec<String>,
}

impl Recorder {
    fn new(id: &str) -> Self {
        Recorder {
            id: id.to_string(),
            fired: Vec::new(),
        }
    }
}

impl Node for Recorder {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_timer(&mut self, _sim: &mut Simulator, name: &str, payload: Option<i64>) {
        self.fired.push(format!("{name}:{payload:?}"));
    }
}

#[test]
fn timer_fires_and_records() {
    let mut sim = Simulator::new(1, FaultSchedule::empty());
    let node = Rc::new(RefCell::new(Recorder::new("n1")));
    sim.add(node.clone());
    sim.timer("n1", 5, "tick", Some(42));
    sim.run(None, 1_000_000);

    assert_eq!(node.borrow().fired, vec!["tick:Some(42)".to_string()]);
    assert_eq!(sim.trace.len(), 1);
    assert_eq!(sim.trace[0].time, 5);
    assert_eq!(sim.trace[0].event, "timer");
    assert_eq!(sim.trace[0].fields, "name=tick,node=n1");
}

#[test]
fn timer_on_crashed_node_is_swallowed() {
    let faults = FaultSchedule::new(
        vec![],
        vec![Crash {
            node: "n1".to_string(),
            start: 0,
            end: 100,
        }],
    );
    let mut sim = Simulator::new(1, faults);
    let node = Rc::new(RefCell::new(Recorder::new("n1")));
    sim.add(node.clone());
    sim.timer("n1", 5, "tick", None);
    sim.run(None, 1_000_000);

    assert!(node.borrow().fired.is_empty(), "crashed node fires nothing");
    assert!(sim.trace.is_empty(), "crashed timer records nothing");
}

#[test]
fn events_fire_in_time_then_insertion_order() {
    let mut sim = Simulator::new(1, FaultSchedule::empty());
    let order = Rc::new(RefCell::new(Vec::<i64>::new()));

    let o1 = order.clone();
    sim.at(5, Box::new(move |_| o1.borrow_mut().push(1)));
    let o2 = order.clone();
    sim.at(1, Box::new(move |_| o2.borrow_mut().push(2)));
    let o3 = order.clone();
    sim.at(5, Box::new(move |_| o3.borrow_mut().push(3)));

    sim.run(None, 1_000_000);
    assert_eq!(*order.borrow(), vec![2, 1, 3]);
}

#[test]
fn run_stops_at_until_horizon() {
    let mut sim = Simulator::new(1, FaultSchedule::empty());
    let count = Rc::new(RefCell::new(0));

    let c1 = count.clone();
    sim.at(5, Box::new(move |_| *c1.borrow_mut() += 1));
    let c2 = count.clone();
    sim.at(50, Box::new(move |_| *c2.borrow_mut() += 1));

    sim.run(Some(10), 1_000_000);
    assert_eq!(*count.borrow(), 1, "event past the horizon must not fire");
}

#[test]
#[should_panic(expected = "delay")]
fn at_panics_on_negative_delay() {
    let mut sim = Simulator::new(1, FaultSchedule::empty());
    sim.at(-1, Box::new(|_| {}));
}
