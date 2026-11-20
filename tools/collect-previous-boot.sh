#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS — collect diagnostics from the previous boot
# Copyright (C) 2026 Alessandro Ceccarelli

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS_DIR="$ROOT_DIR/diagnostics/runs"
RUN_ID="${1:-}"

if [[ -z "$RUN_ID" ]]; then
    printf 'Usage: %s <run-id>\n' "$0" >&2
    printf 'Available runs:\n' >&2
    if [[ -d "$RUNS_DIR" ]]; then
        find "$RUNS_DIR" -mindepth 1 -maxdepth 1 -type d -printf '  %f\n' | sort >&2
    fi
    exit 2
fi

if ! [[ "$RUN_ID" =~ ^[0-9]{8}-[0-9]{6}$ ]]; then
    printf 'Invalid run id: %s\n' "$RUN_ID" >&2
    exit 2
fi

RUN_DIR="$RUNS_DIR/$RUN_ID"
if [[ ! -d "$RUN_DIR" ]]; then
    printf 'Diagnostic run directory not found: %s\n' "$RUN_DIR" >&2
    exit 2
fi

for command_name in journalctl date; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "$command_name" >&2
        exit 2
    fi
done

printf 'Collecting previous-boot diagnostics into %s\n' "$RUN_DIR"
printf 'This reads the previous boot only; it does not change system configuration.\n'

collect_journal() {
    local output_file="$1"
    shift
    if journalctl "$@" --no-pager --output=short-precise > "$output_file" 2>&1; then
        printf 'Collected %s\n' "$output_file"
    else
        printf 'Journal collection returned a non-zero status for %s.\n' "$output_file" >&2
        printf 'If access was denied, retry with sudo.\n' >&2
    fi
}

collect_journal "$RUN_DIR/previous-boot-kernel.txt" -b -1 -k
collect_journal "$RUN_DIR/previous-boot-system.txt" -b -1

{
    printf 'collected_at=%s\n' "$(date --iso-8601=seconds)"
    printf 'previous_boot_journal_collected=yes\n'
} >> "$RUN_DIR/run-info.txt"

printf '\nCollection finished. Review the files before committing: they may contain machine-specific details.\n'
printf 'Directory: %s\n' "$RUN_DIR"
