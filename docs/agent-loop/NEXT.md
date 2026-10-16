# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..005`,
and this file. Trust repo state over chat.

> **State note (2026-10-16, after the DEC-015 OpenCode migration checkpoint):** the "In flight" list
> below predates `STATUS.md` and `NEXT-SESSION-PROMPT.md` and is **not** current — CTX-001,
> INSTALL-001, CLIENT-API-001, STORE-001, REF-006/007/009, and RESEARCH-001/002 are integrated.
> Reconcile actual task state from `git log`, `git worktree list`, and `STATUS.md` before acting.
> Only the harness statement below and the ordered work in `NEXT-SESSION-PROMPT.md` are current.

> **Restart trigger (DEC-008/015 — compaction is disabled):** if this session approaches its context
> limit, STOP launching new work, finish a safe checkpoint (commit + push + update
> `docs/agent-loop/**`), and tell the user to start a fresh session with the next-session prompt.
> Repository files are authoritative; never rely on conversation memory. Project work is durable by
> Git branch, so a restart loses nothing. `opencode.json` sets `default_agent: orchestrator`, so a
> fresh session opened in this workspace re-takes the lead role; `scripts/agent-loop.sh` runs that as
> a sequence of fresh sessions; otherwise paste `NEXT-SESSION-PROMPT.md`.

## First commands

```bash
cd /home/sulthonuladib/personal/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
ls docs/adr docs/architecture spikes/terminal-engine docs/compatibility/doc-002-audit docs/reviews
```

## Current state (lead session, 2026-10-16)

`master` is clean at `416fa51`. Baseline re-verified: `cargo build --workspace --offline` and
`cargo test --workspace --offline` both exit 0 with 0 failures. All Wave 1/2 research, the review/fix
cycle, the core slices, the GPUI reference workspace, the QA tooling, and **all five ADRs** are
integrated. **DEC-011** (2026-10-16) clarifies ADR-005 Decision 5.

**Historical — superseded (both projects below are integrated; kept for path-ownership history):**

- **CTX-001** — `ctx/ctx-001-context-env`, worktree `/home/sulthonuladib/work/relayable-wt/ctx-001`:
  set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed terminals (ADR-005
  Decision 2) + the §8.1 `tests/context-resolution` integration test. Owns
  `crates/relayable-server/src/{registry,config}.rs`, `tests/Cargo.toml`, `tests/context-resolution/**`.
- **INSTALL-001** — `install/install-001-user-install`, worktree
  `/home/sulthonuladib/work/relayable-wt/install-001`: `scripts/install-user.sh`, a `systemd --user`
  unit, a desktop entry, root `README.md`, and `docs/operations/user-install.md` (the primary user
  goal). Owns those paths only.

Neither in-flight branch may edit `docs/agent-loop/**` or `docs/adr/**`; the Manager integrates and owns
those. **Do not run STORE-001 concurrently with CTX-001**: both touch
`crates/relayable-server/src/registry.rs`.

## Open tasks, in order

1. **Integrated & green:** CTX-001 (INT-015), INSTALL-001 (INT-016), CLIENT-API-001 (INT-017), STORE-001
   (INT-018 `f4d59f8`).
2. **Integrated:** RESEARCH-001/002 (INT-021 `96b4180`); REF-006 (INT-020 `ba92c51`), REF-007/009
   (INT-019 `54fe220`). The gpui-kit / libghostty adoption decision (DEC-014) is still with the user.
3. **Next-phase plan:** `docs/architecture/next-phase-plan.md`. **REF-008** (server split) is the last
   file-size blocker before **REF-010** (`--strict` CI gate). After RESEARCH-001/002: **TERM-003**
   (libghostty-rs spike + ADR-003 amendment), then **UI-004/005/006** (auto-session on open, keymap helper,
   theming), then **DOCS-001** (central docs index).
4. **Cleanup:** remove merged refactor worktrees (AGENTS.md "Worktree lifecycle / cleanup") and centralize
   docs once the gpui-kit / libghostty decisions are recorded.
5. Backlog: `MEM-001` (memory baseline), `REF-004` remainder, `CI-HOST-001`, `UI-003`, `QA-FLAKE-001`
   (pre-existing `server_e2e` flake).
6. **TERM-003 (proposed; user decision pending)** — user prefers **libghostty / `libghostty-rs`**
   (ADR-003 reconsideration trigger #10); adopting it needs the RESEARCH-002 evidence + an ADR-003
   amendment.
7. Update `docs/agent-loop/**` after every meaningful iteration.

## STORE-001 SQLite binding — decided: option A (2026-10-16)

**User approved option A.** STORE-001 (ADR-004 Decision 2) now uses `rusqlite = "=0.40.2"` with the
`bundled` feature (`libsqlite3-sys 0.38.2`, SQLite compiled from the amalgamation; no runtime system
SQLite dependency). A one-time networked fetch populated the cargo cache (2026-10-16); **all subsequent
builds stay `--offline`**. Recorded as **DEC-012**. No `unsafe` FFI (option B) and no further deferral
(option C).

## Review gates (do not skip)

- **All five ADRs are Accepted** (ADR-001/002/003/004/005).
- No REX wire/binary compatibility claims (`full.md` §3); no GoRex source/asset reuse (§2.6).
- UI fidelity: catalogue + design tokens are the acceptance gate; unverified states stay labelled.
- Android is deferred (DEC-010): no Android app/SDK/emulator/Gradle work now, but keep
  `relayable-model`/`relayable-protocol` Android-ready (no Linux-only assumptions; §1.3.6, §5.2).
- Never claim server-crash or host-reboot session survival (§6.3).
- The root workspace must not absorb `spikes/**` (verify `cargo build` still works in each spike dir).

## Approval gates

- **None active.** Android SDK/emulator + JDK pin are deferred (DEC-010); Android is not in scope.

## Environment / cleanup

- Delegation uses fresh bounded agents; one writer per branch/path. Agent roles and their write
  permissions are defined in `.opencode/agents/` (see `AGENTS.md` "Agent roles (OpenCode)").
- **Orchestration is OpenCode (DEC-015).** The herdr + Pi stack is no longer used by this repository:
  `.pi/settings.json` was removed and no script depends on herdr. DEC-009's `agent_pane_busy` fault and
  its `/tmp/relayable-wake-daemon.py` fallback are historical and must not be restarted — the
  mechanism they worked around no longer exists. Host-level herdr configuration was left untouched;
  if on-screen agent panes or SSH-attached sessions are wanted later, that is a host-level choice
  (`herdr integration install opencode`), not a repository dependency.
- Worktree cleanup (AGENTS.md "Worktree lifecycle / cleanup"): done+merged task worktrees may be removed
  by the lead, preserving branches. Check `git worktree list` after any cleanup and record it in
  `WORKLOG.md`.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
