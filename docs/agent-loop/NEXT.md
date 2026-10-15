# NEXT — instructions for the next resumed run

Read `full.md` completely, then `docs/agent-loop/STATUS.md`, `TASKS.md`, `WORKLOG.md` (tail), and this file. Trust these artifacts over prior chat.

## First command to run

```bash
cd /home/sulthonuladib/personal/relayable && git status --short --branch && git log --oneline -5
```

Then confirm the working tree state matches the last WORKLOG checkpoint before doing anything else.

## Current phase

Phase 0 — repository/evidence audit. Repo was spec-only (`full.md`) at start; scaffold now exists.

## Immediate priorities

1. **Collect the three research waves' findings** and review them before implementation:
   - `docs/research/video-review.md`, `ui-catalog.md`, `open-questions.md`, `gorex-review.md`
   - `docs/compatibility/reference-behavior-matrix.md`
   - `docs/operations/**` toolchain audit
2. **ADR-001** — domain model and dependency boundaries (`docs/adr/ADR-001`).
3. **CORE-001** — first engineering deliverable: a **tested** local slice where the server owns a PTY, the CLI creates a session, terminal I/O round-trips, killing the client leaves the session alive, and reconnect retrieves current state.

## Hard constraints to respect

- Do **not** claim video/UI fidelity before frames are actually inspected. Label inaccessible footage honestly (§2.7).
- Do **not** implement or claim REX wire/binary compatibility; no protocol is documented empirically (§3).
- Do **not** copy GoRex source or assets; no verified license was found. Behaviour observation only (§2.6).
- Do **not** install global tools, `sudo`, change system config, or open ports (§0.3.10–11).
- `gradle` is absent → use a committed Gradle wrapper. Android Studio is not permitted as a dependency (§10).
- Never claim server-crash or reboot session survival (§6.3).

## Known environment facts

rustc/cargo 1.98.1 · mise present · JDK 25 (Temurin, via mise) · `android` CLI + `adb` present · `gradle` absent · `ffmpeg`/`yt-dlp`/`chromium` present · Linux x86_64 omarchy · herdr 0.9.5.

## Open questions for the user (only if genuinely blocking)

None at this checkpoint. The two decisions that will eventually need user input are:
- distribution/packaging target confirmation (`.deb` vs `.rpm` vs both, §7.3);
- the `RELAYABLE_*` vs REX-compat environment-variable namespace decision (§8.1, DEC-006).

Both can wait; continue other safe work.
