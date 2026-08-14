"""Screenshot-pixel space ↔ CoreGraphics point space."""

from __future__ import annotations


def screenshot_to_cg(x: float, y: float, scale: float) -> tuple[float, float]:
    """Convert screenshot pixels into CG points.

    Computer-use callers always speak screenshot pixels. The helper posts
    CGEvent using points: ``cg = pixel / scale``.
    """

    if scale <= 0:
        raise ValueError("scale must be positive")
    return x / scale, y / scale


def cg_to_screenshot(x: float, y: float, scale: float) -> tuple[float, float]:
    if scale <= 0:
        raise ValueError("scale must be positive")
    return x * scale, y * scale


def window_local_to_screen(
    local_x: float,
    local_y: float,
    *,
    origin_x: float,
    origin_y: float,
    scale: float,
    image_width: float | None = None,
    image_height: float | None = None,
    bounds_width: float | None = None,
    bounds_height: float | None = None,
) -> tuple[float, float]:
    """Map a window-screenshot pixel to a full-display click pixel.

    ``origin_*`` and ``bounds_*`` are CG points from ``list-windows``.
    Window captures are usually ``bounds * scale`` pixels; if the PNG
    size differs, local pixels are scaled to that rectangle first.
    """

    if scale <= 0:
        raise ValueError("scale must be positive")
    fx, fy = float(local_x), float(local_y)
    if (
        image_width
        and image_height
        and bounds_width
        and bounds_height
        and image_width > 0
        and image_height > 0
    ):
        fx = fx * (bounds_width * scale / image_width)
        fy = fy * (bounds_height * scale / image_height)
    return origin_x * scale + fx, origin_y * scale + fy
