#!/usr/bin/env bash
# Relayable terminal-engine spike driver (TERM-001 / ADR-003, phase 1).
#
# Idempotent: it deletes and regenerates results/raw/ on every run.
# Writes raw output with a provenance header on every result file.
#
# Usage: spikes/terminal-engine/scripts/run.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$CRATE_DIR"

RAW="$CRATE_DIR/results/raw"
INPUTS="$CRATE_DIR/results/inputs"
CHECKS="$RAW/checks"

mkdir -p "$CRATE_DIR/results"
rm -rf "$RAW"
mkdir -p "$RAW" "$CHECKS" "$INPUTS"

FAILURES=0

provenance_header() {
  local cmd="$1" rc="$2"
  printf '# ---- provenance ----\n'
  printf '# command: %s\n' "$cmd"
  printf '# exit_code: %s\n' "$rc"
  printf '# utc_timestamp: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '# git_head: %s\n' "$(git rev-parse HEAD 2>/dev/null || echo unknown)"
  printf '# rustc: %s\n' "$(rustc --version 2>&1)"
  printf '# cargo: %s\n' "$(cargo --version 2>&1)"
  printf '# host: %s\n' "$(uname -srm)"
  printf '# os: %s\n' "$(grep -m1 '^PRETTY_NAME=' /etc/os-release 2>/dev/null | cut -d= -f2- | tr -d '"' || echo unknown)"
  printf '# ---- output ----\n'
}

# Run a command, capture stdout+stderr with a provenance header, record the
# exact exit code, and never abort the whole run on a single failure.
run_capture() {
  local out="$1"; shift
  local tmp
  tmp="$(mktemp)"
  printf '\n$ %s\n' "$*"
  set +e
  "$@" >"$tmp" 2>&1
  local rc=$?
  set -e
  {
    provenance_header "$*" "$rc"
    cat "$tmp"
  } > "$out"
  rm -f "$tmp"
  printf '%s (exit %d)\n' "--> ${out#"$CRATE_DIR"/}" "$rc"
  if [ "$rc" -ne 0 ]; then
    FAILURES=$((FAILURES + 1))
  fi
  return 0
}

stamp_text() {
  local f="$1"
  [ -f "$f" ] || return 0
  local tmp
  tmp="$(mktemp)"
  provenance_header "payload file written by a spike binary listed in results/raw/*.summary.txt" 0 > "$tmp"
  cat "$f" >> "$tmp"
  mv "$tmp" "$f"
}

stamp_bin_meta() {
  local f="$1"
  provenance_header "exact raw bytes captured by capture_stream; see ${f%.bin}.summary.txt" 0 > "$f.meta"
}

# --- 1. Tool availability (real command -v output before any capture) -------
printf '\n$ tool availability + versions (shell block)\n'
{
  provenance_header "tool availability + versions (shell block)" 0
  echo "## command -v results"
  for t in nvim vim less top htop tmux stty tput python3 busybox nano script ps seq mktemp; do
    printf '%s: ' "$t"
    command -v "$t" || echo "(absent)"
  done
  echo
  echo "## versions"
  for c in "nvim --version" "less --version" "top --version" "stty --version" "tput -V" "python3 --version"; do
    echo "\$ $c"
    set +e
    sh -c "$c" 2>&1 | head -n 3
    set -e
  done
} > "$RAW/tools_available.txt"
printf '%s\n' '--> results/raw/tools_available.txt'

# --- 2. Build and static checks --------------------------------------------
run_capture "$CHECKS/cargo_build.txt" cargo build
run_capture "$CHECKS/cargo_fmt_check.txt" cargo fmt --check
run_capture "$CHECKS/cargo_clippy.txt" cargo clippy --all-targets

# --- 3. Detector self-test (makes NOT_OBSERVED claims meaningful) ----------
run_capture "$RAW/esc_selftest.txt" ./target/debug/esc_selftest

# --- 4. PTY size / resize / SIGWINCH ----------------------------------------
run_capture "$RAW/pty_size.txt" ./target/debug/pty_size

# --- 5. Raw full-screen program captures ------------------------------------
# Generated input for `less` so it has more than one screen of content.
seq 1 400 > "$INPUTS/less_input.txt"
printf 'generated %s (%s lines)\n' "results/inputs/less_input.txt" "$(wc -l < "$INPUTS/less_input.txt")"

run_capture "$RAW/nvim.summary.txt" ./target/debug/capture_stream \
  --name nvim --out "$RAW" --timeout-ms 8000 --send '2000::qa!\r' -- nvim --clean

run_capture "$RAW/nvim_mouse.summary.txt" ./target/debug/capture_stream \
  --name nvim_mouse --out "$RAW" --timeout-ms 8000 --send '2000::qa!\r' -- nvim --clean '+set mouse=a'

# Title output is opt-in in Neovim; enable it to observe OSC 0/2 on the wire.
run_capture "$RAW/nvim_title.summary.txt" ./target/debug/capture_stream \
  --name nvim_title --out "$RAW" --timeout-ms 6000 --send '2000::qa!\r' -- nvim --clean '+set title'

run_capture "$RAW/less.summary.txt" ./target/debug/capture_stream \
  --name less --out "$RAW" --timeout-ms 4000 --send '1500:q' -- less "$INPUTS/less_input.txt"

run_capture "$RAW/top.summary.txt" ./target/debug/capture_stream \
  --name top --out "$RAW" --timeout-ms 5000 --send '2000:q' -- top -d 1

# OSC 8 hyperlinks: no full-screen program on this host emits them, so use
# coreutils `ls --hyperlink=always` (a real program) as the OSC 8 probe.
mkdir -p "$INPUTS/hyperlink_dir"
printf 'alpha\n' > "$INPUTS/hyperlink_dir/alpha.txt"
printf 'beta\n' > "$INPUTS/hyperlink_dir/beta.txt"
run_capture "$RAW/osc8_ls.summary.txt" ./target/debug/capture_stream \
  --name osc8_ls --out "$RAW" --timeout-ms 3000 -- ls --hyperlink=always --color=never -1 "$INPUTS/hyperlink_dir"

# --- 6. Scrollback / RSS observations (64 MiB payload each) -----------------
run_capture "$RAW/scrollback_discard.txt" \
  ./target/debug/scrollback_mem --mib 64 --retain discard --interval-ms 25
run_capture "$RAW/scrollback_unbounded.txt" \
  ./target/debug/scrollback_mem --mib 64 --retain unbounded --interval-ms 25
run_capture "$RAW/scrollback_bounded.txt" \
  ./target/debug/scrollback_mem --mib 64 --retain bounded --cap-lines 1000 --interval-ms 25

# --- 7. Provenance stamping for binary/text payload files -------------------
shopt -s nullglob
for f in "$RAW"/*.esc "$RAW"/*.seq; do
  stamp_text "$f"
done
for f in "$RAW"/*.bin; do
  stamp_bin_meta "$f"
done
shopt -u nullglob

# --- 8. Run summary ---------------------------------------------------------
OVERALL_RC=$([ "$FAILURES" -eq 0 ] && echo 0 || echo 1)
{
  provenance_header "./scripts/run.sh (aggregate summary)" "$OVERALL_RC"
  printf 'aggregate_failures=%d\n' "$FAILURES"
  printf 'raw_dir=%s\n' "$RAW"
  printf 'files:\n'
  find "$RAW" -maxdepth 2 -type f | sort | sed "s#$CRATE_DIR/##"
} > "$RAW/run_meta.txt"

printf '\nrun.sh complete: aggregate_failures=%d\n' "$FAILURES"
if [ "$FAILURES" -ne 0 ]; then
  exit 1
fi
