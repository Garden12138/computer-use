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
