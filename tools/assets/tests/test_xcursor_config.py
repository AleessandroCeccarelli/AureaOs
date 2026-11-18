#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Tests for deterministic xcursorgen configuration."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from xcursor_config import write_cursor_config


class XcursorConfigTests(unittest.TestCase):
    def test_writes_static_cursor_record(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            image = root / "arrow.png"
            image.write_bytes(b"fixture")
            config = root / "default.cursor"

            write_cursor_config(config, [(32, 4, 5, image, 0)])

            self.assertEqual(config.read_text(encoding="utf-8"), "32 4 5 arrow.png 0\n")

    def test_writes_image_path_relative_to_config(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            image_dir = root / "images" / "default"
            image_dir.mkdir(parents=True)
            image = image_dir / "arrow.png"
            image.write_bytes(b"fixture")
            config = root / "default.cursor"

            write_cursor_config(config, [(32, 4, 5, image, 0)])

            self.assertEqual(
                config.read_text(encoding="utf-8"),
                "32 4 5 images/default/arrow.png 0\\n",
            )

    def test_rejects_hotspot_outside_raster(self) -> None
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            image = root / "arrow.png"
            image.write_bytes(b"fixture")
            with self.assertRaises(ValueError):
                write_cursor_config(root / "cursor", [(32, 32, 0, image, 0)])

    def test_rejects_missing_image(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with self.assertRaises(FileNotFoundError):
                write_cursor_config(root / "cursor", [(32, 1, 1, root / "missing.png", 0)])


if __name__ == "__main__":
    unittest.main(verbosity=2)
