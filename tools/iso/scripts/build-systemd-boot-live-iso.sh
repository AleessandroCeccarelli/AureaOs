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
(( ESP_SIZE_MIB >= 16 && ESP_SIZE_MIB <= 2047 )) || fail "EFI system partition must be between 16 and 2047 MiB"

for tool in ukify mcopy mmd mkfs.vfat truncate xorriso sort cp; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
done

readonly BOOTLOADER="/usr/lib/systemd/boot/efi/systemd-bootx64.efi"
readonly UKI_STUB="/usr/lib/systemd/boot/efi/linuxx64.efi.stub"
[[ -s "$BOOTLOADER" ]] || fail "Debian systemd-boot EFI binary is missing: $BOOTLOADER"
[[ -s "$UKI_STUB" ]] || fail "Debian systemd UKI stub is missing: $UKI_STUB"

# Work on a hard-link clone on the same filesystem. The assembly removes
# split kernel/initramfs and GRUB staging files; never mutate live-build's
# resumable binary tree until a final ISO has been validated.
WORK_DIR="$(mktemp -d "$(dirname -- "$BINARY_DIR")/.aureaos-uki.XXXXXX")"
ASSEMBLY_DIR="$WORK_DIR/binary"
TEMP_ISO="${OUTPUT_PATH}.tmp.$$"
cleanup() {
    rm -rf -- "$WORK_DIR"
    rm -f -- "$TEMP_ISO"
}
trap cleanup EXIT

mkdir -p "$ASSEMBLY_DIR"
cp -al -- "$BINARY_DIR/." "$ASSEMBLY_DIR/" ||
    fail "cannot create a same-filesystem assembly snapshot"

LIVE_DIR="$ASSEMBLY_DIR/live"
shopt -s nullglob
KERNELS=("$LIVE_DIR"/vmlinuz-*)
(("${#KERNELS[@]}" > 0)) || fail "no Debian kernel found in /live"


readonly ESP_IMAGE="$ASSEMBLY_DIR/boot/efi.img"
mkdir -p "$ASSEMBLY_DIR/boot"

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
        --stub="$UKI_STUB" \
        --linux="$KERNEL" \
        --initrd="$INITRD" \
        --uname="$VERSION" \
        --cmdline="$CMDLINE" \
        --os-release="@$OS_RELEASE" \
        --output="$UKI_PATH"

    [[ -s "$UKI_PATH" ]] || fail "ukify did not create $UKI_PATH"
    ukify --all inspect "$UKI_PATH" > "$WORK_DIR/$UKI_NAME.inspect"
    grep -Fq 'boot=live' "$WORK_DIR/$UKI_NAME.inspect" ||
        fail "$UKI_NAME does not contain the required boot=live command line"

    printf 'title AureaOS Live (Linux %s)\nefi /EFI/Linux/%s\n' \
        "$VERSION" "$UKI_NAME" > "$WORK_DIR/$ENTRY_NAME"

done

# El Torito stores the EFI image load count in a 16-bit sector field. For an
# EFI image larger than 65535 sectors, xorriso writes the UEFI full-medium
# convention (load count 0/1). -efi-boot-part --efi-boot-image also exposes
# this same FAT image as a GPT EFI System Partition, the USB/Ventoy boot path.
# The Live ESP is intentionally 256 MiB: a complete UKI includes kernel and
# initramfs and must not be truncated or split to satisfy the catalog field.
UKI_BYTES=0
for UKI in "$WORK_DIR"/aureaos-live-*.efi; do
    UKI_BYTES=$((UKI_BYTES + $(stat -c '%s' "$UKI")))
done
BOOTLOADER_BYTES="$(stat -c '%s' "$BOOTLOADER")"
ESP_BYTES=$((ESP_SIZE_MIB * 1024 * 1024))
# Keep FAT16 within its portable cluster-count limit for every supported ESP
# size. 256 MiB needs 8 KiB clusters; the maximum 2047 MiB ESP needs 32 KiB.
FAT_SECTORS_PER_CLUSTER=1
while (( (ESP_BYTES / 512 + FAT_SECTORS_PER_CLUSTER - 1) / FAT_SECTORS_PER_CLUSTER > 65524 )); do
    FAT_SECTORS_PER_CLUSTER=$((FAT_SECTORS_PER_CLUSTER * 2))
done
(( FAT_SECTORS_PER_CLUSTER <= 64 )) ||
    fail "configured ESP is too large for a portable FAT16 cluster layout"
(( UKI_BYTES + 2 * BOOTLOADER_BYTES + 8 * 1024 * 1024 < ESP_BYTES )) || {
    echo "UKI payload total: $UKI_BYTES bytes; systemd-boot copies: $((2 * BOOTLOADER_BYTES)) bytes; ESP: $ESP_BYTES bytes." >&2
    fail "UKI payloads do not fit in the configured ${ESP_SIZE_MIB} MiB Live ESP; increase AUREAOS_ESP_SIZE_MIB"
}
if (( ESP_BYTES > 65535 * 512 )); then
    echo "EFI image exceeds the El Torito 16-bit load-count range; using xorriso's UEFI full-medium convention and the matching GPT ESP."
fi

# systemd-boot scans this FAT image as its ESP. No firmware NVRAM entries are
# created and no host disk is touched by this build-time operation.
rm -f -- "$ESP_IMAGE"
truncate -s "${ESP_SIZE_MIB}M" "$ESP_IMAGE"
mkfs.vfat -F 16 -s "$FAT_SECTORS_PER_CLUSTER" -n AUREAOS "$ESP_IMAGE" >/dev/null

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
rm -rf -- "$ASSEMBLY_DIR/boot/grub" "$ASSEMBLY_DIR/boot/isolinux" \
    "$ASSEMBLY_DIR/isolinux" "$ASSEMBLY_DIR/syslinux"

mkdir -p -- "$(dirname -- "$OUTPUT_PATH")"
rm -f -- "$TEMP_ISO"
xorriso -as mkisofs \
    -r -J -joliet-long -iso-level 3 \
    -V AUREAOS_LIVE \
    -o "$TEMP_ISO" \
    -eltorito-alt-boot \
    -e boot/efi.img \
    -no-emul-boot \
    -efi-boot-part --efi-boot-image \
    "$ASSEMBLY_DIR"

# Validate the exact staged boot tree before publishing anything. The source
# live-build binary tree remains untouched and can be resumed if a check fails.
bash "$ISO_DIR/tests/validate-live-image.sh" --binary-dir "$ASSEMBLY_DIR"

[[ -s "$TEMP_ISO" ]] || fail "xorriso did not create the final ISO"

# Validate the exact mastered ISO, including its El Torito EFI entry and GPT
# EFI System Partition, before the temporary image is atomically published.
bash "$ISO_DIR/tests/validate-live-image.sh" --iso "$TEMP_ISO"

[[ -s "$TEMP_ISO" ]] || fail "final ISO disappeared before publication"
mv -f -- "$TEMP_ISO" "$OUTPUT_PATH"

echo "Created UEFI-only AureaOS Live ISO with systemd-boot and UKI:"
echo "  $OUTPUT_PATH"
echo "  ESP: boot/efi.img inside the ISO (${ESP_SIZE_MIB} MiB FAT16/GPT ESP)"
echo "  FAT16 sectors per cluster: $FAT_SECTORS_PER_CLUSTER"
echo "  Default entry: $DEFAULT_ENTRY"
