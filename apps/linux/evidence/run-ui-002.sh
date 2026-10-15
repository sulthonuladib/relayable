#!/usr/bin/env bash
# UI-002 reference-workspace capture harness.
#
# Runs the real app against a live `relayable-server` (or `--mock`) under the
# real Wayland session, pins the window to reference dimensions with a Hyprland
# window rule, drives it with `wtype`, and captures PNG evidence with `grim`.
#
# Every line this script prints is from a real command; nothing is simulated.
# It tees its full transcript to evidence/ui-002-run.log.
#
# Usage (from apps/linux):
#   bash evidence/run-ui-002.sh [socket]
#
# Requirements (all present on this host): hyprctl, grim, identify, wtype,
# python3, and a built `target/debug/relayable` plus a running
# `relayable-server` on `$RELAYABLE_SERVER` (default
# /tmp/relayable-ui002-ui/relayable.sock).

set -u
cd "$(dirname "$0")/.."   # apps/linux

SOCK="${1:-${RELAYABLE_SERVER:-/tmp/relayable-ui002-ui/relayable.sock}}"
APP=./target/debug/relayable
OUT=evidence
TMP=/tmp/relayable-ui002-ui
LOG="$OUT/ui-002-run.log"
mkdir -p "$TMP"

exec > >(tee "$LOG") 2>&1
set -x

echo "date_utc=$(date -u +%FT%TZ)"
echo "socket=$SOCK"
echo "display=$(printf 'WAYLAND_DISPLAY=%s DISPLAY=%s' "${WAYLAND_DISPLAY:-}" "${DISPLAY:-}")"

# Small helper scripts typed into the pane with wtype.
cat > "$TMP/osc.sh" <<'EOF'
#!/bin/sh
printf '\033]2;Relayable Demo\033\\'
printf '\033]7;file://host%s\033\\' "$HOME"
stty size
echo RELAYABLE-OSC-SENT
EOF
chmod +x "$TMP/osc.sh"
cat > "$TMP/size.sh" <<'EOF'
#!/bin/sh
echo RELAYABLE-SIZE-BEFORE
stty size
EOF
chmod +x "$TMP/size.sh"

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

capture() {
  local tag="$1" geom
  geom="$(geometry_for)"
  echo "capture tag=$tag geometry=$geom"
  if [ -n "$geom" ]; then
    grim -g "$geom" "$OUT/ui-002-$tag.png"
    echo "grim_window_exit=$?"
    identify "$OUT/ui-002-$tag.png"
  else
    echo "capture skipped: no relayable window found"
  fi
  grim "$OUT/ui-002-$tag-fullscreen.png"
  echo "grim_fullscreen_exit=$?"
  identify "$OUT/ui-002-$tag-fullscreen.png"
}

apply_rule() {
  local w="$1" h="$2"
  # Disable the previous runtime rule (if any) before adding a new one.
  hyprctl eval "if RELAYABLE_UI002_RULE then RELAYABLE_UI002_RULE:set_enabled(false) end; RELAYABLE_UI002_RULE = hl.window_rule({ match = { class = \"^relayable$\" }, float = true, size = { $w, $h }, move = { 10, 40 } })"
  echo "window_rule_exit=$?"
}

focus_app() {
  hyprctl eval 'hl.dispatch(hl.dsp.focus({ window = "class:^(relayable)$" }))'
  echo "focus_exit=$?"
  sleep 0.4
}

launch_app() {
  local tag="$1" mode="$2" w="$3" h="$4"
  local args=(--canvas "$w" "$h")
  [ "$mode" = "mock" ] && args+=(--mock)
  apply_rule "$w" "$h"
  sleep 0.4
  RELAYABLE_SERVER="$SOCK" "$APP" "${args[@]}" > "$TMP/$tag.log" 2>&1 &
  APP_PID=$!
  echo "app_pid=$APP_PID tag=$tag mode=$mode"
  sleep 5
  echo "--- app log $TMP/$tag.log ---"
  cat "$TMP/$tag.log"
  echo "--- end app log ---"
}

stop_app() {
  if kill -0 "$APP_PID" 2>/dev/null; then
    kill -TERM "$APP_PID"
    echo "kill_exit=$?"
    wait "$APP_PID"
    echo "wait_exit=$?"
  fi
}

# --- Scenario 1: live baseline workspace ----------------------------------
launch_app baseline live 1099 706
capture workspace
stop_app

# --- Scenarios 2-4: multiple tabs, live program/cwd, tab switching, split --
launch_app tabs live 1099 706
focus_app
wtype "$TMP/osc.sh"
echo "wtype_osc_exit=$?"
wtype -k Return
echo "wtype_enter_exit=$?"
sleep 1.5
capture tabs-one-osc
focus_app
wtype -M ctrl -M shift -k n
echo "wtype_newtab1_exit=$?"
sleep 2
focus_app
wtype -M ctrl -M shift -k n
echo "wtype_newtab2_exit=$?"
sleep 2
focus_app
# Capture `tabs` on tab 2 so it is genuinely distinct from switch-tab3
# (the earlier revision captured it right after creating tab 3, which made it
# byte-identical to the tab-3 frame).
wtype -M ctrl -k 2
echo "wtype_tab2_exit=$?"
sleep 1.5
capture tabs
focus_app
wtype -M ctrl -k 1
echo "wtype_tab1_exit=$?"
sleep 1.5
capture switch-tab1
focus_app
wtype -M ctrl -k 3
echo "wtype_tab3_exit=$?"
sleep 1.5
capture switch-tab3
focus_app
wtype -M ctrl -M shift -k e
echo "wtype_split_exit=$?"
sleep 3
capture split
focus_app
wtype -M ctrl -M shift -k s
echo "wtype_picker_exit=$?"
sleep 1.5
capture picker
focus_app
wtype -k Escape
echo "wtype_picker_escape_exit=$?"
sleep 0.5
echo "--- app log after interaction ---"
cat "$TMP/tabs.log"
echo "--- end ---"
stop_app

# --- Scenario: N concurrent live panes (D1) + 3-pane resize (D2) ----------
# Three panes in one row must each get an equal share and all stay live; the
# app log records per-stream output byte totals and each pane's cols x rows.
launch_app npanes live 1099 706
focus_app
wtype 'echo PANE-ONE-MARKER'; echo "wtype_pane1_exit=$?"
wtype -k Return; echo "wtype_pane1_enter_exit=$?"
sleep 1
focus_app
wtype -M ctrl -M shift -k e; echo "wtype_npanes_split1_exit=$?"
sleep 3
wtype 'echo PANE-TWO-MARKER'; echo "wtype_pane2_exit=$?"
wtype -k Return; echo "wtype_pane2_enter_exit=$?"
sleep 1
focus_app
wtype -M ctrl -M shift -k e; echo "wtype_npanes_split2_exit=$?"
sleep 3
wtype 'echo PANE-THREE-MARKER'; echo "wtype_pane3_exit=$?"
wtype -k Return; echo "wtype_pane3_enter_exit=$?"
sleep 1.5
capture npanes
# Prove pane 1 is LIVE while unfocused: focus it and type again; its stream must
# still be attached (D1) and must produce new output.
focus_app
wtype -M ctrl -M shift -k Left; echo "wtype_focus_prev1_exit=$?"
wtype -M ctrl -M shift -k Left; echo "wtype_focus_prev2_exit=$?"
sleep 0.5
wtype 'echo PANE-ONE-SECOND'; echo "wtype_pane1b_exit=$?"
wtype -k Return; echo "wtype_pane1b_enter_exit=$?"
sleep 1.5
capture npanes-live
# Close the focused pane (keyboard path): the remaining panes must stay attached
# and focus must move to a neighbour without any spurious re-attach.
focus_app
wtype -M ctrl -M shift -k w; echo "wtype_close_exit=$?"
sleep 1.5
capture npanes-closed
echo "--- app log after npanes ---"
cat "$TMP/npanes.log"
echo "--- end npanes log ---"
stop_app

# --- Scenario 5/6: smaller and larger window ------------------------------
launch_app smaller live 900 600
capture smaller
stop_app

launch_app larger live 1400 900
capture larger
stop_app

# --- Scenario 7: mock mode ------------------------------------------------
launch_app mock mock 1099 706
capture mock
stop_app

# --- Scenario 8: resize proof (pane cols/rows change with window size) -----
launch_app resize live 1099 706
focus_app
wtype "$TMP/size.sh"
echo "wtype_size_exit=$?"
wtype -k Return
echo "wtype_size_enter_exit=$?"
sleep 1
capture resize-before
# Hyprland window rules only apply when a window is opened, so shrink the
# *existing* floating window with a window resize dispatcher instead.
hyprctl eval 'hl.dispatch(hl.dsp.window.resize({ x = -199, y = -106, relative = true }))'
echo "window_resize_exit=$?"
sleep 2
capture resize-after
echo "--- app log after resize ---"
cat "$TMP/resize.log"
echo "--- end ---"
stop_app

# --- Teardown: disable the runtime rule, kill the server, remove socket ----
hyprctl eval 'if RELAYABLE_UI002_RULE then RELAYABLE_UI002_RULE:set_enabled(false) end'
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
rm -rf "$TMP"
echo "scratch_removed=$([ -d "$TMP" ] && echo no || echo yes)"
echo "date_utc_end=$(date -u +%FT%TZ)"
