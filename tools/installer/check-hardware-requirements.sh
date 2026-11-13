#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

# AureaOS installation preflight. This checks host CPU/RAM and the presence
# of at least one hardware Vulkan 1.3 GPU. It does not certify scanout.
readonly MIN_MEMORY_KIB=$((8 * 1024 * 1024))
CPUINFO_PATH="${AUREAOS_CPUINFO_PATH:-/proc/cpuinfo}"
MEMINFO_PATH="${AUREAOS_MEMINFO_PATH:-/proc/meminfo}"
VULKANINFO_BIN="${AUREAOS_VULKANINFO_BIN:-vulkaninfo}"

failures=0

fail() {
  printf 'ERRORE: %s\n' "$*" >&2
  failures=$((failures + 1))
}

printf '%s\n' '===== Verifica requisiti hardware AureaOS ====='

if [[ "$(uname -m)" != "x86_64" ]]; then
  fail "AureaOS richiede architettura AMD64/x86-64; rilevata: $(uname -m)"
else
  echo "Architettura: AMD64"
fi

if [[ ! -r "$CPUINFO_PATH" ]]; then
  fail "Impossibile leggere le funzionalità CPU da $CPUINFO_PATH"
else
  # Linux exposes AVX only when the OS has enabled the required extended
  # processor state, so checking the kernel flags also covers OSXSAVE/XCR0.
  required_flags=(
    cx16 lahf_lm popcnt sse3 ssse3 sse4_1 sse4_2
    avx avx2 bmi1 bmi2 f16c fma lzcnt movbe xsave
  )
  cpu_flags="$(awk -F: '/^[[:space:]]*flags[[:space:]]*:/ { sub(/^[[:space:]]*/, "", $2); print $2; exit }' "$CPUINFO_PATH")"
  missing_flags=()
  for flag in "${required_flags[@]}"; do
    if [[ " $cpu_flags " != *" $flag "* ]]; then
      missing_flags+=("$flag")
    fi
  done

  if (("${#missing_flags[@]}" > 0)); then
    fail "CPU non compatibile con x86-64-v3; istruzioni mancanti: ${missing_flags[*]}"
  else
    echo "CPU: x86-64-v3 verificato (vendor Intel/AMD non vincolante)"
  fi
fi

if [[ ! -r "$MEMINFO_PATH" ]]; then
  fail "Impossibile leggere la memoria di sistema da $MEMINFO_PATH"
else
  total_kib="$(awk '/^MemTotal:/ { print $2; exit }' "$MEMINFO_PATH")"
  if [[ ! "$total_kib" =~ ^[0-9]+$ ]] || ((total_kib < MIN_MEMORY_KIB)); then
    fail "RAM insufficiente: richiesti almeno 8 GiB"
  else
    printf 'RAM: %.1f GiB\n' "$(awk -v kib="$total_kib" 'BEGIN { print kib / 1048576 }')"
  fi
fi

if ! command -v "$VULKANINFO_BIN" >/dev/null 2>&1; then
  fail "vulkaninfo non disponibile: installare vulkan-tools e i driver Vulkan della GPU"
else
  vulkan_summary="$( "$VULKANINFO_BIN" --summary 2>&1 )" || {
    fail "Vulkan non è inizializzabile: $vulkan_summary"
    vulkan_summary=""
  }

  if [[ -n "$vulkan_summary" ]]; then
    # Inspect each physical-device block independently. A CPU Vulkan
    # implementation (for example lavapipe) is not a qualifying display GPU.
    if printf '%s\n' "$vulkan_summary" | awk '
      /^GPU[0-9]+:/ {
        if (in_gpu && is_hardware_gpu && api_13) found = 1
        in_gpu = 1
        is_hardware_gpu = 0
        api_13 = 0
        next
      }
      /deviceType =/ && /INTEGRATED_GPU|DISCRETE_GPU/ { is_hardware_gpu = 1 }
      /apiVersion =/ {
        version = $3
        split(version, parts, ".")
        if (parts[1] > 1 || (parts[1] == 1 && parts[2] >= 3)) api_13 = 1
      }
      END {
        if (in_gpu && is_hardware_gpu && api_13) found = 1
        exit(found ? 0 : 1)
      }
    '; then
      echo "GPU: almeno una GPU integrata o dedicata espone Vulkan 1.3"
    else
      fail "Nessuna GPU hardware con Vulkan 1.3 rilevata (le GPU CPU/software non sono sufficienti)"
    fi
  fi
fi

if ((failures > 0)); then
  printf '\nInstallazione bloccata: %d requisito/i non soddisfatto/i.\n' "$failures" >&2
  exit 1
fi

cat <<'NOTICE'

Preflight superato.
NOTA: questa verifica non certifica ancora DMA-BUF, formati/modifier,
sincronizzazione o presentazione DRM/KMS. Questi requisiti devono essere
verificati dal test di compatibilità grafica e dal compositor sul dispositivo.
NOTICE
