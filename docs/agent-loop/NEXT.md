# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), and this file. Trust repo state over chat.

## First commands

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8
git worktree list && ls docs/research docs/compatibility docs/operations
```

## Context-window policy (DEC-008)

- `.pi/settings.json` sets `compaction.enabled: false`; never invoke `/compact` unless the user
  explicitly asks. Repo files are authoritative; use fresh bounded agents and concise handoffs; do
  not inject full reports or `full.md` into task briefs. Checkpoint before context pressure.

## Current state (end of Wave 1 — research complete and integrated)

`master` contains all Wave 1 research:

| Path | Content | Merge |
|---|---|---|
| `docs/operations/` | `toolchain-audit.md`, `android-toolchain.md` (ENV-001) | `b24133c` (tip `a31a839`) |
| `docs/research/` | `video-review.md`, `gorex-review.md`, `ui-catalog.md`, `open-questions.md`, `design-tokens.md` (REF-001..003) | `c0578ca` (tip `269018c`) |
| `docs/compatibility/` | `reference-behavior-matrix.md` (DOC-001) | `49795a0` (tip `9b0fe37`) |

INT-001 is complete. All three research Leads are idle with open assignments and clean worktrees;
nothing was pushed; no tracked media exists in any branch.

## Environment / operational facts

- Host: Linux `falcon`; rustc/cargo 1.98.1; pi 1.2.0; pi-herdsman 0.27.0; herdr 0.9.5; Node 26 via
  mise; Android CLI 1.0.16500706 + adb 1.0.41 + KVM + Temurin 25.0.4.1; no Gradle (wrapper required);
  no Android SDK/images/licenses.
- Worktrees: primary `/home/sulthonuladib/work/relayable` (`master`); linked research worktrees at
  `~/.herdr/worktrees/relayable/research-{ui-reference,protocol-compat,env-audit}`.
- **Herdr agent-start fault is NOT fixed.** `herdr agent start` fails with `agent_pane_busy` while
  the automatic-rename fish hook spawns background bash on every prompt. The Option D wake daemon
  was **stopped** (`/tmp/relayable-wake-daemon.stop`). Before any future agent delegation either
  apply DEC-009 option A (comment out the hook source in `~/.config/fish/config.fish` lines 40–43)
  or restart the daemon: `setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 3600 wX wY wZ`.
  The daemon only helps panes in workspaces `wX/wY/wZ`; new worktrees/workspaces need their IDs added.
- Android SDK/emulator install, license acceptance, and a project JDK pin are an approval gate.

## Next actions (in order)

1. Spot-check the integrated research docs while consuming them (they are evidence, not truth):
   cited URLs/timestamps/frame references were verified by the Leads' own review loops, but re-check
   any claim an ADR depends on.
2. **TERM-001 + ADR-003:** terminal-engine candidates, GPUI pin decision (crates.io `gpui` 0.2.2
   Apache-2.0 vs `Glass-HQ/gpui` with no root LICENSE), then the terminal spike.
3. **ADR-001 + ADR-002:** domain/dependency boundaries and protocol serialization/transport, using
   `docs/compatibility/reference-behavior-matrix.md`. No REX wire/binary compatibility claims;
   DEC-006 (`RELAYABLE_*` vs REX aliases) still open.
4. **CORE-001** (server-owned PTY + CLI-created session + terminal I/O + reconnection) with an
   automated integration test, then **CORE-002**; **UI-001/UI-002** against `docs/research/`
   catalogue + `design-tokens.md`; **QA-001** smoke/CI.
5. Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md` after every iteration.

## Review gates (do not skip)

- UI fidelity: frames were actually inspected (see `docs/research/video-review.md`); no generic
  redesign; unverified states stay labelled.
- No REX wire/binary compatibility claims (§3).
- No GoRex source/asset reuse; no repo-level license found (§2.6).
- Android Studio is not a dependency; commit a Gradle wrapper (§10).
- Never claim server-crash or host-reboot session survival (§6.3).

## Open questions (approval gates)

- **DEC-009:** Herdr fault remediation — A) stop sourcing the automatic-rename fish hook
  (recommended), B) project-local `managed-lead` overlay with write tools, C) update/reinstall the
  plugin. Without a fix or the daemon, agent starts fail.
- **Android:** install SDK/emulator + accept licenses + choose project JDK pin? (user approval)
- **Packaging target** (`.deb`/`.rpm`) and **DEC-006 env namespace** can wait.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
