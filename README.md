# computer-use

[中文说明](README.zh-CN.md)

Native computer-use runtime. Humans use the CLI; Cursor / Codex / Claude / OpenClaw use MCP or the skills in `skills/`. Chrome is a normal desktop app — no Playwright, Selenium, CDP, or Chrome debug port.

v0.2 keeps one JSONL command set and swaps an OS-native helper:

| OS | Helper | Input | Window | Capture |
| --- | --- | --- | --- | --- |
| macOS | Swift (`scripts/build-helper.sh`) | CGEvent | AXUIElement / CGWindow | Screen Capture |
| Windows | Rust (`scripts/build-helper-native.sh`) | SendInput | User32 (`window_id` = HWND) | Windows.Graphics.Capture |
| Linux X11 | Rust | XTEST | EWMH | XGetImage |
| Linux Wayland | Rust | XDG RemoteDesktop + libei | foreign-toplevel if granted | XDG portal (Screenshot / ScreenCast) |

Linux picks X11 vs Wayland from `XDG_SESSION_TYPE` (then `WAYLAND_DISPLAY` / `DISPLAY`). Do not use PyAutoGUI, pynput, AutoHotkey, SendKeys, WinAppDriver, or browser drivers.

This macOS checkout does **not** run Windows or Linux GUI acceptance. Use `pytest` and `cargo test -p computer-use-core`. Real clicks and screenshots belong on the matching OS; see `tests/manual/`.

## Install

### macOS

```bash
./scripts/build-helper.sh
python3 -m pip install -e .
```

Grant **Accessibility** and **Screen Recording** to `ComputerUseHelper` (the `.app` under `dist/`). Because the CLI is launched from Terminal/Python, also grant **Screen Recording** to **Terminal** (or iTerm/Warp) if `doctor` still shows `screen_recording: false`. Fully quit the terminal and rerun:

```bash
computer-use doctor
```

### Windows / Linux

On that OS:

```bash
./scripts/build-helper-native.sh
python3 -m pip install -e .
computer-use doctor
```

`doctor` is capability negotiation. Read `platform`, `backend`, `capabilities`, and `limitations` before `focus_window` or window screenshots. On Wayland, `focus_window` is often false and portal prompts are expected. Windows may return `focus_denied` when `SetForegroundWindow` is blocked.

Override the helper with `COMPUTER_USE_HELPER=/path/to/computer-use-helper`.

## CLI

```bash
computer-use screenshot --out /tmp/screen.png
computer-use screenshot --window-id 131718 --grid --out /tmp/cu-grid.png
computer-use list-windows
computer-use focus-app "Google Chrome"
computer-use hotkey cmd l
computer-use type "https://example.com"
computer-use key enter
computer-use click 820 550
computer-use --pacing conservative scroll 0 -240
computer-use browser-open-profile ComputerUse
computer-use browser-open-url "https://example.com"
computer-use --pacing normal browser-save-page ./out/page.html --scrolls 8
```

`--json` prints machine JSON. `COMPUTER_USE_PACING` / `--pacing` is `off` | `normal` (default) | `conservative`.

Coordinates are always in the latest screenshot's pixel space. `--grid` draws axis ticks in that same space so a window shot can be clicked without adding the window origin by hand.

Browser recipes (`browser-open-profile` / `browser-open-url` / `browser-save-page`) are **macOS-only in v0.2**. Windows/Linux helpers return `unsupported`.

## MCP

```json
{
  "mcpServers": {
    "computer-use": {
      "command": "python3",
      "args": ["-u", "-m", "computer_use", "--pacing", "off", "mcp"]
    }
  }
}
```

## Layout

- `helper/macos` — Swift helper
- `helper/native` — Rust workspace (`computer-use-core`, `computer-use-windows`, `computer-use-linux`)
- `src/computer_use` — CLI, MCP, Pacer, helper dispatch
- `skills/` — agent instructions
- `drivers/README.md` — OS backend table
- `tests/manual/` — per-OS checklists (not a substitute for running GUI tests on that OS)

On Xiaohongshu, Bojin's red button is **保存网页** on the list/search page and **保存笔记** after a note is open.
