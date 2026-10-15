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

## Checkpoint 9 — Clean-start handoff: `doc-002` evidence merged; worktrees retired (2026-10-15)

**Objective:** the user asked why `research/doc-002` was not on `master` and directed that it be
merged so the project starts clean and new agents can use fresh worktrees.

**Finding reported to the user:** the only branch not contained in `master` was `research/doc-002`
(the DOC-002 audit evidence + a master merge). `research/adr-001-002` and `research/term-001-adr-003`
were already merged (`31bed30`, `78b83f2`); they merely still had local branches/worktrees because
`remove_worktree` needs interactive confirmation.

**Actions:**

1. Verified with `git branch --merged/--no-merged master`: only `research/doc-002` was unmerged.
2. `git merge --no-ff research/doc-002` → `a1f1719` (INT-003). `master` now carries
   `docs/compatibility/doc-002-audit/**` (README, `citation-audit.md`, `citation-raw.tsv`,
   `fetch-report.md`, scripts, fetch log).
3. All three Wave 2 branches are merged; their worktrees were removed (`remove_worktree`; branches
   and Pi sessions preserved) so Herdsman can hand out fresh worktrees for the next wave.
4. Updated `STATUS.md`, `TASKS.md` (INT-003), `BACKLOG.md`, `TESTS.md`, `NEXT.md`.

**Result on `master`:** `docs/adr/ADR-001..003`, `docs/architecture/interface-contract.md`,
`spikes/terminal-engine/**`, `docs/compatibility/doc-002-audit/**`, plus all Wave 1 research.

**Commands:** `TESTS.md` rows 61–64.

**Next tasks:** see `NEXT.md` — REVIEW-001, DOC-002 writer pass, TERM-002, CORE-001, then CORE-002 →
UI-001/UI-002 → QA-001, on fresh worktrees.

---

## Checkpoint 8 — Wave 2 handoff: INT-002 merged; DOC-002 blocked; wind-down (2026-10-15)

**Objective:** the user directed a wind-down — no new delegation, finish in-flight work, checkpoint,
hand off, commit + push, then clean up agents and merged worktrees so a new provider/agent can
resume from files.

**Actions:**

1. Propagated the wind-down instruction to all three Wave 2 Leads, requiring them to cascade it to
their subagents (finish only the current bounded task, launch nothing new), commit safe work on
their branch, and return a concise handoff.
2. Two Leads committed before settling, one did not:
   - `research/term-001-adr-003`: `085375c` (PTY/SIGWINCH/byte-stream/RSS spike harness) + `2e41da4`
     (ADR-003, `Proposed`). 58 files, +8300. Phase 2 (engine-level adapters) cancelled by wind-down.
   - `research/adr-001-002`: `69e5600` (ADR-001, ADR-002, `docs/architecture/interface-contract.md`,
     3 files, +1244).
   - `research/doc-002`: **not executed.** Its Lead session had agent-management tools only (no
     `bash`/`edit`/`write`), so with new delegation forbidden it could not land the matrix edit and
     stopped honestly. It did complete and verify the evidence: **zero URL drift** (21 URLs re-fetched
     2026-10-15T16:57:25–34Z; every sha256 identical to DOC-001 and `cmp`-identical to the originals)
     and a full citation audit: 433 links = 386 line-number citations + 47 non-line refs; **193 MATCH /
     115 SHIFT / 78 NOT_FOUND** across 7 files (the defect is far broader than `events.md`).
3. Reviewed both committed branches: scope limited to owned paths, secret scan clean, and the
   spike's recorded checks all exit 0 (`cargo build`, `cargo clippy --all-targets`,
   `cargo fmt --check`); the real-PTY probe reports `SIGWINCH_OBSERVED=true`, `VERDICT=PASS`.
4. Integrated both into `master` (INT-002): `78b83f2` (term-001), `31bed30` (adr-001-002).
5. Preserved the volatile `/tmp/doc002/` evidence durably on the `research/doc-002` branch
   (`docs/compatibility/doc-002-audit/`, commit `2ff3447`, 316 KB) and merged `master` into that
   branch (`669cb84`) so a resumed session has current state.
6. Updated `docs/agent-loop/**` (this checkpoint, `STATUS.md`, `TASKS.md`, `BACKLOG.md`,
   `DECISIONS.md`, `TESTS.md`, `NEXT.md`) and committed + pushed `master`.

**Result on `master`:** `docs/adr/ADR-001..003`, `docs/architecture/interface-contract.md`,
`spikes/terminal-engine/**`, plus all Wave 1 research. ADR-001/002/003 are `Proposed`.

**Commands:** see `TESTS.md` rows 48–56 (review, merges, evidence preservation).

**Risks / caveats:**

- ADR-003's engine pin (`alacritty_terminal 0.26.0`, `vte 0.15.0`, `portable-pty 0.9.0`,
  `gpui ="=0.2.2"`) is provisional; TERM-002 must validate the engine-level claim.
- DOC-002's ~142 remaining wrong citations beyond `events.md` stay unfixed until the writer pass.
- `/tmp/doc002/` and `/tmp/term001-evidence/` may be cleared on reboot; the doc-002 audit is now
  committed, and the spike's raw results are committed, so the durable evidence survives.

**Next tasks:** see `NEXT.md` — DOC-002 writer pass; TERM-002; CORE-001 (first engineering slice)
using the ADR-001/002 interface contract; then CORE-002 → UI-001/UI-002 → QA-001.

---

## Checkpoint 7 — Wave 2 kickoff: DEC-009 resolved, DOC-002 + TERM-001/ADR-003 + ADR-001/002 launched (2026-10-15)

**Objective:** resume from `NEXT.md`; clear the DEC-009 agent-provisioning gate; launch the Wave 2
work in the user's requested order (DOC-002 → TERM-001+ADR-003 → ADR-001/002 → CORE-001/002 →
UI-001/002) with one writer per branch/path (DEC-008).

**Recovered context:** read `full.md` completely, `AGENTS.md`, `.pi/settings.json`, and
`docs/agent-loop/{NEXT,STATUS,BACKLOG,TASKS,DECISIONS,TESTS,WORKLOG}.md`. Git: `master` at `b96da27`,
clean, up to date with `origin/master`; only the primary worktree `w11` existed; the Wave 1 research
worktrees had been removed; `/tmp/relayable-wake-daemon.stop` present and no daemon process running.

**DEC-009 gate resolved (option A, user-authorized):** the automatic-rename fish hook block in
`~/.config/fish/config.fish` (original lines 40–43) is now commented out with an in-file restore
pointer. Backup `/tmp/relayable-config.fish.bak` (md5 `60a6536db6ac375f7c0e4d4f2872d609`). Verified
`fish -n` exit 0 and a fresh `fish -c 'functions | grep …'` shows no hook functions loaded. This is a
host config change made under the user's explicit instruction; no repo file depends on it.

**Wave 2 projects launched (all base `master` @ `b96da27`):**

| Workspace | Branch | Session | Tasks | Owned paths |
|---|---|---|---|---|
| `w12` | `research/doc-002` | `3534d553-bec1-455c-83df-807ee3ca724a` | DOC-002 | `docs/compatibility/reference-behavior-matrix.md` |
| `w13` | `research/term-001-adr-003` | `8f442f6e-6f8c-4a03-9016-dcf3f5827aa1` | TERM-001, ADR-003 | `docs/adr/ADR-003-*.md`, `spikes/terminal-engine/**` |
| `w14` | `research/adr-001-002` | `6a4c1607-f4d3-42fb-b1fc-d6e2dce6e7ba` | ADR-001, ADR-002 | `docs/adr/ADR-001-*.md`, `docs/adr/ADR-002-*.md`, `docs/architecture/**` |

`delegate_project` returned `ok` for all three; a `list_staff` immediately after showed
`research/doc-002` `runtime_state: working` with 1 active agent — the option A fix works. Paths are
disjoint (one file / docs-adr files / spike dir), so no overlapping writers. The three Leads were
told to read `full.md` and `AGENTS.md` from their own checkouts rather than have them injected
(DEC-008).

**Commands run:**

| Command | Result | Exit |
|---|---|---|
| `cp -a ~/.config/fish/config.fish /tmp/relayable-config.fish.bak` | backup created | 0 |
| comment out the hook block in `~/.config/fish/config.fish` | 3 code lines commented | — |
| `fish -n ~/.config/fish/config.fish` | no syntax errors | 0 |
| `fish -c 'functions \| grep -i -E "har_\|automatic"'` | `no automatic-rename hook functions loaded` | 0 |
| `delegate_project` ×3 (doc-002, term-001-adr-003, adr-001-002) | `ok:true`, sessions/workspaces as above | 0 |
| `list_staff` | `w12` Lead `working`, 1 active agent; `w11` + 3 linked worktrees | 0 |

**Risks / caveats:**

- Option A is a host-level change outside the repo; it is reversible (uncomment the block) and the
  backup hash is recorded. The wake-daemon stopgap stays as fallback documentation.
- The Wave 2 Leads are fresh sessions; their briefs point at repo evidence, not prior chat.
- `docs/agent-loop/**` is Manager-owned; the three branches must not touch it.
- The `~/.herdr/worktrees/relayable/research-reference-ui` directory from the pre-move repoKey is a
  stale, non-registered worktree stub (broken `.git` pointer); left untouched and noted, not removed.

**Next tasks:**

1. Monitor the three Leads; review their diffs and claimed validation independently (INT-002).
2. Integrate DOC-002, then ADR-001/002 and TERM-001/ADR-003 into `master` as they complete.
3. Launch CORE-001 (server-owned PTY + CLI session + terminal I/O) once ADR-001/002 are integrated;
   then CORE-002, UI-001/UI-002, QA-001.
4. Keep `docs/agent-loop/**` updated after each integration; ask the user only for genuine blockers.

---

## Checkpoint 6 — Wave 1 complete, all research integrated; handoff (2026-10-15)

**Objective:** finalize the research wave at a state acceptable for the next phase, per the user's
wrap-up instruction.

**Actions:**

- Messaged all three Leads a wrap-up instruction (commit cleanly, leave the worktree clean, no new
  waves, immediate concise handoff); `env-audit` was confirmed complete.
- Reviewed branch scopes (`git log`/`git diff --name-only`/worktree status/secret scan): only the
  owned paths changed and all worktrees are clean.
- Integrated into `master` with `--no-ff` merges:
  - `research/env-audit` → `b24133c` (2 docs, +1591; tip `a31a839`) — already recorded in Checkpoint 5
  - `research/ui-reference` → `c0578ca` (5 docs, +1902; tip `269018c`)
  - `research/protocol-compat` → `49795a0` (1 doc, +648; tip `9b0fe37`)
- INT-001 is complete. `docs/operations/`, `docs/research/`, and `docs/compatibility/` now exist on
  `master`.
- Stopped the Option D stopgap daemon (`touch /tmp/relayable-wake-daemon.stop`) and verified it is
  gone. The underlying Herdr fault remains unresolved (DEC-009 A/B/C pending).

**State:** all Leads idle with open assignments; worktrees clean; nothing pushed; no tracked media;
no leaked panes. Handoff written to `NEXT.md`.

**DOC-001 wrap-up (Lead, tip `9b0fe37`):** 648 lines, clean tree, one file; labels 342
`verified-official-docs` / 50 `unknown` / 2 `inferred` / 1 `intentionally-different`; explicit
no-wire-compatibility statement; 22-row retrieval log (URL/HTTP/exit/timestamp/sha256). Known gaps
recorded as **DOC-002**: (a) `events.md` line-number citations do not match the canonical `.md`
line numbers (quoted text is verbatim); (b) the live drift re-check of the 7 doc URLs was cancelled
by the wrap-up — all facts are as-of 2026-10-15 16:03–16:08Z; (c) the independent reviewer's
live-drift pass and consolidated review were stopped (its non-live cache check matched every
sampled high-risk fact).

**ui-reference final tip (`4102de1`):** provenance pins and catalogue attributions fixed (3 files,
+15/−11; commit only `docs/research/**`), merged in `55d8147`. It is self-validated by its author
(the stop instruction forbade another verification wave); the last independent audit covers
`269018c`. Known follow-up tracked as **REF-004**: `design-tokens.md:210` mixes a 29 px palette-row
value (a 480p video estimate) into clone-derived numbers; calibration items remain in
`open-questions.md`. The fidelity gate stays partially open for un-footaged states (settings,
inspector, in-app disconnected/loading/error, stacked splits, hover/drag, notifications).

**Next:** ADR-003/TERM-001 and ADR-001/002 from the integrated evidence, then CORE-001/002 and UI
work; Android install and Herdr remediation are approval gates.

---

## Checkpoint 5 — ENV-001 complete and integrated (INT-001, 1/3) (2026-10-15)

**Objective:** review and integrate the completed `research/env-audit` branch.

**Review (Manager):**

- `git log --oneline 3db73c2..research/env-audit` → 6 commits; `git diff --name-only` → only
  `docs/operations/{toolchain-audit,android-toolchain}.md` (+1591 lines); worktree clean; a
  secret-pattern scan found nothing.
- The Lead's workflow included an independent verification pass (3 defects), remediation, focused
  re-verification (2 residual inaccuracies), and a final accuracy fix. Verification/remediation
  artifacts are in the pi-herdsman results store.

**Integration:** `git merge --no-ff research/env-audit` → merge commit `b24133c` on `master`;
INT-001 is 1/3 complete (ENV done; REF and DOC in flight).

**Key findings to feed ADRs (full detail in the merged docs):**

- Rust 1.98.1 stable (rustfmt 1.9.0, clippy 0.1.98, single GNU target); mise 2026.10.5 manages
  java/node but not Rust; crates.io requires a descriptive User-Agent; registry cache 914 MB.
- crates.io `gpui` 0.2.2, Apache-2.0, repo `zed-industries/zed`; not cached → first build needs
  network. `Glass-HQ/gpui` is reachable but has no root LICENSE (404; manifests declare
  Apache-2.0) → artifact/license decision required in ADR-003 before pinning (gap G3).
- systemd 261 user manager running; `Linger=yes`; `/run/user/1000` mode 700; `journalctl --user`
  usable; no relayable units.
- Wayland + XWayland, 798 fonts (JetBrainsMono Nerd Font present), 20 CPUs, 31 GiB RAM, 51 GB free,
  `/dev/dri` present; wayland/xkbcommon/fontconfig/Vulkan/EGL libs present.
- Android: CLI 1.0.16500706 + adb 1.0.41 + KVM + Temurin 25.0.4.1 ready; **no SDK, sdkmanager,
  avdmanager, emulator, AVDs, images, licenses, or Gradle wrapper**; AGP 9.4 needs Gradle ≥ 9.6 and
  JDK ≥ 17 (Java 25 needs Gradle ≥ 9.1); AGP-on-JDK-25 unverified. Android Studio absent and not a
  dependency.

**Approval gate surfaced to the user:** Android SDK/emulator installation, license acceptance, and
project JDK pin (JDK-25-only risk) for AND-001; disk is 68% used.

**Next:** continue REF/DOC; integrate as they complete; then ADR-003 from the ENV evidence and
ADR-001/002 from DOC-001; then CORE-001.

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

**Result so far:** the stopgap worked — after the retry instruction, all three Leads have 2 active
Agents each (`ui-reference`, `protocol-compat`, `env-audit`); outcomes (files, commits, validation)
are pending the Leads' reports. The Leads' earlier "fleet-wide blocked" escalations describe the
pre-retry state and are superseded by this result; their "pane leak" caution is not supported by
`herdr pane list` observations (rollback cleans up failed attempts). The daemon window now expires
~00:05; if a later wave still needs it, restart the daemon or apply the permanent fix.

**Window extended:** the 30-minute instance was replaced with a fresh 3600 s instance (pid 2781970,
stop file unchanged, expires ~00:05) so the planned workers → writer → reviewer waves are covered.
All three Leads were notified of the extension.

**Wave 1 delivered (`research/ui-reference`):** `docs/research/video-review.md` (`f067f92`,
+562 lines) and `docs/research/gorex-review.md` (`a7e14a3`, +420 lines). All five videos were actually
inspected (metadata + auto-captions + downloaded frames; 332 JPEGs in the gitignored
`docs/research/evidence/`); two transient capture errors were retried and recorded, not hidden;
video 3's figures are quoted verbatim with the stated environment and labelled non-reproducible;
video 5 yielded verbatim `rex` CLI commands. Wave 2 (`ref-catalog`: `ui-catalog.md` +
`open-questions.md`) is in flight under the same stopgap, and an independent read-only audit of the
GoRex artifact is running. 0/3 `agent_start` failures. Stop the daemon with
`touch /tmp/relayable-wake-daemon.stop` once the permanent fix is applied or the wave settles.

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
