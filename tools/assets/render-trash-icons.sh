#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS
# Copyright (C) 2026 Alessandro Ceccarelli
#
# Render approved desktop Trash SVG masters using the existing asset renderer.
# rsvg-convert is a developer tool; librsvg is never an AureaOS runtime dependency.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
readonly SOURCE_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/source/system/trash"
readonly OUTPUT_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/generated/trash"
readonly SIZES=(16 24 32 48 64 96 128 256)
readonly STATES=(empty half full)
readonly THEMES=(light dark)

RENDERER="${RSVG_CONVERT:-rsvg-convert}"

if ! command -v "${RENDERER}" >/dev/null 2>&1; then
    printf 'AureaOS icon export requires rsvg-convert.\n' >&2
    printf 'Debian 13: sudo apt install librsvg2-bin\n' >&2
    printf 'Fedora: sudo dnf install librsvg2-tools\n' >&2
    exit 1
fi

renderer_version="$("${RENDERER}" --version 2>&1 | head -n 1)"
if [[ -z "${renderer_version}" ]]; then
    printf 'Unable to determine rsvg-convert version.\n' >&2
    exit 1
fi

mkdir -p -- "${OUTPUT_ROOT}"
manifest="${OUTPUT_ROOT}/EXPORT-MANIFEST.txt"
{
    printf 'AureaOS desktop Trash raster exports\n'
    printf 'Renderer: %s\n' "${renderer_version}"
    printf 'Sizes: %s\n' "${SIZES[*]}"
    printf 'Source: aurea-human-interfaces/icons/source/system/trash\n'
    printf 'Generated: trash-{light,dark}-{empty,half,full}-{size}.png\n'
} > "${manifest}"

for theme in "${THEMES[@]}"; do
    for state in "${STATES[@]}"; do
        source="${SOURCE_ROOT}/${theme}/trash-${state}.svg"
        if [[ ! -s "${source}" ]]; then
            printf 'Missing Trash SVG source: %s\n' "${source}" >&2
            exit 1
        fi

        for size in "${SIZES[@]}"; do
            output="${OUTPUT_ROOT}/trash-${theme}-${state}-${size}.png"
            "${RENDERER}" --format=png --width="${size}" --height="${size}" \
                "${source}" --output="${output}"

            if [[ ! -s "${output}" ]]; then
                printf 'Renderer produced an empty asset: %s\n' "${output}" >&2
                exit 1
            fi

            signature="$(od -An -tx1 -N8 "${output}" | tr -d ' \n')"
            if [[ "${signature}" != "89504e470d0a1a0a" ]]; then
                printf 'Renderer output is not a PNG: %s\n' "${output}" >&2
                exit 1
            fi

            read -r w1 w2 w3 w4 h1 h2 h3 h4 bit_depth color_type _compression _filter _interlace \
                < <(od -An -tu1 -j16 -N13 "${output}")
            actual_width=$(( (w1 << 24) | (w2 << 16) | (w3 << 8) | w4 ))
            actual_height=$(( (h1 << 24) | (h2 << 16) | (h3 << 8) | h4 ))
            if [[ "${actual_width}" -ne "${size}" || "${actual_height}" -ne "${size}" ]]; then
                printf 'Unexpected PNG dimensions for %s: %sx%s (expected %sx%s)\n' \
                    "${output}" "${actual_width}" "${actual_height}" "${size}" "${size}" >&2
                exit 1
            fi
            if [[ "${bit_depth}" -ne 8 || "${color_type}" -ne 6 ]]; then
                printf 'Expected 8-bit RGBA PNG for %s (depth=%s, color type=%s)\n' \
                    "${output}" "${bit_depth}" "${color_type}" >&2
                exit 1
            fi
        done
    done
done

printf 'Generated %s Trash PNG assets with %s\n' \
    "$(( ${#THEMES[@]} * ${#STATES[@]} * ${#SIZES[@]} ))" "${renderer_version}"
