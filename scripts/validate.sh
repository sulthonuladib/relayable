#!/usr/bin/env bash
#
# Relayable CORE-001 workspace validation.
#
# Runs the full offline check suite and exits non-zero on the first failure.
# No `set -e` is used: every command's exit code is reported explicitly, then the
# script stops on the first failure. No network access is performed.

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# Run one command, print it and its exit code, and succeed only on exit 0.
run() {
    echo
    echo "=== $* ==="
    "$@"
    code=$?
    echo "--- exit code: $code ---"
    if [ "$code" -ne 0 ]; then
        echo "FAILED: $* (exit $code)" >&2
    fi
    return "$code"
}

# 1. Formatting.
run cargo fmt --all --check || exit $?

# 2. Lints, all targets (including the integration tests).
run cargo clippy --workspace --all-targets --offline -- -D warnings || exit $?

# 3. Build every workspace binary (the integration harness locates them next to
#    the test executable).
run cargo build --workspace --offline || exit $?

# 4. Every workspace test, including the integration tests.
run cargo test --workspace --offline || exit $?

# 5. The required acceptance test, with its per-property evidence lines.
run cargo test -p relayable-integration-tests --offline --test lifecycle -- --nocapture || exit $?

# 6. ADR-001 dependency-direction assertion (`cargo tree`, no JSON parsing).
run bash scripts/check-dependency-direction.sh || exit $?

# 7. The phase-1 spike is a separate workspace root and must still build.
(
    cd spikes/terminal-engine || exit 1
    run cargo build --offline || exit $?
) || exit $?

echo
echo "All CORE-001 validation checks passed."
exit 0
