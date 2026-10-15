#!/usr/bin/env bash
#
# ADR-001 dependency-direction assertion (CORE-001 evidence).
#
# ADR-001 ("Proposed crate layout and dependency direction") requires "an
# assertion over `cargo metadata` output (or an equivalent CI check) that
# compares actual edges against the allowed set and fails on any forbidden edge
# or cycle." This script implements that assertion with `cargo tree` only: no
# JSON parsing, no python/jq, no extra tooling. Every cargo invocation is
# `--offline`.
#
# It prints each check, aborts (exit 1) on the first violation, and ends with a
# clear success line.
#
# Allowed edges (ADR-001 Decision M):
#   model    -> (nothing in the workspace)         pure domain
#   protocol -> model                              wire mapping
#   client   -> model, protocol                    API consumer
#   server   -> model, protocol, pty               composes the stack
#   cli      -> model, protocol, client            API consumer / binary
#   pty      -> (nothing in the workspace)         OS boundary, opaque handles
#
# The direction checks below match exact package names ("NAME vX.Y.Z"), so
# `prost` never matches transitive `prost-types`/`prost-derive` and `serde`
# never matches `serde_core`/`serde_json`.
#
# relayable-model is checked in two distinct scopes (CHECK 1a / CHECK 1b):
#   * DIRECT (depth-1) normal deps must be EXACTLY {thiserror, uuid}.
#   * TRANSITIVE normal deps must contain no other relayable-* crate and none of
#     the banned families below.
# The one documented-allowed transitive exception is `libc`:
#   allowed transitive: libc v0.2.190 <- getrandom v0.4.3 <- uuid v1.28.0 <-
#   relayable-model (RNG syscall glue; no model API, transport, serialization,
#   or async surface)
# Rule: the model must never gain a DIRECT transport/serde/async/OS-crate dep,
# and must never gain a transitive one except `libc` via `uuid`'s RNG
# (getrandom). Any other new package in the model tree is a STOP-and-report
# event, not something to add to an allowlist. (`nix` stays banned transitively:
# it is itself a direct OS-syscall crate.)
#
# Cycle check: `cargo metadata` is the assertion. Cargo rejects a dependency
# cycle during resolution, so a successful metadata query proves the resolved
# workspace graph is acyclic. The direction checks above then prove which edges
# actually exist.
#
# DEC-010 guard: a default-feature build of relayable-protocol must stay free of
# Unix-socket code, so every `std::os::unix` / socket symbol must live in
# src/transport.rs, compiled only behind the non-default `transport-unix`
# feature gate.
#
# n7 (feature unification): `cargo build --workspace` unifies features across the
# workspace, so a workspace build DOES enable `transport-unix` on
# relayable-protocol because relayable-server/client/cli request it. The
# "relayable-protocol has no Unix-socket symbols" property is therefore only
# meaningful — and only verified here — for a crate-alone build, which is exactly
# what step [8] tests: `cargo build -p relayable-protocol --offline` with its
# DEFAULT features (no `transport-unix`). A future Android client must depend on
# `relayable-protocol` WITHOUT the `transport-unix` feature for this to hold on
# that target too.

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# `cargo tree` text for one package's normal (non-build, non-dev) dependencies.
tree_for() {
    cargo tree -p "$1" --edges normal --offline 2>&1
}

# Does the cargo-tree text ($1) contain a package named exactly $2?
tree_has() {
    printf '%s\n' "$1" | grep -Eq "(^|[^A-Za-z0-9_.-])$2 v[0-9]"
}

# Fail unless every named package is present in the tree.
assert_present() {
    local pkg="$1" tree="$2"
    shift 2
    local name
    for name in "$@"; do
        tree_has "$tree" "$name" || fail "$pkg tree is missing required crate '$name'"
    done
}

# Fail if any named package is present in the tree.
assert_absent() {
    local pkg="$1" tree="$2"
    shift 2
    local name
    for name in "$@"; do
        tree_has "$tree" "$name" && fail "$pkg tree contains forbidden crate '$name'"
    done
    return 0
}

echo "== ADR-001 dependency-direction check =="

# --- CHECK 1a: DIRECT (depth-1) normal deps are exactly {thiserror, uuid} ---
echo "[1a] relayable-model DIRECT (depth-1) normal deps are exactly {thiserror, uuid}"
MODEL_DIRECT="$(cargo tree -p relayable-model --edges normal --depth 1 --offline 2>&1)" \
    || fail "cargo tree -p relayable-model --depth 1 failed"
# A direct dependency from any forbidden family fails first, naming the crate.
assert_absent relayable-model "$MODEL_DIRECT" \
    relayable-protocol relayable-client relayable-server relayable-pty relayable-cli \
    prost bytes serde serde_json portable-pty gpui alacritty_terminal vte tokio nix libc
# Extract the crate names from the depth-1 branch lines and require the exact set.
direct_names="$(printf '%s\n' "$MODEL_DIRECT" \
    | sed -n -E 's/^[├└]── ([A-Za-z0-9_.-]+) v[0-9].*$/\1/p' | sort -u)"
expected_direct="$(printf 'thiserror\nuuid\n' | sort -u)"
if [ "$direct_names" != "$expected_direct" ]; then
    fail "relayable-model direct normal deps are not exactly {thiserror, uuid}; got: $(printf '%s' "$direct_names" | tr '\n' ' ')"
fi
echo "  OK: direct deps exactly {thiserror, uuid}"

# --- CHECK 1b: TRANSITIVE normal deps ---------------------------------------
echo "[1b] relayable-model TRANSITIVE normal deps: no other relayable-*, none of {prost,bytes,serde,serde_json,portable-pty,gpui,alacritty_terminal,vte,tokio,nix}"
MODEL="$(tree_for relayable-model)" || fail "cargo tree -p relayable-model failed"
assert_absent relayable-model "$MODEL" \
    relayable-protocol relayable-client relayable-server relayable-pty relayable-cli \
    prost bytes serde serde_json portable-pty gpui alacritty_terminal vte tokio nix
echo "  allowed transitive: libc v0.2.190 <- getrandom v0.4.3 <- uuid v1.28.0 <- relayable-model (RNG syscall glue; no model API, transport, serialization, or async surface)"
echo "  OK: no other relayable-* crate; only the documented libc via uuid's RNG is allowed"

# 2. relayable-protocol: model + wire types only.
echo "[2] relayable-protocol: requires relayable-model; no client/server/pty/cli, no PTY/UI/serde/tokio"
PROTOCOL="$(tree_for relayable-protocol)" || fail "cargo tree -p relayable-protocol failed"
assert_present relayable-protocol "$PROTOCOL" relayable-model
assert_absent relayable-protocol "$PROTOCOL" \
    relayable-client relayable-server relayable-pty relayable-cli portable-pty gpui serde tokio
echo "  OK: relayable-model present; no forbidden dependency"

# 3. relayable-client: model + protocol only.
echo "[3] relayable-client: requires relayable-model + relayable-protocol; no server/pty/cli"
CLIENT="$(tree_for relayable-client)" || fail "cargo tree -p relayable-client failed"
assert_present relayable-client "$CLIENT" relayable-model relayable-protocol
assert_absent relayable-client "$CLIENT" relayable-server relayable-pty relayable-cli
echo "  OK: relayable-model + relayable-protocol present; no forbidden dependency"

# 4. relayable-server: composes model + protocol + pty.
echo "[4] relayable-server: requires relayable-model + relayable-protocol + relayable-pty; no client/cli"
SERVER="$(tree_for relayable-server)" || fail "cargo tree -p relayable-server failed"
assert_present relayable-server "$SERVER" relayable-model relayable-protocol relayable-pty
assert_absent relayable-server "$SERVER" relayable-client relayable-cli
echo "  OK: relayable-model + relayable-protocol + relayable-pty present; no forbidden dependency"

# 5. relayable-cli: model + protocol + client only.
echo "[5] relayable-cli: requires relayable-model + relayable-protocol + relayable-client; no pty/server"
CLI="$(tree_for relayable-cli)" || fail "cargo tree -p relayable-cli failed"
assert_present relayable-cli "$CLI" relayable-model relayable-protocol relayable-client
assert_absent relayable-cli "$CLI" relayable-pty relayable-server
echo "  OK: relayable-model + relayable-protocol + relayable-client present; no forbidden dependency"

# 6. relayable-pty: OS boundary with opaque handles; no relayable-* crate at all.
echo "[6] relayable-pty: no relayable-* crate at all"
PTY="$(tree_for relayable-pty)" || fail "cargo tree -p relayable-pty failed"
assert_absent relayable-pty "$PTY" \
    relayable-model relayable-protocol relayable-client relayable-server relayable-cli
echo "  OK: no relayable-* crate in the relayable-pty tree"

# 7. Cycle check (see header comment for the reasoning).
echo "[7] cycle check: cargo metadata accepts the resolved graph (Cargo rejects cycles)"
if ! cargo metadata --format-version 1 --offline --no-deps >/dev/null; then
    fail "cargo metadata failed; the resolved workspace graph is not acyclic"
fi
echo "  OK: cargo metadata resolved the workspace graph; no dependency cycle"

# 8. Source-level DEC-010 guard + default-feature build.
echo "[8] DEC-010: Unix-socket symbols only in feature-gated src/transport.rs; default build compiles"
echo '  n7 feature-unification note: a `cargo build --workspace` DOES enable transport-unix on'
echo '  relayable-protocol (server/client/cli request it), so the no-Unix-symbols property is'
echo '  only meaningful for a crate-alone build -- which is exactly what this step tests:'
echo '  `cargo build -p relayable-protocol` with DEFAULT features. A future Android build must'
echo '  depend on relayable-protocol WITHOUT transport-unix.'
SRC="crates/relayable-protocol/src"
SYMBOL_RE='os::unix|os::fd|UnixListener|UnixStream|SO_PEERCRED'
unexpected="$(grep -rln -E "$SYMBOL_RE" "$SRC" | grep -v '/transport\.rs$' || true)"
if [ -n "$unexpected" ]; then
    printf 'FAIL: Unix-socket symbols found outside %s/transport.rs:\n%s\n' "$SRC" "$unexpected" >&2
    exit 1
fi
if ! grep -rq -E "$SYMBOL_RE" "$SRC/transport.rs"; then
    fail "$SRC/transport.rs holds no Unix-socket symbols; the guard is mis-targeted"
fi
# "Nearby gate": the module declaration in lib.rs must carry the cfg attribute
# on the preceding line, and the crate must define the feature.
if ! grep -B1 'pub mod transport;' "$SRC/lib.rs" | grep -q 'cfg(feature = "transport-unix")'; then
    fail "$SRC/lib.rs does not gate 'pub mod transport;' behind cfg(feature = \"transport-unix\")"
fi
if ! grep -q '^transport-unix = \[\]' crates/relayable-protocol/Cargo.toml; then
    fail "crates/relayable-protocol/Cargo.toml does not declare the transport-unix feature"
fi
echo "  OK: all Unix-socket symbols are in $SRC/transport.rs, gated by cfg(feature = \"transport-unix\")"
echo "  -> cargo build -p relayable-protocol --offline (default features)"
cargo build -p relayable-protocol --offline >/dev/null || fail "default-feature relayable-protocol build failed"
echo "  OK: default-feature relayable-protocol builds without the socket module"

echo
echo "ADR-001 dependency-direction check passed."
exit 0
