#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT

# A destination collision must never replace an existing directory or its data.
mkdir "$WORK/source-collision" "$WORK/destination-collision"
printf 'preserve-me\n' > "$WORK/destination-collision/sentinel"
mv --no-clobber --no-target-directory -- "$WORK/source-collision" "$WORK/destination-collision"
[[ -d "$WORK/source-collision" ]]
[[ "$(cat "$WORK/destination-collision/sentinel")" == "preserve-me" ]]

# A symlink at the final name must remain untouched.
mkdir "$WORK/source-symlink" "$WORK/symlink-target"
ln -s "$WORK/symlink-target" "$WORK/destination-symlink"
mv --no-clobber --no-target-directory -- "$WORK/source-symlink" "$WORK/destination-symlink"
[[ -d "$WORK/source-symlink" ]]
[[ -L "$WORK/destination-symlink" ]]
[[ "$(readlink "$WORK/destination-symlink")" == "$WORK/symlink-target" ]]

# A relative destination is anchored to the directory inode held as cwd,
# even if its pathname is renamed and replaced before publication.
mkdir "$WORK/parent" "$WORK/source-pinned"
cd -- "$WORK/parent"
mv -- "$WORK/parent" "$WORK/parent-moved"
mkdir "$WORK/parent"
mv --no-clobber --no-target-directory -- "$WORK/source-pinned" ./rootfs
[[ -d "$WORK/parent-moved/rootfs" ]]
[[ ! -e "$WORK/parent/rootfs" ]]

echo "All rootfs publication fixture tests passed."
