#!/usr/bin/env bash
#
# Relayable file-size check (docs/architecture/code-organization.md).
#
# Reports hand-written Rust files that exceed the code-organization limits:
#   production source : target 500,  hard cap 1000
#   tests             : target 800,  hard cap 1200
#   entry/re-export   : target 150,  hard cap 300   (lib.rs / main.rs / mod.rs)
#
# Generated code (prost OUT_DIR, checked-in generated) and `spikes/**` are exempt.
#
# Modes:
#   --report (default) : print a table and exit 0 (used while refactors are pending).
#   --strict           : exit 1 when any file is over its hard cap (wired into CI as REF-010).
#
# Only `set -u`; every file is visited.

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

MODE="report"
case "${1:-}" in
    --strict) MODE="strict" ;;
    --report|"") MODE="report" ;;
    -h|--help)
        sed -n '2,18p' "$0"
        exit 0
        ;;
    *)
        echo "check-file-sizes: unknown option: $1" >&2
        exit 2
        ;;
esac

PROD_TARGET=500;  PROD_CAP=1000
TEST_TARGET=800;  TEST_CAP=1200
ENTRY_TARGET=150; ENTRY_CAP=300

over_cap=0
over_target=0
checked=0

printf '%-6s %7s %7s  %s\n' "KIND" "LINES" "CAP" "FILE"
printf '%s\n' "----------------------------------------------------------------------"

while IFS= read -r file; do
    # Skip build output and the throwaway spike workspaces.
    case "$file" in
        ./target/*|*/target/*|./spikes/*|*/spikes/*) continue ;;
    esac
    [ -f "$file" ] || continue
    base="$(basename "$file")"
    lines="$(wc -l < "$file")"
    checked=$((checked + 1))

    if [ "$base" = "tests.rs" ] \
        || printf '%s' "$file" | grep -qE '(^|/)tests/'; then
        kind="test"; target=$TEST_TARGET; cap=$TEST_CAP
    elif [ "$base" = "lib.rs" ] || [ "$base" = "main.rs" ] || [ "$base" = "mod.rs" ]; then
        kind="entry"; target=$ENTRY_TARGET; cap=$ENTRY_CAP
    else
        kind="prod"; target=$PROD_TARGET; cap=$PROD_CAP
    fi

    if [ "$lines" -gt "$cap" ]; then
        printf '%-6s %7s %7s  %s\n' "$kind" "$lines" "$cap" "$file"
        over_cap=$((over_cap + 1))
    elif [ "$lines" -gt "$target" ]; then
        printf '%-6s %7s %7s  %s\n' "$kind" "$lines" "$cap" "$file"
        over_target=$((over_target + 1))
    fi
done < <(find . -name '*.rs' -type f | sort)

echo
echo "checked=$checked files; over-hard-cap=$over_cap; over-target-but-under-cap=$over_target"
if [ "$over_cap" -gt 0 ]; then
    echo "Hard-cap violations must be split: docs/architecture/code-organization.md (REF-006..REF-009)."
    if [ "$MODE" = "strict" ]; then
        exit 1
    fi
fi
echo "mode=$MODE; see docs/architecture/code-organization.md"
exit 0
