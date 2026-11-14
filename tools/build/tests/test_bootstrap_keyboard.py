#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regression tests for the non-interactive Italian keyboard configuration."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "debian/packages/base.list"
KEYBOARD = ROOT / "debian/config/keyboard"
BOOTSTRAP = ROOT / "tools/build/bootstrap-debian.sh"


class BootstrapKeyboardTests(unittest.TestCase):
    def test_keyboard_configuration_package_is_in_base_manifest(self) -> None:
        packages = {
            token
            for line in MANIFEST.read_text(encoding="utf-8").splitlines()
            for token in line.split("#", 1)[0].split()
        }
        self.assertIn("keyboard-configuration", packages)

    def test_console_layout_is_italian_and_configuration_is_noninteractive(self) -> None:
        config = KEYBOARD.read_text(encoding="utf-8")
        self.assertIn('XKBMODEL="pc105"', config)
        self.assertIn('XKBLAYOUT="it"', config)
        self.assertIn('XKBVARIANT=""', config)
        self.assertIn('XKBOPTIONS=""', config)
        self.assertIn('BACKSPACE="guess"', config)

    def test_bootstrap_installs_and_verifies_the_versioned_configuration(self) -> None:
        source = BOOTSTRAP.read_text(encoding="utf-8")
        self.assertIn('KEYBOARD_CONFIG="$REPO_ROOT/debian/config/keyboard"', source)
        self.assertIn(
            '"$KEYBOARD_CONFIG" "$ROOTFS_WORK/etc/default/keyboard"',
            source,
        )
        self.assertIn('dpkg-query -W -f="$DPkg_QUERY_STATUS_FORMAT" keyboard-configuration', source)
        self.assertIn('cmp -s "$KEYBOARD_CONFIG" "$ROOTFS_WORK/etc/default/keyboard"', source)
        self.assertIn('sudo readlink -- "$ROOTFS_WORK/etc/vconsole.conf"', source)
        self.assertIn('audit-rootfs-symlinks.py', source)


if __name__ == "__main__":
    unittest.main(verbosity=2)
