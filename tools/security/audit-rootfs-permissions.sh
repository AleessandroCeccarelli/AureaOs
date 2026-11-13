#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

if (($# != 1)); then
  echo "Uso: $0 /percorso/rootfs" >&2
  exit 2
fi

ROOTFS="${1%/}"
if [[ -z "$ROOTFS" || ! -d "$ROOTFS" || -L "$ROOTFS" ]]; then
  echo "Errore: rootfs non valida: $1" >&2
  exit 2
fi
ROOTFS="$(realpath -e -- "$ROOTFS")"

failures=0

fail() {
  printf 'FAIL  %s\n' "$1" >&2
  failures=$((failures + 1))
}

check_entry() {
  local relative_path="$1"
  local expected="$2"
  local path="$ROOTFS/$relative_path"
  local actual

  if [[ ! -e "$path" || -L "$path" ]]; then
    fail "$relative_path assente o symlink inatteso"
    return
  fi

  actual="$(stat -c '%a:%U:%G' -- "$path")"
  if [[ "$actual" != "$expected" ]]; then
    fail "$relative_path: atteso $expected, trovato $actual"
  else
    printf 'PASS  %-24s %s\n' "$relative_path" "$actual"
  fi
}

echo "===== AureaOS rootfs permission audit ====="

check_entry "." "755:root:root"
check_entry "etc" "755:root:root"
check_entry "usr" "755:root:root"
check_entry "var" "755:root:root"
check_entry "root" "700:root:root"
check_entry "tmp" "1777:root:root"
check_entry "etc/passwd" "644:root:root"
check_entry "etc/group" "644:root:root"
check_entry "etc/shadow" "640:root:shadow"
check_entry "etc/gshadow" "640:root:shadow"

if [[ -e "$ROOTFS/etc/sudoers" ]]; then
  check_entry "etc/sudoers" "440:root:root"
fi
if [[ -e "$ROOTFS/etc/sudoers.d" ]]; then
  check_entry "etc/sudoers.d" "750:root:root"
fi

echo
echo "Verifica elementi world-writable nell'intera rootfs..."
# Only the standard shared temporary directories may be world-writable, and
# only with the sticky bit. Any world-writable regular file is rejected.
while IFS= read -r -d '' path; do
  relative="${path#"$ROOTFS"/}"
  mode="$(stat -c '%a' -- "$path")"
  if [[ "$relative" == "tmp" || "$relative" == "var/tmp" || "$relative" == "dev/shm" ]]; then
    if [[ -d "$path" && "$mode" == 1777 ]]; then
      printf 'PASS  shared temporary directory %s (%s)\n' "$relative" "$mode"
      continue
    fi
  fi
  fail "elemento world-writable non consentito: /$relative (mode $mode)"
done < <(find "$ROOTFS" -xdev \( -type f -o -type d \) -perm -0002 -print0)

echo
echo "Verifica SUID/SGID contro allowlist..."
# Debian base utilities whose privilege bits are expected. Any new path must
# be reviewed and added deliberately; package additions cannot silently expand
# the privileged executable surface.
declare -A SUID_SGID_ALLOWLIST=(
  ["usr/bin/chfn"]=4755
  ["usr/bin/chsh"]=4755
  ["usr/bin/gpasswd"]=4755
  ["usr/bin/mount"]=4755
  ["usr/bin/newgrp"]=4755
  ["usr/bin/passwd"]=4755
  ["usr/bin/su"]=4755
  ["usr/bin/umount"]=4755
)

while IFS= read -r -d '' path; do
  relative="${path#"$ROOTFS"/}"
  mode="$(stat -c '%a' -- "$path")"
  if [[ -z "${SUID_SGID_ALLOWLIST[$relative]+allowed}" ]]; then
    fail "SUID/SGID non autorizzato: /$relative ($mode)"
  elif [[ "$mode" != "${SUID_SGID_ALLOWLIST[$relative]}" ]]; then
    fail "mode SUID/SGID inatteso per /$relative: atteso ${SUID_SGID_ALLOWLIST[$relative]}, trovato $mode"
  else
    printf 'PASS  SUID/SGID allowlist /%s (%s)\n' "$relative" "$mode"
  fi
done < <(find "$ROOTFS" -xdev -type f -perm /6000 -print0)

echo
echo "Verifica file capabilities..."
if ! command -v getcap >/dev/null 2>&1; then
  fail "getcap non disponibile: installare libcap (strumento getcap) sull'host di build"
else
  capabilities="$(getcap -r "$ROOTFS" 2>/dev/null || true)"
  if [[ -n "$capabilities" ]]; then
    while IFS= read -r capability; do
      fail "file capability non approvata: $capability"
    done <<< "$capabilities"
  else
    echo "PASS  nessuna file capability presente"
  fi
fi

echo
if ((failures > 0)); then
  printf 'Audit permessi FALLITO: %d verifica/e non superata/e.\n' "$failures" >&2
  exit 1
fi

echo "Audit permessi superato."
