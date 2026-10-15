# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..003`,
and this file. Trust repo state over chat.

## First commands

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
ls docs/adr docs/architecture spikes/terminal-engine
```

## Current state (end of Wave 2)

`master` is clean and pushed. Wave 2 architecture is integrated:

- `31bed30` — **ADR-001** (domain model + dependency boundaries), **ADR-002** (proto3 + `prost`;
  4-byte length-prefix framing; pathname Unix socket local; TLS 1.3 remote, off by default), and
  `docs/architecture/interface-contract.md` (the `full.md` §14.4 interface note).
- `78b83f2` — **TERM-001 phase 1** spike (`spikes/terminal-engine/**`, standalone crate; `cargo
  build`/`clippy`/`fmt --check` exit 0; real-PTY `SIGWINCH` probe `VERDICT=PASS`) + **ADR-003**
  (`Proposed`; pins `alacritty_terminal 0.26.0`, `vte 0.15.0`, `portable-pty 0.9.0`, `gpui = "=0.2.2"`).
- ADR-001/002/003 are **`Proposed`**, not Accepted — reconfirm before CORE-001.

**Blocked — DOC-002 (matrix citation repair), not applied.** Evidence is durable on branch
`research/doc-002` at `docs/compatibility/doc-002-audit/` (commit `2ff3447`; branch merged up to
`669cb84`). Findings: **zero URL drift** (21 URLs re-fetched, all sha256 identical to DOC-001);
**386 line-number citations, 193 MATCH / 115 SHIFT / 78 NOT_FOUND across 7 files** (the defect is far
broader than `events.md`). One writer pass applies `citation-audit.md` to
`docs/compatibility/reference-behavior-matrix.md`, converts un-locatable rows to
`[file.md §<heading>](url)` + the existing verbatim quote (never invent a number), updates the §4
retrieval log + "as-of" timestamps, and commits.

## Next actions (in order)

1. **DOC-002 writer pass** (READY; needs a writer with `edit`/`bash`): resume `research/doc-002`, apply
   `docs/compatibility/doc-002-audit/citation-audit.md`, verify, commit; then merge to `master`.
2. **CORE-001** — first engineering slice. Create the Cargo workspace + `relayable-model`,
   `relayable-protocol`, `relayable-pty`, `relayable-server`, `relayable-client`, `relayable-cli` per
   the ADR-001 `Proposed` layout and the interface contract. Automated integration test must prove:
   foreground server owns a PTY; CLI creates a session and returns a stable ID; terminal I/O
   round-trips; client disconnect leaves the child process alive; reconnect retrieves current state.
   Use `portable-pty 0.9.0`. Do **not** absorb `spikes/terminal-engine` into the workspace.
3. **TERM-002** (READY) — engine-level spike phase 2 that validates ADR-003's provisional engine pin.
4. Then **CORE-002**, **UI-001/UI-002** (against `docs/research/` + `design-tokens.md`), **QA-001**;
   **ADR-004** (DEC-005 ownership/persistence) and **ADR-005** (DEC-006 env namespace; ADR-002 D14
   recommends `RELAYABLE_*`-only — Manager decision pending).
5. Update `docs/agent-loop/**` after every meaningful iteration.

## Review gates (do not skip)

- No REX wire/binary compatibility claims (`full.md` §3); no GoRex source/asset reuse (§2.6).
- UI fidelity: catalogue + design tokens are the acceptance gate; unverified states stay labelled.
- Android Studio is not a dependency; commit a Gradle wrapper when Android work starts (§10).
- Never claim server-crash or host-reboot session survival (§6.3).
- Only the standalone `spikes/terminal-engine` crate exists in-repo; no Cargo workspace yet.

## Approval gates

- **Android SDK/emulator install + project JDK pin** — only needed for Android work; unresolved.

## Cleanup performed at this handoff

- All three Wave 2 Leads stopped; merged worktrees removed (`research/adr-001-002`,
  `research/term-001-adr-003`). Branches and Pi sessions preserved.
- **`research/doc-002` worktree retained** — its branch is not merged and holds the DOC-002 evidence;
  resume it for the writer pass.
- DEC-009 remains resolved (option A: automatic-rename fish hook sourcing disabled); wake daemon stopped.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
