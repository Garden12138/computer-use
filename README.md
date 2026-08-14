# computer-use

Native computer-use runtime. Humans use the CLI; Cursor / Codex / Claude / OpenClaw use MCP or the skills in `skills/`. Chrome is driven as a normal desktop app — no Playwright, Selenium, CDP, or Chrome debug port.

v0.2 keeps one JSONL command set and swaps OS helpers:

- **macOS** — Swift + CGEvent + AXUIElement + Screen Capture (`scripts/build-helper.sh`)
- **Windows** — Rust + SendInput + User32 + Windows.Graphics.Capture (`scripts/build-helper-native.sh` on Windows)
- **Linux** — Rust with **X11** (XTEST / EWMH / XGetImage) or **Wayland** (XDG portals / libei) backends, chosen from `XDG_SESSION_TYPE`

This Mac checkout does **not** run Windows or Linux GUI acceptance. Use `pytest` and `cargo test -p computer-use-core`. Real clicks/screenshots belong on the matching OS; see `tests/manual/`.

## Install

macOS:

```bash
./scripts/build-helper.sh
python3 -m pip install -e .
```

Grant **Accessibility** and **Screen Recording** to `ComputerUseHelper` (the `.app` under `dist/`). Because the CLI is launched from Terminal/Python, also grant **Screen Recording** to **Terminal** (or iTerm/Warp) if `doctor` still shows `screen_recording: false`. Then fully quit the terminal and rerun:

```bash
computer-use doctor
```

Windows / Linux (on that OS):

```bash
./scripts/build-helper-native.sh
python3 -m pip install -e .
computer-use doctor
```

`doctor` is capability negotiation. Read `platform`, `backend`, `capabilities`, and `limitations` before calling `focus_window` or window screenshots. On Wayland, `focus_window` is often false and portal prompts are expected.

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

Coordinates are always in the latest screenshot's pixel space. `--grid` draws axis ticks in that same space so a window shot can be clicked without adding the window origin by hand.

On Xiaohongshu, Bojin's red button is **保存网页** on the list/search page and **保存笔记** after a note is open.
