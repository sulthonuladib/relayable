# NEXT — handoff for the next session

Read `full.md` completely, then `AGENTS.md`, `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `DECISIONS.md`,
`TESTS.md`, `WORKLOG.md` (tail), and this file. Trust repo state over chat.

## First commands

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8
git worktree list && git branch -a
```

## Current state (Wave 2 kickoff)

`master` @ `b96da27` (pushed) contains all Wave 1 research. Wave 2 is in flight on three branches,
all based on `master` @ `b96da27`:

| Workspace | Branch | Session | Tasks | Owned paths |
|---|---|---|---|---|
| `w12` | `research/doc-002` | `3534d553-bec1-455c-83df-807ee3ca724a` | DOC-002 | `docs/compatibility/reference-behavior-matrix.md` |
| `w13` | `research/term-001-adr-003` | `8f442f6e-6f8c-4a03-9016-dcf3f5827aa1` | TERM-001, ADR-003 | `docs/adr/ADR-003-*.md`, `spikes/terminal-engine/**` |
| `w14` | `research/adr-001-002` | `6a4c1607-f4d3-42fb-b1fc-d6e2dce6e7ba` | ADR-001, ADR-002 | `docs/adr/ADR-001-*.md`, `docs/adr/ADR-002-*.md`, `docs/architecture/**` |

Manager (this session) owns `docs/agent-loop/**` and integration (INT-002).

## DEC-009 gate — RESOLVED (option A applied 2026-10-15)

The automatic-rename fish hook block in `~/.config/fish/config.fish` (original lines 40–43) is
commented out; backup `/tmp/relayable-config.fish.bak` (md5
`60a6536db6ac375f7c0e4d4f2872d609`). `delegate_project` now succeeds. Restore = uncomment the
block. The wake daemon `/tmp/relayable-wake-daemon.py` remains a fallback but is **stopped**;
restart only if delegation regresses:
`setsid python3 /tmp/relayable-wake-daemon.py /tmp/relayable-wake-daemon.stop 3600 <workspace-ids...>`.

## Next actions (in order)

1. Monitor the three Leads; when each reports, independently review its diff (`git diff --stat`,
   scope check, secret scan), run any claimed validation where feasible, then integrate into
   `master` (INT-002). Update `docs/agent-loop/**` after each merge.
2. Integrate DOC-002 (matrix citation fix + URL drift), then ADR-001/002, then TERM-001/ADR-003.
3. After ADR-001/002 are on `master`: launch CORE-001 (server-owned PTY + CLI-created session +
   terminal I/O + reconnection) with an automated integration test; then CORE-002; then UI-001/UI-002
   against `docs/research/` + `design-tokens.md`; then QA-001 smoke/CI.
4. Keep one writer per branch/path; update `STATUS.md`, `BACKLOG.md`, `TASKS.md`, `TESTS.md`,
   `WORKLOG.md`, `NEXT.md` after every meaningful iteration.

## Review gates (do not skip)

- UI fidelity: frames were actually inspected; no generic redesign; unverified states stay labelled.
- No REX wire/binary compatibility claims (§3).
- No GoRex source/asset reuse; no repo-level license found (§2.6).
- Android Studio is not a dependency; commit a Gradle wrapper (§10).
- Never claim server-crash or host-reboot session survival (§6.3).

## Open approval gates

- **Android SDK/emulator install + project JDK pin** — only needed for Android work; unresolved.
- **DEC-006 env namespace** (`RELAYABLE_*` vs REX aliases) — ADR-001/002 Lead will recommend; Manager
  decides on integration.
- **Packaging target** (`.deb`/`.rpm`) — later.

## If context pressure hits

Follow DEC-008: finish a safe checkpoint, stop launching new work, record results and repo state,
write a short handoff; do not compact to keep the session alive.
