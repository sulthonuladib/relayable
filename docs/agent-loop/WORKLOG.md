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

## Checkpoint 22 — UI-002 in flight: reference-still durability + client API gaps recorded (2026-10-15)

**Objective:** launch UI-002 and settle its evidence-policy question.

**UI-002 finding:** the 13 reference stills cited by `docs/research/design-tokens.md` §2.4 do not exist
in-repo — `docs/research/evidence/**` was gitignored and is lost (the Wave-1 worktree was removed and
gitignored files are not in Git). UI-002 regenerated them locally from the surviving 1080p sources in
`~/.cache/relayable-research/hires/` (V1 1170×1080, V4 1624×1080) into the gitignored
`apps/linux/evidence-local/ref/`. **Manager decision:** commit only Relayable's own screenshots + a
written discrepancy table; keep composites/reference frames local-only (`full.md` §0.2/§2.6). The
durable evidence is `docs/research/ui-catalog.md` + `design-tokens.md`; stills must be regenerated for
future comparisons.

**UI-002 token re-verification:** against the regenerated stills (1:1, 1920×1080): top bar 40 px +
1 px sep rgb(29,50,66); top-bar bg mean rgb(5,27,47) (claimed 4,27,46); active pill exactly 205×30
fill rgb(19,41,62); terminal row pitch ≈17.1 px; window crop 1099×706. Only the known REF-004 palette
attribution conflict surfaced. `v1-t0222` shows the command palette, not vertical tabs → vertical-tab
geometry is unmeasurable and will not be implemented (honest gap).

**Client API gap recorded (`CLIENT-API-001`):** `relayable-client`'s only raw-byte API
(`wait_for_output`) returns `Err(Timeout)` and drops already-acked bytes, so it cannot drive a live
terminal; UI-002 uses a wire client on `relayable-protocol` as a workaround.

**Active:** UI-002 (`w1D`). **Next:** integrate UI-002 → QA-001 → CTX-001 → ADR-004.

---

## Checkpoint 21 — CORE-002 integrated: events, disconnect detection, resume cursor, reattach (2026-10-15)

**Objective:** add the reconnection semantics CORE-001 deferred.

**Deliverable:** CORE-002 (`e8f206e`, 21 files, +4385/−152) merged as INT-011 `91662af`. Event bus +
`client_connection_state_changed`/`client_disconnected`/`session_state_changed`/`writer_lease_changed`/
`terminal_child_exited`; idle/transport-drop disconnect detection (session survives); `session.resume`
with `snapshot_required` fallbacks (`no_cursor`/`cursor_below_floor`/`cursor_ahead`); snapshot-at-`S` +
events no-gap/no-duplicate; writer-lease takeover. No `relayable-model`/contract change; extensions in
`protocol/compatibility/core-002-method-set.md`.

**Independent validation:** `bash scripts/validate.sh` on `master` → `All CORE-001 validation checks
passed.` (fmt/clippy/build 0; **148 tests**, CORE-001's 104 unchanged). Property evidence in `TESTS.md`.
Note discovered: the `lifecycle` tests abort if `target/debug/relayable` is older than source; the
harness requires a prior `cargo build --workspace` (which `validate.sh` does) — not a regression.

**Non-claims carried:** no crash/host-reboot survival; no automatic client reconnect/backoff; no CLI
`events`/`wait`/`attach` surface; `flow_control_overflow` still untested (`QA-FLOW-001`).

**Next:** UI-002 (reference workspace) → QA-001 → CTX-001 → ADR-004.

---

## Checkpoint 20 — UI-001 integrated; ADR-003 Accepted (2026-10-15)

**Objective:** resolve the `gpui` pin blocker and prove a Linux window.

**Deliverable:** UI-001 (`37272db`, 4 commits, 27 files, +10341) merged as INT-010 `09928db` — all
inside `apps/linux/**` (its own workspace; root `Cargo.toml`/`Cargo.lock` and `crates/**` untouched).

**Blocker resolved:** `cargo update -p libc --precise 0.2.189` in `apps/linux` downgrades `libc
0.2.190 → 0.2.189` (lockfile hold; no `[patch.crates-io]` needed). Independently reproduced:
`cargo check --locked` exit 0, `cargo test --locked` 9 passed, lockfile has exactly `libc 0.2.189` +
`xattr 0.2.3`. A real Wayland window was launched (`hyprctl`: `class=relayable`, `title=Relayable`,
`950x1030`) and screenshotted (`apps/linux/evidence/relayable-window.png`, a real PNG); Manager
visually confirmed the render (dark `rgb(9,30,46)` surface + "Relayable" label).

**Decisions:** **ADR-003 → Accepted** (DEC-004): engine pins validated by TERM-002, `gpui` pin proven
by UI-001; reconsideration trigger 4 discharged. Recorded the forward `libc` lockfile-hold constraint
(a future `apps/linux` dependency on a `libc = "=0.2.190"` crate would re-break `gpui`).

**Also noted:** REF-004's palette conflict is confirmed real (`design-tokens.md` §204/§210/§212);
`apps/linux/src/tokens.rs` encodes the measured range and documents the conflict without resolving it.

**Next:** CORE-002 (`w1B`, in flight) → UI-002 → QA-001 → CTX-001 → ADR-004.

---

## Checkpoint 19 — TERM-002 integrated; ADR-003 engine pins validated, GPUI pin blocked (2026-10-15)

**Objective:** complete the engine-level spike and resolve ADR-003's engine gate.

**Deliverable:** TERM-002 (`af37bc5`+`7ff32cf3`, 327 files, +77884) merged as INT-009 `60d4395`
(`spikes/terminal-engine-phase2/**` only; root workspace still `exclude = ["spikes"]`). Independently
re-validated: build/clippy/fmt exit 0, `cargo test` 42 passed.

**Verdict:** `alacritty_terminal 0.26.0` + `vte 0.15.0` + `portable-pty 0.9.0` = 32 PASS / 1 documented
UNSUPPORTED / 0 FAIL on real PTYs; **engine pins accepted, no engine change**. `gpui = "=0.2.2"`
resolves but `cargo check` exits 101 (`xattr 0.2.3` → `libc 0.2.190` removed `ENOATTR`); remedies are
hold `libc` ≤0.2.189 or patch `xattr`, **not applied**.

**Actions:** ADR-003 updated with the phase-2 result, the `gpui` blocker, and the required UI-001
remedy — **kept `Proposed`**. DEC-004 updated.

**Next:** UI-001 must (a) apply and prove a contained `gpui` remedy and (b) prove a minimal window
builds/runs, then ADR-003 can move to Accepted. CORE-002 (`w1B`) is still in flight.

---

## Checkpoint 18 — CORE-001 integrated: first engineering vertical slice (2026-10-15)

**Objective:** land the first real Linux vertical slice and verify it independently.

**Deliverable:** CORE-001 (`653326e`, 53 files, +12010) merged as INT-008 `d3e0286`. The repo now has
the Cargo workspace root + `crates/relayable-{model,protocol,pty,server,client,cli}` + `protocol/schema/*.proto`
+ `tests/{lifecycle,protocol,reconnect,multi-client,fixtures}` + `scripts/validate.sh`.

**Independent validation on `master` (full detail in `TESTS.md` 76–78):** `scripts/validate.sh` → all
exit 0; `cargo test --workspace` green (Σ ≈ 104); acceptance test proves the five required properties
(server-owned PTY with `ppid == server pid`; CLI session `session:<uuidv7>`; I/O round-trip; disconnect
survival with the same pid; reconnect restoring ids/dims/writer-lease/survival-nonce) plus resize,
close-reap, SIGTERM shutdown, and bounded input; ADR-001 dependency-direction check passed; the
phase-1 spike still builds (`crates`/`spikes` isolation holds); DEC-010 Unix-symbol guard passed.

**Accepted decisions:** the first-release method set and CLI/JSON surface are recorded in
`protocol/compatibility/core-001-{method-set,cli-json}.md` (the parent contract defers the method list
to the server-API task); `RELAYABLE_*`-only, no `REX_*`; no `rust-toolchain.toml` (1.98.1 is not an
installed rustup toolchain name).

**Residual gap:** ADR-005 Decision 2 (set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER`
in the managed terminal) and its §8.1 test are not implemented — tracked as `CTX-001`.

**Still in flight:** TERM-002 (`w17`). **Next:** CORE-002 (disconnect survival + reattach snapshot;
now largely demonstrated at the protocol level) → UI-001/UI-002 → QA-001 → ADR-004.

---

## Checkpoint 17 — ADR-FIX-001 integrated; DEC-002/DEC-003 Accepted (2026-10-15)

**Objective:** clear the REVIEW-001 blocking documentation items before CORE-001 builds on the ADRs.

**Result:** ADR-FIX-001 (`a58cfc6`, 3 files +135/−86) reviewed and integrated (INT-007 `f474c83`):
- B-1 Unverified table refreshed — items #1–#9 SETTLED with repo-tracked evidence, #10 Android QUIC still open;
- B-2 overstated authn/authz coverage claim removed;
- B-3 D11 row 17 added (no client-side decode→re-encode path; `prost` drops unknown fields, 24→9 bytes);
- B-4 TLS 1.3 minimum-version disclosure (`rustls` default `tls12` feature disabled);
- B-5 `aws-lc-rs` default provider + Android cross-compile implication disclosed;
- ADR-001 A-2/A-6 and ADR-003 C-2 (SGR 337/2/335) corrected.
No decision changed; ADR-003 remains `Proposed` with unchanged pins; `interface-contract.md` still `PtyProcessId`.

**Status flips:** ADR-001 and ADR-002 → **Accepted** (DEC-002/DEC-003); ADR-003 stays `Proposed`
(DEC-004) pending TERM-002/UI-001.

**Commands:** `git merge --no-ff adr/review-001-fixes` → `f474c83`; ADR statuses + `DECISIONS.md` updated; pushed.

**Still in flight:** CORE-001 (`w18`, scaffolding untracked), TERM-002 (`w17`, harness).

---

## Checkpoint 16 — ADR-005 integrated; DEC-006 Accepted (2026-10-15)

**Objective:** resolve the environment-variable namespace.

**Result:** ADR-005 (`8585d08`, 365 lines) reviewed and accepted. Decision: `RELAYABLE_*`-only — exact
names `RELAYABLE_SESSION`, `RELAYABLE_BLOCK`, `RELAYABLE_SERVER`, `RELAYABLE_CLIENT`; no `REX_*` alias
is read or written; explicit flags beat env context; missing context fails with a documented non-zero
exit code (no guessing). Grounded in `full.md` §0.1/§3/§8.1, ADR-002 D14, matrix §7.3/§8.3. Test plan
proposed (not created); `tests/**` is CORE-001-owned.

**Commands:** `git merge --no-ff adr/adr-005-env-namespace` → INT-006 `74e7e20`; ADR status set to
Accepted; DECISIONS DEC-006 → Accepted; TASKS/STATUS updated; pushed.

**Still in flight:** ADR-FIX-001 (`w19`), CORE-001 (`w18`), TERM-002 (`w17`).

---

## Checkpoint 15 — DOC-002 integrated; recheck false positive corrected (2026-10-15)

**Objective:** settle the DOC-002 citation repair.

**What happened:** my automatic `audit.py` recheck appeared to show 10 defective numeric citations, and
I sent the Lead a revision request. Re-adjudicating each row against the raw sources and the
*adjudicated* `citation-audit.md` proved the recheck was a false positive (`audit.py` mis-handles
paraphrased evidence and repeated quotes). The request was withdrawn, the Lead confirmed the branch was
unchanged, and the correction was recorded in `TESTS.md`. DOC-002 verified against the adjudicated
audit: 0 invented refs, all 78 NOT_FOUND converted to heading anchors, URL set unchanged, evidence-label
counts byte-identical pre/post (verified independently).

**Commands:** `git merge --no-ff docs/doc-002-matrix-repair` → INT-005 `16c8bc9` (only
`docs/compatibility/reference-behavior-matrix.md`, +197/−193); pushed.

**In flight:** TERM-002 (`w17`), CORE-001 (`w18`), ADR-FIX-001 (`w19`).

---

## Checkpoint 14 — REVIEW-001 integrated; DOC-002 sent back; ADR-FIX-001 + CORE-001 launched (2026-10-15)

**Objective:** integrate the review, correct the DOC-002 repair, and start the fix pass.

**Commands/actions and results**

| # | Action | Result |
|---|---|---|
| 1 | reviewed `115a5bb` scope + secret scan; `git merge --no-ff review/review-001-adr-pass` | INT-004 `774d394` — only `docs/reviews/**` (5 files, +2348); pushed |
| 2 | re-ran `audit.py` against the DOC-002 branch matrix (`1634f42`) | 433 links: MATCH 298 / NO_LINE_REF 125 / **SHIFT 8 / AMBIGUOUS 1 / NOT_FOUND 1** → repair incomplete; 10 defective rows returned to the DOC-002 Lead with a re-run recipe (see `TESTS.md` 71–72) |
| 3 | `delegate_project` ADR-FIX-001 | session `e605e0cc-e299-46c8-acfa-9164955d7aed`, workspace `w19`, branch `adr/review-001-fixes` (documentation correctness only; no decision change) |
| 4 | `delegate_project` CORE-001 | session `3096aa58-8c0c-4198-b636-2e4197d00ecc`, workspace `w18`, branch `core/core-001-vertical-slice` |
| 5 | `AGENTS.md` protoc row + delegation note; `TASKS.md` statuses | Manager-owned edits (this commit) |

**Live projects:** TERM-002 (`w17`, 2 agents), CORE-001 (`w18`, 2 agents), ADR-FIX-001 (`w19`),
DOC-002 revision (`w16`). REVIEW-001 integrated and its Lead idle.

**Next tasks:** await TERM-002 + CORE-001 reports; review/merge ADR-FIX-001 then flip DEC-002/DEC-003 to
Accepted if the blocking items are cleared; re-check DOC-002 and merge; then CORE-002/UI-001.

---

## Checkpoint 13 — REVIEW-001 findings recorded; CORE-001 blocker cleared (2026-10-15)

**Objective:** satisfy the "independent review before CORE-001" gate. REVIEW-001 (branch
`review/review-001-adr-pass`) produced `docs/reviews/REVIEW-001-adr-review.md` (327 lines) plus four
evidence files in `docs/reviews/evidence/`. The Lead is applying final internal-consistency fixes; the
branch is not yet committed/merged, so this checkpoint records the findings from the review worktree.

**Verdicts (from the review report):**

| Artifact | Verdict | Blocking items |
|---|---|---|
| ADR-001 | accept-with-nits | A-1: same entity had two Rust newtype names (`PtyProcessId` vs `PtyId`) |
| ADR-002 | accept-with-nits | B-1 stale Unverified table (9/10 settled, only #10 Android QUIC open); B-2 overstated authn/authz coverage row; B-3 missing D11 no-decode→re-encode test |
| ADR-003 + spike | keep `Proposed` | C-1: no committed evidence exercises any engine → TERM-002 is a hard gate for engine adoption; UI-001 gates the `gpui = "=0.2.2"` pin |

**Manager action taken now:** A-1 reconciled — `docs/architecture/interface-contract.md` now names the
type `PtyProcessId` (was `PtyId`), matching ADR-001 and the `PtyProcess` entity. This was the only item
the review marked as blocking for CORE-001 model code. ADR-002's B-1/B-2/B-3 and the ADR-001 nits are
tracked as `ADR-FIX-001`; they gate marking DEC-003 Accepted, not CORE-001.

**Interpretation:** the review confirms `TERM-002` is **not** a precondition for CORE-001 provided
CORE-001 does not adopt `alacritty_terminal`/`vte` (PTY-level `portable-pty` evidence already exists).

**Not yet done:** review branch commit/merge; DOC-002 branch merge; TERM-002 completion; ADR-FIX-001;
DEC-002/003 status change to Accepted.

---

## Checkpoint 12 — Post-Android-deferral: REVIEW-001 + DOC-002 repair + TERM-002 launched (2026-10-15)

**Objective:** resume the ordered Linux work after the DEC-010 Android deferral, with REVIEW-001
before CORE-001 per `NEXT.md`. Three independent, disjoint-path projects launched from `master`
`a1091e5` (verified clean and level with `origin/master`).

**Commands run and observed results**

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 71 | `git status --short --branch`; `git rev-parse HEAD` | `## master...origin/master` clean; `a1091e58f32d363ceb2614ab4915329b5de6a869` | 0 |
| 72 | `git worktree list`; `git branch -a` | only the primary checkout; 5 preserved research branches + `origin/master` | 0 |
| 73 | `command -v protoc`; `protoc --version` | `/usr/bin/protoc` → `libprotoc 36.1` (symlink to `protoc-36.1.0`) | 0 |
| 74 | `cargo --version`; `rustc --version` | cargo `1.98.1`; rustc `1.98.1` | 0 |
| 75 | `delegate_project` REVIEW-001 (base `master`) | `ok:true`; session `34ddd30b-ba3b-4c11-94aa-2c0b1f1ceeb3`; workspace `w15`; branch `review/review-001-adr-pass` | 0 |
| 76 | `delegate_project` DOC-002 writer pass | `ok:true`; session `6d07f391-a261-4d5e-97d6-efefbc0c2330`; workspace `w16`; branch `docs/doc-002-matrix-repair` | 0 |
| 77 | `delegate_project` TERM-002 | `ok:true`; session `549ef32a-fef9-4919-95e8-352a3ff02975`; workspace `w17`; branch `spike/term-002-engine-validation` | 0 |
| 78 | `list_staff` | all three Leads active (`runtime_state` done/working); w15 2 Agents, w16 1 Agent, w17 working directly; no `agent_start` failures | observation |

**Note on `protoc`:** DEC-009's remediation holds — all three `delegate_project` calls succeeded on the
first attempt, no `agent_pane_busy`. Also, `/usr/bin/protoc` = `libprotoc 36.1` is present, which
addresses ADR-002's "`protoc` presence on this host is unverified" build-time prerequisite (the
reviewer is asked to confirm it independently).

**Owned paths (one writer each, disjoint):** REVIEW-001 → `docs/reviews/**` (read-only everywhere
else); DOC-002 → `docs/compatibility/reference-behavior-matrix.md` only; TERM-002 →
`spikes/terminal-engine-phase2/**` only (standalone, not a workspace, phase-1 spike untouched).

**CORE-001 is held** until REVIEW-001 findings are recorded and any blocking issues are resolved, per
`NEXT.md`/`TASKS.md`.

**Next tasks:** await the three Lead reports; review diffs; integrate DOC-002 and TERM-002; decide
ADR-001/002/003 status (Proposed → Accepted) using REVIEW-001; then launch CORE-001.

---

## Checkpoint 11 — Scope: Android client deferred (DEC-010, user direction) (2026-10-15)

**Objective:** the user directed that Android apps are not needed for now and that the remaining
Linux platform work is the main goal.

**Decision:** DEC-010 recorded — no Android app/scaffold/SDK/emulator/Gradle work in the current
phase; AND-001 deferred; the Android SDK/JDK-pin approval gate is removed. The model/protocol stay
Android-ready (`full.md` §1.3.6, §5.2). Consistent with `full.md` §1.4 (Android is a future client)
and §1.5 (defer full Android implementation until foundations stabilize).

**Files changed:** `docs/agent-loop/{DECISIONS,TASKS,BACKLOG,STATUS,NEXT,WORKLOG}.md`.

**Next tasks:** the Linux slice is unchanged — REVIEW-001 → DOC-002 writer pass → TERM-002 →
CORE-001 → CORE-002 → UI-001/UI-002 → QA-001; plus ADR-004 (DEC-005) and ADR-005 (DEC-006).

---

## Checkpoint 10 — Herdr auto-rename plugin fully disabled (2026-10-15)

**Objective:** the user asked where the `herdr-automatic-rename` plugin is documented and directed
that it be fully disabled.

**Repo references found:** `DECISIONS.md` DEC-009 (row + section), `STATUS.md`, `NEXT.md`,
`WORKLOG.md` Checkpoints 3 and 7, `TESTS.md` rows 15–23 and 40–43. No other repo file mentions it.

**Mechanism distinction (important):** the plugin's `[[startup]]`/`[[events]]` handlers are separate
from the fish **shell hook**. The hook (`~/.config/fish/config.fish` lines 40–43) is what spawned a
background `bash` on every prompt and caused `agent_pane_busy`; disabling the plugin alone would not
stop it. The hook stays commented (DEC-009 option A).

**Action:** `herdr plugin disable herdr-automatic-rename` → result `plugin_disabled`, `enabled:false`;
`herdr plugin list` showed `disabled`. This was then upgraded to a full **uninstall** on the user's
instruction: `herdr plugin uninstall herdr-automatic-rename` → `No plugins installed.` Herdr left the
checkout (~1.4 MB, git `b90c4086…`) and an empty config dir behind, so both were moved to
`/tmp/herdr-automatic-rename-removed/` and the stale lock removed; `ls ~/.config/herdr/plugins/github`
is now empty and the `config.fish` glob matches 0 files. Finally, at the user's request the now-dead
commented hook block was **deleted from `config.fish`** (`fish -n` exit 0; `grep` shows no
`automatic-rename` reference). Reinstall from
`github:qu8n/herdr-automatic-rename@b90c4086…` if ever wanted.

**Files changed:** `docs/agent-loop/{DECISIONS,STATUS,NEXT,TESTS,WORKLOG}.md`.

**Next tasks:** unchanged — see `NEXT.md` (REVIEW-001 → DOC-002 writer pass → TERM-002 → CORE-001 →
CORE-002 → UI-001/UI-002 → QA-001).

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
