#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Root-only tests for atomic rootfs publication."""

from __future__ import annotations

import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from publish_rootfs import publish_rootfs


class RootfsPublicationTests(unittest.TestCase):
    def setUp(self) -> None:
        if os.geteuid() != 0:
            self.skipTest("run as root to exercise ownership and sticky-directory policy")

        self.temporary = tempfile.TemporaryDirectory(prefix="aureaos-publish-test-")
        self.root = Path(self.temporary.name)
        self.source_parent = self.root / "source"
        self.destination_parent = self.root / "destination"
        self.source_parent.mkdir(mode=0o700)
        self.destination_parent.mkdir(mode=0o755)
        os.chmod(self.source_parent, 0o1777)
        os.chmod(self.destination_parent, 0o755)

        self.source_name = "aureaos-rootfs.fixture"
        self.destination_name = "rootfs"
        self.source = self.source_parent / self.source_name
        self.source.mkdir(mode=0o700)
        os.chmod(self.source, 0o700)
        (self.source / "sentinel").write_text("verified-rootfs", encoding="utf-8")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def publish(self) -> None:
        publish_rootfs(
            str(self.source),
            self.destination_name,
            destination_parent=str(self.destination_parent),
            source_parent_path=str(self.source_parent),
        )

    def test_publication_preserves_staging_inode_and_contents(self) -> None:
        before = self.source.stat()
        self.publish()

        published = self.destination_parent / self.destination_name
        after = published.stat()
        self.assertEqual((before.st_dev, before.st_ino), (after.st_dev, after.st_ino))
        self.assertEqual(stat.S_IMODE(after.st_mode), 0o755)
        self.assertEqual((published / "sentinel").read_text(encoding="utf-8"), "verified-rootfs")
        self.assertFalse(self.source.exists())

    def test_existing_directory_is_never_replaced(self) -> None:
        target = self.destination_parent / self.destination_name
        target.mkdir()
        (target / "keep").write_text("do-not-overwrite", encoding="utf-8")

        with self.assertRaises(FileExistsError):
            self.publish()

        self.assertTrue(self.source.is_dir())
        self.assertEqual((target / "keep").read_text(encoding="utf-8"), "do-not-overwrite")

    def test_existing_symlink_is_never_replaced(self) -> None:
        target = self.destination_parent / self.destination_name
        target.symlink_to(self.root / "outside")

        with self.assertRaises(FileExistsError):
            self.publish()

        self.assertTrue(target.is_symlink())
        self.assertTrue(self.source.is_dir())

    def test_source_parent_must_be_root_owned_sticky_directory(self) -> None:
        os.chmod(self.source_parent, 0o755)
        with self.assertRaises(PermissionError):
            self.publish()

    def test_staging_directory_must_be_root_owned_and_private(self) -> None:
        os.chmod(self.source, 0o755)
        with self.assertRaises(PermissionError):
            self.publish()


if __name__ == "__main__":
    unittest.main(verbosity=2)
