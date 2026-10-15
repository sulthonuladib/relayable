# DECISIONS — index

Detailed ADRs live under `docs/adr/`. Each ADR contains: context, options, decision, consequences, and what evidence would cause reconsideration (`full.md` §5.2).

| ID | Decision | Status | ADR |
|---|---|---|---|
| DEC-000 | Work state is tracked in-repository (`docs/agent-loop/`), not in chat | **Accepted** | — |
| DEC-001 | Repo is Phase 0 spec-only; establish minimal workspace rather than migrating existing code | **Accepted** | — |
| DEC-002 | Domain model and dependency direction | **Accepted** — ADR-001 integrated `31bed30`; REVIEW-001 corrections applied via ADR-FIX-001 (INT-007 `f474c83`) | `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md` |
| DEC-003 | Protocol serialization + transport | **Accepted** — ADR-002 integrated `31bed30`; REVIEW-001 B-1/B-2/B-3 corrections applied via ADR-FIX-001 (INT-007 `f474c83`) | `docs/adr/ADR-002-protocol-schema-and-transport.md` |
| DEC-004 | Terminal engine selection | **Proposed** — ADR-003 integrated `78b83f2`; TERM-002 (phase 2) validates the engine pins `alacritty_terminal 0.26.0`/`vte 0.15.0`/`portable-pty 0.9.0` (32 PASS/0 FAIL); the `gpui = "=0.2.2"` pin does not compile (xattr/libc `ENOATTR`) and awaits a UI-001 remedy — ADR-003 stays `Proposed` | `docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md` |
| DEC-005 | Local/remote process ownership and persistence guarantees | Pending | `docs/adr/ADR-004` |
| DEC-006 | Env var namespace: `RELAYABLE_*`, REX aliases, or both | **Accepted** — `RELAYABLE_*`-only, no `REX_*` aliases; `ADR-005` integrated `74e7e20` | `docs/adr/ADR-005-environment-variable-namespace.md` |
| DEC-007 | Wave 1 research split into three branch-isolated, documentation-only projects with exclusive path ownership; Manager integrates before Wave 2 | **Accepted** | — |
| DEC-008 | Context-window policy: no automatic/manual compaction; repository files are authoritative; fresh bounded agent sessions; checkpoint-and-handoff at context pressure | **Accepted** | — |
| DEC-009 | Herdr agent-provisioning fault remediation (stop sourcing the automatic-rename fish hook vs project-local lead tool overlay) | **Accepted — hook disabled + plugin uninstalled 2026-10-15** | — |
| DEC-010 | Android client deferred from current scope; keep the model/protocol Android-ready | **Accepted** — user direction 2026-10-15; consistent with `full.md` §1.4/§1.5 | — |

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
