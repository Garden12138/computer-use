"""CLI: computer-use <cmd> ..."""

from __future__ import annotations

import argparse
import json
import sys
import time
from typing import Any

def build_parser() -> argparse.ArgumentParser:
    flags = argparse.ArgumentParser(add_help=False)
    flags.add_argument("--json", action="store_true", help="print machine JSON")
    flags.add_argument("--pacing", default=None, help="off | normal | conservative")
    flags.add_argument("--seed", type=int, default=None)
    parser = argparse.ArgumentParser(
        prog="computer-use",
        description="Native macOS computer-use runtime",
        parents=[flags],
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    def p(name: str, **kwargs: Any) -> argparse.ArgumentParser:
        return sub.add_parser(name, parents=[flags], **kwargs)

    p("mcp", help="run MCP stdio server")
    p("doctor")
    shot = p("screenshot")
    shot.add_argument("--out", dest="path")
    shot.add_argument("--window-id", type=int, dest="window_id")
    shot.add_argument("--grid", action="store_true", help="overlay click-coordinate grid")
    shot.add_argument("--grid-step", type=int, default=50, dest="grid_step")
    p("list-windows")
    p("get-screen-size")
    p("get-active-window")
    cur = p("cursor", help="read current mouse position in screenshot pixels")
    cur.add_argument("--wait", type=float, default=3, help="seconds to move the mouse before reading")

    move = p("move")
    move.add_argument("x", type=float)
    move.add_argument("y", type=float)
    click = p("click")
    click.add_argument("x", type=float)
    click.add_argument("y", type=float)
    dbl = p("double-click")
    dbl.add_argument("x", type=float)
    dbl.add_argument("y", type=float)
    scroll = p("scroll")
    scroll.add_argument("delta_x", type=float)
    scroll.add_argument("delta_y", type=float)
    typ = p("type")
    typ.add_argument("text")
    key = p("key")
    key.add_argument("key")
    hot = p("hotkey")
    hot.add_argument("keys", nargs="+")
    wait = p("wait")
    wait.add_argument("seconds", type=float)
    focus = p("focus-app")
    focus.add_argument("app")
    fw = p("focus-window")
    fw.add_argument("--app")
    fw.add_argument("--title")
    fw.add_argument("--window-id", type=int, dest="window_id")
    bop = p("browser-open-profile")
    bop.add_argument("profile")
    bou = p("browser-open-url")
    bou.add_argument("url")
    bsp = p("browser-save-page")
    bsp.add_argument("path")
    bsp.add_argument("--scrolls", type=int, default=8)
    return parser


def _cmd_name(raw: str) -> str:
    return raw.replace("-", "_")


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.cmd == "mcp":
        from computer_use.mcp_server import run_stdio

        run_stdio(pacing=args.pacing, seed=args.seed)
        return 0
    from computer_use.runtime import Computer

    computer = Computer(pacing=args.pacing, seed=args.seed)
    params = {k: v for k, v in vars(args).items() if k not in {"cmd", "json", "pacing", "seed"} and v is not None}
    cmd = _cmd_name(args.cmd)
    if cmd == "cursor":
        seconds = float(params.pop("wait", 3) or 0)
        if seconds > 0:
            print(f"把鼠标移到目标上，{seconds:.0f} 秒后读取坐标…", file=sys.stderr)
            time.sleep(seconds)
        result = computer.run("cursor", {})
    elif cmd == "browser_save_page":
        result = computer.save_page(params["path"], scrolls=int(params.get("scrolls", 8)))
    else:
        result = computer.run(cmd, params)
    if args.json:
        json.dump(result, sys.stdout, ensure_ascii=False)
        sys.stdout.write("\n")
    else:
        _print_human(result)
    return 0 if result.get("ok") else 1


def _print_human(result: dict[str, Any]) -> None:
    if not result.get("ok"):
        err = result.get("error") or {}
        print(f"error: {err.get('code')}: {err.get('message')}", file=sys.stderr)
        return
    data = result.get("data") or {}
    if "x" in data and "y" in data and len(data) <= 6:
        print(f"{data['x']:.0f} {data['y']:.0f}")
        return
    print(json.dumps(data, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    raise SystemExit(main())
