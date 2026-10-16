# NEXT-SESSION PROMPT — copy/paste into a fresh OpenCode session

> Durable prompt to start the next Relayable lead session. `master` was clean when this was written;
> check `git log` for the current tip. Compaction is disabled (`opencode.json` → `compaction.auto:
> false`, DEC-008/DEC-015); repo files are authoritative. `scripts/agent-loop.sh` runs this prompt as
> a sequence of fresh sessions.

---

Continue the Relayable project. Read `full.md` completely, then `AGENTS.md` and
`docs/agent-loop/{NEXT,STATUS,BACKLOG,TASKS,DECISIONS,TESTS}.md` plus the `WORKLOG.md` tail,
`docs/architecture/interface-contract.md`, `docs/architecture/next-phase-plan.md`,
`docs/architecture/code-organization.md`, and `docs/adr/ADR-001..005`. Trust repo files over chat.

**State:** `master` is clean, integrated, and pushed. **No live projects; all worktrees removed**
(only the primary checkout). Worktree cleanup is **automatic** under the standing user authorization in
`AGENTS.md` — use plain `git worktree remove` / `scripts/cleanup-worktrees.sh --apply <branch>...` for
done+merged worktrees; no per-removal confirmation is needed. The local app is **installed and runnable** on
this host (`~/.local/bin/{relayable,relayable-server,relayable-desktop}`, `systemd --user` unit installed but
not enabled — see `README.md` "Quick start").

**Integrated (all green):** CTX-001 (INT-015 `dd1b986`), INSTALL-001 (INT-016 `39e5d49`), CLIENT-API-001
(INT-017 `52433f7`), STORE-001 (INT-018 `f4d59f8`), REF-007/009 (INT-019 `54fe220`), REF-006 (INT-020
`ba92c51`), RESEARCH-001/002 (INT-021 `96b4180`). `scripts/{validate,smoke,ci}.sh` all pass;
`cargo test --workspace` = **194 pass / 0 fail** (build before test: the `tests/fixtures/binary.rs`
freshness guard requires `cargo build --workspace` first — see `QA-HARNESS-001`).

**Accepted decisions to honor:** DEC-008 (no compaction; fresh bounded agents; checkpoint before context
pressure), DEC-010 (Android deferred), DEC-011 (unique-session CLI default), DEC-012 (rusqlite `=0.40.2`
bundled; builds stay offline), DEC-013 (no Superlogical attribution on Relayable surfaces), DEC-014
(below). **New rules:** `AGENTS.md` "Code organization" (files ~500 target/1000 cap; `scripts/check-file-sizes.sh`),
"Worktree lifecycle / cleanup", and "Agent coordination (no polling)" — agents are pinged on completion;
never poll peers.

**Decision pending from the user (DEC-014):** RESEARCH-001/002 recommend **Reject/Defer gpui-kit** (it cannot
coexist with the pinned `gpui = "=0.2.2"`; no umbrella license file) and **Defer/Partial libghostty-rs**
(terminal API is `main`-only and self-declared unstable; Zig 0.16 is a build blocker under the
no-global-install rule; leading binding unlicensed). **Do not start TERM-003 or adopt either dependency
until the user decides.** The desktop-UX goals do NOT need gpui-kit and can be built against the existing
pin. Evidence: `docs/research/gpui-kit.md`, `docs/research/libghostty.md`.

**Ordered work:**
1. **REF-008** — split `crates/relayable-server/src/session.rs` (1037) and `crates/relayable-server/tests/server_e2e.rs` (1465); this is the last hard-cap blocker. Then **REF-010** — wire `scripts/check-file-sizes.sh --strict` into `scripts/ci.sh` + `scripts/validate.sh` and verify clean.
2. **Desktop UX (no new deps):** `UI-003` migrate `apps/linux` off its raw `relayable-protocol` data plane onto `Client::next_output`; `UI-004` auto-create/attach a session when the app opens (explicit empty/loading/error/disconnected states; keep `--mock` labelled); `UI-005` keymapping helper (discoverable shortcut overlay); `UI-006` token-driven theming. **Preserve reference-video fidelity (`full.md` §0.2/§2.4).**
3. **DOCS-001** — centralize the `docs/` tree behind `docs/README.md`; consolidate duplicates/stale content (keep evidence intact). Do this AFTER the DEC-014 decision is recorded.
4. Backlog: `MEM-001` (memory baseline), `CI-HOST-001`, `REF-004` remainder, `QA-FLAKE-001` (pre-existing `server_e2e` ~1% flake), `QA-HARNESS-001` (freshness-guard heuristic).
5. **TERM-003** (only if the user overrides DEC-014) — libghostty-rs spike + ADR-003 amendment.

**First command:**
```bash
cd /home/sulthonuladib/personal/relayable && git status --short --branch && git log --oneline -8 && \
  cat docs/agent-loop/NEXT.md && git worktree list
```

**Constraints:** no `sudo`, no global installs, no system-config or firewall changes without explicit user
approval; no network dependency for builds (only the pre-fetched `rusqlite` cache); never invent CLI/API
flags (`full.md` §0.3.5); no REX wire/binary compatibility claims (§3); no GoRex source/asset reuse (§2.6);
never claim server-crash or host-reboot session survival (§6.3); one writer per branch/path; the
orchestrator owns `docs/agent-loop/**`, `docs/adr/**`, `AGENTS.md`, `opencode.json`, root `Cargo.toml`,
and integration (`.opencode/agents/`).

**Approval gates:** none active. Android remains deferred (DEC-010). TERM-003 / engine change is gated on
the user's DEC-014 decision.
