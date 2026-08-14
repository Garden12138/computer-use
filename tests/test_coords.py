import pytest

from computer_use.coords import cg_to_screenshot, screenshot_to_cg, window_local_to_screen


def test_retina_scale_two() -> None:
    assert screenshot_to_cg(820, 550, 2.0) == (410.0, 275.0)


def test_round_trip() -> None:
    cg = screenshot_to_cg(100, 40, 2.0)
    assert cg_to_screenshot(*cg, 2.0) == (100.0, 40.0)


def test_invalid_scale() -> None:
    with pytest.raises(ValueError):
        screenshot_to_cg(1, 1, 0)


def test_window_local_to_screen_scale_one() -> None:
    assert window_local_to_screen(1346, 515, origin_x=154, origin_y=30, scale=1) == (1500.0, 545.0)


def test_window_local_to_screen_retina() -> None:
    assert window_local_to_screen(200, 100, origin_x=100, origin_y=40, scale=2) == (400.0, 180.0)
