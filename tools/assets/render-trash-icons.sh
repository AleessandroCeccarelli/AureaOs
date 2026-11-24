#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS
# Copyright (C) 2026 Alessandro Ceccarelli
#
# Generate static RGBA8 desktop Trash assets from the approved SVG masters.
# SVG rasterization and PNG decoding run on the developer host only.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
readonly SOURCE_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/source/system/trash"
readonly OUTPUT_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/generated/trash"
readonly SIZES=(64 96 128 192)
readonly STATES=(empty half full)
readonly THEMES=(light dark)

RENDERER="${RSVG_CONVERT:-rsvg-convert}"
PYTHON="${PYTHON:-python3}"

if ! command -v "${RENDERER}" >/dev/null 2>&1; then
    printf 'AureaOS icon export requires rsvg-convert.\n' >&2
    printf 'Debian 13: sudo apt install librsvg2-bin\n' >&2
    printf 'Fedora: sudo dnf install librsvg2-tools\n' >&2
    exit 1
fi
if ! command -v "${PYTHON}" >/dev/null 2>&1; then
    printf 'AureaOS icon export requires Python 3.\n' >&2
    exit 1
fi

renderer_version="$("${RENDERER}" --version 2>&1 | head -n 1)"
if [[ -z "${renderer_version}" ]]; then
    printf 'Unable to determine rsvg-convert version.\n' >&2
    exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf -- "${temporary_dir}"' EXIT
mkdir -p -- "${OUTPUT_ROOT}"

manifest="${OUTPUT_ROOT}/EXPORT-MANIFEST.txt"
{
    printf 'AureaOS desktop Trash RGBA8 assets\n'
    printf 'Renderer: %s\n' "${renderer_version}"
    printf 'Decoder: Python standard library (zlib)\n'
    printf 'Sizes: %s\n' "${SIZES[*]}"
    printf 'Source: aurea-human-interfaces/icons/source/system/trash\n'
    printf 'Generated: trash-{light,dark}-{empty,half,full}-{size}.rgba\n'
    printf 'Format: tightly packed RGBA8, row-major, no row padding\n'
} > "${manifest}"

for theme in "${THEMES[@]}"; do
    for state in "${STATES[@]}"; do
        source="${SOURCE_ROOT}/${theme}/trash-${state}.svg"
        if [[ ! -s "${source}" ]]; then
            printf 'Missing Trash SVG source: %s\n' "${source}" >&2
            exit 1
        fi

        for size in "${SIZES[@]}"; do
            temporary_png="${temporary_dir}/trash-${theme}-${state}-${size}.png"
            output="${OUTPUT_ROOT}/trash-${theme}-${state}-${size}.rgba"
            "${RENDERER}" --format=png --width="${size}" --height="${size}" \
                "${source}" --output="${temporary_png}"

            "${PYTHON}" "${SCRIPT_DIR}/png_to_rgba.py" "${temporary_png}" "${output}"
            expected_bytes=$(( size * size * 4 ))
            actual_bytes="$(stat -c '%s' "${output}")"
            if [[ "${actual_bytes}" -ne "${expected_bytes}" ]]; then
                printf 'Unexpected RGBA byte length for %s: %s (expected %s)\n' \
                    "${output}" "${actual_bytes}" "${expected_bytes}" >&2
                exit 1
            fi
        done
    done
done

printf 'Generated %s RGBA8 Trash assets with %s\n' \
    "$(( ${#THEMES[@]} * ${#STATES[@]} * ${#SIZES[@]} ))" "${renderer_version}"
