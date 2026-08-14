"""JSON-lines protocol shared by the Python client and Swift helper."""

from __future__ import annotations

from typing import Any, Mapping
from uuid import uuid4

COMMANDS = frozenset(
    {
        "screenshot",
        "move",
        "click",
        "double_click",
        "scroll",
        "type",
        "key",
        "hotkey",
        "drag",
        "wait",
        "list_windows",
        "focus_app",
        "focus_window",
        "get_screen_size",
        "get_active_window",
        "cursor",
        "doctor",
        "browser_open_profile",
        "browser_open_url",
        "browser_save_page",
    }
)

OBSERVE_COMMANDS = frozenset(
    {
        "screenshot",
        "list_windows",
        "get_screen_size",
        "get_active_window",
        "cursor",
        "doctor",
    }
)

ERROR_FOCUS_DENIED = "focus_denied"
ERROR_UNSUPPORTED = "unsupported"

DOCTOR_CAPABILITY_KEYS = (
    "screenshot",
    "window_screenshot",
    "move",
    "click",
    "scroll",
    "type",
    "list_windows",
    "focus_window",
)


def doctor_capabilities(
    *,
    screenshot: bool = True,
    window_screenshot: bool = True,
    move: bool = True,
    click: bool = True,
    scroll: bool = True,
    type: bool = True,
    list_windows: bool = True,
    focus_window: bool = True,
) -> dict[str, bool]:
    return {
        "screenshot": screenshot,
        "window_screenshot": window_screenshot,
        "move": move,
        "click": click,
        "scroll": scroll,
        "type": type,
        "list_windows": list_windows,
        "focus_window": focus_window,
    }


def validate_doctor_data(data: Mapping[str, Any]) -> None:
    """Raise ValueError if doctor payload is missing required capability fields."""

    for key in ("platform", "backend", "capabilities", "limitations", "ready"):
        if key not in data:
            raise ValueError(f"doctor missing {key}")
    backend = data["backend"]
    if not isinstance(backend, Mapping):
        raise ValueError("doctor backend must be an object")
    for key in ("input", "screen", "window"):
        if key not in backend:
            raise ValueError(f"doctor backend missing {key}")
    caps = data["capabilities"]
    if not isinstance(caps, Mapping):
        raise ValueError("doctor capabilities must be an object")
    for key in DOCTOR_CAPABILITY_KEYS:
        if key not in caps:
            raise ValueError(f"doctor capabilities missing {key}")
    if not isinstance(data["limitations"], list):
        raise ValueError("doctor limitations must be a list")


def request_payload(cmd: str, params: Mapping[str, Any] | None = None, *, request_id: str | None = None) -> dict[str, Any]:
    if cmd not in COMMANDS:
        raise ValueError(f"unknown command: {cmd}")
    body: dict[str, Any] = {"id": request_id or uuid4().hex, "cmd": cmd}
    if params:
        body.update(params)
    return body


def success_payload(request_id: str, data: Mapping[str, Any] | None = None) -> dict[str, Any]:
    return {"id": request_id, "ok": True, "data": dict(data or {})}


def error_payload(request_id: str, code: str, message: str, details: Mapping[str, Any] | None = None) -> dict[str, Any]:
    error: dict[str, Any] = {"code": code, "message": message}
    if details:
        error["details"] = dict(details)
    return {"id": request_id, "ok": False, "error": error}
