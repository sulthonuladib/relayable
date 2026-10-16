#!/usr/bin/env bash
#
# Relayable Linux end-to-end smoke test (`full.md` §13.4).
#
# The required order is followed exactly:
#   build -> launch a foreground test server -> create a session (CLI) ->
#   send input -> verify output -> disconnect the client -> reconnect ->
#   verify the server-owned process continued -> shut down cleanly ->
#   print concise logs.
#
# Safety: no `sudo`, no global installs, no network (cargo runs `--offline`), no
# systemd unit and no system service. Everything runs under a private
# `mktemp -d` directory; the foreground server is always terminated by the EXIT
# trap and the temp directory is always removed. Requires no root.
#
# The frozen `relayable` CLI has no send/attach command
# (`protocol/compatibility/core-001-cli-json.md`), so the send/verify/disconnect
# and reconnect steps are driven by the real `relayable-client` library through
# the test-support `smoke-driver` binary (`tests/src/bin/smoke-driver.rs`).
#
# Test-only binary path overrides (default to the real workspace build):
#   SMOKE_SERVER_BIN, SMOKE_CLI_BIN, SMOKE_DRIVER_BIN

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || { echo "[smoke] cannot cd to $ROOT" >&2; exit 1; }

SERVER_BIN="${SMOKE_SERVER_BIN:-$ROOT/target/debug/relayable-server}"
CLI_BIN="${SMOKE_CLI_BIN:-$ROOT/target/debug/relayable}"
DRIVER_BIN="${SMOKE_DRIVER_BIN:-$ROOT/target/debug/smoke-driver}"

log() { printf '[smoke] %s\n' "$*"; }
die() { printf '[smoke] ERROR: %s\n' "$*" >&2; exit 1; }

SERVER_PID=""
WORK_DIR=""
SOCKET=""

# Always reap the server and remove the temp directory, even on failure.
cleanup() {
    local status=$?
    if [ -n "$SERVER_PID" ]; then
        if kill -0 "$SERVER_PID" 2>/dev/null; then
            log "cleanup: terminating leftover server pid $SERVER_PID"
            kill -TERM "$SERVER_PID" 2>/dev/null || true
            local waited=0
            while [ "$waited" -lt 50 ] && kill -0 "$SERVER_PID" 2>/dev/null; do
                sleep 0.1
                waited=$((waited + 1))
            done
            if kill -0 "$SERVER_PID" 2>/dev/null; then
                kill -KILL "$SERVER_PID" 2>/dev/null || true
            fi
        fi
        wait "$SERVER_PID" 2>/dev/null || true
        SERVER_PID=""
    fi
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        rm -rf "$WORK_DIR"
    fi
    return "$status"
}
trap cleanup EXIT

log "Relayable full.md §13.4 Linux smoke test"
log "repo root: $ROOT"
log "workspace target dir: $ROOT/target"

# --- 1. build -------------------------------------------------------------
log "step 1/9: cargo build --workspace --offline"
cargo build --workspace --offline || die "cargo build --workspace --offline failed"
for bin in "$SERVER_BIN" "$CLI_BIN" "$DRIVER_BIN"; do
    [ -x "$bin" ] || die "required binary missing: $bin (workspace target dir is $ROOT/target; did \`cargo build --workspace\` build every target?)"
done
log "binaries: server=$SERVER_BIN cli=$CLI_BIN driver=$DRIVER_BIN"

# --- 2. private temp dir / socket ----------------------------------------
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/relayable-smoke.XXXXXX")" || die "mktemp failed"
SOCKET="$WORK_DIR/relayable.sock"
SERVER_LOG="$WORK_DIR/server.log"
# No root, no systemd, no system service: a private temp dir and a unix socket.
log "step 2/9: temp dir $WORK_DIR, socket $SOCKET (no root, no system service)"

# --- 3. launch foreground test server ------------------------------------
log "step 3/9: launching foreground server ($SERVER_BIN run)"
# Persistence (STORE-001/ADR-004) is isolated to this run's temp dir so the smoke
test never touches the real $HOME/.local/state/relayable or contends on its lock.
RELAYABLE_SERVER="$SOCKET" \
RELAYABLE_STATE_DIR="$WORK_DIR/state" \
RELAYABLE_IDLE_TIMEOUT_MILLIS=20000 \
RELAYABLE_HEARTBEAT_INTERVAL_MILLIS=5000 \
SHELL=/bin/sh TERM=xterm-256color \
"$SERVER_BIN" run >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
log "server pid: $SERVER_PID"

ready=0
attempt=0
while [ "$attempt" -lt 100 ]; do
    if [ -S "$SOCKET" ] && RELAYABLE_SERVER="$SOCKET" "$CLI_BIN" ls --json >/dev/null 2>&1; then
        ready=1
        break
    fi
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        break
    fi
    attempt=$((attempt + 1))
    sleep 0.1
done
if [ "$ready" -ne 1 ]; then
    echo "----- $SERVER_LOG -----" >&2
    cat "$SERVER_LOG" >&2 || true
    die "server never became ready on $SOCKET within ~10s"
fi
log "server ready on $SOCKET"

# --- 4. create a session through the frozen CLI --------------------------
NEW_JSON="$WORK_DIR/new.json"
log "step 4/9: creating a session via the CLI (relayable new --json)"
if ! RELAYABLE_SERVER="$SOCKET" "$CLI_BIN" new --json >"$NEW_JSON" 2>"$WORK_DIR/new.err"; then
    cat "$WORK_DIR/new.err" >&2 || true
    die "relayable new --json failed"
fi
SESSION_ID="$("$DRIVER_BIN" field "$NEW_JSON" session_id)" || die "could not read session_id from $NEW_JSON"
BLOCK_ID="$("$DRIVER_BIN" field "$NEW_JSON" block_id)" || die "could not read block_id from $NEW_JSON"
PROGRAM_PID="$("$DRIVER_BIN" field "$NEW_JSON" program_pid)" || die "could not read program_pid from $NEW_JSON"
[ -n "$SESSION_ID" ] || die "session_id is empty"
[ -n "$BLOCK_ID" ] || die "block_id is empty"
[ -n "$PROGRAM_PID" ] || die "program_pid is empty"
log "session_id:  $SESSION_ID"
log "block_id:    $BLOCK_ID"
log "program_pid: $PROGRAM_PID"

NONCE="$$-$RANDOM"

# --- 5. send input, verify output, then disconnect the client ------------
log "step 5/9: sending input, verifying output, then disconnecting the client"
"$DRIVER_BIN" establish --socket "$SOCKET" --block "$BLOCK_ID" --nonce "$NONCE" \
    || die "smoke-driver establish failed"

# --- 6. reconnect, then verify the server-owned process continued --------
log "step 6/9: reconnecting, then verifying the server-owned process continued"
"$DRIVER_BIN" resume --socket "$SOCKET" --block "$BLOCK_ID" --nonce "$NONCE" \
    || die "smoke-driver resume failed"
kill -0 "$PROGRAM_PID" 2>/dev/null \
    || die "server-owned pid $PROGRAM_PID did not survive the client disconnect"

# --- 7. inspect: same pid, active session --------------------------------
INSPECT_JSON="$WORK_DIR/inspect.json"
log "step 7/9: verifying session inspect reports the same pid and an active session"
if ! RELAYABLE_SERVER="$SOCKET" "$CLI_BIN" session inspect --json -s "$SESSION_ID" \
        >"$INSPECT_JSON" 2>"$WORK_DIR/inspect.err"; then
    cat "$WORK_DIR/inspect.err" >&2 || true
    die "relayable session inspect failed"
fi
INSPECT_PID="$("$DRIVER_BIN" field "$INSPECT_JSON" blocks.0.program_pid)" \
    || die "session inspect has no blocks[0].program_pid (see $INSPECT_JSON)"
INSPECT_STATE="$("$DRIVER_BIN" field "$INSPECT_JSON" session.state)" \
    || die "session inspect has no session.state (see $INSPECT_JSON)"
INSPECT_SESSION="$("$DRIVER_BIN" field "$INSPECT_JSON" session.session_id)" \
    || die "session inspect has no session.session_id (see $INSPECT_JSON)"
[ "$INSPECT_PID" = "$PROGRAM_PID" ] \
    || die "session inspect reports program_pid=$INSPECT_PID, expected $PROGRAM_PID"
[ "$INSPECT_STATE" = "active" ] \
    || die "session inspect reports state=$INSPECT_STATE, expected active"
[ "$INSPECT_SESSION" = "$SESSION_ID" ] \
    || die "session inspect reports session_id=$INSPECT_SESSION, expected $SESSION_ID"
log "inspect OK: session active, program_pid=$PROGRAM_PID"

# --- 8. close the session, then shut the server down cleanly -------------
CLOSE_JSON="$WORK_DIR/close.json"
log "step 8/9: closing the session, then shutting the server down with SIGTERM"
RELAYABLE_SERVER="$SOCKET" "$CLI_BIN" session close --json -s "$SESSION_ID" \
    >"$CLOSE_JSON" 2>"$WORK_DIR/close.err"
close_code=$?
log "session close exit code: $close_code"
if [ "$close_code" -ne 0 ]; then
    cat "$WORK_DIR/close.err" >&2 || true
    die "relayable session close failed (exit $close_code)"
fi

kill -TERM "$SERVER_PID" || die "kill -TERM $SERVER_PID failed"
wait "$SERVER_PID"
server_exit=$?
SERVER_PID=""
log "server exit code after SIGTERM: $server_exit"
[ "$server_exit" -eq 0 ] || die "server exited $server_exit, expected 0 on SIGTERM"
[ ! -e "$SOCKET" ] || die "server did not unlink the socket $SOCKET on shutdown"
log "server shut down cleanly and unlinked $SOCKET"

# --- 9. cleanup ----------------------------------------------------------
log "step 9/9: removing temp dir $WORK_DIR"
rm -rf "$WORK_DIR"
WORK_DIR=""
log "steps that ran: build; server launch; session create; input/output round-trip; client disconnect; process survival; reconnect; session inspect; session close; SIGTERM shutdown; socket unlink; temp-dir cleanup"
echo "SMOKE OK"
exit 0
