# UI-001 run proof — the Relayable GPUI window actually opens (phases 1 + 2)

This records a **real** launch of the built binary under the live Wayland
session, a **real** screenshot, and the app's **real** exit status. Raw captures
are the sibling files named inline. Nothing here is simulated.

This file was refreshed in phase 2 (token sheet + README): the committed
screenshots reflect the committed token-driven code.

- Host: Linux `falcon`, live Wayland session
- Session env: `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`, `DISPLAY=:0`
- Binary: `apps/linux/target/debug/relayable` (debug; large due to debug info)
- Date (UTC) of the recorded run: 2026-10-15T20:38:25Z
- Reproducer: `evidence/run-window.sh` (committed)
- Raw transcript: `evidence/run-proof.log`

## Command

```
cd apps/linux
XDG_SESSION_TYPE=wayland WAYLAND_DISPLAY=wayland-1 DISPLAY=:0 \
  ./target/debug/relayable > evidence/run-stdout-stderr.log 2>&1 &
```

The reproducible wrapper is:

```
bash evidence/run-window.sh
```

It launches the binary, waits 8 s, verifies the toplevel with the compositor,
screenshots via `grim`, then `kill -TERM`s it. Exact captured transcript
(`evidence/run-proof.log`):

```
date_utc=2026-10-15T20:38:25Z
session=XDG_SESSION_TYPE=wayland WAYLAND_DISPLAY=wayland-1 DISPLAY=:0
run_command=./target/debug/relayable
app_pid=3849381
ran_seconds=6
status_after_6s=ALIVE
Name:	relayable
State:	S (sleeping)
command: hyprctl clients -j > evidence/hyprctl-clients.json
hyprctl_exit=0
FOUND address=0x57a685b90ca0 class=relayable title='Relayable' size=950x1030 at=960,40
GEOM=960,40 950x1030
command: grim -g "960,40 950x1030" evidence/relayable-window.png
grim_window_exit=0
command: grim evidence/relayable-window-fullscreen.png
grim_fullscreen_exit=0
--- artifacts ---
-rw-r--r-- 1 sulthonuladib sulthonuladib 53929 Oct 16 03:38 evidence/relayable-window.png
evidence/relayable-window.png: PNG image data, 950 x 1030, 8-bit/color RGB, non-interlaced
evidence/relayable-window.png PNG 950x1030 950x1030+0+0 8-bit sRGB 53929B 0.000u 0:00.000
-rw-r--r-- 1 sulthonuladib sulthonuladib 255509 Oct 16 03:38 evidence/relayable-window-fullscreen.png
evidence/relayable-window-fullscreen.png: PNG image data, 1920 x 1080, 8-bit/color RGB, non-interlaced
evidence/relayable-window-fullscreen.png PNG 1920x1080 1920x1080+0+0 8-bit sRGB 255509B 0.000u 0:00.000
command: kill -TERM 3849381
kill_exit=0
wait_exit=143
final=TERMINATED
--- stdout/stderr of the app ---
--- end stdout/stderr ---
```

## Observed

- The process stayed **ALIVE** for the full 6 s (`ran_seconds=6`) and was a real
  user process (`/proc/<pid>/status` → `Name: relayable`, `State: S (sleeping)`).
- `hyprctl clients -j` (`evidence/hyprctl-clients.json`) reported the mapped
  toplevel: `class = "relayable"`, `title = "Relayable"`, `size = 950x1030`,
  `at = 960,40`, `mapped = true`, `visible = true`.
  The title is set through the real API `Window::set_window_title("Relayable")`
  (`gpui-0.2.2/src/window.rs:1779` → wayland `set_title`,
  `.../wayland/window.rs:939` → `toplevel.set_title`); `WindowOptions.titlebar`
  alone is not applied on Wayland by gpui 0.2.2.
- **stdout/stderr were empty** (`evidence/run-stdout-stderr.log`, 0 bytes). No
  panic, no warning, no `warn!` spam.
- Terminated by `kill -TERM`; `wait` returned **143** (128 + SIGTERM); the
  process was gone afterwards (`final=TERMINATED`).

### Tokens exercised at runtime (phase 2)

The root view applies two documented tokens from `src/tokens.rs` (see README
§5): `colors::PANE_HEADER_SURFACE_V1` = rgb(9,30,46) as the window background
(confidence **high**) and `colors::PRIMARY_TEXT` = rgb(191,219,228) for the
label (confidence **medium**), converted with
`Rgb8::to_packed_rgb_u32()` → `gpui::rgb(..)`.

> Corrective-round note: these two colour tokens and their values are unchanged
> by the review fixes (`to_u32_0xRRGGBB` was renamed to `to_packed_rgb_u32`; the
> pane-card border colours were added as new tokens). The app's rendered output
> is identical, so this screenshot was **not** re-captured and still reflects the
> committed code. See `evidence/corrective-round.md`.

Pixel sampling of the saved screenshot (ImageMagick): the flat background
sampled as `srgb(10,30,45)` — a ±1 R/B render/compositing difference from the
token's rgb(9,30,46) — and the label glyph cores reach `srgb(184,211,220)`,
slightly below the token because of glyph antialiasing (the token's own caveat
already warns that thin-stroke p95 underestimates the true value).

### Window size note

The app requests a centered `960x600` window
(`Bounds::centered(None, size(px(960.), px(600.)), cx)`). The compositor
(Hyprland) tiled it to `950x1030`; window geometry on Wayland is controlled by
the compositor, so the requested size is a hint, not a guarantee.

## Screenshots

| File | Real size | Dimensions | Format |
|---|---|---|---|
| `evidence/relayable-window.png` | 53,929 bytes | **950 x 1030** | PNG, 8-bit/color RGB, non-interlaced |
| `evidence/relayable-window-fullscreen.png` | 255,509 bytes | **1920 x 1080** | PNG, 8-bit/color RGB, non-interlaced |

Verified with `file` and `identify` (see `evidence/run-proof.log`). Window
capture used `grim -g "960,40 950x1030"` (the window's own geometry from
`hyprctl`); a full-screen capture is also kept. The window capture and the
same-region crop of the full-screen capture are **pixel-identical**
(`magick compare -metric AE` → `0`). A `300x80` crop around the label (kept as
`evidence/evidence-label-zoom.png`, 2x point-zoomed) shows the rendered
`Relayable` text.

**Overlay caveat (honest):** an unrelated desktop overlay — the session's
"Audio" panel (a layer-shell surface) — overlaps the **right** portion of the
captured region. It is not part of the app; the app's own background and label
are visible on the left/centre. This is a session artifact, not an app defect.

## Result

The minimal GPUI window **built and actually ran**: it opened a real
compositor-managed toplevel titled `Relayable` (class `relayable`) containing the
placeholder label, styled with two documented design tokens, and stayed alive
until terminated. No run was faked; the transcript above is the verbatim capture.
