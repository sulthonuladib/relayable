#!/usr/bin/env bash
# UI-001: launch the Relayable GPUI window under the live Wayland session,
# screenshot it, capture stdout/stderr, then terminate it and record the status.
#
# Every line of output here is from a real command; nothing is simulated.
# Run from `apps/linux`:  bash evidence/run-window.sh
set -u

cd "$(dirname "$0")/.."   # apps/linux
APP=./target/debug/relayable

export XDG_SESSION_TYPE="${XDG_SESSION_TYPE:-wayland}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-1}"
export DISPLAY="${DISPLAY:-:0}"

rm -f evidence/relayable-window.png evidence/relayable-window-fullscreen.png \
      evidence/run-stdout-stderr.log evidence/hyprctl-clients.json

echo "date_utc=$(date -u +%FT%TZ)"
echo "session=$(printf 'XDG_SESSION_TYPE=%s WAYLAND_DISPLAY=%s DISPLAY=%s' "$XDG_SESSION_TYPE" "$WAYLAND_DISPLAY" "$DISPLAY")"
echo "run_command=$APP"

"$APP" > evidence/run-stdout-stderr.log 2>&1 &
APP_PID=$!
echo "app_pid=$APP_PID"

RAN_SECONDS=6
sleep "$RAN_SECONDS"
echo "ran_seconds=$RAN_SECONDS"

if kill -0 "$APP_PID" 2>/dev/null; then
  echo "status_after_${RAN_SECONDS}s=ALIVE"
else
  echo "status_after_${RAN_SECONDS}s=EXITED"
fi
grep -E '^(Name|State):' /proc/"$APP_PID"/status 2>&1 || true

echo "command: hyprctl clients -j > evidence/hyprctl-clients.json"
hyprctl clients -j > evidence/hyprctl-clients.json
echo "hyprctl_exit=$?"

GEOM="$(python3 - <<'PY'
import json
try:
    with open("evidence/hyprctl-clients.json") as fh:
        clients = json.load(fh)
except Exception as exc:  # noqa: BLE001 - report, do not fake
    print("PARSE_ERROR: %s" % exc)
    raise SystemExit(0)
matches = [c for c in clients if c.get("title") == "Relayable"]
if not matches:
    print("NOT_FOUND")
    raise SystemExit(0)
c = matches[0]
x, y = c["at"]
w, h = c["size"]
print("FOUND address=%s class=%s title=%r size=%dx%d at=%d,%d" % (
    c.get("address"), c.get("class"), c.get("title"), w, h, x, y))
print("GEOM=%d,%d %dx%d" % (x, y, w, h))
PY
)"
echo "$GEOM"

GEOM_STR="$(printf '%s\n' "$GEOM" | sed -n 's/^GEOM=//p')"
if [ -n "$GEOM_STR" ]; then
  echo "command: grim -g \"$GEOM_STR\" evidence/relayable-window.png"
  grim -g "$GEOM_STR" evidence/relayable-window.png
  echo "grim_window_exit=$?"
else
  echo "window capture skipped: window geometry not found"
fi

echo "command: grim evidence/relayable-window-fullscreen.png"
grim evidence/relayable-window-fullscreen.png
echo "grim_fullscreen_exit=$?"

echo "--- artifacts ---"
for f in evidence/relayable-window.png evidence/relayable-window-fullscreen.png; do
  ls -la "$f" 2>&1
  file "$f" 2>&1
  identify "$f" 2>&1
done

echo "command: kill -TERM $APP_PID"
kill -TERM "$APP_PID"
echo "kill_exit=$?"
wait "$APP_PID"
WAIT_EXIT=$?
echo "wait_exit=$WAIT_EXIT"
if kill -0 "$APP_PID" 2>/dev/null; then
  echo "final=STILL_ALIVE"
else
  echo "final=TERMINATED"
fi

echo "--- stdout/stderr of the app ---"
cat evidence/run-stdout-stderr.log
echo "--- end stdout/stderr ---"
