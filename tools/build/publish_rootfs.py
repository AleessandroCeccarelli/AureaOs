#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Publish a completed rootfs with an atomic, no-replace directory rename."""

from __future__ import annotations

import ctypes
import errno
import os
import stat
import sys


SOURCE_PARENT = "/var/tmp"
STAGING_PREFIX = "aureaos-rootfs."
RENAME_NOREPLACE = 1


def _open_directory(path: str) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    return os.open(path, flags)


def _rename_noreplace(
    source_parent_fd: int,
    source_name: bytes,
    destination_parent_fd: int,
    destination_name: bytes,
) -> None:
    libc = ctypes.CDLL(None, use_errno=True)
    renameat2 = getattr(libc, "renameat2", None)
    if renameat2 is None:
        raise RuntimeError("libc does not expose renameat2; refusing a non-atomic fallback")

    renameat2.argtypes = [
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    ]
    renameat2.restype = ctypes.c_int

    result = renameat2(
        source_parent_fd,
        source_name,
        destination_parent_fd,
        destination_name,
        RENAME_NOREPLACE,
    )
    if result != 0:
        error_number = ctypes.get_errno()
        if error_number in (errno.ENOSYS, errno.EINVAL, errno.EOPNOTSUPP):
            raise RuntimeError(
                "filesystem/kernel does not support atomic RENAME_NOREPLACE; "
                "staging was left untouched"
            )
        raise OSError(error_number, os.strerror(error_number))


def publish_rootfs(\n    source_path: str,\n    destination_name: str,\n    destination_parent: str = ".",\n    source_parent_path: str = SOURCE_PARENT,\n) -> None:\n    source_parent = os.path.dirname(source_path)\n    source_name = os.path.basename(source_path)\n\n    if (\n        source_parent != source_parent_path\n        or os.path.realpath(source_parent_path) != source_parent_path\n        or not source_name.startswith(STAGING_PREFIX)\n    ):
        raise ValueError("source must be a direct child of /var/tmp with the AureaOS staging prefix")
    if not destination_name or destination_name in (".", "..") or os.sep in destination_name:
        raise ValueError("destination must be a single valid basename")

    source_parent_fd = _open_directory(SOURCE_PARENT)
    destination_parent_fd = _open_directory(destination_parent)
    source_fd = -1

    try:
        source_parent_stat = os.fstat(source_parent_fd)
        if source_parent_stat.st_uid != 0 or stat.S_IMODE(source_parent_stat.st_mode) != 0o1777:
            raise PermissionError("/var/tmp must be a real root-owned directory with mode 1777")

        destination_parent_stat = os.fstat(destination_parent_fd)
        if source_parent_stat.st_dev != destination_parent_stat.st_dev:
            raise OSError(errno.EXDEV, "staging and destination must be on the same filesystem")

        source_fd = os.open(
            source_name,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=source_parent_fd,
        )
        source_stat = os.fstat(source_fd)
        if source_stat.st_uid != 0 or stat.S_IMODE(source_stat.st_mode) != 0o700:
            raise PermissionError("staging rootfs must be a root-owned directory with mode 0700")
        if source_stat.st_dev != destination_parent_stat.st_dev:
            raise OSError(errno.EXDEV, "staging and destination must be on the same filesystem")

        try:
            os.stat(destination_name, dir_fd=destination_parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            raise FileExistsError(errno.EEXIST, "destination already exists", destination_name)

        _rename_noreplace(
            source_parent_fd,
            os.fsencode(source_name),
            destination_parent_fd,
            os.fsencode(destination_name),
        )

        published_stat = os.stat(
            destination_name,
            dir_fd=destination_parent_fd,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISDIR(published_stat.st_mode)
            or published_stat.st_dev != source_stat.st_dev
            or published_stat.st_ino != source_stat.st_ino
        ):
            raise RuntimeError("published directory identity does not match the protected staging inode")

        try:
            os.stat(source_name, dir_fd=source_parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            return
        raise RuntimeError("staging entry still exists after publication")
    finally:
        if source_fd >= 0:
            os.close(source_fd)
        os.close(destination_parent_fd)
        os.close(source_parent_fd)


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"Usage: {argv[0]} STAGING_PATH DESTINATION_BASENAME", file=sys.stderr)
        return 2

    try:
        publish_rootfs(argv[1], argv[2])
    except (OSError, RuntimeError, ValueError) as error:
        print(f"Rootfs publication failed: {error}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
