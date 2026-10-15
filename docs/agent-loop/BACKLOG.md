# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (end of Wave 1)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master`, clean tree. Wave 1 integrated:
  merges `b24133c` (ENV), `c0578ca` (REF), `49795a0` (DOC) plus the handoff commit.
- **On `master` now:** `docs/operations/` (2 audits), `docs/research/` (5 docs),
  `docs/compatibility/` (1 matrix).
- **Live agents:** none. Three project Leads idle with open assignments; worktrees clean at tips
  (ui-reference `269018c`, protocol-compat `9b0fe37`, env-audit `a31a839`).
- **Blocked:** nothing current. Future agent delegation needs the DEC-009 fix or the wake daemon,
  because the Herdr `agent_pane_busy` fault remains.
- **Approval gates:** DEC-009 (Herdr remediation A/B/C); Android SDK/emulator install (AND-001).
- **Failing tests:** none — no build/test suite exists yet (Phase 0).
- **Next integration step:** none pending; the next milestone is TERM-001/ADR-003 and ADR-001/002,
  then CORE-001.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | TERM-001 + ADR-003 — terminal engine, GPUI artifact/license pin | branch TBD | ENV-001 evidence | READY |
| 2 | ADR-001/ADR-002 — domain model, protocol/transport | branch TBD | DOC-001 matrix | READY |
| 3 | CORE-001 — server-owned PTY + CLI session + terminal I/O | branch TBD | ADR-001/002 | waiting |
| 4 | CORE-002 — disconnect survival + reattach snapshot | branch TBD | CORE-001 | waiting |
| 5 | UI-001/UI-002 — GPUI workspace against catalogue + tokens | branch TBD | UI evidence + ADR-003 | waiting |
| 6 | QA-001 smoke/CI; AND-001 Android scaffold checks | branch TBD | CORE-001 / approval | waiting |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: apply DEC-009 or start the wake daemon (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
