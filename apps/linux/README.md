# Relayable Linux app (`apps/linux`)

The Linux desktop app for **Relayable** (product name only; `full.md` §0.1). It is a GPUI-based
Rust package, deliberately isolated in its own Cargo workspace. This README is the `full.md` §9.1
deliverable.

**Current state: UI-001 + UI-002.** UI-001 proved the ADR-003 `gpui = "=0.2.2"` pin builds and
runs on this host and produced the GPUI-free design-token sheet. **UI-002 implements the reference
workspace UI** — top bar + tab strip, pane headers, split panes with focus, the terminal body
rendered from a live `relayable-server`, a session picker, and Relayable-original connection/mock
banners. Sections below marked **UI-002** are new; the GPUI-pin / TERM-002 remedy (§12) and the
token-sheet provenance (§4) are still-true UI-001 content. The quantitative reference comparison
lives in [`evidence/ui-002-comparison.md`](./evidence/ui-002-comparison.md).

---

## 1. What this crate is

| Path | Role |
|---|---|
| `src/lib.rs` | Library crate `relayable_linux`. Declares `api`, `app`, `geometry`, `term`, `theme`, `tokens`, `ui`. **Not** GPUI-free as a whole: `app` and `ui` carry GPUI views (`full.md` §5.3 only requires GPUI code to live inside `apps/linux`). |
| `src/tokens.rs` | Re-export root of the design-token sheet (`src/tokens/**`), encoded **only** from `docs/research/design-tokens.md`. No GPUI. |
| `src/geometry.rs` | Measured reference geometry as pure data (`v1` / `v4` namespaces) plus layout helpers; reused `tokens::Measured`. No GPUI. |
| `src/theme.rs` | Packed `0xRRGGBB` colours mapped from `tokens`, plus clearly-labelled `*_ORIGINAL` Relayable-original values. No GPUI. |
| `src/term/mod.rs` | GPUI-free headless terminal model: `alacritty_terminal` 0.26.0 + `vte` 0.15.0. |
| `src/api/mod.rs` | GPUI-free live two-plane server client (data + control) and the clearly-labelled mock. |
| `src/app.rs` | **UI-002** — `WorkspaceView`, the GPUI root entity: top bar, tab strip, panes, headers, picker, banners, key handling. |
| `src/ui/mod.rs` | **UI-002** — UI view layer + colour `lerp` used by the terminal ramps. |
| `src/ui/terminal.rs` | **UI-002** — `TerminalCanvas`, paints the terminal grid, cursor, scrollbar. |
| `src/ui/palette.rs` | **UI-002** — Relayable-original xterm-256 ANSI palette (**non-reference**). |
| `src/bin/live-probe.rs` | GPUI-free live proof (Stage A): handshake, `session.create`, WRITER attach, `stty size` resize. |
| `src/main.rs` | The thin GPUI binary (`relayable`); the only file that opens a window. |

Two reasons for the lib/bin split (`full.md` §5.3): the pure data/sheet modules stay independent of
GPUI, and the headless terminal model plus the API client are unit-testable without a display
server. Run it:

```sh
cd apps/linux
cargo run --bin relayable -- [--mock] [--socket PATH] [--canvas W H]
```

## 2. Reference screens/states with source timestamps (`full.md` §9.1)

Sources are the five reference videos; Video 2 (architecture) and Video 3 (memory benchmarks)
contain **no product UI** and are not UI evidence. Wordmarks quoted from the reference product are
research evidence only — Relayable branding and assets are used (`full.md` §0.1).

**Reference videos (`video-review.md` §1.1):** V1 "Superlogical Pre-Alpha Demo"
(<https://youtu.be/XyxShCZrGiw>) · V2 "Superlogical Terminal Multiplexer High-Level Architecture"
(<https://youtu.be/Y6nFMmUPzXM>; no UI) · V3 "Superlogical Server Memory Optimizations Overview"
(<https://youtu.be/T5gV6anSt-4>; no UI) · V4 "Superlogical Remote Sessions Pre-Alpha Demo"
(<https://youtu.be/PdwTjSBW6Y8>) · V5 "Superlogical Pre-Alpha Demo: CLI"
(<https://youtu.be/9fcfDF8SBnc>).

### 2.1 Implemented (catalogue entries → source timestamp → component)

| # | Catalogue entry | Source timestamp(s) | Relayable component (implemented) |
|---|---|---|---|
| 1 | `desktop/workspace-default` | **V1 0:24** | `src/app/render.rs` (`render`), `src/app/top_bar.rs` (`render_top_bar`), `src/app/panes.rs` (`render_body`), `src/main.rs` (1099×706 window) |
| 2 | `desktop/tab-strip` | **V1 0:42–0:47 / 2:22** | `src/app/top_bar.rs` (`render_top_bar`, pill, inactive divider, duplicate-label `-1` rule) |
| 3 | `desktop/pane-header` | **V1 0:42–0:47 / 1:33 + V4 0:58 / 4:49** | `src/app/panes.rs` (`render_pane_header`, four icon buttons on the focused split pane) |
| 4 | `desktop/terminal-body` | **V1 0:47 / 1:33–1:40** | `src/ui/terminal.rs` + `src/term/model.rs` (ramp, 17.25 px pitch, block cursor, scrollbar) |
| 5 | `desktop/split-layout` | **V4 0:58 / 4:49** | `src/app/panes.rs` (`render_pane`), `src/app/state.rs` (`SplitAxis`), focused/unfocused surfaces + 2.5 px borders |
| 6 | `desktop/session-picker` | **V1 1:40** | `src/app/picker.rs` (`render_picker`, `picker_*`) |

Additional observed behaviour implemented: live program/cwd in tab + header from shell **OSC 2 / OSC 7**
(`src/api/osc.rs` OSC tracker), tab switching, split/focus/close, runtime `terminal.resize`.

### 2.2 Not implemented (with source timestamp)

| # | Catalogue entry | Source timestamp(s) | Why not |
|---|---|---|---|
| 7 | `desktop/session-pill` | V1 2:22; V4 0:58, 4:11, 4:49; V5 7:04 | remote-host chrome; no remote sessions in this slice |
| 8 | `desktop/vertical-tabs` | V1 2:22 | **geometry unmeasurable** — `v1-t0222` does not depict a vertical-tab layout at all (it is the command-palette pre-switch frame; `evidence-local/ref/REFERENCE-FRAMES.md` §5) |
| 9 | `desktop/command-palette` | V4 0:30 | optional in the UI-002 brief; geometry was measured (442×667 px, 37.5 px pitch) but core layout came first |
| 10 | `desktop/remote-host` | V4 0:58, 4:11, 4:49 | no remote transport in this slice |
| 11 | `desktop/reconnect-after-quit` | V4 3:59 → 4:11 | no in-app chrome was observed; reconnection is proven live but rendered as a Relayable-original banner (see §2.3) |
| 12 | `desktop/cli-client-terminal` | V5 1:04–2:28, 4:25, 5:35, ≈6:00 | CLI runs in a client terminal, not the app |
| 13 | `desktop/cli-help` | V5 7:04 | same |

`desktop/settings`, `desktop/inspector` and an in-app `desktop/disconnected` screen were **not
observed** in the footage (`ui-catalog.md` "Coverage and limitations").

### 2.3 Relayable-original, non-reference states

The catalogue has **no entry** for connecting / reconnecting / disconnected / error / loading /
empty states. `docs/research/open-questions.md` **Q9** resolves that these must be designed as
**Relayable-original and labelled non-reference** (the only "disconnect" evidence is the whole app
being quit, V4 3:59, and restored, V4 4:11). UI-002 renders them as a full-width connection banner
(and an empty-state panel) explicitly documented as Relayable-original; they are **not** reproduced
from a reference and must not be presented as such.

## 3. Component mapping

| Screen / state | Source files |
|---|---|
| Top bar, separator, tab strip, pill, `+`, inactive dividers | `src/app/top_bar.rs` (`render_top_bar`), `src/geometry.rs` (`workspace_columns*`), `src/theme.rs` |
| Pane card, split axis, focus, borders, pane header + icon buttons | `src/app/panes.rs` (`render_body`, `render_pane`, `render_pane_header`, `render_icon_button`, `icon_glyph`) |
| Terminal body (grid, ramps, cursor, scrollbar, scroll) | `src/ui/terminal.rs`, `src/term/model.rs`, `src/ui/mod.rs` (`lerp_packed`) |
| ANSI cell colours | `src/ui/palette.rs` (**non-reference** xterm-256) |
| Session picker popup | `src/app/picker.rs` (`render_picker`, `picker_matches`, `picker_accept`), `src/app/keys.rs` (`handle_picker_key`) |
| Connection / mock banners, empty state | `src/app/top_bar.rs` (`render_connection_banner`, `render_mock_banner`), `src/app/panes.rs` (`render_empty_state`) |
| Live server I/O, OSC title/cwd, reconnect, mock | `src/api/mod.rs` (`data.rs`, `control.rs`, `osc.rs`, `mock.rs`) |
| Window creation, options, keybinding help | `src/main.rs` |
| Measured tokens | `src/tokens/**` (sheet) → `src/geometry.rs` / `src/theme.rs` (typed access) |

## 4. Design tokens

**Single source:** `docs/research/design-tokens.md` (measurements with per-value method and
confidence). No value exists in `src/tokens/**` that is not stated there. Every token carries its
`source` stamp, `confidence` label and `caveat` in the type system (`Measured<T>`); ranges stay
ranges (`Approx`).

- **`geometry` / `theme` split:** `src/geometry.rs` holds pure layout data (namespaced `v1` at
  1170×1080 and `v4` at 1624×1080) and helpers; `src/theme.rs` holds packed `0xRRGGBB` colours
  mapped from `tokens`, because gpui 0.2.2 takes `rgb(hex: u32)`. `src/tokens.rs` remains the
  re-export root of the provenance sheet values under `src/tokens/**`.
- **`ORIGINAL_INVENTED` labelling:** anything never measured is marked Relayable-original in code —
  `theme::*_ORIGINAL`, `geometry::invented::*_ORIGINAL`, `ui/palette.rs`. No invented value is
  presented as a reference token.
- **Geometry is namespaced per source:** `design-tokens.md` §2.3 states the V1/V4 frames use
  different export scales and that absolute pixel values are **not comparable across V1 and V4**.
  No px→point conversion and no cross-video comparison is performed. `BASELINE_CANVAS_OUTER`
  (1099×706) is treated as logical px **by assumption only** (§2.3 provides no conversion).
- **Deliberately absent** (`tokens::unmeasured`, carrying the document's own wording): font
  family/weight, exact type sizes, motion/hover timings, pixel-exact corner radii, the V1
  pane-header height, V4 window vertical bounds, terminal colour under possible translucency,
  scrollbar/cursor/selection colours, Video 5 CLI text, the traffic-light-adjacent toolbar button.
- **REF-004 conflict recorded, not resolved:** `design-tokens.md:210` lists "29 px palette rows"
  among clone-derived numbers while `:212` gives the GoRex value 34 px and `:204` marks the earlier
  29 px estimate "wrong: 37.5 px". `tokens.rs` encodes only the measured video values and flags the
  conflict in a doc comment; `docs/**` was **not** edited.

Applied measured tokens and their confidence are tabulated in
[`evidence/ui-002-ui-notes.md`](./evidence/ui-002-ui-notes.md) §2.

## 5. Known fidelity differences

Full ranked list with raw measurements: [`evidence/ui-002-comparison.md`](./evidence/ui-002-comparison.md) §4.
Summary in `full.md` §9.3 impact order:

**Layout / hierarchy.** Vertical tabs absent (unmeasurable); command palette absent (optional);
session pill / remote-host chip absent; macOS chrome (traffic lights, rounded window, shadow) not
reproduced; connection states are a Relayable-original banner the reference never showed.

**Proportions.** 3 px vertical chrome offset (bar y0 vs reference y3); pane header 41 px is a V4
inference because the V1 header is not separable; pill radius 9 px is a low-confidence visual
estimate.

**Colours / contrast.** Measured tokens match within ±1–2/channel: pill fill rgb(19,40,62) vs
reference rgb(19,41,62); top bar rgb(4,27,46) vs rgb(5,27,47); separator rgb(30,48,66) vs
rgb(30,49,66); split borders rgb(31,81,114)/rgb(17,53,71) vs rgb(31,82,115)/rgb(17,54,71); ramps
within ±1–5/channel. The active pill's 1 px top/bottom edge highlight (reference rgb(50,75,99) /
rgb(54,76,99)) is **not rendered**.

**Typography.** Font family and sizes are unmeasured; JetBrainsMono Nerd Font 13 px (cell width
7.8 px, row pitch 17.25 px) is a Relayable proposal. Reference row pitch ≈17.1 px.

**Icons.** The four pane-header button glyphs are Relayable-original; the reference icon-by-icon
identity/order is low-medium confidence.

**Microspacing.** Separator is 1 px (measured) vs the 1.5 px token midpoint; picker search field
28.5 px / row 35 px are range midpoints.

**Deliberate deviations (documented, not defects).**

1. **Reserved empty 128 px traffic-light region.** The reference strip starts at x=128 because the
   macOS traffic lights held that space; traffic lights are **not app tokens**
   (`design-tokens.md` §3.2). Relayable leaves the region empty and preserves the measured pill
   position.
2. **Flush top bar at y=0.** The reference bar starts ~3 px below the crop top (macOS window
   border). Relayable's Wayland toplevel has no macOS border, so the bar is flush; the 40 px bar +
   1 px separator geometry is preserved.
3. **1 px separator vs the 1–2 px token.** A local re-measure found exactly 1 px, so the integral
   value is rendered while the sheet keeps the 1.5 px range midpoint.
4. **V1 pane-header height inferred from V4.** V1 is explicitly unmeasurable; the 41 px V4 midpoint
   is used.
5. **Square Wayland toplevel** vs the reference's rounded macOS window (with shadow).
6. **~3 % desktop-surface bleed.** gpui 0.2.2's Linux backend treats client-side decorations as
   transparent; Hyprland draws no server decorations, leaving a residual surface alpha. The app
   re-asserts opaque server-side decorations and `WindowBackgroundAppearance::Opaque` each frame,
   which reduces but does not eliminate the bleed. Quantified in `ui-002-comparison.md` §3.5.
7. **ANSI palette, cursor colour, scrollbar and connection/mock banner visuals are
   Relayable-original** (not measurable from the reference).
8. **Four pane buttons only when a tab is split.** V1's single-pane frame shows none; V4's focused
   split pane shows four.

## 6. Screenshot-test procedure (updated for UI-002)

This is the real, proven procedure used for the committed UI-002 screenshots, not a proposal. It
runs against the live Wayland session (`XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`).

**Geometry must be pinned by the compositor.** The app **cannot resize a Wayland toplevel**: gpui
0.2.2's Wayland backend never calls `xdg_toplevel.set_size`
(`gpui-0.2.2/src/platform/linux/wayland/window.rs`), so the initial `window_bounds` is the only
size control and the compositor decides the final geometry. This host is **Hyprland 0.56.2** with a
**Lua config provider**, so `hyprctl keyword windowrulev2` does **not** work. The working mechanism
is the Lua `hl.window_rule` eval:

```sh
hyprctl eval "hl.window_rule({ match = { class = \"^relayable$\" }, float = true, \
  size = { 1099, 706 }, move = { 10, 40 } })"
```

The runtime rule is retained in a Lua global (`RELAYABLE_UI002_RULE`) and disabled at teardown with
`RELAYABLE_UI002_RULE:set_enabled(false)`; it is session-only and never written to disk. Geometry is
read back from `hyprctl clients -j` (find `class == "relayable"`, read `at`/`size`), then captured
window-only with `grim -g "<x>,<y> <w>x<h>"` and verified with `identify`. Full run:

```sh
cd apps/linux
cargo build                                   # 1. build
# start a live relayable-server (from the repository root workspace) on the socket the
# harness expects, e.g. RELAYABLE_SERVER=/tmp/relayable-ui002-ui/relayable.sock
bash evidence/run-ui-002.sh                   # 2-8. launch, drive, capture, verify, teardown
```

`evidence/run-ui-002.sh` (a live harness; every line is a real command) does: pins the window,
launches `./target/debug/relayable` with the session env, reads the geometry back, drives the UI
with `wtype` (Ctrl+Shift+N splits, Ctrl+2/1/3 tabs, Ctrl+Shift+S picker, Ctrl+Shift+W close,
Ctrl+Shift+Left focus), captures each state window-only via `grim -g` plus a full-screen companion,
verifies every PNG with `identify`, then tears down (disable the runtime rule, stop the server if a
`server.pid` was recorded, remove the scratch socket/dir). Teardown asserts `ps -ef` and
`ls -d /tmp/relayable-ui002*` are empty; the transcript is tee'd to `evidence/ui-002-run.log`.

- **Baseline canvas 1099×706** (`geometry::BASELINE_CANVAS_OUTER`) and the smaller/larger
  verification (900×600 and 1400×900; runtime shrink to 900×600 was also sent as
  `terminal.resize` and accepted).
- **Overlay caveat.** The `omarchy-bar` layer-shell surface occupies the top **30 px**, so the
  window is positioned at y=40 to clear it and window-only captures use the client's own `at`/`size`
  (never the bar). A full-screen `omarchy-keyboard-panel` overlay surface is registered but did not
  appear over any capture; full-screen captures intentionally include the bar and are supplementary.
- Window-only PNGs are 1099×706 (or 900×600 / 1400×900); every state also has a 1920×1080
  `-fullscreen` companion.

`full.md` §9.3 also asks for a side-by-side comparison. The reference stills are local-only (see
§8), so the comparison is reproducible only where they exist; its raw numbers are committed in
[`evidence/ui-002-comparison.md`](./evidence/ui-002-comparison.md).

## 7. Live API usage

The app consumes a **real `relayable-server`, never a mock core**, through a **two-connection**
design in `src/api/mod.rs`:

- **Data plane** — a private `relayable_protocol::transport::FrameConnection` speaking the
  documented wire protocol on the public `relayable-protocol` transport: handshake, `block.attach`
  as **WRITER**, then `TerminalData` with the **mandatory `FlowControl` acknowledgement for every
  attached stream** (the server's credit window charges output payload bytes), plus
  `terminal.input` and `terminal.resize`. It holds many concurrent attachments and acks output for
  every attached stream, not just the focused one.
- **Control plane** — the `relayable-client` crate (path dependency) on its own connection for
  `session.create` / `session.list` / `session.view` and for `session.resume` events, which drive
  the connection-state visuals. It never calls `block.attach`, so it never contends for a writer
  lease (multi-client is supported).

**Required note — `relayable-client` cannot drive a live terminal view as shipped.** Its only
raw-byte API, `Client::wait_for_output` (public at `crates/relayable-client/src/lib.rs:558`), is
needle-based: `wait_for_output_inner` (`crates/relayable-client/src/lib.rs:794`) returns
`Err(ClientError::Timeout)` when the needle is absent (lines 811/816) and **drops the
`accumulated` bytes it has already read and credit-acked** (the ack is sent at line 845). There is
no public incremental-bytes API. The `relayable-protocol` transport data plane is the workaround.
This is a core-API gap; it is **reported here, not fixed** (core crates are outside this task).

**`libc` situation.** UI-002 adds the path deps above plus `alacritty_terminal = "=0.26.0"` and
`vte = "=0.15.0"`. `apps/linux` is its own workspace, so this cannot affect the root workspace or
`crates/**`. `apps/linux/Cargo.lock` holds **`libc 0.2.189` + `xattr 0.2.3`** (the UI-001 hold,
re-applied with `cargo update -p libc --precise 0.2.189`). No `[patch.crates-io]` was needed
because `relayable-server`'s `libc = "=0.2.190"` pin is **unreachable** from the
protocol/model/client crates (none depends on `relayable-server`). **Do not loosen it** —
loosening past 0.2.189 reintroduces the `xattr 0.2.3` `ENOATTR` compile failure. Full record:
[`evidence/ui-002-libc-remedy.md`](./evidence/ui-002-libc-remedy.md).

## 8. Reference-evidence durability

The reference stills are **not repo-durable.** `docs/research/evidence/**` is gitignored
(`.gitignore:11`) and is **absent from this worktree**, so **there are no reference stills in the
repository.** The durable records are the written
`docs/research/ui-catalog.md` + `docs/research/design-tokens.md`. Stills must be regenerated for
any future comparison from the cached 1080p sources under
`~/.cache/relayable-research/hires/` — V1 `XyxShCZrGiw.webm` (1170×1080) and V4
`a-/b-PdwTjSBW6Y8.webm` (1624×1080) — using the exact `-ss` times in `design-tokens.md` §2.4, or
re-downloaded. The regeneration recipes used by this stage are recorded locally in
`apps/linux/evidence-local/ref/REFERENCE-FRAMES.md`. **Reference imagery is local-only and never
committed** (`full.md` §0.2/§2.6); comparison composites live under the gitignored
`apps/linux/evidence-local/`.

## 9. Mock policy

**Live is the default.** `--mock` is opt-in and the UI must display a visible MOCK label: in mock
mode the app shows a persistent amber strip
`MOCK — synthetic in-process data; not a live relayable-server`. `ApiHandle::is_mock()` returns
`true` and the mock never pretends to be live (`full.md` §9.2). The committed
`evidence/ui-002-mock.png` samples the strip at rgb(136,107,27) ≈ the `MOCK_BANNER_BG` token
rgb(138,109,26).

## 10. Remaining gaps

- **Vertical tabs** — `v1-t0222` does not show a vertical-tab layout, so its geometry is
  unmeasurable; implementing it would require invented geometry (forbidden).
- **Command palette** — V4-observed (442×667 px, 37.5 px rows) but not implemented.
- **Remote host** — V4-observed; out of scope for this slice (no remote transport).
- **Session pill** — remote-host chrome; not implemented.
- **Wire-model limitation** — core-002 has **no `block.create`**, so a new split pane comes from a
  new session (`session.create` → `block.attach`); panes are capped at the server's advertised
  `max_concurrent_streams` (default 8). This is a protocol limitation, not a UI choice.
- **Input** — printable keys, control bytes, arrows/Home/End/Page keys, Alt/Meta prefixes; no
  bracketed paste, mouse reporting, IME, or text selection yet.
- **Session picker** lists all server sessions but can only switch to sessions this app created (no
  `block_id` is exposed by the frozen client API for arbitrary sessions).
- **~3 % desktop bleed** through the GPUI surface (see §5, deviation 6).

## 11. Keybindings (Relayable-original)

Linux bindings were never observed in the reference (macOS only; `open-questions.md` Q4); these are
a documented Relayable-original deviation. `relayable --help` prints the same table.

| Keys | Action |
|---|---|
| `Ctrl+Shift+E` | split pane right |
| `Ctrl+Shift+O` | split pane down |
| `Ctrl+Shift+W` | close focused pane |
| `Ctrl+Shift+Z` | zoom / unzoom focused pane |
| `Ctrl+Shift+N` | new session (new tab) |
| `Ctrl+Shift+S` | session picker |
| `Ctrl+Shift+T` / `Ctrl+PageDown` | next tab |
| `Ctrl+PageUp` | previous tab |
| `Ctrl+1..9` | select tab |
| `Ctrl+Shift+Right` / `Ctrl+Shift+Left` | focus next / previous pane |

## 12. `gpui = "=0.2.2"` pin and the TERM-002 remedy

- Pin: `gpui = "=0.2.2"` (ADR-003 Option H), the exact crates.io artifact.
- Blocker: on this host the pin resolves but `cargo check` exits **101** at `xattr 0.2.3` because
  `libc 0.2.190` removed `libc::ENOATTR` (`xattr-0.2.3/src/sys/mod.rs:12`).
- Remedy (contained to this workspace), run from `apps/linux`:

  ```sh
  cargo update -p libc --precise 0.2.189
  ```

- `apps/linux` is its **own** Cargo workspace (empty `[workspace]` table), so this lockfile hold
  cannot affect the repository-root `Cargo.lock` or `crates/**`.
- **Warning:** loosening `libc` past `0.2.189` inside this workspace reintroduces `cargo check`
  exit 101. `apps/linux/Cargo.lock` is committed as the remedy artifact — do not `cargo update`
  `libc` without re-checking. Full record: [`evidence/TERM-002-remedy.md`](./evidence/TERM-002-remedy.md)
  and [`evidence/ui-002-libc-remedy.md`](./evidence/ui-002-libc-remedy.md).

## 13. Build / validate commands

```sh
cd apps/linux
cargo fmt --check                 # exit 0
cargo clippy --all-targets --locked   # exit 0
cargo test --locked               # exit 0 (58 tests)
cargo check --locked              # exit 0
cargo build --locked              # exit 0
cargo build --release --locked    # exit 0

# from the repository root (proves apps/linux stays a separate workspace):
cargo check --workspace --locked  # exit 0
```

Recorded results: `evidence/validation.md`, `evidence/validation-commands.log` (UI-001),
`evidence/ui-002-validation.log` (UI-002), `evidence/ui-002-stage-a.md`. Toolchain on this host:
cargo/rustc 1.98.1, clippy 0.1.98, rustfmt 1.9.0. Live-run transcript: `evidence/ui-002-run.log`;
reconnect proof: `evidence/ui-002-reconnect.log`; interaction notes: `evidence/ui-002-ui-notes.md`;
comparison: `evidence/ui-002-comparison.md`.
