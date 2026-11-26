#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build a complete Xcursor theme from the AureaOS SVG source masters."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from cursor_manifest import load_manifest, validate_manifest
from cursor_rasterizer import rasterize_cursor
from cursor_svg import validate_svg
from xcursor_config import write_cursor_config

# Standard Xcursor names and common toolkit aliases. Aliases share the exact
# same Xcursor payload so their hotspot and animation timing stay identical.
CURSOR_ALIASES = {
    "left_ptr": "default",
    "arrow": "default",
    "top_left_arrow": "default",
    "hand2": "pointer",
    "hand1": "pointer",
    "pointing_hand": "pointer",
    "xterm": "text",
    "ibeam": "text",
    "cross": "crosshair",
    "question_arrow": "help",
    "whats_this": "help",
    "watch": "busy",
    "wait": "busy",
    "progress": "left_ptr_watch",
    "no-drop": "not-allowed",
    "forbidden": "not-allowed",
    "openhand": "grab",
    "closedhand": "grabbing",
    "fleur": "move",
    "size_all": "move",
    "size_hor": "ew-resize",
    "sb_h_double_arrow": "ew-resize",
    "col-resize": "ew-resize",
    "size_ver": "ns-resize",
    "sb_v_double_arrow": "ns-resize",
    "row-resize": "ns-resize",
    "nwse-resize": "resize-nwse",
    "size_fdiag": "resize-nwse",
    "bd_double_arrow": "resize-nwse",
    "nesw-resize": "resize-nesw",
    "size_bdiag": "resize-nesw",
    "fd_double_arrow": "resize-nesw",
    "pencil": "pencil",
    "up_arrow": "up-arrow",
    "copy": "copy",
    "alias": "alias",
    "pin": "pin",
    "person": "person",
    "vertical-text": "vertical-text",
    "zoom-in": "zoom-in",
    "zoom-out": "zoom-out",
    "context-menu": "context-menu",
    "dnd-copy": "copy",
    "dnd-link": "alias",
    "dnd-no-drop": "not-allowed",
}


def _require_tool(name: str) -> str:
    executable = shutil.which(name)
    if executable is None:
        raise RuntimeError(
            f"required tool '{name}' was not found; install librsvg2-tools "
            "and xcursorgen (Fedora) or librsvg2-bin and xcursorgen (Debian)"
        )
    return executable


def build_theme(
    cursor_directory: Path,
    output_directory: Path,
    manifest_path: Path | None = None,
) -> list[str]:
    cursor_directory = cursor_directory.resolve()
    manifest_path = (manifest_path or (cursor_directory / "manifest.toml")).resolve()
    manifest = load_manifest(manifest_path)
    errors = validate_manifest(manifest, manifest_path)
    if errors:
        raise ValueError("\n".join(errors))

    renderer = _require_tool("rsvg-convert")
    packager = _require_tool("xcursorgen")

    for entry in manifest["cursor"]:
        source = (cursor_directory / entry["source"]).resolve()
        errors = validate_svg(
            source,
            manifest["canvas_width"],
            manifest["canvas_height"],
            allow_animation=entry["name"] == "busy",
        )
        if errors:
            raise ValueError(f"{entry['name']}: " + "; ".join(errors))

    generated_root = (cursor_directory / "generated").resolve()
    generated_root.mkdir(parents=True, exist_ok=True)
    output_directory = output_directory.resolve()
    if not output_directory.is_relative_to(generated_root):
        raise ValueError("output directory must remain inside the generated asset directory")
    if output_directory.exists() or output_directory.is_symlink():
        raise FileExistsError(
            f"output already exists: {output_directory}; move it aside before rebuilding"
        )

    with tempfile.TemporaryDirectory(prefix=".aurea-cursor-build-", dir=generated_root) as temporary:
        staging = Path(temporary) / "AureaOS"
        cursors_dir = staging / "cursors"
        cursors_dir.mkdir(parents=True)
        logical_sizes = manifest["logical_sizes"]
        scales = manifest["output_scales"]

        for entry in manifest["cursor"]:
            name = entry["name"]
            source = (cursor_directory / entry["source"]).resolve()
            hotspot_x, hotspot_y = entry["hotspot"]
            animation_frames = entry.get("animation_frames", 1)
            animation_duration = entry.get("animation_duration_ms", 0)
            delay_ms = round(animation_duration / animation_frames) if animation_frames > 1 else 0
            cursor_records: list[tuple[int, int, int, Path, int]] = []

            for logical_size in logical_sizes:
                for scale in scales:
                    pixel_size = logical_size * scale
                    scaled_hotspot_x = round(hotspot_x * pixel_size / manifest["canvas_width"])
                    scaled_hotspot_y = round(hotspot_y * pixel_size / manifest["canvas_height"])
                    frames = rasterize_cursor(
                        source,
                        Path(temporary) / "images" / name,
                        logical_size,
                        scale,
                        renderer,
                        animation_frames,
                    )
                    for frame_index, frame_path in enumerate(frames):
                        cursor_records.append((
                            pixel_size,
                            scaled_hotspot_x,
                            scaled_hotspot_y,
                            frame_path,
                            delay_ms if animation_frames > 1 else 0,
                        ))

            config_path = Path(temporary) / f"{name}.cursor"
            write_cursor_config(config_path, cursor_records)
            subprocess.run(
                [packager, str(config_path), str(cursors_dir / name)],
                cwd=Path(temporary),
                check=True,
                capture_output=True,
                text=True,
            )

        for alias, target in CURSOR_ALIASES.items():
            alias_path = cursors_dir / alias
            target_path = cursors_dir / target
            if target_path.is_file() and not alias_path.exists():
                alias_path.symlink_to(target)

        (staging / "index.theme").write_text(
            "[Icon Theme]\n"
            "Name=AureaOS\n"
            "Comment=Native AureaOS cursor theme\n"
            "Directories=cursors\n\n"
            "[cursors]\n"
            "Size=32\n"
            "Type=Fixed\n"
            "Context=Cursors\n",
            encoding="utf-8",
        )
        output_directory.parent.mkdir(parents=True, exist_ok=True)
        staging.rename(output_directory)

    return sorted(path.name for path in (output_directory / "cursors").iterdir())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cursor-dir",
        type=Path,
        default=Path(__file__).resolve().parents[2] / "aurea-human-interfaces/cursors",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="cursor manifest (use manifest-dark.toml for the dark palette)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="output theme directory (must be inside cursors/generated)",
    )
    arguments = parser.parse_args(argv)
    default_output = arguments.cursor_dir / "generated" / "AureaOS"
    output = arguments.output or default_output

    try:
        generated = build_theme(arguments.cursor_dir, output, arguments.manifest)
    except (OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Cursor theme build failed: {error}", file=sys.stderr)
        return 1

    print(f"Built AureaOS Xcursor theme at: {output}")
    print("Generated cursors: " + ", ".join(generated))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
