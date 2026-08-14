# computer-use

[English](README.md)

原生 Computer Use 运行时。人用 CLI；Cursor / Codex / Claude / OpenClaw 用 MCP 或 `skills/`。Chrome 按普通桌面应用驱动，不使用 Playwright、Selenium、CDP 或 Chrome debug port。

v0.2 上层命令集（JSONL）三平台一致，底层换成各操作系统的原生 helper：

| 系统 | Helper | 输入 | 窗口 | 截图 |
| --- | --- | --- | --- | --- |
| macOS | Swift（`scripts/build-helper.sh`） | CGEvent | AXUIElement / CGWindow | Screen Capture |
| Windows | Rust（`scripts/build-helper-native.sh`） | SendInput | User32（`window_id` = HWND） | Windows.Graphics.Capture |
| Linux X11 | Rust | XTEST | EWMH | XGetImage |
| Linux Wayland | Rust | XDG RemoteDesktop + libei | compositor 授权时的 foreign-toplevel | XDG portal（Screenshot / ScreenCast） |

Linux 根据 `XDG_SESSION_TYPE`（其次 `WAYLAND_DISPLAY` / `DISPLAY`）选择 X11 或 Wayland。不要使用 PyAutoGUI、pynput、AutoHotkey、SendKeys、WinAppDriver 或浏览器驱动。

本仓库若在 macOS 上开发，**不对 Windows / Linux 做 GUI 真实验收**。本地只跑 `pytest` 和 `cargo test -p computer-use-core`。真实点击与截图请在对应系统上做，清单见 `tests/manual/`。

## 安装

### macOS

```bash
./scripts/build-helper.sh
python3 -m pip install -e .
```

把 **辅助功能** 和 **屏幕录制** 授权给 `ComputerUseHelper`（`dist/` 下的 `.app`）。CLI 从终端/Python 拉起时，若 `doctor` 仍显示 `screen_recording: false`，还要给 **终端**（或 iTerm/Warp）开屏幕录制。完全退出终端后再跑：

```bash
computer-use doctor
```

### Windows / Linux

在目标操作系统上：

```bash
./scripts/build-helper-native.sh
python3 -m pip install -e .
computer-use doctor
```

`doctor` 做能力协商。调用 `focus_window` 或窗口截图前先看 `platform`、`backend`、`capabilities`、`limitations`。Wayland 上 `focus_window` 常常为 false，portal 弹授权是预期行为。Windows 在 `SetForegroundWindow` 被系统拦住时会返回 `focus_denied`。

可用 `COMPUTER_USE_HELPER=/path/to/computer-use-helper` 覆盖 helper 路径。

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

`--json` 输出机器可读 JSON。`COMPUTER_USE_PACING` / `--pacing` 为 `off` | `normal`（默认）| `conservative`。

坐标永远是**最近一次截图的像素空间**。`--grid` 在同一空间画轴刻度，窗口截图可以直接点，不必再手工加窗口原点。

浏览器配方（`browser-open-profile` / `browser-open-url` / `browser-save-page`）在 **v0.2 仅保证 macOS**。Windows/Linux helper 会返回 `unsupported`。

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

## 目录

- `helper/macos` — Swift helper
- `helper/native` — Rust workspace（`computer-use-core`、`computer-use-windows`、`computer-use-linux`）
- `src/computer_use` — CLI、MCP、Pacer、helper 分发
- `skills/` — agent 说明
- `drivers/README.md` — 各系统 backend 对照
- `tests/manual/` — 分系统手工清单（不能代替在该系统上跑 GUI）

小红书场景里，铂金右侧红按钮在列表/搜索页是 **保存网页**，打开笔记后是 **保存笔记**。
