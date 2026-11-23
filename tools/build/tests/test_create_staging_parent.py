#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Root-only tests for pinned rootfs staging-parent creation."""

from __future__ import annotations

import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from create_staging_parent import create_staging_parent


class StagingParentTests(unittest.TestCase):
    def setUp(self) -> None:
        if os.geteuid() != 0:
            self.skipTest("run as root to exercise staging ownership")
        self.temporary = tempfile.TemporaryDirectory(prefix="aureaos-stage-test-")
        self.parent = Path(self.temporary.name) / "destination"
        self.parent.mkdir(mode=0o755)
        os.chmod(self.parent, 0o755)
        parent_stat = self.parent.stat()
        self.parent_id = f"{parent_stat.st_dev}:{parent_stat.st_ino}"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_creates_private_root_owned_staging_parent(self) -> None:
        name, identity = create_staging_parent(str(self.parent), self.parent_id)
        staging = self.parent / name
        staging_stat = staging.stat()

        self.assertTrue(name.startswith(".aureaos-build."))
        self.assertEqual(identity, f"{staging_stat.st_dev}:{staging_stat.st_ino}")
        self.assertEqual(staging_stat.st_uid, 0)
        self.assertEqual(stat.S_IMODE(staging_stat.st_mode), 0o711)

    def test_rejects_changed_destination_parent_identity(self) -> None:
        with self.assertRaises(PermissionError):
            create_staging_parent(str(self.parent), "1:1")

    def test_rejects_symlink_destination_parent(self) -> None:
        link = self.parent.parent / "destination-link"
        link.symlink_to(self.parent, target_is_directory=True)
        with self.assertRaises(ValueError):
            create_staging_parent(str(link), self.parent_id)


if __name__ == "__main__":
    unittest.main(verbosity=2)
