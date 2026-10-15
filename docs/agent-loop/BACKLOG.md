# BACKLOG — Relayable prioritized queue

Acceptance criteria for every task live in `TASKS.md` (full ledger). This file is the ordered,
actionable view: what runs next, dependency order, and the current execution state. Update at every
checkpoint; keep it short.

## Current execution state

- **Repository:** `/home/sulthonuladib/work/relayable`, branch `master` (context policy `ccfdb7a`,
  `3db73c2`; ENV-001 merged `b24133c`).
- **Live agents:** REF and DOC research Agents are running under the Option D stopgap;
  `research/env-audit` finished (2 docs integrated). Assignments for REF/DOC remain open.
- **Blocked:** REF-001..003 and DOC-001 were blocked in `delegate_agent` by a fleet-wide Herdr fault
  (`herdr agent start` → `agent_pane_busy` because the automatic-rename fish hook spawns background
  bash on every prompt). Both are now running under the Option D stopgap; ENV-001 is complete. Root
  cause + A/B evidence: `TESTS.md` rows 15–32 and `WORKLOG.md` Checkpoints 3–5.
- **Remediation pending user approval (approval gate):** Option A — stop sourcing the
  automatic-rename fish hook in `~/.config/fish/config.fish` (lines 40–43); Option B — project-local
  `managed-lead` overlay granting Leads bash/edit/write so they execute directly; Option C —
  update/reinstall the plugin.
- **Uncommitted changes:** none after the ENV-001 integration checkpoint.
- **Stopgap active (Option D, no config change):** bounded wake daemon running (until ~00:05) with
  0 `agent_start` failures in the window; see `WORKLOG.md` Checkpoints 4–5. The permanent fix is
  still the pending DEC-009 decision.
- **Failing tests:** none — no build/test suite exists yet (Phase 0).
- **Next integration step:** integrate REF and DOC branches as their Lead reports complete;
  ADR-003 (GPUI/terminal) can start from the merged ENV-001 evidence; then ADR-001/002 from DOC-001;
  then CORE-001.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | REF-001/002/003 — five videos + GoRex evidence | `research/ui-reference` | — | running (wave 2/3; wave 1 committed) |
| 2 | DOC-001 — official REX behavior matrix | `research/protocol-compat` | — | running (2 writers in flight) |
| 3 | ENV-001 — toolchain audit (no installs) | `research/env-audit` | — | DONE — integrated `b24133c` |
| 4 | INT-001 — integrate Wave 1 into `master` | `master` | 1–3 | IN_PROGRESS (1/3) |
| 5 | TERM-001 + ADR-003 — terminal engine spike/decision | branch TBD | ENV-001 | READY (ENV inputs available) |
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
