"""seed-sim: deterministic simulation testing for distributed code.

Run message-passing systems inside one deterministic thread, replay any run from
a single 64-bit seed, and shrink a failing fault schedule down to a minimal
reproducer.
"""

from .faults import Crash, FaultSchedule, Partition
from .rng import Rng
from .shrink import ddmin
from .sim import Node, Simulator

__all__ = [
    "Rng",
    "Simulator",
    "Node",
    "FaultSchedule",
    "Partition",
    "Crash",
    "ddmin",
]

__version__ = "0.1.0"
