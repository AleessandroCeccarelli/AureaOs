#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
readonly IMAGE="docker.io/library/debian:trixie-slim"

OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
RESUME="${RESUME:-0}"
RESUME_FROM_COMMIT="${RESUME_FROM_COMMIT:-}"
readonly SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
readonly SOURCE_COMMIT_SHORT="${SOURCE_COMMIT:0:12}"
SOURCE_BRANCH="$(git -C "$REPO_ROOT" branch --show-current)"
SOURCE_BRANCH="${SOURCE_BRANCH:-snapshot}"
SOURCE_REMOTE="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || true)"
BUILD_PARENT="${BUILD_PARENT:-${TMPDIR:-/var/tmp}/aureaos-podman-build-$SOURCE_COMMIT_SHORT}"
readonly BUILD_DIR="$BUILD_PARENT/build"

usage() {
    cat <<'USAGE'
Build AureaOS Debian Live ISO in a disposable Debian Trixie container.

Usage:
  bash tools/iso/build-live-iso-podman.sh

Resume an interrupted build:
  RESUME=1 BUILD_PARENT=/path/to/previous-build bash tools/iso/build-live-iso-podman.sh

Resume final ISO assembly after a factory-only script update:
  RESUME=1 RESUME_FROM_COMMIT=<workspace-source-commit> \
    BUILD_PARENT=/path/to/previous-build bash tools/iso/build-live-iso-podman.sh

Environment:
  OUTPUT_DIR          ISO destination (default: ~/Documenti)
  BUILD_PARENT        Parent of the persistent build workspace
  RESUME              0 for fresh build, 1 to resume a failed build
  RESUME_FROM_COMMIT  Exact old workspace marker; permits only a completed-rootfs resume
USAGE
}

if (($#)); then
    case "$1" in --help|-h) usage; exit 0 ;; *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;; esac
fi

for command_name in podman git realpath flock sudo tar mktemp; do
    command -v "$command_name" >/dev/null 2>&1 || { echo "Required host command not found: $command_name" >&2; exit 1; }
done

[[ "$RESUME" == 0 || "$RESUME" == 1 ]] || { echo "RESUME must be 0 or 1." >&2; exit 2; }
if [[ -n "$RESUME_FROM_COMMIT" ]]; then
    [[ "$RESUME" == 1 && "$RESUME_FROM_COMMIT" =~ ^[0-9a-f]{40}$ ]] || {
        echo "RESUME_FROM_COMMIT is valid only for resume and must be a full 40-character SHA." >&2
        exit 2
    }
fi

if [[ -n "$(git -C "$REPO_ROOT" status --porcelain --untracked-files=all)" ]]; then
    echo "The repository has uncommitted or untracked changes." >&2
    echo "Commit the intended AureaOS state before creating a reproducible Live ISO." >&2
    exit 1
fi

[[ -f "$SCRIPT_DIR/build-live-iso.sh" ]] || { echo "Debian Live build script is missing." >&2; exit 1; }
if [[ "$OUTPUT_DIR" != /* || "$BUILD_PARENT" != /* ]]; then
    echo "Output and build paths must be absolute." >&2
    exit 2
fi
OUTPUT_DIR="$(realpath -m -- "$OUTPUT_DIR")"
BUILD_PARENT="$(realpath -m -- "$BUILD_PARENT")"
mkdir -p -- "$OUTPUT_DIR" "$BUILD_PARENT"

exec 9>"$BUILD_PARENT/.aureaos-live-build.lock"
if ! flock -n 9; then
    echo "Another AureaOS ISO build is already using: $BUILD_PARENT" >&2
    exit 1
fi

case "$OUTPUT_DIR/" in "$REPO_ROOT/"*) echo "Output directory must be outside the repository." >&2; exit 2 ;; esac
case "$BUILD_PARENT/" in "$REPO_ROOT/"*) echo "Build directory must be outside the repository." >&2; exit 2 ;; esac

if [[ "$RESUME" == 1 ]]; then
    [[ -d "$BUILD_DIR/config" ]] || { echo "Cannot resume: expected Debian Live workspace config at $BUILD_DIR/config" >&2; exit 1; }
elif [[ -e "$BUILD_DIR" || -L "$BUILD_DIR" ]]; then
    echo "Build directory already exists: $BUILD_DIR" >&2
    echo "Set RESUME=1 to continue it, or select another BUILD_PARENT." >&2
    exit 1
fi

# The worktree may be linked: its .git file can point outside the mounted tree.
# Prepare a standalone immutable Git archive on Fedora; no Git metadata enters Podman.
SOURCE_SNAPSHOT="$(mktemp -d "${TMPDIR:-/var/tmp}/aureaos-iso-snapshot-$SOURCE_COMMIT_SHORT.XXXXXX")"
trap 'rm -rf -- "$SOURCE_SNAPSHOT"' EXIT
git -C "$REPO_ROOT" archive --format=tar "$SOURCE_COMMIT" | tar -xf - -C "$SOURCE_SNAPSHOT"

echo "Repository: $REPO_ROOT"
echo "Source snapshot commit: $SOURCE_COMMIT"
echo "Debian Live workspace: $BUILD_DIR"
echo "ISO destination: $OUTPUT_DIR/AureaOS.iso"
echo "Mode: $([[ "$RESUME" == 1 ]] && echo resume || echo fresh)"
[[ -z "$RESUME_FROM_COMMIT" ]] || echo "Finalization-only resume from OS payload: $RESUME_FROM_COMMIT"

sudo podman run --rm --privileged --security-opt label=disable \
    --env "RESUME=$RESUME" \
    --env "AUREAOS_COMMIT=$SOURCE_COMMIT" \
    --env "AUREAOS_BRANCH=$SOURCE_BRANCH" \
    --env "AUREAOS_REMOTE=$SOURCE_REMOTE" \
    --env AUREAOS_WORKTREE_CLEAN=true \
    --env "AUREAOS_RESUME_FROM_COMMIT=$RESUME_FROM_COMMIT" \
    --volume "$SOURCE_SNAPSHOT:/workspace:ro" \
    --volume "$OUTPUT_DIR:/output:rw" \
    --volume "$BUILD_PARENT:/build-parent:rw" \
    --workdir /workspace \
    "$IMAGE" bash -lc '
        set -Eeuo pipefail
        export DEBIAN_FRONTEND=noninteractive
        apt-get update
        apt-get install -y --no-install-recommends \
            live-build tar sudo ca-certificates xorriso python3 \
            systemd-boot-efi systemd-ukify mtools dosfstools \
            squashfs-tools initramfs-tools-core librsvg2-bin x11-apps

        BUILD_ARGS=(--output-dir /output --build-dir /build-parent/build)
        if [[ "$RESUME" == 1 ]]; then BUILD_ARGS+=(--resume); fi
        bash tools/iso/build-live-iso.sh "${BUILD_ARGS[@]}"
    '

echo
echo "Container build completed. Expected artifacts:"
echo "  $OUTPUT_DIR/AureaOS.iso"
echo "  $OUTPUT_DIR/AureaOS.iso.sha256"
