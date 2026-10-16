# Code organization standard (Relayable)

Status: **binding** (Manager ruling, 2026-10-16). This standard exists because oversized single files
hurt readability and review, hide module boundaries, and increase merge conflicts (`AGENTS.md`
non-negotiable 6). It applies to hand-written Rust in `crates/**` and `apps/linux/**`.

## Limits

Calibrated to common Rust practice (community guidance is roughly 500–1000 lines per file): aim for
about 500, and treat 1000 as the hard ceiling.

| Kind | Target | Hard cap |
|---|---|---|
| Production source (`crates/*/src/**`, `apps/linux/src/**`) | ≤ 500 lines | **1000** |
| Tests (`tests/**`, `*/tests/**`, `tests.rs`, `#[cfg(test)]` modules) | ≤ 800 lines | **1200** |
| Crate entry / re-export (`lib.rs`, `main.rs`, `mod.rs`) | ≤ 150 lines | **300** |
| Generated code (prost `OUT_DIR`, checked-in generated) | exempt | exempt |
| `spikes/**` (throwaway evidence crates) | exempt | exempt |

A file under the target is comfortable. A file between target and cap is acceptable but should be split
when a natural seam exists (the review record may require it). Files **above the hard cap must be split**
(tracked as `REF-005`…`REF-010`).

## Rules

1. **One module, one responsibility.** A file should answer one question ("what is the connect
   handshake?", not "everything about the server").
2. **Prefer `foo.rs` + a `foo/` submodule directory** over one giant `mod.rs`. `mod.rs` (and `lib.rs`)
   stay thin: declare submodules and re-export the public API.
3. **Preserve public paths.** Use `pub use` re-exports so a split never changes a crate's external
   API. A move that changes a public path is a breaking change and needs review.
4. **Pure moves only.** A split commit moves code (plus `mod`/`use` wiring); it does not change
   behavior, logic, formatting of the moved code, or public APIs. Behavior changes go in their own
   commit.
5. **Tests belong next to what they test.** Keep `#[cfg(test)] mod tests` in the same file while small;
   move it to `tests.rs` (`#[cfg(test)] mod tests;`) once it exceeds ~150 lines.
6. **Integration suites split by scenario family**, e.g. `tests/events/lifecycle.rs`,
   `tests/events/replay.rs`, with `tests/events/main.rs` as the thin entry point.
7. **Data tables split by logical group**, e.g. `tokens/colors.rs`, `tokens/spacing.rs`.
8. **Splitting into modules is not creating crates.** Do not add a workspace member/crate without an
   ADR and Manager review (dependency direction, `full.md` §5.2).
9. **One writer per path.** A refactor must not run concurrently with a feature task that owns the same
   files (see the sequencing table below).

## Enforcement

`scripts/check-file-sizes.sh` reports every file over its cap. It is **report-only** (`--report`,
default) until the refactor queue is complete, then becomes a hard gate (`--strict`) wired into
`scripts/ci.sh` and `scripts/validate.sh` (`REF-010`). No file-size check is added to CI while known
violations exist.

## Safe-split procedure (per crate)

1. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --offline -- -D warnings`,
   `cargo build --workspace --offline`, `cargo test --workspace --offline` — start green.
2. Move one responsibility at a time into a new module; wire `mod`/`pub use`.
3. Re-run the same commands (plus the crate's own tests) after each move.
4. Confirm the public API is unchanged (`cargo check` of dependents; no path churn).
5. Commit per responsibility with a message like `refactor(<crate>): split <file> into <modules>`.

## Sequencing (must not collide with in-flight work)

| Phase | Scope | Precondition |
|---|---|---|
| R1 (`REF-005`) | This standard + `scripts/check-file-sizes.sh` | none (Manager) |
| R2 (`REF-006`) | Free paths: `apps/linux/src/**`, `crates/relayable-model/src/**`, `crates/relayable-protocol/src/**`, `crates/relayable-cli/src/**`, `crates/relayable-pty/src/**` | none (no in-flight owner) |
| R3 (`REF-007`) | `crates/relayable-client/src/**` | CLIENT-API-001 merged |
| R4 (`REF-008`) | `crates/relayable-server/src/**` + `crates/relayable-server/tests/**` | STORE-001 merged |
| R5 (`REF-009`) | `tests/**` (integration suites) | CLIENT-API-001 merged |
| R6 (`REF-010`) | Flip `check-file-sizes.sh --strict` into `ci.sh`/`validate.sh` | R2–R5 complete |

Known offenders (measured 2026-10-16). **Above the hard cap → must split:** `apps/linux/src/app.rs` 1876,
`crates/relayable-client/src/lib.rs` 1630, `crates/relayable-server/tests/server_e2e.rs` 1465,
`apps/linux/src/tokens.rs` 1453, `apps/linux/src/api/mod.rs` 1448, `crates/relayable-server/src/session.rs`
1037, `apps/linux/src/term/mod.rs` 497, `crates/relayable-model/src/lib.rs` 563,
`crates/relayable-pty/src/lib.rs` 338, `crates/relayable-cli/src/lib.rs` 367. **Over target but under cap
(split when a seam is clear, not mandatory):** `tests/events/main.rs` 1058,
`crates/relayable-server/src/registry.rs` 718, `tests/lifecycle/main.rs` 638,
`tests/flow-control/main.rs` 594, `tests/fixtures/mod.rs` 592, `apps/linux/src/theme.rs` 579,
`crates/relayable-server/src/connection.rs` 572, `apps/linux/src/geometry.rs` 550,
`crates/relayable-server/src/dispatch.rs` 509, `apps/linux/src/ui/terminal.rs` 476,
`tests/protocol/main.rs` 442, `crates/relayable-cli/src/json.rs` 420,
`crates/relayable-protocol/src/codec.rs` 338.
