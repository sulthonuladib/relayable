# TASKS — Relayable task ledger

Statuses: `BACKLOG` · `READY` · `IN_PROGRESS` · `BLOCKED` · `REVIEW` · `DONE`.
Every task needs: ID, dependencies, owner, acceptance criteria, test/validation command, evidence path.
Seed set derives from `full.md` §17; re-sequence when evidence justifies it.

| ID | Task | Depends on | Owner | Status |
|---|---|---|---|---|
| BOOT-001 | Inspect repo, toolchain, instructions, working tree; preserve user changes | — | Lead | IN_PROGRESS |
| BOOT-002 | Create `AGENTS.md` + `docs/agent-loop/` scaffold | BOOT-001 | Lead | DONE |
| REF-001 | Review video 1 (pre-alpha demo) and catalogue main UI | BOOT-001 | Research/UI | READY |
| REF-002 | Review videos 2–5; timestamped evidence each | BOOT-001 | Research/UI | READY |
| REF-003 | Review GoRex clone; behavioral catalogue + license check | BOOT-001 | Research/UI | READY |
| DOC-001 | Verify official CLI/API/Lua/events/status docs → behavior matrix | BOOT-001 | Architecture/Research | READY |
| ENV-001 | Rust/GPUI/systemd/Android CLI/SDK/emulator/JDK audit; no installs | BOOT-001 | Build/Test | READY |
| ADR-001 | Domain model and dependency boundaries | DOC-001 | Lead | BACKLOG |
| ADR-002 | Protocol schema + transport alternatives | DOC-001 | Lead | BACKLOG |
| ADR-003 | Terminal engine selection (spike-backed) | ENV-001 | Lead | BACKLOG |
| ADR-004 | Local/remote process ownership; persistence guarantees | ADR-001 | Lead | BACKLOG |
| UI-001 | GPUI minimal Linux window build/run proof | ENV-001 | Lead/UI | BACKLOG |
| TERM-001 | Terminal engine candidate prototype (TUI/input/resize spike) | ENV-001 | Lead | BACKLOG |
| AND-001 | Android CLI/JDK/SDK inventory; no global changes | ENV-001 | Build/Test | BACKLOG |
| CORE-001 | Model + server + CLI vertical slice (server-owned PTY) | ADR-001, ADR-002 | Lead | BACKLOG |
| CORE-002 | Client disconnect leaves session alive; reattach snapshot | CORE-001 | Lead | BACKLOG |
| QA-001 | Smoke/CI scripts and build matrix | BOOT-001 | Lead | BACKLOG |

## Acceptance criteria and validation (per task)

**BOOT-001** — inventory recorded in `WORKLOG.md`; `git status` clean before and after; user changes preserved.
Evidence: `docs/agent-loop/WORKLOG.md`.

**BOOT-002** — `AGENTS.md` + five `docs/agent-loop/*.md` files exist and are committed.
Evidence: `git log`, file listing.

**REF-001/002** — for each of the five videos: URL, access route (viewed / captions only / metadata only / inaccessible), whether audio and actual frames were inspected, timestamped workflow timeline, layouts/geometry/typography/colors, observed transitions, verbatim shortcuts/commands when clearly shown, memory/CPU figures verbatim **with stated workload and environment**, and a confidence level per claim.
Owned paths: `docs/research/video-review.md`, `docs/research/ui-catalog.md`, `docs/research/open-questions.md`.
Explicitly permitted: `yt-dlp`, `ffmpeg`, `chromium` are installed. Time-box each access strategy; record failures and continue.
**Forbidden:** production code, branding, protocol, inventing benchmark values or flags.

**REF-003** — `docs/research/gorex-review.md`: repo URL + **commit hash** reviewed, per-state UI observations, component hierarchy/measurements/typography/colors, shortcuts, focus, resize, empty/error states; explicit "confirmed by running" vs "README/code-inferred" separation; feature comparison table (videos/docs vs GoRex vs Relayable); **license re-verification**; and the limitation that GoRex does not replicate remote server connections and uses a local socket.
**Forbidden:** copying, vendoring, translating, or porting GoRex source or non-system assets unless a verified license expressly permits it.

**DOC-001** — `docs/compatibility/reference-behavior-matrix.md`. Every row labelled `observed-video` | `verified-official-docs` | `verified-by-test` | `implemented` | `intentionally-different` | `inferred` | `unknown`. Must cover domain hierarchy, target resolution, stable IDs, CLI flags, JSON output, exit codes, env vars, server API methods/schemas, Lua functions, event names/payloads, block methods, status protocol semantics, multi-client behavior, connection lifecycle.
Must **not** claim binary/wire compatibility with REX.

**ENV-001** — toolchain matrix with raw command output per row; explicit "gaps blocking implementation" section. No installs, upgrades, or system changes.
Owned paths: `docs/operations/**`.
