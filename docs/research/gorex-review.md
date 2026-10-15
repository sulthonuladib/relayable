# GoRex supplemental reference review

> Research artifact for Relayable Phase 0, Wave 1. GoRex is a **supplemental** reference
> (`full.md` §2.6), not the source of truth for Relayable. All statements below are labelled
> either `confirmed-by-running`, `inferred-from-README/source`, or `unknown`.
> **No GoRex source or non-system asset may be copied, vendored, translated, or ported**
> (see §2 License).

## 1. Repository identity

| Field | Value |
|---|---|
| Repository | https://github.com/egoist/gorex |
| Commit reviewed | `b206e67bf0ac422afebc5c9e701a38dd67f2c837` |
| Commit date | 2026-10-08 23:56:12 +0800 |
| Commit subject | `Update MyGo for closing the window from its view` |
| Branch reviewed | default branch, shallow clone of HEAD |
| Date reviewed | 2026-10-15 (UTC) |
| Runtime | Go 1.27 module (`go.mod` requires `go 1.27.1`), MyGo native UI + `mygo/plugins/terminal` (Ghostty libghostty-vt), no webview, no cgo |
| App manifest | `mygo.json` — name `GoRex`, identifier `dev.gorex.app`, version `0.1.0`, copyright string `A replica of Superlogical's Rex, built with MyGo` |
| Build tools | `go tool mygo dev` / `go tool mygo build` (MyGo CLI pinned as a Go tool in `go.mod`); `go test ./...`; `go run ./tools/mkicon` |

There is **no `package.json` and no `Cargo.toml`** in the repository: GoRex is neither Electron
nor Tauri. `mygo.json` is the app-bundle manifest. The declared build entry point is the pinned
Go tool `github.com/egoist/mygo/cmd/mygo`, not npm.

## 2. License verification (mandatory re-check)

Commands and results (run in the scratch clone at the reviewed commit):

```
git ls-files | grep -iE 'licen[cs]e|copying|notice'
  assets/brands/LICENSE.md
  assets/icons/LICENSE

ls -la   # root listing, verbatim summary
  app_test.go  assets/  commands.go  debug.go  docs/  .gitignore  go.mod  go.sum
  host.go  internal/  main.go  mygo.json  programs.go  README.md
  resources/  settings.go  state.go  style.go  tabs.go  tools/  view.go
```

- **No root `LICENSE`, `LICENSE.*`, `COPYING`, or `NOTICE` file exists.**
- `go.mod` (the only dependency manifest) has **no `license` field** — Go modules have no such
  field, and `mygo.json` contains only a `copyright` string, which is **not a license grant**.
- The only license files are asset-scoped:
  - `assets/brands/LICENSE.md` — **CC0 1.0 Universal** (brand assets only).
  - `assets/icons/LICENSE` — **ISC** (Lucide icons; portions of Lucide/Feather MIT).

**Result: no repository-level license found. No license is verified for GoRex source code.**
Therefore, unless a future commit adds a verified license that expressly permits reuse:

- No GoRex source may be copied, vendored, translated, mechanically ported, or used as a
  starting point,
- No GoRex non-system asset (icons, images, fonts, brand art) may be bundled,
- Only **observed behaviour and layout** may be studied, and this review must remain a
  **behaviour catalogue**, not a source port.
- Third-party dependencies (MyGo, libghostty-vt, Lucide, JetBrains Mono, Simple Icons) keep
  their own licenses and notices; reuse of *libraries* is separate from GoRex code reuse.

The single `docs/screenshot.png` in the GoRex repository is a project asset; it must not be
redistributed or committed by Relayable.

## 3. Run attempt and evidence

Environment: Linux `falcon` x86_64, Go 1.27.2 (module requires 1.27.1), GTK-based MyGo
backend, `Xvfb` used as a display. All work was done in the scratch clone and outside the
repository.

| Step | Command | Result |
|---|---|---|
| Clone | `git clone --depth 1 https://github.com/egoist/gorex ~/.cache/relayable-research/gorex` | exit 0 |
| Commit | `git -C … rev-parse HEAD` | `b206e67bf0ac422afebc5c9e701a38dd67f2c837` |
| Build all | `go build ./...` | exit 0 (downloads deps into the Go module cache only) |
| Build binary | `go build -o /tmp/gorex-review-bin .` | exit 0, 20,617,576 bytes |
| Launch (Xvfb, X11) | `Xvfb :96 -screen 0 1440x900x24` + `env -u WAYLAND_DISPLAY GDK_BACKEND=x11 DISPLAY=:96 LIBGL_ALWAYS_SOFTWARE=1 /tmp/gorex-review-bin` | process stayed alive; window rendered; app log: `mygo: native UI draws without the GPU: Unable to create a GL context` |
| Empty Xvfb capture without `GDK_BACKEND=x11` | same but with inherited `WAYLAND_DISPLAY` | unusable captures (two 296-byte uniform blanks; `run-02-ffmpeg.png` is a 6,118-byte black frame with a cursor artifact) — capture was not usable in that mode |

**This is a genuine run of the reference application, not a mock.** No global installs, no
`sudo`, no system configuration changes were made. The GTK/MyGo window is not reproducible on
Xvfb with GPU, so it falls back to a CPU path and reports a GL-context error while still
drawing; this is an environment limitation, not a GoRex defect.

Screenshots captured with the app's own built-in `GOREX_DEBUG` hook (`debug.go`: a `do` file
whose lines run a command title, `shot <name>.png`, `sleep <dur>`, `type <text>`, `focus`,
`tab`, `size`; `shot` writes `win.CapturePage()` output). This is source-visible test
instrumentation, used here only to drive and capture the running app.

**Local-only evidence folder** (outside the repository, never committed):
`~/.cache/relayable-research/evidence/gorex/`

| File | Content |
|---|---|
| `run-05-x11.png` | full-screen root capture, 1440×900: the running window with the GTK menu bar (`Shell Edit View Tabs Window`), host chip, tab strip, pane card, prompt `sulthonuladib@falcon ⇒` |
| `02-split-right.png` | app window, 1000×593, after `Split Right` |
| `03-split-down.png` | app window after a second split (`Split Down` in the focused pane) |
| `04-zoom.png` | app window with the focused pane zoomed |
| `05-unzoom.png` | app window after un-zooming (same tree as `03`) |
| `06-palette.png` | app window with the command palette open over split panes |
| `06b-palette-root.png` | root capture of the palette state |
| `run-01-welcome.png`, `run-02.png` | 296-byte uniform blank captures from the first (Wayland-inherited) attempts — kept only as negative evidence |
| `run-02-ffmpeg.png` | 6,118-byte black frame with a cursor artifact from the same attempts — not usable as reference evidence |

No screenshots are committed to git; the repo `.gitignore` also ignores
`docs/research/**/evidence/`, but screenshots were kept outside the repository by policy.

## 4. What is confirmed by running vs inferred

### 4.1 `confirmed-by-running`

- Window opens at **1000×620** (source `WindowOptions`), minimum 560×340; captured content
  area **1000×593** (top 27 px is the GTK/MyGo menu bar on Linux).
- Window composition (seen in `run-05-x11.png` and the captured states):
  - native GTK menu bar: `Shell`, `Edit`, `View`, `Tabs`, `Window` (top-level labels);
  - title bar: **host chip** (`falcon` / `Linux`, with a machine glyph), **tab pill**
    (`fish ~`, terminal glyph), a wide draggable empty region, a **⌘ palette button**, a
    **+ new-tab button**, and window controls (`−`, `▢`, `×`);
  - content: pane **card** with rounded corners, over a dark purple-brown **gradient**
    background, separated by visible **gaps**;
  - pane **header**: program glyph + `fish` name + `~` directory, and, on the focused pane,
    four right-side icon buttons (split-right, split-down, zoom, close);
  - terminal body with a live shell prompt and block cursor.
- Split behaviour: `Split Right` produces two side-by-side panes of equal width with a gap;
  `Split Down` splits the focused pane; `Zoom Pane` toggles to a single full-size pane;
  toggling back restores the prior tree (03 == 05 layout).
- Command palette: opens as a centred panel over a dimmed workspace; search field with
  magnifier glyph and placeholder `Type a command, or the name of a pane…`; rows show pane
  jump targets first (`fish ~` with a `Tab 1` badge) then commands, each with a leading ⌘
  glyph and a right-aligned shortcut badge (`Ctrl+Shift+T`, `Ctrl+Shift+D`, `Ctrl+Shift+E`,
  `Ctrl+Shift+Enter`, `Ctrl+Shift+W`, `Ctrl+PageDown`); selected row is a steel-blue
  highlight; roughly 10 rows visible at 34 px pitch (source constant: palette row height 34
  in `commands.go`; frame measurement: selected row y=129–162 in `06-palette.png`); panel
  is ≈560 px wide, top edge ≈80 px below the content top (approximate, measured from the
  capture).
- The application log line about CPU drawing (`mygo: native UI draws without the GPU`) and
  the successful CPU fallback rendering.
- The `-server` subcommand, `GOREX_DIR`, `layout.json`/`settings.json`, socket at mode 0600
  in the data directory (source-visible and exercised by the running app, which restored the
  fish session between captures driven by the same server).
- The palette and split states were produced by real commands executed inside the running
  app via its debug hook, not hand-drawn.

### 4.2 `inferred-from-README/source` (not visually exercised here)

- Exact menu **item** lists per top-level menu (see §7) — README and `commands.go`; only the
  top-level menu labels were seen on screen.
- Host popover **contents** (`Model`, `Chip`, `Memory`, `System`, `User`, `Server`,
  `Sessions` rows and the trailing explanation sentence `host.go`); the popover itself was
  not captured.
- Tab context menu, tab drag-reorder, double-click rename, tile-stack icons — `tabs.go` /
  README; not captured.
- Activity semantics: green dot while a program prints, orange dot when a program finished or
  rang the bell out of sight, background-window notification for a long command — README +
  `view.go`/`programs.go`; not observed completing in the captures.
- Light appearance, System appearance, and text-size changes — `commands.go`/`style.go`; only
  dark appearance was captured.
- Persistence across app quit/relaunch, and server snapshot/restore of full-screen programs —
  README + `internal/rex`; only a fluid same-process restore between captures was exercised.
- Windows-specific window controls/frameless behaviour — source flags (`ownControls`,
  `runtime.GOOS == "windows"`); not applicable on this host.
- Accessibility (screen-reader labels, roles, focus order) — source uses `Role`,
  `Label`, `Tooltip`, `Selected`, `Focus`; no actual AT inspection was performed.

### 4.3 `unknown`

- Panel width and typography metrics (approximated from the capture, not measured with a
  tool); palette row pitch is measured at 34 px (see §4.1).
- Focus ring/hover rendering under a real compositor/window manager (Xvfb had no WM).
- Notification presentation on Linux (MyGo/portal-dependent).
- Behaviour with multiple simultaneous clients, reconnection, or server crash recovery.
- Anything about remote/multi-host workflows: not implemented in GoRex (see §9).

## 5. Component hierarchy and measured constants

Source-derived layout constants (`view.go`, `tabs.go`) — `[S]` source, confirmed where the
capture agrees:

| Constant | Value | Meaning |
|---|---|---|
| Window default | 1000×620, min 560×340 | `mygo.WindowOptions` in `main.go` [S] |
| `titleH` | 44 px | title-bar row height [S] |
| `gap` | 8 px | gap around and between pane cards; also divider thickness [S, confirmed visually] |
| `cardR` | 12 px | pane card corner radius [S, confirmed visually] |
| `headerH` | 33 px | pane header height [S] |
| `tabH` | 28 px | tab height; track height `tabH+4` = 32 px, pill radius `tabH/2` = 14 px [S] |
| `titleFree` | 56 px (minus 14 gap) | title-bar free space before the right-side buttons [S] |
| Palette/icon buttons | 30×30 px, icon 17/19 px | title bar [S] |
| Pane header buttons | 26×26 px, icon 16–17 px | only visible on focus or hover [S, confirmed] |
| Activity dot | 6 px busy + 2.5 px halo; 7 px attention | pane header [S] |
| Terminal font | JetBrains Mono (embedded), fallback `SF Mono, Menlo, monospace`, default size **11.6**, line height 1.0, clamp 6–40 | `style.go`, `settings.go` [S] |
| Header title font | 12.5 px, weight 600; directory 12.5 px weight 500 at 86 % alpha | `view.go` [S] |
| Host chip text | 13.5 px weight 700 (name), 11.5 px (model) | `host.go` [S] |
| Split ratio | new split 0.5; divider drag clamp 0.08–0.92; keyboard resize clamp 0.1–0.9; double-click divider = 0.5; equalize preserves aspect ratio | `view.go`, `state.go` [S] |
| Busy threshold | 1500 ms since last terminal output | `view.go` [S] |

Hierarchy (from `view.go`):

```
window (background gradient + optional right tint)
└── title bar (44)
    ├── host chip (button, Role=button, label "Host <name>", max width 190)
    ├── tab strip (track radius 14, padding 2, width = sum of tab widths)
    │   └── tab pill (radius 14, padding L10/R6, gap 9; icon tiles; label; close button 18×18 on hover; attention dot)
    ├── draggable spacer (MinWidth titleFree−14)
    └── buttons: command palette (30×30) · new tab (30×30) [· window controls on Windows]
└── content column (padding 8, MinHeight 0)
    └── pane tree
        ├── leaf = pane card (radius 12, 1 px border, 2 shadows, colors animate 160 ms)
        │   ├── header (33; padding T0 R8 B0 L12; gap 7)
        │   │   ├── program icon 14.5
        │   │   ├── title row (gap 5): name (12.5/600) · directory (12.5/500 @86 %) · activity badge
        │   │   └── hover/focus buttons (26): split-right · split-down · zoom/unzoom · close
        │   └── terminal body (padding T0 R5 B6 L5)
        └── split = row/column of [A (Grow ratio)] [divider 8 px] [B (Grow 1−ratio)]
```

Overlays: command palette panel; host popover; error banner pinned bottom-centre
(`errorBanner`: dot `#ef4444`, radius 12, panel background, close button); rename text field
inside the tab pill.

## 6. Typography and colour tokens (source `style.go`; 120 ms transition constants from
`view.go` ≈119/≈150; verified in dark capture)

- Embedded **JetBrains Mono** (Regular/Bold/Italic/BoldItalic) is registered at startup; the
  terminal family string is `JetBrains Mono, SF Mono, Menlo, monospace`; default size 11.6 px.
- Window background is a **three-stop vertical gradient** plus a soft right-side tint, not a
  flat colour:
  - light: `#f6e7f6 → #efe1e6 → #eee0d3 → #ece1bd`, tint `rgba(242,214,222,0.35)`;
  - dark: `#2c2131 → #231e27 → #221f22 → #2a2619`, tint `rgba(80,40,60,0.25)`.
- Card surfaces (translucent, over the gradient): light `rgba(255,255,255,0.66)`, focused
  `0.95`; dark `rgba(20,20,24,0.5)`, focused `rgba(36,36,41,0.94)`. Borders: light
  `rgba(255,255,255,0.72)`/focused `1.0`; dark `rgba(255,255,255,0.05)`/focused `0.14`.
- Text: light `#1d1d1f` / muted `#636366` / faint `#8e8e93`; dark `#f2f2f7` / `#aeaeb2` /
  `#8e8e93`; icon muted `#8a8a8f` light, `#98989d` dark.
- Controls: hover `rgba(0,0,0,0.055)` light / `rgba(255,255,255,0.08)` dark; pressed
  `rgba(0,0,0,0.1)` / `rgba(255,255,255,0.14)`; 120 ms colour transitions.
- Tabs: active light `rgba(255,255,255,0.97)`, dark `rgba(255,255,255,0.14)`; separator
  `rgba(0,0,0,0.13)` / `rgba(255,255,255,0.14)`.
- Status: busy `#34a853` light / `#4ade80` dark; attention `#f59e0b` / `#fbbf24`; exited
  `#a1a1a6` / `#636366`; Windows close-button hover `#c42b1c`.
- Panels: light `rgba(252,252,253,0.97)` with border `rgba(0,0,0,0.08)`, selection
  `rgba(46,111,208,0.12)`; dark `rgba(38,38,42,0.97)` with border `rgba(255,255,255,0.1)`,
  selection `rgba(108,178,255,0.18)`.
- Terminal themes: light fg `#2a2d31` / bg `#fbfbfb` / cursor `#3a3d42`; dark fg `#e6e6ea` /
  bg `#1e1e22` / cursor `#e6e6ea`; full 16-entry ANSI palettes are defined in `style.go`.

## 7. Menus, popovers and labels (source; top-level labels confirmed on screen)

Top-level menus (seen: `Shell`, `Edit`, `View`, `Tabs`, `Window`) map to commands in
`commands.go`; items are created from each command's `Title` with its `Accel` as the menu
accelerator:

- **Shell**: New Tab, Split Right, Split Down, Close Pane/Tab, Zoom Pane, Equalize Panes,
  Restart Shell in Pane, Clear …, Quit and End All Sessions (separators between groups).
- **Edit**: Clear Screen and Scrollback (⌘K / Ctrl+Shift+K), plus standard edit items as
  MyGo provides.
- **View**: Appearance: Light / Dark / System, Bigger/Smaller/Actual Size text.
- **Tabs**: Next/Previous Tab, `Show Tab 1…9` items with accelerators, Rename Tab….
- **Window**: standard MyGo window items.

Host popover (`host.go`): header with host name and `This computer`/`This Mac`/`This PC`;
rows `Model`, `Chip`, `Memory`, `System`, `User`, and a server block `Server` (`pid <n> · up
<duration>`) and `Sessions` (`<n> open, <n> running a program`); footer sentence: *"Sessions
live in the server: quit GoRex and they keep running, and come back as they were when it
opens again."*

Tab context menu (`tabMenu`) and drag-reorder / double-click rename exist in source; exact
menu labels were not captured.

## 8. Keyboard shortcuts (verbatim from README and `commands.go` `Accel`/`Other`)

macOS accelerators are GoRex's `Accel`; "Windows, Linux" are the non-macOS alternatives
(`Other`). On Linux/macOS, Control+letter belongs to the terminal, so GoRex uses
Control+Shift for its own actions, and Ctrl+Alt is avoided because it types AltGr characters.

| Action | macOS | Windows, Linux |
|---|---|---|
| New tab | ⌘T | Ctrl+Shift+T |
| Split right / down | ⌘D / ⇧⌘D | Ctrl+Shift+D / Ctrl+Shift+E |
| Close pane / tab | ⌘W / ⇧⌘W | Ctrl+Shift+W |
| Zoom pane | ⇧⌘↩ | Ctrl+Shift+Enter |
| Focus pane direction | ⌥⌘ arrows | Alt+arrows |
| Move nearest divider | ⌃⌘ arrows | Alt+Shift+arrows |
| Equalize panes | ⌃⌘= | — |
| Switch tabs | ⌘1…⌘9, ⇧⌘[ ⇧⌘], ⌃Tab | Ctrl+Shift+1…9, Ctrl+PageUp/PageDown, Ctrl+Tab |
| Rename tab | ⇧⌘R | Ctrl+Shift+R |
| Command palette | ⇧⌘P, ⌘P | Ctrl+Shift+P |
| Clear | ⌘K | Ctrl+Shift+K |
| Text size | ⌘+ ⌘− ⌘0 | Ctrl+= Ctrl+- Ctrl+0 |
| Quit and end all sessions | ⌥⌘Q | — |
| Copy / paste | ⌘C / ⌘V | Ctrl+Shift+C / Ctrl+Shift+V |

These strings are reproduced exactly as they appear in the repository; none were invented.
The palette capture independently confirms the Linux forms `Ctrl+Shift+T`, `Ctrl+Shift+D`,
`Ctrl+Shift+E`, `Ctrl+Shift+Enter`, `Ctrl+Shift+W`, `Ctrl+PageDown`.

## 9. Behaviour catalogue by state/area

Each item is labelled `[C]` confirmed-by-running (see §4.1), `[I]` inferred from
README/source, or `[U]` unknown.

- **Window / title bar** `[C]`: hidden native title bar style on macOS, GTK menu bar on
  Linux, custom title bar drawing host chip, tabs, spacer, palette and new-tab buttons; the
  window title tracks the active tab (`name + detail`).
- **Tabs** — pill in a rounded track; width computed per tab (`tabWidth`); active tab is
  lighter; hover reveals an 18×18 close button; attention dot appears on the pill; drag
  reorder, double-click rename, context menu `[I]`; 64 px min width, shrink as more tabs
  appear `[I]`.
- **Split panes** `[C]`: binary tree of vertical/horizontal splits, new split at 0.5; drag a
  gap to resize; hover/drag paints a 36×3 px (or 3×36) grab indicator at 60 % muted colour;
  double-click a divider to reset to half; keyboard resize moves the nearest divider.
- **Zoom** `[C]/[I]`: zoom shows one pane grown to the content area; the header button turns
  from `maximize-2` to `minimize-2` and its tooltip from "Zoom" to "Unzoom"; double-clicking
  a pane header also toggles zoom `[I]`.
- **Pane header** `[C]`: program glyph, name, directory, activity badge; buttons hidden when
  neither focused nor hovered; clicking/double-clicking the header focuses the pane `[I]`.
- **Program activity** `[I]` (semantics source/README, not observed): green dot + halo while
  a program prints (≥1.5 s window), orange dot when a program finished or rang the bell out
  of sight, tooltips "Printing" / "Needs attention"; shell at a prompt shows the shell icon
  and no dot. Program naming (`Node`, `Git Changes` for lazygit, `Codex`, `SSH host`) comes
  from `programs.go` heuristics over the foreground process and cwd.
- **Notifications** `[I]`: a long command finishing while the window is in the background
  posts a notification (README); not exercised.
- **Host information** `[C]` partially: host chip visible (`falcon`, `Linux`); popover
  contents `[I]` per §7; a 1500 ms stale threshold drives the busy badge.
- **Command palette** `[C]`: see §4.1; all commands and all panes are listed and
  fuzzy-matched; the last row visible in the capture is `Next Tab` (list scrolls).
- **Appearance** `[I]`: Light/Dark/System via `View ▸ Appearance`; persisted in
  `settings.json`; `mygo.Theme.SetSource`.
- **Text size** `[I]`: ⌘+/⌘−/⌘0 equivalents adjust the terminal font (6–40) and persist.
- **Focus** `[C]`: exactly one focused pane per tab; the focused card is lighter with a
  brighter border and stronger shadow; terminal focus can move tab focus; focus movement via
  Alt+arrows (Linux) or ⌥⌘ arrows (macOS) `[I]`.
- **Empty/error states** `[I]`: if there is no saved layout, the app opens one new tab in the
  user's home directory; a server connection failure shows a blocking error dialog and quits
  (`GoRex could not start its session server`); a protocol-version mismatch shows an inline
  banner: *"The session server is of another version of GoRex: quit and end all sessions to
  restart it."*; a dev-build server mismatch offers a restart that **ends current sessions**.
- **Persistence UX** `[I]`: sessions live in a background server process (`GoRex -server`),
  so quitting the app detaches panes rather than closing them; layout (`layout.json`,
  `{active, tabs:[{name, root{a,b,ratio,vertical}, focus, zoom}], pane {sid, dir, cols, rows}}`)
  and settings (`settings.json` — appearance, fontSize) live in the data directory
  (`GOREX_DIR`); a rebuild restarts the server and ends its sessions.
- **Multi-client behaviour**: `[U]` not tested; the README does not claim multi-client attach.

## 10. Comparison table

Columns required by `full.md` §2.6. "Original videos/docs" = what the five REX videos and
official docs are expected to testify to (owned by the `ref-video` worker; cited here only
where the GoRex README aligns with the documented REX feature list). "Relayable (Phase 0 —
not implemented)" is the current spec-only state.

| Item | Original videos/docs | GoRex | Relayable (Phase 0 — not implemented) | Discrepancy | Confidence |
|---|---|---|---|---|---|
| Session lifetime independent of client | Core promise (`full.md` §1.2) | `[I]` server process owns PTYs; quit app → sessions continue | Specified, no code | GoRex matches README-stated REX promise | High for GoRex source; unverified in run |
| Tabs + split panes | Shown in Video 1 per `full.md` §2.6 | `[C]` tabs, split right/down, zoom, resize, double-click divider | Planned GPUI client | None observed within this feature | High |
| Pane header (program/icon/dir) | Described for REX in §2.6 | `[C]` glyph + name + directory + activity + 4 buttons | Planned | GoRex adds hover-only buttons — may or may not match video | Medium |
| Persistent sessions / snapshot restore | Core promise | `[I]` headless terminal emulator snapshot on attach | Planned | Not observed across a real app restart | Medium |
| Program activity indicators | Described in §2.6 | `[I]` green busy dot, orange attention dot, tab dots | Planned (program status protocol) | Detailed semantics are GoRex-specific | Medium |
| Notifications | Listed in §2.6 | `[I]` background long-command notification | Deferred/unspecified | Unknown visual form | Low |
| Host info | Listed in §2.6 | `[C]` chip; `[I]` popover with model/chip/memory/OS/user/server/session counts | Host profiles planned | GoRex is single-machine | High |
| Command palette | Listed in §2.6 | `[C]` panel, fuzzy pane+command list, shortcut badges | Planned | None identified | High |
| Appearance settings | Listed in §2.6 | `[I]` Light/Dark/System, persisted | Unspecified | — | Medium |
| Text sizing | Listed in §2.6 | `[I]` 6–40 px, default 11.6, embedded JetBrains Mono | Unspecified | — | Medium |
| Keyboard shortcuts | Video + docs (ref-video worker) | `[I]` README table; `[C]` Linux forms in palette | To be defined | GoRex adds Ctrl+Shift remaps on non-macOS; source of truth remains the videos | Medium |
| Remote / multi-host connections | Video 4 + docs | **Not replicated** (`README`: *"Not replicated: Rex's connections to servers on other machines; GoRex's server listens on a local socket only."*) | Planned native protocol + remote auth | GoRex must not be used as remote evidence | High |
| Window default size | Unknown | `[C]` 1000×620, min 560×340 | Unknown | GoRex's choice, not necessarily REX's | Low |
| Colour/typography tokens | Video frames pending | `[C]/[I]` exact palette in §6 | Planned | GoRex's own palette may differ from REX footage | Medium |

## 11. Mandatory limitations

1. **GoRex does not replicate remote server connections.** The README states this explicitly;
   its server listens on a **local Unix socket only** (source: `internal/rex/server.go`,
   `net.Listen("unix", SocketPath())`, socket chmod `0o600`). GoRex is therefore **not
   evidence** for REX's remote protocol, remote-host discovery, authentication, pairing,
   reconnection, SSH usage, or any network transport. Video 4 (remote sessions) and official
   docs remain the only admissible evidence there.
2. GoRex was reviewed at a single commit; it is a third-party clone and may consciously differ
   from REX. Any GoRex-only behaviour must yield to direct original-product evidence.
3. No license is verified for GoRex code; only observed behaviour may inform Relayable.
4. The run environment was headless Xvfb with CPU drawing; compositor, font rasterisation and
   scale-factor behaviour may differ on a real desktop.
5. Several behaviours in §4.2 are source/README-inferred and were not visually exercised; they
   must be re-verified before being treated as observed REX behaviour.
6. The GoRex README screenshot and other repository assets were not copied, and no GoRex
   asset or screenshot may be committed to Relayable.

## 12. Validation commands and results

```
git status --short                                     # clean before edit (exit 0)
git -C ~/.cache/relayable-research/gorex rev-parse HEAD
  b206e67bf0ac422afebc5c9e701a38dd67f2c837
git -C ~/.cache/relayable-research/gorex log -1 --format='%H%n%ci%n%s'
  b206e67bf0ac422afebc5c9e701a38dd67f2c837
  2026-10-08 23:56:12 +0800
  Update MyGo for closing the window from its view
git ls-files | grep -iE 'licen[cs]e|copying|notice'
  assets/brands/LICENSE.md
  assets/icons/LICENSE                              # no root LICENSE (exit 1 for root pattern)
go build ./...                                       # exit 0
go build -o /tmp/gorex-review-bin .                  # exit 0, 20617576 bytes
Xvfb ... ; gorex-review-bin                          # process alive; window rendered
                                                     # log: "Unable to create a GL context" (CPU fallback)
git check-ignore -v docs/research/evidence/x.png
  .gitignore:11:/docs/research/**/evidence/  docs/research/evidence/x.png   # exit 0
ls -la ~/.cache/relayable-research/evidence/gorex/   # 10 PNG files, all outside the repo
```

Repository-side validation (after writing this file):

```
git diff --cached --name-only   # must list only docs/research/gorex-review.md
git log --oneline -3
git status --short
```

## 13. Remaining work

- Re-verify shortcut strings, notification presentation, and light appearance against original
  REX video footage/official docs before implementing them.
- If a future GoRex license appears, re-evaluate; until then, treat GoRex as a
  behaviour/layout catalogue only.
- Capture a host-popper, tab-context-menu, rename and light-appearance state if a later
  research pass needs those specifics; they are not required to start Relayable's GPUI shell.

## Revision

Revised after independent audit (`result:audit-gorex#1`): palette pitch corrected ≈28 → 34 px (source constant + frame measurement), 120 ms transition attribution corrected to `view.go`, `run-02-ffmpeg.png` framing corrected.
