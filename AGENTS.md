# Relayable — agent instructions

Durable product specification **and** agent-loop instruction: **`full.md`** (read it completely before any architectural decision).

- Product name: **Relayable**. Relayable is an independent project — do not attribute it to Superlogical (the maker of the reference product). Never label the product "REX" outside research/compatibility evidence (`full.md` §0.1).
- Repository state: **Phase 0** — spec-only repo at start; see `docs/agent-loop/STATUS.md`.

## Non-negotiables

1. UI fidelity to the five reference videos is a **primary acceptance criterion** (§0.2, §2.4). Do not invent a generic redesign.
2. Never invent CLI flags, GPUI APIs, Android CLI commands, library APIs, or protocol behavior. Verify (§0.3.5).
3. Never fake tests, screenshots, video review, measurements, benchmarks, or emulator/device runs (§0.3.7).
4. Do not `sudo`, install global packages, change system configuration, modify global shell startup, open firewall ports, or deploy remotely without explicit user approval (§0.3.10–11, §11).
5. Preserve pre-existing user work. Keep unrelated formatting changes out of commits (§0.3.8).
6. Independent agents must not write overlapping files. Delegate by bounded task and **file ownership**; the lead integrates (§0.3.14).

## Work state lives in files, not chat

`docs/agent-loop/` — `STATUS.md`, `TASKS.md`, `BACKLOG.md`, `DECISIONS.md`, `TESTS.md`,
`WORKLOG.md`, `NEXT.md` (§15). At the start of a resumed run, trust these artifacts over prior
conversation.

## Context-window policy

- Automatic compaction is disabled (`opencode.json` → `compaction.auto: false`), per DEC-008. Never
  compact a session unless the user explicitly asks.
- Project knowledge lives in the `docs/agent-loop/` files above, not in conversation summaries.
- Delegate bounded work to fresh agent sessions. Handoffs must be concise: files changed, validation
  actually run, findings/decisions, risks, remaining work. Do not inject full research reports or
  `full.md` into task briefs; point at repository paths instead.
- Before context pressure risks unfinished work: finish a safe checkpoint, stop launching new work,
  record agent results and repository changes, then write a short handoff. Do not compact to keep a
  session alive.
- A resumed session reads `full.md`, repository instructions, `NEXT.md`, `BACKLOG.md`,
  `DECISIONS.md`, and Git state, then continues at the next incomplete acceptance criterion.
- `scripts/agent-loop.sh` continues the loop as a sequence of **fresh** orchestrator sessions
  (deliberately no `--continue`), so context never accumulates across iterations. It refuses to
  start on a dirty tree. `NEXT-SESSION-PROMPT.md` is the manual equivalent. Knobs are
  `LOOP_AGENT` and `LOOP_DRY_RUN` (`LOOP_`-prefixed because the ambient environment already
  exports a generic `AGENT`).

## Loop (summary of §15)

recover context → inspect reality (`git status`) → select the next vertical slice → split bounded work with path ownership → define interface → implement → validate with real commands → review diff → fix regressions → checkpoint → continue.

## Verified local environment

See `docs/agent-loop/WORKLOG.md` for raw output. Summary:

| Tool | State |
|---|---|
| rustc / cargo | 1.99.1, `stable-x86_64-unknown-linux-gnu` (default); no `rust-toolchain.toml` pin. The tree was documented at 1.98.1 until 2026-10-16 (DEC-018: host-level bump; workspace, desktop workspace, `validate.sh` and `smoke.sh` all green on 1.99.1) |
| protoc | `/usr/bin/protoc` → `libprotoc 36.1` (required by `prost-build`; verified 2026-10-15, found absent 2026-10-16, restored by the user the same day) |
| offline cargo cache | **partial — see DEC-018 (2026-10-16).** The root workspace builds/tests offline (`scripts/validate.sh` all 8 steps, `scripts/smoke.sh` OK) and the `apps/linux` set is cached again (`CARGO_NET_OFFLINE=true cargo build --locked` → exit 0), but `spikes/terminal-engine-phase2` (`avt`, …) is **not** cached, so that one `ci.sh` row cannot pass offline. Do **not** run a networked `cargo fetch` at agent initiative; the DEC-012 pattern requires user approval. Always prefer `--offline` / `CARGO_NET_OFFLINE=true` |
| mise | `/usr/bin/mise` — **`mise.toml` (repo root) pins `zig = "0.16.0"`** (user-provided 2026-10-16, committed): `zig` shims resolve inside this checkout, but **not** outside it (a tool invoked from another directory needs the absolute path, e.g. `~/.local/share/mise/installs/zig/0.16.0/zig`). Zig is required only for the TERM-003/libghostty work, not for the Rust workspace |
| android CLI | `/home/sulthonuladib/.local/bin/android` |
| adb | `/usr/bin/adb` |
| java | Temurin 25.0.4.1 LTS via mise (`mise/installs/java/temurin-25`) |
| gradle | **absent** — use the Gradle wrapper, never a system Gradle |
| ffmpeg / yt-dlp / chromium | present — video frame extraction and browsing are feasible |
| OS | Linux `falcon` 7.2.5-3-omarchy, x86_64 |
| opencode | 2.0.30 — agent runtime and orchestrator; replaced the herdr + Pi stack (DEC-015) |

Android Studio is **not** required and must not be a build dependency (§10).

## Code organization

Keep hand-written Rust files small and focused; the binding standard is
`docs/architecture/code-organization.md`. Summary: aim ~500 lines, **hard cap 1000** for production source;
tests 800/**1200**; `lib.rs`/`main.rs`/`mod.rs` thin (≤150, cap 300). Generated code and `spikes/**` are
exempt. Split by responsibility (one module, one job) using `foo.rs` + `foo/` submodules and `pub use`
re-exports, so a split never changes a public path or behavior. `scripts/check-file-sizes.sh` reports
offenders; `--strict` becomes a CI gate after the refactor queue (`REF-005`…`REF-010`) is done. Never
refactor a path while another task owns it (`docs/architecture/code-organization.md` sequencing table).

## Agent coordination (no polling)

- **Never poll peers.** Do not sleep-wait or status-check another agent to see whether it finished.
  A background `subagent` session notifies its parent on completion (event-driven). Reading the
  session list is for a real control or recovery decision, never a progress poll.
- While waiting on a dependency, do independent bounded work on your owned paths, or stop and let the
  completion notification resume you. Do not invent work merely to stay active.
- Use ownership/status reads only for a real control or recovery decision, never as a progress poll.
- Report once at a natural boundary; the orchestrator integrates returning work.

## Agent roles (OpenCode)

Defined in `.opencode/agents/`; the orchestrator is the project `default_agent` (`opencode.json`).
Delegation is **one level deep** — OpenCode's subagent nesting depth is 1, so subagents cannot spawn
subagents. The orchestrator spawns these directly.

| Agent | Mode | May | Must not |
|---|---|---|---|
| `orchestrator` | primary | select the next slice; spawn the roles below; integrate; own the lead paths | read `full.md` into its own context by reflex; paste reports into chat |
| `planner` | subagent | read everything; return bounded briefs naming owned paths | edit any file |
| `researcher` | subagent | read + web; author `docs/research/**`, `docs/compatibility/**` | edit code, `AGENTS.md`, `docs/agent-loop/**`, `docs/adr/**`; invent APIs |
| `implementer` | subagent | edit + shell on its assigned paths; run real validation | touch `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, `opencode.json`, `docs/agent-loop/**`, `docs/adr/**`, `crates/relayable-model/**`, `.opencode/**` |
| `verifier` | subagent | read + run read-only validation; report findings by severity | edit anything |

Subagents inherit their own configured permissions, not the parent's. A role that needs a
lead-owned path may write it **only** when the brief names that path explicitly; the orchestrator
still integrates. Peer sessions (a second lead) are started by the user, not by an agent.

**Model:** every role runs on the pinned model `opencode-go/deepseek-v4.1-flash` (`opencode.json` →
`model`, DEC-016). Subagents inherit the session model, so there is one place to change it. Do not
leave a session on an unpinned default: the ambient default is a free model, not the paid one.

## Path ownership

The lead owns shared/cross-cutting files: `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, `crates/relayable-model/**`, `docs/adr/**`, `docs/agent-loop/**`, and integration. A lead-owned path may be delegated to a bounded worker only by naming it explicitly in that worker's assignment (the lead still integrates).

Workers own only the paths named in their assignment.

## Worktree lifecycle / cleanup

The lead keeps the checkout set tidy. A task's worktree may be removed **only** when its work is merged
into `master` and pushed and the task is recorded `DONE`; the Git branch and the OpenCode session are
preserved.

- Never remove a worktree whose task is in flight/unmerged, and never remove one with uncommitted changes.
- **Standing user authorization (2026-10-16):** for done+merged worktrees the lead removes them
  automatically with **no per-removal confirmation**. Use plain `git worktree remove <path>` (no prompt),
  or the `worktree_remove` tool for a worktree OpenCode created. Both preserve the Git branch.
  `scripts/cleanup-worktrees.sh <branch>...` (add `--apply` for real removal) does a checked,
  non-interactive cleanup of explicitly named, merged, clean branches.
- After removing, re-check `git worktree list`, keep `docs/agent-loop/NEXT.md` current, and record the
  cleanup in `WORKLOG.md`.
- Anything not done+merged still needs an explicit user request; never force-remove.
