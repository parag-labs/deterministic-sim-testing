from seedsim.faults import FaultSchedule, Partition
from examples.replicated_flag import run_scenario


def test_same_seed_same_trace():
    faults = FaultSchedule([Partition("coord", "r2", 0, 50)])
    t1, _, _ = run_scenario(12345, faults)
    t2, _, _ = run_scenario(12345, faults)
    assert t1 == t2, "a run must be a pure function of (seed, faults)"


def test_different_seeds_can_diverge_in_trace():
    # With random network latency, distinct seeds should not all collapse to the
    # same trace - otherwise the seed isn't actually driving anything.
    traces = {tuple(run_scenario(s, FaultSchedule())[0]) for s in range(20)}
    assert len(traces) >= 2


def test_isolating_a_replica_diverges():
    _, diverged, values = run_scenario(1, FaultSchedule([Partition("coord", "r2", 0, 50)]))
    assert diverged
    assert values["r1"] != values["r2"]


def test_no_faults_stays_consistent():
    _, diverged, values = run_scenario(1, FaultSchedule())
    assert not diverged
    assert values == {"r1": 1, "r2": 1}


def test_crashing_the_coordinator_does_not_diverge():
    # Both sends are dropped, so the replicas agree (both still 0) - a real but
    # different outcome from isolating a single replica.
    from seedsim.faults import Crash

    _, diverged, values = run_scenario(1, FaultSchedule(crashes=[Crash("coord", 0, 50)]))
    assert not diverged
    assert values == {"r1": 0, "r2": 0}
