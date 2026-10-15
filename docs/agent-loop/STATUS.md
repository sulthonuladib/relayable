# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 — **Wave 1 research is complete and integrated; Wave 2 (ADRs + first vertical slice) is next.**
**Updated:** End of Wave 1 handoff — see `NEXT.md` and `WORKLOG.md` Checkpoint 6.

## Objective (current)

1. ~~Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.~~ DONE
2. ~~Verified toolchain inventory; no installs.~~ DONE
3. Domain/dependency and protocol ADRs — next.
4. Smallest useful vertical slice (server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers) — after the ADRs.

## Wave 1 research state (all integrated on `master`)

| Branch | Scope | Result |
|---|---|---|
| `research/ui-reference` | five videos, GoRex, UI catalogue, design tokens | COMPLETE — integrated `c0578ca` (5 docs, +1902; tip `269018c`) |
| `research/protocol-compat` | official REX behavior/compatibility matrix | COMPLETE — integrated `49795a0` (1 doc, +648; tip `9b0fe37`) |
| `research/env-audit` | Rust/GPUI/systemd/Android/JDK/Gradle audit | COMPLETE — integrated `b24133c` (2 docs, +1591; tip `a31a839`) |

INT-001 complete. All Leads idle with open assignments, worktrees clean, nothing pushed, no tracked
media in any branch.

## Context-window policy (DEC-008)

- `.pi/settings.json` → `compaction.enabled: false`; never invoke `/compact` unless the user asks.
- Repository files are authoritative; fresh bounded agents; concise handoffs; checkpoint and hand
  off before context pressure.

## Blockers / approval gates

- **Herdr agent provisioning (DEC-009, permanent fix pending):** `herdr agent start` returns
  `agent_pane_busy` while the automatic-rename fish hook spawns background bash in panes. Wave 1 ran
  under the Option D stopgap; that daemon is now **stopped**. Apply option A (recommended: comment
  out the hook source in `~/.config/fish/config.fish` lines 40–43) or restart
  `/tmp/relayable-wake-daemon.py` before the next agent delegation.
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
