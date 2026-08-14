"""Spawn the Swift helper for one JSON command."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any, Mapping

from computer_use.protocol import error_payload, request_payload

_REPO_ROOT = Path(__file__).resolve().parents[2]


def helper_path() -> Path:
    env = os.environ.get("COMPUTER_USE_HELPER")
    if env:
        return Path(env)
    bundled = _REPO_ROOT / "dist" / "ComputerUseHelper.app" / "Contents" / "MacOS" / "computer-use-helper"
    if bundled.exists():
        return bundled
    debug = _REPO_ROOT / "helper" / "macos" / ".build" / "debug" / "computer-use-helper"
    if debug.exists():
        return debug
    release = _REPO_ROOT / "helper" / "macos" / ".build" / "release" / "computer-use-helper"
    if release.exists():
        return release
    which = shutil.which("computer-use-helper")
    if which:
        return Path(which)
    raise FileNotFoundError(
        "computer-use-helper not found. Run scripts/build-helper.sh or set COMPUTER_USE_HELPER."
    )


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
