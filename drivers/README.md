# Native backends

Upper layer (`protocol.py`) is the same on every OS. Helpers speak JSONL on stdio. Do not use PyAutoGUI, pynput, AutoHotkey, SendKeys, WinAppDriver, Selenium, Playwright, or CDP.

| OS | Language | Input | Window | Capture |
| --- | --- | --- | --- | --- |
| macOS | Swift | CGEvent | AXUIElement / CGWindow | Screen Capture |
| Windows | Rust | Win32 SendInput | User32 (`window_id` = HWND) | Windows.Graphics.Capture |
| Linux X11 | Rust | XTEST | EWMH | XGetImage (MIT-SHM later) |
| Linux Wayland | Rust | XDG RemoteDesktop + libei | foreign-toplevel if granted | XDG ScreenCast / Screenshot portal + PipeWire |

Linux chooses X11 vs Wayland from `XDG_SESSION_TYPE` (then `WAYLAND_DISPLAY` / `DISPLAY`).

`doctor` returns `platform`, `session`, `backend`, `capabilities`, `limitations`. If `capabilities.focus_window` is false, do not call `focus_window`. Windows may return `focus_denied` when `SetForegroundWindow` is blocked.

UI Automation / AT-SPI / DXGI Desktop Duplication are out of scope for v0.2.

Build:

- macOS: `scripts/build-helper.sh`
- Windows / Linux: `scripts/build-helper-native.sh` on that OS
