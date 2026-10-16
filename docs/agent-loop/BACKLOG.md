# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (lead session, 2026-10-16)

- **Repository:** `/home/sulthonuladib/personal/relayable`, `master` at `1819f1f` (DEC-015 OpenCode
  migration checkpoint; see `WORKLOG.md` Checkpoint 38). Baseline re-verified: `cargo build --workspace
  --offline` and `cargo test --workspace --offline` both exit 0 with 0 failures.
  (Earlier entries in this file recorded the pre-move path `/home/sulthonuladib/work/relayable`.)
- **Accepted:** all five ADRs (ADR-001 domain, ADR-002 protocol, ADR-003 terminal engine + GPUI pin,
  ADR-004 ownership/persistence, ADR-005 env namespace). DEC-011 (2026-10-16) clarifies ADR-005
  Decision 5's unique-session default. DEC-015 moves the agent runtime to OpenCode.
- **In flight:** none. INSTALL-001 integrated (`39e5d49`, INT-016) + `a3ff5c0`; **CTX-001 integrated**
  (`dd1b986`, INT-015): managed terminals now receive
  `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` (ADR-005 Decision 2) and
  `tests/context-resolution/**` proves env resolution, flag precedence, and no-context failure.
- **Blocked:** none. STORE-001's SQLite binding was decided (DEC-012: `rusqlite =0.40.2` bundled) and
  integrated (INT-018 `f4d59f8`).
- **Primary user goal:** a locally installable/testable Linux app for personal use — a
  `systemd --user`-runnable server + the GPUI client + a user install path (INSTALL-001).
- **Approval gates:** none active — Android SDK/emulator install + project JDK pin are deferred (DEC-010).
- **Failing tests:** none.
- **Primary user goal reached:** a locally installable/testable Linux app (server + CLI + GPUI client +
  user install path) is delivered and verified (`scripts/install-user.sh`, `systemd --user` unit).
- **Integrated:** CLIENT-API-001 (INT-017), STORE-001 (INT-018), REF-007/009 (INT-019), REF-006 (INT-020).
- **In flight:** none. **RESEARCH-001/002** (`research/gpui-kit-libghostty`) integrated (INT-021 `96b4180`);
  the gpui-kit / libghostty decision remains with the user (DEC-014).
- **Refactor queue:** REF-005/006/007/008/009 **DONE** — `check-file-sizes.sh --strict` now exits 0 on
  `master` (`over-hard-cap=0`; 8 files over target but under cap). **REF-010** (wire `--strict` into
  `scripts/ci.sh` + `scripts/validate.sh`) is in flight on `refactor/ref-010-strict-gate`.
- **In flight (lead session `ses_ebab9726a`):** REF-010 (implementer), CLIENT-API-002 (`fe11f40`,
  verification in flight), QA-HARNESS-001 repair (`084554b` was **rejected** by the verifier and must
  not be integrated until repaired), UI-003 (READY, blocked on CLIENT-API-002).
- **Merged worktrees removed** (branches preserved) — except the in-flight set:
  `ref-008`, `ref-010`, `qa-harness-001`, `client-api-002`, `ui-003`.
- **Next-phase plan:** `docs/architecture/next-phase-plan.md` — gpui-kit adoption, libghostty engine
  (ADR-003 reconsideration), desktop UX (auto-session on open, keymap helper, theming), docs
  centralization, and refactor cleanup. New tasks: RESEARCH-001/002, TERM-003, UI-004/005/006, DOCS-001.
- **Orchestration rule:** agents must not poll peers; completion is event-driven (`AGENTS.md`
  "Agent coordination (no polling)").
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
| 8 | STORE-001 — `crates/relayable-store` (SQLite metadata/audit) + reconcile/flock/prune/write path per ADR-004 | `store/store-001-sqlite-metadata` | ADR-004 | DONE — INT-018 `f4d59f8`; DEC-012 `rusqlite =0.40.2` bundled |
| 9 | CLIENT-API-001 — incremental terminal bytes | `client/client-api-001-incremental-output` | CORE-001 | DONE — `52433f7` (INT-017); `next_output`; 164 tests |
| 10 | TERM-003 (proposed) — libghostty-vt license/binding/build spike | TBD | user decision | PROPOSED |
| 11 | REF-006 — split over-cap files on free paths (apps/linux, model, pty, cli) | `refactor/ref-006-free-crates` | REF-005 | DONE — INT-020 `ba92c51` |
| 12 | REF-007/009 — split client + tests | `refactor/ref-007-009-client-tests` | REF-005, CLIENT-API-001 | DONE — INT-019 `54fe220`; REF-008 no longer blocked |
| 13 | REF-010 — wire `check-file-sizes.sh --strict` into CI | TBD | REF-008 | BACKLOG |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is orchestrator-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs. Roles and their write permissions: `.opencode/agents/` (`AGENTS.md`
  "Agent roles (OpenCode)"). Delegation is one level deep.
- No polling: a background subagent session notifies its parent on completion. DEC-009 and its
  wake-daemon fallback are historical — orchestration is OpenCode now (DEC-015), so that fault
  cannot recur and the daemon must not be restarted.
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
