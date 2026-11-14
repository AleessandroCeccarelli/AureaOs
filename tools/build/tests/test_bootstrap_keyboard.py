#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regression tests for the non-interactive Italian keyboard configuration."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "debian/packages/base.list"
KEYBOARD = ROOT / "debian/config/keyboard"
LOCALE = ROOT / "debian/config/locale"
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

    def test_system_locale_is_English_utf8(self) -> None:
        self.assertEqual(
            LOCALE.read_text(encoding="utf-8").strip(),
            "LANG=en_US.UTF-8",
        )

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
        self.assertIn('LOCALE_CONFIG="$REPO_ROOT/debian/config/locale"', source)
        self.assertIn('locale-gen en_US.UTF-8', source)
        self.assertIn('"$LOCALE_CONFIG" "$ROOTFS_WORK/etc/locale.conf"', source)
        self.assertIn("locale -a", source)


if __name__ == "__main__":
    unittest.main(verbosity=2)
