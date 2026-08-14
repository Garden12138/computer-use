---
name: computer-use
description: Drive the local GUI with native screenshot, click, type, scroll, and window focus. Use when CLI or structured APIs are not enough and the task needs seeing or operating an app. Never use Playwright, Selenium, CDP, or Chrome debug ports.
---

# computer-use

Native computer-use runtime. The model decides; this tool executes. macOS, Windows, and Linux share one command set; the helper is OS-native.

## Setup

1. macOS: `scripts/build-helper.sh`. Windows/Linux: `scripts/build-helper-native.sh` on that OS.
2. `pip install -e .`
3. macOS: grant **Accessibility** and **Screen Recording** to `ComputerUseHelper`.
4. `computer-use doctor` until `ready` is true. Read `capabilities` and `limitations`.

## Loop

1. `computer-use doctor` — if `capabilities.focus_window` is false (typical Wayland), skip `focus-window` and use screenshots plus keyboard.
2. `computer-use list-windows` then `focus-window` (prefer `--window-id`; avoid `focus-app` when several Chrome windows exist). On Windows `window_id` is HWND; `focus_denied` means the OS refused foreground steal.
3. Screenshot the **target window** with a click-coordinate grid:

```bash
computer-use --pacing off screenshot --window-id 131718 --grid --out /tmp/cu-grid.png
```

4. Read the red axis numbers. They are already **full-display click pixels**. `click 1500 545` uses the same numbers.
5. Screenshot again (with `--grid`) to verify.

Do not guess pixels from a full-screen shot that includes Dock / Stage Manager / taskbar chrome you did not intend to click.

Browser recipes (`browser-open-profile`, `browser-open-url`, `browser-save-page`) are macOS-only in v0.2.

## Xiaohongshu + Bojin

- Search-results list: the right-side button is **保存网页**.
- After a note is open: the same button becomes **保存笔记**. Click that red button; do not keep looking for the old label.
- The explore-page search box often does not accept `type`. Prefer clicking this window's address bar, then type the search URL.

## Rules

- Do not use Playwright, Selenium, CDP, or `--remote-debugging-port`.
- Captcha, login wall, or risk-control pages: stop and ask the human. Do not solve them.
- Default pacing is `normal`. Use `--pacing conservative` for slower GUI. `--pacing off` disables jitter.
- The real system cursor moves. Do not assume a private virtual cursor.

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
