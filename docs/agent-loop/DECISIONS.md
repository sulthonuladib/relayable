# DECISIONS — index

Detailed ADRs live under `docs/adr/`. Each ADR contains: context, options, decision, consequences, and what evidence would cause reconsideration (`full.md` §5.2).

| ID | Decision | Status | ADR |
|---|---|---|---|
| DEC-000 | Work state is tracked in-repository (`docs/agent-loop/`), not in chat | **Accepted** | — |
| DEC-001 | Repo is Phase 0 spec-only; establish minimal workspace rather than migrating existing code | **Accepted** | — |
| DEC-002 | Domain model and dependency direction | **Accepted** — ADR-001 integrated `31bed30`; REVIEW-001 corrections applied via ADR-FIX-001 (INT-007 `f474c83`) | `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md` |
| DEC-003 | Protocol serialization + transport | **Accepted** — ADR-002 integrated `31bed30`; REVIEW-001 B-1/B-2/B-3 corrections applied via ADR-FIX-001 (INT-007 `f474c83`) | `docs/adr/ADR-002-protocol-schema-and-transport.md` |
| DEC-004 | Terminal engine selection | **Accepted** — ADR-003 integrated `78b83f2`; TERM-002 validates the engine pins (32 PASS/0 FAIL); UI-001 resolved the `gpui = "=0.2.2"` build (libc 0.2.189 lockfile hold) and ran a real window; INT-009 `60d4395` + INT-010 `09928db` | `docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md` |
| DEC-005 | Local/remote process ownership and persistence guarantees | **Accepted** — ADR-004 integrated `9a3fb27` (INT-014); server owns PTYs in-process. The SQLite metadata/audit store is now **implemented (first slice)** by STORE-001 (INT-018 `f4d59f8`); crash/reboot survival still NOT claimed | `docs/adr/ADR-004-process-ownership-and-persistence.md` |
| DEC-006 | Env var namespace: `RELAYABLE_*`, REX aliases, or both | **Accepted** — `RELAYABLE_*`-only, no `REX_*` aliases; `ADR-005` integrated `74e7e20` | `docs/adr/ADR-005-environment-variable-namespace.md` |
| DEC-007 | Wave 1 research split into three branch-isolated, documentation-only projects with exclusive path ownership; Manager integrates before Wave 2 | **Accepted** | — |
| DEC-008 | Context-window policy: no automatic/manual compaction; repository files are authoritative; fresh bounded agent sessions; checkpoint-and-handoff at context pressure | **Accepted** | — |
| DEC-009 | Herdr agent-provisioning fault remediation (stop sourcing the automatic-rename fish hook vs project-local lead tool overlay) | **Accepted — hook disabled + plugin uninstalled 2026-10-15** | — |
| DEC-010 | Android client deferred from current scope; keep the model/protocol Android-ready | **Accepted** — user direction 2026-10-15; consistent with `full.md` §1.4/§1.5 | — |
| DEC-011 | ADR-005 Decision 5 clarification: a **unique** session is not an ambiguous match, so the accepted CORE-001 CLI unique-session default is retained (flag → env → unique-default); zero/multiple sessions still fail with documented codes | **Accepted** — Manager reconciliation 2026-10-16 (addendum in ADR-005) | `docs/adr/ADR-005-environment-variable-namespace.md` |
| DEC-012 | STORE-001 SQLite binding: use `rusqlite = "=0.40.2"` with the `bundled` SQLite (`libsqlite3-sys 0.38.2`); one-time user-approved networked fetch, then offline builds | **Accepted** — user approval 2026-10-16 (option A) | `docs/adr/ADR-004-process-ownership-and-persistence.md` (Decision 2 binding deferred here) |
| DEC-013 | Relayable is an **independent project**; docs must not attribute it to Superlogical (the maker of the reference product REX). Superlogical appears only in research/compatibility evidence | **Accepted** — user direction 2026-10-16 | `full.md` §0.1; `AGENTS.md`; `README.md` |
| DEC-014 | RESEARCH-001/002 outcome: **gpui-kit** = Reject (can't coexist with pinned `gpui =0.2.2`; no umbrella license file) / Defer; **libghostty-rs** = Defer/Partial (terminal API unreleased on `main`, Zig build blocker, leading binding unlicensed) | **Recommendation recorded — user decision pending**; ADR-003 pins unchanged | `docs/research/gpui-kit.md`, `docs/research/libghostty.md` |

## DEC-000 — Work state in-repository

**Context:** `full.md` §15 requires a durable checkpointed loop; §19.14 requires resumability from files alone.
**Decision:** All phase/status/task/next-step state lives under `docs/agent-loop/`. Chat is not authoritative.
**Consequences:** Every agent must read these files at run start and update them at each checkpoint.

## DEC-001 — Phase 0 empty-repo bootstrap

**Context:** Phase 0 discovery found only `full.md`; no code, manifest, CI, or `AGENTS.md`.
**Decision:** Establish the minimal workspace (`AGENTS.md`, loop docs, Cargo workspace, smoke tests) rather than adapting existing architecture.
**Consequences:** No migration risk; §0.4 "if the repository is empty, establish a minimal workspace" applies.
**Reconsider if:** uncommitted or on-disk work is discovered that the initial inventory missed.

## DEC-007 — Wave 1 research orchestration model

**Context:** `full.md` §14 requires bounded delegation with explicit path ownership and one
integrator; §19.5–19.7 requires parallel research before implementation. Three research streams
(video/GoRex UI evidence, official REX behavior docs, toolchain audit) are independent and do not
share files.
**Decision:** run each stream as its own Pi Herdsman project on an isolated Git branch
(`research/ui-reference`, `research/protocol-compat`, `research/env-audit`) with exclusive owned
paths; each branch is documentation-only and must not touch `docs/agent-loop/**` or application
code. The Manager reviews each diff and integrates into `master` (INT-001) before Wave 2 starts.
**Consequences:** no overlapping writers; parallel evidence gathering; integration is a discrete,
reviewable step; Wave 2 (ADRs, terminal spike, CORE-001) waits on integrated evidence.
**Reconsider if:** the streams turn out to share files, or branch isolation costs more than one
integrated research worktree would save.

## DEC-008 — Context-window policy (no compaction; repository-authoritative state)

**Context:** The user requires project knowledge to survive in repository files rather than
conversation summaries, and context exhaustion must never silently lose work (§15.3 checkpoint
discipline; §19.14 resumability from files alone).

**Decision:**

- Automatic compaction is disabled for this project: `compaction.enabled: false` in
  `.pi/settings.json`. The user-level `~/.pi/agent/settings.json` already had the same value and is
  left unchanged.
- `/compact` is never invoked unless the user explicitly requests it.
- Work state is authoritative in `docs/agent-loop/{STATUS,TASKS,BACKLOG,DECISIONS,TESTS,WORKLOG,NEXT}.md`;
  every checkpoint records branches/worktrees, uncommitted changes, running agents, blocked tasks,
  failing tests, and the next integration step.
- Bounded work goes to fresh agent sessions; handoffs are concise (files changed, commands actually
  run, findings/decisions, risks, remaining work). Full research reports and `full.md` are not
  injected into every task brief; recipients read repository paths.
- At usable-context pressure: finish a safe checkpoint, stop launching new work, record background
  results and repository changes, and produce a short handoff instead of compacting.
- A resumed Lead reads `full.md`, repository instructions, `NEXT.md`, `BACKLOG.md`, `DECISIONS.md`,
  and Git state, then continues at the next incomplete acceptance criterion.

**Verified facts (2026-10-15):**

- Pi 1.2.0 supports `compaction.enabled` (default `true`); project `.pi/settings.json` overrides
  agent-directory settings; changes take effect on `/reload` or a new Pi process.
- `SettingsManager.create(cwd).getSettings()` exposes the merged effective settings for verification.
- pi-herdsman 0.27.0 defaults `contextRetirement` to `true` (`config.json` overrides only
  `spawnPlacement`): assigned managed-agent sessions retire at automatic context pressure instead of
  threshold compaction.
- No extension other than pi-herdsman registers compaction hooks; the global `herdr-agent-state.ts`
  extension does not touch compaction.

**Consequences:** long-lived sessions must manage context explicitly through checkpoints and fresh
sessions; no conversation summary is a dependency of the project. Settings changes apply at process
start/reload, so a running session may retain its startup configuration until reloaded; this is
reported rather than worked around by restarting processes.

**Reconsider if:** the user explicitly asks for compaction, or repository-file discipline proves
insufficient for a bounded session.

## DEC-009 — Herdr agent-provisioning fault: option A applied

**Context:** `herdr agent start` failed with `agent_pane_busy` for every programmatic delegation
while the `herdr-automatic-rename` plugin's fish hook spawned background `bash` on every
`fish_preexec`/`fish_postexec`. Diagnosis and A/B evidence: `WORKLOG.md` Checkpoint 3, `TESTS.md`
rows 15–23.

**Decision (user-authorized, 2026-10-15):** apply option A — stop sourcing the hook in
`~/.config/fish/config.fish` (the block at original lines 40–43). New panes now start without the
hook's background `bash` children, so Herdr accepts `agent_start`. The Wave 1 Option D wake daemon
stays stopped; it remains a documented fallback if a future environment regresses.

**Change made:** the `for _f in …herdr-automatic-rename-*/shell/hook.fish` block was first commented
out (with a restore pointer). Backup: `/tmp/relayable-config.fish.bak` (md5
`60a6536db6ac375f7c0e4d4f2872d609`). Later, after the plugin was uninstalled, the block was **removed
from `config.fish` entirely**. Verified with `fish -n` (exit 0), a `grep` showing no
`automatic-rename` reference in `config.fish`, and a fresh `fish -c 'functions'` check showing no hook
functions loaded; a real `delegate_project` then succeeded (`w12`/`research/doc-002`).

**Additional step (user request, 2026-10-15):** the plugin itself was then fully disabled —
`herdr plugin disable herdr-automatic-rename` → `plugin_disabled`, `enabled: false` (verified with
`herdr plugin list`). Disabling the plugin alone would **not** have fixed the fault, because the shell
hook is sourced independently of plugin enable state and its `bash` children are what made Herdr
report `agent_pane_busy`; that is why the `config.fish` hook block was disabled. The plugin was
then **uninstalled** (`herdr plugin uninstall herdr-automatic-rename` → `No plugins installed.`), the
residual checkout/config directories that herdr leaves behind were moved to
`/tmp/herdr-automatic-rename-removed/`, and the now-dead commented hook block was deleted from
`config.fish`. No `automatic-rename` reference remains in `config.fish`. Reinstall from
`github:qu8n/herdr-automatic-rename@b90c4086…` if ever wanted.

**Consequences:** all automatic tab/workspace naming stops, and the plugin is no longer present.
No repo file depends on it. These are host-configuration changes outside the repo, made under the
user's explicit instruction. The Wave 1 Option D wake daemon remains a documented fallback.

**Reconsider if:** Herdr changes its pane-readiness check, or the plugin fixes the hook.

## DEC-010 — Android client deferred (keep Android readiness)

**Context:** the user directed that Android apps are not needed for now and the remaining Linux
platform work is the main goal. `full.md` §1.4 lists Android as a future client; §1.5 defers "full
Android implementation before protocol/test foundations stabilize"; §1.3.6 still requires Android
readiness (reconnects, slow links, client lifecycle, device authorization, event resynchronization)
to be designed in from day one.

**Decision:** no Android app/scaffold/SDK/emulator/Gradle work in the current phase. AND-001 is
deferred and the Android SDK/JDK-pin approval gate is not required. The protocol/model/data design
must remain Android-ready: ADR-001/002 and `docs/architecture/interface-contract.md` are built with a
future Kotlin client in mind, and no Linux-only assumption may leak into `relayable-model` /
`relayable-protocol` (`full.md` §5.2).

**Consequences:** Phase 5/6 (Android) leave the active roadmap; the Linux server + CLI + GPUI client +
terminal + reconnection slice is the priority. `docs/operations/android-toolchain.md` stays as
evidence for whenever Android resumes. No Android dependency changes the default Cargo build.

**Reconsider if:** the user asks to build the Android client, or a protocol change is needed for
mobile.

## DEC-011 — ADR-005 Decision 5 clarification (unique-session default)

**Context:** ADR-005 Decision 5 (Accepted) says a target-requiring command **fails** when the relevant
`RELAYABLE_*` variable is unset and no explicit target is given, and Decision 4 lists the precedence as
flag → env → **no default**. The accepted CORE-001 CLI behavior (`core-001-cli-json.md`,
`crates/relayable-cli/src/resolve.rs`) defaults to the single session when exactly one exists and
otherwise errors (ambiguous → 3, missing → 2). The two documents disagreed.

**Decision:** keep the accepted CORE-001 CLI behavior and read Decision 5 as applying to zero or
**ambiguous** matches. A unique session is not an ambiguous match, so defaulting to it is not "guessing";
the no-context failure still holds when zero or multiple sessions exist. Precedence is
flag → `RELAYABLE_SESSION` → unique-session default. The clarification is recorded as an addendum in
ADR-005; Decisions 1–4 and 6–7 are unchanged.

**Consequences:** no `resolve_session` behavior change and no test churn; the §8.1 context-resolution
test exercises zero-session and two-session no-context failures (CTX-001). `full.md` §8.1's
"never guess silently" is still satisfied, and no REX compatibility claim is made.

**Reconsider if:** `full.md` or the reference docs require strict failure on any unset context even with a
unique match, or a user workflow shows the convenience default causing wrong targeting.

## DEC-012 — STORE-001 SQLite binding (option A: networked fetch once, then offline)

**Context:** ADR-004 Decision 2 chose SQLite for the metadata/audit store and deferred the concrete Rust
binding to crate creation. The root workspace builds `--offline`; no SQLite crate was in the local cargo
cache (verified 2026-10-16). The host has system `libsqlite3` 3.53.4 + headers, but the offline posture
meant the store could not be built without either a network fetch or hand-written FFI.

**Decision (user-approved 2026-10-16, option A):** adopt `rusqlite = "=0.40.2"` with the `bundled`
feature (`libsqlite3-sys 0.38.2`, SQLite compiled from the crate's amalgamation, so no runtime system
SQLite dependency). Perform a **one-time networked fetch** to populate the cargo cache; after that, all
builds stay `--offline`. The user explicitly approved this network access. The cache was warmed on
2026-10-16 (`rusqlite`, `libsqlite3-sys`, `hashlink`, `fallible-iterator`, `fallible-streaming-iterator`).

**Consequences:** the repo's "no network dependency" posture changes exactly once; the dependency graph
and `Cargo.lock` grow; the store crate and its consumers build offline thereafter. `bundled` keeps
Relayable self-contained and avoids depending on a system SQLite version. No terminal bytes are stored
(ADR-004 D2.3 unchanged).

**Reconsider if:** the added dependency breaks offline reproducibility on a fresh machine (then vendor it),
or ADR-004 reconsider-if (4) fires.

## DEC-013 — Relayable is independent; no Superlogical attribution

**Context:** the product spec (`full.md` §0.1) and several docs attributed Relayable to **Superlogical**
("Company: Superlogical", "Relayable (by Superlogical)"). Superlogical is the maker of the **reference
product** (REX, a terminal multiplexer) that informed the UI research; Relayable is a separate product and
must not read as a copy or a Superlogical product.

**Decision (user direction 2026-10-16):** remove every attribution of *Relayable* to Superlogical.
`full.md` no longer declares a company; it states Relayable is an independent project and keeps
Superlogical only as the reference product's vendor in research/compatibility evidence. `AGENTS.md`,
`README.md`, and the ADR `Deciders` rows were updated the same way. The naming rule still forbids labeling
the product "REX"/"REX Superlogical"/"Superlogical Terminal".

**Consequences:** no user-facing Relayable surface names Superlogical. Reference-evidence mentions
(`docs/research/**`, `docs/compatibility/**`, `apps/linux/README.md` video titles, `full.md` evidence URLs)
remain, because they describe the reference product accurately. No code or behavior changes.

**Reconsider if:** the user establishes an official company attribution for Relayable.

## DEC-014 — GPUI-kit and libghostty research outcome (recommendation, user decision pending)

**Context:** the user asked to plan adopting `gpui-kit` (https://gpui-kit.com/docs) and a
libghostty/`libghostty-rs` terminal engine, plus a richer desktop UX. RESEARCH-001/002 (INT-021
`96b4180`) produced `docs/research/gpui-kit.md` and `docs/research/libghostty.md` from live 2026-10-16
fetch + local artifacts.

**Findings (recommendations, not yet a Manager decision):**

- **gpui-kit — Reject for the current pin / Defer any adoption.** It cannot be used alongside
  `gpui = "=0.2.2"`: no version depends on package `gpui`; it exact-pins the `gpui-pre` facade and both
  export extern crate `gpui`, so the GPUI types are incompatible. The umbrella `gpui-kit` crate ships no
  license file at 0.7.3; adopting it pulls TLS/reqwest by default and a styled shadcn-flavoured layer that
  risks the generic redesign `full.md` §0.2/§2.4 forbid; it provides no terminal widget. The unstyled
  `gpui-base` is the only fidelity-safe slice.
- **libghostty / libghostty-rs — Defer / Partial.** Upstream is MIT and headless by design, with valuable
  input-side encoders; but the host's released libghostty (v1.3.1) has **no terminal/grid/scrollback API**
  (that exists only on untagged, self-declared-unstable `main`), **Zig 0.16.x is a build blocker** under the
  no-global-install rule, the leading Rust binding ships no license file, `alacritty_terminal 0.26.0` already
  has committed spike evidence (32 PASS/1 UNSUPPORTED/0 FAIL), and Android is unverified. If ever pursued,
  do it **additively** (input encoders) rather than as a replacement.

**Disposition:** keep ADR-003 accepted and its pins unchanged; do not adopt either dependency in this phase.
The desktop-UX goals (auto-session on open, keymap helper, theming) do **not** require gpui-kit and can be
implemented inside `apps/linux` against the existing `gpui = "=0.2.2"` pin and the design tokens. This is a
**Recommendation pending the user's explicit decision** (they may override with a new directive).

**Reconsider if:** the user decides to re-open the GPUI artifact / fund a Zig toolchain pin (ADR-003
reconsideration triggers), or libghostty ships its terminal API in a tagged, stable, licensed release.

## DEC-015 — Agent runtime migrated from herdr + Pi to OpenCode

**Context:** the loop was built on Pi (agent harness, `pi-herdsman` extension) hosted in herdr
(terminal workspace manager). That stack's costs are recorded in DEC-007/DEC-009: assignment records,
`delegate_project` glue, an `agent_pane_busy` provisioning fault caused by a plugin's shell hook, a
wake-daemon fallback, and Pi-only context retirement. The user asked whether the loop should use
OpenCode instead.

**Decision:**

- OpenCode (2.0.30) is the agent runtime and orchestrator for this repository.
  `opencode.json` sets `default_agent: orchestrator`, `compaction.auto: false` (replacing
  `.pi/settings.json` → `compaction.enabled: false`, preserving DEC-008), and a worktree directory.
- Five roles live in `.opencode/agents/`: `orchestrator` (primary) plus subagents `planner`,
  `researcher`, `implementer`, `verifier`. Role mapping: Manager → orchestrator, worker → implementer,
  independent review (REVIEW-001 pattern) → verifier, Wave 1 research streams → researcher.
- The former middle tier (Lead) is **removed**: OpenCode's subagent nesting depth is 1, so delegation
  is one level deep. The orchestrator spawns each role directly; peer sessions are started by the user.
- Resumption is file-based, as before: `scripts/agent-loop.sh` runs the loop as a sequence of **fresh**
  orchestrator sessions (no `--continue`), reading `docs/agent-loop/NEXT.md`.
- `.pi/settings.json` was removed from the repository.
- **herdr is not a dependency of this repository** and no repo file requires it. Host-level herdr
  configuration was left untouched. herdr is a terminal-multiplexing layer, not an agent runtime, and
  the two compose: `herdr integration install opencode` is available if simultaneous on-screen agent
  panes or SSH-attached terminal sessions are wanted later. Recorded as optional, reversible, and
  outside the repository.

**Verified facts (2026-10-16):**

- `herdr integration status` lists `opencode` as a supported integration target
  (`~/.config/opencode/plugins/herdr-agent-state.js`) — evidence that herdr supervises agents it hosts
  rather than orchestrating them itself.
- Installed `herdr` is **0.9.1** (`pacman -Qi herdr`), not the 0.9.5 recorded in `AGENTS.md`; the
  environment table was corrected.
- The published config schema (`https://opencode.ai/config.json`) defines the encoded agent form with
  `permissions` (an ordered `{action, resource, effect}` ruleset) and `additionalProperties: false`, so
  the deprecated `permission` key would be rejected. `subagent_depth` defaults to 1 and is marked
  deprecated, which is why the flat single-level delegation above is a constraint rather than a
  tunable.
- `opencode debug agents` lists all five roles with the intended modes and merged permission rules;
  `opencode debug config` confirms the project config is discovered and merged over the global one.
- No `herdr`/`pi` references exist in `scripts/*.sh`; the tracked Pi config file was the only
  repository-side dependency.

**Consequences:** Pi/Herdr-specific history in `DECISIONS.md`, `WORKLOG.md`, `TESTS.md`, and the
captured logs under `apps/linux/evidence/**` is **frozen evidence and is not rewritten**. New records
use OpenCode vocabulary. Requirements previously met by the herdr+Pi stack (worktrees, bounded
delegation, completion notification without polling, persistent sessions) now come from OpenCode
features; the capability that has no direct equivalent is a human-facing multi-pane terminal view.

**Reconsider if:** concurrent on-screen agent panes or SSH-attached terminal sessions become a real
workflow need (add the host-level herdr integration), or OpenCode makes nesting depth >1 non-deprecated
and a genuine three-tier org chart is required.

## DEC-016 — Loop model pinned to `opencode-go/deepseek-v4.1-flash` (user directive)

**Context:** the first unattended `scripts/agent-loop.sh` run created fresh sessions with no model
configured anywhere, so they fell back to the ambient default, which resolved to a **free** model
(`ll-alpha-free`). The user requires every loop session to use the paid DeepSeek V4.1 Flash served by
the `opencode-go` provider. Two providers serve an identically named model, so the provider is part of
the decision, not an implementation detail.

**Decision:** pin the model once, at project level — `opencode.json` → `"model":
"opencode-go/deepseek-v4.1-flash"`. Subagents inherit the session model when they configure no `model`
of their own (none do), so one field covers the orchestrator and all four roles.

**Verified facts (2026-10-16):**

- `opencode models` offers three same-named DeepSeek V4.1 Flash candidates:
  `opencode-go/deepseek-v4.1-flash` ($0.15/$0.60 per Mtok — the user's choice),
  `opencode/deepseek-v4.1-flash` ($0.30/$1.20), and the free `agentrouter/deepseek-v4-flash` ($0/$0).
  The display label (`deepseek-v4.1-flash`) does not identify the provider, so verification must read
  session data rather than the label.
- A probe session created with `--agent orchestrator` printed `orchestrator · deepseek-v4.1-flash`, and
  its export reports `"providerID": "opencode-go"` — the intended provider, not the same-named
  alternative.

**Consequences:** unattended work bills at the `opencode-go` rate. Changing models is a one-field edit.
The model exposes `low`/`high`/`max` variants; none is pinned, so the provider default applies — pin a
variant explicitly if reasoning depth or cost must be fixed.

**Reconsider if:** the user names a different model or variant, or one task should run on a cheaper model.

## DEC-017 — UI-003 sequencing: close the `relayable-client` resize parity gap first

**Context:** scoping UI-003 (move the `apps/linux` terminal data plane onto `relayable_client::Client`)
found two places where `Client` cannot express what `apps/linux` does today, so a straight migration
would be a silent functional regression:

- `Client::resize` hardcodes `pixel_width: None, pixel_height: None`
  (`crates/relayable-client/src/client/terminal.rs:142`), while the server applies pixel dimensions to
  the PTY window size (`crates/relayable-server/src/dispatch.rs`, `TerminalSize { .., pixel_width,
  pixel_height }`). The PTY's `ws_xpixel`/`ws_ypixel` would become 0.
- `Client::resize` returns `()` and discards the server's `TerminalResizeAccepted`, so the app could no
  longer report a server-confirmed resize (only a request echo), which the honesty rules (`full.md`
  §0.3.7) make unacceptable to paper over.

Two further parity gaps are accepted and do **not** need a client change: `ApiEvent::Output.stream_seq`
has no consumer in the tree (drop it), and `attach_with_takeover` needs a `SessionId` the app can pass
additively from `SessionCreated`.

**Decision:** add **CLIENT-API-002** — an additive `resize_with_pixels(stream_id, cols, rows,
Option<u16>, Option<u16>) -> Result<TerminalResizeAccepted, ClientError>` plus a `resize` that keeps its
current signature and delegates with `None` pixels — and run it **before** UI-003. UI-003's dependency
list therefore gains CLIENT-API-002.

**Rejected alternatives:** (a) accept the two regressions and document them — rejected because both are
avoidable for a small additive change and one of them would have required presenting request-derived
dimensions as a server echo; (b) fold the client change into UI-003 — rejected because UI-003's owned
paths are `apps/linux/**` and one writer per path must hold (`AGENTS.md`).

**Reconsider if:** UI-003 is dropped, or `resize`'s response is shown to be unnecessary for a
server-confirmed resize.

## DEC-018 — Offline cargo cache drift and toolchain bump on this host (2026-10-16)

**Context:** mid-session, `bash scripts/ci.sh` could not reach `OVERALL: PASS` on this host: the
`apps/linux` workspace (725 packages, incl. `gpui 0.2.2`, `alacritty_terminal 0.26.0`) and
`spikes/terminal-engine-phase2` (`avt`, …) were missing from `~/.cargo/registry`, so `cargo build
--locked` failed with `no matching package named 'alacritty_terminal' found` — offline, with no tree
change responsible (a pristine export of `afcc631` failed identically). `rustc`/`cargo` also report
**1.99.1**, while `AGENTS.md`/`STATUS.md` documented 1.98.1; there is no `rust-toolchain.toml` pin.

**What happened:** the desktop dependency set was repopulated by an **explicit networked `cargo fetch
--locked`** run inside the `ui-003` worktree at ~22:49 by that task's implementer, because the shared
cache had been pruned and the workspace could not resolve `gpui`/`alacritty_terminal` offline. It was
reported transparently by that implementer and it preceded the `AGENTS.md` guidance added in this
decision; no `cargo update` was ever run and `apps/linux/Cargo.lock` is unchanged (verified).
`CARGO_NET_OFFLINE=true cargo build --locked` in `apps/linux` now succeeds (exit 0, ~60 s). The
`spikes/terminal-engine-phase2` set (`avt`, …) is still absent, so that one `ci.sh` row cannot pass
offline.

**Decision (this session):** do **not** run a further networked fetch at agent initiative. `apps/linux`
builds offline again, so UI-003/UI-004 are unblocked; the phase-2 spike row is recorded as a known
environment gap and stays red until the user either approves a one-time networked `cargo fetch` for
that spike workspace (the DEC-012 pattern) or accepts the documented red row. The `file-sizes` row added
by REF-010 is PASS and its failure path was demonstrated independently of this gap.

**Verified facts:** `apps/linux` offline build exit 0; `spikes/terminal-engine-phase2` offline build
exit 101 (`no matching package named 'avt' found`); committed lockfiles unchanged; `validate.sh` and
`smoke.sh` green on the root workspace both before and after (root cache was never incomplete).

**Reconsider if:** the user approves the fetch (then re-verify `ci.sh` → `OVERALL: PASS`), or the
phase-2 spike is retired as superseded TERM-002 evidence.

## DEC-019 — UI-004 staging and the "which session to restore" policy (2026-10-16)

**Context:** `apps/linux` already auto-creates a session on the `Connecting → Connected` edge, but
three defects make "no manual `new`" untrue: closing the last tab never re-creates a session; a failed
`session.create` leaves `pending_create` set forever (auto-create *and* `Ctrl+Shift+N` dead); and a
retryable connect failure is painted as a terminal `ConnectionState::Error`. Separately, attaching to an
*existing* session is impossible with today's app-side API: the wire `SessionSummary` carries neither a
block id nor `created_at`, so a restore policy cannot be derived from `session.list`.

**Decision:** split UI-004.

- **Stage 1 (delivered, integrated INT-026 `385f46f` — see the amendment below):** fix the auto-session
  defects inside `apps/linux/src/app/**` only.
- **Stage 2 (deferred, needs the user):** attaching to an existing session requires `api/**` changes
  (surface `session.view` / a new command) *and* a policy for which session to restore (last-used id
  persisted locally is Relayable-original, not reference-derived). Tracked as `UI-004-STAGE2` (a)
  together with two technical items that also need `api/**`: (b) a retryable connect failure is still
  painted as a terminal `ConnectionState::Error`, and (c) an unrelated `ApiEvent::Error` clears an
  in-flight create intent, downgrading a split/labelled create to a new tab (verifier nit N1).

**Amendment (2026-10-16, after delivery):** stage 1 as delivered (INT-026 `385f46f`) fixed defects 1 and 2
**plus** a second wedge the implementer found (a create in flight when the transport dies was also void,
because commands are not replayed across a reconnect) — it does **not** fix defect 3, which needs the
error-cause classification that lives in `api/**`; an `app/**`-only version would have required forbidden
string matching. Stage 1's owned paths as delivered: `app/{events,tabs,panes,helpers,tests}.rs` +
`apps/linux/evidence/ui-004-*`; `api/**` was not touched, so it did not collide with UI-003.

**Consequence to state honestly:** after stage 1 the app still creates a *new* session on every launch
instead of restoring the previous one — a deviation from the only behavioural evidence (V1 0:52–1:30,
V4 4:11) that stage 2 closes. All four empty/loading/error/disconnected surfaces are
**Relayable-original**: the catalogue records them as `not observed` in all five videos, so they must be
labelled as such and must not be presented as reference-derived.

**Reconsider if:** the user wants restore-last-session in this phase (then stage 2 becomes the priority
and needs a local persistence decision), or the picker/attach work is dropped entirely.

## Open questions requiring evidence (not yet decidable)

- Terminal engine: which maintained Rust-compatible engine meets §5.4 coverage/licensing/API-stability bar.
- Protocol: protobuf vs alternatives, and TLS vs QUIC vs WebSocket-over-TLS for remote (§6.1, §6.2).
- Env namespace: **RESOLVED 2026-10-15** — `RELAYABLE_*`-only, no `REX_*` aliases (DEC-006 → ADR-005, integrated `74e7e20`).
- **Android SDK/emulator install — DEFERRED (DEC-010):** ENV-001 found no SDK/emulator/images/licenses
  and no project JDK pin (JDK 25 only; AGP-on-JDK-25 unverified). Android work is out of current
  scope, so no install/approval is needed now. Evidence: `docs/operations/android-toolchain.md`.
- **GPUI artifact/license pin:** crates.io `gpui` 0.2.2 (Apache-2.0) vs `Glass-HQ/gpui` (no root
  LICENSE, manifests declare Apache-2.0) — decide in ADR-003 before pinning.
- Whether any REX wire protocol is documented enough to attempt interoperability (currently assumed **not**).
- **DEC-009 — RESOLVED 2026-10-15:** hook sourcing disabled **and** the `herdr-automatic-rename`
  plugin uninstalled (residual dirs cleared); delegation verified working. Details above and in
  `WORKLOG.md` Checkpoints 7 and 10.
