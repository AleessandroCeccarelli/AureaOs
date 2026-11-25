#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS
# Copyright (C) 2026 Alessandro Ceccarelli
#
# Export the canonical context-menu SVG masters to bounded RGBA8 assets.
# Noto Sans is resolved by the host font system during rasterization only.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
readonly SOURCE_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/source/system/context-menu"
readonly OUTPUT_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/generated/context-menu"
readonly SIZE=256

RENDERER="${RSVG_CONVERT:-rsvg-convert}"
PYTHON="${PYTHON:-python3}"

if ! command -v "${RENDERER}" >/dev/null 2>&1; then
    printf 'Context-menu export requires rsvg-convert.\n' >&2
    printf 'Debian 13: sudo apt install librsvg2-bin fonts-noto-core\n' >&2
    printf 'Fedora: sudo dnf install librsvg2-tools google-noto-sans-fonts\n' >&2
    exit 1
fi
if ! command -v "${PYTHON}" >/dev/null 2>&1; then
    printf 'Context-menu export requires Python 3.\n' >&2
    exit 1
fi

renderer_version="$("${RENDERER}" --version 2>&1 | head -n 1)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf -- "${temporary_dir}"' EXIT
mkdir -p -- "${OUTPUT_ROOT}"

manifest="${OUTPUT_ROOT}/EXPORT-MANIFEST.txt"
{
    printf 'AureaOS desktop context-menu RGBA8 assets\n'
    printf 'Renderer: %s\n' "${renderer_version}"
    printf 'Font family: Noto Sans (Google Fonts / SIL OFL 1.1)\n'
    printf 'Font role: build-time rasterization only; no runtime font dependency\n'
    printf 'Dimensions: %s x %s\n' "${SIZE}" "${SIZE}"
    printf 'Format: tightly packed RGBA8, row-major, no row padding\n'
} > "${manifest}"

for theme in light dark; do
    source="${SOURCE_ROOT}/${theme}.svg"
    output="${OUTPUT_ROOT}/context-menu-${theme}-${SIZE}.rgba"
    temporary_png="${temporary_dir}/context-menu-${theme}.png"

    if [[ ! -s "${source}" ]]; then
        printf 'Missing context-menu SVG source: %s\n' "${source}" >&2
        exit 1
    fi

    "${RENDERER}" --format=png --width="${SIZE}" --height="${SIZE}" \
        "${source}" --output="${temporary_png}"
    "${PYTHON}" "${SCRIPT_DIR}/png_to_rgba.py" "${temporary_png}" "${output}"

    expected_bytes=$(( SIZE * SIZE * 4 ))
    actual_bytes="$(stat -c '%s' "${output}")"
    if [[ "${actual_bytes}" -ne "${expected_bytes}" ]]; then
        printf 'Unexpected RGBA byte length for %s: %s (expected %s)\n' \
            "${output}" "${actual_bytes}" "${expected_bytes}" >&2
        exit 1
    fi
done

printf 'Generated light and dark context-menu RGBA8 assets with %s\n' "${renderer_version}"
