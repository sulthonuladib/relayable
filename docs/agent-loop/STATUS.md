# STATUS — Relayable

**Phase:** 0 — Repository, evidence audit, and architecture decisions
**Milestone:** Phase 0 exit gate (§16): evidence status clear, ADRs recorded, buildable minimal shell exists.
**Updated:** Phase 0 kickoff checkpoint (see `WORKLOG.md` for raw evidence)

## Objective (current)

Stand up the evidence base and the first working vertical slice:

1. Honest, timestamped evidence for all five reference videos + GoRex + official REX docs.
2. Verified toolchain inventory; no installs.
3. Domain/dependency and protocol ADRs.
4. Smallest useful vertical slice: server-owned PTY, CLI-created session, terminal I/O, disconnect survives, reconnect recovers.

## Repository state

- Branch `master`. Commits: `1b0d8c8 init and commit full.md`, `7181f92 plan update`.
- At Phase 0 start the repo contained **only** `full.md` (1279 lines). No `AGENTS.md`, no `Cargo.toml`, no source, no CI.
- No pre-existing user code changes were present to preserve; nothing was discarded.

## Blockers

None currently. Video access is expected to be feasible (`yt-dlp`, `ffmpeg`, `chromium` are installed) — this must still be proven per-video and honestly labelled if a specific video fails.

## Known risks

- GPUI is under active development and may have breaking API changes; it must be pinned and isolated inside `apps/linux` (§5.3).
- The original REX wire protocol is **not** documented; binary/wire compatibility must not be claimed (§3).
- GoRex is a clone reference only. Its license was previously not found at the expected path — re-verify before any reuse. No GoRex source or assets may be copied (§2.6).
- Terminal engine choice is unresolved and gates UI work (§5.4).

## Guarantees that must NOT be claimed

Server-crash or host-reboot session survival is **not** guaranteed and must not be claimed unless a supervisor design is implemented and tested (§1.3.10, §6.3).
