"""Minimal MCP stdio server (JSON-RPC + Content-Length framing)."""

from __future__ import annotations

import json
import sys
from typing import Any, Callable, Mapping

from computer_use.protocol import COMMANDS

TOOLS = [
    {
        "name": f"computer_{cmd}",
        "description": f"Native computer-use primitive: {cmd}",
        "inputSchema": {
            "type": "object",
            "properties": {
                "x": {"type": "number"},
                "y": {"type": "number"},
                "delta_x": {"type": "number"},
                "delta_y": {"type": "number"},
                "text": {"type": "string"},
                "key": {"type": "string"},
                "keys": {"type": "array", "items": {"type": "string"}},
                "app": {"type": "string"},
                "title": {"type": "string"},
                "path": {"type": "string"},
                "url": {"type": "string"},
                "profile": {"type": "string"},
                "window_id": {"type": "integer"},
                "grid": {"type": "boolean"},
                "grid_step": {"type": "integer"},
                "seconds": {"type": "number"},
                "duration": {"type": "number"},
                "button": {"type": "string"},
                "scrolls": {"type": "integer"},
            },
        },
    }
    for cmd in sorted(COMMANDS)
]

WriteFn = Callable[[dict[str, Any]], None]
ComputerFn = Callable[[], Any]


SUPPORTED_PROTOCOL_VERSIONS = (
    "2025-11-25",
    "2025-06-18",
    "2025-03-26",
    "2024-11-05",
    "2024-10-07",
)


def run_stdio(*, pacing: str | None = None, seed: int | None = None) -> None:
    computer_holder: list[Any] = []

    def get_computer() -> Any:
        if not computer_holder:
            from computer_use.runtime import Computer

            computer_holder.append(Computer(pacing=pacing, seed=seed))
        return computer_holder[0]

    while True:
        msg = _read_message()
        if msg is None:
            return
        if not msg:
            continue
        handle_message(msg, write=_write_message, get_computer=get_computer)


def handle_message(
    msg: Mapping[str, Any],
    *,
    write: WriteFn,
    computer: Any | None = None,
    get_computer: ComputerFn | None = None,
) -> None:
    method = msg.get("method")
    req_id = msg.get("id")
    if method == "initialize":
        requested = str((msg.get("params") or {}).get("protocolVersion") or "")
        version = requested if requested in SUPPORTED_PROTOCOL_VERSIONS else SUPPORTED_PROTOCOL_VERSIONS[0]
        write(
            {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": version,
                    "capabilities": {"tools": {"listChanged": False}},
                    "serverInfo": {"name": "computer-use", "version": "0.1.0"},
                },
            }
        )
        return
    if method == "ping":
        write({"jsonrpc": "2.0", "id": req_id, "result": {}})
        return
    if method == "notifications/initialized" or method == "notifications/cancelled":
        return
    if method == "tools/list":
        write({"jsonrpc": "2.0", "id": req_id, "result": {"tools": TOOLS}})
        return
    if method == "tools/call":
        params = msg.get("params") or {}
        name = str(params.get("name") or "")
        args = params.get("arguments") or {}
        cmd = name.removeprefix("computer_")
        if cmd not in COMMANDS:
            write(
                {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": [{"type": "text", "text": f"unknown tool {name}"}],
                        "isError": True,
                    },
                }
            )
            return
        runtime = computer if computer is not None else (get_computer() if get_computer else None)
        if runtime is None:
            write(
                {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": [{"type": "text", "text": "computer runtime is not available"}],
                        "isError": True,
                    },
                }
            )
            return
        if cmd == "browser_save_page":
            result = runtime.save_page(str(args.get("path")), scrolls=int(args.get("scrolls") or 8))
        else:
            result = runtime.run(cmd, args)
        write(
            {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {"content": _result_content(cmd, result), "isError": not result.get("ok", False)},
            }
        )
        return
    if req_id is not None:
        write({"jsonrpc": "2.0", "id": req_id, "error": {"code": -32601, "message": f"Unknown method {method}"}})


def _result_content(cmd: str, result: dict[str, Any]) -> list[dict[str, Any]]:
    # Keep MCP payloads small. Cursor times out if screenshot PNG is inlined as base64.
    _ = cmd
    return [{"type": "text", "text": json.dumps(result, ensure_ascii=False)}]


def _read_message(stream: Any | None = None) -> dict[str, Any] | None:
    stdin = stream if stream is not None else sys.stdin.buffer
    line = stdin.readline()
    if not line:
        return None
    if line in (b"\r\n", b"\n"):
        return {}
    stripped = line.lstrip()
    if stripped.startswith(b"{") or stripped.startswith(b"["):
        return json.loads(line)
    headers: dict[str, str] = {}
    decoded = line.decode("utf-8", "replace").strip()
    if ":" in decoded:
        key, value = decoded.split(":", 1)
        headers[key.strip().lower()] = value.strip()
    while True:
        nxt = stdin.readline()
        if not nxt:
            return None
        if nxt in (b"\r\n", b"\n"):
            break
        decoded = nxt.decode("utf-8", "replace").strip()
        if ":" in decoded:
            key, value = decoded.split(":", 1)
            headers[key.strip().lower()] = value.strip()
    length = int(headers.get("content-length") or "0")
    if length <= 0:
        return {}
    body = stdin.read(length)
    if not body:
        return None
    return json.loads(body.decode("utf-8"))


def _encode_message(msg: dict[str, Any]) -> bytes:
    # Cursor's MCP stdio client is newline-delimited JSON, not LSP Content-Length.
    return (json.dumps(msg, ensure_ascii=False) + "\n").encode("utf-8")


def _write_message(msg: dict[str, Any]) -> None:
    sys.stdout.buffer.write(_encode_message(msg))
    sys.stdout.buffer.flush()
