#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd -P)"
AUDIT="$REPO_ROOT/tools/security/audit-rootfs-permissions.sh"

if ((EUID != 0)); then
  echo "Run this test as root: sudo bash $0" >&2
  exit 2
fi

if ! getent group shadow >/dev/null 2>&1; then
  echo "The host must provide the shadow group for the Debian rootfs fixture." >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT
FAKE_BIN="$WORK/bin"
mkdir -p "$FAKE_BIN"

write_find_failure() {
  cat > "$FAKE_BIN/find" <<'EOF'
#!/usr/bin/env bash
exit 1
EOF
  chmod 0755 "$FAKE_BIN/find"
}
write_getcap() {
  local exit_code="$1"
  local output="${2:-}"
  cat > "$FAKE_BIN/getcap" <<EOF
#!/usr/bin/env bash
printf '%s\n' '$output'
exit $exit_code
EOF
  chmod 0755 "$FAKE_BIN/getcap"
}

make_valid_rootfs() {
  local rootfs="$1"
  local shadow_gid= messagebus_gid=100 mail_gid=8 journal_gid=101 relative

  shadow_gid="$(getent group shadow | cut -d: -f3)"
  mkdir -p "$rootfs"/{etc,usr/bin,usr/sbin,usr/lib/dbus-1.0,usr/lib/polkit-1,var/tmp,var/mail,var/log/journal,root,tmp,dev/shm}
  touch "$rootfs/etc/passwd" "$rootfs/etc/group" "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
  printf 'root:x:0:\nshadow:x:%s:\nmessagebus:x:%s:\nmail:x:%s:\nsystemd-journal:x:%s:\n' \
    "$shadow_gid" "$messagebus_gid" "$mail_gid" "$journal_gid" > "$rootfs/etc/group"

  local -a root_suid_files=(
    usr/bin/chfn
    usr/bin/chsh
    usr/bin/gpasswd
    usr/bin/mount
    usr/bin/newgrp
    usr/bin/passwd
    usr/bin/su
    usr/bin/umount
    usr/bin/sudo
    usr/lib/polkit-1/polkit-agent-helper-1
  )
  local -a shadow_sgid_files=(
    usr/bin/chage
    usr/bin/expiry
    usr/sbin/unix_chkpwd
  )

  for relative in "${root_suid_files[@]}" "${shadow_sgid_files[@]}" \
    usr/lib/dbus-1.0/dbus-daemon-launch-helper; do
    touch "$rootfs/$relative"
  done

  chown -R root:root "$rootfs"
  chown "root:$shadow_gid" "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
  for relative in "${shadow_sgid_files[@]}"; do
    chown "root:$shadow_gid" "$rootfs/$relative"
  done
  chown "root:$messagebus_gid" "$rootfs/usr/lib/dbus-1.0/dbus-daemon-launch-helper"
  chown "root:$mail_gid" "$rootfs/var/mail"
  chown "root:$journal_gid" "$rootfs/var/log/journal"

  chmod 0755 "$rootfs" "$rootfs/etc" "$rootfs/usr" "$rootfs/var"
  chmod 0700 "$rootfs/root"
  chmod 1777 "$rootfs/tmp" "$rootfs/var/tmp" "$rootfs/dev/shm"
  chmod 2775 "$rootfs/var/mail"
  chmod 2755 "$rootfs/var/log/journal"
  chmod 0644 "$rootfs/etc/passwd" "$rootfs/etc/group"
  chmod 0640 "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
  for relative in "${root_suid_files[@]}"; do
    chmod 4755 "$rootfs/$relative"
  done
  for relative in "${shadow_sgid_files[@]}"; do
    chmod 2755 "$rootfs/$relative"
  done
  chmod 4754 "$rootfs/usr/lib/dbus-1.0/dbus-daemon-launch-helper"
}

expect_pass() {
  local label="$1"
  shift
  if "$@"; then
    printf 'PASS  %s\n' "$label"
  else
    printf 'FAIL  %s (expected success)\n' "$label" >&2
    exit 1
  fi
}

expect_fail() {
  local label="$1"
  shift
  if "$@"; then
    printf 'FAIL  %s (expected failure)\n' "$label" >&2
    exit 1
  else
    printf 'PASS  %s\n' "$label"
  fi
}

ROOTFS="$WORK/rootfs"
make_valid_rootfs "$ROOTFS"
write_getcap 0
expect_pass "valid minimal rootfs" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

mkdir -p "$ROOTFS/usr/bin"
touch "$ROOTFS/usr/bin/passwd"
chmod 4755 "$ROOTFS/usr/bin/passwd"
expect_pass "accept allowlisted SUID mode" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
chmod 4555 "$ROOTFS/usr/bin/passwd"
expect_fail "reject unexpected allowlisted SUID mode" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
chmod 4755 "$ROOTFS/usr/bin/passwd"
chown 1000:1000 "$ROOTFS/usr/bin/sudo"
expect_fail "reject untrusted SUID owner and group" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
chown root:root "$ROOTFS/usr/bin/sudo"
rm -f "$ROOTFS/usr/bin/passwd"

touch "$ROOTFS/unapproved-world-writable"
chmod 0666 "$ROOTFS/unapproved-world-writable"
expect_fail "reject world-writable file" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rm -f "$ROOTFS/unapproved-world-writable"

mkfifo "$ROOTFS/world-writable-fifo"
chmod 0666 "$ROOTFS/world-writable-fifo"
expect_fail "reject world-writable FIFO" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rm -f "$ROOTFS/world-writable-fifo"

mkdir -p "$ROOTFS/usr/local/bin"
touch "$ROOTFS/usr/local/bin/unapproved-suid"
chmod 4755 "$ROOTFS/usr/local/bin/unapproved-suid"
expect_fail "reject unapproved SUID executable" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rm -f "$ROOTFS/usr/local/bin/unapproved-suid"

mkdir "$ROOTFS/unapproved-sgid-directory"
chmod 2755 "$ROOTFS/unapproved-sgid-directory"
expect_fail "reject unapproved SGID directory" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rmdir "$ROOTFS/unapproved-sgid-directory"

chmod 2777 "$ROOTFS/var/mail"
expect_fail "reject unexpected SGID mail-spool mode" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
chmod 2775 "$ROOTFS/var/mail"

write_getcap 0 "$ROOTFS/usr/bin/example cap_net_raw=ep"
expect_fail "reject reported file capabilities" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

write_getcap 2
expect_fail "fail closed when getcap exits unsuccessfully" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

write_getcap 0
write_find_failure
expect_fail "fail closed when rootfs traversal fails" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

echo "All rootfs permission audit fixture tests passed."
