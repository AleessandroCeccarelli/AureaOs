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

for command_name in journalctl date awk; do
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
        return 0
    fi

    # Keep output files owned by the logged-in user: sudo applies only to
    # journalctl, while the shell performs the redirection.
    if command -v sudo >/dev/null 2>&1 &&
        sudo journalctl "$@" --no-pager --output=short-precise > "$output_file" 2>&1; then
        printf 'Collected %s (with elevated journal access)\n' "$output_file"
        return 0
    fi

    printf 'WARNING: unable to collect %s. Check journal permissions.\n' "$output_file" >&2
    return 1
}

collect_journal "$RUN_DIR/previous-boot-kernel.txt" -b -1 -k || true
collect_journal "$RUN_DIR/previous-boot-system.txt" -b -1 || true

STARTED_EPOCH="$(awk -F= '$1 == "started_epoch" { print $2; exit }' "$RUN_DIR/run-info.txt" 2>/dev/null || true)"
STATE_HOME="${XDG_STATE_HOME:-}"
if [[ -z "$STATE_HOME" && -n "${HOME:-}" ]]; then
    STATE_HOME="$HOME/.local/state"
fi

if [[ "$STARTED_EPOCH" =~ ^[0-9]+$ && -n "$STATE_HOME" ]]; then
    APP_LOG="$STATE_HOME/aureaos/logs/aurea-desktop.log"
    if [[ -f "$APP_LOG" ]]; then
        if awk -v started="$STARTED_EPOCH" '
            $1 ~ /^[0-9]+$/ && $1 >= started { print }
        ' "$APP_LOG" > "$RUN_DIR/aurea-desktop-app.txt"; then
            chmod 600 "$RUN_DIR/aurea-desktop-app.txt"
            printf 'Collected application diagnostics from %s\n' "$APP_LOG"
        else
            printf 'WARNING: unable to filter the persistent application log.\n' >&2
            rm -f -- "$RUN_DIR/aurea-desktop-app.txt"
        fi
    else
        printf 'Persistent application log not found: %s\n' "$APP_LOG"
    fi
else
    printf 'Application log not collected: run start time or state directory is unavailable.\n'
fi

{
    printf 'collected_at=%s\n' "$(date --iso-8601=seconds)"
    printf 'previous_boot_journal_collection_attempted=yes\n'
    printf 'application_log_collection_attempted=yes\n'
} >> "$RUN_DIR/run-info.txt"

printf '\nCollection finished. Review the files before committing: they may contain machine-specific details.\n'
printf 'Directory: %s\n' "$RUN_DIR"
