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

`docs/agent-loop/` — `STATUS.md`, `TASKS.md`, `DECISIONS.md`, `WORKLOG.md`, `NEXT.md` (§15).
At the start of a resumed run, trust these artifacts over prior conversation.

## Loop (summary of §15)

recover context → inspect reality (`git status`) → select the next vertical slice → split bounded work with path ownership → define interface → implement → validate with real commands → review diff → fix regressions → checkpoint → continue.

## Verified local environment

See `docs/agent-loop/WORKLOG.md` for raw output. Summary:

| Tool | State |
|---|---|
| rustc / cargo | 1.98.1, `stable-x86_64-unknown-linux-gnu` (default) |
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

The lead owns shared/cross-cutting files: `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, `crates/relayable-model/**`, `docs/adr/**`, `docs/agent-loop/**`, and integration.

Workers own only the paths named in their assignment.
