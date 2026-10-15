# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 exit gate (§16): evidence status clear, ADRs recorded, buildable minimal shell exists.
**Updated:** Wave 1 Leads resumed but blocked by a fleet-wide Herdr agent-provisioning fault; remediation awaits user approval (see `BACKLOG.md`, `WORKLOG.md` Checkpoint 3).

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
| `research/ui-reference` | wX | five videos + GoRex catalogue (`docs/research/**`) | Lead alive, holding; blocked in `delegate_agent`; branch at `3db73c2`, no writes |
| `research/protocol-compat` | wY | official REX behavior matrix (`docs/compatibility/**`) | Lead alive, holding; blocked in `delegate_agent`; branch at `3db73c2`, no writes |
| `research/env-audit` | wZ | toolchain audit (`docs/operations/**`) | Lead alive, holding; blocked in `delegate_agent`; branch at `3db73c2`, no writes |

All three Leads were resumed and bound; their Agent delegations fail at `stage: agent_start` with
Herdr `agent_pane_busy` because the `herdr-automatic-rename` fish hook spawns background bash on
every prompt (A/B-confirmed; `TESTS.md` rows 15–23). No research files exist on the branches.

Remediation options (approval gate): **A** — stop sourcing the fish hook in
`~/.config/fish/config.fish` (recommended, cosmetic effect only); **B** — project-local
`managed-lead` overlay granting Leads bash/edit/write so they execute directly; **C** —
update/reinstall the plugin. The Manager applies the approved fix, then the Leads retry delegation
and the Manager integrates (INT-001) before Wave 2.

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

- **Agent provisioning (fleet-wide, remediation pending approval):** Herdr's `agent start` reports
  `agent_pane_busy` for programmatically provisioned panes because the automatic-rename fish hook
  spawns background bash on every prompt. All three Wave 1 Leads are blocked in `delegate_agent`;
  no research files were written. Evidence: `TESTS.md` rows 15–23; options and recommendation in
  `WORKLOG.md` Checkpoint 3. Option A (recommended) needs approval to edit
  `~/.config/fish/config.fish`; Option B is repo-only.

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
