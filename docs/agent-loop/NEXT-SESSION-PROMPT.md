# NEXT-SESSION PROMPT — copy/paste into a fresh Pi session

> Durable copy of the prompt to start the next Relayable Manager session. `master` was clean and
> pushed at `416fa51` when this was written; the restart trigger below is DEC-008 (compaction is
> disabled).

---

Continue the Relayable project. Read `full.md` completely, then `AGENTS.md` and
`docs/agent-loop/{NEXT,STATUS,BACKLOG,TASKS,DECISIONS,TESTS}.md` plus the `WORKLOG.md` tail, and
`docs/architecture/interface-contract.md` + `docs/adr/ADR-001..005`. Trust repo files over chat.

**State:** `master` is clean, integrated, and pushed. **All five ADRs are Accepted** (ADR-001 domain,
ADR-002 protocol, ADR-003 terminal engine + GPUI pin, ADR-004 ownership/persistence, ADR-005 env
namespace). REVIEW-001, DOC-002, TERM-002, CORE-001, CORE-002, UI-001, UI-002, QA-001, ADR-004 are
integrated (`774d394`, `16c8bc9`, `60d4395`, `d3e0286`, `91662af`, `09928db`, `d46c66c`, `4a258c2`,
`9a3fb27`, `74e7e20`). No live projects. The core workspace is green (`cargo test --workspace` 150
pass; `scripts/{validate,smoke,ci}.sh` pass); the GPUI reference workspace in `apps/linux` builds and
runs against the live server.

**Scope:** Android is **deferred** (DEC-010) — no Android app/SDK/emulator/Gradle work, but keep
`relayable-model`/`relayable-protocol` Android-ready.

**Primary user goal:** reach a **locally installable / testable Linux app for personal use** — a
`systemd --user`-runnable server + the GPUI client + a user install path — plus the remaining
correctness items. Work continuously in bounded projects (one writer per branch/path; merge each to
`master` as it completes; update `docs/agent-loop/**` after every iteration). Do not stop after one
task.

**Ordered work (adjust if evidence changes):**
1. **CTX-001** — implement ADR-005's `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER` in managed
   terminals and add the ADR-005 §8.1 context-resolution test.
2. **INSTALL-001** — packaging + user install: a `scripts/install-user.sh` (release build; install the
   `relayable`/`relayable-server` binaries, a `systemd --user` unit, and a desktop entry under XDG
   paths; no root, no system-wide changes), a working foreground server mode, and a README quickstart.
   Inspect the real CLI first — never invent flags (`full.md` §0.3.5).
3. **STORE-001** — `crates/relayable-store` per ADR-004 (SQLite metadata/audit; start-up
   reconciliation, `flock`, prune, write path). Designed, not implemented.
4. **Backlog:** CLIENT-API-001 (client raw-byte streaming), MEM-001 (measured memory baseline),
   CI-HOST-001 (hosted CI), REF-004 remainder.
5. **TERM-003 (libghostty)** — only if the user asks.

**Constraints:** follow DEC-008 (no compaction; repo files authoritative; fresh bounded agents;
concise handoffs; checkpoint before context pressure). No REX wire/binary compatibility claims
(`full.md` §3); no GoRex source/asset reuse (§2.6). Never claim server-crash or host-reboot session
survival (§6.3). UI fidelity to the five reference videos is a primary acceptance criterion (§0.2).
No `sudo`, global installs, system config changes, or firewall changes without explicit user approval.

**First command:**
```bash
cd /home/sulthonuladib/work/relayable && git status --short --branch && git log --oneline -8 && \
  cat docs/agent-loop/NEXT.md
```
