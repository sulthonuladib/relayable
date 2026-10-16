# STATUS — Relayable

**Phase:** 1–2 (Linux server + CLI + GPUI reference workspace delivered; packaging/user-install in progress)
**Milestone:** All five ADRs Accepted; core + reference workspace + QA tooling integrated. **In flight:** CTX-001 (context env vars + §8.1 test) and INSTALL-001 (user install path).
**Updated:** Manager session 2026-10-16 — see `WORKLOG.md` Checkpoint 26.

## Current execution (Manager session, 2026-10-16)

- `master` clean at `416fa51`; baseline re-verified green: `cargo build --workspace --offline` and
  `cargo test --workspace --offline` both exit 0 with 0 failures.
- **CTX-001** in progress on `ctx/ctx-001-context-env` (implementer `ctx001`): set
  `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed terminals + the ADR-005 §8.1
  context-resolution integration test (`tests/context-resolution/**`).
- **INSTALL-001** in progress on `install/install-001-user-install` (implementer `install001`):
  `scripts/install-user.sh`, `packaging/systemd/user/relayable-server.service`, `packaging/desktop/relayable.desktop`,
  root `README.md`, `docs/operations/user-install.md` — the primary user goal (locally installable app).
- **DEC-011** (2026-10-16): ADR-005 Decision 5 clarification — the accepted CORE-001 unique-session CLI
  default is retained; no-context fails only for zero/ambiguous matches.
- Next: integrate CTX-001/INSTALL-001, then STORE-001.

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

- `.pi/settings.json` → `compaction.enabled: false`; never invoke `/compact` unless the user asks.
- Repository files are authoritative; fresh bounded agents; concise handoffs; checkpoint and hand
  off before context pressure.

## Blockers / approval gates

- **Herdr agent provisioning (DEC-009) — RESOLVED 2026-10-15:** the automatic-rename fish hook block
  was removed from `~/.config/fish/config.fish` and the `herdr-automatic-rename` plugin
  **uninstalled** (residual dirs cleared; `herdr plugin list` → `No plugins installed.`); verified
  with `fish -n` and a successful `delegate_project`. The Option D wake daemon remains a documented
  fallback. Details: `DECISIONS.md` DEC-009, `WORKLOG.md` Checkpoints 7 and 10.
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
