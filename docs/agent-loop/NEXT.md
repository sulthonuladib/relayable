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

- **TERM-003 stage B** — implementer on branch `spike/term-003-libghostty` (worktree
  `/home/sulthonuladib/work/relayable-wt/term-003`), a throwaway crate with its own workspace at
  `spikes/terminal-engine-phase3/`. It must link the **already-built** `libghostty-vt` and prove
  byte-in → terminal-state-out from Rust (text + escape sequences, cells, cursor, resize/reflow, scrollback,
  OSC title), with measured time/size/RSS and the API-instability warning carried into the verdict.
  Zig is pinned in the repo's `mise.toml` (0.16.0) — it resolves **only inside this checkout**; elsewhere use
  `~/.local/share/mise/installs/zig/0.16.0/zig`. Artifacts live in the vendor clone (never committed).
- **Nothing else is in flight.** `master` is at INT-028 (`5abe262`; toolchain commit `5c735fb`), clean and
  pushed, fully green: `validate.sh` 206/0, `smoke.sh` OK, `--strict` exit 0, desktop **72/0**, live probe
  `PROBE_EXIT=0`, `ci.sh` **OVERALL: PASS (16/0/3)**.
- **Only the main checkout exists** plus any in-flight spike worktree — removable under the standing cleanup
  authorization once merged.

## Next queue (in order)

1. **`UI-005`** — implement from **`docs/architecture/ui-005-plan.md`** (a complete planner brief: current
   binding table at `app/keys.rs:10-61`, the single route `render.rs:44-49`, the owned paths, the pure
   `shortcuts()` table + `shortcut_for` interface, the `Ctrl+Shift+/` toggle decision with its live-verify
   caveat, the display-free test list, and the acceptance commands). Bindings stay **Relayable-original**.
2. **`UI-006`** — local token-driven theming against `docs/research/design-tokens.md`; **not** gpui-kit
   (DEC-014). Do not restyle: reference geometry/hierarchy is the acceptance gate.
3. **`DOCS-001`** — `docs/README.md` index + consolidation (evidence preserved); then **`DOC-004`** (the
   ADR-004 residuals plus the citation drift DOC-003 proved but did not fix).
4. Backlog: `CLIENT-API-003` (ack-rate limit; core change), `QA-DESKTOP-001` (wire the live probe into a
   script so the drain-all ack invariant is guarded automatically), `UI-004-STAGE2` items (b) and (c) only —
   (a) is **declined** by DEC-020 — plus `MEM-001`, `CI-HOST-001`, `QA-FLAKE-001`, `TOOL-001`, `TOOL-002`.
5. **Cheat-sheet:** desktop = `cd apps/linux && CARGO_NET_OFFLINE=true cargo <fmt|clippy --all-targets --locked
   -- -D warnings|test --locked|check --locked>`; never `cargo update` there. Root = `scripts/validate.sh`,
   `scripts/smoke.sh`, `scripts/check-file-sizes.sh --strict`, `scripts/ci.sh`.

## Pending user decisions (the loop continues without them)

1. **TERM-003 stage B approvals (DEC-022) — blocking; stage A evidence is in
   `docs/research/libghostty-adoption.md`.** The released tag **does** contain the engine (as a Zig module);
   `mise` offers exactly the required Zig. Requested:
   - **A1** — `mise install zig@0.16.0` (per-user, no `sudo`, no global package).
   - **A2** — one-time networked provisioning of the pinned source artifact (+ `zig fetch` seed); a
     checksummed per-commit artifact exists, but 3 of 6 URL probes 404'd, so a vendored tarball is safer.
   - **A3** — confirm stage B may add a throwaway spike crate with its own workspace under `spikes/**`.
   - Plus the **adoption shape**: replace `alacritty_terminal`+`vte` / additive (encoders only) / own thin
     C ABI over tag `v1.3.1`. Note the swap is **not** contained to `term/**` — `apps/linux/src/api/osc.rs:1`
     uses `vte::Perform` directly. No ADR-003 amendment is drafted; ADR-003's pins stay authoritative until
     one lands, and "no engine change" remains a defensible outcome if the spike fails.
2. **DEC-014 (gpui-kit part only)** — unchanged: research says Reject/Defer; UI-006 is a local token-driven
   theme, not a component-layer adoption. The libghostty half of DEC-014 is superseded by DEC-021/022.
3. **DEC-020** — recorded: the app creates a **new** session on open (no restore); `UI-004-STAGE2` (a) is
   closed as declined. (If that misreads your intent, it is a one-line ledger change.)
4. **Worktree cleanup** — standing authorization applies; `git worktree list` currently shows only the main
   checkout.

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
