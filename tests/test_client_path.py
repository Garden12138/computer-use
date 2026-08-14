from pathlib import Path

import pytest

from computer_use.client import helper_path


def test_env_overrides_platform(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    helper = tmp_path / "custom-helper"
    helper.write_text("", encoding="utf-8")
    monkeypatch.setenv("COMPUTER_USE_HELPER", str(helper))
    assert helper_path(platform="win32", repo_root=tmp_path) == helper


def test_darwin_prefers_app_bundle(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.delenv("COMPUTER_USE_HELPER", raising=False)
    bundled = tmp_path / "dist" / "ComputerUseHelper.app" / "Contents" / "MacOS" / "computer-use-helper"
    bundled.parent.mkdir(parents=True)
    bundled.write_text("", encoding="utf-8")
    assert helper_path(platform="darwin", repo_root=tmp_path) == bundled


def test_win32_prefers_dist_exe(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.delenv("COMPUTER_USE_HELPER", raising=False)
    exe = tmp_path / "dist" / "computer-use-helper.exe"
    exe.parent.mkdir(parents=True)
    exe.write_text("", encoding="utf-8")
    assert helper_path(platform="win32", repo_root=tmp_path) == exe


def test_linux_falls_back_to_native_release(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.delenv("COMPUTER_USE_HELPER", raising=False)
    bin_path = tmp_path / "helper" / "native" / "target" / "release" / "computer-use-helper"
    bin_path.parent.mkdir(parents=True)
    bin_path.write_text("", encoding="utf-8")
    assert helper_path(platform="linux", repo_root=tmp_path) == bin_path


def test_missing_helper_mentions_build_script(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.delenv("COMPUTER_USE_HELPER", raising=False)
    monkeypatch.setattr("computer_use.client.shutil.which", lambda _: None)
    with pytest.raises(FileNotFoundError, match="build-helper-native"):
        helper_path(platform="win32", repo_root=tmp_path)
    with pytest.raises(FileNotFoundError, match="build-helper.sh"):
        helper_path(platform="darwin", repo_root=tmp_path)
