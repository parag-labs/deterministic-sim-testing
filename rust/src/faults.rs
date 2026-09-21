//! Fault schedules: the injected adversity a run is subjected to.
//!
//! A fault is a plain, comparable value with a time window. The schedule is part
//! of the reproducer (seed + faults fully determine a run), and because faults are
//! value types they can be fed straight into the shrinker as opaque items.

/// The link between two nodes is down for `[start, end)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partition {
    pub a: String,
    pub b: String,
    pub start: i64,
    pub end: i64,
}

/// `node` is down for `[start, end)`: it sends nothing and receives nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crash {
    pub node: String,
    pub start: i64,
    pub end: i64,
}

/// A fault a run can be subjected to (a partition or a crash).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Partition(Partition),
    Crash(Crash),
}

/// The full set of partitions and crashes applied to a run.
#[derive(Clone, Debug, Default)]
pub struct FaultSchedule {
    pub partitions: Vec<Partition>,
    pub crashes: Vec<Crash>,
}

impl FaultSchedule {
    /// Builds a schedule from the given partitions and crashes.
    pub fn new(partitions: Vec<Partition>, crashes: Vec<Crash>) -> Self {
        FaultSchedule {
            partitions,
            crashes,
        }
    }

    /// An empty schedule (no faults).
    pub fn empty() -> Self {
        FaultSchedule::default()
    }

    /// Reports whether the (unordered) link between `a` and `b` is down at `t`.
    pub fn is_partitioned(&self, a: &str, b: &str, t: i64) -> bool {
        self.partitions.iter().any(|p| {
            (p.start..p.end).contains(&t) && ((p.a == a && p.b == b) || (p.a == b && p.b == a))
        })
    }

    /// Reports whether `node` is down at `t`.
    pub fn is_crashed(&self, node: &str, t: i64) -> bool {
        self.crashes
            .iter()
            .any(|c| c.node == node && (c.start..c.end).contains(&t))
    }

    /// Returns every fault in the schedule (partitions first, then crashes).
    pub fn items(&self) -> Vec<Fault> {
        self.partitions
            .iter()
            .cloned()
            .map(Fault::Partition)
            .chain(self.crashes.iter().cloned().map(Fault::Crash))
            .collect()
    }

    /// Partitions a flat list of faults back into a schedule.
    pub fn of(items: &[Fault]) -> Self {
        let mut partitions = Vec::new();
        let mut crashes = Vec::new();
        for item in items {
            match item {
                Fault::Partition(p) => partitions.push(p.clone()),
                Fault::Crash(c) => crashes.push(c.clone()),
            }
        }
        FaultSchedule {
            partitions,
            crashes,
        }
    }
}
