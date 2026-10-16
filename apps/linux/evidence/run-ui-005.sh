#!/usr/bin/env bash
# UI-005 shortcut-reference capture harness.
#
# Runs the real app (mock and live) under the real Wayland session, pins the
# window to the reference dimensions with a Hyprland window rule, drives it with
# `wtype`, and captures PNG evidence with `grim`.
#
# Every line this script prints is from a real command; nothing is simulated.
# It tees its full transcript to evidence/ui-005-run.log.
#
# Usage (from apps/linux):
#   bash evidence/run-ui-005.sh [socket]
#
# Requirements (all present on this host): hyprctl, grim, identify, wtype,
# python3, and a built `target/debug/relayable` plus a built
# `target/debug/relayable-server` (started here for the live scenario).
#
# What it proves:
#   * `Ctrl+Shift+?` opens the Relayable-original shortcut reference, and the
#     app logs the open/close edge; the toggle and `Esc` both close it.
#   * the window and the workspace body geometry are unchanged by the overlay
#     (pre/post `hyprctl clients -j`, every PNG is 1099x706).
#   * a bare `?` typed at the prompt still reaches the pane, and `Esc` outside
#     the overlay still sends 0x1b (the live scenario runs `cat -v`, which echoes
#     the escape as `^[`).
#   * the id-dispatched handler arms split right (`Ctrl+Shift+E`) and zoom
#     (`Ctrl+Shift+Z`) still do what the keymap says.

set -u
cd "$(dirname "$0")/.."   # apps/linux

SOCK="${1:-${RELAYABLE_SERVER:-/tmp/relayable-ui005-ui/relayable.sock}}"
APP=./target/debug/relayable
SERVER=../../target/debug/relayable-server
CLI=../../target/debug/relayable
OUT=evidence
TMP=/tmp/relayable-ui005-ui
LOG="$OUT/ui-005-run.log"
mkdir -p "$TMP"

exec > >(tee "$LOG") 2>&1
set -x

FAILURES=0
check() { # check <label> <actual> <expected>
  if [ "$2" = "$3" ]; then
    echo "CHECK PASS: $1 = $2"
  else
    echo "CHECK FAIL: $1 = $2 (expected $3)"
    FAILURES=$((FAILURES + 1))
  fi
}

same_frame() { # same_frame <label> <tag-a> <tag-b> <yes|no>
  local actual=no
  if cmp -s "$OUT/ui-005-$2.png" "$OUT/ui-005-$3.png"; then actual=yes; fi
  echo "cmp $OUT/ui-005-$2.png $OUT/ui-005-$3.png -> $actual"
  check "$1" "$actual" "$4"
}

echo "date_utc=$(date -u +%FT%TZ)"
echo "socket=$SOCK"
echo "display=$(printf 'WAYLAND_DISPLAY=%s DISPLAY=%s' "${WAYLAND_DISPLAY:-}" "${DISPLAY:-}")"
echo "app=$(realpath "$APP") server=$(realpath "$SERVER")"

geometry_for() {
  hyprctl clients -j | python3 -c '
import json, sys
try:
    clients = json.load(sys.stdin)
except Exception as exc:  # noqa: BLE001 - report, never fake
    print("PARSE_ERROR %s" % exc, file=sys.stderr)
    raise SystemExit(0)
for c in clients:
    if c.get("class") == "relayable":
        x, y = c["at"]; w, h = c["size"]
        print("%d,%d %dx%d" % (x, y, w, h), end="")
        break
'
}

capture() { # capture <tag> [expect_w> <expect_h>]
  local tag="$1" geom size
  geom="$(geometry_for)"
  echo "capture tag=$tag geometry=$geom"
  if [ -n "$geom" ]; then
    grim -g "$geom" "$OUT/ui-005-$tag.png"
    echo "grim_window_exit=$?"
    size="$(identify -format '%wx%h' "$OUT/ui-005-$tag.png")"
    echo "png=$OUT/ui-005-$tag.png size=$size"
    if [ $# -eq 3 ]; then check "png $tag size" "$size" "$2x$3"; fi
  else
    echo "capture skipped: no relayable window found"
  fi
  grim "$OUT/ui-005-$tag-fullscreen.png"
  echo "grim_fullscreen_exit=$?"
  identify "$OUT/ui-005-$tag-fullscreen.png"
}

apply_rule() {
  local w="$1" h="$2"
  hyprctl eval "if RELAYABLE_UI005_RULE then RELAYABLE_UI005_RULE:set_enabled(false) end; RELAYABLE_UI005_RULE = hl.window_rule({ match = { class = \"^relayable$\" }, float = true, size = { $w, $h }, move = { 10, 40 } })"
  echo "window_rule_exit=$?"
}

focus_app() {
  hyprctl eval 'hl.dispatch(hl.dsp.focus({ window = "class:^(relayable)$" }))'
  echo "focus_exit=$?"
  sleep 0.4
}

launch_app() { # launch_app <tag> <mode> <w> <h>
  local tag="$1" mode="$2" w="$3" h="$4"
  local args=(--canvas "$w" "$h")
  [ "$mode" = "mock" ] && args+=(--mock)
  apply_rule "$w" "$h"
  sleep 0.4
  RELAYABLE_SERVER="$SOCK" "$APP" "${args[@]}" > "$TMP/$tag.log" 2>&1 &
  APP_PID=$!
  echo "app_pid=$APP_PID tag=$tag mode=$mode"
  sleep 5
}

stop_app() {
  if kill -0 "$APP_PID" 2>/dev/null; then
    kill -TERM "$APP_PID"
    echo "kill_exit=$?"
    wait "$APP_PID"
    echo "wait_exit=$?"
  fi
}

dump_app_log() { # dump_app_log <tag> [grep pattern]
  echo "--- app log $TMP/$1.log ---"
  cat "$TMP/$1.log"
  echo "--- end app log ---"
}

edge_count() { # edge_count <tag> <opened|closed>
  # `grep -c` prints the count itself (0 included) and exits 1 on no match.
  grep -c "shortcut reference $2" "$TMP/$1.log" 2>/dev/null || true
}

# ---------------------------------------------------------------------------
# Scenario A: mock mode — the overlay itself, on the baseline canvas.
# ---------------------------------------------------------------------------
launch_app mock-open mock 1099 706
focus_app
GEOM_BEFORE="$(geometry_for)"
echo "geometry_before=$GEOM_BEFORE"
capture mock-before 1099 706

# Ctrl+Shift+/ opens the reference.
wtype -M ctrl -M shift -k question
echo "wtype_toggle_open_exit=$?"
sleep 1
capture mock-overlay 1099 706
check "toggle opened the overlay (log)" "$(edge_count mock-open opened)" 1

# Esc closes it, and only while it is open.
wtype -k Escape
echo "wtype_overlay_escape_exit=$?"
sleep 1
capture mock-escape-closed 1099 706
check "Esc closed the overlay (log)" "$(edge_count mock-open closed)" 1

# The toggle opens and closes it as well.
wtype -M ctrl -M shift -k question
echo "wtype_toggle2_open_exit=$?"
sleep 0.6
capture mock-overlay-2 1099 706
wtype -M ctrl -M shift -k question
echo "wtype_toggle2_close_exit=$?"
sleep 0.6
capture mock-toggle-closed 1099 706
check "toggle opened twice (log)" "$(edge_count mock-open opened)" 2
check "toggle closed it (log)" "$(edge_count mock-open closed)" 2

# A bare `?` is not the toggle: the mock server echoes it back into the pane.
wtype -k question
echo "wtype_bare_question_exit=$?"
sleep 1
capture mock-question-pane 1099 706

# Ctrl+/ must NOT open the overlay (the documented contingency is not taken) and
# must not change the frame at all.
wtype -M ctrl -k slash
echo "wtype_ctrl_slash_exit=$?"
sleep 0.6
capture mock-ctrl-slash 1099 706
check "Ctrl+/ did not open the overlay (log)" "$(edge_count mock-open opened)" 2
same_frame "Ctrl+/ changed nothing on screen" mock-question-pane mock-ctrl-slash yes
# Closing the overlay restores the pre-overlay frame byte for byte.
same_frame "closing the overlay leaves no trace" mock-before mock-escape-closed yes
# The overlay renders deterministically (same input, same pixels).
same_frame "the overlay is deterministic" mock-overlay mock-overlay-2 yes

# Two id-dispatched arms, visible in pixels: split right and zoom.
wtype -M ctrl -M shift -k e
echo "wtype_split_exit=$?"
sleep 3
capture mock-split 1099 706
wtype -M ctrl -M shift -k z
echo "wtype_zoom_exit=$?"
sleep 1
capture mock-zoom 1099 706
wtype -M ctrl -M shift -k z
echo "wtype_unzoom_exit=$?"
sleep 1

GEOM_AFTER="$(geometry_for)"
echo "geometry_after=$GEOM_AFTER"
check "window geometry unchanged by the overlay" "$GEOM_AFTER" "$GEOM_BEFORE"
dump_app_log mock-open
stop_app

# ---------------------------------------------------------------------------
# Scenario B: live server — bare `?` and `Esc` still reach the pane.
# ---------------------------------------------------------------------------
RELAYABLE_SERVER="$SOCK" RELAYABLE_STATE_DIR="$TMP/state" SHELL=/bin/sh TERM=xterm-256color \
  "$SERVER" run > "$TMP/server.log" 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "$TMP/server.pid"
echo "server_pid=$SERVER_PID"
ready=0
attempt=0
while [ "$attempt" -lt 100 ]; do
  if [ -S "$SOCK" ] && RELAYABLE_SERVER="$SOCK" "$CLI" ls --json >/dev/null 2>&1; then
    ready=1
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.1
done
check "live server ready" "$ready" 1

launch_app live live 1099 706
focus_app
capture live-baseline 1099 706

# `cat -v` keeps the tty in canonical echo mode, so a bare `?` and `Esc` show up
# in the pane exactly as the kernel echoes them (`?` and `^[`).
wtype 'cat -v'
echo "wtype_catv_exit=$?"
wtype -k Return
echo "wtype_catv_enter_exit=$?"
sleep 1
capture live-catv 1099 706

wtype -k question
echo "wtype_live_question_exit=$?"
sleep 1
capture live-question 1099 706

wtype -k Escape
echo "wtype_live_escape_exit=$?"
sleep 1
capture live-escape 1099 706

# The overlay opens over the live workspace and Esc closes it.
wtype -M ctrl -M shift -k question
echo "wtype_live_toggle_exit=$?"
sleep 1
capture live-overlay 1099 706
check "live toggle opened the overlay (log)" "$(edge_count live opened)" 1
same_frame "the live overlay covers the pane" live-escape live-overlay no
wtype -k Escape
echo "wtype_live_close_exit=$?"
sleep 1
capture live-closed 1099 706
check "live Esc closed the overlay (log)" "$(edge_count live closed)" 1
# The pane really changed while `cat -v` ran: the bare `?` and the `Esc` echo
# (`^[`) are in the frame (see evidence/ui-005-live-escape.png).
same_frame "the live pane received the keys" live-catv live-escape no
same_frame "closing the live overlay restores the pane frame" live-escape live-closed yes

# Leave `cat`: Ctrl+C is not an app binding, it is 0x03 to the pane.
wtype -M ctrl -k c
echo "wtype_ctrl_c_exit=$?"
sleep 0.5
dump_app_log live
stop_app

# --- Teardown --------------------------------------------------------------
hyprctl eval 'if RELAYABLE_UI005_RULE then RELAYABLE_UI005_RULE:set_enabled(false) end'
echo "rule_disable_exit=$?"
if [ -f "$TMP/server.pid" ]; then
  server_pid="$(sed 's/[^0-9]//g' "$TMP/server.pid")"
  if [ -n "$server_pid" ] && kill -0 "$server_pid" 2>/dev/null; then
    kill -TERM "$server_pid"
    echo "server_kill_exit=$?"
    wait "$server_pid" 2>/dev/null
    echo "server_wait_exit=$?"
  fi
fi
rm -f "$SOCK"
echo "socket_removed=$([ -e "$SOCK" ] && echo no || echo yes)"
if [ -f "$TMP/server.log" ]; then
  echo "--- server log ---"
  cat "$TMP/server.log"
  echo "--- end server log ---"
fi

echo "check_failures=$FAILURES"
echo "date_utc_end=$(date -u +%FT%TZ)"
if [ "$FAILURES" -eq 0 ]; then
  echo "UI-005 CAPTURE OK"
else
  echo "UI-005 CAPTURE FAILED"
fi
