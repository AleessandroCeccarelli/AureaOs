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

failures=0

check_entry() {
  local relative_path="$1"
  local expected="$2"
  local path="$ROOTFS/$relative_path"
  local actual

  if [[ ! -e "$path" || -L "$path" ]]; then
    printf 'FAIL  %-24s assente o symlink inatteso\n' "$relative_path" >&2
    failures=$((failures + 1))
    return
  fi

  actual="$(stat -c '%a:%U:%G' -- "$path")"
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL  %-24s atteso %s, trovato %s\n' "$relative_path" "$expected" "$actual" >&2
    failures=$((failures + 1))
  else
    printf 'PASS  %-24s %s\n' "$relative_path" "$actual"
  fi
}

echo "===== AureaOS rootfs permission audit ====="

# Debian Trixie base filesystem invariants. These checks intentionally target
# exact security-sensitive paths instead of applying recursive chmod rules.
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
echo "Verifica assenza di elementi world-writable sotto /etc e /usr..."
world_writable="$(find "$ROOTFS/etc" "$ROOTFS/usr" -xdev \
  \( -type f -o -type d \) -perm -0002 -print -quit)"

if [[ -n "$world_writable" ]]; then
  printf 'FAIL  elemento world-writable: %s\n' "$world_writable" >&2
  failures=$((failures + 1))
else
  echo "PASS  nessun file o directory world-writable in /etc e /usr"
fi

echo
echo "Inventario SUID/SGID presente nella rootfs (richiede revisione allowlist):"
find "$ROOTFS" -xdev -type f -perm /6000 -printf '%P %m %u:%g\n' | sort || true

echo
if ((failures > 0)); then
  printf 'Audit permessi FALLITO: %d verifica/e non superata/e.\n' "$failures" >&2
  exit 1
fi

echo "Audit permessi superato."
