# WORKLOG — Relayable

Append-only checkpoint log. Each entry: objective, files changed, exact commands + exit codes, tests run, evidence paths, risks, next tasks.

---

## Checkpoint 1 — Phase 0 kickoff: repository and toolchain discovery (`full.md` §0.4, §19)

**Objective:** Record actual repository and toolchain state before any code changes. Preserve existing work.

**Files changed in this checkpoint:** `AGENTS.md`, `docs/agent-loop/{STATUS,TASKS,DECISIONS,WORKLOG,NEXT}.md` (new). No source files created yet. Nothing deleted.

### Commands run and observed results

`pwd` → `/home/sulthonuladib/personal/relayable`

`git rev-parse --abbrev-ref HEAD` → `master`

`git status --short` → *(no output — clean working tree)*

`git log --oneline -20` →
```
7181f92 plan update
1b0d8c8 init and commit full.md
```

`ls -la` → only `full.md` (80874 bytes) and `.git/`

`wc -l -c full.md` → `1279 80874`

`find . -maxdepth 2 -type f -not -path './.git/*' | sort` → `./full.md` *(only file in repo)*

`find .. \( -name AGENTS.md -o -name AGENTS.override.md \)` → no `AGENTS.md` inside the Relayable repo. Matches exist only in unrelated sibling projects (`../discord-bot`, `../fold_deez`, `../dillon`, `../scratchpad`, etc.) and in `node_modules` — none apply to this repository.

### Toolchain (`full.md` §0.4)

| Command | Observed |
|---|---|
| `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| `rustup show active-toolchain` | `stable-x86_64-unknown-linux-gnu (default)` |
| `command -v mise` | `/usr/bin/mise` — **present** |
| `command -v android` | `/home/sulthonuladib/.local/bin/android` — **present** |
| `command -v adb` | `/usr/bin/adb` — **present** |
| `command -v java` | `/home/sulthonuladib/.local/share/mise/installs/java/temurin-25/bin/java` |
| `java -version` | `openjdk version "25.0.4.1" 2026-08-18 LTS` — Temurin 25.0.4.1+1 |
| `command -v gradle` | **ABSENT** — Gradle wrapper required (§10.1.7) |
| `command -v ffmpeg` | `/usr/bin/ffmpeg` — **present** (frame extraction feasible) |
| `command -v yt-dlp` | `/usr/bin/yt-dlp` — **present** (video retrieval feasible) |
| `command -v chromium` | `/usr/bin/chromium` — **present** (browsing feasible) |
| `uname -a` | `Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux` |
| `herdr -V` | `herdr 0.9.5` |
| `echo $SHELL` / `$HERDR_ENV` | `/usr/bin/fish` / `1` (tool shell executes as `bash`) |

### Interpretation

- The repository is **empty except for the specification** → `full.md` §0.4 "if the repository is empty, establish a minimal workspace" applies. No existing architecture to preserve or migrate.
- **No blocker for video research:** `yt-dlp`, `ffmpeg`, and `chromium` are all installed, so the §2.1 workflow can proceed to actual frame inspection. This must still be proven per-video, and failures recorded honestly rather than assumed away.
- **No blocker for Android tooling discovery:** `android` CLI, `adb`, and JDK 25 are present. Only `gradle` is missing, which is expected — §10.1.7 requires committing a Gradle wrapper and never depending on a system Gradle.
- `mise` is present, so if a toolchain pin is later needed, `mise.toml` is the compatible mechanism (§0.3.11) — not silent global installation.

### Tests run

None. No build system exists yet. No test claimed.

### Risks / caveats

- `java -version` is JDK **25**; Android Gradle Plugin compatibility with JDK 25 is unverified and must be checked before any Android build attempt (§10.1.4).
- Toolchain versions above are recorded, not validated against project requirements — no `Cargo.toml`/`rust-toolchain.toml` exists yet to pin against.

### Next tasks (ordered)

1. BOOT-002 — commit scaffold (this checkpoint).
2. REF-001/002/003, DOC-001, ENV-001 — delegate bounded research with disjoint path ownership.
3. ADR-001 — domain model and dependency boundaries once DOC-001 evidence lands.
4. CORE-001 — server-owned PTY + CLI session + terminal I/O + reconnect vertical slice.

---

## Checkpoint 4 — Stopgap mitigation (Option D) activated (2026-10-15)

**Objective:** unblock the three holding Wave 1 Leads without a global config change while the
DEC-009 remediation decision is pending.

**Actions:**

- Started `/tmp/relayable-wake-daemon.py` for a bounded 1800 s (pid in
  `/tmp/relayable-wake-daemon.pid`): every ~0.25 s it sends one empty `Enter` to each agent-less
  pane in workspaces `wX`, `wY`, `wZ`, clearing Herdr's `agent_pane_busy` state inside Herdsman's
  `agent_start` retry window. Stop file `/tmp/relayable-wake-daemon.stop`; log
  `/tmp/relayable-wake-daemon.log`.
- Sent each Lead a bounded retry instruction (`message_staff`): retry delegation now, ≤2 concurrent
  workers, stop after 3 consecutive `agent_start` failures, stay in owned paths, commit, report.

**State:** all three Leads were idle before the messages; daemon verified running; no config or
system change; no leaked panes. This is a time-boxed stopgap only — the permanent fix is still the
pending DEC-009 decision.

**Result so far:** the stopgap worked — after the retry instruction, `research/ui-reference` has 2
active Agents and `research/protocol-compat` has 2 active Agents (its herd run owns the handoff
until they settle); `research/env-audit` is working. Outcomes (files, commits, validation) are
pending the Leads' reports. The daemon window expires ~30 min after activation; if a later wave
(e.g. a writer after workers) still needs it, restart the daemon or apply the permanent fix.

**Next:** collect the Leads' retry results; if agents start, review/integrate Wave 1 (INT-001). If
`agent_start` still fails, stop the daemon (`touch /tmp/relayable-wake-daemon.stop`) and report the
fresh evidence.

---

## Checkpoint 3 — Wave 1 execution blocked by Herdr agent-provisioning fault (2026-10-15)

**Objective:** let the three resumed Wave 1 research projects execute and produce their evidence.

### What happened

- All three projects were resumed (sequential `resume_project`; the Herdr pane-busy state was
  cleared by a bounded background loop that sent empty `Enter` to the target pane during launch).
- All three Leads ran. `research/ui-reference` and `research/protocol-compat` then delegated to
  writer Agents; every `delegate_agent` failed identically at `stage: agent_start` with Herdr
  `agent_pane_busy` / `agent target pane <id> is not an available shell`, and Herdsman rolled each
  attempt back (`rollback: true`, `list_agents` = 0). `research/env-audit` hit the same fault.
- Each Lead reported blocked with **no repo writes**; all three branches remain at `3db73c2`. The
  Leads stopped retrying to avoid leaking panes; they hold for Manager instruction.

### Root cause (evidence in `TESTS.md` rows 15–23)

The `herdr-automatic-rename` plugin's fish hook (sourced from `~/.config/fish/config.fish` lines
40–43) spawns background `bash automatic-rename.sh` on every `fish_preexec`/`fish_postexec`.
Herdr's `agent start` requires the pane to be at its prompt with the shell alone in the foreground;
the hook's live `bash` children cause `agent_pane_busy`. A controlled A/B test in a throwaway tab
(hook active = busy with two bash children; hook functions removed = available with only `fish`)
confirms the causation. `pane run`, which Herdsman uses as its readiness marker, always triggers
the hook immediately before `agent start`.

### Current state

- Three project assignments preserved; three Leads alive but idle/holding (`runtime_state: done`).
- No research files or commits on any research branch.
- Test tabs closed; no leaked panes (`herdr pane list --workspace wZ` shows only `wZ:p1`).
- No Herdr restart, no global config change, and no tool install was performed.

### Options (pending user decision = approval gate)

- **A (recommended, needs approval):** stop sourcing the automatic-rename fish hook in
  `~/.config/fish/config.fish` (lines 40–43). New panes then start agents reliably and no Herdr or
  Lead restart is needed. Cost: live per-command tab auto-renaming stops (cosmetic; event-based
  naming still works). Reversible by restoring the lines.
- **B (no global config change):** add a project-local `managed-lead` overlay granting Leads
  bash/edit/write, then `stop_lead` + `resume_project` each Lead so they execute their research
  directly. Reversible; changes the in-repo delegation model while the environment is faulty.
- **C:** update/reinstall the plugin (unproven) or report the hook behavior upstream.

### Next tasks

1. Obtain the user's decision on A/B/C (approval gate).
2. Apply the chosen fix and message the Leads to retry delegation (Option A) or restart them with
   write tools (Option B).
3. Resume Wave 1 review/integration (INT-001) and then Wave 2 (`TERM-001`/`ADR-003`,
   `ADR-001`/`ADR-002`, `CORE-001`).

---

## Checkpoint 2 — Context-window policy implementation + Wave 1 delegation recovery (2026-10-15)

**Objective:** implement and verify the user's strict context-window policy (no automatic/manual
compaction; repository-authoritative state; fresh bounded agents; checkpoint-and-handoff), then
resume the failed Wave 1 research delegations.

**Files changed:** `.pi/settings.json` (new); `AGENTS.md` (context policy); `docs/agent-loop/`:
`BACKLOG.md` (new), `TESTS.md` (new), `DECISIONS.md` (DEC-008), `STATUS.md`, `NEXT.md`,
`WORKLOG.md` (this entry). Policy commit `ccfdb7a`; evidence commit follows this entry.

### What was inspected

- Pi `1.2.0` (`pi --version`); docs `settings.md` / `compaction.md`: `compaction.enabled` is a
  supported setting (default `true`); project `.pi/settings.json` overrides agent-directory settings;
  changes take effect on `/reload` or a new Pi process.
- Global `~/.pi/agent/settings.json`: `compaction.enabled` already `false` — left unmodified.
- Extensions: global `herdr-agent-state.ts` registers no compaction hooks; pi-herdsman 0.27.0
  registers `session_before_compact` context-retirement handling; `contextRetirement` defaults to
  `true` (`config.json` overrides only `spawnPlacement`).
- Project `.pi/` did not exist before this checkpoint.
- Failed delegation state: Herdr worktrees wX/wY/wZ and branches created at `1c84bca`; panes are
  idle fish shells in the worktree cwds; Herdsman persisted assignment records for the three
  branches (repoKey `/home/sulthonuladib/work/relayable/.git`), so recovery is `resume_project`, not
  a new `delegate_project`.

### Commands and results (full table in `TESTS.md`)

- `pi --version` → `1.2.0` (exit 0).
- `node /tmp/verify-settings.mjs <primary checkout + 3 worktree cwds>` → every path:
  `trusted: true`, `effectiveCompaction.enabled=false`, `projectCompaction.enabled=false`,
  `errors: []` (exit 0).
- `git -C <worktree> merge --ff-only master` ×3 → fast-forward `1c84bca..ccfdb7a` (exit 0); each
  worktree now carries `.pi/settings.json` and the updated loop docs.
- `herdr pane list` / `herdr pane process-info --pane wX:p1` / `herdr pane read wX:p1` → panes alive,
  `agent_status: unknown`, fish at the worktree prompt, readiness marker echoed.

### Interpretation

- Compaction is disabled for every Pi process whose cwd is the primary checkout or any of the three
  research worktrees, which covers Manager, project Leads, and agents launched inside those cwds.
- Settings resolve at process start; a running process keeps its startup configuration. This Manager
  session started 22:30 local and the global settings file mtime is 22:33, so no claim is made about
  this process's in-memory value; no process was restarted to apply the preference (per the user's
  instruction). Newly started processes use the verified effective value.

### Risks / caveats

- The persisted Wave 1 assignments were snapshotted with `full.md` embedded in the task text. New
  briefs will not attach `full.md` (DEC-008); recipients read it from the checkout instead.
- No build/test suite exists yet, so this checkpoint validates configuration and infrastructure only.
- The failed pane launch is not yet root-caused; recovery is sequential `resume_project` with
  verification after each.

### Next tasks

1. Resume the three research projects sequentially; verify each Lead is live before the next.
2. Review Wave 1 diffs, integrate (INT-001), then Wave 2 (`ADR-003`/`TERM-001`, `ADR-001`/`ADR-002`,
   `CORE-001`/`CORE-002`, UI work against the integrated catalogue).
