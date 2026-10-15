# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 — **Wave 1 research integrated; Wave 2 (ADRs, terminal spike, first vertical slice) in flight.**
**Updated:** Wave 2 kickoff — see `WORKLOG.md` Checkpoint 7 and `NEXT.md`.

## Objective (current)

1. ~~Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.~~ DONE
2. ~~Verified toolchain inventory; no installs.~~ DONE
3. Domain/dependency and protocol ADRs — IN_PROGRESS (`research/adr-001-002`).
4. Terminal engine + GPUI pin ADR and spike — IN_PROGRESS (`research/term-001-adr-003`).
5. Smallest useful vertical slice (server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers) — after ADR-001/002.

## Wave 1 research state (all integrated on `master`)

| Branch | Scope | Result |
|---|---|---|
| `research/ui-reference` | five videos, GoRex, UI catalogue, design tokens | COMPLETE — integrated `c0578ca` (5 docs, +1902; tip `269018c`) |
| `research/protocol-compat` | official REX behavior/compatibility matrix | COMPLETE — integrated `49795a0` (1 doc, +648; tip `9b0fe37`) |
| `research/env-audit` | Rust/GPUI/systemd/Android/JDK/Gradle audit | COMPLETE — integrated `b24133c` (2 docs, +1591; tip `a31a839`) |

INT-001 complete. Wave 1 branches were removed with their worktrees during the user's wrap-up
(branches and Pi sessions preserved); nothing pushed; no tracked media in any branch.

## Wave 2 state (in flight, base `master` @ `b96da27`)

| Workspace | Branch | Session | Tasks | State |
|---|---|---|---|---|
| `w12` | `research/doc-002` | `3534d553-bec1-455c-83df-807ee3ca724a` | DOC-002 | IN_PROGRESS (Lead `working`) |
| `w13` | `research/term-001-adr-003` | `8f442f6e-6f8c-4a03-9016-dcf3f5827aa1` | TERM-001, ADR-003 | IN_PROGRESS |
| `w14` | `research/adr-001-002` | `6a4c1607-f4d3-42fb-b1fc-d6e2dce6e7ba` | ADR-001, ADR-002 | IN_PROGRESS |

Manager (this session) owns `docs/agent-loop/**` and integration on `master` (INT-002).

## Context-window policy (DEC-008)

- `.pi/settings.json` → `compaction.enabled: false`; never invoke `/compact` unless the user asks.
- Repository files are authoritative; fresh bounded agents; concise handoffs; checkpoint and hand
  off before context pressure.

## Blockers / approval gates

- **Herdr agent provisioning (DEC-009) — RESOLVED 2026-10-15:** option A applied (automatic-rename
  fish hook sourcing disabled in `~/.config/fish/config.fish`); verified with `fish -n` and a
  successful `delegate_project`. The Option D wake daemon remains a documented fallback. Details:
  `DECISIONS.md` DEC-009, `WORKLOG.md` Checkpoint 7.
- **Android SDK/emulator install (AND-001 gate):** no SDK, sdkmanager/avdmanager/emulator, AVDs,
  images, or licenses; no project JDK pin (JDK 25 only; AGP-on-JDK-25 unverified). Install and
  license acceptance need explicit user approval; disk is 68% used. Evidence:
  `docs/operations/android-toolchain.md`.

## Known risks

- ADR-003 must decide the GPUI artifact/license pin: crates.io `gpui` 0.2.2 (Apache-2.0) vs
  `Glass-HQ/gpui` (no root LICENSE; manifests declare Apache-2.0); not cached locally.
- The original REX wire protocol is not documented — no binary/wire compatibility claims (§3).
- GoRex has no repository-level license → behavior observation only, no reuse (§2.6).
- Research docs are evidence, not truth: spot-check cited URLs/timestamps before relying on a claim.
- No build/test suite or workspace exists yet; Phase 0 exit gate still requires the first vertical
  slice.

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a
supervisor design is implemented and tested (§1.3.10, §6.3).
