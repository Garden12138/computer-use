from computer_use.protocol import COMMANDS, error_payload, request_payload, success_payload


def test_request_payload_includes_cmd_and_id() -> None:
    body = request_payload("click", {"x": 10, "y": 20}, request_id="abc")
    assert body["id"] == "abc"
    assert body["cmd"] == "click"
    assert body["x"] == 10
    assert body["y"] == 20


def test_unknown_command_is_rejected() -> None:
    try:
        request_payload("evaluate_js")
    except ValueError as exc:
        assert "unknown command" in str(exc)
    else:
        raise AssertionError("expected ValueError")


def test_success_and_error_shapes() -> None:
    ok = success_payload("1", {"width": 100})
    assert ok == {"id": "1", "ok": True, "data": {"width": 100}}
    err = error_payload("1", "permission_denied", "grant Accessibility")
    assert err["ok"] is False
    assert err["error"]["code"] == "permission_denied"


def test_v01_command_set() -> None:
    assert "screenshot" in COMMANDS
    assert "cursor" in COMMANDS
    assert "browser_save_page" in COMMANDS
    assert "querySelector" not in COMMANDS
