#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Validate the UEFI systemd-boot + UKI payload and Debian Live filesystem.
set -Eeuo pipefail

usage() {
    echo "Usage: $0 --binary-dir PATH | --iso PATH" >&2
    exit 2
}

fail() {
    echo "AureaOS image validation: $*" >&2
    exit 1
}

MODE=""
TARGET=""
while (($#)); do
    case "$1" in
        --binary-dir|--iso)
            [[ $# -ge 2 && -n "$2" ]] || usage
            [[ -z "$MODE" ]] || usage
            MODE="$1"
            TARGET="$2"
            shift 2
            ;;
        *) usage ;;
    esac
done
[[ -n "$MODE" && -n "$TARGET" ]] || usage

for tool in mdir mcopy mtype ukify grep unsquashfs readlink stat find lsinitramfs dpkg-query; do
    command -v "$tool" >/dev/null 2>&1 || fail "required validation tool not found: $tool"
done

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/aureaos-image-check.XXXXXX")"
cleanup() {
    rm -rf -- "$TEMP_DIR"
}
trap cleanup EXIT

if [[ "$MODE" == "--iso" ]]; then
    [[ -s "$TARGET" ]] || fail "ISO is missing or empty: $TARGET"
    command -v xorriso >/dev/null 2>&1 || fail "xorriso is required to inspect the ISO"
    mkdir -p "$TEMP_DIR/live" "$TEMP_DIR/boot"
    xorriso -osirrox on -indev "$TARGET" \
        -extract /live "$TEMP_DIR/live" \
        -extract /boot "$TEMP_DIR/boot" >/dev/null 2>&1 || \
        fail "cannot extract the Live payload or complete boot directory"
    ROOT="$TEMP_DIR"
    REPORT="$(xorriso -indev "$TARGET" -report_el_torito as_mkisofs 2>&1)" ||
        fail "cannot inspect the ISO El Torito boot catalog"
    grep -Fq 'boot/efi.img' <<<"$REPORT" ||
        fail "ISO El Torito catalog does not reference the systemd-boot EFI image"
    grep -Eq -- '(-e|--efi-boot).*boot/efi\.img' <<<"$REPORT" ||
        fail "ISO El Torito catalog does not declare boot/efi.img as an EFI boot image"
    grep -Fq -- '-no-emul-boot' <<<"$REPORT" ||
        fail "ISO EFI El Torito entry is not configured for no-emulation boot"
    SYSTEM_AREA_REPORT="$(xorriso -indev "$TARGET" -report_system_area plain 2>&1)" ||
        fail "cannot inspect the ISO system area and GPT partition map"
    grep -Eiq 'GPT|GUID Partition Table' <<<"$SYSTEM_AREA_REPORT" ||
        fail "ISO does not contain a detectable GPT partition table"
    grep -Eiq 'EFI System|EFI.*partition|partition.*EFI' <<<"$SYSTEM_AREA_REPORT" ||
        fail "ISO GPT does not expose an EFI System Partition"
else
    [[ -d "$TARGET" ]] || fail "binary tree does not exist: $TARGET"
    ROOT="$TARGET"
fi

LIVE_DIR="$ROOT/live"
ESP_IMAGE="$ROOT/boot/efi.img"
[[ -d "$LIVE_DIR" ]] || fail "missing /live directory in Live payload"
[[ -s "$LIVE_DIR/filesystem.squashfs" ]] ||
    fail "Debian Live filesystem.squashfs is missing or empty"
[[ -s "$ESP_IMAGE" ]] || fail "systemd-boot EFI image is missing or empty"

# Inspect the actual compressed root filesystem, not just the outer ISO
# directory. Extract only small, explicitly required files. Kernel modules and
# firmware are checked against the SquashFS metadata listing below: extracting
# those entire trees needlessly expands thousands of files and can stall builds.
ROOTFS="$TEMP_DIR/rootfs"
mkdir -p "$ROOTFS"

# Never pass whole icon-theme directories to unsquashfs: that recursively
# expands every inherited icon and can turn a small validation into a huge,
# opaque extraction. Extract only the exact files that the checks below use.
SQUASHFS_EXTRACT_PATHS=(
    usr/local/bin/aurea-desktop
    usr/local/bin/aurea-live-session
    etc/systemd/system/aurea-desktop.service
    etc/pam.d/aurea-desktop
    etc/systemd/system/multi-user.target.wants/aurea-desktop.service
    etc/os-release usr/lib/os-release etc/debian_version usr/lib/systemd/systemd sbin/init
    usr/lib/systemd/system/multi-user.target etc/systemd/system/default.target
    var/lib/dpkg/status
    usr/share/vulkan/icd.d
    usr/lib/x86_64-linux-gnu/libvulkan_radeon.so
    usr/lib/x86_64-linux-gnu/libvulkan_lvp.so
    usr/share/aureaos/human-interfaces/icons/generated/context-menu/context-menu-light-256.rgba
    usr/share/aureaos/human-interfaces/icons/generated/context-menu/context-menu-dark-256.rgba
)
CURSOR_NAMES=(default pointer text vertical-text crosshair help busy left_ptr_watch not-allowed copy alias grab grabbing move all-scroll ew-resize ns-resize resize-nesw resize-nwse pencil up-arrow pin person zoom-in zoom-out context-menu)
for theme in AureaOS-Light AureaOS-Dark; do
    SQUASHFS_EXTRACT_PATHS+=("usr/share/icons/$theme/index.theme")
    for cursor in "${CURSOR_NAMES[@]}"; do
        SQUASHFS_EXTRACT_PATHS+=("usr/share/icons/$theme/cursors/$cursor")
    done
done

unsquashfs -f -d "$ROOTFS" "$LIVE_DIR/filesystem.squashfs" "${SQUASHFS_EXTRACT_PATHS[@]}" \
    >/dev/null 2>&1 || fail "cannot inspect the AureaOS desktop session and artwork inside filesystem.squashfs"

[[ -x "$ROOTFS/usr/local/bin/aurea-desktop" ]] ||
    fail "AureaOS desktop executable is missing or not executable in the Live rootfs"
[[ -x "$ROOTFS/usr/local/bin/aurea-live-session" ]] ||
    fail "AureaOS Live session launcher is missing or not executable"
grep -aFq "aureaos-blue-flow" "$ROOTFS/usr/local/bin/aurea-desktop" ||
    fail "AureaOS compositor binary does not contain the selected blue-flow default wallpaper"
[[ -s "$ROOTFS/usr/lib/os-release" && -s "$ROOTFS/etc/os-release" ]] ||
    fail "Debian os-release metadata or its /etc/os-release link target is missing from the Live rootfs"
grep -Fxq 'ID=debian' "$ROOTFS/etc/os-release" ||
    fail "Live rootfs is not identified as Debian"
grep -Fxq 'VERSION_CODENAME=trixie' "$ROOTFS/etc/os-release" ||
    fail "Live rootfs is not Debian Trixie"
DEBIAN_VERSION="$(head -n 1 "$ROOTFS/etc/debian_version")"
[[ "$DEBIAN_VERSION" == 13 || "$DEBIAN_VERSION" == 13.* ]] ||
    fail "Live rootfs is not Debian 13"
[[ -x "$ROOTFS/usr/lib/systemd/systemd" && -L "$ROOTFS/sbin/init" ]] ||
    fail "Debian systemd PID 1 integration is missing from the Live rootfs"
[[ -s "$ROOTFS/usr/lib/systemd/system/multi-user.target" ]] ||
    fail "Debian systemd multi-user target is missing"
[[ -L "$ROOTFS/etc/systemd/system/default.target" ]] ||
    fail "Debian systemd default target is not configured"
DEFAULT_TARGET="$(readlink "$ROOTFS/etc/systemd/system/default.target")"
[[ "${DEFAULT_TARGET##*/}" == "multi-user.target" ]] ||
    fail "Debian Live default target is not multi-user.target"
for package in live-boot live-boot-initramfs-tools live-config live-config-systemd systemd-sysv; do
    status="$(dpkg-query --admindir="$ROOTFS/var/lib/dpkg" -W -f='${db:Status-Status}' "$package" 2>/dev/null)" ||
        fail "Debian Live boot package is missing: $package"
    [[ "$status" == installed ]] ||
        fail "Debian Live boot package is not installed: $package"
done

# Verify the native graphics/input stack in the actual compressed rootfs.
# Package-list declarations alone are insufficient: these checks inspect the
# installed package database and the driver/runtime payload shipped in SquashFS.
for package in linux-image-amd64 firmware-amd-graphics libvulkan1 mesa-vulkan-drivers \
    libdrm2 libgbm1 libinput10 libseat1 libudev1 libxkbcommon0 xkb-data libpam-systemd \
    libturbojpeg0 libheif1 libheif-plugin-libde265 libheif-plugin-dav1d; do
    status="$(dpkg-query --admindir="$ROOTFS/var/lib/dpkg" -W -f='${db:Status-Status}' "$package" 2>/dev/null)" ||
        fail "AureaOS runtime dependency is missing from the Live rootfs: $package"
    [[ "$status" == installed ]] ||
        fail "AureaOS runtime dependency is not installed in the Live rootfs: $package"
done

# Query only the two hardware payload paths needed by this check. A full
# -ll traversal walks every inode in the Debian rootfs (often hundreds of
# thousands of entries) and makes post-mastering validation needlessly slow.
# Quoted patterns are passed to unsquashfs, which applies its own globbing.
SQUASHFS_GPU_LIST="$TEMP_DIR/squashfs-gpu.list"
unsquashfs -ll "$LIVE_DIR/filesystem.squashfs" \
    'usr/lib/modules/*/kernel/drivers/gpu/drm/amd/amdgpu/amdgpu.ko*' \
    'usr/lib/firmware/amdgpu/*.bin*' > "$SQUASHFS_GPU_LIST" || \
    fail "cannot inspect AMD GPU module and firmware metadata"
grep -Eq 'usr/lib/modules/[^[:space:]]+/kernel/drivers/gpu/drm/amd/amdgpu/amdgpu\\.ko(\\.(xz|zst))?([[:space:]]|$)' "$SQUASHFS_GPU_LIST" || \
    fail "AMD amdgpu kernel module is missing from the Live rootfs"
grep -Eq 'usr/lib/firmware/amdgpu/[^[:space:]]+\\.bin(\\.(xz|zst))?([[:space:]]|$)' "$SQUASHFS_GPU_LIST" || \
    fail "AMD GPU firmware payload is missing from the Live rootfs"
[[ -s "$ROOTFS/usr/share/vulkan/icd.d/radeon_icd.json" ]] ||
    fail "Mesa RADV Vulkan ICD manifest is missing from the Live rootfs"
grep -Fq 'libvulkan_radeon.so' "$ROOTFS/usr/share/vulkan/icd.d/radeon_icd.json" ||
    fail "Mesa RADV ICD manifest does not reference the expected Radeon Vulkan driver"
[[ -s "$ROOTFS/usr/lib/x86_64-linux-gnu/libvulkan_radeon.so" ]] ||
    fail "Mesa RADV Vulkan driver library is missing from the Live rootfs"
[[ -s "$ROOTFS/usr/share/vulkan/icd.d/lvp_icd.json" ]] ||
    fail "Mesa Lavapipe Vulkan ICD manifest is missing from the Live rootfs"
grep -Fq 'libvulkan_lvp.so' "$ROOTFS/usr/share/vulkan/icd.d/lvp_icd.json" ||
    fail "Mesa Lavapipe ICD manifest does not reference the expected software Vulkan driver"
[[ -s "$ROOTFS/usr/lib/x86_64-linux-gnu/libvulkan_lvp.so" ]] ||
    fail "Mesa Lavapipe software Vulkan driver library is missing from the Live rootfs"

[[ -s "$ROOTFS/etc/systemd/system/aurea-desktop.service" ]] ||
    fail "AureaOS systemd service is missing from the Live rootfs"
[[ -s "$ROOTFS/etc/pam.d/aurea-desktop" ]] ||
    fail "AureaOS PAM session definition is missing from the Live rootfs"
grep -Fq 'PAMName=aurea-desktop' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not register the dedicated PAM session"
grep -Fq 'User=aurea' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not run as the unprivileged aurea user"
grep -Fxq 'Environment=AUREAOS_LIVE_MINIMAL=1' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS Live service does not disable the Welcome window for the wallpaper-only first boot"
grep -Fq 'ExecStart=/usr/local/bin/aurea-live-session' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not start the Live session launcher"
grep -Fq 'After=systemd-logind.service systemd-user-sessions.service' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service is not ordered after logind and user-session readiness"
grep -Fq 'Wants=systemd-logind.service' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not pull in systemd-logind"
grep -Fxq 'Environment=LIBSEAT_BACKEND=logind' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS service does not select the logind libseat backend"
grep -Fxq 'Environment=XDG_SEAT=seat0' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS service does not identify the expected logind seat"
grep -Fxq 'TTYPath=/dev/tty1' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS service is not attached to the configured first virtual terminal"
grep -Fq 'pam_systemd.so' "$ROOTFS/etc/pam.d/aurea-desktop" ||
    fail "AureaOS PAM service does not invoke pam_systemd"
ENABLED_SERVICE="$ROOTFS/etc/systemd/system/multi-user.target.wants/aurea-desktop.service"
[[ -L "$ENABLED_SERVICE" ]] ||
    fail "AureaOS desktop service is not enabled in multi-user.target"
ENABLED_TARGET="$(readlink "$ENABLED_SERVICE")"
[[ "${ENABLED_TARGET##*/}" == "aurea-desktop.service" ]] ||
    fail "AureaOS desktop service enablement link has an unexpected target"

ASSET_ROOT="$ROOTFS/usr/share/aureaos/human-interfaces/icons/generated/context-menu"
asset_count=0
for theme in light dark; do
    asset="$ASSET_ROOT/context-menu-$theme-256.rgba"
    [[ -s "$asset" ]] || fail "missing $theme context-menu artwork in the Live rootfs"
    [[ "$(stat -c '%s' "$asset")" == 262144 ]] ||
        fail "invalid byte length for $theme context-menu artwork"
    asset_count=$((asset_count + 1))
done
[[ "$asset_count" == 2 ]] || fail "expected 2 context-menu RGBA8 assets, found $asset_count"

# Validate both installed Xcursor themes in the extracted final SquashFS.
cursor_count=0
for theme in AureaOS-Light AureaOS-Dark; do
    theme_root="$ROOTFS/usr/share/icons/$theme"
    [[ -s "$theme_root/index.theme" ]] || fail "missing Xcursor theme index: $theme"
    for cursor in default pointer text vertical-text crosshair help busy left_ptr_watch not-allowed copy alias grab grabbing move all-scroll ew-resize ns-resize resize-nesw resize-nwse pencil up-arrow pin person zoom-in zoom-out context-menu; do
        [[ -s "$theme_root/cursors/$cursor" ]] || fail "missing Xcursor asset: $theme/$cursor"
        cursor_count=$((cursor_count + 1))
    done
done
[[ "$cursor_count" == 52 ]] || fail "expected 52 canonical Xcursor files across both themes, found $cursor_count"

# The final image is UEFI-only. No GRUB configuration or split kernel/initrd
# files should remain in the published Live payload.
[[ ! -e "$ROOT/boot/grub" ]] || fail "GRUB boot payload remains in the final image"
shopt -s nullglob
split_boot_files=("$LIVE_DIR"/vmlinuz-* "$LIVE_DIR"/initrd.img-*)
(("${#split_boot_files[@]}" == 0)) ||
    fail "split kernel/initramfs files remain outside the UKI"

mdir -i "$ESP_IMAGE" ::/EFI/BOOT/BOOTX64.EFI >/dev/null ||
    fail "UEFI removable-media fallback loader BOOTX64.EFI is missing"
mdir -i "$ESP_IMAGE" ::/EFI/systemd/systemd-bootx64.efi >/dev/null ||
    fail "systemd-boot EFI binary is missing"
mdir -i "$ESP_IMAGE" ::/loader/loader.conf >/dev/null ||
    fail "systemd-boot loader.conf is missing"
mdir -i "$ESP_IMAGE" ::/loader/entries >/dev/null ||
    fail "systemd-boot entry directory is missing"
mdir -i "$ESP_IMAGE" ::/EFI/Linux >/dev/null ||
    fail "UKI directory is missing"

mtype -i "$ESP_IMAGE" ::/loader/loader.conf > "$TEMP_DIR/loader.conf" ||
    fail "cannot read systemd-boot loader.conf"
grep -Eq '^default aureaos-live-.+\.conf$' "$TEMP_DIR/loader.conf" ||
    fail "systemd-boot default entry is not an AureaOS Live entry"
grep -Eq '^timeout 5$' "$TEMP_DIR/loader.conf" ||
    fail "systemd-boot timeout is not five seconds"

mcopy -i "$ESP_IMAGE" '::/EFI/Linux/*.efi' "$TEMP_DIR/" >/dev/null ||
    fail "cannot extract UKI files from the EFI image"
UKIS=("$TEMP_DIR"/aureaos-live-*.efi)
(("${#UKIS[@]}" > 0)) || fail "no AureaOS Live UKI found"

ENTRY_COUNT=0
for UKI in "${UKIS[@]}"; do
    UKI_NAME="${UKI##*/}"
    VERSION="${UKI_NAME#aureaos-live-}"
    VERSION="${VERSION%.efi}"
    ENTRY="$TEMP_DIR/aureaos-live-$VERSION.conf"

    ukify --all inspect "$UKI" > "$TEMP_DIR/$UKI_NAME.inspect" ||
        fail "ukify cannot inspect $UKI_NAME"
    for section in .linux .initrd .osrel .cmdline; do
        grep -Fq "$section" "$TEMP_DIR/$UKI_NAME.inspect" ||
            fail "$UKI_NAME is missing UKI section $section"
    done
    grep -Fq 'boot=live' "$TEMP_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME does not embed boot=live"
    grep -Fq 'ID=aureaos' "$TEMP_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME does not identify AureaOS in .osrel"

    # Inspect the initramfs embedded in this exact UKI, not merely the source
    # initrd that existed earlier in the live-build workspace.
    UKI_INITRD="$TEMP_DIR/$UKI_NAME.initrd"
    ukify inspect --section=".initrd:binary@$UKI_INITRD" "$UKI" >/dev/null ||
        fail "cannot extract the initramfs embedded in $UKI_NAME"
    [[ -s "$UKI_INITRD" ]] || fail "$UKI_NAME contains an empty initramfs"
    UKI_INITRD_LIST="$TEMP_DIR/$UKI_NAME.initrd.list"
    lsinitramfs "$UKI_INITRD" > "$UKI_INITRD_LIST" ||
        fail "cannot list the initramfs embedded in $UKI_NAME"
    grep -Fxq 'scripts/live' "$UKI_INITRD_LIST" ||
        fail "$UKI_NAME initramfs does not contain scripts/live"
    mcopy -i "$ESP_IMAGE" "::/loader/entries/aureaos-live-$VERSION.conf" "$ENTRY" >/dev/null ||
        fail "boot entry for $UKI_NAME is missing"
    grep -Fq "efi /EFI/Linux/$UKI_NAME" "$ENTRY" ||
        fail "boot entry does not point to $UKI_NAME"

    echo "Validated UKI: $UKI_NAME"
    ENTRY_COUNT=$((ENTRY_COUNT + 1))
done

echo "AureaOS Live image validation passed ($MODE):"
echo "  systemd-boot EFI loader"
echo "  $ENTRY_COUNT UKI boot entr(y/ies)"
echo "  Debian Live filesystem"
