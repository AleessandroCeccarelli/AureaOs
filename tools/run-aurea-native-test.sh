#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS — native desktop launcher and diagnostic session coordinator
# Copyright (C) 2026 Alessandro Ceccarelli

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS_DIR="$ROOT_DIR/diagnostics/runs"
COLLECTOR="$ROOT_DIR/tools/collect-previous-boot.sh"
TIMEOUT_SECONDS="${AUREA_TEST_TIMEOUT:-45}"

if ! [[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( TIMEOUT_SECONDS < 5 || TIMEOUT_SECONDS > 600 )); then
    printf 'AUREA_TEST_TIMEOUT must be an integer between 5 and 600 seconds.\n' >&2
    exit 2
fi

if [[ -z "${XDG_RUNTIME_DIR:-}" || ! -d "$XDG_RUNTIME_DIR" ]]; then
    printf 'XDG_RUNTIME_DIR is not set to an existing directory. Start from a normal local user session.\n' >&2
    exit 2
fi

for command_name in cargo timeout tee date mkdir chmod find sort; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "$command_name" >&2
        exit 2
    fi
done

umask 077
mkdir -p -- "$RUNS_DIR"

# Recover interrupted sessions before creating a new one. An unfinished
# run-info file is the durable marker; no separate state database is needed.
shopt -s nullglob
for previous_info in "$RUNS_DIR"/*/run-info.txt; do
    previous_dir="${previous_info%/run-info.txt}"
    previous_id="${previous_dir##*/}"
    if ! grep -q '^finished_at=' "$previous_info"; then
        printf '\nRecovering interrupted diagnostic session: %s\n' "$previous_id"
        if [[ -x "$COLLECTOR" ]]; then
            bash "$COLLECTOR" "$previous_id" || printf 'WARNING: recovery was incomplete for %s.\n' "$previous_id" >&2
        elif [[ -f "$COLLECTOR" ]]; then
            bash "$COLLECTOR" "$previous_id" || printf 'WARNING: recovery was incomplete for %s.\n' "$previous_id" >&2
        else
            printf 'WARNING: diagnostic collector is missing: %s\n' "$COLLECTOR" >&2
        fi
        printf 'recovery_attempted_at=%s\n' "$(date --iso-8601=seconds)" >> "$previous_info"
    fi
done
shopt -u nullglob

RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$RUNS_DIR/$RUN_ID"
if ! mkdir -- "$RUN_DIR"; then
    printf 'A diagnostic run already exists for %s; wait one second and retry.\n' "$RUN_ID" >&2
    exit 2
fi
chmod 700 "$RUN_DIR"

LOG_FILE="$RUN_DIR/native-test.txt"
exec > >(tee -a "$LOG_FILE") 2>&1

STARTED_EPOCH="$(date +%s)"
STARTED_AT="$(date --iso-8601=seconds)"
GIT_HEAD="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || printf unavailable)"

write_metadata() {
    {
        printf 'run_id=%s\n' "$RUN_ID"
        printf 'started_at=%s\n' "$STARTED_AT"
        printf 'started_epoch=%s\n' "$STARTED_EPOCH"
        printf 'hostname=%s\n' "$(hostname 2>/dev/null || printf unknown)"
        printf 'kernel=%s\n' "$(uname -srmo 2>/dev/null || printf unknown)"
        printf 'user_id=%s\n' "$(id -u)"
        printf 'session_type=%s\n' "${XDG_SESSION_TYPE:-unset}"
        printf 'wayland_display_present=%s\n' "$([[ -n "${WAYLAND_DISPLAY:-}" ]] && printf yes || printf no)"
        printf 'display_present=%s\n' "$([[ -n "${DISPLAY:-}" ]] && printf yes || printf no)"
        printf 'timeout_seconds=%s\n' "$TIMEOUT_SECONDS"
        printf 'git_head=%s\n' "$GIT_HEAD"
    } > "$RUN_DIR/run-info.txt"
}
write_metadata

printf 'AureaOS native diagnostic session: %s\n' "$RUN_ID"
printf 'Report directory: %s\n' "$RUN_DIR"
printf 'Build revision: %s\n' "$GIT_HEAD"
printf 'Maximum runtime: %s seconds\n' "$TIMEOUT_SECONDS"

# Capture a safe, allowlisted hardware/software snapshot. Optional tools are
# detected rather than installed or invoked with elevated privileges.
{
    printf '=== Kernel command line ===\n'
    cat /proc/cmdline 2>/dev/null || true
    printf '\n=== PCI devices and active drivers ===\n'
    if command -v lspci >/dev/null 2>&1; then lspci -nnk; else printf 'lspci unavailable\n'; fi
    printf '\n=== DRM devices ===\n'
    for device in /sys/class/drm/card[0-9]*; do
        [[ -e "$device" ]] || continue
        printf '%s: ' "${device##*/}"
        readlink -f "$device/device/driver" 2>/dev/null || true
    done
    printf '\n=== Vulkan summary ===\n'
    if command -v vulkaninfo >/dev/null 2>&1; then
        timeout 12s vulkaninfo --summary 2>&1 || true
    else
        printf 'vulkaninfo unavailable\n'
    fi
    printf '\n=== Relevant kernel modules ===\n'
    if command -v lsmod >/dev/null 2>&1; then
        lsmod | grep -Ei '^(amdgpu|i915|xe|nouveau|nvidia|drm)' || true
    fi
} > "$RUN_DIR/system-snapshot.txt"
chmod 600 "$RUN_DIR/system-snapshot.txt"

cd "$ROOT_DIR"
export RUST_BACKTRACE=1
export RUST_LIB_BACKTRACE=1

printf '\n[1/2] Building Aurea Desktop...\n'
cargo build --locked -p aurea-desktop --bins

printf '\n[2/2] Starting the native DRM/Vulkan desktop...\n'
set +e
timeout --signal=INT --kill-after=10s "${TIMEOUT_SECONDS}s" ./target/debug/aurea-desktop
STATUS=$?
set -e

{
    printf 'finished_at=%s\n' "$(date --iso-8601=seconds)"
    printf 'exit_status=%s\n' "$STATUS"
} >> "$RUN_DIR/run-info.txt"

case "$STATUS" in
    0) printf 'AureaOS desktop exited cleanly.\n' ;;
    124) printf 'Test time limit reached; SIGINT was sent for graceful shutdown.\n' ;;
    137) printf 'WARNING: SIGKILL was required. Inspect the report before another run.\n' ;;
    *) printf 'AureaOS exited with status %s.\n' "$STATUS" ;;
esac

# Capture the current boot journal as well as the persistent app log.
if [[ -f "$COLLECTOR" ]]; then
    bash "$COLLECTOR" "$RUN_ID" --current-boot || printf 'WARNING: post-run collection was incomplete.\n' >&2
fi

printf '\nDiagnostic report ready: %s\n' "$RUN_DIR"
printf 'Review privacy-sensitive details before committing and pushing.\n'
exit "$STATUS"
