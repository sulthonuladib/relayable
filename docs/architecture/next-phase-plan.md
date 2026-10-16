# Next-phase plan — desktop app, terminal engine, docs (2026-10-16)

Owner: Manager. Status: **planning** (research in flight). This records the user's 2026-10-16 directives
and how they are sequenced without colliding with in-flight work.

## Objectives

1. **GPUI kit** — evaluate `gpui-kit` (https://gpui-kit.com/docs) and plan adoption for `apps/linux`
   (components, provider, theming) **without breaking reference-video UI fidelity** (`full.md` §0.2/§2.4).
2. **Terminal engine** — evaluate/Adopt **libghostty / `libghostty-rs`** to replace the provisional
   `alacritty_terminal` + `vte` engine. This is ADR-003 reconsideration trigger #10; it needs an
   evidence spike + an ADR-003 amendment before any code switch.
3. **Desktop UX** — auto-create (or auto-attach) a session when the app opens; a **keymapping helper**
   (discoverable shortcuts, e.g. a `?` overlay); a real **theming** system driven by the design tokens.
4. **Cleanup + docs centralization** — after the refactor projects merge and the decisions above land:
   remove merged worktrees, and consolidate the `docs/` tree behind a central index.

## Sequencing (non-collision)

| Phase | Tasks | Precondition |
|---|---|---|
| 0 (now) | `RESEARCH-001`/`RESEARCH-002` (gpui-kit, libghostty) — docs-only, `w1P` | none |
| 1 | Integrate returning refactors (`REF-006`, `REF-007/009`, then `REF-008` after STORE-001) and clean their worktrees | refactor handoffs |
| 2 | `TERM-003` engine spike (libghostty-rs) + ADR-003 amendment | `RESEARCH-002` |
| 3 | `UI-004` auto-session-on-open, `UI-005` keymap helper, `UI-006` theming; `UI-003` migrate to `Client::next_output` | `RESEARCH-001` + `TERM-003` if the engine touches `term/` |
| 4 | `DOCS-001` central docs index/consolidation | phases 2–3 decisions recorded |
| 5 | `REF-010` file-size CI gate | all refactors merged |

One writer per branch/path throughout; `apps/linux/**` is a single writer at a time (phases 2–3 must not
run concurrently on the same files).

## Task ledger additions

- **RESEARCH-001** (part of `w1P`) — `docs/research/gpui-kit.md`: license, `gpui =0.2.2` compatibility,
  component inventory, fidelity impact, recommendation.
- **RESEARCH-002** (part of `w1P`) — `docs/research/libghostty.md`: license, build requirements, API,
  Android story, comparison vs ADR-003 pins, ADR-003 reconsideration evidence.
- **TERM-003** (proposed → READY after RESEARCH-002) — libghostty-rs engine spike + ADR-003 amendment.
  Not started until the research recommends and the license/build story is verified.
- **UI-004** — desktop: create/attach a session automatically on app open (no manual `new`); clear
  empty/loading/error/disconnected states; keep `--mock` labelled.
- **UI-005** — keymapping helper: discoverable shortcut overlay + `relayable --help`-style reference;
  Linux bindings are Relayable-original (documented deviation, `open-questions.md` Q4).
- **UI-006** — theming: single token-driven theme (light/dark as evidenced), applied through the chosen
  component layer; no arbitrary restyle that diverges from the reference geometry.
- **UI-003** (existing) — migrate `apps/linux` off the raw protocol data plane onto `Client::next_output`.
- **DOCS-001** — `docs/README.md` index + consolidation of research/compatibility/operations/architecture
  trees; keep evidence intact, remove duplication.
- **REF-010** (existing) — wire `scripts/check-file-sizes.sh --strict` into CI once splits are done.

## Constraints / gates

- **Fidelity first** (`full.md` §0.2, §2.4): a component library or theme must not change the reference
  screen geometry/hierarchy. Any divergence is recorded as an explicit deviation.
- **No invented APIs/flags** (`full.md` §0.3.5): adopt only what the upstream docs and a real build prove.
- **No global installs / system changes** (`full.md` §0.3.10–11): if libghostty needs a toolchain not on
  this host (e.g. Zig), report it — do not install; propose a `mise` pin or defer.
- **Android readiness** (DEC-010): the engine choice must not break the future Android client; the model
  and protocol stay host-independent.
- **No REX compatibility claims** (`full.md` §3); no GoRex source/asset reuse (§2.6).

## Open decisions (need evidence, then a Manager ADR/decision)

1. gpui-kit: adopt / partial / defer (depends on 0.2.2 compatibility and fidelity).
2. libghostty-rs: adopt as the engine + required ADR-003 amendment (license, build, API coverage).
3. Theming approach: gpui-kit theme vs a local token-driven theme in `apps/linux`.
