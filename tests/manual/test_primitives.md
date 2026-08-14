# Manual primitive checks (macOS; requires Accessibility + Screen Recording)

1. `computer-use doctor` → `ready: true`, `platform: macos`, `capabilities.focus_window: true`
2. `computer-use screenshot --out /tmp/cu.png` → PNG exists, JSON has width/height/scale
3. `computer-use screenshot --window-id <id> --grid --out /tmp/cu-grid.png` → red axis numbers are click coordinates (window origin already added)
4. `computer-use list-windows` lists Chrome if it is open
5. `computer-use focus-app "Google Chrome"`
6. `computer-use hotkey cmd l` focuses the omnibox
7. `computer-use type "about:blank"` then `computer-use key enter`
