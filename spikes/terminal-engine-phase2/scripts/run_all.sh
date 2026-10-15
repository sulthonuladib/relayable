#!/usr/bin/env bash
# Relayable TERM-002 / ADR-003 phase B — one reproducible entry point.
#
# Stamps provenance into results/raw/run_all_meta.txt, writes fixtures, runs the
# real-program PTY captures, then runs `analyze` (fixtures + replays + checks +
# throughput observation), and finally the three quality gates. Prints the
# PASS/FAIL/NOT_TESTED/UNSUPPORTED tally and exits non-zero if any gating check
# FAILs or any quality gate is non-zero.
#
# Reproduce with:  spikes/terminal-engine-phase2/scripts/run_all.sh
#
# Ownership note: this script deliberately deletes only the result subtrees it
# owns (checks/, streams/, fixtures/, perf/ and the summary files) plus the
# generated compare/matrix.* files (compare/README.txt is preserved). It never
# touches results/raw/deps/ (owned by the phase-A dependency probe) or
# spikes/terminal-engine/** (phase 1, read-only).

set -u -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$CRATE_DIR/../.." && pwd)"
cd "$CRATE_DIR"

RAW="$CRATE_DIR/results/raw"
CHECKS="$RAW/checks"
STREAMS="$RAW/streams"
FIXTURES="$RAW/fixtures"
PERF="$RAW/perf"
COMPARE="$RAW/compare"
META="$RAW/run_all_meta.txt"

# D10: stamp the script's REAL exit code, whatever path we leave by. The trap
# is registered before any work so even an early `exit 2` is recorded.
EXIT_STAMP="$RAW/run_all_exit.txt"
trap 'rc=$?; { echo "exit_code=$rc"; echo "utc_timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"; echo "script=scripts/run_all.sh"; } > "$EXIT_STAMP"' EXIT

CAPTURE_BIN="$CRATE_DIR/target/debug/capture"
ANALYZE_BIN="$CRATE_DIR/target/debug/analyze"

rm -rf "$CHECKS" "$STREAMS" "$FIXTURES" "$PERF"
rm -f "$RAW/summary.txt" "$RAW/summary.json" "$RAW/run_all_meta.txt" "$RAW/run_all_exit.txt"
mkdir -p "$CHECKS" "$STREAMS" "$FIXTURES" "$PERF" "$COMPARE"
rm -f "$COMPARE/matrix.txt" "$COMPARE/matrix.json"

# Hyperlink target directory for the OSC 8 real-program leg.
mkdir -p "$FIXTURES/hyperlink_dir"
printf 'alpha\n' > "$FIXTURES/hyperlink_dir/alpha.txt"
printf 'beta\n' > "$FIXTURES/hyperlink_dir/beta.txt"

provenance() {
  printf '# ---- provenance ----\n'
  printf '# command: %s\n' "$1"
  printf '# exit_code: %s\n' "$2"
  printf '# utc_timestamp: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '# hostname: %s\n' "$(hostname)"
  printf '# uname: %s\n' "$(uname -a)"
  printf '# rustc --version -v:\n'
  rustc --version -v 2>&1 | sed 's/^/#   /'
  printf '# cargo_version: %s\n' "$(cargo --version 2>&1)"
  printf '# git_head: %s\n' "$(git rev-parse HEAD 2>/dev/null || echo unknown)"
  printf '# source_tree_sha256: %s\n' "$(source_tree_sha256)"
  printf '# git_status_short:\n'
  git status --short 2>/dev/null | sed 's/^/#   /'
  printf '# ---- output ----\n'
}

# Deterministic identity of the exact phase-2 code revision (the tree is
# untracked during the spike, so `git rev-parse HEAD` is the BASE commit, not
# the code revision). Matches report::source_tree_sha256() and the recipe in
# README.md: sha256 over the sorted, newline-joined `relative_path:file_sha256`
# list for the phase-2 Cargo.toml, Cargo.lock, scripts/run_all.sh and src/**.
source_tree_sha256() {
  (
    cd "$REPO_ROOT" || return 1
    {
      for f in \
        spikes/terminal-engine-phase2/Cargo.toml \
        spikes/terminal-engine-phase2/Cargo.lock \
        spikes/terminal-engine-phase2/scripts/run_all.sh; do
        [ -f "$f" ] || continue
        printf '%s:%s\n' "$f" "$(sha256sum -- "$f" | cut -d' ' -f1)"
      done
      find spikes/terminal-engine-phase2/src -type f -print0 \
        | xargs -0 -r sha256sum \
        | while read -r sum path; do printf '%s:%s\n' "$path" "$sum"; done
    } | LC_ALL=C sort | sha256sum | cut -d' ' -f1
  )
}

# Run a command, capture stdout+stderr with a provenance header and the real
# exit code; never abort the run.
run_gate() {
  local out="$1"; shift
  local tmp rc
  tmp="$(mktemp)"
  "$@" >"$tmp" 2>&1
  rc=$?
  {
    provenance "$*" "$rc"
    cat "$tmp"
  } > "$out"
  rm -f "$tmp"
  printf '  %-22s exit=%d  -> %s\n' "$(basename "$out" .txt)" "$rc" "${out#"$CRATE_DIR"/}"
  return "$rc"
}

capture() {
  local name="$1"; shift
  local summary="$STREAMS/$name.summary.txt"
  echo "### capture $name"
  if "$CAPTURE_BIN" --name "$name" --out "$STREAMS" "$@" >"$summary" 2>&1; then
    grep -E '^BYTES_RECEIVED=|^VERDICT=' "$summary" | sed 's/^/    /'
    return 0
  fi
  echo "    capture failed (see streams/$name.summary.txt)"
  CAPTURE_FAILURES=$((CAPTURE_FAILURES + 1))
  return 0
}

# --- 1. Provenance stamp ---------------------------------------------------
{
  echo "# Relayable TERM-002 phase B — run_all.sh provenance"
  echo "# utc_timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "# hostname: $(hostname)"
  echo "# uname: $(uname -a)"
  echo "# TERM=${TERM:-<unset>}"
  echo "# COLORTERM=${COLORTERM:-<unset>}"
  echo "# rustc_version_v:"
  rustc --version -v 2>&1 | sed 's/^/  /'
  echo "# cargo_version: $(cargo --version 2>&1)"
  echo "# git_head: $(git rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "# git_head_note: git_head is the BASE commit the spike branch was cut from; the phase-2 code is untracked during the spike, so git_head is NOT the code revision. Use source_tree_sha256 for the code revision."
  echo "# source_tree_sha256: $(source_tree_sha256)"
  echo "# source_tree_sha256_note: sha256 over the sorted newline-joined '<relative_path>:<file_sha256>' list for spikes/terminal-engine-phase2/{Cargo.toml,Cargo.lock,src/**,scripts/run_all.sh}; recompute with the recipe in README.md and report::source_tree_sha256()."
  echo "# git_status_short:"
  git status --short 2>/dev/null | sed 's/^/  /'
  echo "# tool_paths:"
  for t in nvim vim htop tmux less top ls seq printf sh stty python3; do
    printf '#   %s=%s\n' "$t" "$(command -v "$t" 2>/dev/null || echo '(absent)')"
  done
  echo "# nvim_version_head1: $(nvim --version 2>/dev/null | head -1 || echo '(absent)')"
  echo "# tmux_version: $(tmux -V 2>/dev/null || echo '(absent)')"
  echo "# vim_presence: $(command -v vim >/dev/null 2>&1 && echo present || echo absent) (nvim $(nvim --version 2>/dev/null | head -1) is used instead)"
  echo "# htop_presence: $(command -v htop >/dev/null 2>&1 && echo present || echo absent)"
  echo "# pinned_crates (from Cargo.lock):"
  for c in portable-pty alacritty_terminal vte vt100 termwiz avt; do
    v="$(awk -v c="$c" '$1=="name" && $3=="\""c"\"" {getline; print $3}' Cargo.lock | tr -d '"' | head -1)"
    printf '#   %s=%s\n' "$c" "${v:-<not-found>}"
  done
} > "$META"

# --- 2. Build the harness binaries -----------------------------------------
echo "### cargo build (harness binaries)"
cargo build >"$CHECKS/_harness_build.txt" 2>&1
BUILD_RC=$?
{
  provenance "cargo build (harness binaries, before capture)" "$BUILD_RC"
  cat "$CHECKS/_harness_build.txt"
} > "$CHECKS/_harness_build.tmp"
mv "$CHECKS/_harness_build.tmp" "$CHECKS/_harness_build.txt"
echo "  cargo build exit=$BUILD_RC"
if [ "$BUILD_RC" -ne 0 ] || [ ! -x "$CAPTURE_BIN" ] || [ ! -x "$ANALYZE_BIN" ]; then
  echo "FATAL: harness binaries did not build; see results/raw/checks/_harness_build.txt"
  exit 2
fi

# --- 3. Real-program PTY captures ------------------------------------------
CAPTURE_FAILURES=0

capture bell --duration-ms 1500 --no-version -- /bin/sh -c 'printf "\a"'

capture osc8_ls --duration-ms 2000 --no-version -- \
  ls --hyperlink=always --color=never -1 "$FIXTURES/hyperlink_dir"

if command -v nvim >/dev/null 2>&1; then
  NVIM_PRESENT=1
  capture nvim_alt --duration-ms 6000 --send '2500::qa!\r' --no-version -- nvim --clean
  capture nvim_title --duration-ms 6000 --send '2500::qa!\r' --no-version -- \
    nvim --clean '+set title'
  capture nvim_mouse --duration-ms 6000 --send '2500::qa!\r' --no-version -- \
    nvim --clean '+set mouse=a'
else
  NVIM_PRESENT=0
  echo "### nvim absent: skipping nvim_alt / nvim_title / nvim_mouse (documented skip; analyze records NOT_TESTED)"
fi

# tmux is present on this host; htop and vim are not (recorded in
# run_all_meta.txt). tmux gets a real capture so the real-program evidence set
# covers a second full-screen program (§5.4 multiplexers). It uses a PRIVATE
# socket only (`-L relayable-term002-spike`), so the user's default tmux server
# is never touched, and `-f /dev/null` avoids reading the user's tmux config.
# The pane prints a deterministic marker that the `tmux_real` check asserts on
# the engine's alternate-screen snapshot. The private server is killed and its
# absence verified immediately below, with the exact commands and real exit
# codes recorded in results/raw/streams/tmux.cleanup.txt.
TMUX_SOCKET="relayable-term002-spike"
TMUX_PRESENT=0
if command -v tmux >/dev/null 2>&1; then
  TMUX_PRESENT=1
  capture tmux --duration-ms 3000 --no-version -- \
    tmux -L "$TMUX_SOCKET" -f /dev/null new-session \
    'printf "RELAYABLE_TMUX_MARKER\n"; sleep 4'
else
  echo "### tmux absent: skipping tmux capture (documented; analyze records NOT_TESTED)"
fi

# --- 3b. tmux private-server cleanup (N3) ----------------------------------
# Never leave a private tmux server behind, and prove it with real exit codes.
TMUX_CLEANUP="$STREAMS/tmux.cleanup.txt"
{
  echo "# tmux private-socket cleanup"
  echo "socket=$TMUX_SOCKET"
  echo "user_default_socket_untouched=1 (only -L $TMUX_SOCKET was ever used)"
  if command -v tmux >/dev/null 2>&1; then
    tmux -L "$TMUX_SOCKET" kill-server
    kill_rc=$?
    echo "command=tmux -L $TMUX_SOCKET kill-server"
    echo "kill_server_exit=$kill_rc (non-zero is expected when the server already exited)"
    tmux -L "$TMUX_SOCKET" list-sessions >/dev/null 2>&1
    list_rc=$?
    echo "command=tmux -L $TMUX_SOCKET list-sessions"
    echo "verify_no_server_exit=$list_rc (0 = a server is still running; non-zero = none)"
    if [ "$list_rc" -eq 0 ]; then
      echo "leftover_server=YES"
    else
      echo "leftover_server=none"
    fi
  else
    echo "tmux_absent=1 (cleanup skipped; no private server was created)"
  fi
  echo "utc_timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$TMUX_CLEANUP"
cat "$TMUX_CLEANUP"

# Application-cursor-key round trip. `/bin/sh -c 'read ...'` keeps the tty in
# canonical mode, so the injected ESC[A is echoed by the line discipline
# (^[[A) and also reaches the child (printed back as GOT:<raw>).
capture cursor_keys --duration-ms 2000 --send '500:\x1b[A' --send '900:\r' \
  --no-version -- /bin/sh -c 'read x; printf "GOT:%s\n" "$x"'

# PTY resize / SIGWINCH round trip. NOTE: the literal brief command used
# `sleep 5`; on this host /bin/sh is bash, and a WINCH trap that arrives during
# a single long foreground `sleep` is not run until the sleep exits. Replacing
# it with `while :; do sleep 0.1; done` makes the trap run promptly while still
# using the same `trap "stty size" WINCH` mechanism. Pre size = 80x24,
# post size = 100x30.
capture resize_winch --duration-ms 3000 --resize-ms 1000:30x100 --no-version -- \
  /bin/sh -c 'trap "echo WINCHED; stty size" WINCH; echo READY; while :; do sleep 0.1; done'

capture seq_20000 --duration-ms 3000 --no-version -- seq 1 20000

# Unicode real leg (UTF-8 CJK + combining acute + emoji). $'...' resolves the
# \u0301 escape into the real combining code point and \\n into a printf escape.
UNI_ARG=$'printf "%s\\n" "寬字元 日本語 e\u0301 😀"'
capture unicode --duration-ms 1500 --no-version -- /bin/sh -c "$UNI_ARG"

# --- 4. Per-engine ISOLATED throughput/memory observation (D4) -------------
# Each engine is timed in its OWN process so RSS is not cross-contaminated.
# Each run is a real analyze invocation; results are merged by step 4b.
echo "### per-engine isolated perf observation"
PERF_RC=0
for eng in alacritty_terminal vt100 avt termwiz; do
  run_gate "$CHECKS/_perf_${eng}.txt" \
    "$ANALYZE_BIN" --perf-only "$eng" --results-dir "$RAW" --perf-mib 8 --perf-runs 3
  rc=$?
  if [ "$rc" -ne 0 ]; then PERF_RC=$rc; fi
done

# --- 5. Analyze (fixtures + replays + checks + merged perf observation) ----
echo "### analyze"
"$ANALYZE_BIN" --results-dir "$RAW" --perf --perf-mib 8 >"$CHECKS/_analyze_stdout.txt" 2>&1
ANALYZE_RC=$?
{
  provenance "analyze --results-dir results/raw --perf --perf-mib 8" "$ANALYZE_RC"
  cat "$CHECKS/_analyze_stdout.txt"
} > "$CHECKS/_analyze_stdout.tmp"
mv "$CHECKS/_analyze_stdout.tmp" "$CHECKS/_analyze_stdout.txt"
sed -n 's/^/  /p' "$CHECKS/_analyze_stdout.txt" | tail -40
echo "  analyze exit=$ANALYZE_RC"

# --- 4b. Comparison matrix (per-check x per-engine) -------------------------
# Derived deterministically from summary.json; never hand-written. This is the
# artifact the phase-C comparison leg is reviewed from.
echo "### comparison matrix"
MATRIX_RC=0
if [ -f "$RAW/summary.json" ]; then
  python3 - "$RAW/summary.json" "$COMPARE" <<'PY'
import json
import sys

summary_path, out_dir = sys.argv[1], sys.argv[2]
with open(summary_path, encoding="utf-8") as fh:
    summary = json.load(fh)

engines = []
for check in summary.get("checks", []):
    name = check["engine"]
    if name not in [e["name"] for e in engines]:
        engines.append({"name": name, "version": check.get("engine_version", "?")})

checks = []
order = []
for check in summary.get("checks", []):
    cid = check["check"]
    if cid not in order:
        order.append(cid)
        checks.append({"check": cid, "kind": check.get("kind", "?"), "statuses": {}})
    next(c for c in checks if c["check"] == cid)["statuses"][check["engine"]] = check["status"]

with open(f"{out_dir}/matrix.json", "w", encoding="utf-8") as fh:
    json.dump(
        {"source": "results/raw/summary.json", "engines": engines, "checks": checks},
        fh,
        indent=2,
        sort_keys=False,
    )
    fh.write("\n")

check_w = max([len(c["check"]) for c in checks] + [5])
kind_w = max([len(c["kind"]) for c in checks] + [4])
eng_w = {e["name"]: max(len(e["name"]), 6) for e in engines}
header = "check".ljust(check_w) + "  " + "kind".ljust(kind_w) + "  " + "  ".join(
    e["name"].ljust(eng_w[e["name"]]) for e in engines
)
lines = [
    "Relayable TERM-002 phase C - comparison matrix (per-check x per-engine)",
    "generated from results/raw/summary.json by scripts/run_all.sh",
    "",
    "engine versions:",
]
lines += [f"  {e['name'].ljust(20)} {e['version']}" for e in engines]
lines += ["", header, "-" * len(header)]
for c in checks:
    row = c["check"].ljust(check_w) + "  " + c["kind"].ljust(kind_w) + "  "
    row += "  ".join(
        str(c["statuses"].get(e["name"], "-")).ljust(eng_w[e["name"]]) for e in engines
    )
    lines.append(row.rstrip())
with open(f"{out_dir}/matrix.txt", "w", encoding="utf-8") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"wrote {out_dir}/matrix.txt and {out_dir}/matrix.json")
PY
  MATRIX_RC=$?
else
  echo "WARN: summary.json missing; comparison matrix not generated"
  MATRIX_RC=1
fi
echo "  matrix generation exit=$MATRIX_RC"

# --- 5b. Gating-engine NOT_TESTED check (D10) ------------------------------
# A missing capture must not silently yield OVERALL=PASS. Any NOT_TESTED on the
# gating engine (alacritty_terminal) fails the gate, except the documented
# nvim-absence skip when nvim is genuinely absent.
echo "### gating-engine NOT_TESTED check"
GATE_NOT_TESTED_RC=0
GATE_ALL=""
GATE_UNDOCUMENTED=""
if [ -f "$RAW/summary.json" ]; then
  GATE_OUT="$(python3 - "$RAW/summary.json" "${NVIM_PRESENT:-0}" "${TMUX_PRESENT:-0}" <<'PY'
import json, sys
summary = json.load(open(sys.argv[1], encoding="utf-8"))
nvim_present = sys.argv[2] == "1"
tmux_present = sys.argv[3] == "1"
documented = {"alt_screen_real", "osc_title_real", "mouse_modes_real"}
if not tmux_present:
    documented.add("tmux_real")
names = sorted({
    c["check"] for c in summary.get("checks", [])
    if c["engine"] == "alacritty_terminal" and c["status"] == "NOT_TESTED"
})
undocumented = [n for n in names if nvim_present or n not in documented]
print("all=" + ",".join(names))
print("undocumented=" + ",".join(undocumented))
PY
)"
  GATE_ALL="$(printf '%s\n' "$GATE_OUT" | sed -n 's/^all=//p')"
  GATE_UNDOCUMENTED="$(printf '%s\n' "$GATE_OUT" | sed -n 's/^undocumented=//p')"
else
  GATE_UNDOCUMENTED="summary.json-missing"
  GATE_ALL="summary.json-missing"
fi
if [ -n "$GATE_UNDOCUMENTED" ]; then
  echo "GATE FAILURE: gating engine alacritty_terminal has NOT_TESTED checks: $GATE_UNDOCUMENTED"
  echo "GATE FAILURE: a missing capture must not silently yield OVERALL=PASS"
  GATE_NOT_TESTED_RC=1
elif [ -n "$GATE_ALL" ]; then
  echo "gate note: documented nvim-absence skip for alacritty_terminal: $GATE_ALL"
fi
{
  echo "gating_engine_not_tested_all=$GATE_ALL"
  echo "gating_engine_not_tested_undocumented=$GATE_UNDOCUMENTED"
  echo "gating_engine_not_tested_gate_exit=$GATE_NOT_TESTED_RC"
  echo "nvim_present=$NVIM_PRESENT"
  echo "vim_present=$(command -v vim >/dev/null 2>&1 && echo 1 || echo 0)"
  echo "htop_present=$(command -v htop >/dev/null 2>&1 && echo 1 || echo 0)"
  echo "tmux_present=$(command -v tmux >/dev/null 2>&1 && echo 1 || echo 0)"
} >> "$META"

# --- 5. Quality gates (must all be exit 0) ---------------------------------
echo "### quality gates"
run_gate "$CHECKS/cargo_fmt_check.txt" cargo fmt --check
GATE_FMT_RC=$?
run_gate "$CHECKS/cargo_build.txt" cargo build
GATE_BUILD_RC=$?
run_gate "$CHECKS/cargo_clippy.txt" cargo clippy --all-targets
GATE_CLIPPY_RC=$?
run_gate "$CHECKS/cargo_test.txt" cargo test
GATE_TEST_RC=$?

# --- 6. End-of-run summary --------------------------------------------------
{
  echo ""
  echo "# ---- end of run summary ----"
  echo "capture_failures=$CAPTURE_FAILURES"
  echo "analyze_exit=$ANALYZE_RC"
  echo "perf_isolated_exit=$PERF_RC"
  echo "matrix_generation_exit=$MATRIX_RC"
  echo "gating_engine_not_tested_gate_exit=$GATE_NOT_TESTED_RC"
  echo "cargo_fmt_check_exit=$GATE_FMT_RC"
  echo "cargo_build_exit=$GATE_BUILD_RC"
  echo "cargo_clippy_exit=$GATE_CLIPPY_RC"
  echo "cargo_test_exit=$GATE_TEST_RC"
  echo "summary_txt=$RAW/summary.txt"
  echo "summary_json=$RAW/summary.json"
} >> "$META"

echo
echo "=================== FINAL TALLY ==================="
if [ -f "$RAW/summary.txt" ]; then
  sed -n '/^Tally:/,/^Per-engine tally:/p' "$RAW/summary.txt"
fi
echo "capture_failures=$CAPTURE_FAILURES"
echo "analyze_exit=$ANALYZE_RC (0 = no engine check FAILed and self-check clean)"
echo "perf_isolated_exit=$PERF_RC"
echo "matrix_generation_exit=$MATRIX_RC"
echo "gating_engine_not_tested_gate_exit=$GATE_NOT_TESTED_RC (0 = no undocumented NOT_TESTED on alacritty_terminal)"
echo "cargo_fmt_check_exit=$GATE_FMT_RC"
echo "cargo_build_exit=$GATE_BUILD_RC"
echo "cargo_clippy_exit=$GATE_CLIPPY_RC"
echo "cargo_test_exit=$GATE_TEST_RC"
echo "run_all_exit_stamp=$EXIT_STAMP"
echo "==================================================="

if [ "$ANALYZE_RC" -ne 0 ] || [ "$MATRIX_RC" -ne 0 ] || [ "$CAPTURE_FAILURES" -ne 0 ] || [ "$PERF_RC" -ne 0 ] \
  || [ "$GATE_NOT_TESTED_RC" -ne 0 ] \
  || [ "$GATE_FMT_RC" -ne 0 ] || [ "$GATE_BUILD_RC" -ne 0 ] || [ "$GATE_CLIPPY_RC" -ne 0 ] \
  || [ "$GATE_TEST_RC" -ne 0 ]; then
  echo "OVERALL=FAIL"
  exit 1
fi
echo "OVERALL=PASS"
exit 0
