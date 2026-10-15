# UI-001 validation log — `apps/linux` (phases 1 + 2 + corrective round)

All commands run from `apps/linux`. Raw self-contained capture (with exit codes
interleaved): `evidence/validation-commands.log`; harness: `evidence/validate.sh`.

Toolchain (verbatim):

```
cargo 1.98.1 (797e8a9bc 2026-08-05)
rustc 1.98.1 (48a229cea 2026-09-01)
clippy 0.1.98 (48a229ceae 2026-09-01)
rustfmt 1.9.0-stable (48a229ceae 2026-09-01)
```

## Corrective round — current command log (all `--locked`)

| # | Command | Exit | Notes |
|---|---|---|---|
| 1 | `cargo check --locked` | **0** | current code |
| 2 | `cargo test --locked` | **0** | 9 token-sheet tests pass |
| 3 | `cargo build --locked` | **0** | debug binary `target/debug/relayable` |
| 4 | `cargo fmt --check` | **0** | clean |
| 5 | `cargo clippy --all-targets --locked` | **0** | no lints from `relayable-linux` |

Verbatim from `validation-commands.log`:

```
date_utc=2026-10-15T20:45:42Z
### command: cargo check --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
--- check_exit=0 ---
### command: cargo test --locked
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs ... 0 tests
     Doc-tests relayable_linux ... 0 tests
--- test_exit=0 ---
### command: cargo build --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.88s
--- build_exit=0 ---
### command: cargo fmt --check
--- fmt_check_exit=0 ---
### command: cargo clippy --all-targets --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
--- clippy_exit=0 ---
```

The 9 token tests (`src/tokens.rs` `#[cfg(test)]`):

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

## Corrective-round changes (see `evidence/corrective-round.md`)

- **Fix 1:** added the two missing measured colour tokens
  `colors::PANE_CARD_BORDER_FOCUSED` = rgb(31,82,115) (medium-high,
  "includes antialias") and `colors::PANE_CARD_BORDER_UNFOCUSED` = rgb(17,54,71)
  (medium, "one frame"); both are now covered by
  `documented_colour_equals_exact_triple` and by the provenance test.
- **Fix 2:** renamed `Rgb8::to_u32_0xRRGGBB` → `Rgb8::to_packed_rgb_u32` and
  deleted the `#[allow(non_snake_case)]`. `rg '#\[allow' apps/linux/src` now
  finds **no** `#[allow(...)]` anywhere in `apps/linux/**`.
- **No screenshot re-capture:** `src/main.rs`'s rendered colours are unchanged
  (`PANE_HEADER_SURFACE_V1` = rgb(9,30,46) and `PRIMARY_TEXT` = rgb(191,219,228);
  `git diff src/main.rs` shows only the method rename + rustfmt wrapping), so the
  committed phase-2 screenshots still reflect the committed code.

## Full history (phases 1–2)

| # | Command | Exit | Notes |
|---|---|---|---|
| 1 | `cargo check` (before remedy, phase 1) | **101** | TERM-002 blocker — `evidence/repro-cargo-check.log` |
| 2 | `cargo update -p libc --precise 0.2.189` | **0** | remedy — `evidence/remedy-cargo-update.log` |
| 3 | `cargo check` (after remedy) | **0** | `evidence/fixed-cargo-check.log` |
| 4 | `cargo build` / `cargo build --release` | **0** | `evidence/build-debug.log`, `evidence/build-release.log` |
| 5 | `bash evidence/run-window.sh` | **0** | real run + screenshot — `evidence/run-proof.md` |
| 6 | `cargo fmt --check` (initial, phase 1) | **1** | import-ordering diff — `evidence/fmt-check-before-fix.log` |
| 7 | `cargo fmt` then `cargo fmt --check` | **0** | clean |

## Known non-blocking warning (not from our code)

Both build profiles print a `future-incompat` warning for the transitive
`proc-macro-error2 v2.0.1` crate pulled in by the `gpui 0.2.2` dependency tree.
It is not produced by `relayable-linux` and is out of scope for UI-001.

## Lockfile invariant (TERM-002 remedy)

`apps/linux/Cargo.lock` still holds exactly one `libc` = **0.2.189** and one
`xattr` = 0.2.3. The corrective round changed no dependency (`--locked` commands
all pass), so the remedy holds (see `evidence/TERM-002-remedy.md`).

## Real run

See `run-proof.md`: the binary opened a real Wayland toplevel titled `Relayable`
(class `relayable`), ran with no stdout/stderr, and was terminated with `SIGTERM`
(`wait_exit=143`). Screenshots verified with `file`/`identify`.
