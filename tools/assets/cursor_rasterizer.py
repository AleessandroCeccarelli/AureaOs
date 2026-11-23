# SPDX-License-Identifier: GPL-3.0-or-later
"""Rasterize static and animated AureaOS SVG cursor masters."""

from __future__ import annotations

import math
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

SVG_NAMESPACE = "{http://www.w3.org/2000/svg}"


def _render_svg(source: Path, destination: Path, size: int, renderer: str) -> None:
    subprocess.run(
        [renderer, "--format=png", f"--width={size}", f"--height={size}",
         str(source), f"--output={destination}"],
        check=True,
        capture_output=True,
        text=True,
    )
    if not destination.is_file() or destination.stat().st_size == 0:
        raise RuntimeError(f"SVG renderer did not produce an image: {source.name}")


def rasterize_cursor(
    source: Path,
    destination_dir: Path,
    logical_size: int,
    scale: int,
    renderer: str,
    animation_frames: int = 1,
) -> list[Path]:
    """Render one cursor at a device scale; return frames in playback order."""
    pixel_size = logical_size * scale
    destination_dir.mkdir(parents=True, exist_ok=True)

    if animation_frames == 1:
        output = destination_dir / f"{source.stem}-{pixel_size}.png"
        _render_svg(source, output, pixel_size, renderer)
        return [output]

    tree = ET.parse(source)
    root = tree.getroot()
    animated_groups = [
        element for element in root.iter(f"{SVG_NAMESPACE}g")
        if any(child.tag.rsplit("}", 1)[-1].startswith("animate")
               for child in list(element))
    ]
    if len(animated_groups) != 1:
        raise ValueError(
            f"{source.name}: expected exactly one animated SVG group, "
            f"found {len(animated_groups)}"
        )

    group = animated_groups[0]
    for child in list(group):
        if child.tag.rsplit("}", 1)[-1].startswith("animate"):
            group.remove(child)

    outputs: list[Path] = []
    with tempfile.TemporaryDirectory(prefix="aurea-cursor-svg-") as temporary:
        temporary_svg = Path(temporary) / "frame.svg"
        for frame in range(animation_frames):
            angle = 360.0 * frame / animation_frames
            group.set("transform", f"rotate({angle:.6f} 32 32)")
            tree.write(temporary_svg, encoding="utf-8", xml_declaration=True)
            output = destination_dir / f"{source.stem}-{pixel_size}-{frame:02d}.png"
            _render_svg(temporary_svg, output, pixel_size, renderer)
            outputs.append(output)

    return outputs
