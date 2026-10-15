# Relayable — agent instructions

Durable product specification **and** agent-loop instruction: **`full.md`** (read it completely before any architectural decision).

- Product name: **Relayable**. Company: **Superlogical**. Never label the product "REX" outside research/compatibility evidence (`full.md` §0.1).
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

## Path ownership

The lead owns shared/cross-cutting files: `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, `crates/relayable-model/**`, `docs/adr/**`, `docs/agent-loop/**`, and integration. A lead-owned path may be delegated to a bounded worker only by naming it explicitly in that worker's assignment (the lead still integrates).

Workers own only the paths named in their assignment.
