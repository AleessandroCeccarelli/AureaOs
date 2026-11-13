#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

SCRIPT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd -P)/tools/installer/check-hardware-requirements.sh"
TMP="$(mktemp -d)"
trap 'rm -rf -- "$TMP"' EXIT

mkdir -p "$TMP/bin"
cat > "$TMP/bin/vulkaninfo" <<'VULKAN'
#!/usr/bin/env bash
cat <<'SUMMARY'
Devices:
========
GPU0:
    apiVersion = 1.3.280
    deviceType = PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU
    deviceName = Intel Test GPU
GPU1:
    apiVersion = 1.4.300
    deviceType = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU
    deviceName = AMD Test GPU
SUMMARY
VULKAN
chmod +x "$TMP/bin/vulkaninfo"

flags='cx16 lahf_lm popcnt sse3 ssse3 sse4_1 sse4_2 avx avx2 bmi1 bmi2 f16c fma lzcnt movbe xsave'
printf 'processor : 0\nflags : %s\n' "$flags" > "$TMP/cpuinfo"
printf 'MemTotal: 16777216 kB\n' > "$TMP/meminfo"

if ! PATH="$TMP/bin:$PATH" AUREAOS_CPUINFO_PATH="$TMP/cpuinfo" AUREAOS_MEMINFO_PATH="$TMP/meminfo" "$SCRIPT"; then
  echo "Expected a compatible Intel/AMD-neutral host to pass." >&2
  exit 1
fi

printf 'processor : 0\nflags : sse2\n' > "$TMP/cpuinfo"
if PATH="$TMP/bin:$PATH" AUREAOS_CPUINFO_PATH="$TMP/cpuinfo" AUREAOS_MEMINFO_PATH="$TMP/meminfo" "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected a CPU below x86-64-v3 to fail." >&2
  exit 1
fi

flags='cx16 lahf_lm popcnt sse3 ssse3 sse4_1 sse4_2 avx avx2 bmi1 bmi2 f16c fma lzcnt movbe xsave'
printf 'processor : 0\nflags : %s\n' "$flags" > "$TMP/cpuinfo"
printf 'MemTotal: 4194304 kB\n' > "$TMP/meminfo"
if PATH="$TMP/bin:$PATH" AUREAOS_CPUINFO_PATH="$TMP/cpuinfo" AUREAOS_MEMINFO_PATH="$TMP/meminfo" "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected a host below 8 GiB RAM to fail." >&2
  exit 1
fi

cat > "$TMP/bin/vulkaninfo" <<'VULKAN'
#!/usr/bin/env bash
cat <<'SUMMARY'
Devices:
========
GPU0:
    apiVersion = 1.3.280
    deviceType = PHYSICAL_DEVICE_TYPE_CPU
    deviceName = llvmpipe
SUMMARY
VULKAN
chmod +x "$TMP/bin/vulkaninfo"
printf 'MemTotal: 16777216 kB\n' > "$TMP/meminfo"
if PATH="$TMP/bin:$PATH" AUREAOS_CPUINFO_PATH="$TMP/cpuinfo" AUREAOS_MEMINFO_PATH="$TMP/meminfo" "$SCRIPT" >/dev/null 2>&1; then
  echo "Expected a software-only Vulkan implementation to fail." >&2
  exit 1
fi

echo "Hardware preflight tests passed."
