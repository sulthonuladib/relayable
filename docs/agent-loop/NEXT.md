# NEXT — exact current state and next action

Read `full.md` completely, then repository instructions (`AGENTS.md`), `STATUS.md`, `BACKLOG.md`,
`TASKS.md`, `DECISIONS.md`, `TESTS.md`, `WORKLOG.md` (tail), and this file. Trust these artifacts
over prior chat.

## First command to run

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -6 && git worktree list
```

> Path note: the Phase 0 kickoff WORKLOG recorded `/home/sulthonuladib/personal/relayable`; that path
> no longer exists. The checkout is `/home/sulthonuladib/work/relayable`.

## Context-window policy (DEC-008)

- `.pi/settings.json` sets `compaction.enabled: false`; never invoke `/compact` unless the user
  explicitly asks. Global settings already had the same value.
- Repo files are authoritative; keep `docs/agent-loop/` current. Do not depend on conversation
  summaries.
- Delegate bounded work to fresh agent sessions; keep handoffs concise; do not attach full research
  reports or `full.md` to task briefs.
- At context pressure: checkpoint, stop launching new work, record results, write a short handoff.

## Current phase and state

Phase 0 — Wave 1 research is executing under the Option D stopgap. `research/env-audit` (ENV-001) is
complete and integrated into `master` (`b24133c`); `research/ui-reference` (REF) has committed wave 1
and is writing wave 2/3; `research/protocol-compat` (DOC) has its writers in flight. The permanent
Herdr fix is still the pending DEC-009 decision.

Recovery options (do not apply without the user's decision — DEC-009):

- **A (recommended):** stop sourcing the hook in `~/.config/fish/config.fish` lines 40–43. New panes
  then start agents reliably; no Herdr or Lead restart is needed; live per-command tab auto-renaming
  stops (cosmetic; event-based naming still works). Reversible.
- **B:** add a project-local `managed-lead` overlay granting Leads bash/edit/write, then
  `stop_lead` + `resume_project` each Lead so they execute directly. Repo-only and reversible;
  changes the in-repo delegation model while the environment is faulty.
- **C:** update/reinstall the plugin (unproven) or report upstream.

Until a fix is applied, any Herdsman agent start needs a pane wake: run a bounded background loop
that sends `herdr pane send-keys <pane> Enter` every ~0.3 s during the launch, or expect
`agent_pane_busy`. Do not remove worktrees or restart Herdr without explicit user confirmation.

**Stopgap active:** `/tmp/relayable-wake-daemon.py` is running (pid file
`/tmp/relayable-wake-daemon.pid`, stop file `/tmp/relayable-wake-daemon.stop`, expires ~00:05) with
0 `agent_start` failures so far. Restart it if a later wave still needs agent starts; stop it once
the permanent fix is applied or the wave settles (`WORKLOG.md` Checkpoints 4–5).

| Branch | Worktree | Produces | Exclusive paths |
|---|---|---|---|
| `research/ui-reference` | `~/.herdr/worktrees/relayable/research-ui-reference` (wX) | `docs/research/{video-review,ui-catalog,open-questions,gorex-review}.md` | `docs/research/**` |
| `research/protocol-compat` | `~/.herdr/worktrees/relayable/research-protocol-compat` (wY) | `docs/compatibility/reference-behavior-matrix.md` | `docs/compatibility/**` |
| `research/env-audit` | `~/.herdr/worktrees/relayable/research-env-audit` (wZ) | `docs/operations/toolchain-audit.md` (+ optional `android-toolchain.md`) | `docs/operations/**` |

The branches are at `3db73c2` (policy + evidence commits) and contain **no** research commits yet.

## Next actions (in order)

1. Apply the user's chosen fix; with Option A, message each Lead to retry delegation without
   restarting it; with Option B, `stop_lead` + `resume_project` each Lead so the overlay applies.
2. When reports arrive: review each diff (`git log --stat`, `git show`), verify honesty (access
   routes, confidence, no invented values), confirm no out-of-scope edits; then INT-001 merge into
   `master` (paths are disjoint, conflicts should be zero).
3. Wave 2, in order: `TERM-001` spike + `ADR-003`; `ADR-001`/`ADR-002`; `CORE-001` (server-owned PTY,
   CLI-created session, terminal I/O, reconnection) with an automated integration test; `CORE-002`;
   `UI-001`/`UI-002` against the integrated UI catalogue.
4. Update `BACKLOG.md`, `TESTS.md`, `WORKLOG.md`, `NEXT.md`, `STATUS.md` after every meaningful
   iteration; add a `DECISIONS.md` entry when a decision changes.

## Review gate for Wave 1 (do not skip)

- Do **not** claim video/UI fidelity before frames were actually inspected (§2.7).
- Do **not** claim REX wire/binary compatibility (§3).
- Do **not** copy GoRex source or assets; verify license first (§2.6).
- Treat research docs as evidence to spot-check, not truth to trust.

## Hard constraints

- No global installs, `sudo`, system config changes, or opened ports (§0.3.10–11).
- `gradle` is absent → use a committed Gradle wrapper; Android Studio is not permitted (§10).
- Never claim server-crash or reboot session survival (§6.3).
- Preserve unrelated user work; keep unrelated formatting changes out of commits.

## Known environment facts (verified)

rustc/cargo 1.98.1 · mise present · JDK 25 (Temurin, via mise) · `android` CLI + `adb` present ·
`gradle` absent · `ffmpeg`/`yt-dlp`/`chromium`/`curl` present · Node 26.8.2 / npm 11.19.1 (mise) ·
`/dev/kvm` present · `systemd --user` running · Linux x86_64 omarchy · herdr 0.9.5 · pi 1.2.0 ·
pi-herdsman 0.27.0 · ~51G free on `/home` · repo `.gitignore` excludes `/docs/research/evidence/`
and `**/evidence-local/`.

## Open questions for the user (only if genuinely blocking)

None at this checkpoint. Decisions that will eventually need user input:

- distribution/packaging target confirmation (`.deb` vs `.rpm` vs both, §7.3);
- the `RELAYABLE_*` vs REX-compat environment-variable namespace decision (§8.1, DEC-006);
- **Android SDK/emulator install (approval gate for AND-001):** ENV-001 found no Android SDK,
  sdkmanager/avdmanager/emulator, AVDs, system images, or licenses, and no project JDK pin
  (JDK 25 only; AGP-on-JDK-25 unverified). Downloading/installing the SDK and accepting licenses
  requires explicit user approval; disk is 68% used. Evidence:
  `docs/operations/android-toolchain.md`.
