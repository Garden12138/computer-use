# computer-use

macOS v0.1 native computer-use runtime. Humans use the CLI; Cursor / Codex / Claude / OpenClaw use MCP or the skills in `skills/`. Chrome is driven as a normal desktop app — no Playwright, Selenium, CDP, or Chrome debug port.

## Install

```bash
./scripts/build-helper.sh
python3 -m pip install -e .
```

Grant **Accessibility** and **Screen Recording** to `ComputerUseHelper` (the `.app` under `dist/`). Because the CLI is launched from Terminal/Python, also grant **Screen Recording** to **Terminal** (or iTerm/Warp) if `doctor` still shows `screen_recording: false`. Then fully quit the terminal and rerun:

```bash
computer-use doctor
```

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

- `helper/macos` — Swift helper (`AXUIElement`, `CGEvent`, screen capture)
- `src/computer_use` — CLI, MCP, Pacer
- `skills/` — agent instructions

Coordinates are always in the latest screenshot's pixel space. `--grid` draws axis ticks in that same space so a window shot can be clicked without adding the window origin by hand.

On Xiaohongshu, Bojin's red button is **保存网页** on the list/search page and **保存笔记** after a note is open.
