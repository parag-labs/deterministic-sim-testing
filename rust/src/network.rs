//! The message-passing layer, where latency and loss are decided.
//!
//! Delay and (optional) random drop are drawn from the simulator's seeded `Rng`,
//! and partitions/crashes are consulted from the fault schedule - both at send
//! time and again at delivery time, because a node can crash while a message is in
//! flight. Keeping this in one place is what lets the whole run stay deterministic.

use crate::sim::{Msg, MsgValue, Simulator};

/// The simulated message layer for a `Simulator`.
pub struct Network {
    pub min_delay: i64,
    pub max_delay: i64,
    pub drop_prob: f64,
}

impl Network {
    /// Creates a network with the default latency window `[1, 10]` and no loss.
    pub fn new() -> Self {
        Network {
            min_delay: 1,
            max_delay: 10,
            drop_prob: 0.0,
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Network::new()
    }
}

impl Simulator {
    /// Routes a message from `src` to `dst`, applying latency, loss, and faults.
    pub fn send(&mut self, src: &str, dst: &str, msg: Msg) {
        let now = self.now();
        let kind = kind_of(&msg);

        // Blocked before it ever leaves: partition, or either end down.
        if self.faults.is_partitioned(src, dst, now)
            || self.faults.is_crashed(src, now)
            || self.faults.is_crashed(dst, now)
        {
            self.record("drop", &drop_fields(src, dst, kind));
            return;
        }

        // Only consume an rng draw for loss when loss is actually configured, so
        // enabling drop_prob doesn't silently shift the delay stream.
        if self.net.drop_prob > 0.0 && self.rng.chance(self.net.drop_prob) {
            self.record("drop", &drop_fields(src, dst, kind));
            return;
        }

        let delay = self.rng.between(self.net.min_delay, self.net.max_delay);
        let src_owned = src.to_string();
        let dst_owned = dst.to_string();
        self.at(
            delay,
            Box::new(move |sim: &mut Simulator| {
                let kind = kind_of(&msg);
                // Re-check at arrival: the link may have partitioned or the
                // destination may have crashed while the message was in flight.
                if sim.faults.is_partitioned(&src_owned, &dst_owned, sim.now())
                    || sim.faults.is_crashed(&dst_owned, sim.now())
                {
                    sim.record("drop", &drop_fields(&src_owned, &dst_owned, kind));
                    return;
                }
                sim.record("deliver", &drop_fields(&src_owned, &dst_owned, kind));
                if let Some(node) = sim.nodes.get(&dst_owned).cloned() {
                    node.borrow_mut().on_message(sim, &src_owned, &msg);
                }
            }),
        );
    }
}

fn kind_of(msg: &Msg) -> MsgValue {
    msg.get("kind")
        .cloned()
        .unwrap_or_else(|| MsgValue::Str("None".to_string()))
}

fn drop_fields<'a>(src: &'a str, dst: &'a str, kind: MsgValue) -> [(&'a str, MsgValue); 3] {
    [
        ("src", MsgValue::Str(src.to_string())),
        ("dst", MsgValue::Str(dst.to_string())),
        ("kind", kind),
    ]
}
