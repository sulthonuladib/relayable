# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (Manager session, 2026-10-16)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master` clean and pushed at `416fa51`
  (NEXT handoff). Baseline re-verified: `cargo build --workspace --offline` and
  `cargo test --workspace --offline` both exit 0 with 0 failures.
- **Accepted:** all five ADRs (ADR-001 domain, ADR-002 protocol, ADR-003 terminal engine + GPUI pin,
  ADR-004 ownership/persistence, ADR-005 env namespace). DEC-011 (2026-10-16) clarifies ADR-005
  Decision 5's unique-session default.
- **In flight:** **CTX-001** (`ctx/ctx-001-context-env`, worktree `/home/sulthonuladib/work/relayable-wt/ctx-001`)
  and **INSTALL-001** (`install/install-001-user-install`, worktree `/home/sulthonuladib/work/relayable-wt/install-001`).
  Disjoint owned paths; Manager integrates.
- **Primary user goal:** a locally installable/testable Linux app for personal use — a
  `systemd --user`-runnable server + the GPUI client + a user install path (INSTALL-001).
- **Approval gates:** none active — Android SDK/emulator install + project JDK pin are deferred (DEC-010).
- **Failing tests:** none.
- **Next integration step:** review CTX-001 and INSTALL-001 diffs, run independent validation, merge to `master`
  (INT-015/INT-016); then start STORE-001 (which shares `crates/relayable-server/src/registry.rs` with CTX-001).

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | TERM-002 — engine-level spike validates ADR-003's provisional pin | `spike/term-002-engine-validation` | TERM-001 | DONE — `60d4395`; engine pins accept; gpui pin blocked |
| 2 | CORE-002 — reconnect/events + reattach-snapshot hardening | `core/core-002-reconnect-events` | CORE-001 | DONE — `91662af` (148 tests) |
| 3 | UI-002 — reference workspace (DONE `d46c66c`); UI-001 DONE `09928db` | `ui/ui-002-reference-workspace` | UI evidence + ADR-003 | DONE |
| 4 | QA-001 — smoke/CI scripts + `QA-FLOW-001` (`SMOKE OK`, ci 15 PASS/3 SKIP) | `qa/qa-001-smoke-ci` | CORE-001 | DONE — `4a258c2` |
| 5 | CTX-001 — ADR-005 context env vars in managed terminals + §8.1 test | `ctx/ctx-001-context-env` | ADR-005, CORE-001 | IN_PROGRESS (implementer `ctx001`) |
| 6 | INSTALL-001 — user install path (script + systemd user unit + desktop entry + README) | `install/install-001-user-install` | CORE-001, UI-002 | IN_PROGRESS (implementer `install001`) |
| 7 | ADR-004 (DEC-005 ownership/persistence) | `adr/adr-004-ownership-persistence` | CORE-002 | DONE — `9a3fb27`; Accepted; store work → `STORE-001` |
| 9 | STORE-001 (after CTX-001) / MEM-001 / CI-HOST-001 / CLIENT-API-001 / REF-004 remainder | TBD | — | BACKLOG |
| 8 | TERM-003 (proposed) — libghostty-vt license/binding/build spike | TBD | user decision | PROPOSED |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: DEC-009 is resolved (option A applied). The wake-daemon stopgap is
  documented fallback only; restart it if delegation regresses (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
