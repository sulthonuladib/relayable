# TESTS — commands actually executed and results

Append-only evidence log. Each entry records the date, purpose, exact command, observed result, and
exit status. Never write a result here that was not actually produced by a command that ran.

## 2026-10-15 — Wave 1 kickoff, context-policy verification, delegation recovery

Environment: Linux `falcon` x86_64 (omarchy), `pi 1.2.0`, pi-herdsman 0.27.0, mise-managed
Node 26.8.2, repo `/home/sulthonuladib/work/relayable`.

| # | Command | Observed result | Exit |
|---|---|---|---|
| 1 | `pi --version` | `1.2.0` | 0 |
| 2 | `cat ~/.pi/agent/settings.json` | global settings include `"compaction": { "enabled": false }` (plus theme/model/package keys) | 0 |
| 3 | `ls -la .pi` (before policy) | project `.pi/` did not exist | 2 |
| 4 | `git status --short --branch` | `## master...origin/master`, clean | 0 |
| 5 | `git worktree list` | `master` + 3 linked research worktrees, all at `1c84bca` | 0 |
| 6 | `git branch -a` | `master`, `research/{ui-reference,protocol-compat,env-audit}`, `origin/master` | 0 |
| 7 | `herdr pane list` | `wW:p1` manager `working`; `wX:p1`, `wY:p1`, `wZ:p1` `agent_status: unknown` with shells in worktree cwds | 0 |
| 8 | `herdr pane process-info --pane wX:p1` | `shell_pid` = foreground `fish` process in the worktree cwd | 0 |
| 9 | `herdr pane read wX:p1 --lines 20 --source recent` | fish prompt at worktree cwd; Herdsman readiness marker echoed | 0 |
| 10 | `cat ~/.pi/agent/pi-herdsman/config.json` | `{"spawnPlacement":"subtree"}` — no `contextRetirement` override | 0 |
| 11 | inspect pi-herdsman 0.27.0 `dist/index.js` | `contextRetirement` default is `true`; `session_before_compact` handler cancels threshold compaction for assigned managed agents | 0 |
| 12 | inspect pi extensions | only `herdr-agent-state.ts` globally; no compaction hooks. `pi-herdsman` registers the context-retirement `session_before_compact` hook | 0 |
| 13 | `git -C <worktree> merge --ff-only master` ×3 (ui-reference, protocol-compat, env-audit) | `Updating 1c84bca..ccfdb7a; Fast-forward` for each; each worktree now contains `.pi/settings.json` | 0 |
| 14 | `node /tmp/verify-settings.mjs <primary + 3 worktree cwds>` (`SettingsManager.create(cwd).getSettings()`) | every path: `trusted: true`, `effectiveCompaction.enabled=false`, `projectCompaction.enabled=false`, `errors: []` | 0 |

### Notes

- The failed `delegate_project` calls persisted assignment records under
  `~/.pi/agent/pi-herdsman/runtime/supervision-v2/...` (`research/ui-reference`,
  `research/protocol-compat`, `research/env-audit`, repoKey `/home/sulthonuladib/work/relayable/.git`).
  This is why recovery must use `resume_project`.
- Stale assignment records also exist for the pre-move repoKey `/home/sulthonuladib/personal/relayable`
  (branches `research/reference-ui`, `research/protocol-matrix`, `audit/build-env`). That checkout no
  longer exists; no live Lead owns them.
- No build or test suite exists yet, so no pass/fail suite results are claimed.

## 2026-10-15 — Herdr agent-provisioning blocker diagnosis (pane busy)

Symptom: every programmatic agent launch fails at `stage: agent_start` with Herdr
`agent_pane_busy` / `agent target pane <id> is not an available shell`. This affects Manager→Lead
resume, Lead→Agent `delegate_agent`, and any Herdsman-spawned writer agent.

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 15 | `herdr pane process-info --pane wX:p1` | foreground processes `[fish, bash(defunct)]` while Herdr reports busy | 0 |
| 16 | `herdr agent start probe --kind pi --pane wX:p1 --timeout 4000 -- --version` | `agent_pane_busy` | 1 |
| 17 | `herdr pane send-keys wX:p1 Enter`, then the same probe | `timeout waiting for agent startup` (= pane accepted; `pi --version` launched) | 1 |
| 18 | `herdr pane run wY:p1 "echo __TIMING__"`, probes every 5 s for 50 s | `agent_pane_busy` for ≥50 s | 1 |
| 19 | A/B test in throwaway tab `wZ:tD`: `pane run` marker with plugin hook active | `agent_pane_busy`; process-info `[fish, bash, bash]` | 1 |
| 20 | A/B test: `functions -e _har_preexec _har_precmd`, then `pane run` marker, then `agent start --timeout 4000` | `timeout waiting for agent startup` (= accepted); process-info `[fish]` only | 1 |
| 21 | `herdr tab create ... --env HERDR_PANE_ID=`, marker + probe | Hooks still ran (`[fish, bash, bash]`), busy — Herdr overrides the env var; env simulation inconclusive | 1 |
| 22 | `herdr tab close wZ:tD` / `wZ:tE`; `herdr pane list --workspace wZ` | test tabs removed; workspace shows only `wZ:p1` | 0 |
| 23 | `resume_project` ui-reference / protocol-compat / env-audit with a background pane wake loop (empty `Enter` every 0.25–0.4 s during launch) | all three Leads started; ui-reference and protocol-compat bound as Herdr agents (`working`), env-audit needed a second resume | 0 |
| 24 | `setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 1800 wX wY wZ` | daemon running (pid 2765519); `ps` confirms the exact command line; stop file `/tmp/relayable-wake-daemon.stop` | 0 |
| 25 | `message_staff` to the three Lead sessions | all returned `ok: true` (delivery ids `2b82f497…`, `fe14f159…`, `68ede95e…`) | 0 |
| 26 | Fresh supervision snapshot after the retry instruction | Option D stopgap working: `research/ui-reference` active=2 Agents, `research/protocol-compat` active=2 Agents (herd run owns its handoff), `research/env-audit` working; no `agent_start` failures reported in this window | observation |
| 27 | Restart bounded stopgap for the full wave: stop old daemon via `/tmp/relayable-wake-daemon.stop`, then `setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 3600 wX wY wZ` | old daemon exited; new daemon pid 2781970 running (expires ~00:05) | 0 |
| 28 | Fresh supervision snapshot after the retry window | all three Leads have 2 active Agents each (`ui-reference` 2, `protocol-compat` 2, `env-audit` 2); the Leads' earlier "fleet-wide blocked" escalations are superseded by this result | observation |
| 29 | `herdr pane list` after rollbacks (earlier check) | no leaked panes: only the four base panes and the transient agent panes existed; Herdsman rollback cleans up failed attempts | 0 |
| 30 | `ui-reference` Lead report after the retry instruction | wave 1 Agents started on the first attempt with no rollback: `ref-video` (session `01a1404e-6f78-704c-9851-9401bd0ee55d`) → `docs/research/video-review.md`, `gorex` (session `01a1404e-7db4-7182-826f-8f8615acf878`) → `docs/research/gorex-review.md`; wave 2 (`ui-catalog.md`, `open-questions.md`) queued | observation |
| 31 | `message_staff` extension notice to all three Leads (window to ~00:05) | all returned `ok: true` (ids `8833d7ec…`, `1fb42be9…`, `4dd29d78…`) | 0 |
| 32 | `ui-reference` wave-1 completion report | `docs/research/video-review.md` = `f067f92` (+562), `docs/research/gorex-review.md` = `a7e14a3` (+420); all five videos inspected with frames + captions; 332 JPEGs kept in gitignored `docs/research/evidence/`; two transient capture errors retried and resolved; video-3 figures labelled non-reproducible; video-5 verbatim `rex` commands captured; wave 2 (`ui-catalog.md`, `open-questions.md`) launched; 0/3 `agent_start` failures | observation |
| 33 | Manager review of `research/env-audit`: `git log --oneline 3db73c2..research/env-audit`, `git diff --name-only/--stat`, worktree status, secret-pattern scan | 6 commits; only `docs/operations/*.md` changed (+1591 lines); worktree clean; no secret-like matches; independent verify/remediate artifacts present | 0 |
| 34 | `git merge --no-ff research/env-audit -m "INT-001: integrate research/env-audit (ENV-001 toolchain audits)"` | merge commit `b24133c`; `docs/operations/` present on `master`; `git status --short` empty | 0 |
| 35 | Wrap-up review of `research/ui-reference` (tip `269018c`) and `research/protocol-compat` (tip `9b0fe37`): `git log`, `git diff --name-only`, worktree status, secret scan | only `docs/research/**` (5 files) and `docs/compatibility/**` (1 file) changed; both worktrees clean; scan found nothing | 0 |
| 36 | `git merge --no-ff research/ui-reference` → `c0578ca`; `git merge --no-ff research/protocol-compat` → `49795a0` | clean merges (disjoint paths): 5 research docs (+1902) and 1 compatibility matrix (+648) on `master` | 0 |
| 37 | `touch /tmp/relayable-wake-daemon.stop` | Option D daemon exited; `ps -p <pid>` shows no process | 0 |
| 38 | `protocol-compat` Lead wrap-up report | tip `9b0fe37` clean, 1 file, 648 lines; labels 342/50/2/1; explicit no-wire statement; 22-row retrieval log. Known gaps: `events.md` line-number citations defective; 7-URL live drift re-check cancelled (facts as-of 16:03–16:08Z); reviewer live pass stopped → tracked as DOC-002 | observation |
| 39 | `git log master..research/ui-reference` → `4102de1`; scope check; `git merge --no-ff research/ui-reference` | 3 files (+15/−11, only `docs/research/**`); merge commit `55d8147`; protocol-compat and env-audit had no commits ahead of `master` | 0 |

**Root cause:** `~/.config/fish/config.fish` (lines 40–43) sources the `herdr-automatic-rename`
fish hook (`shell/hook.fish`). Its `fish_preexec`/`fish_postexec` handlers spawn background `bash`
(`automatic-rename.sh`) on every command/prompt. Herdr's `agent start` requires the pane to be at
its prompt with only the shell in the foreground; the hook's live `bash` children make it report
`agent_pane_busy`. Herdsman's readiness marker (`pane run`) always triggers the hook immediately
before `agent start`.

**Remediation options:** A — stop sourcing the fish hook in `~/.config/fish/config.fish`
(recommended, needs user approval; cosmetic loss of live per-command tab renaming); B — add a
project-local `managed-lead` overlay granting Leads bash/edit/write so they execute directly
(repo-only, reversible, changes the delegation model); C — update/reinstall the plugin (unproven)
or report upstream. See `WORKLOG.md` Checkpoint 3.

## 2026-10-15 — Wave 2 kickoff: DEC-009 option A applied; three Wave 2 projects launched

Environment: Linux `falcon` x86_64 (omarchy), `pi 1.2.0`, pi-herdsman 0.27.0, repo
`/home/sulthonuladib/work/relayable` (`master` @ `b96da27`).

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 40 | `cp -a ~/.config/fish/config.fish /tmp/relayable-config.fish.bak` | backup created; `md5sum` matches source (`60a6536db6ac375f7c0e4d4f2872d609`) | 0 |
| 41 | comment out the automatic-rename hook block (original lines 41–43) in `~/.config/fish/config.fish` | block commented; restore pointer added in-file | 0 |
| 42 | `fish -n ~/.config/fish/config.fish` | no syntax errors | 0 |
| 43 | `fish -c 'functions \| grep -i -E "har_\|automatic"'` | `no automatic-rename hook functions loaded` | 0 |
| 44 | `delegate_project` `research/doc-002` (base `master`) | `ok:true`; session `3534d553-bec1-455c-83df-807ee3ca724a`; workspace `w12` | 0 |
| 45 | `list_staff` | `research/doc-002` Lead `runtime_state: working`, 1 active agent; worktrees `w11` + `w12` | 0 |
| 46 | `delegate_project` `research/term-001-adr-003` | `ok:true`; session `8f442f6e-6f8c-4a03-9016-dcf3f5827aa1`; workspace `w13` | 0 |
| 47 | `delegate_project` `research/adr-001-002` | `ok:true`; session `6a4c1607-f4d3-42fb-b1fc-d6e2dce6e7ba`; workspace `w14` | 0 |

**Interpretation:** option A resolves the `agent_pane_busy` fault (rows 42–45); three disjoint-path
Wave 2 projects are running. No repo code changed by the host fix; the repo work is on the three
branches pending INT-002 review.

## 2026-10-15 — INT-002: Wave 2 review + integration; DOC-002 evidence preserved

Host/repo: Linux `falcon` x86_64 (omarchy), rustc/cargo 1.98.1, `master` @ `f096f23` before merges.

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 48 | `git log master..<branch>` + `git diff --stat master...<branch>` for `term-001-adr-003` and `adr-001-002` | `2e41da4`+`085375c` (spike+ADR-003, 58 files +8300); `69e5600` (ADR-001/002+interface contract, 3 files +1244); only owned paths changed | 0 |
| 49 | secret-pattern scan over both branch diffs (`grep -iE 'ghp_|github_pat_|AKIA…|PRIVATE KEY|password|secret|api[_-]?key'`) | no matches | 1 (no match) |
| 50 | `cat spikes/terminal-engine/results/raw/checks/cargo_build.txt` | `cargo build` `exit_code: 0`; provenance stamped (git_head `b96da27…`, rustc 1.98.1) | 0 |
| 51 | `cat …/checks/cargo_clippy.txt` / `cargo_fmt_check.txt` | `cargo clippy --all-targets` exit 0; `cargo fmt --check` exit 0 | 0 |
| 52 | `cat …/raw/pty_size.txt` | real PTY probe: `CHILD_STTY_SIZE 24 80 → 40 120`, `SIGWINCH_OBSERVED=true`, `VERDICT=PASS` | 0 |
| 53 | `git merge --no-ff research/term-001-adr-003` | merge `78b83f2`; clean (disjoint from `master`) | 0 |
| 54 | `git merge --no-ff research/adr-001-002` | merge `31bed30`; clean (disjoint) | 0 |
| 55 | copy `/tmp/doc002/{citation-audit.md,citation-raw.tsv,fetch-report.md,audit.py,adjudicate.py,urls.txt,fetch.sh,fetch.log}` → `research/doc-002:docs/compatibility/doc-002-audit/`; commit | `2ff3447` (316 KB evidence, durable; matrix repair still NOT applied) | 0 |
| 56 | `git merge master` into `research/doc-002` | merge `669cb84`; branch now current | 0 |
| 57 | `git push origin master` (after `3c3d5ff`) | `f096f23..3c3d5ff master -> master` | 0 |
| 58 | `git push origin research/doc-002` | `* [new branch] research/doc-002` | 0 |
| 59 | `stop_lead` ×3 (doc-002, term-001, adr-001-002) | all three stopped; project work preserved | 0 |
| 60 | `remove_worktree research/adr-001-002` and `research/term-001-adr-003` (×2 attempts) | `cancelled; no changes made` — interactive confirmation declined; worktrees remain | — |
| 61 | `git branch --merged master` / `git branch --no-merged master` | merged: `adr-001-002`, `term-001-adr-003`, Wave 1 branches; unmerged: only `research/doc-002` | 0 |
| 62 | `git merge --no-ff research/doc-002 -m "INT-003: …"` | merge `a1f1719`; `docs/compatibility/doc-002-audit/**` added to `master` | 0 |
| 63 | `git branch --merged master \| grep -E 'doc-002\|adr-001-002\|term-001'` | all three Wave 2 branches now merged | 0 |
| 64 | `remove_worktree` ×3 (`research/doc-002`, `research/adr-001-002`, `research/term-001-adr-003`) | all three `removed checkout …; assignment retired. Git branch and Pi conversation preserved.`; `git worktree list` shows only the primary checkout | 0 |
| 65 | `herdr plugin list` (before) | `herdr-automatic-rename (Herdr Automatic Rename) enabled [github:qu8n/herdr-automatic-rename@b90c4086…]` | 0 |
| 66 | `herdr plugin disable herdr-automatic-rename` then `herdr plugin list` | result `plugin_disabled`, `enabled:false`; list shows `disabled` | 0 |
| 67 | `herdr plugin uninstall herdr-automatic-rename` | `Uninstalled herdr-automatic-rename.`; `herdr plugin list` → `No plugins installed.` | 0 |
| 68 | move residual checkout + empty config dir to `/tmp/herdr-automatic-rename-removed/`; `rm` stale lock; re-check | `github/` empty, config dir gone; fish glob `matches=0`; plugin still unregistered | 0 |
| 69 | delete the now-dead commented hook block from `~/.config/fish/config.fish`; `fish -n`; `grep -niE 'automatic-rename'` | syntax OK; no `automatic-rename` reference remains; backup `/tmp/relayable-config.fish.bak` intact | 0 |
| 70 | set `autoActivateManager: true` in `~/.pi/agent/pi-herdsman/config.json` (user-approved convenience) | JSON valid; new sessions in the primary workspace can auto-take the Manager role; revert by removing the key | 0 |

**Not run / not claimed:** the spike's engine-level phase 2 did not run (cancelled by wind-down); the
DOC-002 matrix repair was not applied (the Lead session had no edit tools and wind-down forbade a new
writer); no workspace-wide `cargo test` exists because no Cargo workspace exists yet.

## 2026-10-15 — Manager independent re-check of the DOC-002 matrix repair + REVIEW-001 findings recorded

Environment: Linux `falcon` x86_64, repo `/home/sulthonuladib/work/relayable`, `master` @ `a1091e5`
(before the review-record commit), python3 from the host.

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 71 | `cp docs/compatibility/doc-002-audit/audit.py /tmp/doc002/recheck.py`; `sed` MATRIX→ the repaired branch copy; `sed` output tsv→`/tmp/doc002/recheck-raw.tsv`; `python3 /tmp/doc002/recheck.py` | **TOTAL 433 links: MATCH 298 / NO_LINE_REF 125 / SHIFT 8 / AMBIGUOUS 1 / NOT_FOUND 1** among the remaining numeric citations — the repair is incomplete; 10 defective rows identified and returned to the DOC-002 Lead with a re-run recipe | 0 |
| 72 | `awk -F'\t' '$7=="SHIFT"||$7=="NOT_FOUND"||$7=="AMBIGUOUS"' /tmp/doc002/recheck-raw.tsv` | matrix lines 85 (cli.md:81), 85 (server-api.md:85–86), 219 (server-api.md:63–71), 299 (lua.md:125–132, AMBIGUOUS), 304 (lua.md:313–327), 475 (program-status.md:79–81), 490 (program-status.md:110–111), 528 (program-status.md:168), 529 (program-status.md:169), 594 (events.md:71, NOT_FOUND) | 0 |
| 73 | read `docs/reviews/REVIEW-001-adr-review.md` (review worktree) | verdicts: ADR-001 accept-with-nits (A-1 blocking for CORE-001 model code); ADR-002 accept-with-nits (B-1/B-2/B-3 blocking for DEC-003 acceptance); ADR-003 keep `Proposed` (TERM-002 gates engine, UI-001 gates gpui pin); ADR-002 Unverified: 9/10 settled, #10 Android QUIC open | 0 |
| 74 | `edit docs/architecture/interface-contract.md` (`PtyId` → `PtyProcessId`) | A-1 reconciled; the only review item blocking CORE-001 model code | 0 |
| 75 | `commit` + `git push origin master` | `446fe5b review-001: record verdicts; reconcile PTY ID newtype (A-1) unblocking CORE-001` | 0 |

**Not yet verified:** REVIEW-001 branch not committed/merged at re-check time; TERM-002 and CORE-001
in flight; `ADR-FIX-001` (ADR-002 B-1/B-2/B-3 + ADR-001 nits) not started.

## 2026-10-15 — CORE-001 vertical slice: independent Manager validation on `master` (`d3e0286`)

Environment: Linux `falcon` x86_64, rustc/cargo 1.98.1, `protoc` libprotoc 36.1, repo
`/home/sulthonuladib/work/relayable`. Run AFTER `git merge --no-ff core/core-001-vertical-slice`
(INT-008 `d3e0286`).

| # | Command | Observed result | Exit |
|---|---|---|---|
| 76 | `bash scripts/validate.sh` | fmt/clippy(`-D warnings`)/build all exit 0; `cargo test --workspace` all suites green (Σ ≈ 104 passed / 0 failed); lifecycle acceptance 4/4; ADR-001 dependency-direction check passed; `cargo build` in `spikes/terminal-engine` exit 0; **All CORE-001 validation checks passed** | 0 |
| 77 | `cargo test -p relayable-integration-tests --offline --test lifecycle -- --nocapture` | property-0 server start (socket mode `0600`); property-1 server-owned PTY (`ppid == server pid`, `fd0=/dev/pts/*`, `stty '24 80'`); property-1b resize(100,40)→`stty '40 100'`; property-2 `session_id=session:01a1412f-…` + tagged window/block/pty; property-3 I/O round-trip; property-4 disconnect survival (same pid active after client drop); property-5 reconnect (same ids + dims + writer lease + survival nonce); property-6 close reaps pid; property-7 SIGTERM exit 0 + socket removed; 4 passed / 0 failed | 0 |
| 78 | `cargo build --workspace --offline` (post-merge) | `Finished dev profile` (5.36 s) | 0 |

**Decisions recorded:** the CORE-001 first-release method set (`protocol/compatibility/core-001-method-set.md`)
and CLI/JSON surface (`core-001-cli-json.md`) are accepted as the server-API task’s decision (the parent
contract defers the method list to it). `RELAYABLE_SERVER`/`RELAYABLE_SESSION` are CLI inputs; no `REX_*`
is read. `rust-toolchain.toml` intentionally not created (1.98.1 is not an installed rustup toolchain
name); `rust-version = "1.98"` is used instead.

**Known residual gap:** the server does not yet set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER`
in the managed child terminal (ADR-005 Decision 2) and the ADR-005 §8.1 context test does not exist —
tracked as `CTX-001`.

**Not yet verified:** TERM-002 engine verdict; UI-001 (GPUI pin build/run).

## 2026-10-15 — CORRECTION: the DOC-002 recheck (rows 71–72) was a false positive

Rows 71–72 above flagged 10 "defective" numeric citations in the DOC-002 repair using the automatic
`docs/compatibility/doc-002-audit/audit.py`. **That was the wrong standard.** The assignment's authority
is the *adjudicated* `docs/compatibility/doc-002-audit/citation-audit.md` (produced by `adjudicate.py`,
which manually resolves rows where `audit.py`'s automatic fragment union is unreliable). Re-adjudicating
each flagged row directly against `/tmp/doc002/raw/**`:

- `events.md:71` — sanctioned (audit row 446† says "keep 71"); line 71 really is the
  `client_connection_changed | client_id, principal, connection_state` row.
- `program-status.md:168` / `:169` — sanctioned; they really are the `title` / `msg` rows (the shared
  "base64 UTF-8" quote makes `audit.py` report `actual=168-169` for both, a SHIFT false positive).
- `cli.md:81`, `server-api.md:63–71`, `server-api.md:85–86`, `lua.md:125–132`, `lua.md:313–327`,
  `program-status.md:79–81`, `program-status.md:110–111` — all adjudicated MATCH or corrected in
  `citation-audit.md`.

`audit.py` re-derives spans automatically and false-positives on paraphrased evidence and on repeated
quotes. The repaired matrix agrees with the adjudicated audit (0 invented refs; 78 NOT_FOUND converted
to heading anchors). **DOC-002 is accepted; the revision request (`ea419e3b`/`88505e2b`) was withdrawn.**

## 2026-10-15 — TERM-002 engine-level spike: independent Manager validation (`60d4395`)

Environment: Linux `falcon` x86_64, rustc/cargo 1.98.1, repo `/home/sulthonuladib/work/relayable`.
Run in the spike's own workspace root (`spikes/terminal-engine-phase2`).

| # | Command | Observed result | Exit |
|---|---|---|---|
| 79 | `cargo build --offline`; `cargo clippy --all-targets --offline -- -D warnings`; `cargo fmt --check` | all exit 0 | 0 |
| 80 | `cargo test --offline` | `42 passed; 0 failed` (+0 in bins/doc-tests) | 0 |
| 81 | read `spikes/terminal-engine-phase2/results/REPORT.md` §3–§9 | `alacritty_terminal 0.26.0` + `vte 0.15.0` + `portable-pty 0.9.0` = 32 PASS / 1 documented UNSUPPORTED / 0 FAIL / 0 NOT_TESTED on real PTYs; `gpui = "=0.2.2"` resolves but `cargo check` exits 101 at `xattr 0.2.3` (`libc::ENOATTR` removed in libc 0.2.190); remedies (hold libc ≤0.2.189 or patch xattr) NOT applied | 0 |

**Decisions recorded:** engine pins accepted; no engine change (`vt100` lacks OSC 8 + reflow; `avt` lacks
title/mouse/bracketed-paste/OSC 8). `gpui` pin accepted only with the documented caveat; ADR-003 stays
`Proposed` until UI-001 proves a contained remedy and a window build/run. Full report caveats: vte is
exercised only transitively through `alacritty_terminal`; palette→RGB, Android rendering, and the
`TerminalSnapshot` mapping are untested in this spike.

## 2026-10-15 — UI-001 GPUI window: independent Manager validation (`09928db`)

Environment: Linux `falcon` x86_64, cargo/rustc 1.98.1, live Wayland session. Run in `apps/linux`
(its own workspace).

| # | Command | Observed result | Exit |
|---|---|---|---|
| 82 | `cargo check --locked` | exit 0; only a transitive `proc-macro-error2` future-incompat warning (not from `relayable-linux`) | 0 |
| 83 | `cargo test --locked` | `9 passed; 0 failed` | 0 |
| 84 | `grep -A2 'name = "libc"' apps/linux/Cargo.lock` (and xattr) | exactly one `libc 0.2.189` and one `xattr 0.2.3` | 0 |
| 85 | inspect `apps/linux/evidence/relayable-window.png` (`file` + read) | real PNG `950 x 1030`; Manager visually confirmed the Relayable window render (dark `rgb(9,30,46)` surface + "Relayable" label) | 0 |
| 86 | `cargo check --workspace --offline` (repo root, post-merge) | exit 0 — the core workspace is unaffected by `apps/linux` | 0 |

**Decisions recorded:** the `gpui = "=0.2.2"` build is resolved by a `libc 0.2.189` lockfile hold in the
separate `apps/linux` workspace (no `[patch.crates-io]` needed); **ADR-003 → Accepted** (DEC-004)
with reconsideration trigger 4 discharged. Forward constraint recorded in ADR-003 consequences: a
future `apps/linux` dependency on a crate pinning `libc = "=0.2.190"` would re-break `gpui`.
