#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
BUILD_PARENT="${BUILD_PARENT:-${TMPDIR:-/var/tmp}/aureaos-podman-build}"
readonly BUILD_DIR="$BUILD_PARENT/build"
readonly IMAGE="docker.io/library/debian:trixie-slim"

usage() {
    cat <<'USAGE'
Build the AureaOS Debian Live ISO inside a disposable Debian Trixie Podman container.

Usage:
  bash tools/iso/build-live-iso-podman.sh

Environment overrides:
  OUTPUT_DIR   Destination directory (default: ~/Documenti)
  BUILD_PARENT Parent for temporary build data (default: /var/tmp/aureaos-podman-build)

The repository is mounted read-only. ISO output and temporary build data stay
outside the repository. A failed build preserves its build directory for diagnosis.
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

if [[ -e "$BUILD_DIR" || -L "$BUILD_DIR" ]]; then
    echo "Build directory already exists: $BUILD_DIR" >&2
    echo "Existing data will not be removed. Inspect it or choose another BUILD_PARENT." >&2
    exit 1
fi

echo "Repository: $REPO_ROOT"
echo "ISO destination: $OUTPUT_DIR/AureaOS.iso"
echo "Temporary build: $BUILD_DIR"
echo "Starting rootful Podman; the container needs mount/chroot privileges for live-build."

sudo podman run --rm --privileged --security-opt label=disable \
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
        bash tools/iso/build-live-iso.sh --output-dir /output --build-dir /build-parent/build
    '

echo
echo "Container build completed. Verify the ISO and checksum:"
echo "  $OUTPUT_DIR/AureaOS.iso"
echo "  $OUTPUT_DIR/AureaOS.iso.sha256"
