# UI-001 validation log — `apps/linux` (phase 1)

All commands run from `apps/linux`. Raw self-contained capture (with exit codes
interleaved): `evidence/validation-commands.log`; harness: `evidence/validate.sh`.

Toolchain (verbatim):

```
cargo 1.98.1 (797e8a9bc 2026-08-05)
rustc 1.98.1 (48a229cea 2026-09-01)
clippy 0.1.98 (48a229ceae 2026-09-01)
rustfmt 1.9.0-stable (48a229ceae 2026-09-01)
```

## Command log

| # | Command | Exit | Notes |
|---|---|---|---|
| 1 | `cargo check` (before remedy) | **101** | TERM-002 blocker — `evidence/repro-cargo-check.log` |
| 2 | `cargo update -p libc --precise 0.2.189` | **0** | remedy — `evidence/remedy-cargo-update.log` |
| 3 | `cargo check` (after remedy) | **0** | `evidence/fixed-cargo-check.log` |
| 4 | `cargo build` | **0** | `evidence/build-debug.log`; binary `target/debug/relayable` |
| 5 | `cargo build --release` | **0** | `evidence/build-release.log`; `target/release/relayable` (24,879,280 bytes) |
| 6 | `bash evidence/run-window.sh` | **0** | real run + screenshot + terminate — `evidence/run-proof.md` |
| 7 | `cargo fmt --check` (before fix) | **1** | import ordering diff — `evidence/fmt-check-before-fix.log` |
| 8 | `cargo fmt` | **0** | fixed import ordering in `src/main.rs` |
| 9 | `cargo fmt --check` (after fix) | **0** | clean |
| 10 | `cargo clippy --all-targets` | **0** | no warnings from `relayable-linux` — `evidence/clippy.log` |

## Cluster (steps 1–5, 9–10), verbatim from `validation-commands.log`

```
date_utc=2026-10-15T20:32:04Z
### command: cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
--- check_exit=0 ---
### command: cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.84s
--- build_exit=0 ---
### command: cargo fmt --check
--- fmt_check_exit=0 ---
### command: cargo clippy --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
--- clippy_exit=0 ---
### command: cargo build --release
    Finished `release` profile [optimized] target(s) in 0.21s
--- build_release_exit=0 ---
```

## Findings fixed in owned files

- `cargo fmt --check` initially failed (exit 1) on the `use gpui::{...}` import
  ordering in `src/main.rs`; fixed with `cargo fmt`. Now clean.
- `cargo clippy --all-targets` produced **no** lints for `relayable-linux`.

## Known non-blocking warning (not from our code)

Both build profiles print a `future-incompat` warning for the transitive
`proc-macro-error2 v2.0.1` crate pulled in by the `gpui 0.2.2` dependency tree.
It is not produced by `relayable-linux` and is out of scope for UI-001.

## Real run

See `run-proof.md`: the binary opened a real Wayland toplevel titled `Relayable`
(class `relayable`), ran 6 s, emitted no stdout/stderr, and was terminated with
`SIGTERM` (`wait_exit=143`). Screenshots verified with `file`/`identify`.
