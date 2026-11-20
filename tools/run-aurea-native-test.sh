#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS — bounded native DRM/Vulkan test launcher
# Copyright (C) 2026 Alessandro Ceccarelli

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
RUNS_DIR="$ROOT_DIR/diagnostics/runs"
TIMEOUT_SECONDS="${AUREA_TEST_TIMEOUT:-45}"

if ! [[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( TIMEOUT_SECONDS < 5 || TIMEOUT_SECONDS > 600 )); then
    printf 'AUREA_TEST_TIMEOUT must be an integer between 5 and 600 seconds.\n' >&2
    exit 2
fi

if [[ -z "${XDG_RUNTIME_DIR:-}" || ! -d "$XDG_RUNTIME_DIR" ]]; then
    printf 'XDG_RUNTIME_DIR is not set to an existing directory. Log in through the normal user session first.\n' >&2
    exit 2
fi

for command_name in cargo timeout tee date mkdir chmod; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "$command_name" >&2
        exit 2
    fi
done

umask 077
mkdir -p -- "$RUNS_DIR"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$RUNS_DIR/$RUN_ID"

# Never append a new run to an older report created in the same second.
if ! mkdir -- "$RUN_DIR"; then
    printf 'A diagnostic run already exists for %s; wait one second and retry.\n' "$RUN_ID" >&2
    exit 2
fi
chmod 700 "$RUN_DIR"

LOG_FILE="$RUN_DIR/native-test.txt"
exec > >(tee -a "$LOG_FILE") 2>&1

STARTED_EPOCH="$(date +%s)"
STARTED_AT="$(date --iso-8601=seconds)"

write_metadata() {
    {
        printf 'run_id=%s\n' "$RUN_ID"
        printf 'started_at=%s\n' "$STARTED_AT"
        printf 'started_epoch=%s\n' "$STARTED_EPOCH"
        printf 'hostname=%s\n' "$(hostname 2>/dev/null || printf unknown)"
        printf 'kernel=%s\n' "$(uname -srmo 2>/dev/null || printf unknown)"
        printf 'user_id=%s\n' "$(id -u)"
        printf 'session_type=%s\n' "${XDG_SESSION_TYPE:-unset}"
        printf 'runtime_dir_present=%s\n' "$([[ -n "${XDG_RUNTIME_DIR:-}" && -d "$XDG_RUNTIME_DIR" ]] && printf yes || printf no)"
        printf 'wayland_display_present=%s\n' "$([[ -n "${WAYLAND_DISPLAY:-}" ]] && printf yes || printf no)"
        printf 'display_present=%s\n' "$([[ -n "${DISPLAY:-}" ]] && printf yes || printf no)"
        printf 'timeout_seconds=%s\n' "$TIMEOUT_SECONDS"
        printf 'git_head=%s\n' "$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || printf unavailable)"
    } > "$RUN_DIR/run-info.txt"
}
write_metadata

printf 'AureaOS native test started: %s\n' "$STARTED_AT"
printf 'Diagnostic directory: %s\n' "$RUN_DIR"
printf 'Maximum runtime: %s seconds\n' "$TIMEOUT_SECONDS"
printf 'The desktop receives SIGINT at the deadline, then SIGKILL after 10 seconds if it cannot exit.\n'
printf 'Logs are stored in the repository; inspect them before committing or pushing.\n'

cd "$ROOT_DIR"
export RUST_BACKTRACE=1

printf '\n[1/2] Building the native desktop...\n'
cargo build --locked -p aurea-desktop --bins

printf '\n[2/2] Starting the bounded native DRM/Vulkan session...\n'
set +e
timeout --signal=INT --kill-after=10s "${TIMEOUT_SECONDS}s" ./target/debug/aurea-desktop
STATUS=$?
set -e

{
    printf 'finished_at=%s\n' "$(date --iso-8601=seconds)"
    printf 'exit_status=%s\n' "$STATUS"
} >> "$RUN_DIR/run-info.txt"

case "$STATUS" in
    0)
        printf 'AureaOS compositor exited cleanly.\n'
        ;;
    124)
        printf 'Test time limit reached; timeout sent SIGINT for graceful shutdown.\n'
        ;;
    137)
        printf 'WARNING: graceful shutdown failed; timeout required SIGKILL. Inspect diagnostics before another run.\n'
        ;;
    *)
        printf 'AureaOS test session exited with status %s. Inspect diagnostics for the failure.\n' "$STATUS"
        ;;
esac

printf 'Diagnostic directory: %s\n' "$RUN_DIR"
printf 'After reboot, run: bash tools/collect-previous-boot.sh %s\n' "$RUN_ID"
exit "$STATUS"
