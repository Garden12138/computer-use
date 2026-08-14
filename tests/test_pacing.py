from random import Random

from computer_use.pacing import Pacer, profile_config


class _RecordingSleep:
    def __init__(self) -> None:
        self.calls: list[float] = []

    def __call__(self, seconds: float) -> None:
        self.calls.append(seconds)


def _pacer(name: str = "conservative", seed: int = 1234) -> tuple[Pacer, _RecordingSleep]:
    sleep = _RecordingSleep()
    pacer = Pacer(profile_config(name), rng=Random(seed), sleep=sleep)
    return pacer, sleep


def test_action_intervals_are_jittered_within_bounds_not_constant() -> None:
    pacer, _sleep = _pacer()
    for _ in range(10):
        pacer.before_action("click")
    cfg = pacer.config.action_interval
    assert all(cfg.min_ms <= value <= cfg.max_ms for value in pacer.slept_ms)
    assert len({round(value, 3) for value in pacer.slept_ms}) > 1


def test_fixed_seed_makes_pacing_reproducible() -> None:
    first, _ = _pacer("normal", seed=7)
    second, _ = _pacer("normal", seed=7)
    for _ in range(10):
        first.before_action("click")
        second.before_action("click")
    assert first.slept_ms == second.slept_ms


def test_scroll_distance_is_within_bounds() -> None:
    pacer, _ = _pacer()
    distances = {pacer.scroll_distance() for _ in range(30)}
    scroll = pacer.config.scroll
    assert distances
    assert all(scroll.min_distance <= value <= scroll.max_distance for value in distances)
    assert len(distances) > 1


def test_off_profile_does_not_sleep() -> None:
    pacer, sleep = _pacer("off")
    pacer.before_action("click")
    pacer.after_action("click")
    assert sleep.calls == []
    assert pacer.decorate("click", {"x": 10, "y": 10}) == {"x": 10, "y": 10}


def test_observe_commands_are_not_paced() -> None:
    pacer, sleep = _pacer()
    pacer.before_action("screenshot")
    pacer.after_action("list_windows")
    assert sleep.calls == []
