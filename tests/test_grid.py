from pathlib import Path

from PIL import Image

from computer_use.grid import overlay_grid


def test_overlay_grid_labels_use_screen_coordinates(tmp_path: Path) -> None:
    src = tmp_path / "window.png"
    Image.new("RGB", (200, 120), (240, 240, 240)).save(src)
    out = tmp_path / "grid.png"

    info = overlay_grid(
        str(src),
        origin_x=154,
        origin_y=30,
        scale=1,
        step=50,
        out_path=str(out),
    )

    assert info["path"] == str(out)
    assert info["origin_x"] == 154
    assert info["origin_y"] == 30
    assert info["step"] == 50
    assert (154, 30) in info["ticks"]
    assert (204, 80) in info["ticks"]
    labeled = Image.open(out)
    assert labeled.size == (200, 120)
    assert labeled.getpixel((0, 0)) != (240, 240, 240)


def test_overlay_grid_default_writes_beside_source(tmp_path: Path) -> None:
    src = tmp_path / "shot.png"
    Image.new("RGB", (80, 80), (10, 10, 10)).save(src)
    info = overlay_grid(str(src), origin_x=0, origin_y=0, scale=1, step=40)
    assert Path(info["path"]).name == "shot-grid.png"
    assert Path(info["path"]).exists()
