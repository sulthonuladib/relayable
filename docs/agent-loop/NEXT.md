# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..003`,
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

## Current state (Manager integration session, 2026-10-15)

`master` is clean and pushed at `dca5263`. The Wave 1/2 research and the review/fix cycle are integrated.

- **ADR-001** (domain model + dependency boundaries) and **ADR-002** (proto3 + `prost`; 4-byte
  length-prefix framing; pathname Unix socket local; TLS 1.3 remote, off by default) — **Accepted**
  (`31bed30`, corrections via INT-007 `f474c83`).
- **ADR-005** (env namespace `RELAYABLE_*`-only, no `REX_*` aliases) — **Accepted** (`74e7e20`,
  `99c4c0c`); DEC-006 resolved.
- **ADR-003** (terminal engine + GPUI pin) — **`Proposed`**; engine-level spike **TERM-002** is the hard
  gate for the engine choice, **UI-001** gates the `gpui = "=0.2.2"` pin.
- **REVIEW-001** (`774d394`, report + evidence in `docs/reviews/**`) and **DOC-002** citation repair
  (`16c8bc9`) integrated.
- `docs/architecture/interface-contract.md` is frozen and uses `PtyProcessId`.
- **In flight:** **CORE-001** (`core/core-001-vertical-slice`) and **TERM-002** (`spike/term-002-engine-validation`).
- Host: `protoc` = `libprotoc 36.1`; both spike crates self-root with an empty `[workspace]` table.

## Open tasks, in order

1. **CORE-001** (in progress) — Cargo workspace + `relayable-{model,protocol,pty,server,client,cli}`
   per ADR-001/002 + the interface contract. The automated integration test must prove: foreground
   server owns a PTY; CLI creates a session and returns a stable ID; terminal I/O round-trips; client
   disconnect leaves the child alive; reconnect retrieves current state. Use `portable-pty 0.9.0`; do
   **not** adopt `alacritty_terminal`/`vte` (TERM-002 gate); do not absorb `spikes/**`.
2. **TERM-002** (in progress) — engine-level spike validating ADR-003's provisional pin; verdict
   accept as-is / accept with caveats / change the pin.
3. **CORE-002** — disconnect survival + reattach snapshot.
4. **UI-001/UI-002** — GPUI workspace against `docs/research/` + `design-tokens.md`; UI-001 also proves
   the `gpui` pin builds/runs on this host.
5. **QA-001** — smoke/CI scripts and build matrix.
6. **ADR-004** (DEC-005 ownership/persistence) — after CORE-002.
7. **TERM-003 (proposed; user decision pending)** — libghostty-vt license / maintained-Rust-binding /
   build evidence spike (ADR-003 reconsideration trigger #10).
8. Update `docs/agent-loop/**` after every meaningful iteration.

## Review gates (do not skip)

- ADR-001/002/005 are **Accepted**; ADR-003 stays `Proposed` until TERM-002 (engine) and UI-001
  (GPUI pin) report.
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
- All Wave 2 Leads stopped; all Wave 2 branches merged; worktrees retired. Branches preserved.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
