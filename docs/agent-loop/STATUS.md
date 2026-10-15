# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 exit gate (§16): evidence status clear, ADRs recorded, buildable minimal shell exists.
**Updated:** Wave 1 research delegated on three branch-isolated projects (see `WORKLOG.md` Checkpoint 2 for exact delegation evidence).

## Objective (current)

Stand up the evidence base and the first working vertical slice:

1. Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.
2. Verified toolchain inventory; no installs.
3. Domain/dependency and protocol ADRs.
4. Smallest useful vertical slice: server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers.

## Wave 1 research (running now — parallel, documentation-only)

| Branch | Scope | Exclusive paths |
|---|---|---|
| `research/ui-reference` | five reference videos + GoRex catalogue | `docs/research/**` |
| `research/protocol-compat` | official REX CLI/API/Lua/events/status docs → behavior matrix | `docs/compatibility/**` |
| `research/env-audit` | Rust/GPUI/systemd/Android CLI/SDK/emulator/JDK audit | `docs/operations/**` |

The Manager reviews each branch diff and integrates it into `master` (INT-001) before Wave 2
(ADRs, terminal spike, CORE-001). No Wave 1 branch edits `docs/agent-loop/**` or application code.

## Repository state

- Branch `master`. Commits: `1b0d8c8 init and commit full.md`, `7181f92 plan update`,
  `d2e13f7 Phase 0: add AGENTS.md, agent-loop scaffold, and gitignore`.
- Working tree was clean at Wave 1 start (`git status --short` → empty output).
- **Host path correction:** the Phase 0 kickoff WORKLOG recorded `/home/sulthonuladib/personal/relayable`.
  That path no longer exists. The checkout is now `/home/sulthonuladib/work/relayable` — same history,
  same remote (`git@github.com:sulthonuladib/relayable`). `NEXT.md` has been corrected.
- At Phase 0 start the repo contained only `full.md` (1279 lines). No pre-existing user code changes
  were present to preserve; nothing was discarded.

## Blockers

None currently. Research access is expected to be feasible (`yt-dlp`, `ffmpeg`, `chromium`, `curl`,
Node 26 are installed) and must still be proven per-video and per-source, recorded honestly.

## Known risks

- GPUI is under active development and may have breaking API changes; it must be pinned and isolated
  inside `apps/linux` (§5.3). Availability and acquisition path are unverified until ENV-001 lands.
- The original REX wire protocol is **not** documented; binary/wire compatibility must not be
  claimed (§3).
- GoRex is a clone reference only. Its license was previously not found at the expected path —
  re-verify before any reuse. No GoRex source or assets may be copied (§2.6).
- Terminal engine choice is unresolved and gates UI work (§5.4); `ADR-003` requires the ENV-001 and
  TERM-001 evidence.
- JDK 25 (Temurin) is present; Android Gradle Plugin compatibility with JDK 25 is unverified and
  must be checked before any Android build attempt (§10.1.4).
- Wave 1 research may find video footage inaccessible; in that case the UI fidelity gate stays
  explicitly open and no fidelity claim may be made (§2.7).

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a
supervisor design is implemented and tested (§1.3.10, §6.3).
