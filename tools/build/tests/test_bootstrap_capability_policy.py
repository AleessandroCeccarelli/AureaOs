#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regression tests for the bootstrap container capability boundary."""

from __future__ import annotations

import unittest
from pathlib import Path


BOOTSTRAP = Path(__file__).resolve().parents[1] / "bootstrap-debian.sh"


class BootstrapCapabilityPolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = BOOTSTRAP.read_text(encoding="utf-8")

    def test_only_debootstrap_second_stage_may_retain_sys_admin(self) -> None:
        self.assertIn(
            'run_in_rootfs_with_network_mode shared no "$@"',
            self.source,
        )
        self.assertIn(
            'run_in_rootfs_with_network_mode private no "$@"',
            self.source,
        )
        self.assertIn(
            'run_in_rootfs_with_network_mode private yes "$@"',
            self.source,
        )
        self.assertIn(
            'if [[ "$allow_mount_capability" != "yes" ]]; then\n'
            '    capability_options+=(--drop-capability=CAP_SYS_ADMIN)\n'
            "  fi",
            self.source,
        )

    def test_second_stage_uses_the_narrow_capability_exception(self) -> None:
        self.assertIn(
            "run_debootstrap_second_stage /debootstrap/debootstrap --second-stage",
            self.source,
        )
        self.assertIn(
            "run_in_rootfs_private_network env DEBIAN_FRONTEND=noninteractive",
            self.source,
        )
        self.assertIn(
            "run_in_rootfs_private_network apt-get clean",
            self.source,
        )

    def test_nspawn_refreshes_sudo_credentials_before_privileged_execution(self) -> None:
        self.assertIn("  sudo -v\\n\\n  sudo systemd-nspawn", self.source)

    def test_other_dangerous_capabilities_remain_dropped(self) -> None:
        for capability in (
            "CAP_SYS_MODULE",
            "CAP_SYS_RAWIO",
            "CAP_SYS_BOOT",
            "CAP_SYS_PTRACE",
            "CAP_NET_ADMIN",
            "CAP_NET_RAW",
        ):
            with self.subTest(capability=capability):
                self.assertIn(f"--drop-capability={capability}", self.source)


if __name__ == "__main__":
    unittest.main(verbosity=2)
