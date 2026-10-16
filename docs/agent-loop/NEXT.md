# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), `docs/architecture/interface-contract.md`, `docs/adr/ADR-001..005`,
and this file. Trust repo state over chat.

> **State note (2026-10-16, after the DEC-015 OpenCode migration checkpoint):** the "In flight" list
> below predates `STATUS.md` and `NEXT-SESSION-PROMPT.md` and is **not** current — CTX-001,
> INSTALL-001, CLIENT-API-001, STORE-001, REF-006/007/009, and RESEARCH-001/002 are integrated.
> Reconcile actual task state from `git log`, `git worktree list`, and `STATUS.md` before acting.
> Only the harness statement below and the ordered work in `NEXT-SESSION-PROMPT.md` are current.

> **Restart trigger (DEC-008/015 — compaction is disabled):** if this session approaches its context
> limit, STOP launching new work, finish a safe checkpoint (commit + push + update
> `docs/agent-loop/**`), and tell the user to start a fresh session with the next-session prompt.
> Repository files are authoritative; never rely on conversation memory. Project work is durable by
> Git branch, so a restart loses nothing. `opencode.json` sets `default_agent: orchestrator`, so a
> fresh session opened in this workspace re-takes the lead role; `scripts/agent-loop.sh` runs that as
> a sequence of fresh sessions; otherwise paste `NEXT-SESSION-PROMPT.md`.

## First commands

```bash
cd /home/sulthonuladib/personal/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
ls docs/adr docs/architecture spikes/terminal-engine docs/compatibility/doc-002-audit docs/reviews
```

## Current state (lead session `ses_ebab9726a`, 2026-10-16)

`master` is clean and pushed; **the whole refactor wave is integrated** — REF-008 (INT-022 `afcc631`),
CLIENT-API-002 (INT-023 `494917f`), QA-HARNESS-001 (INT-024 `7c8f5a2` + `1a54cc4`), REF-010 (INT-025
`4a4cb16`). Baseline **verified green**: `scripts/validate.sh` exits 0 (8 steps, **206 passed /
0 failed**), `scripts/smoke.sh` → `SMOKE OK`, `scripts/check-file-sizes.sh --strict` → exit 0
(`over-hard-cap=0`). `protoc` is present. `ci.sh`'s new `file-sizes` row is PASS; one pre-existing row
(`spike-terminal-engine-phase2`) cannot pass offline — see DEC-018.

**Read next:** `STATUS.md` "Current execution", `WORKLOG.md` Checkpoints 40–41, `DECISIONS.md`
DEC-017–019, and the "Pending user decisions" section below.

> **A stopped loop session ran with the wrong model (2026-10-16):** the first unattended run used the
> session default, which resolved to a free model (`ll-alpha-free`), because no model was pinned. It was
> stopped before making changes; the tree was clean. `opencode.json` now pins the model, and it is
> verified (the session export reports `providerID: opencode-go`).

**Historical — superseded (both projects below are integrated; kept for path-ownership history):**

- **CTX-001** — `ctx/ctx-001-context-env`, worktree `/home/sulthonuladib/work/relayable-wt/ctx-001`:
  set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed terminals (ADR-005
  Decision 2) + the §8.1 `tests/context-resolution` integration test. Owns
  `crates/relayable-server/src/{registry,config}.rs`, `tests/Cargo.toml`, `tests/context-resolution/**`.
- **INSTALL-001** — `install/install-001-user-install`, worktree
  `/home/sulthonuladib/work/relayable-wt/install-001`: `scripts/install-user.sh`, a `systemd --user`
  unit, a desktop entry, root `README.md`, and `docs/operations/user-install.md` (the primary user
  goal). Owns those paths only.

Neither in-flight branch may edit `docs/agent-loop/**` or `docs/adr/**`; the Manager integrates and owns
those. **Do not run STORE-001 concurrently with CTX-001**: both touch
`crates/relayable-server/src/registry.rs`.

## Single-writer claims (lead session `ses_ebab9726a`, 2026-10-16)

**Read this before launching any writer.** A second lead session ran concurrently on this checkout
(`WORKLOG.md` Checkpoint 39); both read the same queue. To prevent two writers on one path, this
session claims the **whole ordered queue**:

`REF-008` (in flight, `dbee61d`), `QA-HARNESS-001` (in flight), `CLIENT-API-002` (new, in flight),
`REF-010`, `UI-003`, `UI-004`, `UI-005`, `UI-006`, `DOCS-001`.

Any other live session must **not** spawn writers on those paths. It may take a backlog item outside
that set (`MEM-001`, `CI-HOST-001`, `REF-004` remainder, `QA-FLAKE-001`) after recording the claim
here, or stop and report. `docs/agent-loop/**` + `docs/adr/**` + `docs/architecture/**` bookkeeping is
also owned by this session; other sessions should append to `WORKLOG.md` only via a checkpoint of
their own and never rewrite this file's queue without recording it.

**Delegation lesson (supersedes the open question in Checkpoint 39):** OpenCode subagents *do* work
when the child is spawned from the main checkout and simply told which worktree path to work in (this
session's REF-008/QA-HARNESS-001/CLIENT-API-002 implementers and the UI-003 planner all ran).
`session_move` on a *child* session is what produced `Agent not found: implementer` for the other
session. Do not move a child session; name the worktree path in the brief.

## In flight (lead session, 2026-10-16)

- **UI-003** — implementer, branch `ui/ui-003-next-output`, worktree
  `/home/sulthonuladib/work/relayable-wt/ui-003`. **Server restart mid-session:** branch head is still
  `494917f` and the work is **uncommitted in the tree** (568 insertions / 418 deletions across
  `api/{data,data_thread,mock,mod,tests,types}.rs`, `app/streams.rs`, `bin/live-probe.rs`, plus untracked
  `apps/linux/evidence/ui-003-live-probe.log` ending `PROBE_EXIT=0` after a flow-control observation).
  Owns `apps/linux/src/api/{data,data_thread,mod,types,mock,tests}.rs`, `apps/linux/src/app/streams.rs`,
  `apps/linux/src/bin/live-probe.rs`, `apps/linux/README.md`, `apps/linux/evidence/ui-003-*.log`.
  Moves the terminal data plane onto `relayable_client::Client` (`next_output` +
  `resize_with_pixels`), keeping the two-plane split and the reference geometry; **must drain every
  attached stream each `pump()`** (credit-window acking) and must not touch `crates/**`,
  `apps/linux/Cargo.lock`, `src/{main,ui,term,geometry*,theme*,tokens}**`.
- **UI-004 stage 1** — implementer done, commit `ff1e8fc` on `ui/ui-004-stage1-auto-session`
  (worktree `/home/sulthonuladib/work/relayable-wt/ui-004-stage1`). **Awaiting independent verification
  before integration** (the verification call was aborted by the shutdown): auto-session invariant
  re-checked at 4 sites, `abandon_pending_create` on `Error` **and** `Disconnected`, desktop tests 58→65,
  M1–M5 mutations caught but the `close_focused` call site is not covered headlessly, defect 3 and
  restore-session deferred (DEC-019).
- **DOC-003** — committed `af72801` (28 split-invalidated citations repaired in ADR-004,
  `docs/architecture/code-organization.md`, `protocol/compatibility/core-001-method-set.md`); residuals
  and the non-split-induced drift DOC-003 proved are recorded as **DOC-004**.
- Nothing else is in flight. The `ref-008`/`ref-010`/`client-api-002`/`qa-harness-001` worktrees were
  removed (branches preserved).

## Pending user decisions (the loop continues without them)

1. **DEC-014 — gpui-kit / libghostty.** Unchanged: research recommends Reject/Defer, ADR-003's pins
   stay. UI-006 (theming) is therefore scoped as a **local token-driven theme** inside `apps/linux`,
   not as adopting a component layer.
2. **DEC-018 — `spikes/terminal-engine-phase2` offline cache gap.** `avt` is not in the cargo cache, so
   that one `ci.sh` row cannot pass offline. Approving a one-time networked `cargo fetch` in that spike
   workspace (DEC-012 pattern) restores `ci.sh → OVERALL: PASS`. No agent fetches it without approval.
3. **DEC-019 stage 2 — restore-an-existing-session policy.** After UI-004 stage 1 the app still creates a
   *new* session on each launch instead of restoring the previous one (a deviation from the only
   behavioural reference evidence). Stage 2 needs `apps/linux/src/api/**` work plus a decision: persist
   the last-used session id locally (Relayable-original) or keep always-creating.
4. **Worktree cleanup authorization:** standing authorization for done+merged worktrees applies
   (AGENTS.md); no per-removal confirmation is needed.

## Open tasks, in order

1. **UI-003** (in flight) — verify → integrate.
2. **UI-004 stage 1** (DEC-019) — auto-session invariant + truthful empty/creating text, `app/**` only.
3. **UI-005** — keymapping helper / discoverable shortcut overlay (after UI-004).
4. **UI-006** — local token-driven theming (no gpui-kit, per DEC-014) — verify fidelity against
   `docs/research/design-tokens.md`.
5. **Offline-safe queue, can run any time:** **DOC-003** (repair the citations invalidated by the
   refactor splits — `docs/adr/ADR-004-*` ×9–10, `docs/architecture/code-organization.md`,
   `protocol/compatibility/core-001-method-set.md`; citation-only, no decision changed),
   **DOCS-001** (`docs/README.md` index + consolidation, evidence preserved), **MEM-001** (memory
   baseline, root workspace), **CI-HOST-001**, **QA-FLAKE-001** (`server_e2e`
   `resume_shapes_and_writer_takeover` / `idle_timeout_reason_is_reported_to_a_subscriber`).
6. **TERM-003** stays proposed pending the user's DEC-014 answer.
7. Update `docs/agent-loop/**` after every meaningful iteration.
8. **Cleanup:** remove the merged worktrees (`ref-008`, `ref-010`, `client-api-002`, `qa-harness-001`)
   under the standing authorization; keep branches.

## STORE-001 SQLite binding — decided: option A (2026-10-16)

**User approved option A.** STORE-001 (ADR-004 Decision 2) now uses `rusqlite = "=0.40.2"` with the
`bundled` feature (`libsqlite3-sys 0.38.2`, SQLite compiled from the amalgamation; no runtime system
SQLite dependency). A one-time networked fetch populated the cargo cache (2026-10-16); **all subsequent
builds stay `--offline`**. Recorded as **DEC-012**. No `unsafe` FFI (option B) and no further deferral
(option C).

## Review gates (do not skip)

- **All five ADRs are Accepted** (ADR-001/002/003/004/005).
- No REX wire/binary compatibility claims (`full.md` §3); no GoRex source/asset reuse (§2.6).
- UI fidelity: catalogue + design tokens are the acceptance gate; unverified states stay labelled.
- Android is deferred (DEC-010): no Android app/SDK/emulator/Gradle work now, but keep
  `relayable-model`/`relayable-protocol` Android-ready (no Linux-only assumptions; §1.3.6, §5.2).
- Never claim server-crash or host-reboot session survival (§6.3).
- The root workspace must not absorb `spikes/**` (verify `cargo build` still works in each spike dir).

## Approval gates

- **None active** for the current queue. Android SDK/emulator + JDK pin are deferred (DEC-010).
- **Networked cargo fetch (DEC-018):** needs explicit user approval before any agent runs it; the
  `spikes/terminal-engine-phase2` `ci.sh` row stays red until then. Recorded above under "Pending user
  decisions".

## Environment / cleanup

- **Toolchain/cache state (DEC-018, 2026-10-16):** `rustc`/`cargo` **1.99.1**; root workspace and
  `apps/linux` build offline, `spikes/terminal-engine-phase2` does not (`avt` uncached). Prefer
  `--offline` / `CARGO_NET_OFFLINE=true` in every cargo invocation you write; never `cargo update` in
  `apps/linux` (the lockfile holds the TERM-002 `libc 0.2.189` remedy).
- Delegation uses fresh bounded agents; one writer per branch/path. Agent roles and their write
  permissions are defined in `.opencode/agents/` (see `AGENTS.md` "Agent roles (OpenCode)"). **Spawn a
  child from the main checkout and name the worktree path in the brief; never `session_move` a child**
  (that produced `Agent not found: implementer` for another session).
- **Orchestration is OpenCode (DEC-015).** The herdr + Pi stack is no longer used by this repository:
  `.pi/settings.json` was removed and no script depends on herdr. DEC-009's `agent_pane_busy` fault and
  its `/tmp/relayable-wake-daemon.py` fallback are historical and must not be restarted — the
  mechanism they worked around no longer exists. Host-level herdr configuration was left untouched;
  if on-screen agent panes or SSH-attached sessions are wanted later, that is a host-level choice
  (`herdr integration install opencode`), not a repository dependency.
- **Git remote is SSH** (`git@github.com:sulthonuladib/relayable.git`, 2026-10-16). The HTTPS remote
  cannot authenticate non-interactively on this host (no credential helper and `gh` is not logged
  in), so an unattended loop session cannot push over HTTPS. Push with plain `git push origin master`.
- Worktree cleanup (AGENTS.md "Worktree lifecycle / cleanup"): done+merged task worktrees may be removed
  by the lead, preserving branches. Check `git worktree list` after any cleanup and record it in
  `WORKLOG.md`.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.

## Concurrency

- **One lead session per checkout.** On 2026-10-16 two lead sessions ran concurrently and both targeted
  REF-008, so two implementers were pointed at one worktree; the duplicate was moved out of the repository
  (`WORKLOG.md` Checkpoint 39). Before launching work, check `opencode session list` for another live lead
  and `git worktree list` for worktrees whose ledger entry says another owner.
- **Delegation is unverified under OpenCode.** A background `subagent(agent: "implementer")` spawn was
  accepted but the child failed with `Agent not found: implementer` after `session_move`; verify the spawn
  path (and its interaction with worktrees) before relying on delegated work (`WORKLOG.md` Checkpoint 39).
