//! Rust port of deterministic-sim-testing: deterministic simulation testing for
//! distributed code.
//!
//! Run message-passing systems inside one deterministic thread, replay any run
//! from a single 64-bit seed, and shrink a failing fault schedule down to a
//! minimal reproducer. The engine - the splitmix64 PRNG, the discrete-event
//! simulator, the fault model, and ddmin shrinking - behaves identically to the
//! Python, C# and Java ports.

pub mod faults;
pub mod network;
pub mod replicated_flag;
pub mod rng;
pub mod shrink;
pub mod sim;

pub use faults::{Crash, Fault, FaultSchedule, Partition};
pub use replicated_flag::{
    random_faults, run_scenario, search_for_divergence, Coordinator, Found, Replica,
    ScenarioResult, HORIZON, NODES,
};
pub use rng::Rng;
pub use shrink::ddmin;
pub use sim::{Msg, MsgValue, Node, Simulator, TraceEntry};

/// Mirrors the reference implementation's `__version__`.
pub const VERSION: &str = "0.1.0";
