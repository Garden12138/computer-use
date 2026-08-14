# Manual primitive checks (Windows)

Run on a real Windows 10 1903+ machine after `scripts/build-helper-native.sh`. Do not treat a macOS checkout as acceptance.

1. `computer-use doctor` → `platform: windows`, `backend.input: win32-sendinput`, `backend.screen: windows-graphics-capture`, `ready: true`
2. `computer-use screenshot --out %TEMP%\cu.png`
3. `computer-use list-windows` — `window_id` is HWND
4. `computer-use screenshot --window-id <hwnd> --grid --out %TEMP%\cu-grid.png`
5. `computer-use get-screen-size` and `computer-use cursor --wait 2`
6. `computer-use click` using grid numbers from the screenshot
7. `computer-use focus-window --window-id <hwnd>` — if the OS blocks it, expect `focus_denied`
8. Browser recipes should return `unsupported`
