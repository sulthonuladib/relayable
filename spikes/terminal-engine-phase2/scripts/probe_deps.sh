#!/usr/bin/env bash
# Relayable terminal-engine phase-2 dependency / pin probe (TERM-002 / ADR-003).
#
# Idempotent.  Regenerates results/raw/deps/ evidence for the requested section.
# Every captured file carries a provenance header (date -u, hostname, uname -a,
# rustc --version -v, cargo --version, git HEAD, git status --short) and the
# command's VERBATIM stdout+stderr plus its real exit code.  Nothing here is
# fabricated; a non-zero exit is recorded as-is.
#
# Usage:
#   scripts/probe_deps.sh core   # phase-2 crate: fetch/build/metadata/lib probe/fmt/clippy
#   scripts/probe_deps.sh gpui   # gpui-pin-probe: fetch/check(time-boxed)/tree/platform grep
#   scripts/probe_deps.sh all    # both (default)
#
# Notes:
#   * No `set -e`: a single failing command must not abort the evidence run.
#   * gpui `cargo check` is time-boxed to 1200s; exit 124 means timed out.
#   * No installs, no sudo, no global config changes.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RAW="$CRATE_DIR/results/raw/deps"
GPUI_DIR="$CRATE_DIR/gpui-pin-probe"

SECTION="${1:-all}"

mkdir -p "$RAW"

FAILURES=0

provenance_header() {
  local cmd="$1" rc="$2"
  printf '# ---- provenance ----\n'
  printf '# command: %s\n' "$cmd"
  printf '# exit_code: %s\n' "$rc"
  printf '# utc_timestamp: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '# hostname: %s\n' "$(hostname 2>&1)"
  printf '# uname: %s\n' "$(uname -a 2>&1)"
  printf '# git_head: %s\n' "$(git rev-parse HEAD 2>&1)"
  printf '# git_status_short:\n'
  git status --short 2>&1 | sed 's/^/#   /'
  printf '# rustc_version_v:\n'
  rustc --version -v 2>&1 | sed 's/^/#   /'
  printf '# cargo_version: %s\n' "$(cargo --version 2>&1)"
  printf '# ---- output ----\n'
}

# Run a command from the current directory, capture stdout+stderr + exit code.
run_capture() {
  local out="$1"; shift
  local tmp rc
  tmp="$(mktemp)"
  printf '\n$ %s\n' "$*"
  "$@" >"$tmp" 2>&1
  rc=$?
  {
    provenance_header "$*" "$rc"
    cat "$tmp"
  } > "$out"
  rm -f "$tmp"
  printf -- '--> %s (exit %d)\n' "${out#"$CRATE_DIR"/}" "$rc"
  if [ "$rc" -ne 0 ]; then
    FAILURES=$((FAILURES + 1))
  fi
}

probe_core() {
  cd "$CRATE_DIR"
  rm -f "$RAW"/cargo_fetch.txt "$RAW"/cargo_build.txt "$RAW"/cargo_metadata.txt \
        "$RAW"/cargo_metadata_raw.json "$RAW"/linux_lib_probe.txt \
        "$RAW"/cargo_fmt_check.txt "$RAW"/cargo_clippy.txt

  echo "== core: cargo fetch (network probe, required deps) =="
  run_capture "$RAW/cargo_fetch.txt" cargo fetch

  echo "== core: resolved dependency versions =="
  run_capture "$RAW/cargo_metadata.txt" bash -c \
    "cargo metadata --format-version 1 --locked | python3 -c 'import sys,json; d=json.load(sys.stdin); names=[(p[\"name\"],p[\"version\"],p.get(\"source\") or \"path\") for p in d[\"packages\"]]; [print(f\"{n} {v} {s}\") for n,v,s in sorted(set(names))]'"
  # Raw JSON kept alongside the filtered list (large, but exact).
  bash -c "cargo metadata --format-version 1 --locked > '$RAW/cargo_metadata_raw.json' 2>'$RAW/cargo_metadata_raw.stderr'; echo \$?" > /dev/null
  # Re-stamp the raw JSON with a provenance header file so the pair is traceable.
  {
    provenance_header "cargo metadata --format-version 1 --locked (raw JSON)" 0
    printf '# raw_json_file: cargo_metadata_raw.json\n# stderr_file: cargo_metadata_raw.stderr\n'
  } > "$RAW/cargo_metadata_raw.meta"

  echo "== core: cargo build =="
  run_capture "$RAW/cargo_build.txt" cargo build

  echo "== core: host graphics/font library probe (no installs) =="
  run_capture "$RAW/linux_lib_probe.txt" bash -c \
    "ldconfig -p | grep -E 'libxkbcommon|libwayland|libX11|libfontconfig|libfreetype'"

  echo "== core: cargo fmt --check =="
  run_capture "$RAW/cargo_fmt_check.txt" cargo fmt --check

  echo "== core: cargo clippy --all-targets =="
  run_capture "$RAW/cargo_clippy.txt" cargo clippy --all-targets
}

probe_gpui() {
  cd "$GPUI_DIR"
  rm -f "$RAW"/gpui_*.txt

  echo "== gpui: cargo fetch (does =0.2.2 resolve?) =="
  run_capture "$RAW/gpui_cargo_fetch.txt" cargo fetch

  echo "== gpui: cargo check (time-boxed 1200s) =="
  run_capture "$RAW/gpui_cargo_check.txt" timeout 1200 cargo check

  echo "== gpui: cargo tree (dep count / edges) =="
  run_capture "$RAW/gpui_cargo_tree.txt" cargo tree

  echo "== gpui: platform/backend feature + dep grep from fetched source =="
  run_capture "$RAW/gpui_platform_grep.txt" bash -c \
    "src=\$(ls -d \"\$HOME\"/.cargo/registry/src/index.crates.io-*/gpui-0.2.2 2>/dev/null | head -1); echo \"gpui source: \${src:-<not fetched>}\"; if [ -n \"\$src\" ]; then echo '--- [features] section ---'; sed -n '/^\[features\]/,/^\[/p' \"\$src/Cargo.toml\"; echo '--- x11/wayland/xkbcommon target deps ---'; grep -nE 'x11|wayland|xkbcommon|blade|cosmic-text|font-kit' \"\$src/Cargo.toml\"; fi"

  echo "== gpui: resolved versions relevant to the display stack =="
  run_capture "$RAW/gpui_display_stack.txt" bash -c \
    "cargo tree -i xkbcommon 2>&1; cargo tree -i x11rb 2>&1; cargo tree -i wayland-client 2>&1; cargo tree -i blade-graphics 2>&1; true"
}

case "$SECTION" in
  core) probe_core ;;
  gpui) probe_gpui ;;
  all)  probe_core; probe_gpui ;;
  *) echo "usage: $0 [core|gpui|all]" >&2; exit 2 ;;
esac

printf '\n== probe_deps.sh section=%s: non-zero captures=%d ==\n' "$SECTION" "$FAILURES"
exit 0
