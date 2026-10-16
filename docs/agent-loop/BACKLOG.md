# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (Manager session, 2026-10-16)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master` clean and pushed at `416fa51`
  (NEXT handoff). Baseline re-verified: `cargo build --workspace --offline` and
  `cargo test --workspace --offline` both exit 0 with 0 failures.
- **Accepted:** all five ADRs (ADR-001 domain, ADR-002 protocol, ADR-003 terminal engine + GPUI pin,
  ADR-004 ownership/persistence, ADR-005 env namespace). DEC-011 (2026-10-16) clarifies ADR-005
  Decision 5's unique-session default.
- **In flight:** **INSTALL-001**
  (`install/install-001-user-install`, worktree `/home/sulthonuladib/work/relayable-wt/install-001`).
  **CTX-001 integrated** (`dd1b986`, INT-015): managed terminals now receive
  `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` (ADR-005 Decision 2) and
  `tests/context-resolution/**` proves env resolution, flag precedence, and no-context failure.
- **Blocked:** STORE-001 needs an SQLite binding decision (no SQLite crate is in the offline cargo cache;
  system `libsqlite3` 3.53.4 is present). See `NEXT.md` "Decision needed by the user".
- **Primary user goal:** a locally installable/testable Linux app for personal use — a
  `systemd --user`-runnable server + the GPUI client + a user install path (INSTALL-001).
- **Approval gates:** none active — Android SDK/emulator install + project JDK pin are deferred (DEC-010).
- **Failing tests:** none.
- **Primary user goal reached:** a locally installable/testable Linux app (server + CLI + GPUI client +
  user install path) is delivered and verified (`scripts/install-user.sh`, `systemd --user` unit).
- **Three projects in flight:** CLIENT-API-001 (`client/...`, `w1H`), STORE-001 (`store/...`, `w1J`),
  and REF-006 (`refactor/ref-006-free-crates`, `w1M`, code-organization splits on free paths).
  Disjoint owned paths; Manager integrates.
- **Code organization:** `docs/architecture/code-organization.md` + `scripts/check-file-sizes.sh`
  (REF-005 DONE). Queue: REF-006 (free paths, in flight), REF-007 (client, after CLIENT-API-001),
  REF-008 (server, after STORE-001), REF-009 (`tests/**`, after CLIENT-API-001), REF-010 (wire --strict).

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | TERM-002 — engine-level spike validates ADR-003's provisional pin | `spike/term-002-engine-validation` | TERM-001 | DONE — `60d4395`; engine pins accept; gpui pin blocked |
| 2 | CORE-002 — reconnect/events + reattach-snapshot hardening | `core/core-002-reconnect-events` | CORE-001 | DONE — `91662af` (148 tests) |
| 3 | UI-002 — reference workspace (DONE `d46c66c`); UI-001 DONE `09928db` | `ui/ui-002-reference-workspace` | UI evidence + ADR-003 | DONE |
| 4 | QA-001 — smoke/CI scripts + `QA-FLOW-001` (`SMOKE OK`, ci 15 PASS/3 SKIP) | `qa/qa-001-smoke-ci` | CORE-001 | DONE — `4a258c2` |
| 5 | CTX-001 — ADR-005 context env vars in managed terminals + §8.1 test | `ctx/ctx-001-context-env` | ADR-005, CORE-001 | DONE — `dd1b986` (INT-015); 4 integration + 2 unit tests |
| 6 | INSTALL-001 — user install path (script + systemd user unit + desktop entry + README) | `install/install-001-user-install` | CORE-001, UI-002 | DONE — `39e5d49` (INT-016) + `a3ff5c0`; sandbox install + live round-trip; `--with-desktop` builds |
| 7 | ADR-004 (DEC-005 ownership/persistence) | `adr/adr-004-ownership-persistence` | CORE-002 | DONE — `9a3fb27`; Accepted; store work → `STORE-001` |
| 8 | STORE-001 — `crates/relayable-store` (SQLite metadata/audit) + reconcile/flock/prune/write path per ADR-004 | `store/store-001-sqlite-metadata` | ADR-004 | IN_PROGRESS (Herdr `w1J`); DEC-012 `rusqlite =0.40.2` bundled |
| 9 | CLIENT-API-001 (in flight) / MEM-001 / CI-HOST-001 / REF-004 remainder | TBD | — | IN_PROGRESS / BACKLOG |
| 10 | TERM-003 (proposed) — libghostty-vt license/binding/build spike | TBD | user decision | PROPOSED |
| 11 | REF-006 — split over-cap files on free paths (apps/linux, model, pty, cli) | `refactor/ref-006-free-crates` | REF-005 | IN_PROGRESS (Herdr `w1M`) |
| 12 | REF-007/008/009 — split client / server / tests after their feature tasks merge | TBD | REF-006 + feature merges | BACKLOG |
| 13 | REF-010 — wire `check-file-sizes.sh --strict` into CI | TBD | REF-006..009 | BACKLOG |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: DEC-009 is resolved (option A applied). The wake-daemon stopgap is
  documented fallback only; restart it if delegation regresses (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
