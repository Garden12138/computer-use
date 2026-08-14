# Manual primitive checks (requires Accessibility + Screen Recording)

1. `computer-use doctor` → `ready: true`
2. `computer-use screenshot --out /tmp/cu.png` → PNG exists, JSON has width/height/scale
3. `computer-use screenshot --window-id <id> --grid --out /tmp/cu-grid.png` → red axis numbers are click coordinates (window origin already added)
3. `computer-use list-windows` lists Chrome if it is open
4. `computer-use focus-app "Google Chrome"`
5. `computer-use hotkey cmd l` focuses the omnibox
6. `computer-use type "about:blank"` then `computer-use key enter`
