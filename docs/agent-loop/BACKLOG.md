# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (Wave 2 kickoff, 2026-10-15)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master` @ `b96da27`, clean tree, pushed.
  Wave 1 integrated: merges `b24133c` (ENV), `c0578ca` (REF), `49795a0` (DOC) + handoff commit.
- **On `master` now:** `docs/operations/` (2 audits), `docs/research/` (5 docs),
  `docs/compatibility/` (1 matrix).
- **Live agents:** three Wave 2 project Leads `working` on fresh worktrees:
  `research/doc-002` (`w12`), `research/term-001-adr-003` (`w13`), `research/adr-001-002` (`w14`).
- **Blocked:** nothing current. DEC-009 resolved (option A applied); the wake-daemon stopgap remains
  documented fallback only.
- **Approval gates:** Android SDK/emulator install + project JDK pin (AND-001) — only for Android work.
- **Failing tests:** none — no build/test suite exists yet (Phase 0).
- **Next integration step:** INT-002 — review and merge the three Wave 2 branches as they complete,
  then start CORE-001.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | DOC-002 — matrix citation fix + URL drift re-check | `research/doc-002` | DOC-001 | IN_PROGRESS (`w12`) |
| 2 | ADR-001/ADR-002 — domain model, protocol/transport | `research/adr-001-002` | DOC-001 matrix | IN_PROGRESS (`w14`) |
| 3 | TERM-001 + ADR-003 — terminal engine, GPUI artifact/license pin | `research/term-001-adr-003` | ENV-001 evidence | IN_PROGRESS (`w13`) |
| 4 | CORE-001 — server-owned PTY + CLI session + terminal I/O | TBD | ADR-001/002 | waiting |
| 5 | CORE-002 — disconnect survival + reattach snapshot | TBD | CORE-001 | waiting |
| 6 | UI-001/UI-002 — GPUI workspace against catalogue + tokens | TBD | UI evidence + ADR-003 | waiting |
| 7 | QA-001 smoke/CI; AND-001 Android scaffold checks | TBD | CORE-001 / approval | waiting |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: DEC-009 is resolved (option A applied). The wake-daemon stopgap is
  documented fallback only; restart it if delegation regresses (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
