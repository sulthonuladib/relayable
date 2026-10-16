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
| Crate entry / re-export (`lib.rs`, `main.rs`, `mod.rs`, excluding test paths) | ≤ 150 lines | **300** |
| Generated code (prost `OUT_DIR`, checked-in generated) | exempt | exempt |
| `spikes/**` (throwaway evidence crates) | exempt | exempt |

Classification is **path-first**: any file under `tests/**` or `*/tests/**`, or named `tests.rs`, is a
**test** file (800/1200) even when its basename is `main.rs` or `mod.rs`. The entry limits apply to
`lib.rs`/`main.rs`/`mod.rs` in production paths only. `scripts/check-file-sizes.sh` implements exactly this
order.

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

Known offenders (live list: `bash scripts/check-file-sizes.sh`; measured 2026-10-16 after the
REF-006/REF-007/REF-009 integrations). **Above the hard cap → must split (REF-008):**
`crates/relayable-server/src/session.rs` 1037 (production cap 1000) and
`crates/relayable-server/tests/server_e2e.rs` 1472 (test cap 1200). **Over target but under cap (split
when a seam is clear, not mandatory):** `crates/relayable-server/tests/store_wiring.rs` 921,
`crates/relayable-server/src/registry.rs` 808, `crates/relayable-server/src/connection.rs` 572,
`crates/relayable-store/src/store.rs` 566, `crates/relayable-server/src/dispatch.rs` 509. The earlier
offender list here was stale: REF-006/REF-007/REF-009 already split `apps/linux/src/**`,
`crates/relayable-{model,pty,cli}/src/lib.rs`, `crates/relayable-protocol/src/**`,
`crates/relayable-client/src/**`, `tests/events/**` and `tests/fixtures/**`, and those offenders are gone.
