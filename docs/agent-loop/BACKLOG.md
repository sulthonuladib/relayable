# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (Manager integration session, 2026-10-15)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master` clean and pushed at `d3e0286` (INT-008).
- **Accepted:** ADR-001 (domain/dependency), ADR-002 (proto3+prost framing/transport), ADR-005
  (`RELAYABLE_*`-only env namespace). **ADR-003** (terminal engine + GPUI pin) stays `Proposed` until
  TERM-002 (engine) and UI-001 (GPUI pin).
- **Integrated:** REVIEW-001 (`774d394`), DOC-002 (`16c8bc9`), ADR-FIX-001 (`f474c83`), ADR-005
  (`74e7e20`), CORE-001 (`d3e0286`). The repo now has the full Cargo workspace + `crates/**` +
  `protocol/schema/**` + `tests/**` + `scripts/validate.sh`.
- **Live agents:** TERM-002 (`spike/term-002-engine-validation`) is the only active project; the
  REVIEW-001/DOC-002/ADR-005/ADR-FIX-001/CORE-001 Leads are stopped (worktrees/branches preserved).
- **Approval gates:** none active — Android SDK/emulator install + project JDK pin are deferred
  (DEC-010).
- **Failing tests:** none — `scripts/validate.sh` passes on `master` (fmt/clippy/build/test 104,
  dependency-direction, DEC-010 guard, spike build); evidence `TESTS.md` 76–78.
- **Next integration step:** none pending.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | TERM-002 — engine-level spike validates ADR-003's provisional pin | `spike/term-002-engine-validation` | TERM-001 | DONE — `60d4395`; engine pins accept; gpui pin blocked |
| 2 | CORE-002 — reconnect/events + reattach-snapshot hardening | `core/core-002-reconnect-events` | CORE-001 | DONE — `91662af` (148 tests) |
| 3 | UI-001/UI-002 — GPUI workspace; UI-001 proves the gpui pin (DONE `09928db`); UI-002 = reference workspace | TBD | UI evidence + ADR-003 | UI-001 DONE; UI-002 READY |
| 4 | QA-001 — smoke/CI scripts and build matrix | TBD | CORE-001 | READY |
| 5 | CTX-001 — ADR-005 context env vars in managed terminals + §8.1 test | TBD | ADR-005, CORE-001 | READY |
| 6 | ADR-004 (DEC-005 ownership/persistence) | TBD | CORE-002 | waiting |
| 7 | TERM-003 (proposed) — libghostty-vt license/binding/build spike | TBD | user decision | PROPOSED |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: DEC-009 is resolved (option A applied). The wake-daemon stopgap is
  documented fallback only; restart it if delegation regresses (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
