#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

ROOTFS_MODE="755"
if (($# == 2)) && [[ "$1" == "--staging" ]]; then
  ROOTFS_MODE="700"
  ROOTFS_INPUT="$2"
elif (($# == 1)); then
  ROOTFS_INPUT="$1"
else
  echo "Uso: $0 [--staging] /percorso/rootfs" >&2
  exit 2
fi

ROOTFS="${ROOTFS_INPUT%/}"
if [[ -z "$ROOTFS" || ! -d "$ROOTFS" || -L "$ROOTFS" ]]; then
  echo "Errore: rootfs non valida: $ROOTFS_INPUT" >&2
  exit 2
fi
ROOTFS="$(realpath -e -- "$ROOTFS")"

failures=0
SCAN_FILE="$(mktemp)"
trap 'rm -f -- "$SCAN_FILE"' EXIT

fail() {
  printf 'FAIL  %s\n' "$1" >&2
  failures=$((failures + 1))
}

rootfs_group_id() {
  local group_name="$1"

  awk -F: -v name="$group_name" '
    $1 == name {
      count++
      gid = $3
    }
    END {
      if (count != 1) {
        exit 1
      }
      print gid
    }
  ' "$ROOTFS/etc/group"
}

check_entry() {
  local relative_path="$1"
  local expected="$2"
  local path="$ROOTFS/$relative_path"
  local expected_mode expected_uid expected_group expected_gid
  local actual

  if [[ ! -e "$path" || -L "$path" ]]; then
    fail "$relative_path assente o symlink inatteso"
    return
  fi

  IFS=: read -r expected_mode expected_uid expected_group <<< "$expected"
  if [[ "$expected_group" =~ ^[0-9]+$ ]]; then
    expected_gid="$expected_group"
  elif ! expected_gid="$(rootfs_group_id "$expected_group")"; then
    fail "$relative_path: gruppo '$expected_group' assente o duplicato in rootfs/etc/group"
    return
  fi

  actual="$(stat -c '%a:%u:%g' -- "$path")"
  expected="$expected_mode:$expected_uid:$expected_gid"
  if [[ "$actual" != "$expected" ]]; then
    fail "$relative_path: atteso $expected, trovato $actual"
  else
    printf 'PASS  %-24s %s\n' "$relative_path" "$actual"
  fi
}

echo "===== AureaOS rootfs permission audit ====="

check_entry "." "$ROOTFS_MODE:0:0"
check_entry "etc" "755:0:0"
check_entry "usr" "755:0:0"
check_entry "var" "755:0:0"
check_entry "root" "700:0:0"
check_entry "tmp" "1777:0:0"
check_entry "etc/passwd" "644:0:0"
check_entry "etc/group" "644:0:0"
check_entry "etc/shadow" "640:0:shadow"
check_entry "etc/gshadow" "640:0:shadow"

if [[ -e "$ROOTFS/etc/sudoers" ]]; then
  check_entry "etc/sudoers" "440:0:0"
fi
if [[ -e "$ROOTFS/etc/sudoers.d" ]]; then
  check_entry "etc/sudoers.d" "750:0:0"
fi

echo
echo "Verifica elementi world-writable nell'intera rootfs..."
# Only the standard shared temporary directories may be world-writable, and
# only with the sticky bit. All other world-writable non-symlink objects fail.
if find "$ROOTFS" -xdev ! -type l -perm -0002 -print0 >"$SCAN_FILE"; then
  while IFS= read -r -d '' path; do
    relative="${path#"$ROOTFS"/}"
    mode="$(stat -c '%a' -- "$path")"
    if [[ "$relative" == "tmp" || "$relative" == "var/tmp" || "$relative" == "dev/shm" ]] &&
       [[ -d "$path" && "$mode" == 1777 ]]; then
      printf 'PASS  shared temporary directory %s (%s)\n' "$relative" "$mode"
      continue
    fi
    fail "elemento world-writable non consentito: /$relative (mode $mode)"
  done < "$SCAN_FILE"
else
  fail "scansione world-writable incompleta: find ha restituito un errore"
fi

echo
echo "Verifica SUID/SGID contro allowlist..."
# Debian base utilities whose privilege bits are expected. Any new path must
# be reviewed and added deliberately; package additions cannot silently expand
# the privileged executable surface.
declare -A SUID_SGID_ALLOWLIST=(
  ["usr/bin/chfn"]="4755:0:0"
  ["usr/bin/chsh"]="4755:0:0"
  ["usr/bin/gpasswd"]="4755:0:0"
  ["usr/bin/mount"]="4755:0:0"
  ["usr/bin/newgrp"]="4755:0:0"
  ["usr/bin/passwd"]="4755:0:0"
  ["usr/bin/su"]="4755:0:0"
  ["usr/bin/umount"]="4755:0:0"
  ["usr/bin/sudo"]="4755:0:0"
  ["usr/bin/chage"]="2755:0:shadow"
  ["usr/bin/expiry"]="2755:0:shadow"
  ["usr/sbin/unix_chkpwd"]="2755:0:shadow"
  ["usr/lib/dbus-1.0/dbus-daemon-launch-helper"]="4754:0:messagebus"
)

if find "$ROOTFS" -xdev ! -type l -perm /6000 -print0 >"$SCAN_FILE"; then
  while IFS= read -r -d '' path; do
    relative="${path#"$ROOTFS"/}"
    actual="$(stat -c '%a:%u:%g' -- "$path")"
    expected="${SUID_SGID_ALLOWLIST[$relative]:-}"
    if [[ -z "$expected" ]]; then
      fail "SUID/SGID non autorizzato: /$relative ($actual)"
    elif [[ ! -f "$path" || -L "$path" ]]; then
      fail "entry SUID/SGID allowlist non è un file regolare: /$relative"
    else
      IFS=: read -r expected_mode expected_uid expected_group <<< "$expected"
      if [[ "$expected_group" =~ ^[0-9]+$ ]]; then
        expected_gid="$expected_group"
      elif ! expected_gid="$(rootfs_group_id "$expected_group")"; then
        fail "gruppo '$expected_group' assente o duplicato per /$relative"
        continue
      fi
      expected="$expected_mode:$expected_uid:$expected_gid"
      if [[ "$actual" != "$expected" ]]; then
        fail "owner/mode SUID/SGID inatteso per /$relative: atteso $expected, trovato $actual"
      else
        printf 'PASS  SUID/SGID allowlist /%s (%s)\n' "$relative" "$actual"
      fi
    fi
  done < "$SCAN_FILE"
else
  fail "scansione SUID/SGID incompleta: find ha restituito un errore"
fi

echo
echo "Verifica file capabilities..."
if ! command -v getcap >/dev/null 2>&1; then
  fail "getcap non disponibile: installare libcap (strumento getcap) sull'host di build"
else
  if capabilities="$(getcap -r "$ROOTFS" 2>&1)"; then
    if [[ -n "$capabilities" ]]; then
      while IFS= read -r capability; do
        [[ -z "$capability" ]] && continue
        fail "file capability non approvata: $capability"
      done <<< "$capabilities"
    else
      echo "PASS  nessuna file capability presente"
    fi
  else
    fail "impossibile completare la scansione file capabilities: $capabilities"
  fi
fi

echo
if ((failures > 0)); then
  printf 'Audit permessi FALLITO: %d verifica/e non superata/e.\n' "$failures" >&2
  exit 1
fi

echo "Audit permessi superato."
