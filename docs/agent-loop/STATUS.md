# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 exit gate (§16): evidence status clear, ADRs recorded, buildable minimal shell exists.
**Updated:** Context-window policy implemented (DEC-008); Wave 1 research branches exist but Leads are not yet running after a pane-launch failure (see `BACKLOG.md`, `WORKLOG.md` Checkpoint 2).

## Objective (current)

Stand up the evidence base and the first working vertical slice:

1. Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.
2. Verified toolchain inventory; no installs.
3. Domain/dependency and protocol ADRs.
4. Smallest useful vertical slice: server-owned PTY, CLI-created session, terminal I/O, disconnect
   survives, reconnect recovers.

## Context-window policy (DEC-008)

- `.pi/settings.json` → `compaction.enabled: false`; `/compact` is never invoked unless the user asks.
- Global `~/.pi/agent/settings.json` already had `compaction.enabled: false` and was not modified.
- pi-herdsman's `contextRetirement` defaults to `true`: assigned managed agents retire at context
  pressure instead of threshold compaction.
- Repository files are authoritative; agents return concise handoffs; settings take effect on
  `/reload` or a new Pi process (a running session may retain its startup configuration).

## Wave 1 research state

| Branch | Worktree | Scope | State |
|---|---|---|---|
| `research/ui-reference` | wX | five videos + GoRex catalogue (`docs/research/**`) | assignment persisted; Lead not running |
| `research/protocol-compat` | wY | official REX behavior matrix (`docs/compatibility/**`) | assignment persisted; Lead not running |
| `research/env-audit` | wZ | toolchain audit (`docs/operations/**`) | assignment persisted; Lead not running |

The three delegations failed at pane launch (`agent target pane wX|wY|wZ:p1 is not an available
shell`) after the branches/worktrees were created. Recovery is `resume_project <branch>`; resume one
project, verify the Lead starts, then resume the others. Worktrees were fast-forwarded to the policy
commit so they contain `.pi/settings.json` and the current loop docs. No research files exist on the
branches yet.

The Manager reviews and integrates each branch into `master` (INT-001) before Wave 2 (ADRs, terminal
spike, CORE-001).

## Repository state

- Branch `master`. Recent commits: `1b0d8c8` init, `7181f92` plan update, `d2e13f7` Phase 0
  scaffold, `1c84bca` Wave 1 backlog/ownership, plus the context-policy commit.
- Working tree clean at the policy checkpoint.
- **Host path correction:** the Phase 0 kickoff WORKLOG recorded `/home/sulthonuladib/personal/relayable`.
  That path no longer exists. The checkout is `/home/sulthonuladib/work/relayable` — same history,
  same remote (`git@github.com:sulthonuladib/relayable`).
- At Phase 0 start the repo contained only `full.md` (1279 lines). No pre-existing user code changes
  were present to preserve; nothing was discarded.

## Blockers

- **Lead launch (recoverable):** the first Wave 1 delegations failed at Herdr pane readiness. The
  assignments and worktrees persist; `resume_project` is the next step. This blocks REF-001..003,
  DOC-001, ENV-001 only until resumed.

## Known risks

- GPUI is under active development; pin it and isolate it inside `apps/linux` (§5.3). Availability
  and acquisition path are unverified until ENV-001 lands.
- The original REX wire protocol is **not** documented; binary/wire compatibility must not be
  claimed (§3).
- GoRex is a clone reference only; its license was not previously found — re-verify before any reuse.
  No GoRex source or assets may be copied (§2.6).
- Terminal engine choice is unresolved and gates UI work (§5.4); `ADR-003` needs ENV-001 + TERM-001.
- JDK 25 (Temurin) is present; Android Gradle Plugin compatibility with JDK 25 is unverified (§10.1.4).
- If any reference footage proves inaccessible, the UI fidelity gate stays explicitly open (§2.7).

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a
supervisor design is implemented and tested (§1.3.10, §6.3).
