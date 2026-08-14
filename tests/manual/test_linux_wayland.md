# Manual primitive checks (Linux Wayland)

Run on a Wayland session after `scripts/build-helper-native.sh`. Portal permission UI is expected. Not valid on this macOS checkout.

1. `computer-use doctor` → `platform: linux`, `session: wayland`, `backend.input: libei`, `backend.screen: portal-pipewire`
2. Confirm `capabilities.focus_window` is usually `false`. Do not assume `list_windows` works.
3. `computer-use screenshot --out /tmp/cu.png` — allow ScreenCast/Screenshot portal if prompted
4. `computer-use click` / `type` — allow RemoteDesktop portal if prompted
5. `computer-use list-windows` / `focus-window` should return `unsupported` unless the compositor granted foreign-toplevel
6. Success means honest capabilities and authorized portals, not X11-style unconditional global control
7. Browser recipes should return `unsupported`
