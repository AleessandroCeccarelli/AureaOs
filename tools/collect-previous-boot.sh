#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS — diagnostic report collector
# Copyright (C) 2026 Alessandro Ceccarelli

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS_DIR="$ROOT_DIR/diagnostics/runs"
RUN_ID="${1:-}"
BOOT_SCOPE="${2:---previous-boot}"

if [[ -z "$RUN_ID" ]]; then
    printf 'Usage: bash %s <run-id> [--previous-boot|--current-boot]\n' "$0" >&2
    exit 2
fi

if ! [[ "$RUN_ID" =~ ^[0-9]{8}-[0-9]{6}$ ]]; then
    printf 'Invalid run id: %s\n' "$RUN_ID" >&2
    exit 2
fi

case "$BOOT_SCOPE" in
    --previous-boot) JOURNAL_BOOT="-1"; PREFIX="previous-boot" ;;
    --current-boot) JOURNAL_BOOT="0"; PREFIX="current-boot" ;;
    *) printf 'Unknown boot scope: %s\n' "$BOOT_SCOPE" >&2; exit 2 ;;
esac

RUN_DIR="$RUNS_DIR/$RUN_ID"
if [[ ! -d "$RUN_DIR" || ! -f "$RUN_DIR/run-info.txt" ]]; then
    printf 'Diagnostic run not found: %s\n' "$RUN_DIR" >&2
    exit 2
fi

for command_name in journalctl date awk chmod grep; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "$command_name" >&2
        exit 2
    fi
done

printf 'Collecting %s diagnostics for %s\n' "$BOOT_SCOPE" "$RUN_ID"

STARTED_EPOCH="$(awk -F= '$1 == "started_epoch" { print $2; exit }' "$RUN_DIR/run-info.txt" 2>/dev/null || true)"
ENDED_EPOCH="$(awk -F= '$1 == "finished_epoch" { value=$2 } $1 == "recovered_epoch" { value=$2 } END { print value }' "$RUN_DIR/run-info.txt" 2>/dev/null || true)"
if ! [[ "$ENDED_EPOCH" =~ ^[0-9]+$ ]]; then ENDED_EPOCH="$(date +%s)"; fi

JOURNAL_RANGE=()
if [[ "$STARTED_EPOCH" =~ ^[0-9]+$ ]]; then
    JOURNAL_RANGE+=(--since "@$STARTED_EPOCH" --until "@$ENDED_EPOCH")
fi

collect_journal() {
    local output_file="$1"
    shift

    if journalctl "$@" --no-pager --output=short-precise > "$output_file" 2>&1 &&
        [[ -s "$output_file" ]] &&
        ! grep -Eiq 'no journal files were opened|permission denied|insufficient permissions' "$output_file"; then
        chmod 600 "$output_file"
        printf 'Collected %s\n' "$output_file"
        return 0
    fi

    # Some systemd versions report denied journal access with exit status 0.
    if command -v sudo >/dev/null 2>&1 &&
        # The shell deliberately writes the report as the logged-in user.
        # shellcheck disable=SC2024
        sudo journalctl "$@" --no-pager --output=short-precise > "$output_file" 2>&1 &&
        [[ -s "$output_file" ]] &&
        ! grep -Eiq 'no journal files were opened|permission denied|insufficient permissions' "$output_file"; then
        chmod 600 "$output_file"
        printf 'Collected %s (with elevated journal access)\n' "$output_file"
        return 0
    fi

    printf 'WARNING: journal unavailable for %s.\n' "$output_file" >&2
    rm -f -- "$output_file"
    return 1
}

# Keep kernel diagnostics (including DRM/GPU driver faults) and Aurea Desktop
# process messages only. Do not export the machine-wide system journal.
collect_journal "$RUN_DIR/$PREFIX-kernel.txt" -b "$JOURNAL_BOOT" -k "${JOURNAL_RANGE[@]}" || true
collect_journal "$RUN_DIR/$PREFIX-aurea-desktop-journal.txt" -b "$JOURNAL_BOOT" _COMM=aurea-desktop "${JOURNAL_RANGE[@]}" || true

if command -v coredumpctl >/dev/null 2>&1 && [[ "${#JOURNAL_RANGE[@]}" -gt 0 ]]; then
    # Keep only Aurea Desktop crash metadata, never unrelated application crashes.
    if coredumpctl --no-pager list --since "@$STARTED_EPOCH" --until "@$ENDED_EPOCH" 2>/dev/null |
        grep -i 'aurea-desktop' > "$RUN_DIR/coredumps.txt" && [[ -s "$RUN_DIR/coredumps.txt" ]]; then
        chmod 600 "$RUN_DIR/coredumps.txt"
        printf 'Collected Aurea Desktop coredump index. Core files themselves are not copied.\n'
    else
        rm -f -- "$RUN_DIR/coredumps.txt"
    fi
fi
STATE_HOME="${XDG_STATE_HOME:-}"
if [[ -z "$STATE_HOME" && -n "${HOME:-}" ]]; then
    STATE_HOME="$HOME/.local/state"
fi

if [[ "$STARTED_EPOCH" =~ ^[0-9]+$ && -n "$STATE_HOME" ]]; then
    APP_LOG="$STATE_HOME/aureaos/logs/aurea-desktop.log"
    if [[ -f "$APP_LOG" ]]; then
        if awk -v started="$STARTED_EPOCH" -v ended="$ENDED_EPOCH" '
            $1 ~ /^[0-9]+$/ && $1 >= started && $1 <= ended { print }
        ' "$APP_LOG" > "$RUN_DIR/aurea-desktop-app.txt"; then
            chmod 600 "$RUN_DIR/aurea-desktop-app.txt"
            printf 'Collected application log entries from %s\n' "$APP_LOG"
        else
            rm -f -- "$RUN_DIR/aurea-desktop-app.txt"
            printf 'WARNING: application log filtering failed.\n' >&2
        fi
    else
        printf 'Persistent application log not found: %s\n' "$APP_LOG"
    fi
else
    printf 'Application log not collected: start timestamp or state directory unavailable.\n'
fi

{
    printf '%s_collected_at=%s\n' "${PREFIX//-/_}" "$(date --iso-8601=seconds)"
} >> "$RUN_DIR/run-info.txt"

printf 'Collection finished: %s\n' "$RUN_DIR"
