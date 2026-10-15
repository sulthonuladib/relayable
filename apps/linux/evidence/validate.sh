#!/usr/bin/env bash
# UI-001 validation harness for the `apps/linux` workspace.
# Runs each required command from `apps/linux`, records the real exit code.
# Usage:  bash evidence/validate.sh   (from apps/linux)
set -u

cd "$(dirname "$0")/.."   # apps/linux

run() {
  local label="$1"; shift
  echo
  echo "### command: $*"
  "$@"
  echo "--- ${label}_exit=$? ---"
}

echo "date_utc=$(date -u +%FT%TZ)"
echo "cargo_version=$(cargo --version)"
echo "rustc_version=$(rustc --version)"
echo "clippy_version=$(cargo clippy --version 2>&1)"
echo "rustfmt_version=$(cargo fmt --version 2>&1)"

run check        cargo check
run test         cargo test
run build        cargo build
run fmt_check    cargo fmt --check
run clippy       cargo clippy --all-targets
run build_release cargo build --release
