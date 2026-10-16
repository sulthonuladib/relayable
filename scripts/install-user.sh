#!/usr/bin/env bash
#
# Relayable per-user installer (full.md §7.1, §7.2, §7.3).
#
# Installs the Relayable CLI (`relayable`) and foreground server
# (`relayable-server`) into the user's XDG bin directory, plus a systemd --user
# unit and (optionally) the GPUI desktop app. No root, no system-wide writes,
# no service enable/start.
#
# Usage:
#   scripts/install-user.sh [--with-desktop] [--skip-daemon-reload]
#
# All target directories honour the standard XDG overrides (HOME, XDG_BIN_HOME,
# XDG_CONFIG_HOME, XDG_DATA_HOME), so the script is sandbox-testable.
#
# Environment:
#   XDG_BIN_HOME     install dir for binaries   (default: $HOME/.local/bin)
#   XDG_CONFIG_HOME  base for systemd user unit (default: $HOME/.config)
#   XDG_DATA_HOME    base for .desktop entry    (default: $HOME/.local/share)
#   CARGO            cargo executable to use    (default: cargo from PATH)

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

WITH_DESKTOP=0
DAEMON_RELOAD=1

log() { printf '[install-user] %s\n' "$*"; }
die() { printf '[install-user] ERROR: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'EOF'
Relayable per-user installer

Usage:
  scripts/install-user.sh [--with-desktop] [--skip-daemon-reload]

Options:
  --with-desktop         Also build apps/linux (GPUI) and install it as
                         `relayable-desktop`. This is a heavy release build and
                         pulls the gpui dependency tree; it is skipped by
                         default.
  --skip-daemon-reload   Do not run `systemctl --user daemon-reload` after
                         installing the unit.
  -h, --help             Show this help.

Installs (all under XDG user directories, no root):
  $XDG_BIN_HOME/relayable
  $XDG_BIN_HOME/relayable-server
  $XDG_BIN_HOME/relayable-desktop        (only with --with-desktop)
  $XDG_CONFIG_HOME/systemd/user/relayable-server.service
  $XDG_DATA_HOME/applications/relayable.desktop
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --with-desktop) WITH_DESKTOP=1 ;;
        --skip-daemon-reload|--no-daemon-reload) DAEMON_RELOAD=0 ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf '[install-user] ERROR: unknown option: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

# --- resolve target directories -------------------------------------------
[ -n "${HOME:-}" ] || die "HOME is not set; refusing to guess an install prefix"

BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

for dir in "$BIN_DIR" "$CONFIG_HOME" "$DATA_HOME"; do
    case "$dir" in
        /*) ;;
        *) die "resolved path is not absolute: $dir (check XDG_* overrides)" ;;
    esac
    [ "$dir" != "/" ] || die "refusing to install directly into /"
done

UNIT_DIR="$CONFIG_HOME/systemd/user"
APPS_DIR="$DATA_HOME/applications"

CARGO="${CARGO:-cargo}"
command -v "$CARGO" >/dev/null 2>&1 || die "cargo not found ($CARGO); install the Rust toolchain first"

log "repository:   $REPO_ROOT"
log "binaries:     $BIN_DIR"
log "unit:         $UNIT_DIR/relayable-server.service"
log "desktop file: $APPS_DIR/relayable.desktop"
log "desktop app:  $([ "$WITH_DESKTOP" -eq 1 ] && echo 'yes (--with-desktop)' || echo 'no (default; use --with-desktop)')"

# The shipped unit hard-codes ExecStart=%h/.local/bin/relayable-server.
if [ "$BIN_DIR" != "$HOME/.local/bin" ]; then
    log "WARNING: XDG_BIN_HOME is not \$HOME/.local/bin; the installed unit still"
    log "         ExecStart=%h/.local/bin/relayable-server run. Edit the unit if you"
    log "         expect the systemd service to find this binary."
fi

# --- build the root release binaries --------------------------------------
log "building relayable and relayable-server (release, offline)"
( cd "$REPO_ROOT" && "$CARGO" build --release --offline --bin relayable --bin relayable-server ) \
    || die "cargo build failed"

CLI_SRC="$REPO_ROOT/target/release/relayable"
SERVER_SRC="$REPO_ROOT/target/release/relayable-server"
[ -x "$CLI_SRC" ] || die "expected binary missing: $CLI_SRC"
[ -x "$SERVER_SRC" ] || die "expected binary missing: $SERVER_SRC"

# --- install binaries, unit, desktop entry --------------------------------
install -Dm755 "$CLI_SRC" "$BIN_DIR/relayable" || die "failed to install relayable"
install -Dm755 "$SERVER_SRC" "$BIN_DIR/relayable-server" || die "failed to install relayable-server"
log "installed: $BIN_DIR/relayable"
log "installed: $BIN_DIR/relayable-server"

install -Dm644 "$REPO_ROOT/packaging/systemd/user/relayable-server.service" \
    "$UNIT_DIR/relayable-server.service" || die "failed to install systemd user unit"
log "installed: $UNIT_DIR/relayable-server.service"

install -Dm644 "$REPO_ROOT/packaging/desktop/relayable.desktop" \
    "$APPS_DIR/relayable.desktop" || die "failed to install desktop entry"
log "installed: $APPS_DIR/relayable.desktop"

# --- optional GPUI desktop app --------------------------------------------
if [ "$WITH_DESKTOP" -eq 1 ]; then
    log "building the GPUI desktop app (release, locked, offline) — this is heavy"
    ( cd "$REPO_ROOT/apps/linux" && "$CARGO" build --release --locked --offline ) \
        || die "cargo build failed for apps/linux"
    DESKTOP_SRC="$REPO_ROOT/apps/linux/target/release/relayable"
    [ -x "$DESKTOP_SRC" ] || die "expected desktop binary missing: $DESKTOP_SRC"
    # Install as `relayable-desktop`: the apps/linux bin is named `relayable`,
    # which would collide with this same script's CLI install.
    install -Dm755 "$DESKTOP_SRC" "$BIN_DIR/relayable-desktop" \
        || die "failed to install relayable-desktop"
    log "installed: $BIN_DIR/relayable-desktop"
else
    log "skipping desktop build (pass --with-desktop to enable)"
fi

# --- refresh the user systemd manager (never enable/start) -----------------
if [ "$DAEMON_RELOAD" -eq 1 ]; then
    if command -v systemctl >/dev/null 2>&1; then
        if systemctl --user daemon-reload; then
            log "ran: systemctl --user daemon-reload"
        else
            log "WARNING: systemctl --user daemon-reload failed; run it manually"
        fi
    else
        log "systemctl not found; skipping daemon-reload"
    fi
else
    log "skipped daemon-reload (--skip-daemon-reload)"
fi

# --- next steps ------------------------------------------------------------
cat <<EOF

[install-user] Done.

Next steps:
  1. If '$BIN_DIR' is not on your PATH, add it, e.g.:
       export PATH="$BIN_DIR:\$PATH"
  2. Run the server in the foreground once to try it:
       relayable-server run
     (or install/enable the user service, below)
  3. Enable and start the user service (this installer never does it for you):
       systemctl --user daemon-reload
       systemctl --user enable --now relayable-server
       systemctl --user status relayable-server
     Logs: journalctl --user -u relayable-server -f
  4. Use the CLI:
       relayable ls
       relayable new
       relayable session inspect -s <session>
       relayable session close   -s <session>
  5. Launch the desktop app (if installed with --with-desktop):
       relayable-desktop
     The desktop entry 'Relayable' is in your application menu.

Details and uninstall: docs/operations/user-install.md
EOF
exit 0
