#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regression tests for the Debian version-control package profile."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "debian/packages/version-control.list"
BOOTSTRAP = ROOT / "tools/build/bootstrap-debian.sh"


class BootstrapVersionControlTests(unittest.TestCase):
    def test_git_is_in_the_version_control_manifest(self) -> None:
        packages = {
            token
            for line in MANIFEST.read_text(encoding="utf-8").splitlines()
            for token in line.split("#", 1)[0].split()
        }
        self.assertEqual(packages, {"git"})

    def test_bootstrap_loads_the_version_control_manifest(self) -> None:
        source = BOOTSTRAP.read_text(encoding="utf-8")
        self.assertIn("version-control.list", source)

    def test_git_is_installed_through_the_common_apt_package_flow(self) -> None:
        source = BOOTSTRAP.read_text(encoding="utf-8")
        self.assertIn(
            'run_in_rootfs apt-get --download-only install --yes --no-install-recommends -- "${packages[@]}"',
            source,
        )
        self.assertIn(
            'apt-get install --yes --no-download --no-install-recommends -- "${packages[@]}"',
            source,
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
