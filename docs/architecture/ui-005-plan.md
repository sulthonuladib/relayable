# UI-005 plan — discoverable shortcut overlay + single-source keymap (2026-10-16)

Owner: lead. Status: **ready to implement** (planner pass on `master` `3fab2f0`). This file exists so the
next session can implement UI-005 without re-doing the scoping pass. Anchors are file:line in the tree.

## Goal
A discoverable shortcut reference in the desktop app: a `?`-style overlay plus **one** source of truth for
the keymap (today the same 10 rows are duplicated in the handler `match`, `--help`, and README §11).
Bindings stay **Relayable-original** — the reference videos are macOS-only and never demonstrate a Linux
shortcut overlay (`docs/research/open-questions.md` Q4; `docs/research/ui-catalog.md` has **no** catalogue
entry for a shortcut overlay → `not observed`; the *command palette* at V4 0:30 is a **different, separate,
unimplemented** surface — do not borrow its measured geometry or present the overlay as it).

## Current reality (verified)
- All bindings are one hand-written `match`: `apps/linux/src/app/keys.rs:10-61` (`handle_key`);
  picker modal gate `:12-15`; `control && shift` `:17-41` (`e` split-right, `o` split-down, `w` close,
  `z` zoom, `s` picker, `n` new tab, `right`/`left` focus, `t` next tab); `control && !shift` `:42-54`
  (PageUp/Down, `Ctrl+1..9`); PTY fallthrough `:56-60`; picker keys `:63-94`; reserved-modifier rule +
  doc `:99-108`; PTY key table `:109-124` (incl. `escape` → `0x1b` at `:113`); `key_char` path `:125-138`.
- Keys reach the app by exactly one route: `render.rs:44-49` (`.key_context("Workspace").track_focus(..)
  .on_key_down(..)` → `handle_key`). `App::bind_keys` exists in gpui 0.2.2 but is **unused** — do not switch.
- The keymap is duplicated three times: `main.rs:24-37` (doc) + `:106-128` (`print_help`), `README.md:350-366`
  (§11). No test links them. No overlay surface exists; `src/ui/palette.rs` is the ANSI **xterm colour
  palette**, not a command palette.
- Precedents to copy: the picker is an `.absolute()` (out-of-flow) overlay mounted at `render.rs:65-67`
  with `.occlude()` available (`gpui-0.2.2/src/elements/div.rs:596,997`); the connection banner
  (`render.rs:58-60`) is **in-flow** and must not gain a sibling.
- Desktop suite is 65 tests; `app/tests.rs` is GPUI-free (pure helpers, precedent for headless tests).

## Bounded brief
**Owned paths:** `apps/linux/src/app/shortcuts.rs` (new), `app/keys.rs`, `app/render.rs`, `app.rs`
(module decl + `shortcuts_open` field), `app/tests.rs`, `src/geometry/invented.rs`, `src/main.rs`
(help derived from the table), `apps/linux/README.md` (§2.3 label + §11 + a §10 gap line),
`apps/linux/evidence/ui-005-notes.md`, `ui-005-run.log`, optional `evidence/run-ui-005.sh` (copy the
`run-ui-002.sh` pattern; keep it in `evidence/`, do **not** add it to `scripts/` — that is `QA-DESKTOP-001`).

**Must not touch:** `src/api/**`, `src/term/**`, `src/ui/**`, `src/tokens/**`, `src/theme.rs`,
`src/geometry.rs`, `src/geometry/{v1,v4}.rs`, `app/{picker,panes,top_bar,state,streams,events,tabs,lifecycle,resize}.rs`,
`Cargo.{toml,lock}`, root `README.md`, `crates/**`, `scripts/**`, `docs/**`.

**Interface**
1. `app/shortcuts.rs`: pure, headless-testable `pub(super) struct Shortcut { id: ShortcutId,
   keys: &'static str, label: &'static str }`, `pub(super) fn shortcuts() -> &'static [Shortcut]`
   (the 10 README §11 rows, same order, `keys: "Ctrl+Shift+E"` form), and
   `pub(super) fn shortcut_for(keystroke: &Keystroke) -> Option<ShortcutId>` re-encoding exactly today's
   predicates (ctrl+shift `e o w z s n right left t`; ctrl+no-shift `pagedown pageup`, `1..9`; plus the
   toggle below).
2. **`handle_key` must dispatch on the id from `shortcut_for`**, not re-match literal strings — otherwise a
   drift test cannot fail (the same call-site gap the UI-004 verifier flagged).
3. `render_shortcuts_overlay(&self, cx) -> Div` in `app/shortcuts.rs`, `.absolute()` + `.occlude()` like the
   picker, mounted at `render.rs` **after** the picker. Style only from existing tokens
   (`theme.rs:142-181,224,276` — `SESSION_PICKER_BG/BORDER`, `PALETTE_SEARCH`, `PRIMARY_TEXT`,
   `SECONDARY_TEXT`, `UI_LABEL_FONT_SIZE_PROPOSED`, `SESSION_PICKER_RADIUS_ORIGINAL`); new spacing/anchoring
   constants named `*_ORIGINAL` in `geometry/invented.rs`. **No new colour literals** (UI-006 will re-derive
   tokens; the overlay then follows automatically).
4. **Toggle vs terminal input (decision):** use the already-reserved `Ctrl+Shift` namespace —
   `Ctrl+Shift+/`, matched as `keystroke.modifiers.control && keystroke.key == "?"` and displayed
   `Ctrl+Shift+?`. Rationale: gpui's Linux platform clears `shift` for shift+`/` and yields `key == "?"`
   (`gpui-0.2.2/src/platform/linux/platform.rs:842,905-910`), and bare `?` must keep reaching the PTY
   (`keys.rs:125-138` forwards it today). A literal bare `?` would be a stolen-character deviation and would
   have to be documented as such. `Esc` closes **only** while `shortcuts_open` (gate like `keys.rs:12-15`);
   outside the overlay `Esc` still sends `0x1b`. While open the overlay is modal: only `Esc` and the toggle
   are handled, everything else is swallowed (same contract as `handle_picker_key`). Overlay and picker are
   mutually exclusive — state the rule once and test it.
5. `print_help` renders from `shortcuts()` (no second literal table); add a **drift test** asserting every
   `keys` string in `shortcuts()` appears verbatim in `README.md` §11 (`include_str!("../../README.md")`).

## Acceptance + validation
- `cd apps/linux && CARGO_NET_OFFLINE=true cargo fmt --check` · `… cargo clippy --all-targets --locked -- -D warnings`
  · `… cargo test --locked` (**65 + N passed / 0 failed, none removed**) · `… cargo check --locked`.
  Never `cargo update` here (the lockfile holds the TERM-002 `libc 0.2.189` remedy).
- Root: `scripts/validate.sh` (exit 0, step 4 = 206/0), `scripts/smoke.sh` (`SMOKE OK`),
  `scripts/check-file-sizes.sh --strict` (exit 0).
- **Display-free proofs (all must be able to fail):** every `Shortcut` id is produced by a platform-shaped
  `Keystroke` through `shortcut_for`; no duplicate id/keys; README §11 drift; toggle/`Esc` routing table over
  `(keystroke, shortcuts_open)`; and `terminal_bytes` still maps `?`, `Escape`, letters/digits/arrows to PTY
  bytes — including the toggle key producing `None` for the PTY.
- **Live (pixels + wiring):** `--mock --canvas 1099 706` on `wayland-1`, Hyprland Lua window rule, drive
  with `wtype -M ctrl -M shift -k question`, capture with `grim -g`, verify with `identify` (pattern:
  `evidence/run-ui-002.sh:129-167`, README §6); assert the open/close edge is logged, body geometry is
  unchanged (pre/post `hyprctl clients -j` + PNG size), and a bare `?` typed at the prompt still reaches the
  pane.

## Risks / unknowns
- **`Ctrl+Shift+/` normalization is read-in-source, not yet verified live on this host** (the harness
  precedent covers only letters/digits). Confirm live before committing the binding; contingency is `Ctrl+/`
  (`key == "/"`) or an explicitly documented bare-`?` deviation.
- gpui 0.2.2: `Div::absolute`, `occlude`, `on_key_down`, `KeyDownEvent`, `Keystroke::from_xkb` are verified
  in the cached crate source; **not** verified are focus/`bind_keys` routing under this Wayland backend, layer
  surfaces, and how `occlude` interacts with the shared root `.on_key_down`. Do not switch to `bind_keys`.
- Stale text to fix in-scope: `apps/linux/README.md:221-222` (wtype-driven keys list) and the §11 deviation
  note (must name the overlay).
- UI-006 need not go first.
