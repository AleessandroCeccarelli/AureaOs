#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS
# Copyright (C) 2026 Alessandro Ceccarelli
#
# Export the canonical context-menu SVG masters to bounded RGBA8 assets.
# Typography is resolved from the Noto Sans font retained in Human Interfaces.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
readonly SOURCE_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/source/system/context-menu"
readonly OUTPUT_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/generated/context-menu"
readonly FONT_ROOT="${REPO_ROOT}/aurea-human-interfaces/fonts/source/google-fonts/noto-sans"
readonly FONT_FILE="${FONT_ROOT}/NotoSans-Variable.ttf"
readonly SIZE=256

RENDERER="${RSVG_CONVERT:-rsvg-convert}"
PYTHON="${PYTHON:-python3}"

if ! command -v "${RENDERER}" >/dev/null 2>&1; then
    printf 'Context-menu export requires rsvg-convert.\n' >&2
    printf 'Debian 13: sudo apt install librsvg2-bin\n' >&2
    printf 'Fedora: sudo dnf install librsvg2-tools\n' >&2
    exit 1
fi
if ! command -v "${PYTHON}" >/dev/null 2>&1; then
    printf 'Context-menu export requires Python 3.\n' >&2
    exit 1
fi
if [[ ! -s "${FONT_FILE}" ]]; then
    printf 'Bundled Noto Sans font is missing: %s\n' "${FONT_FILE}" >&2
    exit 1
fi

renderer_version="$("${RENDERER}" --version 2>&1 | head -n 1)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf -- "${temporary_dir}"' EXIT
mkdir -p -- "${OUTPUT_ROOT}" "${temporary_dir}/font-cache"

# Give fontconfig the repository-owned font without installing or copying it
# into the host. Keep the host's normal font configuration as a fallback.
fontconfig_file="${temporary_dir}/fonts.conf"
cat > "${fontconfig_file}" <<EOF
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
  <dir>${FONT_ROOT}</dir>
  <cachedir>${temporary_dir}/font-cache</cachedir>
</fontconfig>
EOF

export FONTCONFIG_FILE="${fontconfig_file}"

manifest="${OUTPUT_ROOT}/EXPORT-MANIFEST.txt"
{
    printf 'AureaOS desktop context-menu RGBA8 assets\n'
    printf 'Renderer: %s\n' "${renderer_version}"
    printf 'Font family: bundled Noto Sans Variable (Google Fonts / SIL OFL 1.1)\n'
    printf 'Font source: aurea-human-interfaces/fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf\n'
    printf 'Font role: host-side SVG rasterization only; no runtime font dependency\n'
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
