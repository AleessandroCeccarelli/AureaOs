#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
SOURCE_DIR="${REPO_ROOT}/components/aurea-compositor/shaders"
OUTPUT_DIR="${1:-${REPO_ROOT}/target/aurea-shaders}"

if ! command -v glslangValidator >/dev/null 2>&1; then
    printf '%s\n' "glslangValidator is required. On Fedora install it with:" "  sudo dnf install glslang"
    exit 1
fi

mkdir -p "${OUTPUT_DIR}"

glslangValidator -V --target-env vulkan1.3 \
    -S vert "${SOURCE_DIR}/triangle.vert" \
    -o "${OUTPUT_DIR}/triangle.vert.spv"

glslangValidator -V --target-env vulkan1.3 \
    -S frag "${SOURCE_DIR}/triangle.frag" \
    -o "${OUTPUT_DIR}/triangle.frag.spv"

printf 'Compiled Vulkan shaders into %s\n' "${OUTPUT_DIR}"
