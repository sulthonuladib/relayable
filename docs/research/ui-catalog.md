# Relayable UI reference catalogue — Phase 0, Wave 2

Status: research artifact, **not** a UI spec for uncharted screens. This catalogue covers only what
was actually observed in the five reference videos.

## Evidence basis and conventions

- Primary source: `docs/research/video-review.md` (commit `f067f92`), which records the frame-cited
  visual review of all five videos. Every video-derived claim below cites a video timestamp and/or an
  evidence frame path from that file.
- Supplemental source: `docs/research/gorex-review.md` (commit `a7e14a3`), a behaviour-only review of
  the third-party GoRex clone. There is **no GoRex licence**; anything from it is tagged
  `[clone-derived]` and is never treated as original-product evidence.
- Frames live under `docs/research/evidence/video1..video5/**` (332 JPEGs, gitignored by
  `.gitignore:11`; read-only for this task, never staged). Paths below are relative to the repo root.
- Video 2 (architecture) and Video 3 (memory benchmarks) contain **no product UI** and are not UI
  evidence. Video numbers are V1…V5 as in `video-review.md` §1.1.
- Confidence labels: **high** (legible frame + consistent across frames), **medium-high**,
  **medium**, **low** (`video-review.md` uses the same vocabulary).
- Geometry: no pixel-exact measurement was possible from 480p JPEGs (`video-review.md` §1.3). Every
  proportion/number below is an estimate read off the cited frame; those are marked
  `[low-confidence estimate, 480p]`. Rounded values are **not** implementation constants.
- State categories: for each screen the entry lists empty / loading / error / disconnected / focused /
  selected. `not observed` means the evidence does not show that state; nothing is invented for it.
- Implementation paths are **proposed Phase-1 locations** subject to the lead's architecture
  decision; nothing under `apps/` exists yet. Screenshot-test names are the planned visual-diff IDs.
- `full.md` §0.1: the wordmarks/chrome strings below ("Superlogical", "Rex", `rex …`) are quoted
  research evidence only. Relayable UI must use Relayable branding and independently created assets.

## Entry coverage at a glance

| # | Entry | Video + timestamp | Evidence frame(s) |
|---|---|---|---|
| 1 | `desktop/workspace-default` | V1 0:24 | `video1/still-0024.jpg` |
| 2 | `desktop/session-pill` | V1 2:22; V4 0:58, 4:11, 4:49; V5 7:04 | `video1/still-0222.jpg`, `video4/up-0058.jpg`, `video4/up-0411.jpg`, `video4/up-0449.jpg`, `video5/up-0704.jpg` |
| 3 | `desktop/tab-strip` | V1 0:42–0:47, 0:55–0:58, 1:33–1:40, 2:22; V4 0:58, 4:11, 4:49; V5 7:04 | `video1/tabs-0047.jpg`, `video1/tabs-0133.jpg`, `video1/topbar-0058.jpg`, `video1/still-0222.jpg`, `video4/up-0058.jpg`, `video4/up-0411.jpg`, `video4/up-0449.jpg`, `video5/up-0704.jpg` |
| 4 | `desktop/pane-header` | V1 0:42–0:47, 1:33; V4 0:58, 4:49 | `video1/tabs-0047.jpg`, `video1/tabs-0133.jpg`, `video4/up-0058.jpg`, `video4/up-0449.jpg` |
| 5 | `desktop/terminal-body` | V1 0:47, 0:55–0:58, 1:33–1:40, 2:03 | `video1/still-0047.jpg`, `video1/still-0058.jpg`, `video1/still-0133.jpg`, `video1/still-0203.jpg` |
| 6 | `desktop/session-picker` (session switcher palette) | V1 1:40 | `video1/topbar-0140.jpg`, `video1/still-0140.jpg` |
| 7 | `desktop/vertical-tabs` | V1 2:22 | `video1/still-0222.jpg`, `video1/topbar-0222.jpg` |
| 8 | `desktop/command-palette` | V4 0:30 | `video4/up-0030.jpg`, `video4/still-0030.jpg` |
| 9 | `desktop/split-layout` | V4 0:58, 4:49 | `video4/up-0058.jpg`, `video4/still-0058.jpg`, `video4/up-0449.jpg` |
| 10 | `desktop/remote-host` | V4 0:58, 4:11, 4:49 | `video4/up-0058.jpg`, `video4/up-0411.jpg`, `video4/up-0449.jpg` |
| 11 | `desktop/reconnect-after-quit` | V4 3:59 → 4:11 | `video4/still-0359.jpg`, `video4/up-0411.jpg` |
| 12 | `desktop/cli-client-terminal` | V5 1:04–2:28 (on-screen 14:59–15:00), 4:25, 5:35 | `video5/still-0104.jpg`, `video5/up-0425.jpg`, `video5/up-0535.jpg` |
| 13 | `desktop/cli-help` | V5 7:04 | `video5/still-0704.jpg`, `video5/up-0704.jpg` |

`desktop/disconnected` as an in-app state, `desktop/settings`, and `desktop/inspector` were **not
observed**; see Coverage and limitations at the end.

---

### desktop/workspace-default

- **Source:** V1 0:24; evidence `docs/research/evidence/video1/still-0024.jpg` (also `still-0013.jpg`
  for launch-from-Dock).
- **Component hierarchy and geometry (observed):** single macOS window, full-window terminal.
  Top bar at window top containing (left→right) macOS traffic lights, then the tab strip, then `+`
  at the far right. Below the top bar: per-pane header strip, then terminal body filling the rest of
  the window. No footer/status bar and no sidebar in this frame.
  - Top bar height ≈ 3.5–4.5% of window height `[low-confidence estimate, 480p]`.
  - Terminal body occupies the remainder; one pane only.
- **Interaction/state transitions:** app launched from Dock (V1 0:13, `still-0013.jpg`) → interactive
  prompt (0:24). Counter keeps running while the client quits/relaunches (V1 0:52–1:30, spoken;
  persistence is not a visual chrome change).
- **Exact labels:** macOS login banner `Last login: Fri 28 Aug 14:23:18 on ttys003` (frame, high);
  right-aligned date line `Fri 28 Aug 14:23:19 -07:00` (frame, high). No product-specific labels.
- **States:**
  - empty: `not observed`; loading: `not observed` (launch is a single frame, no spinner shown);
  - error: `not observed`; disconnected: `not observed`;
  - focused: terminal cursor/prompt visible in the single pane (high);
  - selected: only one pane/tab at this timestamp — selected treatment not distinguishable.
- **Confidence/unresolved:** high for structure; medium-low for proportions. Unresolved: exact window
  size, window-scale/local font, background colour value.
- **Relayable mapping:** planned `apps/linux/src/ui/workspace.rs` (proposed); test
  `screenshot_workspace_default`.
- **GoRex cross-reference:** GoRex runs at 1000×620 min 560×340 with a 44 px title bar
  `[clone-derived]` — do not adopt these as REX values; Video 1 does not show a window size.

### desktop/session-pill

- **Source:** V1 2:22 (`video1/still-0222.jpg`); V4 0:58, 4:11, 4:49 (`video4/up-0058.jpg`,
  `video4/up-0411.jpg`, `video4/up-0449.jpg`); V5 7:04 (`video5/up-0704.jpg`).
- **Component hierarchy and geometry (observed):** a chip at the top-left of the window, immediately
  right of the traffic lights, preceding the tab strip. It has a leading glyph (a panel/window icon)
  and text; in V4 the text has a smaller second line underneath showing the host endpoint or
  `This Mac`. Occupies roughly the left 8–13% of the top bar `[low-confidence estimate, 480p]`.
- **Interaction/state transitions:** V4: host chip shows the current session and target
  (`drifting-cedar` → renamed `demo`/`Demo`), then after reconnect shows the local session
  `cosmic-summit` + `This Mac`; V4 0:30 the palette lists `Change Session` with current value
  `cosmic-summit` (this is the session switcher entry point). V1 2:22 shows `Demo`.
- **Exact labels:** `Demo` (V1 2:22, V4 4:49); `drifting-cedar` and `hill.hamster-dinosaur.ts.net`
  (V4 0:58); `cosmic-summit` and `This Mac` (V4 4:11); `Ghostty` (V5 7:04 — legible; role
  unconfirmed, possibly window/session title, medium confidence).
- **States:**
  - empty/loading/error/disconnected: `not observed` for the pill itself;
  - focused: no focus ring captured — `not observed`;
  - selected: chip is the identity indicator, not a selected list row; selected-session state is
    observed in `desktop/session-picker` (`✓ Demo`).
- **Confidence/unresolved:** high for presence and labels; medium for V5 `Ghostty` role; the
  two-line endpoint subtitle is high (V4 frames). Unresolved: exact chip sizing/padding; whether the
  leading glyph is a menu trigger.
- **Relayable mapping:** planned `apps/linux/src/ui/session_chip.rs` (proposed); test
  `screenshot_session_pill_local` and `screenshot_session_pill_remote`.
- **GoRex cross-reference:** GoRex has a host chip (`falcon` / `Linux`) plus a host popover with
  `Model/Chip/Memory/System/User/Server/Sessions` rows `[clone-derived]`. The popover was not seen in
  REX footage — **discrepancy: treat the popover as clone-derived only.**

### desktop/tab-strip

- **Source:** V1 0:42–0:47 (`video1/tabs-0047.jpg`), 0:55–0:58 (`video1/topbar-0058.jpg`),
  1:33–1:40 (`video1/tabs-0133.jpg`), 2:22 (`video1/still-0222.jpg`); V4 0:58, 4:11, 4:49
  (`video4/up-0058.jpg`, `video4/up-0411.jpg`, `video4/up-0449.jpg`); V5 7:04
  (`video5/up-0704.jpg`).
- **Component hierarchy and geometry (observed):** horizontal row of rounded tab segments inside the
  top bar, each with a leading `>_` terminal glyph and a label; `+` control at the right end; large
  empty draggable region after `+` in V1 `tabs-0047`. Active tab is a lighter rounded pill; inactive
  tabs are flat/darker. One observed tab count per frame: 3 (V1 0:47: `btop`, `htop`, `~`), 4
  (V1 ~1:33: `btop`, `htop`, two `~/Documents/ghostty` shells with numeric suffix), 4 in V1 2:22, 1
  in V4 0:58/4:11/4:49, 1 in V5 7:04.
  - Tab pill height ≈ 35–40 px at 480p upscale `[low-confidence estimate, 480p]`; pill radius fully
    rounded in `tabs-0047`.
  - In V1 0:47 the active tab is the third (`~`); in V4 the single tab is active.
- **Interaction/state transitions:** switching tabs updates the pane content; duplicate cwd creates
  a `-1`-suffixed label (V1 ~1:33, medium-high from a 3x crop — write `-1` as observed convention,
  not a rule); rename via `Rename Tab` palette command (V4 0:30); `Close Tab` palette command
  (V4 0:30). Number-key switching implied by `⌘1` key overlay (V4 4:11) and `⌘1`/`⌘2` glyph
  sightings (video-review §8.3).
- **Exact labels:** `btop`, `htop`, `~`, `~/Documents/ghostty` (+ `-1` suffix duplicate), `nvim`
  (V1); `mitchellh@hill: ~` (V4 remote); `~/Documents/ghostty` (V4 4:11, V5).
- **States:**
  - empty (zero tabs): `not observed`; loading: `not observed`; error: `not observed`;
    disconnected: `not observed`;
  - focused: the terminal focus in the active tab is visible (cursor); active-tab focus ring
    `not observed`;
  - selected: active tab lighter pill — observed (high, V1 0:47; V4).
- **Confidence/unresolved:** high for content/presence; medium for exact pill geometry and for the
  duplicate-`-1` rule. Unresolved: label truncation, hover close button (not seen), overflow
  behaviour, Linux keybindings for tab switching.
- **Relayable mapping:** planned `apps/linux/src/ui/tab_strip.rs` (proposed); tests
  `screenshot_tab_strip_horizontal`, `screenshot_tab_strip_active_selected`.
- **GoRex cross-reference:** GoRex tabs are pills in a rounded track with 14 px radius, hover close
  button, drag reorder, double-click rename, attention dot `[clone-derived]`. The video only confirms
  pill shape, label content, `+`, and lighter active state; extra GoRex affordances are clone-derived.

### desktop/pane-header

- **Source:** V1 0:42–0:47 (`video1/tabs-0047.jpg`), 1:33 (`video1/tabs-0133.jpg`); V4 0:58
  (`video4/up-0058.jpg`), 4:49 (`video4/up-0449.jpg`).
- **Component hierarchy and geometry (observed):** thin strip immediately below the tab strip and
  above the terminal body. Left: a small program/terminal icon plus the pane's cwd or program name
  (`~`, `~/Documents/ghostty-nvim`-style per video-review §2.2; V4 shows `shell`). Right side of the
  **focused** pane: four icon buttons (split-right, split-down, zoom, close `×`); the unfocused pane's
  header shows no buttons (V4 `up-0058`, `up-0449`). Header height ≈ 30–36 px at 480p
  `[low-confidence estimate, 480p]`.
- **Interaction/state transitions:** focus moves between split panes (V4 0:58 two panes; only left
  header shows buttons). In V1 the pane header label follows the pane's program/cwd
  (`btop`/`nvim` context).
- **Exact labels:** `~` (V1 0:47), `shell` (V4 0:58, 4:49); directory strings from V1 1:33 are
  medium confidence (3x crop).
- **States:**
  - empty/loading/error/disconnected: `not observed`;
  - focused: buttons visible + lighter pane card (V4, high);
  - selected: pane selection is the focus state; no separate selected appearance observed.
- **Confidence/unresolved:** high for structure and focused/unfocused difference; medium for exact
  labels beyond `~`/`shell`. Unresolved: icon-by-icon button identity/order (read as split-right,
  split-down, zoom, close from 480p — low-medium), header button hover state.
- **Relayable mapping:** planned `apps/linux/src/ui/pane_header.rs` (proposed); tests
  `screenshot_pane_header_focused`, `screenshot_pane_header_unfocused`.
- **GoRex cross-reference:** GoRex pane header = 33 px, glyph 14.5, name 12.5/600 + directory
  12.5/500 @86% alpha, four 26×26 hover/focus buttons `[clone-derived]`. Video confirms the four
  buttons appear on the focused pane; numeric sizes are clone-derived, not REX measurements.

### desktop/terminal-body

- **Source:** V1 0:47 (`video1/still-0047.jpg`), 0:55–0:58 (`video1/still-0058.jpg`), 1:33–1:40
  (`video1/still-0133.jpg`), 2:03 (`video1/still-0203.jpg`).
- **Component hierarchy and geometry (observed):** fills the pane below the pane header, edge to
  edge, dark navy background; monospace text with ANSI prompt colours; block cursor; right-aligned
  clock/date line inside the terminal (`Fri 28 Aug 14:23:19 -07:00`). V1 2:03 shows a long
  scrollback column with a native scrollbar on the right edge. Neovim (V1 1:33–1:40) draws its own
  full-pane UI (line numbers, `NORMAL` status line) — the product chrome does not change around it.
- **Interaction/state transitions:** typed `cd ~/Documents/ghostty` (0:47); counter output during
  client restart (0:55–1:30); native scrollback scrolling (2:03, spoken "native scrolling").
- **Exact labels:** `Last login: …` banner; typed command `cd ~/Documents/ghostty`; counter digits;
  nvim `NORMAL` status line; prompt glyph `~  >` and V4-style prompt `~/D/ghostty` with a
  purple badge (`36` in V4, `31` in V5) — the **badge meaning is unknown** (duration/exit/other).
- **States:**
  - empty: `not observed`; loading: **not observed** — Video 2 describes background scrollback
    streaming that "can show a loading state" while scrolling back, but no such UI frame exists;
  - error: `not observed`; disconnected: `not observed`;
  - focused: block cursor in the focused pane (high);
  - selected: terminal text selection not captured — `not observed`; native scrollbar visible (2:03,
    medium-high).
- **Confidence/unresolved:** high for content and scrollbar; medium for prompt badge semantics;
  background colours unmeasured. Unresolved: terminal theme/palette values, cursor style, scrollback
  loading visual, selection colour.
- **Relayable mapping:** planned `apps/linux/src/ui/terminal.rs` (proposed); tests
  `screenshot_terminal_default`, `screenshot_terminal_scrollback`.
- **GoRex cross-reference:** GoRex terminal font JetBrains Mono 11.6 px, dark bg `#1e1e22`
  `[clone-derived]` — informational only; Video 1's colours were not colour-sampled.

### desktop/session-picker (session switcher palette)

- **Source:** V1 1:40 (`video1/topbar-0140.jpg`, `video1/still-0140.jpg`).
- **Component hierarchy and geometry (observed):** small rounded dark popup over the terminal, near
  the top of the window. Top: search field with placeholder `Filter or create...`. Below: list rows;
  first row highlighted with a leading checkmark: `✓ Demo`. Further rows include `New Session` and
  `Add Remote Host...` (list partly occluded by the cursor). Popup width is a small minority of the
  window width (well under half) `[low-confidence estimate, 480p]`.
- **Interaction/state transitions:** opened over the terminal; current session marked with `✓`;
  `New Session` creates a session (Video 1 1:36–1:52, spoken: session named `work`, then switched
  back to `ghosty`); `Add Remote Host...` is the remote entry point reused in V4.
- **Exact labels:** `Filter or create...` (high), `✓ Demo` (high), `New Session`, `Add Remote
  Host...` (medium-high; occluded cursor). No shortcut badges legible in this frame.
- **States:**
  - empty session list: `not observed`; loading: `not observed`; error: `not observed`;
    disconnected: `not observed`;
  - focused: cursor visible in the search field (high);
  - selected: `Demo` row highlighted with checkmark (high).
- **Confidence/unresolved:** high for placeholder/rows; medium for exact popup geometry and whether
  extra rows exist. Unresolved: popup width/position rules, keyboard shortcuts, "create" behaviour.
- **Relayable mapping:** planned `apps/linux/src/ui/session_picker.rs` (proposed); test
  `screenshot_session_picker_open`.
- **GoRex cross-reference:** GoRex has one combined command palette, not a separate session picker
  `[clone-derived]`. **Discrepancy:** REX shows a distinct session switcher (`Filter or create...`)
  separate from the command palette (`Search commands...`); prefer the video.

### desktop/vertical-tabs

- **Source:** V1 2:22 (`video1/still-0222.jpg`, `video1/topbar-0222.jpg`).
- **Component hierarchy and geometry (observed):** left vertical tab sidebar with `>_`-icon tab rows;
  window no longer fills the screen (macOS desktop visible behind). A compact palette/menu is open
  near the top-left with two legible rows: `Switch to Vertical Tabs` with shortcut `⇧⌘T` and
  `Add Remote Host...` with shortcut `⇧⌘R`. Sidebar rows are small (high-info-density, ≤ one text
  line each) `[low-confidence estimate, 480p]`. Horizontal tab strip still visible in the same frame.
- **Interaction/state transitions:** action `Switch to Vertical Tabs` toggles the sidebar layout
  (invoked in this frame); the spoken context (V1 2:20–2:30) is "there are vertical tabs… I still
  prefer horizontal, but they are there". Whether this palette is the session switcher, a context
  menu, or the command palette is unresolved (see `open-questions.md` Q3).
- **Exact labels:** `Switch to Vertical Tabs` + `⇧⌘T`, `Add Remote Host...` + `⇧⌘R` (medium-high,
  480p palette rows); sidebar row labels not legible (low).
- **States:**
  - empty/loading/error/disconnected: `not observed`;
  - focused: active sidebar row treatment not confidently observed (low);
  - selected: if the horizontal active-pill convention carries over, the active vertical row is
    expected lighter — `not observed` to high confidence.
- **Confidence/unresolved:** medium-high for the two palette rows; low for sidebar geometry/row
  states. Unresolved: sidebar width, icon/label layout, per-row active/hover/close affordances,
  keyboard toggle binding on Linux.
- **Relayable mapping:** planned `apps/linux/src/ui/vertical_tabs.rs` (proposed); test
  `screenshot_vertical_tabs`.
- **GoRex cross-reference:** GoRex has no vertical-tabs mode `[clone-derived]` — the video is the
  only evidence for this state.

### desktop/command-palette

- **Source:** V4 0:30 (`video4/up-0030.jpg`, `video4/still-0030.jpg`).
- **Component hierarchy and geometry (observed):** centred floating panel over the workspace,
  vertical above the bottom of the window. Top: search field with magnifier glyph and placeholder
  `Search commands...`. Then a scrollable command list, with a group header `Pane` before the pane
  commands. Rows right-align shortcut badges; the selected row is filled and shows the current value
  plus a trailing chevron (`Change Session` → `cosmic-summit` → `>`).
  - Panel width ≈ 28% of window width (~390 px in a 1448 px-wide capture) `[low-confidence estimate,
    480p]`; row pitch ≈ 29 px `[low-confidence estimate, 480p]`; panel top ≈ 80–90 px below the
    window top `[low-confidence estimate, 480p]`; roughly 18 rows visible including the `Pane` header.
- **Interaction/state transitions:** opened with `⇧⌘P` (key overlay `⇧⌘P` shown bottom-centre in the
  same frame, high). Typing filters (search field). Arrow keys move the selected row (inferred from
  fill + chevron, medium). Commands observed in the list include the session, tab, window, and pane
  groups. `Go to Directory...` opens a new terminal in a folder and works locally and remotely
  (V4 4:49, spoken + frame `up-0449.jpg`); new shortcut `command shift G`.
- **Exact labels (all from this frame unless noted):** `Search commands...`; `Change Session`
  (`cosmic-summit`); `Rename Session`; `Rename Tab` (`~/Documents/ghostty`); `Add Remote Host...`;
  `New Session` (`⇧⌘N`); `New Tab` (`⌘T`); `Go to Director...` (sic, truncated in frame) + value
  `…uments/ghostty` + `⇧⌘G`; `New Window` (`⌘N`); `Close Tab` (`⌘W`); `Refresh Sessions` (`⌘R`);
  `Close Session`; `Close Window` (`⇧⌘W`); group header `Pane`; `Split Pane Left` (`◇⌘←`),
  `Split Pane Right` (`◇⌘→`), `Split Pane Up` (`◇⌘↑`). Shortcut glyphs read at 480p are low
  confidence except where `video-review.md` also lists them (§8.3).
- **States:**
  - empty (no results): `not observed`; loading: `not observed`; error: `not observed`;
    disconnected: `not observed`;
  - focused: text cursor in the search field (high);
  - selected: `Change Session` row filled (mint/green in this frame; V1 session picker used blue —
    exact palette unmeasured), with trailing chevron (high).
- **Confidence/unresolved:** high for structure, placeholder, and command names; low-medium for
  geometry and for specific shortcut readings. Unresolved: fuzzy-match behaviour, empty-results
  state, palette colour token, whether `Pane` is the only group.
- **Relayable mapping:** planned `apps/linux/src/ui/command_palette.rs` (proposed); tests
  `screenshot_command_palette_open`, `screenshot_command_palette_selected`.
- **GoRex cross-reference:** GoRex palette placeholder is `Type a command, or the name of a pane…`
  and it mixes pane jump targets with commands, ≈560 px wide, ~28 px rows `[clone-derived]`.
  **Discrepancy:** placeholder and pane-jump rows differ; prefer the video's `Search commands...`.
  GoRex Linux accelerators (`Ctrl+Shift+P`, etc.) are clone-derived conventions, not REX evidence.

### desktop/split-layout

- **Source:** V4 0:58 (`video4/up-0058.jpg`, `video4/still-0058.jpg`), 4:49 (`video4/up-0449.jpg`).
- **Component hierarchy and geometry (observed):** two side-by-side panes of equal width separated
  by a visible gap; each pane has its own pane header (`shell` with icon). Focused pane header has
  the four right-side buttons and a lighter card; the unfocused pane header shows only icon + `shell`
  and a darker card. Created over one connection ("I could open a split and I have two now, and this
  is over the same connection", V4 spoken, 0:58).
  - Gap between panes ≈ 8–12 px at 480p upscale `[low-confidence estimate, 480p]`; split is ~50/50 at
    creation (consistent with the spoken "split" action; exact ratio unmeasured).
  - Pane card corner radius and border present but unmeasured.
- **Interaction/state transitions:** pane content independent (left runs `loginctl list-sessions`;
  right idle prompt). V4 4:49 shows `uname -a` in the left pane and `cd /` in the right on the same
  remote host. Split commands from the palette: `Split Pane Left/Right/Up` (V4 0:30). Focus movement
  between panes is visible as the button/card change; the exact key binding is not shown.
- **Exact labels:** `shell` (both pane headers, high); `loginctl list-sessions` and its output
  (high); `[mitchellh@hill:~]$` prompt (high); `uname -a` → `Linux hill 6.18.48 #1-NixOS SMP
  PREEMPT_DYNAMIC Fri Aug 28 06:22:54 UTC 2026 x86_64 GNU/Linux` (high); `cd /` → `[mitchellh@hill:/]$`
  (high).
- **States:**
  - empty/loading/error/disconnected: `not observed` for the split container;
  - focused: focused pane lighter + buttons (high); unfocused pane darker + no buttons (high);
  - selected: n/a; pane focus is the observed selection model.
- **Confidence/unresolved:** high for side-by-side split, headers, and focus difference; low for
  gap/radius/ratio. Unresolved: split keyboard shortcuts on Linux, resize-drag indicator, divider
  hover, vertical (stacked) split geometry (only `Split Pane Up` exists in the palette; no stacked
  frame captured).
- **Relayable mapping:** planned `apps/linux/src/ui/split_layout.rs` (proposed); tests
  `screenshot_split_horizontal`, `screenshot_split_focus_left`.
- **GoRex cross-reference:** GoRex new split ratio 0.5, divider drag clamp 0.08–0.92, 8 px gaps,
  12 px card radius, 36×3 px divider grab indicator `[clone-derived]`; only the 0.5 starting ratio
  is consistent with what the video implies, and even that is unmeasured from the video.

### desktop/remote-host

- **Source:** V4 0:58 (`video4/up-0058.jpg`), 4:11 (`video4/up-0411.jpg`), 4:49 (`video4/up-0449.jpg`).
- **Component hierarchy and geometry (observed):** the remote session reuses the same workspace,
  tab-strip, and pane chrome. Distinguishing elements: the session pill shows the session name with
  the host endpoint as a second line (`drifting-cedar` / `hill.hamster-dinosaur.ts.net`, later
  `Demo` / `hill.hamster-dinosaur.ts.net`), and the tab label switches to user@host form
  (`mitchellh@hill: ~`). Terminal prompts show the remote account (`[mitchellh@hill:~]$`).
  - Remote add flow: command palette → `Add Remote Host...` (V4 0:30); palette `Change Session`
    value `cosmic-summit`; session auto-named `drifting-cedar` on connect, then renamed `demo`
    (V4 spoken 0:58 + frames).
- **Interaction/state transitions:** connect → remote session appears in session list → split opens
  a second pane over the same connection → CLI identity checks run inside the remote pane. V4 4:11
  frame shows `rex -S hill.hamster-dinosaur.ts.net whoami` →
  `you: mitchellh@github over tailscale via tailnet-serve/login-server from mitchellh-mbp, acting as
  mitchellh`; `rex -S root@… whoami` → `acting as root`;
  `rex -S … session new` → `session:01a082b8-f7a2-78cd-bc19-fb524efd8bad` (all high, legible frame).
- **Exact labels:** endpoint `hill.hamster-dinosaur.ts.net` (high); tab `mitchellh@hill: ~` (high);
  whoami output strings above (high); session UUID `session:01a082b8-f7a2-78cd-bc19-fb524efd8bad`
  (high). CLI command spellings (`-S`, `whoami`, `session new`) are evidence, not Relayable flags.
- **States:**
  - empty (no hosts): `not observed`; loading (connecting): `not observed`;
  - **error (unreachable host / bad auth / protocol mismatch): `not observed` — a genuine gap;**
  - disconnected (in-app): `not observed`; reconnection is covered separately;
  - focused: remote tab/pane focus behaves as local (high);
  - selected: remote session not shown selected in the picker in any captured frame — `not observed`.
- **Confidence/unresolved:** high for the shown flow and labels; **unknown** for transport, Tailscale
  requirement, pairing/credentials, and failure UX (V4 spoken declines to detail the protocol;
  `video-review.md` §5.2). These are out of scope for a visual catalogue but block a faithful
  remote/error UI.
- **Relayable mapping:** planned `apps/linux/src/ui/remote_session.rs` (proposed); tests
  `screenshot_remote_host_connected`, `screenshot_remote_host_error` (latter currently has **no
  reference frame** and must not be claimed faithful).
- **GoRex cross-reference:** GoRex does **not** replicate remote connections (local socket only)
  `[clone-derived]`; it is inadmissible as remote evidence.

### desktop/reconnect-after-quit

- **Source:** V4 3:59 (`video4/still-0359.jpg`, key overlay `⌘Q`) → V4 4:11 (`video4/up-0411.jpg`).
- **Component hierarchy and geometry (observed):** before quit the workspace is full-screen; the
  captured quit frame shows the macOS desktop with no app window (key overlay `⌘Q`). After relaunch
  the window returns with the same tab (`~/Documents/ghostty`), the same pane, and the prior
  terminal scrollback (the `whoami` / `session new` output from earlier in the session is still
  visible in `up-0411.jpg`). Session pill then reads `cosmic-summit` / `This Mac`.
- **Interaction/state transitions:** quit (`⌘Q`) → relaunch → reconnect to the server-side session;
  spoken (3:59) "we reconnect and we go back into the last session we were in… all our sessions are
  still here". The app itself is not visible while "disconnected", so no in-app connection state
  UI is evidenced.
- **Exact labels:** `cosmic-summit`, `This Mac` (high); prior scrollback commands/output (high).
- **States:**
  - loading (reconnect splash/spinner): `not observed`;
  - error (failed reconnect): `not observed`;
  - disconnected: only the app being absent (desktop visible) — **no in-app disconnected chrome was
    observed**;
  - focused/selected: rebuilt identically to `desktop/workspace-default` on return (high).
- **Confidence/unresolved:** high for the lifecycle and state restoration; **no** visual evidence for
  connecting, reconnecting, or failed states. Unresolved: time-to-reconnect visual, whether a
  disconnected banner exists.
- **Relayable mapping:** planned `apps/linux/src/ui/connection.rs` (proposed); test
  `screenshot_reconnect_restored` exists in concept only; disconnected/error visuals remain **open**
  (see `open-questions.md`).
- **GoRex cross-reference:** GoRex persistence UX is a background server process with layout JSON
  `[clone-derived]`; it was not observed across a real app restart in the GoRex run — do not cite it
  as REX-equivalent.

### desktop/cli-client-terminal

- **Source:** V5 1:04–2:28 (on-screen 14:59–15:00; caption/clock misalignment caveat), 4:25, 5:35.
  Frames `video5/still-0104.jpg`, `video5/up-0425.jpg`, `video5/up-0535.jpg`.
- **Component hierarchy and geometry (observed):** the standard workspace chrome is unchanged
  (session pill `Ghostty`, tab `~/Documents/ghostty`, pane header, terminal) and `rex` CLI commands
  run **inside a client terminal**; output appears in the same scrollback and propagates live to the
  GUI (V5 spoken 0:56: "Since I'm in a Rex client"; §6.3: changes are immediately visible on all
  connected clients).
- **Interaction/state transitions:** typed commands produce confirmation lines or tables; a
  `block call` prints pretty JSON; `block close` removes a block; `rex block inspect`/`session
  inspect` print aligned tables. No custom REX chrome appears for CLI output.
- **Exact labels (legible frames):** command prompt `~/D/ghostty` with purple badge `31`;
  `rex session inspect Video` output (`session: session:01a0a1ee-…`, `label: Video`,
  `active-window: window:…`, `windows: 2`, `clients:` rows, `WINDOW/LABEL/ACTIVE/FOCUSED/BLOCK/BLOCKS`
  table; `Window e64939f0`); `rex block list -s Video` table
  (`WINDOW ID / FLAVOR / LABEL / PLACEMENT`, flavor `com.superlogical.terminal.shell`, labels
  `shell`, `nvim`, `sleep`); `block call` JSON `{ "last_exit": { "exit_code": 0 }, "home":
  "/Users/mitchellh" }`. Full help text in entry 13.
- **States:**
  - empty/loading/error/disconnected: `not observed` in this entry's frames (CLI error/exit-code
    states were never exercised on camera);
  - focused: terminal cursor (high);
  - selected: n/a — `not observed`.
- **Confidence/unresolved:** high for shown commands/outputs; medium for `$(which …)` substitution
  glyphs; caption/frame timing is loosely aligned (V5 caveat). Unresolved: exit codes, `--json`,
  env vars, Lua, `api` output — all **unknown from video** (not visual).
- **Relayable mapping:** planned `apps/linux/src/ui/terminal.rs` (same as terminal body); test
  `screenshot_cli_output_tables`.
- **GoRex cross-reference:** GoRex is a UI clone and does not implement the REX CLI semantics
  `[clone-derived]`.

### desktop/cli-help

- **Source:** V5 7:04 (`video5/still-0704.jpg`, `video5/up-0704.jpg`).
- **Component hierarchy and geometry (observed):** the same single-pane workspace (session pill
  `Ghostty`, tab `~/Documents/ghostty`, no palette) with the full `rex` (no arguments) help text
  rendered as plain terminal output: `The multiplexer for all work.` then `Usage:`, `Common
  commands:`, `Layout:`, `Resources:`, `Automation:`, `Server:` sections in aligned two-column form.
- **Interaction/state transitions:** none beyond scrolling; this is the terminal's normal text
  rendering. Long help text exceeds one screen (bottom of list continues below the captured area).
- **Exact labels:** verbatim help text catalogued in `video-review.md` §6.1 (high confidence, fully
  legible): `new/attach/ls, list/run/split/send/send-key/capture/kill`, `focus/zoom/move/swap/
  resize`, `session/window/block/client`, `events/wait/api`, `server/login-server/whoami`.
  `rex` wordmark in help is REX branding evidence only (§0.1).
- **States:** empty/loading/error/disconnected/focused/selected: `not observed` except the visible
  terminal cursor; the help text itself is fully observed (high).
- **Confidence/unresolved:** high (frame fully legible). Unresolved: none visual; flags for
  `send`/`send-key`/`capture` were not legible in the demo.
- **Relayable mapping:** planned `apps/linux/src/ui/terminal.rs`; test
  `screenshot_cli_help_verbatim` (Relayable text must be its own `relayable` help output, not REX's).
- **GoRex cross-reference:** none applicable `[clone-derived]` (different binary/UI).

---

## Coverage and limitations (complete relative to evidence, not a wish list)

Observed product screens/states (13 entries above): workspace, session pill, horizontal tabs, pane
header, terminal body/scrollback, session picker, vertical tabs, command palette, split layout,
remote host, reconnect-after-quit, CLI-in-client, CLI help.

**NOT observed anywhere in the five videos — no catalogue entry is justified for these:**

- `desktop/settings` / appearance preferences (light mode, font size, theme).
- `desktop/inspector`, status panels, notifications/toasts, program-activity indicators.
- Empty session list; connecting/loading spinners; any in-app disconnected/reconnecting banner;
  unreachable-host or auth-error UI; connection failure recovery.
- Hover states, drag-to-resize dividers, tab drag-reorder, tab context menu, rename text field,
  close-button hover.
- Window controls on non-macOS; Linux-native chrome of any kind (all demos are macOS).
- Browser/mobile clients (roadmap in V2 only).
- Scrollback background-loading visual (described in V2 but never shown).
- Vertical (stacked) split geometry — only `Split Pane Up` menu text exists, no frame.
- V5 palette frame: `video-review.md` §2.3 refers to a pink-highlighted palette in V5, but no V5
  palette frame path is catalogued; treat that detail as unverified until a frame is identified.

**Non-UI videos:** V2 (architecture) and V3 (memory benchmarks) contain no product screenshots.
V3's charts and X post are evidence for performance claims only; all figures are **non-reproducible
as published** (missing REX build, toolchain, measurement method, idle/active state — see
`video-review.md` §4.2). They must never be used as UI evidence.

**Discrepancies video vs. GoRex (video wins):**

| Topic | Video evidence | GoRex (`[clone-derived]`) | Resolution |
|---|---|---|---|
| Palette identity | session switcher `Filter or create...` (V1 1:40) is separate from command palette `Search commands...` (V4 0:30) | one combined palette, placeholder `Type a command, or the name of a pane…` | Implement two entry points per video |
| Palette selection colour | V1 blue fill; V4 mint/green fill; V5 pink reported without a captured frame | steel-blue `rgba(108,178,255,0.18)` dark selection | Unresolved (palette token question) |
| Linux shortcuts | not demonstrated (macOS only) | `Ctrl+Shift+…` remaps | Video is silent; GoRex mapping is clone-derived, not REX |
| Pane header buttons | visible on focused pane only (V4) | focused **or hover**; 26×26; specific icons | Video confirms focused-only after the fact; do not infer hover |
| Window background | dark terminal surface | three-stop gradient per theme | Unverified; measure from video if higher-res frames appear |
| Host popover | not shown | `Model/Chip/Memory/…` popover | clone-derived only |
| Light appearance / text sizing | not shown | implemented | clone-derived only |
| Remote connections | V4 remote flow with Tailscale names | explicitly **not replicated** (local socket) | Video is the only remote evidence |
| Window default size | unknown | 1000×620, min 560×340 | Do not adopt as REX value |

**Media hygiene:** all frames under `docs/research/evidence/**` are gitignored analysis artifacts;
none is staged or committed, and none may be redistributed or shipped (`full.md` §2.3, §2.6).
