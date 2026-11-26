#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly ISO_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
readonly OS_RELEASE="$ISO_DIR/os-release"
readonly ESP_SIZE_MIB="${AUREAOS_ESP_SIZE_MIB:-256}"

usage() {
    echo "Usage: $0 --binary-dir PATH --output PATH" >&2
    exit 2
}

fail() {
    echo "AureaOS systemd-boot ISO assembly: $*" >&2
    exit 1
}

BINARY_DIR=""
OUTPUT_PATH=""
while (($#)); do
    case "$1" in
        --binary-dir|--output)
            [[ $# -ge 2 && -n "$2" ]] || usage
            if [[ "$1" == "--binary-dir" ]]; then
                [[ -z "$BINARY_DIR" ]] || usage
                BINARY_DIR="$2"
            else
                [[ -z "$OUTPUT_PATH" ]] || usage
                OUTPUT_PATH="$2"
            fi
            shift 2
            ;;
        *) usage ;;
    esac
done
[[ -n "$BINARY_DIR" && -n "$OUTPUT_PATH" ]] || usage
[[ "$BINARY_DIR" = /* && "$OUTPUT_PATH" = /* ]] || fail "paths must be absolute"
[[ -d "$BINARY_DIR/live" ]] || fail "Debian Live payload is missing"
[[ -s "$OS_RELEASE" ]] || fail "AureaOS UKI os-release metadata is missing"
[[ "$ESP_SIZE_MIB" =~ ^[0-9]+$ ]] || fail "AUREAOS_ESP_SIZE_MIB must be an integer"
(( ESP_SIZE_MIB >= 64 )) || fail "EFI image must be at least 64 MiB"

for tool in ukify mcopy mmd mkfs.vfat truncate xorriso sort; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
done

readonly BOOTLOADER="/usr/lib/systemd/boot/efi/systemd-bootx64.efi"
readonly UKI_STUB="/usr/lib/systemd/boot/efi/linuxx64.efi.stub"
[[ -s "$BOOTLOADER" ]] || fail "Debian systemd-boot EFI binary is missing: $BOOTLOADER"
[[ -s "$UKI_STUB" ]] || fail "Debian systemd UKI stub is missing: $UKI_STUB"

LIVE_DIR="$BINARY_DIR/live"
shopt -s nullglob
KERNELS=("$LIVE_DIR"/vmlinuz-*)
(("${#KERNELS[@]}" > 0)) || fail "no Debian kernel found in /live"

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/aureaos-uki.XXXXXX")"
TEMP_ISO="${OUTPUT_PATH}.tmp.$$"
cleanup() {
    rm -rf -- "$WORK_DIR"
    rm -f -- "$TEMP_ISO"
}
trap cleanup EXIT

readonly ESP_IMAGE="$BINARY_DIR/boot/efi.img"
mkdir -p "$BINARY_DIR/boot"

CMDLINE='boot=live components quiet loglevel=3 systemd.show_status=false live-config.nocomponents=user-setup live-config.nottyautologin'
DEFAULT_ENTRY=""
LATEST_KERNEL="$(printf '%s\n' "${KERNELS[@]}" | sort -V | tail -n 1)"
LATEST_VERSION="${LATEST_KERNEL##*/vmlinuz-}"
DEFAULT_ENTRY="aureaos-live-$LATEST_VERSION.conf"

# A UKI is generated from the exact kernel/initramfs pair live-build staged.
# Every installed kernel receives a matching entry; the newest version is default.
for KERNEL in "${KERNELS[@]}"; do
    VERSION="${KERNEL##*/vmlinuz-}"
    INITRD="$LIVE_DIR/initrd.img-$VERSION"
    [[ -s "$KERNEL" ]] || fail "empty kernel: $KERNEL"
    [[ -s "$INITRD" ]] || fail "matching initramfs is missing: $INITRD"

    UKI_NAME="aureaos-live-$VERSION.efi"
    ENTRY_NAME="aureaos-live-$VERSION.conf"
    UKI_PATH="$WORK_DIR/$UKI_NAME"

    ukify build \
        --linux="$KERNEL" \
        --initrd="$INITRD" \
        --cmdline="$CMDLINE" \
        --os-release="@$OS_RELEASE" \
        --output="$UKI_PATH"

    [[ -s "$UKI_PATH" ]] || fail "ukify did not create $UKI_PATH"
    ukify inspect --all "$UKI_PATH" > "$WORK_DIR/$UKI_NAME.inspect"
    grep -Fq 'boot=live' "$WORK_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME does not contain the required boot=live command line"

    printf 'title AureaOS Live (Linux %s)\nefi /EFI/Linux/%s\n' \
        "$VERSION" "$UKI_NAME" > "$WORK_DIR/$ENTRY_NAME"

done

# systemd-boot scans this FAT image as its ESP. No firmware NVRAM entries are
# created and no host disk is touched by this build-time operation.
rm -f -- "$ESP_IMAGE"
truncate -s "${ESP_SIZE_MIB}M" "$ESP_IMAGE"
mkfs.vfat -F 32 -n AUREAOS "$ESP_IMAGE" >/dev/null

for directory in EFI EFI/BOOT EFI/Linux EFI/systemd loader loader/entries; do
    mmd -i "$ESP_IMAGE" "::/$directory"
done

mcopy -i "$ESP_IMAGE" "$BOOTLOADER" ::/EFI/BOOT/BOOTX64.EFI
mcopy -i "$ESP_IMAGE" "$BOOTLOADER" ::/EFI/systemd/systemd-bootx64.efi

for KERNEL in "${KERNELS[@]}"; do
    VERSION="${KERNEL##*/vmlinuz-}"
    UKI_NAME="aureaos-live-$VERSION.efi"
    ENTRY_NAME="aureaos-live-$VERSION.conf"
    mcopy -i "$ESP_IMAGE" "$WORK_DIR/$UKI_NAME" "::/EFI/Linux/$UKI_NAME"
    mcopy -i "$ESP_IMAGE" "$WORK_DIR/$ENTRY_NAME" "::/loader/entries/$ENTRY_NAME"
done

cat > "$WORK_DIR/loader.conf" <<EOF
default $DEFAULT_ENTRY
timeout 5
console-mode keep
editor no
auto-entries no
auto-firmware no
EOF
mcopy -i "$ESP_IMAGE" "$WORK_DIR/loader.conf" ::/loader/loader.conf

# The kernel and initramfs now live inside the UKI. Keep only the Live
# filesystem payload in /live; this avoids shipping duplicate boot artifacts.
for file in "$LIVE_DIR"/vmlinuz-* "$LIVE_DIR"/initrd.img-*; do
    [[ -f "$file" ]] && rm -f -- "$file"
done

# Remove all intermediate GRUB/legacy boot payloads before mastering the
# final UEFI-only ISO. GRUB may be used by live-build internally to stage the
# binary tree, but it is never included in the published AureaOS image.
rm -rf -- "$BINARY_DIR/boot/grub" "$BINARY_DIR/boot/isolinux" \
    "$BINARY_DIR/isolinux" "$BINARY_DIR/syslinux"

mkdir -p -- "$(dirname -- "$OUTPUT_PATH")"
rm -f -- "$TEMP_ISO"
xorriso -as mkisofs \
    -r -J -joliet-long -iso-level 3 \
    -V AUREAOS_LIVE \
    -o "$TEMP_ISO" \
    -eltorito-alt-boot \
    -e boot/efi.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    "$BINARY_DIR"

[[ -s "$TEMP_ISO" ]] || fail "xorriso did not create the final ISO"
mv -f -- "$TEMP_ISO" "$OUTPUT_PATH"

echo "Created UEFI-only AureaOS Live ISO with systemd-boot and UKI:"
echo "  $OUTPUT_PATH"
echo "  ESP: $ESP_IMAGE (${ESP_SIZE_MIB} MiB)"
echo "  Default entry: $DEFAULT_ENTRY"
