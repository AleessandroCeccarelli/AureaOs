#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Validate Debian Live boot prerequisites inside the target chroot.
set -eu

fail() {
    echo "AureaOS boot preflight: $*" >&2
    exit 1
}

for package in live-boot live-boot-initramfs-tools live-config live-config-systemd systemd-sysv initramfs-tools linux-image-amd64; do
    status="$(dpkg-query -W -f='${db:Status-Status}' "$package" 2>/dev/null || true)"
    [ "$status" = installed ] || fail "required package is not installed: $package"
done

[ -x /usr/share/initramfs-tools/hooks/live ] ||
    fail "live-boot initramfs hook is missing"
[ -x /usr/share/initramfs-tools/scripts/live ] ||
    fail "live-boot early-boot script is missing"
command -v update-initramfs >/dev/null 2>&1 ||
    fail "update-initramfs is unavailable"
command -v lsinitramfs >/dev/null 2>&1 ||
    fail "lsinitramfs is unavailable"

# Regenerate initramfs only after the live-boot backend and kernel are installed.
update-initramfs -u -k all

found=0
for initrd in /boot/initrd.img-*; do
    [ -f "$initrd" ] || continue
    found=1
    if ! lsinitramfs "$initrd" | grep -Fxq 'scripts/live'; then
        fail "$initrd does not contain scripts/live"
    fi
    echo "Validated initramfs: $initrd"
done

[ "$found" -eq 1 ] || fail "no generated /boot/initrd.img-* found"
echo "AureaOS boot preflight passed inside chroot."
