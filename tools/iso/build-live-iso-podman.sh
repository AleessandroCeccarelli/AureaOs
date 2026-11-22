#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
BUILD_PARENT="${BUILD_PARENT:-${TMPDIR:-/var/tmp}/aureaos-podman-build}"
readonly BUILD_DIR="$BUILD_PARENT/build"
readonly IMAGE="docker.io/library/debian:trixie-slim"
RESUME="${RESUME:-0}"

usage() {
    cat <<'USAGE'
Build the AureaOS Debian Live ISO inside a disposable Debian Trixie Podman container.

Usage:
  bash tools/iso/build-live-iso-podman.sh

Resume a failed build:
  RESUME=1 BUILD_PARENT=/path/to/previous-build bash tools/iso/build-live-iso-podman.sh

Environment overrides:
  OUTPUT_DIR   Destination directory (default: ~/Documenti)
  BUILD_PARENT Parent for temporary build data (default: /var/tmp/aureaos-podman-build)
  RESUME       Set to 1 to continue an existing failed build directory

The repository is mounted read-only. ISO output and temporary build data stay
outside the repository. Failed builds preserve their workspace and caches.
USAGE
}

if (($#)); then
    case "$1" in
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
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
    echo "The Debian Live build script is missing." >&2
    exit 1
}

if [[ "$OUTPUT_DIR" != /* || "$BUILD_PARENT" != /* ]]; then
    echo "Output and build paths must be absolute." >&2
    exit 2
fi

OUTPUT_DIR="$(realpath -m -- "$OUTPUT_DIR")"
BUILD_PARENT="$(realpath -m -- "$BUILD_PARENT")"
mkdir -p -- "$OUTPUT_DIR" "$BUILD_PARENT"

case "$OUTPUT_DIR/" in
    "$REPO_ROOT/"*) echo "Output directory must be outside the repository." >&2; exit 2 ;;
esac
case "$BUILD_PARENT/" in
    "$REPO_ROOT/"*) echo "Build directory must be outside the repository." >&2; exit 2 ;;
esac

if [[ "$RESUME" == 1 ]]; then
    [[ -d "$BUILD_DIR/config" ]] || {
        echo "Cannot resume: existing build workspace not found at $BUILD_DIR" >&2
        exit 1
    }
elif [[ -e "$BUILD_DIR" || -L "$BUILD_DIR" ]]; then
    echo "Build directory already exists: $BUILD_DIR" >&2
    echo "Set RESUME=1 to continue it, or choose another BUILD_PARENT." >&2
    exit 1
fi

echo "Repository: $REPO_ROOT"
echo "ISO destination: $OUTPUT_DIR/AureaOS.iso"
echo "Temporary build: $BUILD_DIR"
if [[ "$RESUME" == 1 ]]; then
    echo "Mode: resume existing live-build workspace (chroot and caches preserved)"
else
    echo "Mode: fresh build"
fi
echo "Starting rootful Podman; the container needs mount/chroot privileges for live-build."

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
        apt-get install -y live-build git tar sudo ca-certificates xorriso
        git config --global --add safe.directory /workspace
        BUILD_ARGS=(--output-dir /output --build-dir /build-parent/build)
        if [[ "$RESUME" == 1 ]]; then
            BUILD_ARGS+=(--resume)
        fi
        bash tools/iso/build-live-iso.sh "${BUILD_ARGS[@]}"
    '

echo
echo "Container build completed. Verify the ISO and checksum:"
echo "  $OUTPUT_DIR/AureaOS.iso"
echo "  $OUTPUT_DIR/AureaOS.iso.sha256"
