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

for tool in mdir mcopy mtype ukify grep; do
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
    mkdir -p "$TEMP_DIR/live"
    xorriso -osirrox on -indev "$TARGET" \
        -extract /live "$TEMP_DIR/live" \
        -extract /boot/efi.img "$TEMP_DIR/efi.img" >/dev/null 2>&1 ||
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
    grep -Fq 'boot=live' "$TEMP_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME does not embed boot=live"
    grep -Eiq 'initrd' "$TEMP_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME has no inspectable initramfs section"
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
