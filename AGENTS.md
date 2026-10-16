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

- Automatic compaction is disabled (`.pi/settings.json` → `compaction.enabled: false`). Never
  invoke `/compact` unless the user explicitly asks.
- Project knowledge lives in the `docs/agent-loop/` files above, not in conversation summaries.
- Delegate bounded work to fresh agent sessions. Handoffs must be concise: files changed, validation
  actually run, findings/decisions, risks, remaining work. Do not inject full research reports or
  `full.md` into task briefs; point at repository paths instead.
- Before context pressure risks unfinished work: finish a safe checkpoint, stop launching new work,
  record agent results and repository changes, then write a short handoff. Do not compact to keep a
  session alive.
- A resumed session reads `full.md`, repository instructions, `NEXT.md`, `BACKLOG.md`,
  `DECISIONS.md`, and Git state, then continues at the next incomplete acceptance criterion.

## Loop (summary of §15)

recover context → inspect reality (`git status`) → select the next vertical slice → split bounded work with path ownership → define interface → implement → validate with real commands → review diff → fix regressions → checkpoint → continue.

## Verified local environment

See `docs/agent-loop/WORKLOG.md` for raw output. Summary:

| Tool | State |
|---|---|
| rustc / cargo | 1.98.1, `stable-x86_64-unknown-linux-gnu` (default) |
| protoc | `/usr/bin/protoc` → `libprotoc 36.1` (required by `prost-build`; verified 2026-10-15 by REVIEW-001) |
| mise | `/usr/bin/mise` |
| android CLI | `/home/sulthonuladib/.local/bin/android` |
| adb | `/usr/bin/adb` |
| java | Temurin 25.0.4.1 LTS via mise (`mise/installs/java/temurin-25`) |
| gradle | **absent** — use the Gradle wrapper, never a system Gradle |
| ffmpeg / yt-dlp / chromium | present — video frame extraction and browsing are feasible |
| OS | Linux `falcon` 7.2.5-3-omarchy, x86_64 |
| herdr | 0.9.5 |

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

- **Never poll peers.** Do not sleep-wait, status-check, or `list_agents`/`list_staff` another agent to see
  whether it finished. Agents are pinged immediately when a dependency settles (event-driven).
- While waiting on a dependency, do independent bounded work on your owned paths, or stop and let the
  completion notification resume you. Do not invent work merely to stay active.
- Use ownership/status reads only for a real control or recovery decision, never as a progress poll.
- Report once at a natural boundary; the Manager integrates returning work.

## Path ownership

The lead owns shared/cross-cutting files: `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, `crates/relayable-model/**`, `docs/adr/**`, `docs/agent-loop/**`, and integration. A lead-owned path may be delegated to a bounded worker only by naming it explicitly in that worker's assignment (the lead still integrates).

Workers own only the paths named in their assignment.

## Worktree lifecycle / cleanup

The Manager keeps the checkout set tidy. A task's worktree may be removed **only** when its work is merged
into `master` and pushed and the task is recorded `DONE`; the Git branch and Pi conversation are preserved.

- Never remove a worktree whose task is in flight/unmerged, and never remove one with uncommitted changes.
- **Standing user authorization (2026-10-16):** for done+merged worktrees the Manager removes them
  automatically with **no per-removal confirmation**. Use plain `git worktree remove <path>` (no prompt);
  use `remove_worktree` only when you also want the Herdsman assignment retired. Both preserve the Git
  branch. `scripts/cleanup-worktrees.sh <branch>...` (add `--apply` for real removal) does a checked,
  non-interactive cleanup of explicitly named, merged, clean branches.
- After removing, re-check `git worktree list`, keep `docs/agent-loop/NEXT.md` current, and record the
  cleanup in `WORKLOG.md`.
- Anything not done+merged still needs an explicit user request; never force-remove.
