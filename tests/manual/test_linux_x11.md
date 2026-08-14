# Manual primitive checks (Linux X11)

Run on an Xorg session (`echo $XDG_SESSION_TYPE` → `x11`) after `scripts/build-helper-native.sh`. Not valid on this macOS checkout.

1. `computer-use doctor` → `platform: linux`, `session: x11`, `backend.input: xtest`, `backend.window: ewmh`
2. `computer-use screenshot --out /tmp/cu.png`
3. `computer-use list-windows` — `window_id` is an X11 Window
4. `computer-use screenshot --window-id <id> --grid --out /tmp/cu-grid.png`
5. `computer-use focus-window --window-id <id>` via EWMH `_NET_ACTIVE_WINDOW`
6. `computer-use click` / `hotkey ctrl l` against a focused app
7. Browser recipes should return `unsupported`
