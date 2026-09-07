from seedsim.faults import Crash, FaultSchedule
from seedsim.shrink import ddmin
from examples.replicated_flag import run_scenario, search_for_divergence


def test_search_finds_a_diverging_schedule():
    found = search_for_divergence(range(500))
    assert found is not None, "expected some seed in 0..499 to diverge"
    seed, faults = found
    _, diverged, _ = run_scenario(seed, FaultSchedule.of(faults))
    assert diverged


def test_ddmin_strips_the_irrelevant_faults():
    found = search_for_divergence(range(500))
    assert found is not None
    seed, faults = found

    # Pad the real reproducer with two faults that happen long after everything
    # is over - they can't possibly matter.
    noise = [Crash("r1", 900, 950), Crash("r2", 800, 850)]
    padded = list(faults) + noise

    def still_fails(subset) -> bool:
        return run_scenario(seed, FaultSchedule.of(subset))[1]

    minimal = ddmin(padded, still_fails)

    assert still_fails(minimal)
    assert len(minimal) < len(padded)
    # The late-firing noise must be gone from the minimal reproducer.
    for n in noise:
        assert n not in minimal


def test_ddmin_requires_a_failing_input():
    import pytest

    def never_fails(_subset) -> bool:
        return False

    with pytest.raises(ValueError):
        ddmin([1, 2, 3], never_fails)
