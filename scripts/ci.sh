#!/usr/bin/env bash
#
# Relayable CI entry point — the `full.md` §13.5 build matrix as ONE runnable script.
#
# WHAT THIS IS
#   `scripts/ci.sh` runs the §13.5 build matrix end to end on a single host, prints
#   the exact command and real exit code of every step, and ends with one summary
#   table (step | status PASS/FAIL/SKIP | command | reason) plus an overall exit
#   status. It runs EVERY requested check even after one fails, collects all
#   failures, and exits 1 if any required check failed. Skips are NOT failures but
#   are listed loudly and completely. This extends — it does not replace or weaken
#   — the existing CORE-001 scripts:
#     * scripts/validate.sh                   (root workspace, stop-on-first-failure)
#     * scripts/check-dependency-direction.sh (ADR-001 dependency direction)
#     * scripts/check-file-sizes.sh --strict  (file-size hard caps, REF-010)
#
# MATRIX (`full.md` §13.5)
#   1. cargo fmt --all --check
#   2. cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
#      -> `--all-features` matches full.md §13.5. The root workspace has exactly
#         one feature (relayable-protocol `transport-unix`), and it is already
#         enabled by dependency unification (client/cli/server request it), so
#         the flag changes nothing today but keeps the invocation literal.
#   3. cargo build --workspace --offline
#   4. cargo test --workspace --offline
#      -> build MUST precede test: tests/fixtures/mod.rs `bin()` enforces binary
#         freshness and refuses stale binaries (docs/agent-loop/TESTS.md row 89).
#   5. bash scripts/check-dependency-direction.sh   (ADR-001)
#   6. bash scripts/check-file-sizes.sh --strict    (REF-010): the
#      docs/architecture/code-organization.md hard caps are a required check, not
#      a report — a file above its cap fails the row and the overall result.
#   7. Linux desktop build (`apps/linux`, its OWN Cargo workspace root, empty
#      `[workspace]`): cargo fmt --check, cargo build --locked, cargo check
#      --locked, cargo test --locked, cargo clippy --all-targets --locked, plus
#      the TERM-002 `libc = 0.2.189` lockfile assertion. No `cargo update` is
#      ever run: loosening `libc` past
#      0.2.189 reintroduces `cargo check` exit 101 at `xattr 0.2.3`
#      (`libc::ENOATTR` removed) — apps/linux/README.md §12.
#   8. Standalone spike builds (each its own workspace root): spikes/terminal-engine
#      and spikes/terminal-engine-phase2 — `cargo build` in each.
#   9. Protocol fixtures/tests: `-p relayable-protocol` and
#      `-p relayable-integration-tests --test protocol`.
#  10. Explicitly skipped rows: Android/Gradle (`./gradlew test`, DEC-010), Linux UI
#      smoke/screenshot (needs a live display backend), package build tests (not
#      relevant yet).
#
# NETWORK
#   No network dependency. Every ROOT-workspace cargo invocation keeps `--offline`
#   exactly as the tree already does. This script additionally exports
#   CARGO_NET_OFFLINE=true so NO cargo invocation — including the separate
#   apps/linux and spike workspaces — can silently reach the network. A command
#   that genuinely needs an uncached crate fails visibly and is reported as a
#   blocker instead of being allowed to fetch. The `file-sizes` row runs no cargo
#   command at all (a local `find`/`wc` scan), so it neither needs nor uses the
#   network.
#
# DISPLAY
#   A GUI run/screenshot needs a Wayland display and is NOT part of ci.sh. The
#   Linux UI smoke/screenshot row is reported as best-effort/SKIP with the exact
#   display requirement (apps/linux/README.md §6).
#
# ANDROID (DEC-010)
#   No Android/Gradle work is attempted. DEC-010 defers the Android client, so the
#   §13.5 `./gradlew test` row is an explicit SKIP with the DEC-010 reason. There is
#   no `apps/android` package and no Android SDK/emulator/Gradle used.
#
# .github/workflows
#   Deliberately NOT added: a hosted runner would need network access to fetch
#   crates, which the verified offline workflow forbids. `scripts/ci.sh` is the
#   single local entry point.
#
# Only `set -u` is used — no `set -e` — because every step's exit code must be
# observed and collected so later steps still run.

set -u

# Hard guarantee: no cargo invocation may touch the network. See header.
export CARGO_NET_OFFLINE=true

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# Collected matrix rows: parallel arrays keep the summary order intact.
STEP_NAMES=()
STEP_STATES=()
STEP_CMDS=()
STEP_REASONS=()
FAILURES=0

record() {
    STEP_NAMES+=("$1")
    STEP_STATES+=("$2")
    STEP_CMDS+=("$3")
    STEP_REASONS+=("$4")
}

# Run one command from a directory, print the exact command and its real exit
# code, and record PASS/FAIL. Never aborts the script: later steps still run.
run() {
    local step="$1"
    local dir="$2"
    shift 2
    echo
    echo "=== [$step] (cd $dir) $* ==="
    ( cd "$dir" && "$@" )
    local code=$?
    echo "--- [$step] exit code: $code ---"
    if [ "$code" -ne 0 ]; then
        echo "FAILED: [$step] $* (exit $code)" >&2
        record "$step" FAIL "$*" "exit $code"
        FAILURES=$((FAILURES + 1))
    else
        record "$step" PASS "$*" ""
    fi
}

# Record an explicitly skipped matrix row. Skips are not failures but are always
# printed and listed in the summary.
skip() {
    local step="$1"
    local cmd="$2"
    local reason="$3"
    echo
    echo "=== [$step] SKIP: $cmd ==="
    echo "    reason: $reason"
    record "$step" SKIP "$cmd" "$reason"
}

# TERM-002 remedy guard: EVERY `libc` entry in apps/linux/Cargo.lock must be
# 0.2.189. All entries are checked (not just the first) so the guard cannot go
# silently weak if a second libc version ever appears.
check_libc_pin() {
    local step="desktop-libc-pin"
    local lock="apps/linux/Cargo.lock"
    local cmd="assert every libc entry == 0.2.189 in $lock"
    echo
    echo "=== [$step] $cmd ==="
    local versions
    versions="$(grep -A1 '^name = "libc"$' "$lock" 2>/dev/null \
        | sed -n 's/^version = "\(.*\)"$/\1/p')"
    local count=0
    local bad=0
    local listed=""
    while IFS= read -r version; do
        [ -n "$version" ] || continue
        count=$((count + 1))
        listed="${listed}${listed:+, }${version}"
        [ "$version" = "0.2.189" ] || bad=1
    done <<< "$versions"
    echo "libc entries in $lock: $count (${listed:-<not found>})"
    if [ "$count" -gt 0 ] && [ "$bad" -eq 0 ]; then
        echo "OK: all $count libc entries pinned to 0.2.189 (TERM-002 remedy intact)"
        record "$step" PASS "$cmd" ""
    else
        echo "FAIL: $lock has $count libc entries (${listed:-<none>}); every entry must be 0.2.189 (TERM-002 remedy)." >&2
        echo "      'cargo check' exits 101 at xattr 0.2.3 when libc is 0.2.190+ (libc::ENOATTR removed)." >&2
        echo "      Fix (apps/linux): cargo update -p libc --precise 0.2.189 — but ci.sh never runs cargo update." >&2
        record "$step" FAIL "$cmd" "libc=${listed:-<none>}"
        FAILURES=$((FAILURES + 1))
    fi
}

echo "=== Relayable CI — full.md §13.5 build matrix ==="
echo "date_utc=$(date -u +%FT%TZ)"
echo "cargo=$(cargo --version)  rustc=$(rustc --version)"
echo "root=$ROOT"
echo "CARGO_NET_OFFLINE=$CARGO_NET_OFFLINE (no cargo command may reach the network)"

# --- §13.5 core workspace (root Cargo workspace, offline) --------------------
run fmt            . cargo fmt --all --check
run clippy         . cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
run build          . cargo build --workspace --offline
run test           . cargo test --workspace --offline
run dependency-dir . bash scripts/check-dependency-direction.sh
run file-sizes     . bash scripts/check-file-sizes.sh --strict

# --- §13.5 Linux desktop (separate workspace root at apps/linux) -------------
# Documented commands from apps/linux/README.md §13; `--locked` uses the committed
# apps/linux/Cargo.lock. CARGO_NET_OFFLINE=true keeps them from silently fetching.
DESKTOP="apps/linux"
run desktop-fmt     "$DESKTOP" cargo fmt --check
run desktop-build   "$DESKTOP" cargo build --locked
run desktop-check   "$DESKTOP" cargo check --locked
run desktop-test    "$DESKTOP" cargo test --locked
run desktop-clippy  "$DESKTOP" cargo clippy --all-targets --locked
check_libc_pin

# NOTE: launching the GUI and capturing screenshots needs a live Wayland display
# (apps/linux/README.md §6) and is intentionally NOT run by ci.sh.

# --- §13.5 standalone spikes (each its own workspace root) -------------------
run spike-terminal-engine        spikes/terminal-engine        cargo build
run spike-terminal-engine-phase2 spikes/terminal-engine-phase2 cargo build

# --- §13.5 protocol compatibility/fixtures -----------------------------------
run protocol-tests        . cargo test -p relayable-protocol --offline
run protocol-integration  . cargo test -p relayable-integration-tests --offline --test protocol

# --- §13.5 rows intentionally skipped in this phase --------------------------
skip android-gradle "./gradlew test" \
    "DEC-010: Android client deferred; no Android app/scaffold/SDK/emulator/Gradle work in current scope"
skip linux-ui-smoke "linux UI smoke/screenshot (apps/linux/evidence/run-ui-002.sh)" \
    "not run by this matrix; requires a live display backend (observed: XDG_SESSION_TYPE=${XDG_SESSION_TYPE:-<unset>}, WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>}, DISPLAY=${DISPLAY:-<unset>}). GUI run/screenshot is NOT part of ci.sh (apps/linux/README.md §6)"
skip package-build "package build tests (deb/rpm/systemd packaging)" \
    "not relevant yet: no packaging recipe exists in this phase (§13.5 says 'when relevant')"

# --- summary -----------------------------------------------------------------
echo
echo "=============================== SUMMARY ==============================="
printf '%-26s | %-4s | %s | %s\n' "STEP" "STAT" "COMMAND" "REASON"
printf '%-26s-+-%-4s-+-%s-+-%s\n' "--------------------------" "----" "-------" "------"
for i in "${!STEP_NAMES[@]}"; do
    printf '%-26s | %-4s | %s | %s\n' \
        "${STEP_NAMES[$i]}" "${STEP_STATES[$i]}" "${STEP_CMDS[$i]}" "${STEP_REASONS[$i]:--}"
done

echo
echo "--- SKIPPED checks (explicit, not counted as failures) ---"
any_skip=0
for i in "${!STEP_NAMES[@]}"; do
    if [ "${STEP_STATES[$i]}" = "SKIP" ]; then
        echo "  SKIP ${STEP_NAMES[$i]}: ${STEP_CMDS[$i]}"
        echo "       reason: ${STEP_REASONS[$i]}"
        any_skip=1
    fi
done
if [ "$any_skip" -eq 0 ]; then
    echo "  (none)"
fi

echo
if [ "$FAILURES" -eq 0 ]; then
    echo "OVERALL: PASS (0 required checks failed; skips listed above are not failures)"
    exit 0
else
    echo "OVERALL: FAIL ($FAILURES required check(s) failed; skips are not failures)" >&2
    exit 1
fi
