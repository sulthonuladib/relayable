#!/usr/bin/env bash
#
# Non-interactive cleanup of done+merged task worktrees (standing user authorization, 2026-10-16).
#
# Use this instead of the confirmation-gated `remove_worktree` when you just want the checkout gone.
# It NEVER force-removes and NEVER touches the primary checkout or the Git branches.
#
# Usage:
#   scripts/cleanup-worktrees.sh <branch>...          # dry run: report what would be removed
#   scripts/cleanup-worktrees.sh --apply <branch>...  # remove the eligible worktrees
#
# A named branch's worktree is removed only when ALL hold:
#   * it is a linked worktree of this repo (not the primary checkout),
#   * its branch tip is an ancestor of `master` (i.e. merged),
#   * the worktree has no uncommitted/untracked changes.
# Anything else is skipped with a reason. The branch itself is always preserved.

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

APPLY=0
if [ "${1:-}" = "--apply" ]; then
    APPLY=1
    shift
fi

if [ "$#" -eq 0 ]; then
    echo "usage: scripts/cleanup-worktrees.sh [--apply] <branch>..." >&2
    exit 2
fi

PRIMARY="$(git rev-parse --show-toplevel)"
removed=0
skipped=0

for branch in "$@"; do
    path="$(git worktree list --porcelain \
        | awk -v br="refs/heads/$branch" '$1 == "worktree" { p = $2 } $1 == "branch" && $2 == br { print p }')"
    if [ -z "$path" ]; then
        echo "SKIP  $branch: no linked worktree"
        skipped=$((skipped + 1))
        continue
    fi
    if [ "$path" = "$PRIMARY" ]; then
        echo "SKIP  $branch: primary checkout ($path)"
        skipped=$((skipped + 1))
        continue
    fi
    if ! git rev-parse --verify --quiet "refs/heads/$branch" >/dev/null; then
        echo "SKIP  $branch: no such local branch"
        skipped=$((skipped + 1))
        continue
    fi
    if ! git merge-base --is-ancestor "refs/heads/$branch" master; then
        echo "SKIP  $branch: not merged into master"
        skipped=$((skipped + 1))
        continue
    fi
    if [ -n "$(git -C "$path" status --porcelain 2>/dev/null)" ]; then
        echo "SKIP  $branch: worktree has uncommitted changes ($path)"
        skipped=$((skipped + 1))
        continue
    fi

    if [ "$APPLY" -eq 1 ]; then
        if git worktree remove "$path"; then
            echo "REMOVED $branch ($path) — branch preserved"
            removed=$((removed + 1))
        else
            echo "SKIP  $branch: git worktree remove failed ($path)"
            skipped=$((skipped + 1))
        fi
    else
        echo "WOULD REMOVE $branch ($path) — branch preserved"
        removed=$((removed + 1))
    fi
done

mode="dry-run"; [ "$APPLY" -eq 1 ] && mode="apply"
echo "cleanup-worktrees: mode=$mode eligible=$removed skipped=$skipped (branches always preserved)"
