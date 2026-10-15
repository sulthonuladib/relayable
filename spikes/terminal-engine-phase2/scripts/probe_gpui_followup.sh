#!/usr/bin/env bash
# Relayable TERM-002 / ADR-003 gpui =0.2.2 compile-blocker follow-up probe.
#
# Purpose: characterize the xattr/libc ENOATTR compile failure precisely enough
# that the ADR-003 pin verdict and the UI-001 handoff are accurate.  READ-ONLY
# w.r.t. source: this script never edits source crates, never edits a version
# pin, never adds a [patch], and never mutates the canonical Cargo.lock.
#
# Attempts (a)-(c) run on the canonical gpui-pin-probe manifest with --locked
# (which provably leaves the canonical lock byte-identical).  Feature variants
# run in throwaway workspaces under gpui-pin-probe/feature-matrix/<name>/, each
# with its OWN Cargo.lock, so the canonical lock is never touched.  update
# checks run with --dry-run only.
#
# Every captured file carries a provenance header (date -u, hostname, uname -a,
# rustc --version -v, cargo --version, git HEAD, git status --short) plus the
# command's VERBATIM stdout+stderr and its real exit code.  Nothing is
# fabricated; a non-zero exit is recorded as-is.
#
# Usage:
#   scripts/probe_gpui_followup.sh static   # cheap: trees, dry-run updates, registry source evidence
#   scripts/probe_gpui_followup.sh matrix   # time-boxed retry matrix (timeout 900 each)
#   scripts/probe_gpui_followup.sh cold     # one true cold full check in a fresh target dir
#   scripts/probe_gpui_followup.sh all      # static + matrix (default)
#
# No sudo, no installs, no global config changes, no running gpui binaries.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GPUI_DIR="$CRATE_DIR/gpui-pin-probe"
OUT="$CRATE_DIR/results/raw/gpui-followup"
REG="${HOME}/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f"
INDEX_CACHE="${HOME}/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache"

SECTION="${1:-all}"
mkdir -p "$OUT"

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

# Run a command from the current directory, capture stdout+stderr + real exit
# code + wall-clock duration into $1.  Extra provenance lines may be passed via
# the $PROBE_EXTRA env var (e.g. which manifest variant was active).
run_capture() {
  local out="$1"; shift
  local tmp rc start end dur
  tmp="$(mktemp)"
  start="$(date +%s.%N)"
  printf '\n$ %s\n' "$*"
  "$@" >"$tmp" 2>&1
  rc=$?
  end="$(date +%s.%N)"
  dur="$(awk -v a="$start" -v b="$end" 'BEGIN{printf "%.3f", b-a}')"
  {
    provenance_header "$*" "$rc"
    [ -n "${PROBE_EXTRA:-}" ] && printf '# extra: %s\n' "$PROBE_EXTRA"
    printf '# wall_clock_s: %s\n' "$dur"
    printf '# ---- output ----\n'
    cat "$tmp"
  } > "$out"
  rm -f "$tmp"
  printf -- '--> %s (exit %d, %.3fs)\n' "${out#"$CRATE_DIR"/}" "$rc" "$dur"
  if [ "$rc" -ne 0 ]; then
    FAILURES=$((FAILURES + 1))
  fi
}

probe_static() {
  cd "$GPUI_DIR"
  rm -f "$OUT"/tree_*.txt "$OUT"/update_*.txt "$OUT"/search_*.txt \
        "$OUT"/registry_*.txt "$OUT"/lock_*.txt "$OUT"/manifest_*.txt

  echo "== static: exact dependency path to xattr =="
  run_capture "$OUT/tree_invert_xattr.txt" cargo tree -i xattr
  run_capture "$OUT/tree_gpui_depth.txt" cargo tree -e normal,build --prefix depth -p gpui
  run_capture "$OUT/tree_invert_zed_async_tar.txt" cargo tree -i zed-async-tar
  run_capture "$OUT/tree_duplicates.txt" cargo tree -d

  echo "== static: available versions (read-only) =="
  run_capture "$OUT/update_dryrun_xattr.txt" cargo update --dry-run --verbose -p xattr
  run_capture "$OUT/update_dryrun_libc.txt" cargo update --dry-run --verbose -p libc
  run_capture "$OUT/search_xattr.txt" cargo search xattr --limit 20
  run_capture "$OUT/search_gpui.txt" cargo search gpui --limit 20
  run_capture "$OUT/index_xattr_versions.txt" python3 - "$INDEX_CACHE/xa/tt/xattr" <<'PY'
import sys, json
data = open(sys.argv[1], "rb").read()
for part in data.split(b"\x00"):
    i = part.find(b"{")
    if i < 0:
        continue
    try:
        o = json.loads(part[i:])
    except Exception:
        continue
    print("xattr", o.get("vers"), "yanked=", o.get("yanked"),
          "rust_version=", o.get("rust_version"))
PY

  echo "== static: lockfile facts =="
  run_capture "$OUT/lock_entries.txt" bash -c 'cd "$0" && echo "--- xattr lock entry ---" && grep -n -A7 "^name = \"xattr\"" Cargo.lock && echo "--- libc lock entry ---" && grep -n -A4 "^name = \"libc\"" Cargo.lock && echo "--- count of libc entries ---" && grep -c "^name = \"libc\"" Cargo.lock' "$GPUI_DIR"

  echo "== static: registry source evidence =="
  run_capture "$OUT/registry_xattr_sys_mod.txt" cat -n "$REG/xattr-0.2.3/src/sys/mod.rs"
  run_capture "$OUT/registry_xattr_sys_linux_macos_mod.txt" cat -n "$REG/xattr-0.2.3/src/sys/linux_macos/mod.rs"
  run_capture "$OUT/registry_xattr_lib_rs.txt" cat -n "$REG/xattr-0.2.3/src/lib.rs"
  run_capture "$OUT/registry_xattr_manifest.txt" bash -c 'echo "=== $0/Cargo.toml.orig ==="; cat -n "$0/Cargo.toml.orig"; echo; echo "=== $0/Cargo.toml ==="; cat -n "$0/Cargo.toml"' "$REG/xattr-0.2.3"
  run_capture "$OUT/registry_zed_async_tar_manifest.txt" bash -c 'echo "=== $0/Cargo.toml.orig ==="; cat -n "$0/Cargo.toml.orig"; echo; echo "=== $0/Cargo.toml (dep blocks) ==="; grep -n -A3 "xattr\|\[features\]\|^default" "$0/Cargo.toml"' "$REG/zed-async-tar-0.5.0-zed"
  run_capture "$OUT/registry_gpui_features.txt" bash -c 'echo "=== $0/Cargo.toml.orig [features] ==="; sed -n "/^\[features\]/,/^\[dependencies\]/p" "$0/Cargo.toml.orig" | cat -n; echo; echo "=== deps on http_client / async-tar / xattr ==="; grep -n "http_client\|async-tar\|zed-async-tar\|xattr" "$0/Cargo.toml.orig"' "$REG/gpui-0.2.2"
  run_capture "$OUT/registry_gpui_http_client_manifest.txt" bash -c 'echo "=== $0/Cargo.toml.orig ==="; cat -n "$0/Cargo.toml.orig"; echo; echo "=== async-tar dep block (normalized) ==="; grep -n -A3 "async-tar" "$0/Cargo.toml"' "$REG/gpui_http_client-0.2.2"
  run_capture "$OUT/registry_xattr_1_6_1_sys_mod.txt" bash -c 'echo "=== xattr 1.6.1 src/sys/mod.rs (the ENOATTR const is GONE) ==="; cat -n "$0/src/sys/mod.rs"; echo; echo "=== xattr 1.6.1 libc requirement ==="; grep -n "libc" "$0/Cargo.toml.orig"' "$REG/xattr-1.6.1"

  echo "== static: libc ENOATTR across pinned/available versions =="
  run_capture "$OUT/registry_libc_enoattr_matrix.txt" bash -c '
    for V in 0.2.147 0.2.189 0.2.190; do
      L="'"$REG"'/libc-$V"
      echo "======== libc $V ========"
      grep -rn "ENOATTR" "$L/src" 2>/dev/null || echo "(no ENOATTR anywhere under src/)"
      echo
    done
    echo "======== libc 0.2.189 linux definition context ========"
    sed -n "1345,1360p" "'"$REG"'/libc-0.2.189/src/unix/linux_like/linux/mod.rs" | cat -n
    echo
    echo "======== libc 0.2.147 linux definition context ========"
    sed -n "2224,2240p" "'"$REG"'/libc-0.2.147/src/unix/linux_like/linux/mod.rs" | cat -n
  '
  run_capture "$OUT/registry_libc_0_2_190_changelog.txt" bash -c 'echo "=== CHANGELOG head ==="; sed -n "1,4p" "$0/CHANGELOG.md"; echo; echo "=== removal entries ==="; grep -n -iE "enoattr|deprecated|breaking|remov" "$0/CHANGELOG.md" | head -40' "$REG/libc-0.2.190"

  echo "== static: manifest + lock checksums =="
  run_capture "$OUT/manifest_checksums.txt" bash -c 'cd "$0" && sha256sum Cargo.toml Cargo.lock && echo "--- manifest bytes ---" && cat -n Cargo.toml' "$GPUI_DIR"
}

# --- matrix -----------------------------------------------------------------
# The probe's root package has NO features, and `-p gpui --features ...` is
# rejected by cargo ("cannot specify features for packages outside of
# workspace"), so gpui feature selection REQUIRES a manifest that sets them.
# Editing the canonical manifest makes cargo want to rewrite the canonical
# Cargo.lock (forbidden), so feature variants live in their own throwaway
# workspaces under gpui-pin-probe/feature-matrix/<name>/, each with its own
# Cargo.lock.  Attempts (a)-(c) run on the canonical manifest with --locked.
VARIANT_ROOT="$GPUI_DIR/feature-matrix"

matrix_attempt() {
  local name="$1"; shift
  local wd="${MATRIX_CWD:-$GPUI_DIR}"
  local out="$OUT/matrix_${name}.txt"
  local tmp rc start end dur comp checking lasterr
  tmp="$(mktemp)"
  start="$(date +%s.%N)"
  ( cd "$wd" && timeout 900 "$@" ) >"$tmp" 2>&1
  rc=$?
  end="$(date +%s.%N)"
  dur="$(awk -v a="$start" -v b="$end" 'BEGIN{printf "%.1f", b-a}')"
  comp="$(grep -cE '^[[:space:]]+Compiling ' "$tmp")"
  checking="$(grep -cE '^[[:space:]]+Checking ' "$tmp")"
  lasterr="$(grep -E '^error' "$tmp" | tail -1)"
  {
    provenance_header "timeout 900 $* (cwd: ${wd#"$CRATE_DIR"/})" "$rc"
    printf '# attempt: %s\n' "$name"
    printf '# manifest_variant: %s\n' "${PROBE_EXTRA:-<canonical gpui = \\\"=0.2.2\\\">}"
    printf '# wall_clock_s: %s\n' "$dur"
    printf '# compiling_lines: %s\n' "$comp"
    printf '# checking_lines: %s\n' "$checking"
    printf '# last_error_line: %s\n' "$lasterr"
    printf '# ---- output ----\n'
    cat "$tmp"
  } > "$out"
  rm -f "$tmp"
  printf -- '== matrix[%s]: exit=%s dur=%ss compiling=%s checking=%s\n' "$name" "$rc" "$dur" "$comp" "$checking"
  printf -- '   last_error: %s\n' "$lasterr"
  printf '%s|%s|%s|%s|%s|%s|%s\n' "$name" "${PROBE_EXTRA:-<canonical>}" "$rc" "$dur" "$comp" "$checking" "$lasterr" >> "$OUT/matrix_summary.txt"
  return 0
}

create_variant() { # $1 = variant name, $2 = gpui dependency table line
  local name="$1" dep="$2" dir="$VARIANT_ROOT/$1"
  mkdir -p "$dir/src"
  cat > "$dir/Cargo.toml" <<EOF
# TERM-002 feature-selection variant probe.  Own workspace + own lockfile; the
# canonical gpui-pin-probe manifest/lock are NOT touched.  Read-only w.r.t.
# source; no [patch]; version pin is the exact ADR-003 pin.
[package]
name = "relayable-gpui-feature-probe-${name}"
version = "0.0.0"
edition = "2021"
publish = false
license = "Apache-2.0"

[dependencies]
${dep}

[workspace]
EOF
  cat > "$dir/src/main.rs" <<'EOF'
fn main() {}
EOF
}

probe_matrix() {
  local before_toml before_lock after_toml after_lock v avoided
  cd "$GPUI_DIR"
  # Share the canonical target dir so already-checked deps are reused across
  # variants (default would be a fresh per-variant target dir).
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$GPUI_DIR/target}"
  before_toml="$(sha256sum Cargo.toml | awk '{print $1}')"
  before_lock="$(sha256sum Cargo.lock | awk '{print $1}')"
  : > "$OUT/matrix_summary.txt"
  printf 'attempt|manifest_variant|exit_code|duration_s|compiling_lines|checking_lines|last_error_line\n' >> "$OUT/matrix_summary.txt"
  {
    printf '# canonical Cargo.toml sha256 before matrix: %s\n' "$before_toml"
    printf '# canonical Cargo.lock sha256 before matrix: %s\n' "$before_lock"
  } > "$OUT/matrix_lock_guard.txt"

  # (a) isolated xattr (canonical manifest; -p selects the dependency, --locked)
  MATRIX_CWD="$GPUI_DIR" PROBE_EXTRA='canonical gpui = "=0.2.2"; -p xattr --locked' \
    matrix_attempt isolated_xattr cargo check --locked -p xattr
  MATRIX_CWD="$GPUI_DIR" PROBE_EXTRA='canonical gpui = "=0.2.2"; -p xattr --no-default-features --locked' \
    matrix_attempt isolated_xattr_no_default_features cargo check --locked -p xattr --no-default-features

  # (b) http client (canonical manifest)
  MATRIX_CWD="$GPUI_DIR" PROBE_EXTRA='canonical gpui = "=0.2.2"; -p gpui_http_client --locked' \
    matrix_attempt gpui_http_client cargo check --locked -p gpui_http_client

  # (c) --no-default-features on the probe root (proves it does NOT touch gpui defaults)
  MATRIX_CWD="$GPUI_DIR" PROBE_EXTRA='canonical gpui = "=0.2.2" (root pkg has no features); --no-default-features --locked' \
    matrix_attempt no_default_features cargo check --locked --no-default-features

  # (d) gpui feature variants in isolated throwaway workspaces
  rm -rf "$VARIANT_ROOT"
  create_variant no_default_gpui 'gpui = { version = "=0.2.2", default-features = false }'
  MATRIX_CWD="$VARIANT_ROOT/no_default_gpui" PROBE_EXTRA='gpui = { version = "=0.2.2", default-features = false }' \
    matrix_attempt no_default_features_gpui cargo check

  create_variant x11_gpui 'gpui = { version = "=0.2.2", default-features = false, features = ["x11"] }'
  MATRIX_CWD="$VARIANT_ROOT/x11_gpui" PROBE_EXTRA='gpui = { version = "=0.2.2", default-features = false, features = ["x11"] }' \
    matrix_attempt x11_only cargo check

  create_variant wayland_gpui 'gpui = { version = "=0.2.2", default-features = false, features = ["wayland"] }'
  MATRIX_CWD="$VARIANT_ROOT/wayland_gpui" PROBE_EXTRA='gpui = { version = "=0.2.2", default-features = false, features = ["wayland"] }' \
    matrix_attempt wayland_only cargo check

  # (e) only if a feature variant avoided the xattr path (static analysis says
  # none can: gpui always depends on gpui_http_client, which always depends on
  # zed-async-tar with its default `xattr` feature).
  avoided=""
  for v in no_default_features_gpui x11_only wayland_only; do
    if ! grep -q 'ENOATTR' "$OUT/matrix_${v}.txt"; then avoided="$v"; break; fi
  done
  if [ -z "$avoided" ]; then
    printf '# (e) skipped: every gpui feature variant still reached and failed at the xattr ENOATTR path.\n' >> "$OUT/matrix_summary.txt"
  else
    MATRIX_CWD="$VARIANT_ROOT/$avoided" PROBE_EXTRA="full --no-default-features check after $avoided avoided xattr" \
      matrix_attempt after_feature_full_check cargo check --no-default-features
  fi

  after_toml="$(sha256sum "$GPUI_DIR/Cargo.toml" | awk '{print $1}')"
  after_lock="$(sha256sum "$GPUI_DIR/Cargo.lock" | awk '{print $1}')"
  {
    printf '# canonical Cargo.toml sha256 after matrix:  %s\n' "$after_toml"
    printf '# canonical Cargo.lock sha256 after matrix:  %s\n' "$after_lock"
    if [ "$before_toml" = "$after_toml" ] && [ "$before_lock" = "$after_lock" ]; then
      printf '# RESULT: canonical manifest and lock UNCHANGED by the matrix.\n'
    else
      printf '# RESULT: WARNING canonical manifest/lock CHANGED by the matrix.\n'
    fi
  } >> "$OUT/matrix_lock_guard.txt"
  printf -- '\n== matrix done; non-zero captures: %d ==\n' "$FAILURES"
}

probe_cold() {
  cd "$GPUI_DIR"
  local before_toml before_lock after_toml after_lock
  before_toml="$(sha256sum Cargo.toml | awk '{print $1}')"
  before_lock="$(sha256sum Cargo.lock | awk '{print $1}')"
  export CARGO_TARGET_DIR="$GPUI_DIR/target-cold"
  rm -rf "$CARGO_TARGET_DIR"
  MATRIX_CWD="$GPUI_DIR" PROBE_EXTRA='canonical gpui = "=0.2.2" --locked; FRESH EMPTY target dir (true cold check)' \
    matrix_attempt cold_full_check cargo check --locked
  rm -rf "$CARGO_TARGET_DIR"
  after_toml="$(sha256sum "$GPUI_DIR/Cargo.toml" | awk '{print $1}')"
  after_lock="$(sha256sum "$GPUI_DIR/Cargo.lock" | awk '{print $1}')"
  {
    printf '# cold check canonical Cargo.toml before/after: %s / %s\n' "$before_toml" "$after_toml"
    printf '# cold check canonical Cargo.lock before/after: %s / %s\n' "$before_lock" "$after_lock"
  } >> "$OUT/matrix_lock_guard.txt"
}

case "$SECTION" in
  static) probe_static ;;
  matrix) probe_matrix ;;
  cold) probe_cold ;;
  all) probe_static; probe_matrix ;;
  *) echo "usage: $0 [static|matrix|cold|all]" >&2; exit 2 ;;
esac

printf '\n== probe_gpui_followup.sh section=%s: non-zero captures=%d ==\n' "$SECTION" "$FAILURES"
exit 0
