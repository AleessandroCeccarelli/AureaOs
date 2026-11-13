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
            self.skipTest("run as root to exercise ownership and directory-mode policy")

        self.temporary = tempfile.TemporaryDirectory(prefix="aureaos-publish-test-")
        self.root = Path(self.temporary.name)
        self.destination_parent = self.root / "destination"
        self.source_parent = self.destination_parent / ".staging"
        self.destination_parent.mkdir(mode=0o755)
        self.source_parent.mkdir(mode=0o700)
        os.chmod(self.destination_parent, 0o755)
        os.chmod(self.source_parent, 0o711)

        self.source_name = "aureaos-rootfs.fixture"
        self.destination_name = "rootfs"
        self.source = self.source_parent / self.source_name
        self.source.mkdir(mode=0o700)
        os.chmod(self.source, 0o700)
        (self.source / "sentinel").write_text("verified-rootfs", encoding="utf-8")
        destination_stat = self.destination_parent.stat()
        self.expected_destination_id = f"{destination_stat.st_dev}:{destination_stat.st_ino}"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def publish(self, expected_destination_id: str | None = None) -> str:
        return publish_rootfs(
            str(self.source),
            self.destination_name,
            str(self.destination_parent),
            expected_destination_id or self.expected_destination_id,
            source_parent_path=str(self.source_parent),
        )

    def test_publication_preserves_inode_contents_and_final_mode(self) -> None:
        before = self.source.stat()
        location = self.publish()

        published = self.destination_parent / self.destination_name
        after = published.stat()
        self.assertEqual((before.st_dev, before.st_ino), (after.st_dev, after.st_ino))
        self.assertEqual(stat.S_IMODE(after.st_mode), 0o755)
        self.assertEqual((published / "sentinel").read_text(encoding="utf-8"), "verified-rootfs")
        self.assertFalse(self.source.exists())
        self.assertEqual(location, str(published))

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

    def test_staging_parent_must_be_root_owned_and_mode_0711(self) -> None:
        os.chmod(self.source_parent, 0o700)
        with self.assertRaises(PermissionError):
            self.publish()

    def test_staging_root_must_be_root_owned_and_private(self) -> None:
        os.chmod(self.source, 0o755)
        with self.assertRaises(PermissionError):
            self.publish()

    def test_destination_parent_identity_must_match(self) -> None:
        with self.assertRaises(PermissionError):
            self.publish("1:1")


if __name__ == "__main__":
    unittest.main(verbosity=2)
