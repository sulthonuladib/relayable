# Relayable Linux app (`apps/linux`)

The Linux desktop app shell for **Relayable** (product name only; `full.md` §0.1). It is a
GPUI-based Rust package, deliberately isolated in its own Cargo workspace.

**Current state: UI-001 only** — a minimal, real GPUI window that proves the ADR-003
`gpui = "=0.2.2"` pin builds and runs on this host, plus a GPUI-free **design-token sheet**. The full
reference workspace UI is **UI-002 and is not implemented yet**; nothing in the `ui/*.rs` mapping
below exists in code. This README is the `full.md` §9.1 deliverable.

---

## 1. What this crate is

| Path | Role |
|---|---|
| `src/lib.rs` | Library crate `relayable_linux`; GPUI-free, testable. Exposes `pub mod tokens`. |
| `src/tokens.rs` | The design-token sheet, encoded **only** from `docs/research/design-tokens.md`. No GPUI. |
| `src/main.rs` | The thin GPUI binary (`relayable`); the only file that imports `gpui`. |

Two reasons for the lib/bin split (`full.md` §5.3): GPUI types cannot leak into the token sheet if a
future framework revision replaces GPUI, and the token sheet is unit-testable without a display
server.

Run it:

```sh
cd apps/linux
cargo run --bin relayable
```

## 2. Reference screens/states targeted

Sources are the five reference videos from `docs/research/ui-catalog.md` and
`docs/research/video-review.md` §1.1. Video 2 (architecture) and Video 3 (memory benchmarks) contain
**no product UI**; they are not UI evidence. The wordmarks quoted from the reference product are
research evidence only — Relayable branding and assets are used (§0.1).

| # | Catalogue entry | Source timestamp(s) | Planned component (not yet implemented) |
|---|---|---|---|
| 1 | `desktop/workspace-default` | V1 0:24 | `src/ui/workspace.rs` |
| 2 | `desktop/session-pill` | V1 2:22; V4 0:58, 4:11, 4:49; V5 7:04 | `src/ui/session_chip.rs` |
| 3 | `desktop/tab-strip` | V1 0:42–0:47, 0:55–0:58, 1:33–1:40, 2:22; V4 0:58, 4:11, 4:49; V5 7:04 | `src/ui/tab_strip.rs` |
| 4 | `desktop/pane-header` | V1 0:42–0:47, 1:33; V4 0:58, 4:49 | `src/ui/pane_header.rs` |
| 5 | `desktop/terminal-body` | V1 0:47, 0:55–0:58, 1:33–1:40, 2:03 | `src/ui/terminal.rs` |
| 6 | `desktop/session-picker` | V1 1:40 | `src/ui/session_picker.rs` |
| 7 | `desktop/vertical-tabs` | V1 2:22 | `src/ui/vertical_tabs.rs` |
| 8 | `desktop/command-palette` | V4 0:30 | `src/ui/command_palette.rs` |
| 9 | `desktop/split-layout` | V4 0:58, 4:49 | `src/ui/split_layout.rs` |
| 10 | `desktop/remote-host` | V4 0:58, 4:11, 4:49 | `src/ui/remote_session.rs` |
| 11 | `desktop/reconnect-after-quit` | V4 3:59 → 4:11 | `src/ui/connection.rs` |
| 12 | `desktop/cli-client-terminal` | V5 1:04–2:28, 4:25, 5:35, ≈6:00 | `src/ui/terminal.rs` |
| 13 | `desktop/cli-help` | V5 7:04 | `src/ui/terminal.rs`, `src/ui/command_palette.rs` |

`desktop/settings`, `desktop/inspector` and an in-app `desktop/disconnected` state were **not
observed** in the footage (`ui-catalog.md` "Coverage and limitations").

**Reference videos (`video-review.md` §1.1):** V1 "Superlogical Pre-Alpha Demo"
(<https://youtu.be/XyxShCZrGiw>) · V2 "Superlogical Terminal Multiplexer High-Level Architecture"
(<https://youtu.be/Y6nFMmUPzXM>; no UI) · V3 "Superlogical Server Memory Optimizations Overview"
(<https://youtu.be/T5gV6anSt-4>; no UI) · V4 "Superlogical Remote Sessions Pre-Alpha Demo"
(<https://youtu.be/PdwTjSBW6Y8>) · V5 "Superlogical Pre-Alpha Demo: CLI"
(<https://youtu.be/9fcfDF8SBnc>).

### Relayable-original, non-reference states

The catalogue has **no entry** for connecting / reconnecting / disconnected / error / loading /
empty states. `docs/research/open-questions.md` **Q9** resolves that these must be **designed as
Relayable-original and labelled non-reference** (the only "disconnect" evidence is the whole app
being quit, V4 3:59, and restored, V4 4:11). `full.md` §9.2 item 7 also lists them as later work.
They are **not** reproduced from a reference and must not be presented as such.

## 3. Component mapping

The mapping in the table above is the catalogue's **planned** `apps/linux/src/ui/*.rs` layout
(`ui-catalog.md`, "Implementation paths are proposed Phase-1 locations"). It is **planned, not
implemented** — `src/ui/` does not exist yet. UI-002 will create it; one writer per file.

## 4. Design tokens

The token sheet is [`src/tokens.rs`](./src/tokens.rs). Its **single source** is
`docs/research/design-tokens.md` (222 lines). No value in the sheet exists that is not stated in that
document; every token carries its `source` stamp, `confidence` label and `caveat` in the type system
(`Measured<T>`), and ranges stay ranges (`Approx`).

**Available categories**

- **Colours** (`tokens::colors`) — RGB triples measured from V1/V4 frames.
- **Geometry as raw video pixels** (`tokens::geometry::v1`, `tokens::geometry::v4`) — window bounds,
  top bar, tab pill, pane header, splits, palettes, session picker, terminal row pitch.
- **Terminal/pane surfaces as ramps** (`tokens::terminal`) — the main terminal background is a
  two-endpoint vertical ramp because the document says "do not hard-code one value".
- **Measured-but-not-app tokens** (`tokens::not_app_tokens`) — macOS traffic lights ("not app
  tokens") and the low-confidence accent glyph colour ("not a safe token").

**Deliberately absent** (`tokens::unmeasured`, carrying the document's own wording): font
family/weight, exact type sizes, motion/animation and hover timings, pixel-exact corner radii, the
V1 pane-header height, V4 window vertical bounds, terminal colour under possible translucency,
scrollbar/cursor/selection colours, Video 5 CLI text, and the traffic-light-adjacent toolbar button.
These are `not measurable` in the document and are **not** invented here.

**Geometry is namespaced per source** (`geometry::v1` at 1170×1080, `geometry::v4` at 1624×1080):
`design-tokens.md` §2.3 states the frames use different export scales and that absolute pixel values
are **not comparable across V1 and V4**. No px→point conversion and no cross-video comparison is
performed anywhere.

**Document conflict recorded, not resolved:** `design-tokens.md:210` lists "29 px palette rows"
among clone-derived numbers while `:212` gives the GoRex value as 34 px and `:204` marks the earlier
29 px estimate "wrong: 37.5 px". This is the open correction item **REF-004**. `tokens.rs` encodes
only the measured video values and flags the conflict in a doc comment; the research file
(`docs/**`) is Manager-owned and was **not** edited.

## 5. Known fidelity gaps / unverified items

The token sheet is honest about what the evidence supports. Unverified or missing:

- **No logical-pixel conversion.** All geometry is raw **video pixels of the cited frame**
  (`design-tokens.md` §2.3); the recording scale factor is not recoverable, so pixel values are not
  points.
- **V1 and V4 are not comparable** — different export scales (1170×1080 vs 1624×1080).
- **Typography, font family/weight and exact type sizes are unknown.** No frame resolves the font,
  and there is no reliable px→pt conversion.
- **Motion and hover were never captured**, so timings and hover colours are `not measurable`.
- **Terminal colour may be translucent** (wallpaper bleed-through), so the measured ramp is not a
  decided opacity.
- **Scrollbar width/colour, cursor style, selection colour and scrollback loading state are
  unmeasured** (a scrollbar exists in V1 2:03 but was not scanned).
- **Exact corner radii are only low/medium-confidence visual estimates.**
- **Video 5 was not re-captured** in the measurement pass; V5 CLI text is unknown at 1080p.
- **macOS traffic lights are not app tokens** (OS chrome; must not be copied to Linux).
- **REF-004 is an open correction item** and may change palette-row token values.

**Supported fidelity level:** the token sheet reproduces measured reference *colours* and
*proportions* within the confidence each token carries, but it does **not** support pixel-perfect
matching, and no such claim is made (`full.md` §2.4). Matching scale/weight/padding/radii and
hover/active animation requires font and motion evidence the research has not produced.

### Tokens applied in the UI-001 window

| Token | Where | Confidence |
|---|---|---|
| `colors::PANE_HEADER_SURFACE_V1` = rgb(9,30,46) | window root background | **high** |
| `colors::PRIMARY_TEXT` = rgb(191,219,228) | placeholder label colour | **medium** |

This is a deliberate token smoke test (it exercises `to_u32_0xRRGGBB` -> `gpui::rgb`), not the
UI-002 workspace UI. In the captured screenshot the flat background sampled as `srgb(10,30,45)` — a
±1 R/B render/compositing difference from the token's rgb(9,30,46).

## 6. Screenshot-test procedure

This is the real, proven procedure (UI-001 phase 1/2), not a proposal. It runs against the live
Wayland session (`XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:0`) and needs
`grim`, `hyprctl` (Hyprland) and ImageMagick's `identify`.

```sh
cd apps/linux
cargo build                                  # 1. build
bash evidence/run-window.sh                  # 2-5. launch, wait, capture, verify, terminate
```

`evidence/run-window.sh` does exactly:

1. builds nothing itself; launches `./target/debug/relayable` in the background with the session env;
2. waits 8 s and checks the process is still alive (`/proc/<pid>/status`);
3. verifies the toplevel with the compositor:
   `hyprctl clients -j > evidence/hyprctl-clients.json` and finds `title: "Relayable"`,
   `class: relayable`, and its `at`/`size`;
4. captures `grim -g "<x,y WxH>" evidence/relayable-window.png` (window-only) **and**
   `grim evidence/relayable-window-fullscreen.png` (full screen), then verifies each with `file` and
   `identify`;
5. terminates with `kill -TERM` and records `wait` exit (143 = 128+SIGTERM).

Evidence paths: `evidence/run-proof.log` (transcript), `evidence/run-proof.md` (write-up),
`evidence/relayable-window.png`, `evidence/relayable-window-fullscreen.png`,
`evidence/evidence-label-zoom.png`, `evidence/hyprctl-clients.json`,
`evidence/run-stdout-stderr.log` (empty = no panic/warning).

Known capture caveats (unavoidable, recorded, not hidden): an unrelated session **Audio**
layer-shell overlay overlaps the right edge of the captured window region, and Hyprland tiled the
window to 950×1030 (the app requests 960×600; geometry is compositor-controlled on Wayland).

## 7. `gpui = "=0.2.2"` pin and the TERM-002 remedy

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
  `libc` without re-checking. Full record: `evidence/TERM-002-remedy.md`.

## 8. Build / validate commands

```sh
cd apps/linux
cargo check                 # exit 0
cargo test                  # 9 token-sheet tests, exit 0
cargo build                 # exit 0
cargo build --release       # exit 0
cargo fmt --check           # exit 0
cargo clippy --all-targets  # exit 0
```

Recorded results: `evidence/validation.md`, `evidence/validation-commands.log`. Toolchain on this
host: cargo/rustc 1.98.1, clippy 0.1.98, rustfmt 1.9.0.
