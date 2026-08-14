"""Draw a labeled grid on a screenshot. Tick numbers are click coordinates."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw, ImageFont

from computer_use.coords import window_local_to_screen


def overlay_grid(
    image_path: str,
    *,
    origin_x: float = 0,
    origin_y: float = 0,
    scale: float = 1,
    step: int = 50,
    out_path: str | None = None,
    bounds_width: float | None = None,
    bounds_height: float | None = None,
) -> dict[str, Any]:
    if step <= 0:
        raise ValueError("step must be positive")
    src = Path(image_path)
    dest = Path(out_path) if out_path else src.with_name(f"{src.stem}-grid{src.suffix}")
    image = Image.open(src).convert("RGB")
    draw = ImageDraw.Draw(image)
    font = ImageFont.load_default()
    width, height = image.size
    ticks: list[tuple[int, int]] = []
    for local_x in range(0, width, step):
        screen_x, _ = window_local_to_screen(
            local_x,
            0,
            origin_x=origin_x,
            origin_y=origin_y,
            scale=scale,
            image_width=width,
            image_height=height,
            bounds_width=bounds_width,
            bounds_height=bounds_height,
        )
        sx = int(round(screen_x))
        draw.line([(local_x, 0), (local_x, height - 1)], fill=(255, 0, 0), width=1)
        draw.text((local_x + 2, 2), str(sx), fill=(255, 0, 0), font=font)
    for local_y in range(0, height, step):
        _, screen_y = window_local_to_screen(
            0,
            local_y,
            origin_x=origin_x,
            origin_y=origin_y,
            scale=scale,
            image_width=width,
            image_height=height,
            bounds_width=bounds_width,
            bounds_height=bounds_height,
        )
        sy = int(round(screen_y))
        draw.line([(0, local_y), (width - 1, local_y)], fill=(255, 0, 0), width=1)
        draw.text((2, local_y + 2), str(sy), fill=(255, 0, 0), font=font)
        for local_x in range(0, width, step):
            screen_x, screen_y = window_local_to_screen(
                local_x,
                local_y,
                origin_x=origin_x,
                origin_y=origin_y,
                scale=scale,
                image_width=width,
                image_height=height,
                bounds_width=bounds_width,
                bounds_height=bounds_height,
            )
            ticks.append((int(round(screen_x)), int(round(screen_y))))
    dest.parent.mkdir(parents=True, exist_ok=True)
    image.save(dest)
    return {
        "path": str(dest),
        "origin_x": origin_x,
        "origin_y": origin_y,
        "step": step,
        "ticks": ticks,
        "width": width,
        "height": height,
    }
