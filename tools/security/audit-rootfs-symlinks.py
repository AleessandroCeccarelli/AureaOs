#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Fail-closed audit of rootfs symbolic links, including known runtime links."""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path


RUNTIME_PROC_LINKS = {
    "etc/mtab": "../proc/self/mounts",
    "dev/fd": "/proc/self/fd",
    "dev/stdin": "/proc/self/fd/0",
    "dev/stdout": "/proc/self/fd/1",
    "dev/stderr": "/proc/self/fd/2",
}


def within_root(root: str, candidate: str) -> bool:
    try:
        return os.path.commonpath((root, candidate)) == root
    except ValueError:
        return False


def has_run_lock_rule(root: str) -> bool:
    config = Path(root) / "usr/lib/tmpfiles.d/debian.conf"
    try:
        content = config.read_text(encoding="utf-8")
    except OSError:
        return False
    return any(
        re.match(r"^\s*d\s+/run/lock(?:\s|$)", line)
        for line in content.splitlines()
    )


def runtime_exception(root: str, relative: str, target: str) -> bool:
    if RUNTIME_PROC_LINKS.get(relative) == target:
        return os.path.isdir(os.path.join(root, "proc"))
    if relative == "var/lock" and target in {"/run/lock", "../run/lock"}:
        return (
            os.path.isdir(os.path.join(root, "run"))
            and has_run_lock_rule(root)
        )
    return False


def main() -> int:
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} ROOTFS", file=sys.stderr)
        return 2

    supplied = sys.argv[1]
    if os.path.islink(supplied) or not os.path.isdir(supplied):
        print(f"ERROR: invalid rootfs directory: {supplied}", file=sys.stderr)
        return 2

    root = os.path.realpath(supplied)
    failures = 0
    checked = 0

    def walk_error(error: OSError) -> None:
        nonlocal failures
        failures += 1
        print(f"FAIL: traversal error: {error}", file=sys.stderr)

    for directory, dirnames, filenames in os.walk(
        root, topdown=True, followlinks=False, onerror=walk_error
    ):
        entries = list(dirnames) + list(filenames)
        for name in entries:
            path = os.path.join(directory, name)
            if not os.path.islink(path):
                continue

            checked += 1
            relative = os.path.relpath(path, root)
            target = os.readlink(path)
            candidate = os.path.normpath(
                os.path.join(root, target.lstrip("/"))
                if os.path.isabs(target)
                else os.path.join(os.path.dirname(path), target)
            )

            if not within_root(root, candidate):
                failures += 1
                print(f"FAIL: symlink escapes rootfs: /{relative} -> {target}")
                continue

            if os.path.exists(candidate):
                print(f"PASS: /{relative} -> {target}")
                continue

            if runtime_exception(root, relative, target):
                print(f"PASS: expected runtime link /{relative} -> {target}")
                continue

            failures += 1
            print(f"FAIL: unresolved symlink: /{relative} -> {target}")

    print(f"Symlinks checked: {checked}")
    if failures:
        print(f"Symlink audit FAILED: {failures} issue(s).", file=sys.stderr)
        return 1

    print("Symlink audit passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
