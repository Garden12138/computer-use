"""Bounded, seedable action pacing. Helper stays a dumb executor."""

from __future__ import annotations

import time
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass
from random import Random

SleepFn = Callable[[float], None]
MonotonicFn = Callable[[], float]
_SECONDS_PER_MINUTE = 60.0

INPUT_COMMANDS = frozenset(
    {
        "move",
        "click",
        "double_click",
        "scroll",
        "type",
        "key",
        "hotkey",
        "drag",
        "wait",
        "focus_app",
        "focus_window",
        "browser_open_profile",
        "browser_open_url",
        "browser_save_page",
    }
)


@dataclass(frozen=True, slots=True)
class MillisRange:
    min_ms: int
    max_ms: int


@dataclass(frozen=True, slots=True)
class ScrollRange:
    min_distance: int
    max_distance: int
    settle: MillisRange


@dataclass(frozen=True, slots=True)
class PacingConfig:
    name: str
    action_interval: MillisRange
    scroll: ScrollRange
    move_duration: MillisRange
    type_interval: MillisRange
    settle: MillisRange
    pixel_jitter: int
    max_actions_per_minute: int
    long_rest_every_n_actions: int
    long_rest: MillisRange


def profile_config(name: str) -> PacingConfig:
    key = name.strip().lower()
    if key in {"off", "none", "0"}:
        return PacingConfig(
            name="off",
            action_interval=MillisRange(0, 0),
            scroll=ScrollRange(0, 0, MillisRange(0, 0)),
            move_duration=MillisRange(0, 0),
            type_interval=MillisRange(0, 0),
            settle=MillisRange(0, 0),
            pixel_jitter=0,
            max_actions_per_minute=10_000,
            long_rest_every_n_actions=0,
            long_rest=MillisRange(0, 0),
        )
    if key == "conservative":
        return PacingConfig(
            name="conservative",
            action_interval=MillisRange(700, 2400),
            scroll=ScrollRange(120, 320, MillisRange(1100, 2800)),
            move_duration=MillisRange(180, 420),
            type_interval=MillisRange(40, 110),
            settle=MillisRange(1100, 2800),
            pixel_jitter=3,
            max_actions_per_minute=18,
            long_rest_every_n_actions=12,
            long_rest=MillisRange(8000, 20000),
        )
    if key != "normal":
        raise ValueError(f"unknown pacing profile: {name}")
    return PacingConfig(
        name="normal",
        action_interval=MillisRange(350, 1200),
        scroll=ScrollRange(180, 460, MillisRange(600, 1600)),
        move_duration=MillisRange(120, 280),
        type_interval=MillisRange(25, 80),
        settle=MillisRange(600, 1600),
        pixel_jitter=3,
        max_actions_per_minute=40,
        long_rest_every_n_actions=20,
        long_rest=MillisRange(4000, 10000),
    )


class Pacer:
    """Sample jittered waits and geometry for one session."""

    def __init__(
        self,
        config: PacingConfig,
        *,
        rng: Random | None = None,
        sleep: SleepFn = time.sleep,
        monotonic: MonotonicFn = time.monotonic,
    ) -> None:
        self.config = config
        self._rng = rng if rng is not None else Random()
        self._sleep = sleep
        self._monotonic = monotonic
        self._action_times: deque[float] = deque()
        self.slept_ms: list[float] = []
        self._actions = 0

    @property
    def enabled(self) -> bool:
        return self.config.name != "off"

    def sample_ms(self, window: MillisRange) -> float:
        if window.max_ms == window.min_ms:
            return float(window.min_ms)
        return self._rng.uniform(float(window.min_ms), float(window.max_ms))

    def scroll_distance(self) -> int:
        scroll = self.config.scroll
        if scroll.min_distance == scroll.max_distance:
            return scroll.min_distance
        return self._rng.randint(scroll.min_distance, scroll.max_distance)

    def jitter_point(self, x: float, y: float) -> tuple[float, float]:
        j = self.config.pixel_jitter
        if j <= 0:
            return x, y
        return x + self._rng.randint(-j, j), y + self._rng.randint(-j, j)

    def before_action(self, cmd: str) -> None:
        if not self.enabled or cmd not in INPUT_COMMANDS:
            return
        self._enforce_rate_limit()
        self._sleep_ms(self.sample_ms(self.config.action_interval))
        self._action_times.append(self._monotonic())
        self._actions += 1
        every = self.config.long_rest_every_n_actions
        if every and self._actions % every == 0:
            self._sleep_ms(self.sample_ms(self.config.long_rest))

    def after_action(self, cmd: str) -> None:
        if not self.enabled or cmd not in INPUT_COMMANDS:
            return
        window = self.config.scroll.settle if cmd == "scroll" else self.config.settle
        self._sleep_ms(self.sample_ms(window))

    def decorate(self, cmd: str, params: dict) -> dict:
        """Fill duration/interval/delta when the caller omitted them."""

        if not self.enabled:
            return params
        out = dict(params)
        if cmd in {"click", "double_click", "move", "drag"}:
            if "duration" not in out:
                out["duration"] = self.sample_ms(self.config.move_duration) / 1000.0
            if "x" in out and "y" in out:
                x, y = self.jitter_point(float(out["x"]), float(out["y"]))
                out["x"], out["y"] = x, y
            if cmd == "drag" and "end_x" in out and "end_y" in out:
                ex, ey = self.jitter_point(float(out["end_x"]), float(out["end_y"]))
                out["end_x"], out["end_y"] = ex, ey
        if cmd == "type" and "interval" not in out:
            out["interval"] = self.sample_ms(self.config.type_interval) / 1000.0
        if cmd == "scroll" and "delta_y" not in out and "delta_x" not in out:
            out["delta_y"] = -self.scroll_distance()
            out["delta_x"] = 0
        elif cmd == "scroll" and self.config.scroll.min_distance:
            # Keep sign, replace magnitude when caller passed a placeholder 0.
            if int(out.get("delta_y") or 0) == 0 and int(out.get("delta_x") or 0) == 0:
                out["delta_y"] = -self.scroll_distance()
        return out

    def _enforce_rate_limit(self) -> None:
        now = self._monotonic()
        self._prune(now)
        cap = self.config.max_actions_per_minute
        if len(self._action_times) < cap:
            return
        wait_seconds = max(0.0, self._action_times[0] + _SECONDS_PER_MINUTE - now)
        if wait_seconds > 0:
            self._sleep(wait_seconds)
        self._prune(self._monotonic())

    def _prune(self, now: float) -> None:
        window = self._action_times
        while window and now - window[0] > _SECONDS_PER_MINUTE:
            window.popleft()

    def _sleep_ms(self, milliseconds: float) -> None:
        if milliseconds <= 0:
            return
        self.slept_ms.append(milliseconds)
        self._sleep(milliseconds / 1000.0)
