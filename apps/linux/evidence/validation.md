# UI-001 validation log — `apps/linux` (phases 1 + 2)

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
| 1 | `cargo check` (before remedy, phase 1) | **101** | TERM-002 blocker — `evidence/repro-cargo-check.log` |
| 2 | `cargo update -p libc --precise 0.2.189` | **0** | remedy — `evidence/remedy-cargo-update.log` |
| 3 | `cargo check` (after remedy) | **0** | `evidence/fixed-cargo-check.log` |
| 4 | `cargo check` | **0** | current code |
| 5 | `cargo test` | **0** | 9 token-sheet tests pass |
| 6 | `cargo build` | **0** | debug binary `target/debug/relayable` |
| 7 | `cargo build --release` | **0** | `target/release/relayable` |
| 8 | `bash evidence/run-window.sh` | **0** | real run + screenshot + terminate — `evidence/run-proof.md` |
| 9 | `cargo fmt --check` (initial, phase 1) | **1** | import-ordering diff — `evidence/fmt-check-before-fix.log` |
| 10 | `cargo fmt` | **0** | fixed import ordering in `src/main.rs` |
| 11 | `cargo fmt --check` | **0** | clean (also after the phase-2 token refactor) |
| 12 | `cargo clippy --all-targets` | **0** | no lints from `relayable-linux` — `evidence/clippy.log` |

## Phase-2 cluster (steps 4–7, 11–12), verbatim from `validation-commands.log`

```
date_utc=2026-10-15T20:39:31Z
### command: cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
--- check_exit=0 ---
### command: cargo test
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs ... 0 tests
     Doc-tests relayable_linux ... 0 tests
--- test_exit=0 ---
### command: cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
--- build_exit=0 ---
### command: cargo fmt --check
--- fmt_check_exit=0 ---
### command: cargo clippy --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
--- clippy_exit=0 ---
### command: cargo build --release
    Finished `release` profile [optimized] target(s) in 0.19s
--- build_release_exit=0 ---
```

The 9 token tests (`src/tokens.rs` `#[cfg(test)]`) are:

```
tokens::tests::documented_colour_equals_exact_triple ... ok
tokens::tests::rgb_packs_to_0x_rrggbb ... ok
tokens::tests::every_geometry_approx_is_ordered ... ok
tokens::tests::documented_ranges_are_not_collapsed ... ok
tokens::tests::every_measured_token_has_non_empty_source_and_caveat ... ok
tokens::tests::confidence_labels_match_document_vocabulary ... ok
tokens::tests::unmeasured_categories_match_the_document ... ok
tokens::tests::not_app_tokens_are_present_and_reasoned ... ok
tokens::tests::terminal_background_is_a_ramp_not_one_value ... ok
```

## Findings fixed in owned files

- Phase 1: `cargo fmt --check` initially failed (exit 1) on the `use gpui::{...}`
  import ordering in `src/main.rs`; fixed with `cargo fmt`.
- Phase 2: `cargo check` flagged a `non_snake_case` warning for the spec-mandated
  method name `Rgb8::to_u32_0xRRGGBB`; fixed with a scoped
  `#[allow(non_snake_case)]` (the name is required by the UI-001 spec).
- `cargo clippy --all-targets` produced **no** lints for `relayable-linux`.

## Known non-blocking warning (not from our code)

Both build profiles print a `future-incompat` warning for the transitive
`proc-macro-error2 v2.0.1` crate pulled in by the `gpui 0.2.2` dependency tree.
It is not produced by `relayable-linux` and is out of scope for UI-001.

## Lockfile invariant (TERM-002 remedy)

`apps/linux/Cargo.lock` still holds exactly one `libc` = **0.2.189** and one
`xattr` = 0.2.3. The phase-2 lib/bin split and the token module did **not** change
any dependency, so the remedy holds (see `evidence/TERM-002-remedy.md`).

## Real run

See `run-proof.md`: the binary opened a real Wayland toplevel titled `Relayable`
(class `relayable`), ran with no stdout/stderr, and was terminated with `SIGTERM`
(`wait_exit=143`). Screenshots verified with `file`/`identify`.
