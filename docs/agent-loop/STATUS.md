# STATUS — Relayable

**Phase:** 1–2 (Linux server + CLI + GPUI reference workspace delivered; **user install path delivered**)
**Milestone:** All five ADRs Accepted; core + reference workspace + QA tooling + packaging/user-install
integrated; refactor wave complete (`check-file-sizes.sh --strict` enforced in `validate.sh` + `ci.sh`);
desktop auto-session delivered (UI-004 stage 1). **In flight:** UI-003 (in verification, `8d84a84`).
integrated; **refactor wave complete** (every `REF-*` DONE; `check-file-sizes.sh --strict` enforced in
`validate.sh` + `ci.sh`). In flight: UI-003.
**Updated:** lead session `ses_ebab9726a` 2026-10-16 — REF-008 integrated (INT-022 `afcc631`,
`WORKLOG.md` Checkpoint 40); earlier work in Checkpoints 26–39.

## Current execution (lead session `ses_ebab9726a`, 2026-10-16)

- `master` is at **INT-026** (clean, pushed). Post-merge: `scripts/validate.sh` exits 0 (8 steps; step 4
  = **206 passed / 0 failed**), `scripts/smoke.sh` → `SMOKE OK`, `scripts/check-file-sizes.sh --strict` →
  exit 0 (`over-hard-cap=0`), desktop `cargo test --locked` → **65 passed / 0 failed**.
- **Integrated this session:** REF-008 (INT-022 `afcc631`), CLIENT-API-002 (INT-023 `494917f`),
  QA-HARNESS-001 (INT-024 `7c8f5a2` + `1a54cc4`), REF-010 (INT-025 `4a4cb16`), UI-004 stage 1
  (INT-026 `385f46f`), plus DOC-003 (`af72801`, citation repair). Three of these were **rejected or
  qualified by an independent verifier first**; one was repaired before landing, one after a nit, and
  UI-004 stage 1 landed with its nits recorded as `UI-004-STAGE2`.
- **Refactor wave complete** — every `REF-*` task is DONE and the hard-cap gate is enforced in
  `validate.sh` + `ci.sh`.
- **UI-003 in verification** (`ui/ui-003-next-output`, commit `8d84a84`): the `apps/linux` data plane now
  runs on `relayable_client::Client` (`next_output` + `resize_with_pixels`), the raw
  `FrameConnection`/`should_ack` path is deleted, `AttachBlock` carries a `session_id`,
  `Output.stream_seq` is gone, resize reports the server-accepted geometry, and README §7/§10 are
  rewritten. A measured ack-rate limit (**CLIENT-API-003**) means a tiny-write flood can still drop the
  data-plane connection; the pre-migration plane failed the same way, so it is a backlog item, not a
  regression.
- **Environment drift (DEC-018):** `rustc`/`cargo` are now **1.99.1** (no `rust-toolchain.toml` pin);
  the `apps/linux` dependency set was missing from the offline cache and is cached again
  (`CARGO_NET_OFFLINE=true cargo build --locked` → exit 0); `spikes/terminal-engine-phase2` (`avt`) is
  still absent, so that single `ci.sh` row cannot pass offline — **user decision pending**.
- **UI-004 staged (DEC-019):** stage 1 fixes auto-session/`pending_create`/retryable-error inside
  `apps/linux/src/app/**`; stage 2 (attach an existing session) needs `api/**` plus a restore policy and
  is deferred to the user.
- Next: verify/integrate UI-003 → UI-004 stage 1 → UI-005 → UI-006; in parallel (offline-safe) DOC-003
  (citation repair), DOCS-001 (docs index), MEM-001, CI-HOST-001, QA-FLAKE-001.
- **REF-008 DONE** — INT-022 `afcc631`: `crates/relayable-server/src/session.rs` 1037 → 28 lines + 7
  `src/session/*` modules; `tests/server_e2e.rs` 1472 → `tests/server_e2e/**` (9 files, one binary).
  Independent verifier: **accept-with-nits** — pure move (0 tokens/items/tests lost, no new bare `pub`,
  9 + 4 test names preserved); the nits are stale doc citations → **DOC-003**.
- **REF-010 in flight** — `refactor/ref-010-strict-gate` wires `--strict` into `scripts/ci.sh`,
  `scripts/validate.sh`, and the standard's Enforcement section.
- **CLIENT-API-002 in flight** (`fe11f40`, DEC-017) — additive `Client::resize_with_pixels` returning
  `TerminalResizeAccepted`; `resize` unchanged. Verification in flight; UI-003 is gated on it.
- **QA-HARNESS-001 NOT integrated** — the first attempt (`084554b`) was **rejected** by the independent
  verifier: nested `#[cfg(test)] mod <name>;` resolution was wrong, so
  `relayable-protocol/src/codec/tests.rs` and `relayable-cli/src/json/tests.rs` still produced the
  exit-101 symptom, and the checkout-level test could not catch it. A bounded repair is in flight.
- **CTX-001 DONE** — integrated INT-015 `dd1b986`: `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER`
  are set in every managed terminal (`crates/relayable-server/src/registry.rs`), and
  `tests/context-resolution/**` (4 integration tests) plus 2 unit tests prove env resolution,
  `-s`-over-env precedence, and no-context failure (exit 3 ambiguous / exit 2 missing).
- **INSTALL-001 DONE** — integrated INT-016 `39e5d49` (+ `a3ff5c0`): `scripts/install-user.sh` installs
  release `relayable`/`relayable-server` into XDG user dirs with a `systemd --user` unit and a desktop
  entry (GPUI app installed as `relayable-desktop` under `--with-desktop`, which builds in ~2m).
  Independently validated: sandbox install + live CLI round-trip against the installed binaries,
  socket dir 0700 / socket 0600, SIGTERM exits 0 and unlinks the socket.
- **STORE-001 DONE** — integrated INT-018 `f4d59f8` under DEC-012 (`rusqlite =0.40.2`, `bundled`): the
  `crates/relayable-store` crate, server start-up reconciliation, the `flock` single-instance guard, prune,
  and the session/tombstone/audit write path (see `docs/operations/persistence.md`).
- Next: verify + integrate REF-010 and CLIENT-API-002 → UI-003 → UI-004 → UI-005 → UI-006 → DOCS-001 →
  DOC-003 → backlog (`MEM-001`, `CI-HOST-001`, `REF-004` remainder, `QA-FLAKE-001`).
- **Reconciliation + concurrency (2026-10-16, `WORKLOG.md` Checkpoints 39–40):** the STORE-001 rows above
  are corrected to DONE (INT-018). Two lead sessions ran on this checkout at once; this session
  (`ses_ebab9726a…`, "REF-008 onward") owns the whole ordered queue (see `NEXT.md` "Single-writer
  claims"), and the other session yielded REF-008/UI-003 and moved its duplicate implementers out.
  Drive this checkout with **one** lead session at a time.

---

## Historical phase status (Phase 0 → first vertical slice, 2026-10-15)


## Objective (current)

1. ~~Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.~~ DONE
2. ~~Verified toolchain inventory; no installs.~~ DONE
3. Domain/dependency and protocol ADRs — DONE (**ADR-001/002 Accepted** after REVIEW-001 + ADR-FIX-001; `31bed30` + INT-007 `f474c83`).
4. Terminal engine + GPUI pin ADR and spike — **DONE**; ADR-003 **Accepted** (DEC-004): TERM-002 validated the engine pins, UI-001 resolved the `gpui = "=0.2.2"` build (libc 0.2.189 lockfile hold) and ran a real window (`09928db`).
5. Smallest useful vertical slice (server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers) — **DONE**, integrated `d3e0286` (INT-008); 104 tests pass, 5
   acceptance properties independently verified (`TESTS.md` 76–78).
6. DOC-002 matrix citation repair — DONE, integrated `16c8bc9` (INT-005).

## Wave 1 research state (all integrated on `master`)

| Branch | Scope | Result |
|---|---|---|
| `research/ui-reference` | five videos, GoRex, UI catalogue, design tokens | COMPLETE — integrated `c0578ca` (5 docs, +1902; tip `269018c`) |
| `research/protocol-compat` | official REX behavior/compatibility matrix | COMPLETE — integrated `49795a0` (1 doc, +648; tip `9b0fe37`) |
| `research/env-audit` | Rust/GPUI/systemd/Android/JDK/Gradle audit | COMPLETE — integrated `b24133c` (2 docs, +1591; tip `a31a839`) |

INT-001 complete. Wave 1 branches were removed with their worktrees during the user's wrap-up
(branches and Pi sessions preserved); nothing pushed; no tracked media in any branch.

## Wave 2 state (2026-10-15 handoff)

| Branch | Tasks | Result |
|---|---|---|
| `research/adr-001-002` | ADR-001, ADR-002 | COMPLETE — integrated `31bed30` (ADR-001/002 + `docs/architecture/interface-contract.md`, +1244) |
| `research/term-001-adr-003` | TERM-001, ADR-003 | COMPLETE phase 1 — integrated `78b83f2` (PTY spike + ADR-003, +8300); ADR-003 `Proposed`, engine-level phase 2 pending |
| `research/doc-002` | DOC-002 | MERGED (INT-003 `a1f1719`) — evidence at `docs/compatibility/doc-002-audit/` on `master`; the matrix repair is **not** applied (needs a writer pass) |

`master` now contains `docs/adr/ADR-001..003`, `docs/architecture/interface-contract.md`,
`spikes/terminal-engine/**` (58 files), and `docs/compatibility/doc-002-audit/**`. INT-002/INT-003 are
complete: all three Wave 2 branches are merged and their worktrees removed (branches and Pi sessions
preserved) so new agents start on fresh worktrees. All three Wave 2 Leads were stopped.

## Context-window policy (DEC-008)

- `opencode.json` → `compaction.auto: false`; never compact a session unless the user asks.
- Repository files are authoritative; fresh bounded agents; concise handoffs; checkpoint and hand
  off before context pressure. `scripts/agent-loop.sh` resumes in fresh sessions (DEC-015).

## Blockers / approval gates

- **`protoc` — RESOLVED 2026-10-16 (user-installed):** `protoc` was found missing while validating the
  DEC-015 migration, and the user restored `/usr/bin/protoc` → `libprotoc 36.1` the same day. `cargo
  fmt`, `cargo clippy`, `cargo build --workspace`, `cargo test --workspace`, the lifecycle acceptance
  test, and `check-dependency-direction.sh` all pass again (2026-10-16).
- **Offline cargo cache gap — RESOLVED 2026-10-16:** `spikes/terminal-engine/Cargo.lock` pins
  `cfg_aliases 0.2.2`, but the cache held only `0.1.1`, so `scripts/validate.sh` stopped at step 7. A
  one-time networked `cargo fetch` in the spike workspace (DEC-012 pattern) pulled `cfg_aliases 0.2.2`
  and `nix 0.31.3`. **`scripts/validate.sh` now passes end to end offline — all 7 steps exit 0, 194
  tests pass, 0 fail; `scripts/smoke.sh` → `SMOKE OK`.**
- **Agent runtime (DEC-015) — migrated 2026-10-16:** orchestration is OpenCode; `.pi/settings.json` was
  removed and no repository script depends on herdr. The historical herdr provisioning fault below is
  resolved and cannot recur.
- **Herdr agent provisioning (DEC-009) — HISTORICAL, 2026-10-15:** the automatic-rename fish hook block
  was removed from `~/.config/fish/config.fish` and the `herdr-automatic-rename` plugin
  **uninstalled** (`herdr plugin list` → `No plugins installed.`); verified
  with `fish -n` and a successful `delegate_project`. Superseded by DEC-015 — the herdr/Pi delegation
  path it repaired is no longer used, and its wake-daemon fallback must not be restarted. Details:
  `DECISIONS.md` DEC-009, `WORKLOG.md` Checkpoints 7 and 10.
- **Android — DEFERRED (DEC-010, 2026-10-15):** Android apps are out of scope for now; no SDK/
  emulator install, JDK pin, or Gradle work is required. ENV-001's inventory remains as evidence:
  `docs/operations/android-toolchain.md`. The model/protocol must stay Android-ready (§1.3.6).

## Known risks

- ADR-003 pins `gpui = "=0.2.2"` (crates.io, Apache-2.0 at version level) and `alacritty_terminal 0.26.0` /
  `vte 0.15.0` / `portable-pty 0.9.0`, but ADR-003 is `Proposed` and the engine-level spike has not
  run — the engine choice is provisional. Phase 2 spike (TERM-002) still needs to validate it.
- The original REX wire protocol is not documented — no binary/wire compatibility claims (§3).
- GoRex has no repository-level license → behavior observation only, no reuse (§2.6).
- Research docs are evidence, not truth: spot-check cited URLs/timestamps before relying on a claim.
  DOC-002 found ~142 further wrong line citations beyond `events.md` (audit committed on `doc-002`).
- No build/test suite or workspace exists yet; Phase 0 exit gate still requires the first vertical
  slice. The only in-repo Rust code is the standalone `spikes/terminal-engine` crate (not a workspace).
- **Independent review (REVIEW-001) — recorded 2026-10-15:** report at
  `docs/reviews/REVIEW-001-adr-review.md` (+4 evidence files) on branch `review/review-001-adr-pass`.
  Verdicts: ADR-001 and ADR-002 **accept-with-nits**; ADR-003 + spike **keep `Proposed`**. Blocking
  items: A-1 (PTY ID newtype reconciled in `interface-contract.md`), B-1/B-2/B-3 (ADR-002 Unverified
  table refresh, authn/authz coverage-row fix, D11 no-reserialize test) tracked as `ADR-FIX-001`.
  TERM-002 is a hard gate for adopting the engine, **not** for CORE-001 provided CORE-001 does not
  adopt `alacritty_terminal`/`vte`.

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a
supervisor design is implemented and tested (§1.3.10, §6.3).
