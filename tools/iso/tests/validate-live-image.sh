#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Validate the boot payload in a live-build binary tree or an ISO file.
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

command -v lsinitramfs >/dev/null 2>&1 || fail "lsinitramfs is required"
command -v grep >/dev/null 2>&1 || fail "grep is required"

TEMP_DIR=""
cleanup() {
    [[ -z "$TEMP_DIR" ]] || rm -rf -- "$TEMP_DIR"
}
trap cleanup EXIT

if [[ "$MODE" == "--iso" ]]; then
    [[ -s "$TARGET" ]] || fail "ISO is missing or empty: $TARGET"
    command -v xorriso >/dev/null 2>&1 || fail "xorriso is required to inspect the ISO"
    TEMP_DIR="$(mktemp -d)"
    mkdir -p "$TEMP_DIR/live" "$TEMP_DIR/boot/grub"
    xorriso -osirrox on -indev "$TARGET" \
        -extract /live "$TEMP_DIR/live" \
        -extract /boot/grub "$TEMP_DIR/boot/grub" >/dev/null
    ROOT="$TEMP_DIR"
else
    [[ -d "$TARGET" ]] || fail "binary tree does not exist: $TARGET"
    ROOT="$TARGET"
fi

LIVE_DIR="$ROOT/live"
GRUB_DIR="$ROOT/boot/grub"
[[ -d "$LIVE_DIR" ]] || fail "missing /live directory in boot payload"
[[ -d "$GRUB_DIR" ]] || fail "missing /boot/grub directory in boot payload"

shopt -s nullglob
kernels=("$LIVE_DIR"/vmlinuz-*)
initrds=("$LIVE_DIR"/initrd.img-*)
((${#kernels[@]} > 0)) || fail "no kernel found under /live"
((${#initrds[@]} > 0)) || fail "no initramfs found under /live"

for kernel in "${kernels[@]}"; do
    [[ -s "$kernel" ]] || fail "empty kernel: $kernel"
    echo "Kernel found: ${kernel##*/}"
done

for initrd in "${initrds[@]}"; do
    [[ -s "$initrd" ]] || fail "empty initramfs: $initrd"
    listing="$(lsinitramfs "$initrd")" || fail "cannot read initramfs: $initrd"
    grep -Fxq 'scripts/live' <<<"$listing" ||
        fail "${initrd##*/} does not contain scripts/live"
    echo "Live boot script found in: ${initrd##*/}"
done

grub_cfg="$GRUB_DIR/grub.cfg"
global_cfg="$GRUB_DIR/config.cfg"
[[ -s "$grub_cfg" ]] || fail "GRUB grub.cfg is missing"
[[ -s "$global_cfg" ]] || fail "GRUB config.cfg is missing"
grep -Fq 'menuentry "AureaOS"' "$grub_cfg" ||
    fail "AureaOS GRUB menu entry is missing"
grep -Fq 'boot=live' "$grub_cfg" ||
    fail "GRUB entry does not pass boot=live"
grep -Eq '^[[:space:]]*set default=0$' "$global_cfg" ||
    fail "GRUB default entry is not set to 0"
grep -Eq '^[[:space:]]*set timeout=5$' "$global_cfg" ||
    fail "GRUB timeout is not set to 5 seconds"

echo "AureaOS boot image validation passed ($MODE)."
