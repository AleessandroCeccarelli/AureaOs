# SPDX-License-Identifier: GPL-3.0-or-later
"""Load and validate the declarative AureaOS cursor asset manifest."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

EXPECTED_CURSOR_NAMES = {
    "default",
    "pointer",
    "text",
    "grab",
    "grabbing",
    "move",
    "resize-nesw",
    "resize-nwse",
    "busy",
    "crosshair",
    "help",
    "not-allowed",
    "copy",
    "alias",
    "ew-resize",
    "ns-resize",
    "pencil",
    "all-scroll",
    "left_ptr_watch",
}


def load_manifest(path: Path) -> dict[str, Any]:
    with path.open("rb") as manifest_file:
        return tomllib.load(manifest_file)


def validate_manifest(data: dict[str, Any], manifest_path: Path) -> list[str]:
    errors: list[str] = []
    if data.get("schema_version") != 1:
        errors.append("schema_version must be 1")

    width = data.get("canvas_width")
    height = data.get("canvas_height")
    if width != 64 or height != 64:
        errors.append("source canvas must be exactly 64 × 64")

    sizes = data.get("logical_sizes")
    if not isinstance(sizes, list) or not sizes or any(
        not isinstance(size, int) or size <= 0 or size > 256 for size in sizes
    ):
        errors.append("logical_sizes must contain positive integers no greater than 256")
    elif len(set(sizes)) != len(sizes):
        errors.append("logical_sizes must not contain duplicates")

    scales = data.get("output_scales")
    if not isinstance(scales, list) or not scales or any(
        not isinstance(scale, int) or scale not in (1, 2, 3, 4) for scale in scales
    ):
        errors.append("output_scales must contain supported integer scales (1–4)")
    elif len(set(scales)) != len(scales):
        errors.append("output_scales must not contain duplicates")

    entries = data.get("cursor")
    if not isinstance(entries, list):
        errors.append("manifest must define cursor entries")
        return errors

    names: list[str] = []
    manifest_dir = manifest_path.parent.resolve()
    for index, entry in enumerate(entries):
        label = f"cursor entry {index + 1}"
        if not isinstance(entry, dict):
            errors.append(f"{label} must be a table")
            continue

        name = entry.get("name")
        if not isinstance(name, str) or not name:
            errors.append(f"{label} has no valid name")
        else:
            names.append(name)

        source = entry.get("source")
        if not isinstance(source, str) or not source:
            errors.append(f"{label} has no source path")
        else:
            candidate = (manifest_dir / source).resolve()
            if not candidate.is_relative_to(manifest_dir):
                errors.append(f"{label} source escapes the cursor asset directory")
            elif not candidate.is_file():
                errors.append(f"{label} source does not exist: {source}")

        hotspot = entry.get("hotspot")
        if (
            not isinstance(hotspot, list)
            or len(hotspot) != 2
            or any(not isinstance(value, int) for value in hotspot)
        ):
            errors.append(f"{label} hotspot must contain two integer coordinates")
        elif width and height and not (
            0 <= hotspot[0] < width and 0 <= hotspot[1] < height
        ):
            errors.append(f"{label} hotspot is outside the source canvas")

        role = entry.get("role")
        if not isinstance(role, str) or not role:
            errors.append(f"{label} has no role")

        if name == "busy":
            frames = entry.get("animation_frames")
            duration = entry.get("animation_duration_ms")
            if not isinstance(frames, int) or not 2 <= frames <= 60:
                errors.append("busy animation_frames must be between 2 and 60")
            if not isinstance(duration, int) or not 100 <= duration <= 10000:
                errors.append("busy animation_duration_ms must be between 100 and 10000")
        elif "animation_frames" in entry or "animation_duration_ms" in entry:
            errors.append(f"{label} declares animation metadata but is not the busy cursor")

    if len(names) != len(set(names)):
        errors.append("cursor names must be unique")
    missing = EXPECTED_CURSOR_NAMES - set(names)
    unexpected = set(names) - EXPECTED_CURSOR_NAMES
    if missing:
        errors.append(f"missing cursor entries: {', '.join(sorted(missing))}")
    if unexpected:
        errors.append(f"unexpected cursor entries: {', '.join(sorted(unexpected))}")

    return errors
