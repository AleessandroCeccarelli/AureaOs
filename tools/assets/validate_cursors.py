#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Validate all source SVGs and metadata for the AureaOS cursor theme."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from cursor_manifest import load_manifest, validate_manifest
from cursor_svg import validate_svg


def validate(cursor_directory: Path) -> list[str]:
    manifest_path = cursor_directory / "manifest.toml"
    try:
        manifest = load_manifest(manifest_path)
    except (OSError, ValueError) as error:
        return [f"cannot load manifest: {error}"]

    errors = validate_manifest(manifest, manifest_path)
    canvas_width = manifest.get("canvas_width", 0)
    canvas_height = manifest.get("canvas_height", 0)

    for entry in manifest.get("cursor", []):
        if not isinstance(entry, dict):
            continue
        source = entry.get("source")
        name = entry.get("name", "unnamed")
        if not isinstance(source, str):
            continue
        source_path = (manifest_path.parent / source).resolve()
        if not source_path.is_relative_to(manifest_path.parent.resolve()):
            continue
        if not source_path.is_file():
            continue
        for error in validate_svg(
            source_path,
            canvas_width,
            canvas_height,
            allow_animation=name == "busy",
        ):
            errors.append(f"{name}: {error}")

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cursor-dir",
        type=Path,
        default=Path(__file__).resolve().parents[2] / "aurea-human-interfaces/cursors",
        help="cursor theme source directory",
    )
    arguments = parser.parse_args(argv)

    errors = validate(arguments.cursor_dir)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("AureaOS cursor assets: manifest and SVG sources are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
