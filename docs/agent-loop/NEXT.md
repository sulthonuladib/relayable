# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..003`,
and this file. Trust repo state over chat.

## First commands

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
ls docs/adr docs/architecture spikes/terminal-engine docs/compatibility/doc-002-audit
```

## Current state (clean start)

`master` is clean and pushed; **all Wave 2 work is merged and all Wave 2 worktrees are retired**
(the branches and Pi sessions are preserved). Fresh worktrees can be created per task.

- `31bed30` — **ADR-001** (domain model + dependency boundaries), **ADR-002** (proto3 + `prost`;
  4-byte length-prefix framing; pathname Unix socket local; TLS 1.3 remote, off by default), and
  `docs/architecture/interface-contract.md` (`full.md` §14.4 interface note).
- `78b83f2` — **TERM-001 phase 1** PTY spike (`spikes/terminal-engine/**`; `cargo build`/`clippy`/
  `fmt --check` exit 0; real-PTY `SIGWINCH` probe `VERDICT=PASS`) + **ADR-003** (`Proposed`; pins
  `alacritty_terminal 0.26.0`, `vte 0.15.0`, `portable-pty 0.9.0`, `gpui = "=0.2.2"`).
- `a1f1719` — **DOC-002 audit evidence** (`docs/compatibility/doc-002-audit/**`).
- ADR-001/002/003 are **`Proposed`**, not Accepted — reconfirm before CORE-001.

## Open tasks, in order

1. **REVIEW-001** — independent reviewer pass over ADR-001/002 (`69e5600`) and ADR-003/spike
   (`2e41da4`) before CORE-001 builds on them; record findings. No independent review happened during
   the wind-down (wind-down forbade new agents), and ADR-002 carries in-file UNVERIFIED external claims.
2. **DOC-002 writer pass** — apply `docs/compatibility/doc-002-audit/citation-audit.md` to
   `docs/compatibility/reference-behavior-matrix.md`: fix **all 7 source files'** line numbers;
   convert unlocatable rows to `[file.md §<heading>](url)` + the existing verbatim quote (never invent
   a number); update the §4 retrieval log and "as-of" timestamps to the re-check window. Findings:
   **zero URL drift**; **386 line citations = 193 MATCH / 115 SHIFT / 78 NOT_FOUND**. Commit + merge.
3. **TERM-002** — engine-level spike phase 2 validating ADR-003's provisional engine pin.
4. **CORE-001** — first engineering slice: create the Cargo workspace + `relayable-{model,protocol,
   pty,server,client,cli}` per ADR-001/002 + the interface contract. Automated integration test must
   prove: foreground server owns a PTY; CLI creates a session and returns a stable ID; terminal I/O
   round-trips; client disconnect leaves the child process alive; reconnect retrieves current state.
   Use `portable-pty 0.9.0`. Do **not** absorb `spikes/terminal-engine` into the workspace.
5. Then **CORE-002**, **UI-001/UI-002** (against `docs/research/` + `design-tokens.md`), **QA-001**;
   **ADR-004** (DEC-005 ownership/persistence) and **ADR-005** (DEC-006 env namespace; ADR-002 D14
   recommends `RELAYABLE_*`-only — Manager decision pending).
6. Update `docs/agent-loop/**` after every meaningful iteration.

## Review gates (do not skip)

- **Independent-review gap (REVIEW-001):** do the reviewer pass before building CORE-001 on the ADRs.
- No REX wire/binary compatibility claims (`full.md` §3); no GoRex source/asset reuse (§2.6).
- UI fidelity: catalogue + design tokens are the acceptance gate; unverified states stay labelled.
- Android Studio is not a dependency; commit a Gradle wrapper when Android work starts (§10).
- Never claim server-crash or host-reboot session survival (§6.3).
- Only the standalone `spikes/terminal-engine` crate exists in-repo; no Cargo workspace yet.

## Approval gates

- **Android SDK/emulator install + project JDK pin** — only needed for Android work; unresolved.

## Environment / cleanup

- Steering can use fresh bounded agents again; one writer per branch/path.
- DEC-009 is resolved (option A: automatic-rename fish hook sourcing disabled in
  `~/.config/fish/config.fish`; backup `/tmp/relayable-config.fish.bak`) **and the
  `herdr-automatic-rename` plugin is fully disabled** (`herdr plugin list` → `disabled`). Re-enable
  with `herdr plugin enable herdr-automatic-rename` (and uncomment the hook block if you also want
  per-command renaming back). If delegation fails with `agent_pane_busy`, restart the fallback daemon:
  `setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 3600 <workspace-ids…>`.
- All Wave 2 Leads stopped; all Wave 2 branches merged; worktrees retired. Branches preserved.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
