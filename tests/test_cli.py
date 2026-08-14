from computer_use.cli import build_parser


def test_hotkey_and_json_flags() -> None:
    args = build_parser().parse_args(["hotkey", "cmd", "l", "--json"])
    assert args.cmd == "hotkey"
    assert args.keys == ["cmd", "l"]
    assert args.json is True


def test_screenshot_grid_flags() -> None:
    args = build_parser().parse_args(
        ["screenshot", "--window-id", "131718", "--grid", "--grid-step", "50", "--out", "/tmp/g.png"]
    )
    assert args.cmd == "screenshot"
    assert args.window_id == 131718
    assert args.grid is True
    assert args.grid_step == 50
    assert args.path == "/tmp/g.png"


def test_cursor_wait_flag() -> None:
    args = build_parser().parse_args(["cursor", "--wait", "5"])
    assert args.cmd == "cursor"
    assert args.wait == 5
    args = build_parser().parse_args(["browser-save-page", "./out/page.html", "--scrolls", "5"])
    assert args.path == "./out/page.html"
    assert args.scrolls == 5
