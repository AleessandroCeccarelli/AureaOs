
#!/usr/bin/env bash
set -Eeuo pipefail

# AureaOS Roma 1.0 — Debian bootstrap
# Creates a minimal Debian root filesystem outside the repository.

readonly CODENAME="trixie"
readonly ARCH="amd64"
readonly VARIANT="minbase"
readonly COMPONENTS="main"
readonly MIRROR="https://deb.debian.org/debian"

readonly SCRIPT_DIR="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P
)"
readonly REPO_ROOT="$(
  cd -- "$SCRIPT_DIR/../.." && pwd -P
)"

TARGET="${HOME}/Documenti/AureaOS-Build/rootfs"

usage() {
  cat <<'EOF'
AureaOS Debian bootstrap

Usage:
  ./tools/build/bootstrap-debian.sh [--target PATH]
  ./tools/build/bootstrap-debian.sh --help

Options:
  --target PATH  Destination for the new root filesystem.
                 Default: ~/Documenti/AureaOS-Build/rootfs
  --help         Show this help and exit.

Safety:
  - Existing destinations are never overwritten or deleted.
  - The destination must be outside the Git repository.
  - Root privileges are requested through sudo when needed.
EOF
}

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

if [[ "$TARGET" != /* ]]; then
  echo "Errore: usa un percorso assoluto per --target." >&2
  exit 2
fi

if [[ "$TARGET" == "/" ]]; then
  echo "Errore: la radice del filesystem non è una destinazione valida." >&2
  exit 2
fi

if ! command -v debootstrap >/dev/null 2>&1; then
  echo "Errore: debootstrap non è installato." >&2
  echo "Su Fedora puoi installarlo con: sudo dnf install debootstrap" >&2
  exit 1
fi

# Resolve the parent even when the destination does not exist yet.
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
  echo "Errore: la destinazione esiste già:" >&2
  echo "  $TARGET" >&2
  echo "Nessun file è stato modificato o eliminato." >&2
  echo "Scegli una destinazione nuova con --target." >&2
  exit 1
fi

echo "===== AureaOS Debian Bootstrap ====="
echo "Distribuzione: Debian GNU/Linux 13 (${CODENAME})"
echo "Architettura:  ${ARCH}"
echo "Profilo:       ${VARIANT}"
echo "Componenti:    ${COMPONENTS}"
echo "Mirror:        ${MIRROR}"
echo "Destinazione:  ${TARGET}"
echo

echo "Verrà creata una nuova rootfs minimale."
echo "Non verrà creato un sistema avviabile o un'immagine ISO."
echo

sudo -v

sudo debootstrap \
  --arch="$ARCH" \
  --variant="$VARIANT" \
  --components="$COMPONENTS" \
  "$CODENAME" \
  "$TARGET" \
  "$MIRROR"

echo
echo "===== Verifica finale ====="

if [[ ! -f "$TARGET/etc/os-release" ]]; then
  echo "Errore: manca etc/os-release nella rootfs." >&2
  echo "La costruzione potrebbe essere incompleta: controlla i log." >&2
  exit 1
fi

if ! grep -q '^VERSION_CODENAME=trixie$' "$TARGET/etc/os-release"; then
  echo "Errore: il codename della rootfs non corrisponde a trixie." >&2
  exit 1
fi

if ! sudo chroot "$TARGET" dpkg --print-architecture | grep -qx "$ARCH"; then
  echo "Errore: architettura della rootfs non corretta." >&2
  exit 1
fi

echo "Rootfs Debian ${CODENAME} (${ARCH}) creata e verificata."
echo "Percorso: ${TARGET}"
