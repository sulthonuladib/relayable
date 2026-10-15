# BACKLOG — Relayable prioritized queue

Acceptance criteria for every task live in `TASKS.md` (full ledger). This file is the ordered,
actionable view: what runs next, dependency order, and the current execution state. Update at every
checkpoint; keep it short.

## Current execution state

- **Repository:** `/home/sulthonuladib/work/relayable`, branch `master`.
- **Live agents:** none.
- **Branch/worktree state:** three Wave 1 research branches and Herdr worktrees exist, created by the
  first delegation attempt and empty of committed work — `research/ui-reference` (wX),
  `research/protocol-compat` (wY), `research/env-audit` (wZ).
- **Blocked:** REF-001..003, DOC-001, ENV-001 — blocked only on Lead launch, not on content. The
  delegations failed with `agent target pane <id>:p1 is not an available shell`, but Herdsman
  persisted the assignments, so recovery is `resume_project <branch>` (a new `delegate_project` on
  the same branch is rejected while an assignment exists).
- **Uncommitted changes:** none after the policy checkpoint; verify with `git status --short`.
- **Failing tests:** none — no build/test suite exists yet (Phase 0).
- **Next integration step:** resume one research Lead sequentially and verify it launches before
  resuming the other two; then INT-001 review/merge into `master`; then Wave 2.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | REF-001/002/003 — five videos + GoRex evidence | `research/ui-reference` | — | resume needed |
| 2 | DOC-001 — official REX behavior matrix | `research/protocol-compat` | — | resume needed |
| 3 | ENV-001 — toolchain audit (no installs) | `research/env-audit` | — | resume needed |
| 4 | INT-001 — integrate Wave 1 into `master` | `master` | 1–3 | waiting |
| 5 | TERM-001 + ADR-003 — terminal engine spike/decision | branch TBD | ENV-001 | waiting |
| 6 | ADR-001/ADR-002 — domain + protocol decisions | `master` | DOC-001 | waiting |
| 7 | CORE-001 — server-owned PTY + CLI session + terminal I/O | branch TBD | ADR-001/002 | waiting |
| 8 | CORE-002 — disconnect survival + reattach snapshot | branch TBD | CORE-001 | waiting |
| 9 | UI-001/UI-002 — GPUI reference workspace | branch TBD | UI catalogue + ADR-003 | waiting |
| 10 | QA-001 smoke/CI + AND-001 Android scaffold checks | branch TBD | CORE-001 / ENV-001 | waiting |

## Operating rules for the queue

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; never inject full research reports or
  `full.md` into task briefs (point at repository paths instead).
- The first engineering slice is CORE-001 (server-owned PTY, CLI-created session, terminal I/O,
  reconnection). UI work follows the integrated reference catalogue — high fidelity, not redesign.
- After every meaningful iteration: update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`,
  `STATUS.md`, and `DECISIONS.md` when a decision changed.
