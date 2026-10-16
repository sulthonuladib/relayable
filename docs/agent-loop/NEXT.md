# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..005`,
and this file. Trust repo state over chat.

> **Restart trigger (DEC-008 — compaction is disabled):** if this session approaches its context
> limit, STOP launching new work, finish a safe checkpoint (commit + push + update
> `docs/agent-loop/**`), and tell the user to start a fresh session with the next-session prompt.
> Repository files are authoritative; never rely on conversation memory. Project work is durable by
> Git branch, so a restart loses nothing. Manager auto-start is enabled, so a fresh session opened in
> the primary workspace re-takes the Manager role; otherwise the user pastes the prompt.

## First commands

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
ls docs/adr docs/architecture spikes/terminal-engine docs/compatibility/doc-002-audit docs/reviews
```

## Current state (Manager session, 2026-10-16)

`master` is clean at `416fa51`. Baseline re-verified: `cargo build --workspace --offline` and
`cargo test --workspace --offline` both exit 0 with 0 failures. All Wave 1/2 research, the review/fix
cycle, the core slices, the GPUI reference workspace, the QA tooling, and **all five ADRs** are
integrated. **DEC-011** (2026-10-16) clarifies ADR-005 Decision 5.

**In flight (two disjoint-path implementer projects):**

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

1. **CLIENT-API-001 (in flight)** — `client/client-api-001-incremental-output` (Herdr `w1H`): public
   `Client::next_output` incremental terminal-bytes API + backpressure test so the GPUI view can drive a
   live terminal.
2. **STORE-001 (in flight)** — `store/store-001-sqlite-metadata` (Herdr `w1J`): `crates/relayable-store`
   (SQLite metadata/audit) + server start-up reconciliation, `flock`, prune, write path per ADR-004.
   DEC-012: `rusqlite =0.40.2` bundled (one-time fetch done; builds stay offline).
3. **Integrate each returning project** — review the diff, run independent validation
   (`scripts/validate.sh`, `scripts/smoke.sh`, `cargo test --workspace --offline`), merge (INT-017+).
4. Backlog: `MEM-001` (memory baseline), `REF-004` remainder, `CI-HOST-001`.
5. **TERM-003 (proposed; user decision pending)** — libghostty-vt license / maintained-Rust-binding /
   build spike (ADR-003 reconsideration trigger #10).
6. Update `docs/agent-loop/**` after every meaningful iteration.

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

- Steering can use fresh bounded agents again; one writer per branch/path.
- DEC-009 is resolved (the automatic-rename fish hook block was removed from
  `~/.config/fish/config.fish`; backup `/tmp/relayable-config.fish.bak`) **and the
  `herdr-automatic-rename` plugin is uninstalled** (`herdr plugin list` → `No plugins installed.`;
  residual dirs at `/tmp/herdr-automatic-rename-removed/`). If delegation fails with `agent_pane_busy`,
  restart the fallback daemon:
  `setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 3600 <ids…>`.
- Worktree cleanup (AGENTS.md "Worktree lifecycle / cleanup"): done+merged task worktrees may be removed
  by the Manager, preserving branches. In-flight worktrees (`client/...`, `store/...`) are kept. Check
  `git worktree list` after any cleanup and record it in `WORKLOG.md`.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
