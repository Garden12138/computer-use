"""Compose paced helper calls for agents and the CLI."""

from __future__ import annotations

import os
from pathlib import Path
from random import Random
from typing import Any, Mapping

from computer_use.client import HelperClient
from computer_use.grid import overlay_grid
from computer_use.pacing import Pacer, profile_config


class Computer:
    def __init__(
        self,
        *,
        pacing: str | None = None,
        seed: int | None = None,
        client: HelperClient | None = None,
    ) -> None:
        name = pacing or os.environ.get("COMPUTER_USE_PACING") or "normal"
        rng = Random(seed) if seed is not None else None
        self.pacer = Pacer(profile_config(name), rng=rng)
        self.client = client or HelperClient()
        self._last_scale = 2.0

    def run(self, cmd: str, params: Mapping[str, Any] | None = None) -> dict[str, Any]:
        payload = dict(params or {})
        grid = bool(payload.pop("grid", False))
        grid_step = int(payload.pop("grid_step", 50) or 50)
        if "scale" not in payload:
            payload["scale"] = self._last_scale
        payload = self.pacer.decorate(cmd, payload)
        self.pacer.before_action(cmd)
        result = self.client.call(cmd, payload)
        self.pacer.after_action(cmd)
        data = result.get("data") if isinstance(result.get("data"), dict) else {}
        if cmd == "screenshot" and data.get("scale"):
            self._last_scale = float(data["scale"])
        if cmd == "screenshot" and grid and result.get("ok"):
            result = self._apply_grid(result, step=grid_step)
        return result

    def doctor(self) -> dict[str, Any]:
        return self.run("doctor")

    def screenshot(
        self,
        path: str | None = None,
        window_id: int | None = None,
        *,
        grid: bool = False,
        grid_step: int = 50,
    ) -> dict[str, Any]:
        params: dict[str, Any] = {"grid": grid, "grid_step": grid_step}
        if path:
            params["path"] = path
        if window_id is not None:
            params["window_id"] = window_id
        return self.run("screenshot", params)

    def _apply_grid(self, result: dict[str, Any], *, step: int) -> dict[str, Any]:
        data = dict(result.get("data") or {})
        path = data.get("path")
        if not path:
            return result
        origin_x = float(data.get("origin_x") or 0)
        origin_y = float(data.get("origin_y") or 0)
        if (origin_x == 0 and origin_y == 0) and data.get("window_id") is not None:
            origin_x, origin_y, bounds_w, bounds_h = self._window_origin(int(data["window_id"]))
        else:
            bounds_w = data.get("bounds_width")
            bounds_h = data.get("bounds_height")
        overlay = overlay_grid(
            str(path),
            origin_x=origin_x,
            origin_y=origin_y,
            scale=float(data.get("scale") or self._last_scale or 1),
            step=step,
            out_path=str(path),
            bounds_width=float(bounds_w) if bounds_w else None,
            bounds_height=float(bounds_h) if bounds_h else None,
        )
        data.update(overlay)
        data["grid"] = True
        result = dict(result)
        result["data"] = data
        return result

    def _window_origin(self, window_id: int) -> tuple[float, float, float | None, float | None]:
        listed = self.client.call("list_windows", {})
        windows = (listed.get("data") or {}).get("windows") if listed.get("ok") else None
        if not isinstance(windows, list):
            return 0.0, 0.0, None, None
        for window in windows:
            if int(window.get("window_id") or 0) != window_id:
                continue
            bounds = window.get("bounds") or {}
            return (
                float(bounds.get("x") or 0),
                float(bounds.get("y") or 0),
                float(bounds["width"]) if bounds.get("width") is not None else None,
                float(bounds["height"]) if bounds.get("height") is not None else None,
            )
        return 0.0, 0.0, None, None

    def click(self, x: float, y: float, **kwargs: Any) -> dict[str, Any]:
        return self.run("click", {"x": x, "y": y, **kwargs})

    def save_page(self, path: str, scrolls: int = 8) -> dict[str, Any]:
        """Slow jittered scroll then native Chrome Webpage, Complete save."""

        dest = Path(path)
        dest.parent.mkdir(parents=True, exist_ok=True)
        for _ in range(max(scrolls, 0)):
            result = self.run("scroll", {"delta_x": 0, "delta_y": 0})
            if not result.get("ok"):
                return result
        return self.run("browser_save_page", {"path": str(dest)})
