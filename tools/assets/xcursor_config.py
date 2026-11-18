# SPDX-License-Identifier: GPL-3.0-or-later
"""Write deterministic xcursorgen input for a complete cursor image."""

from __future__ import annotations

from pathlib import Path


def write_cursor_config(
    destination: Path,
    images: list[tuple[int, int, int, Path, int]],
) -> None:
    """Write size, hotspot, image path and frame delay records."""
    if not images:
        raise ValueError("a cursor must contain at least one image")

    lines: list[str] = []
    for size, hotspot_x, hotspot_y, image_path, delay_ms in images:
        if size <= 0:
            raise ValueError("cursor image size must be positive")
        if not (0 <= hotspot_x < size and 0 <= hotspot_y < size):
            raise ValueError("cursor hotspot must be inside its raster image")
        if not image_path.is_file():
            raise FileNotFoundError(image_path)
        if delay_ms < 0:
            raise ValueError("cursor frame delay cannot be negative")
        lines.append(
            f"{size} {hotspot_x} {hotspot_y} {image_path.name} {delay_ms}"
        )

    destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
