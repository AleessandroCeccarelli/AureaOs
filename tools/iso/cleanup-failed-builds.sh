#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -Eeuo pipefail

BASE_DIR="${TMPDIR:-/var/tmp}"
CURRENT_COMMIT="$(git rev-parse --short=12 HEAD 2>/dev/null || true)"
APPLY=false

usage() {
    echo "Usage: $0 [--apply]" >&2
    echo "Without --apply, only lists obsolete AureaOS ISO workspaces." >&2
}
while (($#)); do
    case "$1" in
        --apply) APPLY=true; shift ;;
        --help|-h) usage; exit 0 ;;
        *) usage; exit 2 ;;
    esac
done

[[ "$BASE_DIR" = /* && -d "$BASE_DIR" ]] || {
    echo "Temporary build root is missing or not absolute: $BASE_DIR" >&2
    exit 1
}
[[ -n "$CURRENT_COMMIT" ]] || {
    echo "Run this script from the AureaOS Git repository." >&2
    exit 1
}
if "$APPLY" && ((EUID != 0)); then
    echo "Apply mode must run as root: sudo bash tools/iso/cleanup-failed-builds.sh --apply" >&2
    exit 1
fi

shopt -s nullglob
candidates=("$BASE_DIR"/aureaos-podman-build-*)
removed=0
for parent in "${candidates[@]}"; do
    [[ -d "$parent" && ! -L "$parent" ]] || continue
    name="${parent##*/}"
    commit="${name#aureaos-podman-build-}"
    [[ "$commit" =~ ^[0-9a-f]{12}$ ]] || continue
    [[ "$commit" == "$CURRENT_COMMIT" ]] && {
        echo "KEEP current commit workspace: $parent"
        continue
    }

    workspace="$parent/build"
    marker="$workspace/.aureaos-source-commit"
    [[ -d "$workspace" && ! -L "$workspace" && -s "$marker" ]] || {
        echo "SKIP unrecognized/incomplete directory: $parent"
        continue
    }
    recorded="$(<"$marker")"
    [[ "$recorded" == "$commit"* ]] || {
        echo "SKIP commit marker mismatch: $parent (marker: $recorded)"
        continue
    }

    echo "$("$APPLY" && echo REMOVE || echo WOULD REMOVE): $parent"
    "$APPLY" || continue

    # Never delete a workspace while its managed Podman build holds the lock.
    exec 9>"$parent/.aureaos-live-build.lock"
    if ! flock -n 9; then
        echo "SKIP active build (lock is held): $parent" >&2
        exec 9>&-
        continue
    fi
    rm -rf -- "$workspace"
    exec 9>&-
    rm -f -- "$parent/.aureaos-live-build.lock"
    rmdir -- "$parent" 2>/dev/null || true
    removed=$((removed + 1))
done

if "$APPLY"; then
    echo "Cleanup complete. Removed $removed obsolete AureaOS workspace(s)."
else
    echo "Dry run complete. Re-run with --apply to remove only the listed obsolete workspaces."
fi
