from computer_use.protocol import (
    doctor_capabilities,
    error_payload,
    validate_doctor_data,
)


def test_macos_doctor_payload_validates() -> None:
    data = {
        "platform": "macos",
        "session": "aqua",
        "backend": {"input": "cgevent", "screen": "screencapture", "window": "ax-cgwindow"},
        "capabilities": doctor_capabilities(),
        "limitations": [],
        "ready": True,
        "accessibility": True,
        "screen_recording": True,
    }
    validate_doctor_data(data)


def test_wayland_can_disable_focus() -> None:
    data = {
        "platform": "linux",
        "session": "wayland",
        "backend": {"input": "libei", "screen": "portal-pipewire", "window": "foreign-toplevel"},
        "capabilities": doctor_capabilities(list_windows=True, focus_window=False),
        "limitations": ["portal may prompt for ScreenCast / RemoteDesktop"],
        "ready": False,
    }
    validate_doctor_data(data)
    assert data["capabilities"]["focus_window"] is False


def test_validate_doctor_rejects_missing_capability() -> None:
    data = {
        "platform": "windows",
        "backend": {"input": "win32-sendinput", "screen": "windows-graphics-capture", "window": "win32-user32"},
        "capabilities": {"screenshot": True},
        "limitations": [],
        "ready": True,
    }
    try:
        validate_doctor_data(data)
    except ValueError as exc:
        assert "missing" in str(exc)
    else:
        raise AssertionError("expected ValueError")


def test_focus_denied_and_unsupported_error_codes() -> None:
    denied = error_payload("1", "focus_denied", "SetForegroundWindow refused")
    assert denied["error"]["code"] == "focus_denied"
    unsup = error_payload("1", "unsupported", "browser_save_page is macOS-only in v0.2")
    assert unsup["error"]["code"] == "unsupported"
