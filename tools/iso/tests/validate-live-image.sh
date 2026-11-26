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

for tool in mdir mcopy mtype ukify grep unsquashfs readlink stat lsinitramfs dpkg-query; do
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
        -extract /boot/efi.img "$TEMP_DIR/boot/efi.img" >/dev/null 2>&1 ||
        fail "cannot extract the Live payload or EFI boot image"
    ROOT="$TEMP_DIR"
    REPORT="$(xorriso -indev "$TARGET" -report_el_torito as_mkisofs 2>&1)" ||
        fail "cannot inspect the ISO El Torito boot catalog"
    grep -Fq 'boot/efi.img' <<<"$REPORT" ||
        fail "ISO El Torito catalog does not reference the systemd-boot EFI image"
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
# directory. This catches a missing desktop binary, service, PAM hook or
# systemd enablement link before an image is published.
ROOTFS="$TEMP_DIR/rootfs"
mkdir -p "$ROOTFS"
unsquashfs -f -d "$ROOTFS" "$LIVE_DIR/filesystem.squashfs" \
    usr/local/bin/aurea-desktop \
    usr/local/bin/aurea-live-session \
    etc/systemd/system/aurea-desktop.service \
    etc/pam.d/aurea-desktop \
    etc/systemd/system/multi-user.target.wants/aurea-desktop.service \
    etc/os-release etc/debian_version usr/lib/systemd/systemd sbin/init \
    usr/lib/systemd/system/multi-user.target etc/systemd/system/default.target \
    var/lib/dpkg/status \
    usr/share/aureaos/human-interfaces/icons/generated/context-menu \
    >/dev/null 2>&1 || fail "cannot inspect the AureaOS desktop session and artwork inside filesystem.squashfs"

[[ -x "$ROOTFS/usr/local/bin/aurea-desktop" ]] ||
    fail "AureaOS desktop executable is missing or not executable in the Live rootfs"
[[ -x "$ROOTFS/usr/local/bin/aurea-live-session" ]] ||
    fail "AureaOS Live session launcher is missing or not executable"
[[ -s "$ROOTFS/etc/os-release" ]] ||
    fail "Debian os-release metadata is missing from the Live rootfs"
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

[[ -s "$ROOTFS/etc/systemd/system/aurea-desktop.service" ]] ||
    fail "AureaOS systemd service is missing from the Live rootfs"
[[ -s "$ROOTFS/etc/pam.d/aurea-desktop" ]] ||
    fail "AureaOS PAM session definition is missing from the Live rootfs"
grep -Fq 'PAMName=aurea-desktop' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not register the dedicated PAM session"
grep -Fq 'User=aurea' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not run as the unprivileged aurea user"
grep -Fq 'ExecStart=/usr/local/bin/aurea-live-session' "$ROOTFS/etc/systemd/system/aurea-desktop.service" ||
    fail "AureaOS systemd service does not start the Live session launcher"
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

    ukify inspect --all "$UKI" > "$TEMP_DIR/$UKI_NAME.inspect" ||
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
