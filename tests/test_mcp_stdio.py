import io
import json

from computer_use.mcp_server import _read_message, _result_content, handle_message


def _framed(msg: dict) -> bytes:
    raw = json.dumps(msg).encode("utf-8")
    return f"Content-Length: {len(raw)}\r\n\r\n".encode("ascii") + raw


def test_read_content_length_frame() -> None:
    buf = io.BytesIO(_framed({"jsonrpc": "2.0", "id": 1, "method": "initialize"}))
    assert _read_message(buf)["method"] == "initialize"


def test_read_newline_json_does_not_hang() -> None:
    buf = io.BytesIO(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    assert _read_message(buf)["method"] == "notifications/initialized"


def test_empty_content_length_is_skip_not_eof() -> None:
    buf = io.BytesIO(b"Content-Length: 0\r\n\r\n")
    assert _read_message(buf) == {}


def test_eof_is_none() -> None:
    assert _read_message(io.BytesIO(b"")) is None


def test_write_message_is_newline_json_not_content_length() -> None:
    from computer_use.mcp_server import _encode_message

    raw = _encode_message({"jsonrpc": "2.0", "id": 1, "result": {}})
    assert raw.startswith(b"{")
    assert raw.endswith(b"\n")
    assert b"Content-Length" not in raw
    assert json.loads(raw.decode())["id"] == 1


def test_initialize_echoes_cursor_protocol_version() -> None:
    replies: list[dict] = []
    handle_message(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "cursor"}},
        },
        write=replies.append,
        computer=None,
    )
    assert replies[0]["result"]["protocolVersion"] == "2025-11-25"


def test_initialize_and_ping_do_not_need_helper() -> None:
    replies: list[dict] = []
    handle_message(
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        write=replies.append,
        computer=None,
    )
    handle_message(
        {"jsonrpc": "2.0", "id": 2, "method": "ping"},
        write=replies.append,
        computer=None,
    )
    assert replies[0]["result"]["serverInfo"]["name"] == "computer-use"
    assert replies[1]["result"] == {}


def test_screenshot_mcp_payload_is_text_only(tmp_path) -> None:
    shot = tmp_path / "x.png"
    shot.write_bytes(b"\x89PNG" + b"0" * 5000)
    parts = _result_content("screenshot", {"ok": True, "data": {"path": str(shot)}})
    assert parts[0]["type"] == "text"
    assert all(p["type"] != "image" for p in parts)
