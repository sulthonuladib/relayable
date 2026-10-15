# BACKLOG — Relayable prioritized queue

Acceptance criteria live in `TASKS.md`. This file is the ordered actionable view plus current state.

## Current execution state (Wave 2 handoff, 2026-10-15)

- **Repository:** `/home/sulthonuladib/work/relayable`, `master` clean after INT-002. Wave 2 architecture
  integrated: `31bed30` (ADR-001/002 + `docs/architecture/interface-contract.md`) and `78b83f2`
  (TERM-001 PTY spike + ADR-003).
- **On `master` now:** `docs/adr/ADR-001..003`, `docs/architecture/interface-contract.md`,
  `spikes/terminal-engine/**` (58 files), plus all Wave 1 research.
- **Live agents:** none — all three Wave 2 Leads stopped after integration (user wind-down). The
  `research/doc-002` worktree is retained (branch not merged). The `research/adr-001-002` and
  `research/term-001-adr-003` branches are merged, but their **worktrees remain**: `remove_worktree`
  requires interactive user confirmation and was cancelled twice.
- **Blocked:** DOC-002 matrix repair needs one writer pass; evidence is committed at `2ff3447`
  (`docs/compatibility/doc-002-audit/` on `research/doc-002`).
- **Approval gates:** Android SDK/emulator install + project JDK pin (AND-001) — Android work only.
- **Failing tests:** none — no workspace/test suite yet; the standalone spike crate builds with
  `cargo build`, `cargo clippy --all-targets`, `cargo fmt --check` all exit 0 (raw logs committed).
- **Next integration step:** none pending.

## Priority order (next first)

| Rank | Task | Branch | Depends on | State |
|---|---|---|---|---|
| 1 | DOC-002 writer pass — apply `citation-audit.md` to the matrix | `research/doc-002` | evidence `2ff3447` | BLOCKED (needs writer) |
| 2 | TERM-002 — engine-level spike phase 2 validates ADR-003's provisional pin | TBD | TERM-001 | READY |
| 3 | CORE-001 — server-owned PTY + CLI session + terminal I/O + reconnect | TBD | ADR-001/002 interface contract | READY |
| 4 | CORE-002 — disconnect survival + reattach snapshot | TBD | CORE-001 | waiting |
| 5 | UI-001/UI-002 — GPUI workspace against catalogue + tokens | TBD | UI evidence + ADR-003 | waiting |
| 6 | ADR-004 (DEC-005 ownership/persistence), ADR-005 (DEC-006 env namespace), QA-001 | TBD | ADR-001 | waiting |

## Operating rules

- One writer per branch and per path; `docs/agent-loop/**` is Manager-owned.
- Fresh bounded agent sessions per task; concise handoffs; do not inject full reports or `full.md`
  into briefs.
- Before any agent delegation: DEC-009 is resolved (option A applied). The wake-daemon stopgap is
  documented fallback only; restart it if delegation regresses (see `NEXT.md`).
- Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md`, and `DECISIONS.md` after
  every meaningful iteration.
