#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
readonly IMAGE_NAME="AureaOS"

OUTPUT_DIR="${OUTPUT_DIR:-$HOME/Documenti}"
BUILD_ROOT="${BUILD_ROOT:-${TMPDIR:-/var/tmp}/aureaos-live-build}"
RESUME=false

usage() {
    cat <<'USAGE'
Build the AureaOS Debian Trixie Live ISO.

Usage:
  bash tools/iso/build-live-iso.sh [--output-dir PATH] [--build-dir PATH] [--resume]

The build directory is the live-build workspace root. Its configuration is
stored in BUILD_ROOT/config, following the Debian Live Manual layout.

A failed build is preserved. Use --resume with the same build directory to
continue without discarding downloaded packages or the partial chroot.
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
        --resume) RESUME=true; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

for command_name in lb git tar sudo sha256sum realpath find install; do
    command -v "$command_name" >/dev/null 2>&1 || {
        echo "Required build command not found: $command_name" >&2
        exit 1
    }
done

[[ -f "$REPO_ROOT/Cargo.lock" && -f "$REPO_ROOT/Cargo.toml" ]] || {
    echo "AureaOS Cargo workspace is incomplete." >&2
    exit 1
}
for required in \
    "$SCRIPT_DIR/package-lists/aureaos.list.chroot" \
    "$SCRIPT_DIR/hooks/010-build-aureaos.hook.chroot" \
    "$SCRIPT_DIR/scripts/build-systemd-boot-live-iso.sh" \
    "$SCRIPT_DIR/os-release" \
    "$SCRIPT_DIR/systemd/aurea-desktop.service" \
    "$SCRIPT_DIR/pam/aurea-desktop" \
    "$SCRIPT_DIR/scripts/aurea-live-session" \
    "$SCRIPT_DIR/tests/validate-chroot-boot.sh" \
    "$SCRIPT_DIR/tests/validate-live-image.sh"; do
    [[ -f "$required" ]] || { echo "Required ISO input is missing: $required" >&2; exit 1; }
done

if [[ "$BUILD_ROOT" != /* || "$OUTPUT_DIR" != /* ]]; then
    echo "Build and output paths must be absolute." >&2
    exit 2
fi

OUTPUT_DIR="$(realpath -m -- "$OUTPUT_DIR")"
BUILD_ROOT="$(realpath -m -- "$BUILD_ROOT")"

case "$OUTPUT_DIR/" in "$REPO_ROOT/"*) echo "Output directory must be outside the Git repository." >&2; exit 2 ;; esac
case "$BUILD_ROOT/" in "$REPO_ROOT/"*) echo "Build directory must be outside the Git repository." >&2; exit 2 ;; esac

mkdir -p -- "$OUTPUT_DIR"

# Migrate the previous wrapper's nested workspace (BUILD_ROOT/config) in place.
# Keep the complete chroot, .build stage markers, package cache and diagnostics.
if "$RESUME" && [[ -d "$BUILD_ROOT/config/config" && -d "$BUILD_ROOT/config/chroot" && ! -d "$BUILD_ROOT/chroot" ]]; then
    LEGACY_ROOT="$BUILD_ROOT/config"
    MIGRATION_ROOT="${BUILD_ROOT}.workspace-migration"
    if [[ -n "$(find "$BUILD_ROOT" -mindepth 1 -maxdepth 1 ! -name config -print -quit)" || -e "$MIGRATION_ROOT" ]]; then
        echo "Legacy workspace has unexpected sibling data; refusing automatic migration." >&2
        exit 1
    fi
    echo "Migrating the previous nested workspace to Debian's standard layout..."
    mv -- "$LEGACY_ROOT" "$MIGRATION_ROOT"
    rmdir -- "$BUILD_ROOT"
    mv -- "$MIGRATION_ROOT" "$BUILD_ROOT"
fi

CONFIG_DIR="$BUILD_ROOT/config"
SOURCE_DIR="$CONFIG_DIR/includes.chroot/usr/src/aureaos"
BOOT_TEST_DIR="$CONFIG_DIR/includes.chroot/usr/local/libexec/aureaos"

refresh_inputs() {
    mkdir -p "$CONFIG_DIR/package-lists" "$CONFIG_DIR/hooks/live" "$SOURCE_DIR" "$BOOT_TEST_DIR"

    install -m 0644 "$SCRIPT_DIR/package-lists/aureaos.list.chroot" \
        "$CONFIG_DIR/package-lists/aureaos.list.chroot"
    install -m 0755 "$SCRIPT_DIR/hooks/010-build-aureaos.hook.chroot" \
        "$CONFIG_DIR/hooks/live/010-build-aureaos.hook.chroot"

    install -m 0755 "$SCRIPT_DIR/tests/validate-chroot-boot.sh" \
        "$BOOT_TEST_DIR/validate-chroot-boot.sh"

    # Replace only the staged source snapshot, never the live-build chroot.
    rm -rf -- "$SOURCE_DIR"
    mkdir -p -- "$SOURCE_DIR"
    git -C "$REPO_ROOT" archive --format=tar HEAD \
        Cargo.toml Cargo.lock components tools/iso/pam tools/iso/systemd tools/iso/scripts |
        tar -xf - -C "$SOURCE_DIR"

    [[ -s "$SOURCE_DIR/Cargo.toml" && -s "$SOURCE_DIR/Cargo.lock" ]] || {
        echo "The staged AureaOS source snapshot is incomplete." >&2
        return 1
    }
}

if "$RESUME"; then
    [[ -d "$CONFIG_DIR" ]] || {
        echo "Cannot resume: Debian Live configuration not found at $CONFIG_DIR" >&2
        exit 1
    }
    echo "Resuming Debian Live workspace: $BUILD_ROOT"

    # The previous Podman container may have been interrupted while live-build
    # held its workspace lock. The wrapper serializes all managed builds with
    # flock before reaching this point, so a leftover lock is stale here.
    if [[ -e "$BUILD_ROOT/.lock" ]]; then
        if [[ ! -f "$BUILD_ROOT/.lock" || -L "$BUILD_ROOT/.lock" ]]; then
            echo "Unexpected live-build lock path; refusing unsafe recovery." >&2
            exit 1
        fi
        echo "Removing stale live-build workspace lock from the interrupted run."
        rm -f -- "$BUILD_ROOT/.lock"
    fi

    refresh_inputs

    # If the previous chroot hook failed, live-build may consider the include
    # stage complete and skip it. Re-run that exact Debian stage before hooks.
    # Do not do this after chroot_hooks completed: the hook removes build sources.
    if [[ -d "$BUILD_ROOT/chroot" && ! -f "$BUILD_ROOT/.build/chroot_hooks" ]]; then
        echo "Refreshing chroot includes before retrying incomplete hooks..."
        (cd "$BUILD_ROOT" && sudo lb chroot_includes_after_packages --force)
    fi
else
    if [[ -e "$BUILD_ROOT" || -L "$BUILD_ROOT" ]]; then
        echo "Build directory already exists: $BUILD_ROOT" >&2
        echo "Use --resume to continue it, or choose another path. Existing data is preserved." >&2
        exit 1
    fi

    mkdir -p -- "$BUILD_ROOT"
    cd "$BUILD_ROOT"

    # Canonical Debian Live Build layout: workspace/config/{package-lists,hooks,...}
    lb config \
        --mode debian \
        --distribution trixie \
        --architectures amd64 \
        --binary-image iso-hybrid \
        --bootloaders "grub-efi" \
        --archive-areas "main contrib non-free non-free-firmware" \
        --firmware-chroot true \
        --firmware-binary true \
        --linux-packages linux-image \
        --linux-flavours amd64 \
        --debian-installer none \
        --apt-recommends false \
        --bootappend-live "boot=live components quiet loglevel=3 systemd.show_status=false live-config.nocomponents=user-setup live-config.nottyautologin" \
        --bootappend-live-failsafe "none" \
        --iso-application "AureaOS Native Blue Test" \
        --iso-volume "AUREAOS_BLUE_TEST" \
        --image-name "$IMAGE_NAME"

    refresh_inputs
fi

cleanup() {
    local status=$?
    if ((status == 0)); then
        echo "Build workspace and caches preserved: $BUILD_ROOT"
    else
        echo "Build failed; preserving workspace and diagnostics: $BUILD_ROOT" >&2
    fi
}
trap cleanup EXIT

# Refuse to enter the expensive build if the standard Debian layout is broken.
[[ -s "$CONFIG_DIR/includes.chroot/usr/src/aureaos/Cargo.toml" ]] || {
    echo "Staged source missing from config/includes.chroot." >&2
    exit 1
}
[[ -x "$CONFIG_DIR/hooks/live/010-build-aureaos.hook.chroot" ]] || {
    echo "AureaOS chroot hook is missing or not executable." >&2
    exit 1
}

cd "$BUILD_ROOT"
echo "Building Debian Trixie amd64 Live ISO..."
sudo lb build

ISO_PATH="$(find "$BUILD_ROOT" -maxdepth 1 -type f -name '*.iso' -print -quit)"
if [[ -z "$ISO_PATH" || ! -s "$ISO_PATH" ]]; then
    echo "Live Build completed without producing a non-empty ISO." >&2
    find "$BUILD_ROOT" -maxdepth 3 -type f -name '*.iso' -print >&2
    exit 1
fi

DESTINATION="$OUTPUT_DIR/AureaOS.iso"
# Replace Debian Live's intermediate GRUB image with the approved systemd-boot + UKI path.
bash "$SCRIPT_DIR/scripts/build-systemd-boot-live-iso.sh" \
    --binary-dir "$BUILD_ROOT/binary" \
    --output "$ISO_PATH"

# Validate the final staged tree and the exact ISO before publication.
bash "$SCRIPT_DIR/tests/validate-live-image.sh" --binary-dir "$BUILD_ROOT/binary"
install -m 0644 "$ISO_PATH" "$DESTINATION"
# Re-open the actual ISO and validate its extracted boot payload, not only the workspace.
bash "$SCRIPT_DIR/tests/validate-live-image.sh" --iso "$DESTINATION"
(
    cd "$OUTPUT_DIR"
    sha256sum AureaOS.iso > AureaOS.iso.sha256
)

echo
echo "AureaOS Live ISO created:"
echo "$DESTINATION"
echo "SHA256:"
cat "$DESTINATION.sha256"
