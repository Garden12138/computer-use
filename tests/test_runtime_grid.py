from pathlib import Path

from PIL import Image

from computer_use.runtime import Computer


class _FakeClient:
    def __init__(self, path: str) -> None:
        self.path = path

    def call(self, cmd: str, params: dict) -> dict:
        assert "grid" not in params
        return {
            "ok": True,
            "id": "1",
            "data": {
                "path": self.path,
                "width": 200,
                "height": 120,
                "scale": 1,
                "origin_x": 154,
                "origin_y": 30,
                "window_id": 131718,
            },
        }


def test_screenshot_grid_overwrites_png_and_keeps_screen_origin(tmp_path: Path) -> None:
    shot = tmp_path / "win.png"
    Image.new("RGB", (200, 120), (240, 240, 240)).save(shot)
    computer = Computer(pacing="off", client=_FakeClient(str(shot)))
    result = computer.screenshot(path=str(shot), window_id=131718, grid=True, grid_step=50)
    data = result["data"]
    assert data["grid"] is True
    assert data["origin_x"] == 154
    assert data["path"] == str(shot)
    assert (154, 30) in data["ticks"]
    assert Image.open(shot).getpixel((0, 0)) != (240, 240, 240)
