#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS
# Copyright (C) 2026 Alessandro Ceccarelli
#
# Render the approved desktop Trash SVG masters into deterministic PNG sizes.
# This is a developer asset tool; resvg is never a runtime dependency.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
readonly SOURCE_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/source/system/trash"
readonly OUTPUT_ROOT="${REPO_ROOT}/aurea-human-interfaces/icons/generated/trash"
readonly REQUIRED_RESVG_VERSION="0.45.1"
readonly SIZES=(16 24 32 48 64 96 128 256)
readonly STATES=(empty half full)
readonly THEMES=(light dark)

RESVG_BIN="${RESVG_BIN:-resvg}"

if ! command -v "${RESVG_BIN}" >/dev/null 2>&1; then
    printf 'AureaOS icon export requires resvg %s.\n' "${REQUIRED_RESVG_VERSION}" >&2
    printf 'Debian 13: sudo apt install resvg\n' >&2
    printf 'Fedora: cargo install resvg --version %s\n' "${REQUIRED_RESVG_VERSION}" >&2
    exit 1
fi

actual_version="$("${RESVG_BIN}" --version 2>/dev/null | tail -n 1)"
if [[ "${actual_version}" != "${REQUIRED_RESVG_VERSION}" ]]; then
    printf 'Unsupported resvg version: expected %s, found %s.\n' \
        "${REQUIRED_RESVG_VERSION}" "${actual_version:-unknown}" >&2
    exit 1
fi

mkdir -p -- "${OUTPUT_ROOT}"

for theme in "${THEMES[@]}"; do
    for state in "${STATES[@]}"; do
        source="${SOURCE_ROOT}/${theme}/trash-${state}.svg"
        if [[ ! -s "${source}" ]]; then
            printf 'Missing Trash SVG source: %s\n' "${source}" >&2
            exit 1
        fi

        for size in "${SIZES[@]}"; do
            output="${OUTPUT_ROOT}/trash-${theme}-${state}-${size}.png"
            "${RESVG_BIN}" --quiet --width "${size}" --height "${size}" \
                "${source}" "${output}"

            if [[ ! -s "${output}" ]]; then
                printf 'Renderer produced an empty asset: %s\n' "${output}" >&2
                exit 1
            fi

            signature="$(od -An -tx1 -N8 "${output}" | tr -d ' \n')"
            if [[ "${signature}" != "89504e470d0a1a0a" ]]; then
                printf 'Renderer output is not a PNG: %s\n' "${output}" >&2
                exit 1
            fi
        done
    done
done

printf 'Generated %s Trash PNG assets in %s\n' \
    "$(( ${#THEMES[@]} * ${#STATES[@]} * ${#SIZES[@]} ))" "${OUTPUT_ROOT}"
