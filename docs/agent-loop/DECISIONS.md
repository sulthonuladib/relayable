# DECISIONS — index

Detailed ADRs live under `docs/adr/`. Each ADR contains: context, options, decision, consequences, and what evidence would cause reconsideration (`full.md` §5.2).

| ID | Decision | Status | ADR |
|---|---|---|---|
| DEC-000 | Work state is tracked in-repository (`docs/agent-loop/`), not in chat | **Accepted** | — |
| DEC-001 | Repo is Phase 0 spec-only; establish minimal workspace rather than migrating existing code | **Accepted** | — |
| DEC-002 | Domain model and dependency direction | Pending | `docs/adr/ADR-001` |
| DEC-003 | Protocol serialization + transport | Pending — depends on DOC-001 | `docs/adr/ADR-002` |
| DEC-004 | Terminal engine selection | Pending — spike required | `docs/adr/ADR-003` |
| DEC-005 | Local/remote process ownership and persistence guarantees | Pending | `docs/adr/ADR-004` |
| DEC-006 | Env var namespace: `RELAYABLE_*`, REX aliases, or both | Pending — ADR + tests required (§8.1) | `docs/adr/ADR-005` |
| DEC-007 | Wave 1 research split into three branch-isolated, documentation-only projects with exclusive path ownership; Manager integrates before Wave 2 | **Accepted** | — |
| DEC-008 | Context-window policy: no automatic/manual compaction; repository files are authoritative; fresh bounded agent sessions; checkpoint-and-handoff at context pressure | **Accepted** | — |
| DEC-009 | Herdr agent-provisioning fault remediation (stop sourcing the automatic-rename fish hook vs project-local lead tool overlay) | Pending — user approval required | — |

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

## Open questions requiring evidence (not yet decidable)

- Terminal engine: which maintained Rust-compatible engine meets §5.4 coverage/licensing/API-stability bar.
- Protocol: protobuf vs alternatives, and TLS vs QUIC vs WebSocket-over-TLS for remote (§6.1, §6.2).
- Env namespace: `RELAYABLE_*` vs REX-compatible aliases vs both (§8.1).
- **Android SDK/emulator install (approval gate):** ENV-001 found no SDK/emulator/images/licenses
  and no project JDK pin (JDK 25 only; AGP-on-JDK-25 unverified); install + license acceptance need
  explicit user approval. Evidence: `docs/operations/android-toolchain.md`.
- **GPUI artifact/license pin:** crates.io `gpui` 0.2.2 (Apache-2.0) vs `Glass-HQ/gpui` (no root
  LICENSE, manifests declare Apache-2.0) — decide in ADR-003 before pinning.
- Whether any REX wire protocol is documented enough to attempt interoperability (currently assumed **not**).
- **DEC-009 (approval gate):** which remediation for the Herdr agent-provisioning fault — A) stop
  sourcing the automatic-rename fish hook in `~/.config/fish/config.fish` (recommended; cosmetic
  loss of live tab renaming), B) project-local `managed-lead` overlay granting Leads bash/edit/write
  so they execute directly (repo-only, changes the delegation model), or C) update/reinstall the
  plugin. Diagnosis and A/B evidence: `WORKLOG.md` Checkpoint 3, `TESTS.md` rows 15–23.
