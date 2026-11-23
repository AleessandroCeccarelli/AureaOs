#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Create a root-owned staging parent through pinned directory descriptors."""

from __future__ import annotations

import os
import secrets
import stat
import sys


def _open_directory(path: str) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    return os.open(path, flags)


def create_staging_parent(destination_parent: str, expected_parent_id: str) -> tuple[str, str]:
    if os.path.realpath(destination_parent) != destination_parent:
        raise ValueError("destination parent path must be canonical")

    try:
        expected_device, expected_inode = (
            int(value) for value in expected_parent_id.split(":", maxsplit=1)
        )
    except (TypeError, ValueError) as error:
        raise ValueError("expected parent identity must be DEV:INO") from error

    parent_fd = _open_directory(destination_parent)
    try:
        parent_stat = os.fstat(parent_fd)
        if (parent_stat.st_dev, parent_stat.st_ino) != (expected_device, expected_inode):
            raise PermissionError("destination parent identity changed before staging creation")

        for _ in range(32):
            name = f".aureaos-build.{secrets.token_hex(8)}"
            try:
                os.mkdir(name, 0o700, dir_fd=parent_fd)
            except FileExistsError:
                continue

            stage_fd = -1
            try:
                created_stat = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
                stage_fd = os.open(
                    name,
                    os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
                    dir_fd=parent_fd,
                )
                opened_stat = os.fstat(stage_fd)
                if (created_stat.st_dev, created_stat.st_ino) != (
                    opened_stat.st_dev,
                    opened_stat.st_ino,
                ):
                    raise RuntimeError("staging directory entry changed while it was opened")
                if opened_stat.st_uid != 0:
                    raise PermissionError("staging parent is not owned by root")

                # Change permissions through the pinned descriptor, never by
                # following a path in the user-writable destination parent.
                os.fchmod(stage_fd, 0o711)
                final_stat = os.fstat(stage_fd)
                if (
                    final_stat.st_uid != 0
                    or stat.S_IMODE(final_stat.st_mode) != 0o711
                    or (final_stat.st_dev, final_stat.st_ino)
                    != (opened_stat.st_dev, opened_stat.st_ino)
                ):
                    raise RuntimeError("staging parent identity or permissions changed")

                return name, f"{final_stat.st_dev}:{final_stat.st_ino}"
            finally:
                if stage_fd >= 0:
                    os.close(stage_fd)

        raise FileExistsError("unable to allocate a unique rootfs staging directory")
    finally:
        os.close(parent_fd)


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(
            f"Usage: {argv[0]} DESTINATION_PARENT EXPECTED_PARENT_DEV:INO",
            file=sys.stderr,
        )
        return 2

    try:
        name, identity = create_staging_parent(argv[1], argv[2])
    except (OSError, RuntimeError, ValueError) as error:
        print(f"Rootfs staging creation failed: {error}", file=sys.stderr)
        return 1

    print(f"{name} {identity}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
