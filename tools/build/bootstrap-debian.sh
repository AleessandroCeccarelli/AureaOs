
#!/usr/bin/env bash
set -Eeuo pipefail

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

readonly SCRIPT_DIR="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P
)"
readonly REPO_ROOT="$(
  cd -- "$SCRIPT_DIR/../.." && pwd -P
)"
readonly PACKAGE_DIR="$REPO_ROOT/debian/packages"

TARGET="${HOME}/Documenti/AureaOS-Build/rootfs"
BOOT_MODE="none"
WITH_RAR=0

SOURCES_TMP=""
POLICY_TMP=""
POLICY_INSTALLED=0
MOUNTS_ACTIVE=0

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

  if [[ -n "$TARGET" && -d "$TARGET" ]]; then
    if ((POLICY_INSTALLED)); then
      echo "Pulizia: rimozione di policy-rc.d..." >&2
      sudo rm -f -- "$TARGET/usr/sbin/policy-rc.d"
    fi

    if ((MOUNTS_ACTIVE)); then
      echo "Pulizia: smontaggio dei filesystem del chroot..." >&2

      # Unmount nested mounts before their parent mounts.
      sudo umount -R "$TARGET/sys" 2>/dev/null
      sudo umount "$TARGET/proc" 2>/dev/null
      sudo umount -R "$TARGET/dev" 2>/dev/null
    fi
  fi

  [[ -z "$SOURCES_TMP" ]] || rm -f -- "$SOURCES_TMP"
  [[ -z "$POLICY_TMP" ]] || rm -f -- "$POLICY_TMP"

  if ((status != 0)); then
    echo "Bootstrap interrotto con errore (codice $status)." >&2
    echo "La destinazione potrebbe contenere una rootfs incompleta:" >&2
    echo "$TARGET" >&2
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

read_manifest() {
  local file="$1"

  if [[ ! -f "$file" ]]; then
    echo "Errore: manifesto non trovato: $file" >&2
    return 1
  fi

  awk '{ sub(/#.*/, ""); for (i = 1; i <= NF; i++) print $i }' "$file"
}

packages=()

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
  while IFS= read -r package; do
    packages+=("$package")
  done < <(read_manifest "$PACKAGE_DIR/$manifest")
done

case "$BOOT_MODE" in
  bios)
    while IFS= read -r package; do
      packages+=("$package")
    done < <(read_manifest "$PACKAGE_DIR/boot-bios.list")
    ;;
  uefi)
    while IFS= read -r package; do
      packages+=("$package")
    done < <(read_manifest "$PACKAGE_DIR/boot-uefi.list")
    ;;
esac

if ((WITH_RAR)); then
  while IFS= read -r package; do
    packages+=("$package")
  done < <(read_manifest "$PACKAGE_DIR/rar-extraction-optional.list")
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

# Create the base Debian filesystem.
sudo debootstrap \
  --arch="$ARCH" \
  --variant="$VARIANT" \
  --components="$COMPONENTS" \
  "$CODENAME" \
  "$TARGET" \
  "$MIRROR"

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
  "$SOURCES_TMP" "$TARGET/etc/apt/sources.list.d/debian.sources"

# Keep a backup of the legacy sources file if one exists.
if sudo test -f "$TARGET/etc/apt/sources.list"; then
  if sudo test -e "$TARGET/etc/apt/sources.list.bootstrap-backup"; then
    echo "Errore: esiste già sources.list.bootstrap-backup." >&2
    exit 1
  fi

  sudo mv \
    "$TARGET/etc/apt/sources.list" \
    "$TARGET/etc/apt/sources.list.bootstrap-backup"
fi

# Mount the virtual filesystems required by package maintainer scripts,
# systemd-related package configuration and initramfs generation.
echo
echo "===== Preparazione filesystem del chroot ====="

for mountpoint in dev proc sys; do
  if sudo mountpoint -q "$TARGET/$mountpoint"; then
    echo "Errore: $TARGET/$mountpoint è già un mountpoint." >&2
    exit 1
  fi
done

sudo mount --rbind /dev "$TARGET/dev"
MOUNTS_ACTIVE=1
sudo mount --make-rslave "$TARGET/dev"

sudo mount -t proc proc "$TARGET/proc"

sudo mount --rbind /sys "$TARGET/sys"
sudo mount --make-rslave "$TARGET/sys"

echo "Filesystem /dev, /proc e /sys montati."

# Use deb822 sources to refresh package metadata.
echo
echo "===== Aggiornamento cataloghi APT ====="
sudo chroot "$TARGET" apt-get update

# Suppress automatic service starts inside the build chroot.
cat > "$POLICY_TMP" <<'POLICY_RC'
#!/bin/sh
exit 101
POLICY_RC

if sudo test -e "$TARGET/usr/sbin/policy-rc.d"; then
  echo "Errore: policy-rc.d esiste già nella rootfs." >&2
  exit 1
fi

sudo install -D -m 0755 \
  "$POLICY_TMP" "$TARGET/usr/sbin/policy-rc.d"
POLICY_INSTALLED=1

echo
echo "===== Installazione manifesto AureaOS ====="
sudo chroot "$TARGET" env DEBIAN_FRONTEND=noninteractive \
  apt-get install --yes --no-install-recommends "${packages[@]}"

# Remove the temporary service-start policy before final validation.
sudo rm -f -- "$TARGET/usr/sbin/policy-rc.d"
POLICY_INSTALLED=0

echo
echo "===== Verifica finale ====="

if ! sudo test -f "$TARGET/etc/os-release"; then
  echo "Errore: manca etc/os-release nella rootfs." >&2
  exit 1
fi

if ! sudo grep -q '^VERSION_CODENAME=trixie$' "$TARGET/etc/os-release"; then
  echo "Errore: codename della rootfs diverso da trixie." >&2
  exit 1
fi

if ! sudo chroot "$TARGET" dpkg --print-architecture | grep -qx "$ARCH"; then
  echo "Errore: architettura della rootfs non corretta." >&2
  exit 1
fi

if ! sudo grep -q \
  '^Components: main contrib non-free non-free-firmware$' \
  "$TARGET/etc/apt/sources.list.d/debian.sources"; then
  echo "Errore: componenti APT attesi non configurati." >&2
  exit 1
fi

if ! sudo chroot "$TARGET" dpkg-query -W \
  -f='${Status}\n' systemd linux-image-amd64 |
  grep -q '^install ok installed$'; then
  echo "Errore: verifica dei pacchetti fondamentali non riuscita." >&2
  exit 1
fi

echo "Rootfs Debian ${CODENAME} (${ARCH}) creata, configurata e verificata."
echo "Percorso: ${TARGET}"
echo "Nota: non è ancora un'immagine avviabile o installabile."

