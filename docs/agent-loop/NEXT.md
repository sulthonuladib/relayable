# NEXT — instructions for the next resumed run

Read `full.md` completely, then `docs/agent-loop/STATUS.md`, `TASKS.md`, `DECISIONS.md`,
`WORKLOG.md` (tail), and this file. Trust these artifacts over prior chat.

## First command to run

```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -5
```

> Path note: the Phase 0 kickoff WORKLOG recorded `/home/sulthonuladib/personal/relayable`; that path
> no longer exists. The checkout is `/home/sulthonuladib/work/relayable` (same history and remote).

Then confirm the working tree state matches the last WORKLOG checkpoint before doing anything else.

## Current phase

Phase 0 — **Wave 1 research is running on three independent branches**. Manager reviews and
integrates (INT-001) before Wave 2. Check `list_staff` for live Leads and `git worktree list` /
`git branch -a` for branch state.

| Branch | Produces | Exclusive paths |
|---|---|---|
| `research/ui-reference` | `docs/research/{video-review,ui-catalog,open-questions,gorex-review}.md` | `docs/research/**` |
| `research/protocol-compat` | `docs/compatibility/reference-behavior-matrix.md` | `docs/compatibility/**` |
| `research/env-audit` | `docs/operations/toolchain-audit.md` (+ optional `android-toolchain.md`) | `docs/operations/**` |

## Immediate priorities

1. **Collect Wave 1 findings.** For each branch: inspect the diff (`git log --stat`, `git show`),
   verify the documents exist at the paths above, verify honesty (access routes, confidence levels,
   no invented flags/timestamps/benchmarks), and confirm no out-of-scope edits.
2. **INT-001 — integrate** the three research branches into `master`; resolve doc conflicts
   (property ownership is disjoint, so conflicts should be zero); commit integration; record
   evidence in `WORKLOG.md`.
3. **Wave 2 — decisions and first engineering slice**, in order:
   - `TERM-001` terminal engine spike + `ADR-003` (from ENV-001 evidence);
   - `ADR-001` domain model/dependency boundaries and `ADR-002` protocol serialization/transport
     (from DOC-001 evidence);
   - `CORE-001` server-owned PTY + CLI-created session + terminal I/O + reconnection, with an
     automated integration test; then `CORE-002` disconnect survival/reattach;
   - `UI-001`/`UI-002` GPUI workspace only against the integrated UI catalogue — high-fidelity
     recreation, not a generic redesign.
4. Keep `TASKS.md`, `STATUS.md`, `WORKLOG.md`, and this file updated at every checkpoint.

## Wave 1 review gate (do not skip)

- Do **not** claim video/UI fidelity before frames were actually inspected. Label inaccessible
  footage honestly (§2.7).
- Do **not** implement or claim REX wire/binary compatibility; no protocol is documented
  empirically (§3).
- Do **not** copy GoRex source or assets; verify license first. Behaviour observation only (§2.6).
- Treat research docs as evidence to verify, not as truth to trust: spot-check cited URLs/timestamps.
- Do not let any Wave 1 branch's uncommitted files leak into `master` without review.

## Hard constraints to respect

- Do **not** install global tools, `sudo`, change system config, or open ports (§0.3.10–11).
- `gradle` is absent → use a committed Gradle wrapper. Android Studio is not permitted as a
  dependency (§10).
- Never claim server-crash or reboot session survival (§6.3).
- Preserve unrelated user work; keep unrelated formatting changes out of commits.

## Known environment facts (verified Wave 1 kickoff)

rustc/cargo 1.98.1 · mise present · JDK 25 (Temurin, via mise) · `android` CLI + `adb` present ·
`gradle` absent · `ffmpeg`/`yt-dlp`/`chromium`/`curl` present · Node 26.8.2 / npm 11.19.1 (mise) ·
`/dev/kvm` present · `systemd --user` running · Linux x86_64 omarchy · herdr 0.9.5 ·
~51G free on `/home` · repo `.gitignore` already excludes `/docs/research/evidence/` and
`**/evidence-local/` for local-only captures.

## Open questions for the user (only if genuinely blocking)

None at this checkpoint. Decisions that will eventually need user input:

- distribution/packaging target confirmation (`.deb` vs `.rpm` vs both, §7.3);
- the `RELAYABLE_*` vs REX-compat environment-variable namespace decision (§8.1, DEC-006).

Both can wait; continue other safe work.
