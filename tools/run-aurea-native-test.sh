#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AureaOS — bounded native DRM/Vulkan test launcher
# Copyright (C) 2026 Alessandro Ceccarelli

set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_PATH="$ROOT_DIR/tools/run-aurea-native-test.sh"

if [[ "${1:-}" == "--session" ]]; then
    WELCOME_PID=""
    COMPOSITOR_PID=""

    cleanup() {
        if [[ -n "$WELCOME_PID" ]] && kill -0 "$WELCOME_PID" 2>/dev/null; then
            kill -INT "$WELCOME_PID" 2>/dev/null || true
            wait "$WELCOME_PID" 2>/dev/null || true
        fi
        if [[ -n "$COMPOSITOR_PID" ]] && kill -0 "$COMPOSITOR_PID" 2>/dev/null; then
            kill -INT "$COMPOSITOR_PID" 2>/dev/null || true
            wait "$COMPOSITOR_PID" 2>/dev/null || true
        fi
    }
    trap cleanup EXIT

    printf 'Starting AureaOS native compositor...\n'
    ./target/debug/aurea-desktop &
    COMPOSITOR_PID=$!

    SOCKET_PATH="$XDG_RUNTIME_DIR/wayland-0"
    printf 'Waiting for the Wayland socket: %s\n' "$SOCKET_PATH"
    for ((attempt = 0; attempt < 150; attempt++)); do
        if [[ -S "$SOCKET_PATH" ]]; then
            break
        fi
        if ! kill -0 "$COMPOSITOR_PID" 2>/dev/null; then
            set +e
            wait "$COMPOSITOR_PID"
            STATUS=$?
            set -e
            COMPOSITOR_PID=""
            printf 'Compositor exited before creating its Wayland socket (status %s).\n' "$STATUS"
            exit "$STATUS"
        fi
        sleep 0.1
    done

    if [[ ! -S "$SOCKET_PATH" ]]; then
        printf 'Wayland socket did not appear within 15 seconds.\n'
        exit 1
    fi

    printf 'Connecting the Welcome surface to AureaOS...\n'
    WAYLAND_DISPLAY=wayland-0 ./target/debug/aurea-welcome &
    WELCOME_PID=$!
    printf 'Welcome client PID: %s\n' "$WELCOME_PID"

    set +e
    wait "$COMPOSITOR_PID"
    STATUS=$?
    set -e
    COMPOSITOR_PID=""

    cleanup
    WELCOME_PID=""
    exit "$STATUS"
fi

STATE_DIR="${XDG_STATE_HOME:-${HOME:?HOME must be set}/.local/state}"
LOG_DIR="$STATE_DIR/aureaos/logs"
TIMEOUT_SECONDS="${AUREA_TEST_TIMEOUT:-45}"

if ! [[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || (( TIMEOUT_SECONDS < 5 || TIMEOUT_SECONDS > 600 )); then
    printf 'AUREA_TEST_TIMEOUT must be an integer between 5 and 600 seconds.\n' >&2
    exit 2
fi

if [[ -z "${XDG_RUNTIME_DIR:-}" || ! -d "$XDG_RUNTIME_DIR" ]]; then
    printf 'XDG_RUNTIME_DIR is not set to an existing directory. Log in through the normal system session first.\n' >&2
    exit 2
fi

for command_name in cargo timeout tee date; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        printf 'Required command not found: %s\n' "$command_name" >&2
        exit 2
    fi
done

mkdir -p -- "$LOG_DIR"
chmod 700 "$LOG_DIR"
LOG_FILE="$LOG_DIR/native-test-$(date +%Y%m%d-%H%M%S).log"
umask 077
exec > >(tee -a "$LOG_FILE") 2>&1

printf 'AureaOS native test started: %s\n' "$(date --iso-8601=seconds)"
printf 'Log file: %s\n' "$LOG_FILE"
printf 'Maximum runtime: %s seconds\n' "$TIMEOUT_SECONDS"
printf 'The complete test session receives SIGINT at the deadline, then SIGKILL after 10 seconds if it cannot exit.\n'

cd "$ROOT_DIR"
export RUST_BACKTRACE=1

printf '\n[1/2] Building the desktop and standalone Welcome client...\n'
cargo build --locked -p aurea-desktop --bins

printf '\n[2/2] Running the bounded native test session...\n'
set +e
timeout --signal=INT --kill-after=10s "${TIMEOUT_SECONDS}s" bash "$SCRIPT_PATH" --session
STATUS=$?
set -e

case "$STATUS" in
    0)
        printf 'AureaOS compositor exited cleanly.\n'
        ;;
    124)
        printf 'Test time limit reached; timeout sent SIGINT for graceful shutdown.\n'
        ;;
    137)
        printf 'WARNING: graceful shutdown failed; timeout required SIGKILL. Inspect the log before another run.\n'
        ;;
    *)
        printf 'AureaOS test session exited with status %s. Inspect the log for the failure.\n' "$STATUS"
        ;;
esac

printf 'Persistent log: %s\n' "$LOG_FILE"
exit "$STATUS"
