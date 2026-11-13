#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

# Keep bootstrap-generated temporary data private by default.
umask 077

# AureaOS Roma 1.0 — Debian bootstrap and package-profile installer.
# Creates a Debian root filesystem outside the repository.
# Installs the version-controlled terminal-only package manifests.

readonly CODENAME="trixie"
readonly ARCH="amd64"
readonly VARIANT="minbase"
readonly COMPONENTS="main,contrib,non-free,non-free-firmware"
readonly COMPONENTS_SPACE="main contrib non-free non-free-firmware"
readonly MIRROR="https://deb.debian.org/debian"
readonly SECURITY_MIRROR="https://security.debian.org/debian-security"

SCRIPT_DIR="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P
)"
readonly SCRIPT_DIR
REPO_ROOT="$(
  cd -- "$SCRIPT_DIR/../.." && pwd -P
)"
readonly REPO_ROOT
readonly PACKAGE_DIR="$REPO_ROOT/debian/packages"

TARGET="${HOME}/Documenti/AureaOS-Build/rootfs"
ROOTFS_WORK=""
BOOT_MODE="none"
WITH_RAR=0

SOURCES_TMP=""
POLICY_TMP=""
POLICY_INSTALLED=0

# Execute all Debian package configuration inside systemd-nspawn. It provides
# private API filesystems and a restricted capability set instead of exposing
# the host /dev and /sys trees through recursive bind mounts.
run_in_rootfs() {
  sudo systemd-nspawn \
    --quiet \
    --register=no \
    --directory="$ROOTFS_WORK" \
    --private-users=pick \
    --private-users-ownership=map \
    --as-pid2 \
    --drop-capability=CAP_SYS_ADMIN \
    --drop-capability=CAP_SYS_MODULE \
    --drop-capability=CAP_SYS_RAWIO \
    --drop-capability=CAP_SYS_BOOT \
    --drop-capability=CAP_SYS_PTRACE \
    -- "$@"
}

usage() {
  cat <<'USAGE'
AureaOS Debian bootstrap

Usage:
  ./tools/build/bootstrap-debian.sh [OPTIONS]

Options:
  --target PATH       Destination for the new root filesystem.
                      Default: ~/Documenti/AureaOS-Build/rootfs
  --boot-mode MODE    none (default), bios, or uefi.
  --with-rar          Include optional 7zip-rar support.
  --help              Show this help and exit.

Creates a new Debian Trixie rootfs, configures APT and installs
the version-controlled terminal-only package manifests.

The script never intentionally overwrites or deletes an existing
destination. The destination must be outside the Git repository.
No packages are installed on the host system.
USAGE
}

cleanup() {
  local status=$?

  trap - EXIT
  set +e

  if [[ -n "$ROOTFS_WORK" && -d "$ROOTFS_WORK" ]] && ((POLICY_INSTALLED)); then
    echo "Pulizia: rimozione di policy-rc.d..." >&2
    sudo rm -f -- "$ROOTFS_WORK/usr/sbin/policy-rc.d"
  fi

  [[ -z "$SOURCES_TMP" ]] || rm -f -- "$SOURCES_TMP"
  [[ -z "$POLICY_TMP" ]] || rm -f -- "$POLICY_TMP"

  if ((status != 0)); then
    echo "Bootstrap interrotto con errore (codice $status)." >&2
    echo "La rootfs di staging potrebbe essere incompleta:" >&2
    echo "$ROOTFS_WORK" >&2
    echo "Non verrà cancellata automaticamente." >&2
  fi

  exit "$status"
}
trap cleanup EXIT

while (($#)); do
  case "$1" in
    --target)
      if (($# < 2)) || [[ -z "$2" ]]; then
        echo "Errore: --target richiede un percorso." >&2
        exit 2
      fi
      TARGET="$2"
      shift 2
      ;;
    --boot-mode)
      if (($# < 2)) || [[ -z "$2" ]]; then
        echo "Errore: --boot-mode richiede none, bios oppure uefi." >&2
        exit 2
      fi
      BOOT_MODE="$2"
      shift 2
      ;;
    --with-rar)
      WITH_RAR=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Opzione non riconosciuta: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$BOOT_MODE" in
  none|bios|uefi) ;;
  *)
    echo "Errore: --boot-mode accetta solo none, bios oppure uefi." >&2
    exit 2
    ;;
esac

if [[ "$TARGET" != /* || "$TARGET" == "/" ]]; then
  echo "Errore: --target deve essere un percorso assoluto valido." >&2
  exit 2
fi

if ! command -v debootstrap >/dev/null 2>&1; then
  echo "Errore: debootstrap non è installato." >&2
  echo "Su Fedora: sudo dnf install debootstrap" >&2
  exit 1
fi

if ! command -v systemd-nspawn >/dev/null 2>&1; then
  echo "Errore: systemd-nspawn non è installato." >&2
  echo "Su Fedora: sudo dnf install systemd-container" >&2
  exit 1
fi

if ! command -v getcap >/dev/null 2>&1; then
  echo "Errore: getcap non è installato; serve per verificare le file capabilities della rootfs." >&2
  echo "Su Fedora: sudo dnf install libcap" >&2
  exit 1
fi

if [[ ! -d "$PACKAGE_DIR" ]]; then
  echo "Errore: manifesti non trovati: $PACKAGE_DIR" >&2
  exit 1
fi

TARGET_PARENT="$(dirname -- "$TARGET")"
TARGET_NAME="$(basename -- "$TARGET")"

if [[ "$TARGET_NAME" == "." || "$TARGET_NAME" == ".." || -z "$TARGET_NAME" ]]; then
  echo "Errore: nome della destinazione non valido." >&2
  exit 2
fi

# Resolve existing symlink components and reject repository-local paths before
# creating the parent directory. This keeps even a failed invocation from
# leaving accidental directories inside the checkout.
TARGET_PARENT="$(realpath -m -- "$TARGET_PARENT")"
case "$TARGET_PARENT/" in
  "$REPO_ROOT/"*)
    echo "Errore: la rootfs deve stare fuori dal repository Git." >&2
    exit 1
    ;;
esac

mkdir -p -- "$TARGET_PARENT"
TARGET_PARENT="$(cd -- "$TARGET_PARENT" && pwd -P)"
TARGET="${TARGET_PARENT}/${TARGET_NAME}"

case "$TARGET/" in
  "$REPO_ROOT/"*)
    echo "Errore: la rootfs deve stare fuori dal repository Git." >&2
    exit 1
    ;;
esac

if [[ -e "$TARGET" || -L "$TARGET" ]]; then
  echo "Errore: la destinazione esiste già: $TARGET" >&2
  echo "Nessun file è stato modificato o eliminato." >&2
  echo "Scegli un percorso nuovo usando --target." >&2
  exit 1
fi

declare -A SEEN_PACKAGES=()
packages=()

# Load manifests directly so a missing/unreadable file cannot be hidden by
# process substitution. Validate every token before it reaches apt-get.
load_manifest() {
  local manifest="$1"
  local file="$PACKAGE_DIR/$manifest"
  local line package
  local -a entries=()

  if [[ ! -f "$file" || -L "$file" ]]; then
    echo "Errore: manifesto assente, non regolare o symlink: $file" >&2
    return 1
  fi

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"
    entries=()
    read -r -a entries <<< "$line"

    for package in "${entries[@]}"; do
      if [[ ! "$package" =~ ^[a-z0-9][a-z0-9+.-]*(:[a-z0-9][a-z0-9-]*)?$ ]]; then
        echo "Errore: nome pacchetto non valido in $manifest: $package" >&2
        return 1
      fi

      if [[ -z "${SEEN_PACKAGES[$package]+present}" ]]; then
        SEEN_PACKAGES["$package"]=1
        packages+=("$package")
      fi
    done
  done < "$file"
}

for manifest in \
  base.list \
  networking.list \
  bluetooth.list \
  security.list \
  storage.list \
  archives.list \
  firmware.list \
  boot.list
do
  load_manifest "$manifest"
done

case "$BOOT_MODE" in
  bios) load_manifest "boot-bios.list" ;;
  uefi) load_manifest "boot-uefi.list" ;;
esac

if ((WITH_RAR)); then
  load_manifest "rar-extraction-optional.list"
fi

if (("${#packages[@]}" == 0)); then
  echo "Errore: nessun pacchetto valido nei manifesti." >&2
  exit 1
fi

printf '%s\n' \
  "===== AureaOS Debian Bootstrap =====" \
  "Distribuzione: Debian GNU/Linux 13 (${CODENAME})" \
  "Architettura:  ${ARCH}" \
  "Profilo:       ${VARIANT}" \
  "Componenti:    ${COMPONENTS_SPACE}" \
  "Avvio GRUB:    ${BOOT_MODE}" \
  "Supporto RAR:  $([[ $WITH_RAR -eq 1 ]] && echo sì || echo no)" \
  "Destinazione:  ${TARGET}" \
  "Pacchetti manifesti: ${#packages[@]}" \
  ""

echo "Verrà creata una nuova rootfs."
echo "Non verrà creata un'immagine ISO."
echo "Nessun pacchetto verrà installato sul sistema host."
echo

sudo -v

# Pin the destination parent and build in a root-owned staging directory.
cd -- "$TARGET_PARENT"
PINNED_PARENT="$(pwd -P)"
if [[ "$PINNED_PARENT" != "$TARGET_PARENT" ]]; then
  echo "Errore: la directory destinazione è cambiata durante la sua apertura." >&2
  exit 1
fi
PARENT_ID="$(stat -c '%d:%i' -- .)"
PARENT_DEVICE="$(stat -c '%d' -- .)"
if [[ ! -d /var/tmp || -L /var/tmp ]]; then
  echo "Errore: /var/tmp deve essere una directory reale." >&2
  exit 1
fi
VAR_TMP_OWNER_MODE="$(stat -c '%u:%a' -- /var/tmp)"
if [[ "$VAR_TMP_OWNER_MODE" != "0:1777" ]]; then
  echo "Errore: /var/tmp deve appartenere a root ed avere mode 1777." >&2
  exit 1
fi
ROOTFS_WORK="$(sudo mktemp -d --tmpdir=/var/tmp aureaos-rootfs.XXXXXXXXXX)"
STAGING_DEVICE="$(sudo stat -c '%d' -- "$ROOTFS_WORK")"
if [[ "$STAGING_DEVICE" != "$PARENT_DEVICE" ]]; then
  echo "Errore: /var/tmp e la destinazione devono trovarsi sullo stesso filesystem." >&2
  echo "Staging lasciato intatto per ispezione: $ROOTFS_WORK" >&2
  exit 1
fi

# Create the base Debian filesystem in protected staging, never at the
# user-controlled final pathname.
sudo debootstrap \
  --foreign \
  --arch="$ARCH" \
  --variant="$VARIANT" \
  --components="$COMPONENTS" \
  "$CODENAME" \
  "$ROOTFS_WORK" \
  "$MIRROR"

# Complete debootstrap's second stage inside the isolated container.
run_in_rootfs /debootstrap/debootstrap --second-stage

# Prepare temporary configuration files.
SOURCES_TMP="$(mktemp)"
POLICY_TMP="$(mktemp)"

cat > "$SOURCES_TMP" <<APT_SOURCES
Types: deb
URIs: ${MIRROR}
Suites: ${CODENAME} ${CODENAME}-updates
Components: ${COMPONENTS_SPACE}
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

Types: deb
URIs: ${SECURITY_MIRROR}
Suites: ${CODENAME}-security
Components: ${COMPONENTS_SPACE}
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg
APT_SOURCES

sudo install -D -m 0644 \
  "$SOURCES_TMP" "$ROOTFS_WORK/etc/apt/sources.list.d/debian.sources"

# Keep a backup of the legacy sources file if one exists.
if sudo test -f "$ROOTFS_WORK/etc/apt/sources.list"; then
  if sudo test -e "$ROOTFS_WORK/etc/apt/sources.list.bootstrap-backup"; then
    echo "Errore: esiste già sources.list.bootstrap-backup." >&2
    exit 1
  fi

  sudo mv \
    "$ROOTFS_WORK/etc/apt/sources.list" \
    "$ROOTFS_WORK/etc/apt/sources.list.bootstrap-backup"
fi

# Run package maintainer scripts in an isolated container. The container
# retains outbound network access for authenticated Debian repository traffic,
# but does not receive recursive host /dev or /sys bind mounts.
echo
echo "===== Preparazione ambiente Debian isolato ====="
for mountpoint in dev proc sys run; do
  if sudo mountpoint -q "$ROOTFS_WORK/$mountpoint"; then
    echo "Errore: $ROOTFS_WORK/$mountpoint è già un mountpoint." >&2
    exit 1
  fi
done

# Use deb822 sources to refresh package metadata.
echo
echo "===== Aggiornamento cataloghi APT ====="
run_in_rootfs apt-get update

# Suppress automatic service starts inside the build chroot.
cat > "$POLICY_TMP" <<'POLICY_RC'
#!/bin/sh
exit 101
POLICY_RC

if sudo test -e "$ROOTFS_WORK/usr/sbin/policy-rc.d"; then
  echo "Errore: policy-rc.d esiste già nella rootfs." >&2
  exit 1
fi

sudo install -D -m 0755 \
  "$POLICY_TMP" "$ROOTFS_WORK/usr/sbin/policy-rc.d"
POLICY_INSTALLED=1

echo
echo "===== Installazione manifesto AureaOS ====="
run_in_rootfs env DEBIAN_FRONTEND=noninteractive \
  apt-get install --yes --no-install-recommends -- "${packages[@]}"

# Remove the temporary service-start policy before final validation.
sudo rm -f -- "$ROOTFS_WORK/usr/sbin/policy-rc.d"
POLICY_INSTALLED=0

echo
echo "===== Verifica finale ====="

if ! sudo test -f "$ROOTFS_WORK/etc/os-release"; then
  echo "Errore: manca etc/os-release nella rootfs." >&2
  exit 1
fi

if ! sudo grep -q '^VERSION_CODENAME=trixie$' "$ROOTFS_WORK/etc/os-release"; then
  echo "Errore: codename della rootfs diverso da trixie." >&2
  exit 1
fi

if ! run_in_rootfs dpkg --print-architecture | grep -qx "$ARCH"; then
  echo "Errore: architettura della rootfs non corretta." >&2
  exit 1
fi

if ! sudo grep -q \
  '^Components: main contrib non-free non-free-firmware$' \
  "$ROOTFS_WORK/etc/apt/sources.list.d/debian.sources"; then
  echo "Errore: componenti APT attesi non configurati." >&2
  exit 1
fi

# Build dpkg-query's literal format token without asking the shell to expand it.
printf -v DPkg_QUERY_STATUS_FORMAT '%b' '\044{db:Status-Status}'

for required_package in systemd systemd-sysv linux-image-amd64; do
  if ! package_status="$(
    run_in_rootfs dpkg-query -W -f="$DPkg_QUERY_STATUS_FORMAT" "$required_package" 2>/dev/null
  )"; then
    echo "Errore: impossibile verificare il pacchetto $required_package." >&2
    exit 1
  fi

  if [[ "$package_status" != "installed" ]]; then
    echo "Errore: pacchetto fondamentale non installato: $required_package (stato: $package_status)." >&2
    exit 1
  fi
done

# Audit real ownership and mode bits after package installation. Run as root
# because the rootfs /root directory is intentionally not readable by users.
sudo bash "$REPO_ROOT/tools/security/audit-rootfs-permissions.sh" "$ROOTFS_WORK"

# Publish only after all checks pass. The current working directory pins the
# destination parent inode; -T prevents directory nesting and -n refuses
# replacement if a destination appears after the initial validation.
if [[ "$(stat -c '%d:%i' -- "$TARGET_PARENT")" != "$PARENT_ID" ]]; then
  echo "Errore: la directory destinazione è stata sostituita durante il bootstrap." >&2
  echo "Rootfs verificata conservata in staging: $ROOTFS_WORK" >&2
  exit 1
fi
if [[ -e "./$TARGET_NAME" || -L "./$TARGET_NAME" ]]; then
  echo "Errore: la destinazione è comparsa durante il bootstrap: $TARGET" >&2
  echo "Rootfs verificata conservata in staging: $ROOTFS_WORK" >&2
  exit 1
fi

sudo mv --no-clobber --no-target-directory -- "$ROOTFS_WORK" "./$TARGET_NAME"
if [[ -e "$ROOTFS_WORK" || -L "$ROOTFS_WORK" ]]; then
  echo "Errore: pubblicazione non eseguita; la destinazione potrebbe essere occupata." >&2
  echo "Rootfs verificata conservata in staging: $ROOTFS_WORK" >&2
  exit 1
fi
if [[ ! -d "./$TARGET_NAME" || -L "./$TARGET_NAME" ]]; then
  echo "Errore: la rootfs pubblicata non è una directory reale." >&2
  exit 1
fi
ROOTFS_WORK=""

echo "Rootfs Debian ${CODENAME} (${ARCH}) creata, configurata e verificata."
echo "Percorso: ${TARGET}"
echo "Nota: non è ancora un file avviabile o installabile."
