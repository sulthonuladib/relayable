# TASKS — Relayable task ledger

Statuses: `BACKLOG` · `READY` · `IN_PROGRESS` · `BLOCKED` · `REVIEW` · `DONE`.
Every task needs: ID, dependencies, owner, acceptance criteria, test/validation command, evidence path.

**Writer rule:** one owner per branch and per path. The Manager owns `docs/agent-loop/**` and
integration on `master`; parallel project branches must not edit it. Wave 1 branches are
documentation-only: no application code, manifests, or toolchain changes.

## Wave 1 — research (running in parallel, disjoint file ownership)

| Branch (owner) | Tasks | Exclusive owned paths | Not permitted |
|---|---|---|---|
| `research/ui-reference` | REF-001, REF-002, REF-003 | `docs/research/**` | app/protocol code, `docs/compatibility/**`, `docs/operations/**`, `docs/agent-loop/**`, `AGENTS.md`, `full.md` |
| `research/protocol-compat` | DOC-001 | `docs/compatibility/**` | app/protocol code, `docs/research/**`, `docs/operations/**`, `docs/agent-loop/**`, `AGENTS.md`, `full.md` |
| `research/env-audit` | ENV-001 (plus the Android inventory portion of AND-001) | `docs/operations/**` | app/protocol code, other docs trees, `docs/agent-loop/**`, any install/upgrade/system change |

Wave 1 produces evidence only. When a Wave 1 branch completes, the Manager reviews its diff and
integrates it into `master` (INT-001). Wave 2 (ADRs, terminal spike, CORE-001) starts only after
the evidence is integrated.

## Ledger

| ID | Task | Depends on | Owner | Status |
|---|---|---|---|---|
| BOOT-001 | Inspect repo, toolchain, instructions, working tree; preserve user changes | — | Lead | DONE |
| BOOT-002 | Create `AGENTS.md` + `docs/agent-loop/` scaffold | BOOT-001 | Lead | DONE |
| REF-001 | Review video 1 (pre-alpha demo); catalogue main UI | BOOT-001 | research/ui-reference | DONE (integrated c0578ca) |
| REF-002 | Review videos 2–5; timestamped evidence each | BOOT-001 | research/ui-reference | DONE (integrated c0578ca) |
| REF-003 | Review GoRex clone; behavioral catalogue + license check | BOOT-001 | research/ui-reference | DONE (integrated c0578ca) |
| REF-004 | Fix `design-tokens.md:210` mixed attribution (29 px palette rows) + queued calibration items from `open-questions.md`. **Palette-row attribution fixed 2026-10-15** (the "29 px palette rows" entry in the GoRex-not-used list corrected to GoRex's 34 px, with the 29→37.5 px own-estimate correction noted); remaining calibration items from `open-questions.md` still open | REF-003 | TBD | BACKLOG |
| REF-005 | Code-organization standard (`docs/architecture/code-organization.md`) + `scripts/check-file-sizes.sh` (report/strict) | — | Manager | DONE — `2026-10-16`; caps calibrated to 500/1000 prod, 800/1200 tests, 150/300 entry |
| REF-006 | Split over-cap files on free paths: `apps/linux/src/{app,tokens,api/mod,term/mod,theme,geometry}.rs`, `crates/relayable-{model,pty,cli}/src/lib.rs`, `crates/relayable-protocol/src/**` | REF-005 | `refactor/ref-006-free-crates` (Herdr `w1M`) | DONE — integrated INT-020 `ba92c51`; 60 files, 0 public-path changes; 194 tests pass |
| REF-007 | Split `crates/relayable-client/src/**` (~2141-line `lib.rs`) into cohesive modules with a stable public API | REF-005, CLIENT-API-001 | `refactor/ref-007-009-client-tests` (Herdr `w1N`) | DONE — integrated INT-019 `54fe220`; `lib.rs` 2139→78 + 10 modules; public surface identical |
| REF-008 | Split `crates/relayable-server/src/**` + `crates/relayable-server/tests/**` (`session.rs` 1037, `server_e2e.rs` 1472) | REF-005, STORE-001 | `refactor/ref-008-server-split` (implementer; worktree `/home/sulthonuladib/work/relayable-wt/ref-008`) | IN_PROGRESS — last hard-cap blockers before REF-010 |
| REF-009 | Split `tests/**` integration suites (`tests/events/main.rs` 1058, `tests/fixtures/mod.rs` 592) | REF-005, CLIENT-API-001 | `refactor/ref-007-009-client-tests` (Herdr `w1N`) | DONE — integrated INT-019 `54fe220`; `events/main.rs` 1058→28 + 7 modules, `fixtures/mod.rs` 592→27 + 5 modules |
| REF-010 | Wire `scripts/check-file-sizes.sh --strict` into `scripts/ci.sh` + `scripts/validate.sh`; verify clean | REF-006..REF-009 | TBD | BACKLOG |
| RESEARCH-001 | gpui-kit evaluation (license, `gpui =0.2.2` compat, components, fidelity impact) → `docs/research/gpui-kit.md` | — | `research/gpui-kit-libghostty` (Herdr `w1P`) | DONE — integrated INT-021 `96b4180`; verdict **Reject (current pin) / Defer (any adoption)** |
| RESEARCH-002 | libghostty / `libghostty-rs` evaluation (license, build, API, Android, vs ADR-003 pins) → `docs/research/libghostty.md` | — | `research/gpui-kit-libghostty` (Herdr `w1P`) | DONE — integrated INT-021 `96b4180`; verdict **Defer / Partial** |
| TERM-003 | libghostty-rs engine spike + ADR-003 amendment (reconsideration trigger #10); adopt/decline decision | RESEARCH-002 | TBD | PROPOSED — research recommends DEFER (unreleased API, Zig build blocker); **user decision** |
| UI-004 | Desktop: auto-create/attach a session on app open; explicit empty/loading/error/disconnected states | RESEARCH-001 | TBD | BACKLOG |
| UI-005 | Keymapping helper: discoverable shortcut overlay + reference (Relayable-original Linux bindings) | UI-004 | TBD | BACKLOG |
| UI-006 | Theming: token-driven theme through the chosen component layer; no fidelity divergence | RESEARCH-001 | TBD | BACKLOG |
| DOCS-001 | Centralize docs behind `docs/README.md`; consolidate duplicate/stale docs (evidence preserved) | RESEARCH-001/002 decisions | TBD | BACKLOG |
| QA-FLAKE-001 | Pre-existing `server_e2e` flake (~1% under heavy parallel load: `resume_shapes_and_writer_takeover`, `idle_timeout_reason_is_reported_to_a_subscriber`); reproduced on `master` pre-STORE. Root cause is test-side readiness/record-removal ordering | — | TBD | BACKLOG |
| QA-HARNESS-001 | `tests/fixtures/binary.rs` freshness guard scans `crates/*/src/**` including `#[cfg(test)]` files, so a test-only edit doesn't relink `target/debug/relayable` and can trip the guard (exit 101). Tighten the heuristic to production-relevant sources only | — | `qa/qa-harness-001-freshness-guard` (implementer; worktree `/home/sulthonuladib/work/relayable-wt/qa-harness-001`) | IN_PROGRESS |
| DOC-001 | Verify official CLI/API/Lua/events/status docs → behavior matrix | BOOT-001 | research/protocol-compat | DONE (integrated 49795a0) |
| DOC-002 | Fix DOC-001 citation defects and re-check the 7 source URLs for live drift | DOC-001 | `docs/doc-002-matrix-repair` | DONE — integrated `16c8bc9` (INT-005); 193 MATCH kept, 115 SHIFT renumbered, 78 NOT_FOUND → heading anchors, 0 drift; labels unchanged |
| ENV-001 | Rust/GPUI/systemd/Android CLI/SDK/emulator/JDK audit; no installs | BOOT-001 | research/env-audit | DONE (integrated b24133c) |
| INT-001 | Review Wave 1 diffs; integrate research branches into `master`; resolve conflicts | REF-001, REF-002, REF-003, DOC-001, ENV-001 | Manager | DONE (env b24133c, ref c0578ca, doc 49795a0) |
| INT-002 | Review and integrate Wave 2 branches into `master` | DOC-002, ADR-001, ADR-002, ADR-003, TERM-001 | Manager | DONE for `term-001` `78b83f2` + `adr-001-002` `31bed30` |
| INT-003 | Integrate `research/doc-002` evidence onto `master`; retire the branch/worktree for a clean start | DOC-002 | Manager | DONE — `a1f1719` (evidence on `master`; repair still pending) |
| ADR-001 | Domain model and dependency boundaries | DOC-001 | research/adr-001-002 | DONE — integrated `31bed30`, **Accepted** after REVIEW-001/ADR-FIX-001 |
| ADR-002 | Protocol schema + transport alternatives | DOC-001, ADR-001 | research/adr-001-002 | DONE — integrated `31bed30`, **Accepted** after REVIEW-001/ADR-FIX-001 |
| ADR-003 | Terminal engine selection (spike-backed) | ENV-001, TERM-001 | research/term-001-adr-003 | DONE (Proposed) — integrated `78b83f2`; engine-level phase 2 pending as TERM-002 |
| ADR-004 | Local/remote process ownership; persistence guarantees | ADR-001 | `adr/adr-004-ownership-persistence` | DONE — integrated `9a3fb27` (INT-014); **Accepted** (DEC-005); crash/reboot survival NOT claimed |
| STORE-001 | Create `crates/relayable-store` (SQLite metadata/audit) + server start-up reconciliation, `flock`, prune, write path per ADR-004 | ADR-004 | `store/store-001-sqlite-metadata` (Herdr `w1J`) | IN_PROGRESS — `rusqlite =0.40.2` bundled (DEC-012); bounded first slice |
| ADR-005 | Environment-variable namespace (`RELAYABLE_*` vs `REX_*` aliases vs both) | ADR-002 D14 | `adr/adr-005-env-namespace` | DONE — integrated `74e7e20` (INT-006); `RELAYABLE_*`-only; DEC-006 Accepted |
| UI-002 | Main reference workspace UI (GPUI) against the catalogue + tokens, using the live API | UI-001, CORE-002 | `ui/ui-002-reference-workspace` | DONE — integrated `d46c66c` (INT-012); 6 catalogue screens; 58 app tests; live probe + committed own screenshots; `relayable-client` gap → `CLIENT-API-001` |
| UI-001 | GPUI minimal Linux window build/run proof + `gpui` remedy | ENV-001, ADR-003, TERM-002 | `ui/ui-001-gpui-window` | DONE — integrated `09928db` (INT-010); `gpui =0.2.2` builds+runs via `libc 0.2.189` lockfile hold; real Wayland window `class=relayable`; 9 token tests; screenshot committed |
| TERM-001 | Terminal engine candidate prototype (TUI/input/resize spike) | ENV-001 | research/term-001-adr-003 | DONE phase 1 — integrated `78b83f2` (PTY/SIGWINCH/byte-stream/RSS harness) |
| TERM-002 | Engine-level spike phase 2: per-engine adapters validate ADR-003's provisional pin | TERM-001 | `spike/term-002-engine-validation` | DONE — integrated `60d4395` (INT-009); engine pins accept (32 PASS/1 UNSUPPORTED/0 FAIL); `gpui =0.2.2` does not compile (xattr/libc `ENOATTR`) → UI-001 remedy |
| REVIEW-001 | Independent reviewer pass over ADR-001/002 (`69e5600`) and ADR-003/spike (`2e41da4`) before CORE-001 builds on them — no independent review was possible during wind-down | ADR-001, ADR-002, ADR-003 | `review/review-001-adr-pass` | DONE — integrated `774d394` (INT-004); report `docs/reviews/REVIEW-001-adr-review.md` + 4 evidence files; verdicts ADR-001/002 accept-with-nits, ADR-003 keep `Proposed` |
| AND-001 | Android project scaffold checks + `scripts/android-check.sh` (toolchain inventory delivered by ENV-001) | ENV-001 | Build/Test | DEFERRED (DEC-010) — no Android work in current scope |
| ADR-FIX-001 | Apply REVIEW-001 corrections: ADR-002 Unverified-table refresh (B-1) + coverage-row fix (B-2) + D11 no-reserialize test (B-3) + D7 TLS/provider disclosure (B-4/B-5), `protoc` note (B-6), and ADR-001 nits A-2–A-10. Manager-owned `docs/adr/**` + `docs/architecture/interface-contract.md` | REVIEW-001 | `adr/review-001-fixes` | DONE — integrated `f474c83` (INT-007); 3 files +135/−86; no decision changed |
| CORE-001 | Model + server + CLI vertical slice (server-owned PTY) | ADR-001, ADR-002 | `core/core-001-vertical-slice` | DONE — integrated `d3e0286` (INT-008); 53 files/+12010; `cargo test --workspace` 104 pass; 5 acceptance properties verified independently |
| CTX-001 | Context-resolution test + set `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed terminals (ADR-005 Decision 2 + the §8.1 test) | ADR-005, CORE-001 | `ctx/ctx-001-context-env` (implementer `ctx001`) | DONE — integrated INT-015 `dd1b986`; 4 integration tests + 2 unit tests; `tests/context-resolution/**` |
| INSTALL-001 | User install path: `scripts/install-user.sh`, `systemd --user` unit, desktop entry, root README quickstart, `docs/operations/user-install.md` (release binaries → XDG user dirs; no root) | CORE-001, UI-002 | `install/install-001-user-install` (implementer `install001`) | DONE — integrated INT-016 `39e5d49` (+`a3ff5c0` fixes); sandbox install + live round-trip verified; `--with-desktop` builds/installs `relayable-desktop` |
| CORE-002 | Client disconnect leaves session alive; reattach snapshot + event stream/resume | CORE-001 | `core/core-002-reconnect-events` | DONE — integrated `91662af` (INT-011); 148 tests; events/disconnect/resume-cursor/reattach/multi-client proven; no model/contract change |
| QA-001 | Smoke/CI scripts and build matrix | CORE-001 | `qa/qa-001-smoke-ci` | DONE — integrated `4a258c2` (INT-013); `scripts/smoke.sh` (SMOKE OK), `scripts/ci.sh` (15 PASS/3 SKIP), flow-control test |
| QA-FLOW-001 | Automated slow-client output-overflow / backpressure test (CORE-001 implemented the path but left it untested) | CORE-001 | `qa/qa-001-smoke-ci` | DONE — integrated `4a258c2` (INT-013); bounded ≤ window + one frame flush then `flow_control_overflow` |
| MEM-001 | Measured idle/active memory baseline + regression thresholds (`full.md` §12) | QA-001 | TBD | BACKLOG |
| CI-HOST-001 | Decide whether to host CI (`.github/workflows`) given a hosted runner cannot build this tree without network/vendoring | QA-001 | TBD | BACKLOG |
| CLIENT-API-001 | `relayable-client` raw terminal byte streaming: `wait_for_output` returns `Err(Timeout)` and drops already-acked bytes, so it cannot drive a live terminal (UI-002 worked around it with a wire client on `relayable-protocol`) | CORE-001 | `client/client-api-001-incremental-output` (Herdr `w1H`) | DONE — integrated INT-017 `52433f7`; public `Client::next_output` + `tests/client-output` (524 KB across the credit window); 164 tests |
| UI-003 | Migrate `apps/linux` terminal view off its raw `relayable-protocol` data plane onto `Client::next_output`; update `apps/linux/README.md` §7 (stale `wait_for_output` line numbers, API now exists) | CLIENT-API-001 | TBD | BACKLOG |

Wave 2 ordering: `ENV-001` evidence → `TERM-001` spike + `ADR-003` and `ADR-001`/`ADR-002`
(parallel, disjoint paths) → `CORE-001` → `CORE-002` → `UI-001` against the integrated UI catalogue →
`QA-001`, with DOC-002 fixing the matrix citations alongside. Status at the Wave 2/clean-start handoff: ADR-001/002, TERM-001/ADR-003, and the DOC-002 audit
evidence are all integrated on `master` (`31bed30`, `78b83f2`, `a1f1719`); the DOC-002 matrix repair
still needs one writer pass (evidence at `docs/compatibility/doc-002-audit/`); TERM-002 (engine-level
spike) is open. CORE-001 is the first engineering slice: server-owned PTY, CLI-created session,
terminal I/O, reconnection, with an automated integration test — using the ADR-001/002 interface
contract; the verified reference UI remains the UI acceptance gate.

## Acceptance criteria and validation (per task)

**BOOT-001** — inventory recorded in `WORKLOG.md` Checkpoint 1; user changes preserved.
Evidence: `docs/agent-loop/WORKLOG.md`.

**BOOT-002** — `AGENTS.md` + five `docs/agent-loop/*.md` files exist and are committed.
Evidence: `git log`, file listing.

**REF-001/002/003** — for each of the five videos: URL, access route (viewed / captions only /
metadata only / inaccessible), whether audio and actual frames were inspected, timestamped workflow
timeline, layouts/geometry/typography/colors, observed transitions, verbatim shortcuts/commands when
clearly shown, memory/CPU figures verbatim **with stated workload and environment**, and a
confidence level per claim. GoRex review additionally requires the exact commit hash, license
re-check, confirmed-by-running vs inferred separation, and a feature comparison table.
Owned paths on branch `research/ui-reference`: `docs/research/video-review.md`,
`docs/research/ui-catalog.md`, `docs/research/open-questions.md`, `docs/research/gorex-review.md`
(raw captures under `docs/research/evidence/**`, already gitignored, or outside the repo).
Explicitly permitted tools: `yt-dlp`, `ffmpeg`, `chromium`, `curl`, Node/npm for a scratch GoRex
checkout. Time-box each access strategy; record failures and continue.
**Forbidden:** production code, branding, protocol, inventing benchmark values or flags, copying
GoRex source/assets.

**DOC-001** — `docs/compatibility/reference-behavior-matrix.md` on branch `research/protocol-compat`.
Every row labelled `observed-video` | `verified-official-docs` | `verified-by-test` | `implemented` |
`intentionally-different` | `inferred` | `unknown`. Must cover domain hierarchy, target resolution,
stable IDs, CLI flags, JSON output, exit codes, environment variables, server API method
names/schemas, Lua functions, event names/payloads, block methods, OSC 7501 status protocol
semantics, multi-client behavior, and connection lifecycle. Cite retrieval URL + date and quote
exact names/schemas. Must **not** claim binary/wire compatibility with REX.
**Forbidden:** production code, other docs trees.

**ENV-001** — `docs/operations/toolchain-audit.md` on branch `research/env-audit` (and optionally
`docs/operations/android-toolchain.md`): toolchain matrix with raw command output, exit codes, and
date/host context per row; explicit "gaps blocking implementation" section with safe next steps.
Covers Rust/cargo/mise, GPUI availability & license, systemd user mode, Android CLI/`adb`/SDK/
emulator/system images, JDK/Gradle, display backend, disk/KVM. No installs, upgrades, `sudo`, or
system changes.
Evidence: both files delivered (+1591 lines across 6 commits) with an independent verification /
remediation loop; integrated into `master` in `b24133c`.
**Forbidden:** production code, other docs trees, any mutating host command.

**INT-001** — the three research branches' commits reviewed by the Manager, merged into `master`
with conflicts resolved and no owned content dropped; `git log` on `master` shows the integrated
commits; no application code introduced by the research wave.
Evidence: `docs/agent-loop/WORKLOG.md` Checkpoint 3.

**ADR-001..004** — each ADR records context, options, decision, consequences, and what evidence
would cause reconsideration; stored under `docs/adr/`. No ADR may invent an option that violates
full.md §5.2 dependency direction or §6 transport security requirements.

**CORE-001/CORE-002** — an automated integration test proves: foreground server owns a PTY; CLI
creates a session and returns a stable ID; terminal input/output round-trips; killing/disconnecting
the client leaves the child process alive; reconnecting retrieves current state (snapshot or
documented equivalent). Evidence: test output in `WORKLOG.md` with exact commands.

## Previously recorded acceptance criteria (kept for reference)

**REF task rule (full.md §2.2/§2.3/§2.6):** `docs/research/ui-catalog.md` lists only actually
observed screens/states, each with source video + timestamp, component hierarchy and geometry,
interaction notes, exact readable labels, empty/loading/error/disconnected/focused/selected states,
confidence, unresolved details, and the planned Relayable screenshot-test name. The catalogue must
not be a wish list of desired screens.

**DOC task rule (full.md §3):** official live docs are the source of truth for documented behavior.
Verify current content before implementing; keep `observed-video`, `verified-official-docs`,
`verified-by-test`, `implemented`, `intentionally-different`, `inferred`, and `unknown` strictly
distinct.

**ENV task rule (full.md §0.4, §10.1):** discovery checks are not authorization to change the
system. If a required tool is missing, report exactly what is missing and propose a compatible
`mise` pin only as an option; do not install or modify global configuration.
