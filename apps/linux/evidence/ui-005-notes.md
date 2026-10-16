# UI-005 evidence notes — discoverable shortcut reference + single-source keymap

Branch `ui/ui-005-keymap-helper` (worktree `/home/sulthonuladib/work/relayable-wt/ui-005`, from
`master` `f251629`). Scope: `docs/architecture/ui-005-plan.md`.

Files: `src/app/shortcuts.rs` (new, 388 lines), `src/app/keys.rs` (146), `src/app/render.rs` (76),
`src/app.rs`, `src/app/lifecycle.rs` (116), `src/app/tests.rs` (639), `src/geometry/invented.rs` (77),
`src/main.rs` (176), `README.md`, `evidence/*`.

## 1. What was verified, and how

| Claim | Evidence |
|---|---|
| The desktop suite is 72 tests, none removed (65 before UI-005) | §4 |
| Every keymap row is reachable from a **platform-shaped** keystroke | `app/tests.rs` + §3 (live) |
| `Ctrl+Shift+?` **is** the platform shape `key == "?"`, `shift` cleared | `ui-005-toggle-probe.log` (live) |
| The toggle opens/closes the overlay; `Esc` closes it only while open | `ui-005-run.log` edge counts |
| A bare `?` still reaches the pane; `Esc` still sends `0x1b` | `ui-005-live-question.png`, `ui-005-live-escape.png` (`?^[`) |
| Overlay does not move the window or the workspace body | pre/post `hyprctl clients -j`, every PNG 1099×706 |
| Arms still do what the keymap says (split right, zoom) | `ui-005-mock-split.png`, `ui-005-mock-zoom.png` |
| No new colour and no palette geometry | §2 |

## 2. The interface as built

`src/app/shortcuts.rs`:

- `pub enum ShortcutId` — 12 payload-free action ids (`SplitRight` … `ToggleShortcuts`).
- `pub struct Shortcut { pub(super) id, pub keys, pub label }` — one row per action.
- `pub fn shortcuts() -> &'static [Shortcut]` — the table: the 10 README §11 rows in order, then the
  overlay's own toggle. 12 rows, one per id (`Ctrl+Shift+Right`/`Left` are two rows, so the coverage
  test can require one row per id).
- `pub(super) fn shortcut_for(&Keystroke) -> Option<ShortcutId>` — exactly today's predicates.
- `pub(super) fn tab_index(&Keystroke) -> Option<usize>` — the `Ctrl+1..9` digit rule, shared with
  `shortcut_for` so it exists once.
- `pub(super) fn closes_overlay(&Keystroke) -> bool`, `pub(super) enum KeyRoute`, and
  `pub(super) fn route_key(picker_open, shortcuts_open, &Keystroke) -> KeyRoute` — the whole routing
  rule: `Picker` → `CloseShortcuts` / `SwallowShortcuts` → `Shortcut(id)` → `Terminal`.
- `pub(super) fn keys_for(id) -> &'static str` — the displayed chord lookup the edge log uses.
- `WorkspaceView::set_shortcuts_open(bool)` — logs `relayable: shortcut reference opened|closed
  (Ctrl+Shift+?)`, the edge the capture harness asserts on.
- `WorkspaceView::render_shortcuts_overlay(&self) -> Div`.

`keys.rs::handle_key` now switches on `route_key(...)` and dispatches
`ShortcutId` → method in `handle_shortcut(id, keystroke)`; no literal key string is matched outside
`shortcuts.rs`. `SelectTab` gets its index from `tab_index()`, the same rule that matched the key.

### Deviations from the plan (all deliberate)

1. **`app/lifecycle.rs` needed one line.** The plan lists `app/lifecycle.rs` under *must not touch*
   but also gives `app.rs` "module decl + `shortcuts_open` field". The field's initialiser lives in
   `WorkspaceView::new`, which is in `lifecycle.rs` (`shortcuts_open: false,`, one line). Reported to
   the lead as the plan discrepancy it is; the diff is that single line.
2. **`render_shortcuts_overlay(&self)`, not `(&self, cx)`.** The plan's signature carried `cx`, which
   only the picker needs (its rows are clickable). The overlay is non-interactive: keyboard modality
   is the router's job and there is no listener, so an unused `cx` parameter would be dead weight.
3. **`Shortcut` and `shortcuts()` are `pub`** (the plan wrote `pub(super)`).
   `src/main.rs` is a separate crate (its own binary target), so `--help` cannot see a
   `pub(super)` table. Everything else (`ShortcutId` is `pub(super)`, `shortcut_for`, `route_key`,
   `tab_index`, `closes_overlay`, `keys_for`) stays internal to `app`.
4. **`Shortcut::id` is payload-free.** `SelectTab` covers `Ctrl+1..9` with one row; carrying the index
   in the id would break the "one row per id" coverage test. The index comes from `tab_index()`.
5. **`src/main.rs`'s module-doc table was replaced by a pointer.** It was a fourth copy of the same
   rows and a doc comment cannot be generated from a table; `print_help` renders `shortcuts()`.
6. **README §11 re-shaped for the drift test.** The `Ctrl+Shift+Right` / `Ctrl+Shift+Left` row became
   two rows and `Ctrl+Shift+T / Ctrl+PageDown` one code span, so every row can be checked as
   `| \`chord\`` — the check has to be able to fail when a row is deleted, not merely when a chord is
   mentioned somewhere in the section (it did not, see §3, mutation M8).
7. **`evidence/ui-005-toggle-probe.log` is a third evidence file** beyond the plan's
   `ui-005-notes.md` / `ui-005-run.log`: the toggle normalization was confirmed with temporary
   instrumentation that is *not* in any commit, so its transcript has to be durable to be reviewable.
   The same reasoning as the repo's other `evidence/*.log` transcripts.

## 3. The toggle: `Ctrl+Shift+?`, verified live before committing

Decision (unchanged from the plan): `Ctrl+Shift+?`, matched as
`modifiers.control && keystroke.key == "?"`, displayed `Ctrl+Shift+?`.

Why it is safe: `Ctrl+Shift` is already reserved from the terminal
(`keys.rs::terminal_bytes` returns `None` for it), the Linux backend clears `shift` for
shift+`/` and reports `key == "?"` (`gpui-0.2.2/src/platform/linux/platform.rs:910-917`), and a
`Ctrl+/` keystroke is a *different* key (`"/"`), so `Ctrl+Shift+/` was unbound before UI-005 —
nothing was stolen and no PTY input changed behaviour.

The normalization was **read in source and then observed on this host**. With a temporary
`eprintln!` in `handle_key` (removed before the first commit; `grep -c PROBE src/` = 0 and
`strings target/debug/relayable | grep -c "PROBE ctrl="` = 0), `bash` driving the built app under
`wayland-1` produced (`evidence/ui-005-toggle-probe.log`, sha256
`f5ca1336c92a40403be8d33c49ef185666a176da371bd15dc4ea3c292e3a9c92`):

```
wtype -M ctrl -M shift -k question
relayable: PROBE ctrl=true shift=false alt=false platform=false key="?" key_char=Some("?")
relayable: shortcut reference opened (Ctrl+Shift+?)
wtype -M ctrl -k slash
relayable: PROBE ctrl=true shift=false alt=false platform=false key="/" key_char=None
wtype -k question
relayable: PROBE ctrl=false shift=false alt=false platform=false key="?" key_char=Some("?")
wtype -k Escape
relayable: PROBE ctrl=false shift=false alt=false platform=false key="escape" key_char=None
relayable: shortcut reference closed (Ctrl+Shift+?)
```

So `shift` really is cleared and `key` really is `"?"`; the contingency (`Ctrl+/`, `key == "/"`)
was **not** taken. Note `Ctrl+/` reports `key_char=None` (with control held the keysym's UTF-32 is a
control code), so it could not have leaked a byte either way — and the capture shows `Ctrl+/`
changes the frame not at all (byte-identical PNG, §5).

## 4. Tests added, and the mutation each one catches

`app/tests.rs` is GPUI-free (pure helpers, `Keystroke`/`Modifiers` only — no display). 7 new tests,
72 total. Each mutation below was applied with a script, the named test re-run, and the file restored;
the recorded outcome is the real one.

| Test | Catches (mutation) |
|---|---|
| `every_shortcut_id_is_reachable_from_its_platform_keystroke` | a predicate typo (`"e"` → `"E"`), a dropped arm, a row with no predicate, a binding removed from the handler but left in the table, and a binding the router would send to the pane (**M1**: FAILED) |
| `the_keymap_table_has_no_duplicate_row` | duplicate `keys` or id in the table, and a broken `keys_for` lookup that would mislabel the edge log (**M10**: FAILED) |
| `readme_lists_every_shortcut_verbatim` | a chord edited in the table without the docs (**M2**: FAILED) and a README §11 row deleted (**M8**: FAILED after §2.6) |
| `control_digits_select_tabs_and_only_one_to_nine` | an off-by-one in the digit rule (**M3**: FAILED), a rule widened to `Ctrl+0` or `Ctrl+Shift+<digit>` |
| `the_toggle_key_never_reaches_the_pane_and_a_bare_question_mark_does` | the stolen-character deviation — binding a bare `?` (**M5**: FAILED) — and a toggle that leaks a byte |
| `the_overlay_is_modal_and_escape_only_closes_it_while_it_is_open` | an overlay that fires bindings while open (**M9**: FAILED), `Esc` no longer closing it (**M4**: FAILED), the picker gate not winning (**M6**: FAILED) |
| `terminal_bytes_still_maps_the_panes_own_keys` | a terminal-byte regression while adding the toggle (`escape` → `0x07`, **M7**: FAILED) |

`M8` and `M10` as first written did **not** fail (the README check accepted a chord mentioned in
prose; nothing exercised `keys_for`). Both tests were strengthened until they did; the numbers above
are from the final versions.

### Wiring that is **not** covered headlessly

- **The `id` → method arms** in `handle_shortcut` (11 arms). `app/tests.rs` cannot build a
  `WorkspaceView` (it needs a GPUI context and an `ApiHandle`), so the tests cover the *decision*
  (`route_key`/`shortcut_for` are the real functions `handle_key` calls, not copies) but not the
  *action* each id runs. A wrong pairing (`SplitRight` → split down) would pass every headless test.
  What covers it instead: the compiler (an unhandled `ShortcutId` is a compile error) and the live
  capture (`ui-005-mock-split.png` shows a split to the right, `ui-005-mock-zoom.png` a zoomed pane,
  `ui-005-mock-overlay.png` the reference itself). The remaining nine arms are covered by nothing
  beyond the compiler.
- **The rendered overlay DOM**: its pixels are covered only by the live capture, not by a test.
- `set_shortcuts_open`'s `debug_assert!` (overlay/picker exclusivity) is asserted by the routing test
  in the open direction only.

## 5. Live capture — `evidence/run-ui-005.sh`, `evidence/ui-005-run.log`

`bash evidence/run-ui-005.sh` (mock on `wayland-1` at 1099×706 with the Hyprland runtime rule, then
a live `relayable-server` on a private socket under `/tmp`); it prints one `CHECK PASS/FAIL` line per
claim, counts its own failures, and ends with `UI-005 CAPTURE OK` (exit 0, `check_failures=0`, 30
checks). Real output in `ui-005-run.log`; highlights:

The harness was run twice — once while the feature was being verified and once after the final
`cargo fmt`/commit, on this branch's committed source. Between the two runs the 15 **window-only**
PNGs are byte-identical (only the full-screen companions, which include the desktop bar and its
clock, and the log's timestamps and pids differ), which is itself a determinism check on the
rendering.

- `CHECK PASS: toggle opened the overlay (log) = 1` / `Esc closed the overlay (log) = 1`;
  `toggle opened twice` / `toggle closed it` = 2 (both edges, both routes).
- `CHECK PASS: window geometry unchanged by the overlay = 10,40 1099x706` (pre/post
  `hyprctl clients -j`) and every window PNG `1099x706` (`identify`).
- `cmp mock-before.png mock-escape-closed.png -> yes`: closing the overlay restores the frame byte
  for byte — the overlay is out of flow and leaves no trace in the workspace.
- `cmp mock-question-pane.png mock-ctrl-slash.png -> yes` and
  `CHECK PASS: Ctrl+/ did not open the overlay (log) = 2`: the untaken contingency is inert.
- `cmp mock-overlay.png mock-overlay-2.png -> yes`: the overlay renders deterministically.
- Live: `cmp live-catv.png live-escape.png -> no` and `live-escape.png` shows the pane line
  `?^[` — the bare `?` arrived (kernel echo) and `Esc` arrived as `0x1b` (`cat -v` echoes it as
  `^[`), with the overlay closed; `live-escape.png` == `live-closed.png` shows closing the overlay
  over live content restores the pane frame.

Screenshots inspected by eye (not just hash-compared): the reference lists 12 rows in two aligned
columns under the title `Keyboard shortcuts — Relayable-original` and the footer `Esc closes this
reference`, with nothing truncated; the split capture shows the new pane on the **right** (so a
`Horizontal`/`Vertical` swap would be visible); the mock pane shows `mock$ … ?` for the bare `?`.

Fidelity notes: the panel hangs from the same left edge and the same 4 px separator offset as the
picker and reuses only measured tokens (`SESSION_PICKER_BG`/`_BORDER`,
`PICKER`'s radius token `SESSION_PICKER_RADIUS_ORIGINAL`, `PRIMARY_TEXT`, `SECONDARY_TEXT`,
`UI_LABEL_FONT_SIZE_PROPOSED`); its width/key-column/gap/padding/offset are new
`geometry::invented::SHORTCUTS_*_ORIGINAL` values (nothing measurable exists). No colour literal was
added. The connection banner is unchanged and still has no sibling.

Known cosmetic slack: panel width 460 px against a longest rendered row ending at x≈530 (panel right
edge 588), i.e. ~46 px of right slack at the resolved font — kept so a wider fallback font cannot
wrap a row.

Approximation in the overlay: it is modal for the keyboard but `.occlude()`s only its own panel area,
so a **mouse** click outside the panel still reaches the panes — the same contract as the session
picker. A full-window scrim would be an extra surface and new geometry, so it was not added.

## 6. Validation (exact commands, on the committed tree)

Transcript: `evidence/ui-005-validation.log`. All from `apps/linux` unless noted; offline/`--locked`
as required (no `cargo update`; `Cargo.lock` is byte-identical to `master`).

```
CARGO_NET_OFFLINE=true cargo fmt --check                              -> exit 0
CARGO_NET_OFFLINE=true cargo clippy --all-targets --locked -- -D warnings -> exit 0
CARGO_NET_OFFLINE=true cargo test --locked                            -> 72 passed; 0 failed
CARGO_NET_OFFLINE=true cargo check --locked                           -> exit 0
git -C apps/linux diff --exit-code Cargo.lock                          -> exit 0 (unchanged)
bash scripts/validate.sh            (repo root)  -> exit 0, all 8 steps 0, step 4 = 206 passed / 0 failed
bash scripts/smoke.sh               (repo root)  -> SMOKE OK, exit 0
bash scripts/check-file-sizes.sh --strict (root) -> exit 0, checked=151, over-hard-cap=0
```

Only pre-existing noise appears: `cargo`'s future-incompat note for the dependency
`proc-macro-error2 v2.0.1`, and the file-size report's 8 files between target and cap (unchanged from
`master`; `shortcuts.rs` 388 ≤ 500 target, `tests.rs` 639 ≤ 800 target).

## 7. Left for the lead / known stale text

- **README §13** still says `cargo test --locked # exit 0 (58 tests)`. The desktop suite was already
  65 before UI-005 and is 72 now. §13 is outside the sections the brief names (`§2.3`, `§11`, a §10
  line), so it was left alone rather than widened into.
- **`apps/linux/README.md` §6** describes `run-ui-002.sh`'s wtype list; UI-005's own harness is not
  listed there (only the stale `Ctrl+Shift+N splits` wording was corrected to `Ctrl+Shift+N new
  tab`, as the plan's risks section asked).
- The keymap is still per-crate: `apps/linux` has no shared crate for it, and `--help`/README read it
  through the `pub` items described in §2.3.
- `docs/**` was not edited (the plan owns that), so `docs/architecture/ui-005-plan.md` still says
  "65 tests" and `pub(super)`; this file is the correction record.
