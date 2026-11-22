#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
readonly IMAGE="docker.io/library/debian:trixie-slim"

OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
BUILD_PARENT="${BUILD_PARENT:-${TMPDIR:-/var/tmp}/aureaos-podman-build}"
RESUME="${RESUME:-0}"
readonly BUILD_DIR="$BUILD_PARENT/build"

usage() {
    cat <<'USAGE'
Build AureaOS Debian Live ISO in a disposable Debian Trixie container.

Usage:
  bash tools/iso/build-live-iso-podman.sh

Resume:
  RESUME=1 BUILD_PARENT=/path/to/previous-build bash tools/iso/build-live-iso-podman.sh

Environment:
  OUTPUT_DIR   ISO destination (default: ~/Documenti)
  BUILD_PARENT Parent of the persistent build workspace
  RESUME       0 for fresh build, 1 to resume a failed build
USAGE
}

if (($#)); then
    case "$1" in --help|-h) usage; exit 0 ;; *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;; esac
fi

for command_name in podman git realpath; do
    command -v "$command_name" >/dev/null 2>&1 || {
        echo "Required host command not found: $command_name" >&2
        exit 1
    }
done

[[ "$RESUME" == 0 || "$RESUME" == 1 ]] || {
    echo "RESUME must be 0 or 1." >&2
    exit 2
}
[[ -f "$REPO_ROOT/.git/HEAD" || -f "$REPO_ROOT/.git" ]] || {
    echo "A Git working tree is required: $REPO_ROOT" >&2
    exit 1
}
[[ -f "$SCRIPT_DIR/build-live-iso.sh" ]] || {
    echo "Debian Live build script is missing." >&2
    exit 1
}

if [[ "$OUTPUT_DIR" != /* || "$BUILD_PARENT" != /* ]]; then
    echo "Output and build paths must be absolute." >&2
    exit 2
fi

OUTPUT_DIR="$(realpath -m -- "$OUTPUT_DIR")"
BUILD_PARENT="$(realpath -m -- "$BUILD_PARENT")"
mkdir -p -- "$OUTPUT_DIR" "$BUILD_PARENT"

case "$OUTPUT_DIR/" in "$REPO_ROOT/"*) echo "Output directory must be outside the repository." >&2; exit 2 ;; esac
case "$BUILD_PARENT/" in "$REPO_ROOT/"*) echo "Build directory must be outside the repository." >&2; exit 2 ;; esac

if [[ "$RESUME" == 1 ]]; then
    [[ -d "$BUILD_DIR/config" ]] || {
        echo "Cannot resume: expected Debian Live workspace config at $BUILD_DIR/config" >&2
        exit 1
    }
elif [[ -e "$BUILD_DIR" || -L "$BUILD_DIR" ]]; then
    echo "Build directory already exists: $BUILD_DIR" >&2
    echo "Set RESUME=1 to continue it, or select another BUILD_PARENT." >&2
    exit 1
fi

echo "Repository: $REPO_ROOT"
echo "Debian Live workspace: $BUILD_DIR"
echo "ISO destination: $OUTPUT_DIR/AureaOS.iso"
echo "Mode: $([[ "$RESUME" == 1 ]] && echo resume || echo fresh)"
echo "Starting rootful Podman for live-build mount/chroot operations."

sudo podman run --rm --privileged --security-opt label=disable \
    --env "RESUME=$RESUME" \
    --volume "$REPO_ROOT:/workspace:ro" \
    --volume "$OUTPUT_DIR:/output:rw" \
    --volume "$BUILD_PARENT:/build-parent:rw" \
    --workdir /workspace \
    "$IMAGE" bash -lc '
        set -Eeuo pipefail
        export DEBIAN_FRONTEND=noninteractive

        apt-get update
        apt-get install -y --no-install-recommends \
            live-build git tar sudo ca-certificates xorriso

        git config --global --add safe.directory /workspace

        BUILD_ARGS=(--output-dir /output --build-dir /build-parent/build)
        if [[ "$RESUME" == 1 ]]; then
            BUILD_ARGS+=(--resume)
        fi

        bash tools/iso/build-live-iso.sh "${BUILD_ARGS[@]}"
    '

echo
echo "Container build completed. Expected artifacts:"
echo "  $OUTPUT_DIR/AureaOS.iso"
echo "  $OUTPUT_DIR/AureaOS.iso.sha256"
