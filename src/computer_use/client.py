"""Spawn the native helper for one JSON command."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Mapping

from computer_use.protocol import error_payload, request_payload

_REPO_ROOT = Path(__file__).resolve().parents[2]


def helper_path(*, platform: str | None = None, repo_root: Path | None = None) -> Path:
    env = os.environ.get("COMPUTER_USE_HELPER")
    if env:
        return Path(env)
    root = repo_root or _REPO_ROOT
    plat = platform if platform is not None else sys.platform
    candidates = _helper_candidates(plat, root)
    for path in candidates:
        if path.exists():
            return path
    which = shutil.which("computer-use-helper")
    if which:
        return Path(which)
    raise FileNotFoundError(_missing_helper_message(plat))


def _helper_candidates(platform: str, root: Path) -> list[Path]:
    native = root / "helper" / "native" / "target"
    if platform == "darwin":
        return [
            root / "dist" / "ComputerUseHelper.app" / "Contents" / "MacOS" / "computer-use-helper",
            root / "helper" / "macos" / ".build" / "debug" / "computer-use-helper",
            root / "helper" / "macos" / ".build" / "release" / "computer-use-helper",
        ]
    if platform == "win32":
        names = ("computer-use-helper.exe", "computer-use-helper-windows.exe")
        out: list[Path] = []
        for name in names:
            out.extend(
                [
                    root / "dist" / name,
                    native / "release" / name,
                    native / "debug" / name,
                ]
            )
        return out
    if platform.startswith("linux"):
        names = ("computer-use-helper", "computer-use-helper-linux")
        out: list[Path] = []
        for name in names:
            out.extend(
                [
                    root / "dist" / name,
                    native / "release" / name,
                    native / "debug" / name,
                ]
            )
        return out
    return []


def _missing_helper_message(platform: str) -> str:
    if platform == "darwin":
        return "computer-use-helper not found. Run scripts/build-helper.sh or set COMPUTER_USE_HELPER."
    if platform == "win32":
        return (
            "computer-use-helper.exe not found. "
            "On Windows run scripts/build-helper-native.sh (cargo build -p computer-use-windows --release) "
            "or set COMPUTER_USE_HELPER."
        )
    if platform.startswith("linux"):
        return (
            "computer-use-helper not found. "
            "On Linux run scripts/build-helper-native.sh (cargo build -p computer-use-linux --release) "
            "or set COMPUTER_USE_HELPER."
        )
    return f"computer-use-helper not found for platform {platform}. Set COMPUTER_USE_HELPER."


class HelperClient:
    def __init__(self, path: Path | None = None) -> None:
        self._path = path

    @property
    def path(self) -> Path:
        if self._path is None:
            self._path = helper_path()
        return self._path

    def call(self, cmd: str, params: Mapping[str, Any] | None = None) -> dict[str, Any]:
        req = request_payload(cmd, params)
        proc = subprocess.run(
            [str(self.path), "stdio"],
            input=(json.dumps(req, ensure_ascii=False) + "\n").encode("utf-8"),
            capture_output=True,
            timeout=120,
            check=False,
        )
        line = proc.stdout.decode("utf-8", "replace").strip().splitlines()
        if not line:
            return error_payload(
                req["id"],
                "helper_failed",
                proc.stderr.decode("utf-8", "replace")[:500] or "helper produced no output",
                {"exit_code": proc.returncode, "helper": str(self.path)},
            )
        try:
            payload = json.loads(line[-1])
        except json.JSONDecodeError:
            return error_payload(req["id"], "invalid_json", line[-1][:500])
        if cmd == "doctor" and isinstance(payload.get("data"), dict):
            payload["data"]["helper"] = str(self.path)
        return payload
