#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests for AureaOS cursor manifest and SVG validation."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

ASSET_TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ASSET_TOOLS))

from cursor_manifest import load_manifest, validate_manifest
from cursor_svg import validate_svg
from validate_cursors import validate


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
CURSOR_DIRECTORY = REPOSITORY_ROOT / "aurea-human-interfaces/cursors"


class CursorAssetTests(unittest.TestCase):
    def test_repository_cursor_theme_is_valid(self) -> None:
        self.assertEqual(validate(CURSOR_DIRECTORY), [])

    def test_manifest_rejects_hotspot_outside_canvas(self) -> None:
        manifest_path = CURSOR_DIRECTORY / "manifest.toml"
        manifest = load_manifest(manifest_path)
        manifest["cursor"][0]["hotspot"] = [64, 5]

        errors = validate_manifest(manifest, manifest_path)
        self.assertTrue(any("hotspot is outside" in error for error in errors))

    def test_manifest_rejects_source_path_escape(self) -> None:
        manifest_path = CURSOR_DIRECTORY / "manifest.toml"
        manifest = load_manifest(manifest_path)
        manifest["cursor"][0]["source"] = "../../outside.svg"

        errors = validate_manifest(manifest, manifest_path)
        self.assertTrue(any("escapes" in error for error in errors))

    def test_svg_rejects_external_resource_reference(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            svg = Path(temporary) / "external.svg"
            svg.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" '
                'viewBox="0 0 64 64"><image href="https://example.invalid/a.png"/></svg>',
                encoding="utf-8",
            )
            errors = validate_svg(svg, 64, 64, allow_animation=False)
            self.assertTrue(any("external SVG resource" in error for error in errors))

    def test_svg_rejects_script_and_event_handlers(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            svg = Path(temporary) / "unsafe.svg"
            svg.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" '
                'viewBox="0 0 64 64" onload="alert(1)"><script>alert(1)</script></svg>',
                encoding="utf-8",
            )
            errors = validate_svg(svg, 64, 64, allow_animation=False)
            self.assertTrue(any("script elements" in error for error in errors))
            self.assertTrue(any("event handler" in error for error in errors))

    def test_static_cursor_rejects_animation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            svg = Path(temporary) / "animated.svg"
            svg.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" '
                'viewBox="0 0 64 64"><animateTransform/></svg>',
                encoding="utf-8",
            )
            errors = validate_svg(svg, 64, 64, allow_animation=False)
            self.assertTrue(any("only permitted" in error for error in errors))


if __name__ == "__main__":
    unittest.main(verbosity=2)
