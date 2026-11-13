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
printf '%s\\n' '$output'
exit $exit_code
EOF
  chmod 0755 "$FAKE_BIN/getcap"
}

make_valid_rootfs() {
  local rootfs="$1"
  mkdir -p "$rootfs"/{etc,usr,var/tmp,root,tmp,dev/shm}
  touch "$rootfs/etc/passwd" "$rootfs/etc/group" "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
  chown -R root:root "$rootfs"
  chown root:shadow "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
  chmod 0755 "$rootfs" "$rootfs/etc" "$rootfs/usr" "$rootfs/var"
  chmod 0700 "$rootfs/root"
  chmod 1777 "$rootfs/tmp" "$rootfs/var/tmp" "$rootfs/dev/shm"
  chmod 0644 "$rootfs/etc/passwd" "$rootfs/etc/group"
  chmod 0640 "$rootfs/etc/shadow" "$rootfs/etc/gshadow"
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
rm -f "$ROOTFS/usr/bin/passwd"

touch "$ROOTFS/unapproved-world-writable"
chmod 0666 "$ROOTFS/unapproved-world-writable"
expect_fail "reject world-writable file" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rm -f "$ROOTFS/unapproved-world-writable"

mkdir -p "$ROOTFS/usr/local/bin"
touch "$ROOTFS/usr/local/bin/unapproved-suid"
chmod 4755 "$ROOTFS/usr/local/bin/unapproved-suid"
expect_fail "reject unapproved SUID executable" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"
rm -f "$ROOTFS/usr/local/bin/unapproved-suid"

write_getcap 0 "$ROOTFS/usr/bin/example cap_net_raw=ep"
expect_fail "reject reported file capabilities" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

write_getcap 2
expect_fail "fail closed when getcap exits unsuccessfully" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

write_getcap 0
write_find_failure
expect_fail "fail closed when rootfs traversal fails" env PATH="$FAKE_BIN:$PATH" bash "$AUDIT" "$ROOTFS"

echo "All rootfs permission audit fixture tests passed."
