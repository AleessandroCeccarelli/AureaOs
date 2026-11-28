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

for command_name in lb git tar sudo sha256sum realpath find install chmod date awk stat python3; do
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

# A build workspace is tied to one immutable source revision. Never combine
# staged files or a partially built chroot from another commit.
SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
if [[ -n "$(git -C "$REPO_ROOT" status --porcelain)" ]]; then
    echo "The AureaOS working tree is not clean." >&2
    echo "Commit the intended source state before building a reproducible Live ISO." >&2
    exit 1
fi
SOURCE_MARKER="$BUILD_ROOT/.aureaos-source-commit"

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

    # Keep the UKI's embedded initramfs small enough for the El Torito ESP.
    # xz is provided by the Debian base image and is supported by the Debian kernel.
    INITRAMFS_CONF="$CONFIG_DIR/includes.chroot/etc/initramfs-tools/conf.d"
    install -d -m 0755 "$INITRAMFS_CONF"
    printf '%s\\n' 'COMPRESS=xz' > "$INITRAMFS_CONF/aureaos.conf"
    chmod 0644 "$INITRAMFS_CONF/aureaos.conf"

    # Replace only the staged source snapshot, never the live-build chroot.
    rm -rf -- "$SOURCE_DIR"
    mkdir -p -- "$SOURCE_DIR"
    git -C "$REPO_ROOT" archive --format=tar HEAD \
        Cargo.toml Cargo.lock components tools/iso/pam tools/iso/systemd tools/iso/scripts \
        tools/assets/render-context-menu.sh tools/assets/png_to_rgba.py \
        tools/assets/build_cursor_theme.py tools/assets/cursor_manifest.py \
        tools/assets/cursor_rasterizer.py tools/assets/cursor_svg.py \
        tools/assets/xcursor_config.py aurea-human-interfaces/cursors \
        aurea-human-interfaces/resources/wallpapers \
        aurea-human-interfaces/fonts/source/google-fonts/noto-sans \
        aurea-human-interfaces/icons/source/system/context-menu \
        aurea-human-interfaces/icons/source/system/trash |
        tar -xf - -C "$SOURCE_DIR"

    # Generate only the artwork consumed by the current compositor. Rasterize
    # in the disposable Debian builder, then stage raw RGBA into includes.chroot
    # so librsvg and Python never enter the target system.
    bash "$SOURCE_DIR/tools/assets/render-context-menu.sh"

    # Build native Xcursor themes in the disposable Debian builder, never in
    # the target chroot. x11-apps supplies xcursorgen here only; no X11 tools
    # or libraries are installed in the AureaOS Live filesystem.
    CURSOR_SOURCE="$SOURCE_DIR/aurea-human-interfaces/cursors"
    python3 "$SOURCE_DIR/tools/assets/build_cursor_theme.py" \
        --cursor-dir "$CURSOR_SOURCE" \
        --output "$CURSOR_SOURCE/generated/AureaOS-Light"
    python3 "$SOURCE_DIR/tools/assets/build_cursor_theme.py" \
        --cursor-dir "$CURSOR_SOURCE" \
        --manifest "$CURSOR_SOURCE/manifest-dark.toml" \
        --output "$CURSOR_SOURCE/generated/AureaOS-Dark"
    CURSOR_INCLUDE="$CONFIG_DIR/includes.chroot/usr/share/icons"
    install -d -m 0755 "$CURSOR_INCLUDE"
    cp -a "$CURSOR_SOURCE/generated/AureaOS-Light" "$CURSOR_INCLUDE/"
    cp -a "$CURSOR_SOURCE/generated/AureaOS-Dark" "$CURSOR_INCLUDE/"

    ASSET_SOURCE="$SOURCE_DIR/aurea-human-interfaces/icons/generated/context-menu"
    ASSET_INCLUDE="$CONFIG_DIR/includes.chroot/usr/share/aureaos/human-interfaces/icons/generated/context-menu"
    rm -rf -- "$ASSET_INCLUDE"
    mkdir -p -- "$ASSET_INCLUDE"
    for theme in light dark; do
        asset="$ASSET_SOURCE/context-menu-$theme-256.rgba"
        [[ -s "$asset" ]] || {
            echo "Generated $theme context-menu artwork is missing." >&2
            return 1
        }
        [[ "$(stat -c '%s' "$asset")" == 262144 ]] || {
            echo "Generated $theme context-menu artwork has an invalid byte length." >&2
            return 1
        }
        install -m 0644 "$asset" "$ASSET_INCLUDE/"
    done

    [[ -s "$SOURCE_DIR/aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg" ]] || {
        echo "The official AureaOS default wallpaper is missing from the staged source snapshot." >&2
        return 1
    }

    python3 - "$SOURCE_DIR/aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg" <<'PY'
import pathlib
import sys

data = pathlib.Path(sys.argv[1]).read_bytes()
if not data.startswith(bytes((0xFF, 0xD8, 0xFF))):
    raise SystemExit("The selected AureaOS wallpaper is not a valid JPEG.")
offset = 2
dimensions = None
while offset + 4 <= len(data):
    if data[offset] != 0xFF:
        offset += 1
        continue
    marker = data[offset + 1]
    if marker in (0xD9, 0xDA):
        break
    length = int.from_bytes(data[offset + 2:offset + 4], "big")
    if length < 2 or offset + 2 + length > len(data):
        raise SystemExit("The selected AureaOS wallpaper has a malformed JPEG segment.")
    if marker in (0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF):
        height = int.from_bytes(data[offset + 5:offset + 7], "big")
        width = int.from_bytes(data[offset + 7:offset + 9], "big")
        dimensions = (width, height)
        break
    offset += 2 + length
if dimensions != (3840, 2160):
    raise SystemExit(f"Expected 3840x2160 default wallpaper, found {dimensions}.")
print("Validated AureaOS default wallpaper: 3840x2160 JPEG.")
PY

    [[ -s "$SOURCE_DIR/Cargo.toml" && -s "$SOURCE_DIR/Cargo.lock" ]] || {
        echo "The staged AureaOS source snapshot is incomplete." >&2
        return 1
    }

    # Aurea Shell embeds the six approved Trash SVGs with include_bytes!.
    # Keep their source directory in the archive and fail before live-build
    # starts if any compile-time asset is absent from the snapshot.
    for asset in light/trash-empty.svg light/trash-half.svg light/trash-full.svg dark/trash-empty.svg dark/trash-half.svg dark/trash-full.svg; do
        [[ -s "$SOURCE_DIR/aurea-human-interfaces/icons/source/system/trash/$asset" ]] || {
            echo "Required compile-time Trash artwork is missing from the staged source: $asset" >&2
            return 1
        }
    done
}

if "$RESUME"; then
    [[ -d "$CONFIG_DIR" ]] || {
        echo "Cannot resume: Debian Live configuration not found at $CONFIG_DIR" >&2
        exit 1
    }
    [[ -s "$SOURCE_MARKER" ]] || {
        echo "Cannot safely resume this legacy workspace: source commit marker is missing." >&2
        echo "Preserve it for diagnostics and start a fresh build directory." >&2
        exit 1
    }
    RECORDED_COMMIT="$(<"$SOURCE_MARKER")"
    [[ "$RECORDED_COMMIT" == "$SOURCE_COMMIT" ]] || {
        echo "Cannot resume: workspace belongs to commit $RECORDED_COMMIT, current source is $SOURCE_COMMIT." >&2
        echo "Use a fresh build directory for this source revision." >&2
        exit 1
    }
    echo "Resuming Debian Live workspace: $BUILD_ROOT (source $SOURCE_COMMIT)"

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
    printf '%s\n' "$SOURCE_COMMIT" > "$SOURCE_MARKER"
    chmod 0644 "$SOURCE_MARKER"
    cd "$BUILD_ROOT"

    # Canonical Debian Live Build layout: workspace/config/{package-lists,hooks,...}
    lb config \
        --mode debian \
        --distribution trixie \
        --architectures amd64 \
        --binary-image iso-hybrid \
        --bootloaders "syslinux" \
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

mapfile -t ISO_FILES < <(find "$BUILD_ROOT" -maxdepth 1 -type f -name '*.iso' -print | sort)
if (("${#ISO_FILES[@]}" != 1)); then
    echo "Expected exactly one live-build ISO artifact; found ${#ISO_FILES[@]}." >&2
    find "$BUILD_ROOT" -maxdepth 3 -type f -name '*.iso' -print >&2
    exit 1
fi
ISO_PATH="${ISO_FILES[0]}"
[[ -s "$ISO_PATH" ]] || {
    echo "Live Build produced an empty ISO: $ISO_PATH" >&2
    exit 1
}

# Record build provenance before the isolated UKI assembly snapshot is prepared.
BUILD_TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BUILD_ID="$BUILD_TIMESTAMP-$(git -C "$REPO_ROOT" rev-parse --short=12 HEAD)"
COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
BRANCH="$(git -C "$REPO_ROOT" branch --show-current)"
REMOTE="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || true)"
if [[ -z "$(git -C "$REPO_ROOT" status --porcelain)" ]]; then
    WORKTREE_CLEAN=true
else
    WORKTREE_CLEAN=false
fi
KERNEL_PATH="$(find "$BUILD_ROOT/binary/live" -maxdepth 1 -type f -name 'vmlinuz-*' -print | sort -V | tail -n 1)"
[[ -n "$KERNEL_PATH" ]] || { echo "No staged kernel found for build manifest." >&2; exit 1; }
KERNEL_VERSION="${KERNEL_PATH##*/vmlinuz-}"
export BUILD_ID BUILD_TIMESTAMP COMMIT BRANCH REMOTE WORKTREE_CLEAN KERNEL_VERSION

# Replace Debian Live's intermediate GRUB image with the approved systemd-boot + UKI path.
bash "$SCRIPT_DIR/scripts/build-systemd-boot-live-iso.sh" \
    --binary-dir "$BUILD_ROOT/binary" \
    --output "$ISO_PATH"

# The UKI assembler validates its isolated staging tree before mastering.
# Validate the exact ISO emitted by xorriso before preparing publication artifacts.
bash "$SCRIPT_DIR/tests/validate-live-image.sh" --iso "$ISO_PATH"

CANDIDATE_NAME="AureaOS-$BUILD_ID.iso"
CANDIDATE="$OUTPUT_DIR/$CANDIDATE_NAME"
CANDIDATE_TMP="$OUTPUT_DIR/.$CANDIDATE_NAME.tmp.$$"
CANDIDATE_SHA="$CANDIDATE.sha256"
CANDIDATE_MANIFEST="$OUTPUT_DIR/AureaOS-$BUILD_ID.manifest.json"
MANIFEST_TMP="$CANDIDATE_MANIFEST.tmp.$$"
[[ ! -e "$CANDIDATE" && ! -e "$CANDIDATE_MANIFEST" ]] || {
    echo "Build artifact already exists; refusing to overwrite historical evidence." >&2
    exit 1
}
finalize_publication() {
    local status=$?
    rm -f -- "$CANDIDATE_TMP" "$MANIFEST_TMP"
    if ((status == 0)); then
        echo "Build workspace and caches preserved: $BUILD_ROOT"
    else
        echo "Build failed; preserving workspace and diagnostics: $BUILD_ROOT" >&2
    fi
}
trap finalize_publication EXIT

install -m 0644 "$ISO_PATH" "$CANDIDATE_TMP"
# Re-open the exact temporary ISO before it receives a permanent name.
bash "$SCRIPT_DIR/tests/validate-live-image.sh" --iso "$CANDIDATE_TMP"
ISO_SHA256="$(sha256sum "$CANDIDATE_TMP" | awk '{print $1}')"
ISO_BYTES="$(stat -c '%s' "$CANDIDATE_TMP")"
export CANDIDATE_NAME ISO_SHA256 ISO_BYTES

python3 - "$MANIFEST_TMP" <<'PY'
import json
import os
import sys

manifest = {
    "project": "AureaOS",
    "artifact": os.environ["CANDIDATE_NAME"],
    "build_id": os.environ["BUILD_ID"],
    "build_timestamp_utc": os.environ["BUILD_TIMESTAMP"],
    "git": {
        "remote": os.environ.get("REMOTE", ""),
        "branch": os.environ.get("BRANCH", ""),
        "commit": os.environ["COMMIT"],
        "working_tree_clean": os.environ["WORKTREE_CLEAN"] == "true",
        "build_source": "git archive HEAD",
    },
    "platform": {
        "distribution": "Debian GNU/Linux 13 (Trixie)",
        "architecture": "amd64",
        "boot_manager": "systemd-boot",
        "kernel_artifact": "UKI",
        "secure_boot": "disabled",
        "kernel_version": os.environ["KERNEL_VERSION"],
    },
    "validation": {
        "chroot_live_boot_preflight": "passed",
        "debian_13_rootfs": "passed",
        "systemd_multi_user_target": "passed",
        "embedded_uki_scripts_live": "passed",
        "assembly_tree": "passed",
        "iso_payload": "passed",
        "sha256": os.environ["ISO_SHA256"],
        "hardware_boot": "pending",
    },
    "iso": {
        "size_bytes": int(os.environ["ISO_BYTES"]),
        "sha256": os.environ["ISO_SHA256"],
    },
}
with open(sys.argv[1], "w", encoding="utf-8") as stream:
    json.dump(manifest, stream, indent=2, sort_keys=True)
    stream.write("\n")
PY

# Publish uniquely named evidence first. The stable convenience alias is
# replaced only after both automated validations have passed.
mv -- "$CANDIDATE_TMP" "$CANDIDATE"
printf '%s  %s\n' "$ISO_SHA256" "$CANDIDATE_NAME" > "$CANDIDATE_SHA"
mv -- "$MANIFEST_TMP" "$CANDIDATE_MANIFEST"

STABLE_TMP="$OUTPUT_DIR/.AureaOS.iso.tmp.$$"
STABLE_SHA_TMP="$OUTPUT_DIR/.AureaOS.iso.sha256.tmp.$$"
install -m 0644 "$CANDIDATE" "$STABLE_TMP"
printf '%s  AureaOS.iso\n' "$ISO_SHA256" > "$STABLE_SHA_TMP"
mv -f -- "$STABLE_TMP" "$OUTPUT_DIR/AureaOS.iso"
mv -f -- "$STABLE_SHA_TMP" "$OUTPUT_DIR/AureaOS.iso.sha256"

echo
echo "AureaOS Live ISO validated and published:"
echo "  Candidate: $CANDIDATE"
echo "  Stable:    $OUTPUT_DIR/AureaOS.iso"
echo "  Manifest:  $CANDIDATE_MANIFEST"
echo "  Kernel:    $KERNEL_VERSION"
echo "  SHA256:    $ISO_SHA256"
