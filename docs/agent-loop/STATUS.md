# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 — **Wave 2 architecture integrated (ADR-001/002/003, TERM-001 phase 1); DOC-002 repair pending; CORE-001 is next.**
**Updated:** Clean-start handoff — see `WORKLOG.md` Checkpoint 9 and `NEXT.md`.

## Objective (current)

1. ~~Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.~~ DONE
2. ~~Verified toolchain inventory; no installs.~~ DONE
3. Domain/dependency and protocol ADRs — DONE (ADR-001/002 integrated `31bed30`, `Proposed`; confirm before CORE-001).
4. Terminal engine + GPUI pin ADR and spike — DONE phase 1 (spike + ADR-003 integrated `78b83f2`, `Proposed`); engine-level spike phase 2 pending (TERM-002).
5. Smallest useful vertical slice (server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers) — next (CORE-001).
6. DOC-002 matrix citation repair — BLOCKED on a single writer pass; evidence complete and committed.

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
- **Android SDK/emulator install (AND-001 gate):** no SDK, sdkmanager/avdmanager/emulator, AVDs,
  images, or licenses; no project JDK pin (JDK 25 only; AGP-on-JDK-25 unverified). Install and
  license acceptance need explicit user approval; disk is 68% used. Evidence:
  `docs/operations/android-toolchain.md`.

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
- **Independent review gap (REVIEW-001):** ADR-001/002/003 and the spike were merged without an
  independent reviewer pass (wind-down forbade new agents); ADR-002 also carries in-file UNVERIFIED
  external claims. Review before CORE-001 builds on them.

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a
supervisor design is implemented and tested (§1.3.10, §6.3).
