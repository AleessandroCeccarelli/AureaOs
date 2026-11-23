#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Fixture tests for the fail-closed rootfs symbolic-link audit."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
AUDIT = REPO_ROOT / "tools/security/audit-rootfs-symlinks.py"


class RootfsSymlinkAuditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "rootfs"
        for directory in (
            "etc", "dev", "proc", "run", "var", "usr/lib/tmpfiles.d"
        ):
            (self.root / directory).mkdir(parents=True, exist_ok=True)

    def run_audit(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(AUDIT), str(self.root)],
            text=True,
            capture_output=True,
            check=False,
        )

    def test_accepts_expected_proc_runtime_links(self) -> None:
        links = {
            "etc/mtab": "../proc/self/mounts",
            "dev/fd": "/proc/self/fd",
            "dev/stdin": "/proc/self/fd/0",
            "dev/stdout": "/proc/self/fd/1",
            "dev/stderr": "/proc/self/fd/2",
        }
        for relative, target in links.items():
            (self.root / relative).symlink_to(target)

        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Symlink audit passed.", result.stdout)

    def test_accepts_var_lock_only_with_debian_tmpfiles_rule(self) -> None:
        (self.root / "var/lock").symlink_to("/run/lock")
        config = self.root / "usr/lib/tmpfiles.d/debian.conf"
        config.write_text("d /run/lock 1777 root root - -\n", encoding="utf-8")

        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_accepts_debian_relative_var_lock_link(self) -> None:
        (self.root / "var/lock").symlink_to("../run/lock")
        config = self.root / "usr/lib/tmpfiles.d/debian.conf"
        config.write_text("d /run/lock 1777 root root - -\\n", encoding="utf-8")

        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_rejects_var_lock_without_creation_rule(self) -> None:
        (self.root / "var/lock").symlink_to("/run/lock")
        result = self.run_audit()
        self.assertEqual(result.returncode, 1)
        self.assertIn("unresolved symlink: /var/lock", result.stdout)

    def test_rejects_unexpected_broken_symlink(self) -> None:
        (self.root / "etc/unexpected").symlink_to("missing-target")
        result = self.run_audit()
        self.assertEqual(result.returncode, 1)
        self.assertIn("unresolved symlink: /etc/unexpected", result.stdout)

    def test_rejects_symlink_escape(self) -> None:
        (self.root / "etc/escape").symlink_to("../../outside")
        result = self.run_audit()
        self.assertEqual(result.returncode, 1)
        self.assertIn("escapes rootfs: /etc/escape", result.stdout)

    def test_rejects_unresolved_vconsole_configuration(self) -> None:
        (self.root / "etc/vconsole.conf").symlink_to("default/keyboard")
        result = self.run_audit()
        self.assertEqual(result.returncode, 1)
        self.assertIn("unresolved symlink: /etc/vconsole.conf", result.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
