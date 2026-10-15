# BACKLOG — Relayable prioritized queue

Acceptance criteria for every task live in `TASKS.md` (full ledger). This file is the ordered,
actionable view: what runs next, dependency order, and the current execution state. Update at every
checkpoint; keep it short.

## Current execution state

- **Repository:** `/home/sulthonuladib/work/relayable`, branch `master` (context-policy commits
  `ccfdb7a`, `3db73c2`).
- **Live agents:** three Wave 1 Leads are alive but idle/holding (`runtime_state: done`) —
  `research/ui-reference` (wX), `research/protocol-compat` (wY), `research/env-audit` (wZ). Their
  assignments are open and preserved; none has written files or commits.
- **Blocked:** REF-001..003, DOC-001, ENV-001 — blocked in `delegate_agent` by a fleet-wide Herdr
  fault: `herdr agent start` returns `agent_pane_busy` ("not an available shell") because the
  automatic-rename fish hook spawns background bash processes on every prompt. Root cause + A/B
  evidence: `TESTS.md` rows 15–23 and `WORKLOG.md` Checkpoint 3.
- **Remediation pending user approval (approval gate):** Option A — stop sourcing the
  automatic-rename fish hook in `~/.config/fish/config.fish` (lines 40–43); Option B — project-local
  `managed-lead` overlay granting Leads bash/edit/write so they execute directly; Option C —
  update/reinstall the plugin.
- **Uncommitted changes:** none after the blocker checkpoint.
- **Stopgap active (Option D, no config change):** bounded wake daemon running (~30 min) and the
  three Leads instructed to retry delegation; see `WORKLOG.md` Checkpoint 4. Result so far: Agents
  started in `research/ui-reference` (2 active) and `research/protocol-compat` (2 active); outcomes
  pending. The permanent fix is still the pending DEC-009 decision.
- **Failing tests:** none — no build/test suite exists yet (Phase 0).
- **Next integration step:** apply the approved fix, tell the Leads to retry delegation, then INT-001
  review/merge into `master`; Wave 2 follows.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | REF-001/002/003 — five videos + GoRex evidence | `research/ui-reference` | — | blocked (Herdr fault) |
| 2 | DOC-001 — official REX behavior matrix | `research/protocol-compat` | — | blocked (Herdr fault) |
| 3 | ENV-001 — toolchain audit (no installs) | `research/env-audit` | — | blocked (Herdr fault) |
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
