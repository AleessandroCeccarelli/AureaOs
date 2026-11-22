#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
readonly IMAGE_NAME="AureaOS"
BUILD_ROOT="${BUILD_ROOT:-${TMPDIR:-/var/tmp}/aureaos-live-build}"

usage() {
    cat <<'USAGE'
Build the minimal AureaOS Debian Live ISO.

Usage:
  bash tools/iso/build-live-iso.sh [--output-dir PATH] [--build-dir PATH]

By default, the ISO and checksum are saved in ~/Documenti as
AureaOS.iso and AureaOS.iso.sha256. Build files stay outside the repository.
The script builds an amd64 Debian 13 (Trixie) hybrid ISO and never writes
to USB devices or modifies the host operating system.
USAGE
}

while (($#)); do
    case "$1" in
        --output-dir)
            [[ $# -ge 2 && -n "$2" ]] || { echo "--output-dir requires a path" >&2; exit 2; }
            OUTPUT_DIR="$2"; shift 2 ;;
        --build-dir)
            [[ $# -ge 2 && -n "$2" ]] || { echo "--build-dir requires a path" >&2; exit 2; }
            BUILD_ROOT="$2"; shift 2 ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

for command_name in lb git tar sudo sha256sum; do
    command -v "$command_name" >/dev/null 2>&1 || {
        echo "Required build command not found: $command_name" >&2
        exit 1
    }
done

[[ -f "$REPO_ROOT/Cargo.lock" ]] || { echo "Cargo.lock is missing." >&2; exit 1; }
[[ -f "$SCRIPT_DIR/package-lists/aureaos.list.chroot" ]] || { echo "Live package list is missing." >&2; exit 1; }
[[ -f "$SCRIPT_DIR/hooks/010-build-aureaos.hook.chroot" ]] || {
    echo "The chroot build hook is missing: tools/iso/hooks/010-build-aureaos.hook.chroot" >&2
    exit 1
}

if [[ "$BUILD_ROOT" != /* || "$OUTPUT_DIR" != /* ]]; then
    echo "Build and output paths must be absolute." >&2
    exit 2
fi

mkdir -p -- "$OUTPUT_DIR"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd -P)"
case "$OUTPUT_DIR/" in
    "$REPO_ROOT/"*)
        echo "Output directory must be outside the Git repository: $OUTPUT_DIR" >&2
        exit 2
        ;;
esac

if [[ -e "$BUILD_ROOT" || -L "$BUILD_ROOT" ]]; then
    echo "Build directory already exists: $BUILD_ROOT" >&2
    echo "Choose a new path with --build-dir; existing data will not be removed." >&2
    exit 1
fi
mkdir -p -- "$BUILD_ROOT"
BUILD_ROOT="$(cd -- "$BUILD_ROOT" && pwd -P)"
case "$BUILD_ROOT/" in
    "$REPO_ROOT/"*)
        echo "Build directory must be outside the Git repository: $BUILD_ROOT" >&2
        exit 2
        ;;
esac
CONFIG_DIR="$BUILD_ROOT/config"
SOURCE_DIR="$CONFIG_DIR/config/includes.chroot/usr/src/aureaos"

cleanup() {
    local status=$?
    if ((status == 0)); then
        sudo rm -rf -- "$BUILD_ROOT" || {
            echo "Warning: unable to remove temporary build directory: $BUILD_ROOT" >&2
        }
    else
        echo "Build failed; preserving diagnostics and partial build at: $BUILD_ROOT" >&2
    fi
}
trap cleanup EXIT

mkdir -p "$CONFIG_DIR/config/package-lists" "$CONFIG_DIR/config/hooks/live" "$SOURCE_DIR"
cp "$SCRIPT_DIR/package-lists/aureaos.list.chroot" "$CONFIG_DIR/config/package-lists/aureaos.list.chroot"
cp "$SCRIPT_DIR/hooks/010-build-aureaos.hook.chroot" "$CONFIG_DIR/config/hooks/live/010-build-aureaos.hook.chroot"
chmod 0755 "$CONFIG_DIR/config/hooks/live/010-build-aureaos.hook.chroot"

echo "Preparing the exact committed AureaOS source from HEAD..."
git -C "$REPO_ROOT" archive --format=tar HEAD Cargo.toml Cargo.lock components |
    tar -xf - -C "$SOURCE_DIR"

cd "$CONFIG_DIR"
lb config \
    --mode debian \
    --distribution trixie \
    --architectures amd64 \
    --binary-image iso-hybrid \
    --bootloaders "syslinux grub-efi" \
    --archive-areas "main contrib non-free non-free-firmware" \
    --firmware-chroot true \
    --firmware-binary true \
    --linux-packages linux-image \
    --linux-flavours amd64 \
    --debian-installer none \
    --apt-recommends false \
    --iso-application "AureaOS Native Blue Test" \
    --iso-volume "AUREAOS_BLUE_TEST" \
    --image-name "$IMAGE_NAME"

echo "Building Debian Live ISO. This can take a while on the first run..."
sudo lb build

ISO_PATH="$(find "$CONFIG_DIR" -maxdepth 1 -type f -name '*.iso' -print -quit)"
if [[ -z "$ISO_PATH" || ! -s "$ISO_PATH" ]]; then
    echo "No non-empty ISO image was produced." >&2
    find "$CONFIG_DIR" -maxdepth 2 -type f -name '*.iso' -print >&2
    exit 1
fi

DESTINATION="$OUTPUT_DIR/AureaOS.iso"
install -m 0644 "$ISO_PATH" "$DESTINATION"
(
    cd "$OUTPUT_DIR"
    sha256sum AureaOS.iso > AureaOS.iso.sha256
)

echo
echo "AureaOS Live ISO created:"
echo "$DESTINATION"
echo "SHA256:"
cat "$DESTINATION.sha256"
