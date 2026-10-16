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

## Current state (Manager integration session, 2026-10-15)

`master` is clean and pushed at `f987dcf`. All Wave 1/2 research, the review/fix cycle, the core slices,
the GPUI reference workspace, the QA tooling, and **all five ADRs** are integrated.

- **ADR-001** (domain/dependency), **ADR-002** (proto3+prost framing/transport), **ADR-003** (terminal
  engine + GPUI pin), **ADR-004** (process ownership/persistence), **ADR-005** (`RELAYABLE_*` env
  namespace) — **all Accepted**.
- **REVIEW-001** (`774d394`, `docs/reviews/**`) and **DOC-002** citation repair (`16c8bc9`) integrated.
- **Core:** `crates/relayable-{model,protocol,pty,server,client,cli}` + `protocol/schema` + `tests/**`;
  CORE-001 (`d3e0286`) and CORE-002 (`91662af`) integrated; `cargo test --workspace` **150 pass**.
- **UI:** `apps/linux` (separate workspace) builds/runs `gpui = "=0.2.2"` via a `libc 0.2.189` lockfile
  hold; UI-001 (`09928db`) + UI-002 (`d46c66c`) integrated.
- **QA:** `scripts/validate.sh`, `scripts/smoke.sh`, `scripts/ci.sh`; QA-001 (`4a258c2`).
- **No live projects** — all Leads stopped. Next: **CTX-001**.
- Host: `protoc` = `libprotoc 36.1`; both spike crates self-root (`[workspace]`); root workspace
  `exclude = ["spikes"]`.

## Open tasks, in order

1. **CTX-001** — set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed terminals and
   add the ADR-005 §8.1 context-resolution test (currently not implemented).
2. **STORE-001** — create `crates/relayable-store` (SQLite metadata/audit) + server start-up
   reconciliation, `flock`, prune, write path per ADR-004 (designed, not implemented).
3. Backlog: `CLIENT-API-001` (client raw-byte streaming), `MEM-001` (memory baseline),
   `CI-HOST-001` (hosted CI), `REF-004` remainder (design-token calibration).
4. **TERM-003 (proposed; user decision pending)** — libghostty-vt license / maintained-Rust-binding /
   build evidence spike (ADR-003 reconsideration trigger #10).
5. Update `docs/agent-loop/**` after every meaningful iteration.

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
- All project Leads stopped; branches and worktrees preserved (one per completed task).

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
