# UI-002 reference workspace — raw UI notes (for the later README stage)

Worktree: `/home/sulthonuladib/.herdr/worktrees/relayable/ui-ui-002-reference-workspace`
Scope: `apps/linux` only. This file is **raw notes for the README stage**; it names
each screen → catalogue entry, every token with its confidence, every documented
deviation, the screenshot discrepancy list, and the supported fidelity level.

Raw transcripts: `ui-002-run.log` (live capture harness),
`ui-002-live-probe.log` (stage A), `ui-002-stage-a.md` (stage A).

---

## 1. Screens implemented → catalogue entry + source timestamp

| Implemented UI | Catalogue entry (`docs/research/ui-catalog.md`) | Reference frame |
|---|---|---|
| Top bar (40 px) + 1 px separator, horizontal tab strip, `+` | `desktop/workspace-default`, `desktop/tab-strip` | V1 0:24 `v1-t0024`; V1 0:47 `v1-t0047` |
| 205×30 active pill at x = 128 px; inactive flat tabs with 1 px divider; duplicate-label `-1` rule | `desktop/tab-strip` | V1 0:47 `v1-t0047`; V1 1:33 (duplicates) |
| Pane header (41 px): glyph + live program/cwd; four buttons (split-right, split-down, zoom, close) on the focused pane | `desktop/pane-header` | V1 0:47; V4 0:58 `v4-t0058` |
| Terminal body: measured vertical ramp, 17.25 px row pitch, block cursor, scrollback scrollbar | `desktop/terminal-body` | V1 0:47, 1:40 `v1-t0140`, 2:03 |
| Horizontal 50/50 split, 9.5 px gap, focused/unfocused borders + surfaces | `desktop/split-layout` | V4 0:58 `v4-t0058` |
| Session picker popup (183 × 139.5 px, search field, rows, blue selected row) | `desktop/session-picker` | V1 1:40 `v1-t0140` |
| Connection/error/disconnected banner + mock banner | **no reference** — `open-questions.md` Q9 | none (Relayable-original) |

Deliberately **not** implemented (all documented gaps, not oversights):

- `desktop/vertical-tabs` — `v1-t0222` does not depict a vertical-tab layout, so its
  geometry is unmeasurable (`REFERENCE-FRAMES.md` §5). Implementing it would require
  invented geometry, forbidden by the brief.
- `desktop/command-palette` — optional in the brief; core geometry came first.
- `desktop/remote-host`, `desktop/reconnect-after-quit`, `desktop/cli-*` — no remote
  transport or CLI in this slice.
- Session pill (`desktop/session-pill`) — remote-host chrome; no remote sessions here.

---

## 2. Tokens used (value, confidence, where)

All colours come from `src/theme.rs` (packed `0xRRGGBB` mapped from `src/tokens.rs`),
geometry from `src/geometry.rs`.

### V1 chrome (high confidence unless noted)

| Token | Value | Conf. | Use |
|---|---|---|---|
| `TOP_BAR_BG` | rgb(4,27,46) | high | top bar, inter-pane gap background |
| `TOP_BAR_SEPARATOR` | rgb(30,49,66) | high | 1 px separator at y = 40 |
| `ACTIVE_TAB_PILL_FILL` | rgb(19,41,62) | high | active tab pill |
| `INACTIVE_TAB_DIVIDER` | rgb(57,73,95) | medium | 1 px divider between two inactive tabs |
| `PANE_HEADER_SURFACE_V1` | rgb(9,30,46) | high | single-pane header (V1) |
| `TERMINAL_BG_TOP_V1` → `TERMINAL_BG_BOTTOM_V1` | rgb(10,29,43) → rgb(17,27,35) | medium | single-pane terminal ramp (48 bands) |
| `PRIMARY_TEXT` | rgb(191,219,228) | medium | tab/pane labels, terminal default fg |
| `SECONDARY_TEXT` | rgb(108,111,159) | low-medium | inactive tab text, header glyph |

### V4 split chrome (medium-high unless noted)

| Token | Value | Conf. | Use |
|---|---|---|---|
| `FOCUSED_PANE_HEADER_V4` / `UNFOCUSED_PANE_HEADER_V4` | rgb(0,36,53) / rgb(6,27,38) | medium-high | split pane headers |
| `FOCUSED_PANE_BODY_*` / `UNFOCUSED_PANE_BODY_*` | (8,34,45)→(7,31,36) / (7,26,34)→(9,25,28) | medium-high | split pane ramps |
| `PANE_BORDER_FOCUSED_V4` / `PANE_BORDER_UNFOCUSED_V4` | rgb(31,82,115) / rgb(17,54,71) | medium-high / medium | 2.5 px pane borders |

### Session picker (medium unless noted)

`SESSION_PICKER_BG` rgb(14,28,39); `SESSION_PICKER_BORDER` rgb(80,97,111);
`SESSION_PICKER_SELECTED_ROW` rgb(68,156,255) (high); search field reuses
`PALETTE_SEARCH_FIELD` rgb(31,54,70).

### Geometry (raw video px, treated as logical px by assumption — §2.3)

| Token | Value | Conf. |
|---|---|---|
| `TOP_BAR_HEIGHT` | 40 px | high |
| `TOP_BAR_SEPARATOR_RENDER_PX` | **1 px** | high (local re-measure of `v1-t0047`/`v1-t0024` found exactly 1 px; the token range is 1–2 px with a 1.5 px midpoint) |
| `ACTIVE_TAB_PILL_WIDTH/HEIGHT` | 205 × 30 px | high |
| `TAB_LEADING_INSET` | 128 px | medium-high (derived: measured 538 = 128 + 205 + 205) |
| `TAB_PILL_CORNER_RADIUS` | 9 px | low (visual range 8–10 px) |
| `TAB_DIVIDER_RENDER_PX` | 1 px | medium-high |
| `PANE_HEADER_HEIGHT` (V4) | 41 px | medium (range 40–42 px) |
| `INTER_PANE_GAP` | 9.5 px | high (range 8–11 px) |
| `SPLIT_RATIO` | 0.5 | medium-high |
| `PANE_BORDER_*` | 2.5 px | medium-high / medium |
| `TERMINAL_ROW_PITCH` | 17.25 px | high |
| `SESSION_PICKER_*` | 183 × 139.5 px; search 28.5; row 35 | medium |
| `BASELINE_CANVAS_OUTER` | 1099 × 706 px | high (live capture matches exactly) |

### Typography — **Relayable-original / non-reference** (`design-tokens.md` §4, Q2)

- Family resolved at startup from `theme::FONT_FAMILY_PREFERENCE`; on this host it
  resolves to **JetBrainsMono Nerd Font** (logged), cell width **7.800 px**.
- Terminal font size **13 px** (`TERMINAL_FONT_SIZE_PROPOSED`); UI label **12 px**;
  pane label **12 px**. The reference font family/size are **not measurable**.

### Relayable-original chrome (non-reference, `theme.rs`)

Block cursor `CURSOR_FILL_ORIGINAL` rgb(125,128,207) @ alpha 0.55; scrollbar
`PRIMARY_TEXT` @ alpha 0.18 (track) / 0.70 (thumb), width 4 px; pane-card radius 6 px;
connection banner 26 px and mock banner 20 px with their own background colours;
ANSI palette: **standard xterm 256** (`src/ui/palette.rs`), not measured.

---

## 3. Documented deviations (every one explicit)

1. **Empty 128 px traffic-light region.** The reference strip begins at x = 128 because
   the macOS traffic lights held that space; traffic lights are **not app tokens**
   (`design-tokens.md` §3.2). Relayable leaves the strip empty; the measured pill
   position is preserved.
2. **Top bar flush at y = 0.** The reference top bar starts ~3 px below the window top
   (macOS window border). Relayable's Wayland toplevel has no macOS border, so the bar
   is flush; the 40 px bar + 1 px separator geometry is preserved.
3. **1 px separator.** The local re-measurement found exactly 1 px; the token midpoint
   is 1.5 px. Rendering uses the measured integral value.
4. **ANSI palette is non-reference.** `design-tokens.md` §4 marks terminal colours under
   translucency (and the palette) unmeasurable; non-default cells use standard xterm.
5. **Cursor colour/alpha non-reference.** Block shape is observed; colour is not. Uses
   the low-confidence local-prompt periwinkle at 0.55 alpha so the glyph stays legible.
6. **Connection/error/disconnected visuals non-reference (Q9).** The reference never
   showed them; they are a labelled full-width banner + mock strip.
7. **MOCK label.** `--mock` is non-default and shows a persistent amber
   `MOCK — synthetic in-process data; not a live relayable-server` strip.
8. **Split requires a new session.** CORE-002 has no `block.create`, so each pane is its
   own session/block (`session.create` → `block.attach`). A wire-model limitation, not a
   UI choice. Total panes are capped at the server's `max_concurrent_streams = 8`.
9. **Keybindings are Relayable-original (Q4).** Linux bindings were never observed:
   `Ctrl+Shift+E` split right, `Ctrl+Shift+O` split down, `Ctrl+Shift+W` close pane,
   `Ctrl+Shift+Z` zoom, `Ctrl+Shift+N` new session, `Ctrl+Shift+S` picker,
   `Ctrl+Shift+T`/`Ctrl+PageDown` next tab, `Ctrl+PageUp` previous, `Ctrl+1..9` select
   tab, `Ctrl+Shift+←/→` focus pane. Also printed by `--help`.
10. **V1 pane-header height inferred from V4.** V1 is explicitly unmeasurable (no colour
    boundary); the 41 px V4 midpoint is used.
11. **Four pane buttons only when a tab is split.** The V1 single-pane frame shows **no**
    buttons; the V4 focused split pane shows four. Single-pane headers therefore stay
    clean, with splits providing the buttons plus the keybindings.
12. **Pane-card corner radius is non-reference.** The V4 cards are visibly rounded but
    the radius is unmeasured; 6 px is a Relayable-original choice.
13. **Terminal default foreground = `PRIMARY_TEXT`.** The exact terminal font colour is
    not isolatable from compression; the measured primary-text value is used.
14. **Scrollbar is non-reference.** A native scrollbar was observed (V1 2:03) but its
    colours/size were not measured.
15. **Every pane holds its own live stream (Stage D).** Each pane's block is attached
    and credit-acked independently, so a split does not freeze the first pane; focus is
    purely local. The per-connection cap (8) still mirrors the server's advertised
    `max_concurrent_streams`.
16. **Window opacity workaround.** gpui 0.2.2's Linux backend treats client-side
    decorations as transparent; Hyprland forces client-side decorations, leaving a small
    residual surface alpha. The app re-asserts opaque server-side decorations and
    re-applies `WindowBackgroundAppearance::Opaque` each frame, which reduces but does
    not fully eliminate a ~3 % desktop bleed (see §4).
17. **`lib.rs` doc comment corrected (Stage D).** The library target does carry GPUI
    (`src/app.rs`, `src/ui/**`); the stale "GPUI-free" wording was fixed to describe the
    actual module split. `apps/linux/README.md` remains a later-stage task.

---

### Stage D — confirmed defects fixed (independent review)

Nothing was redesigned: the reference layout/geometry and every measured token are
unchanged. Each defect below was confirmed by the independent review and is fixed in
the final binary that produced this run's screenshots.

| ID | Confirmed defect | Fix (path) |
|---|---|---|
| D1 | only one live data-plane stream, so a split froze the first pane until refocus | `DataPlane` holds a `BTreeSet<u64>` of attached stream ids and credit-acks output for every attached stream; focus no longer detaches (`src/api/mod.rs`, `src/app.rs`) |
| D2 | `reconcile_resizes` assumed at most two panes (3rd+ never resized, 0/1 got wrong half-splits) | every shown pane is sized from the real flex division (equal share, measured 9.5 px gap, 50/50 for exactly two) via `geometry::pane_extent`; edge-triggered (`src/app.rs`) |
| D3 | `Attached` could bind to the wrong pane via focus fallback | FIFO `pending_attach: VecDeque<u64>` resolves replies in request order; no focus fallback (`src/app.rs`) |
| D4 | font fallback could pass the unvalidated literal `"monospace"` and panic in `resolve_font` | `choose_font_family` returns only a name present in `all_font_names()`, else the gpui `.ZedMono` alias; the chosen family is logged (`src/app.rs`) |
| D5 | existing panes never re-attached after a reconnect | `ConnectionStateChanged(Connected)` calls `reattach_missing_panes`, and the data plane emits a `data_plane_connected` `ServerEvent` on every (re)connect so a data-plane-only drop also re-attaches; `Disconnected`/`Error` invalidate dead streams, and a `block.attach` error pops its FIFO slot (`src/api/mod.rs`, `src/app.rs`) |
| D6 | drain loop was unbounded per frame; no shutdown | drain capped at `MAX_EVENTS_PER_FRAME = 256`; `on_app_quit` sends `ApiCommand::Shutdown` (`src/app.rs`) |
| D7 | pane-header icon click also triggered the pane card (`AttachBlock`/focus churn on Close) | icon-button listeners call `cx.stop_propagation()` (gpui 0.2.2 mouse bubble) (`src/app.rs`) |
| D8 | invented chrome literals were scattered, and the picker empty row duplicated the measured 35 px token | all moved to `theme.rs` (`*_ORIGINAL`) or `geometry::invented`; the picker empty row now uses `geometry::v1::SESSION_PICKER_LIST_ROW`. No rendered value changed (`src/theme.rs`, `src/geometry.rs`, `src/app.rs`, `src/ui/terminal.rs`) |
| D9 | docs over/under-stated provenance and help text | `geometry.rs` header + `Measured` render-px constants; `lib.rs` comment; `--help` now prints the keybindings (`src/geometry.rs`, `src/lib.rs`, `src/main.rs`) |
| D10 | evidence claimed a tab frame that was byte-identical to another; scratch dirs left behind | `tabs` is captured on tab 2 (md5 `a35faa30…`, distinct from `switch-tab3` `982b4b46…`); `run-ui-002.sh` removes its scratch directory; these notes corrected (`evidence/run-ui-002.sh`, this file) |

---

## 4. Screenshot discrepancy list (ranked by impact)

Reference stills are local-only analysis frames (not committed). Comparison is
quantitative (pixel scans) plus the reference-measured tokens.

### A. Layout / hierarchy (highest impact)

1. **Vertical tabs are absent.** Unmeasurable geometry; documented gap.
2. **Command palette is absent.** Optional in the brief.
3. **Session pill / remote-host chip is absent.** No remote sessions.
4. **Connection state is a banner.** The reference showed nothing (Q9); only visible in
   the non-`Connected` states, so the baseline matches the reference.
5. **`+` and tab strip match** (1 px separator at y = 40; pill at x = 128..332,
   y = 4..33). Measured from `ui-002-workspace.png`.

### B. Proportions

6. **Window is the reference outer size 1099 × 706**, but it is a square Wayland
   toplevel; the reference had macOS rounded corners + shadow.
7. **Pane header 41 px** is a V4 inference; V1's is unmeasurable.
8. **Pill corner radius 9 px** is low confidence; compression softens the reference
   corners.

### C. Colours / contrast

9. **Measured tokens match within ±1–2/channel** at the pill (20,41,62 vs 19,41,62),
   top bar (5,27,46 vs 4,27,46), separator (30,48,66 vs 30,49,66), split borders
   (31,81,114 / 17,53,71 vs 31,82,115 / 17,54,71), ramps (14,29,40 vs 12,28,40).
10. **~3 % desktop bleed** remains through the surface (base ramp pixel +7 in text
    areas); this is a gpui/Hyprland alpha residual, not a token error. Quantified from
    `ui-002-resize-before.png`; see §6.
11. **ANSI/cursor/scrollbar/banner colours** are Relayable-original (non-reference).

### D. Typography

12. **Font family is unknown** (unmeasurable). JetBrainsMono Nerd Font 13 px is a
    Relayable choice; the reference glyph shapes/widths are unverified.
13. **Type sizes are unmeasured**; 12 px UI labels are a proposal.

### E. Microspacing

14. **Separator 1 px vs the 1.5 px token midpoint** (evidence-supported deviation).
15. **Search-field height 28.5 px / row 35 px** are picker range midpoints.

---

## 5. Supported fidelity level

- **High confidence:** window size; top-bar height + separator; tab pill width/height/
  x-position/fill; inactive divider; inter-pane gap; terminal row pitch; split borders/
  surfaces; baseline grid (140×36 at 1099×706).
- **Medium:** pane header height (V4-inferred); background ramps (translucency caveat);
  session-picker size; split ratio; border widths.
- **Unverified / not measurable:** exact font family and type sizes; cursor colour/style
  beyond "block"; ANSI palette; scrollbar colours; every connecting/reconnecting/
  disconnected/error visual; vertical-tabs geometry; command-palette geometry; remote
  host; prompt badge; hover appearances.

**Explicitly not claimed:** pixel-perfect matching. The measured tokens match to within
video-compression tolerances, but fonts, cursor colours, the ANSI palette, and all
connection states are unmeasured and are Relayable-original.

---

## 6. Validation performed

Commands (from `apps/linux`), all exit 0:

```
cargo fmt --check
cargo clippy --all-targets --locked
cargo test --locked          # 58 passed / 0 failed (53 baseline + 5 Stage-D tests)
cargo build --locked
```

From the repository root: `cargo check --workspace --locked` → 0. (Raw transcript:
`evidence/ui-002-validation.log`.)

Live run (`evidence/run-ui-002.sh`, full transcript `ui-002-run.log`):

- Server foreground on `/tmp/relayable-ui002-ui/relayable.sock`, `SHELL=/bin/bash`.
- Hyprland rule pinned the window; read back from `hyprctl clients -j`:
  `class=relayable`, `floating=true`, **`at 10,40`, `size 1099x706`** (every capture
  logged `geometry=10,40 1099x706`).
- App log: `window bounds 1099x706 at (0, 0) scale_factor=1`; `font chosen="JetBrainsMono Nerd Font" (of 273 installed families)`, cell width 7.800, row pitch 17.25, size 13.
- Data-plane handshake logged the advertised limits:
  `server advertised max_concurrent_streams=8 credit_window_bytes=262144`.
- Screenshots (real dimensions from `identify`): `ui-002-workspace.png` 1099×706,
  `-tabs.png` 1099×706, `-tabs-one-osc.png` 1099×706, `-switch-tab1.png` 1099×706,
  `-switch-tab3.png` 1099×706, `-split.png` 1099×706, **`-picker.png` 1099×706**,
  `-smaller.png` 900×600, `-larger.png` 1400×900, `-mock.png` 1099×706,
  **`-npanes.png` 1099×706**, **`-npanes-live.png` 1099×706**,
  **`-npanes-closed.png` 1099×706**, `-resize-before.png`
  1099×706, `-resize-after.png` 900×600; each (except the resizes) with a 1920×1080
  `-fullscreen` companion.
- No `relayable-ui: error` or `disconnected` lines in the final run.
- A separate reconnect run (`evidence/ui-002-reconnect.log`) forced real per-connection
  idle timeouts (server started at 4 s) to exercise D5 without restarting the server; see
  the D5 note in §7.

Interaction proofs (from the transcript):

- **Tab pill moves:** at row y = 12 the active pill fill spans x = 128..332 (tab 1),
  333..537 (tab 2, the `tabs` capture) and 538..742 (tab 3); each run is 205 px wide,
  matching `ACTIVE_TAB_PILL_WIDTH`. So `switch-tab1` → 128..332, `tabs` → 333..537,
  `switch-tab3`/`split` → 538..742. `tabs` is now captured on **tab 2** (harness
  `Ctrl+2`) so it is a genuinely different frame: md5
  `tabs=a35faa301d35e76aadd1fcf10e39e4a1` vs `switch-tab1=e50c8e3e82215bd7e9600f7cfedf6542`
  vs `switch-tab3=982b4b46cc609b3f6de871947480697e`. (An earlier revision captured
  `tabs` immediately after creating tab 3, making it byte-identical to `switch-tab3`;
  that pairing is not used as proof here.)
- **Terminal content changes:** `switch-tab1` (typed `stty size`/OSC output) and
  `switch-tab3` (no typed output) differ: ImageMagick `compare -metric AE` reports
  `1925.23 (0.0024813)` on the new captures, i.e. the two frames are genuinely
  different, not merely a re-saved pill.
- **Three concurrent live panes (D1 + D2):** one row of three panes at 1099×706. Each
  pane's logged grid is `cols=45 rows=35 pane_px=355x619` (content px; the flex share is
  360 px with a 9.5 px gap each side, so 3×360 + 2×9.5 = 1099). At two panes the share is
  539 px content (`cols=69 rows=35`). `ui-002-npanes.png` shows `PANE-ONE-MARKER`,
  `PANE-TWO-MARKER` and `PANE-THREE-MARKER` (OCR-confirmed) recorded by **streams 1, 2
  and 3**. Then pane 1 is focused again and typed into (`echo PANE-ONE-SECOND`), which
  produced `output stream_id=1 … total=470` **without any re-attach** — proof it stayed
  live while unfocused. `ui-002-npanes-live.png` shows the new `PANE-ONE-SECOND` line in
  pane 1 while panes 2 and 3 retain their markers. Per-stream output byte totals across
  the scenario (app log): stream 1 `62→470`, stream 2 `27→422`, stream 3 `62→270` bytes.
- **Close keeps the survivors attached (D7 close path):** with pane 1 focused,
  `Ctrl+Shift+W` removed it; the log then shows the remaining panes resized from the
  three-pane share to the two-pane share (`stream_id=2/3 cols=69 rows=35
  pane_px=539x619`) and still producing output (`stream_id=2 … total=422`,
  `stream_id=3 … total=270`) with **no re-attach**. `ui-002-npanes-closed.png` shows
  exactly panes 2 and 3 (`PANE-TWO-MARKER`, `PANE-THREE-MARKER`, OCR-confirmed). The
  mouse double-dispatch fix itself (`cx.stop_propagation()` in the icon-button listener)
  is code-verified against the gpui 0.2.2 mouse bubble phase; this host has no mouse
  injection tool to click the button in the live run.
- **Live program/cwd:** `pane title/cwd program=Some("Relayable Demo")
  cwd=Some("/home/sulthonuladib")` after the OSC 2/7 script, and the shell's own OSC 2
  title `sulthonuladib@falcon:~`; the header/tab derive from these.
- **Split:** focused border rgb(31,81,114) at x = 554, unfocused rgb(17,53,71) at x = 1;
  focused header rgb(1,36,53), unfocused rgb(6,27,38); gap x = 545..553 ≈ 9 px.
- **Resize:** baseline `terminal.resize … cols=140 rows=36`, accepted; split
  `cols=69 rows=35`; 900×600 → `115×30`; 1400×900 → `179×47`; runtime shrink to 900×600
  sent `stream_id=1 cols=115 rows=30` and was accepted (`resize-after.png` 900×600).
- **Mock:** `ui-002-mock.png` bottom strip sampled rgb(136,107,27) ≈
  `MOCK_BANNER_BG` rgb(138,109,26).
- **Session picker:** `ui-002-picker.png` 1099×706 (opened with `Ctrl+Shift+S`)
  samples `SESSION_PICKER_BG` (14,28,39), search field (31,53,70) ≈
  `PALETTE_SEARCH_FIELD`, selected row (67,154,252) ≈ `SESSION_PICKER_SELECTED_ROW`
  rgb(68,156,255), border (79,96,110) ≈ `SESSION_PICKER_BORDER` rgb(80,97,111).

Teardown: the runtime `hl.window_rule` is disabled via a retained Lua global
(`RELAYABLE_UI002_RULE:set_enabled(false)`); the server was stopped, the scratch socket
and its directory removed (`scratch_removed=yes`), with
`ps -ef | grep -E 'relayable-server|relayable($| )'` and `ls -d /tmp/relayable-ui002*`
both empty. Runtime rules are session-only and are not written to disk.

Compositor contamination checks: the window was positioned at y = 40 to clear the
30 px `omarchy-bar`; window-only captures use the `relayable` client's own `at`/`size`
(no bar). No `omarchy-keyboard-panel` layer appeared over the captures in any run
(checked by sampling the window region). Full-screen captures intentionally include the
bar and are supplementary.

---

## 7. Remaining risks / not done

- **~3 % desktop bleed** through gpui's Wayland surface (gpui 0.2.2 marks client-side
  decorations transparent; Hyprland forces client-side). Screenshots are sampled and the
  bleed is quantified, not hidden.
- **Vertical tabs, command palette, remote host** not implemented (gaps documented).
- **Session picker** lists all server sessions but can only switch to sessions this app
  created (no `block_id` is exposed by the frozen client API for arbitrary sessions).
- **Reconnect re-attach (D5):** proven live in `evidence/ui-002-reconnect.log` with a
  server started at `RELAYABLE_IDLE_TIMEOUT_MILLIS=4000`/`RELAYABLE_HEARTBEAT_INTERVAL_MILLIS=1000`,
  which forces real per-connection idle drops without restarting the server (sessions
  survive). After each `disconnected reason=data connection lost`, the app logs
  `block.attached` for **every** pane again (`stream_id=1 snapshot_bytes=348` and
  `stream_id=2 snapshot_bytes=286` on the two-pane reconnect) and the post-reconnect
  commands produce output on both streams (`stream_id=1 … total=386`,
  `stream_id=2 … total=324`). This is the data-plane-only case, where the control plane
  stays connected and no `ConnectionStateChanged(Connected)` arrives; the
  `data_plane_connected` `ServerEvent` is what triggers the re-attach. The “only panes
  without a live stream re-attach” filter is also unit-tested
  (`reconnect_targets_only_panes_without_a_live_stream`).
- **Input:** printable keys, control bytes, arrows/Home/End/Page keys, Alt/Meta prefixes;
  no bracketed paste, mouse reporting, IME, or text selection yet.
- **No README update** (a later stage owns `apps/linux/README.md`).
