# Relayable reference design tokens — measured from higher-resolution frames

Status: **research artifact, evidence only.** This file records pixel measurements taken from
re-captured, higher-resolution reference frames (1080p). It does not define implementation
constants and it does not replace `docs/research/ui-catalog.md`; it supplies measured values the
lead can later reconcile into that catalogue and into `apps/linux/README.md` (`full.md` §9.1).

Scope of this pass: Video 1 main workspace + Video 4 remote/split screens. Video 5 was not
re-captured (time-box); its 480p evidence remains the only evidence for CLI text frames.

## 1. Evidence location and hygiene

- New frames: `docs/research/evidence/hires/video1/**`, `docs/research/evidence/hires/video4/**`
  (also `.../crops/**` for 2–3× inspection crops).
- `git check-ignore -v docs/research/evidence/hires/x.png` → `.gitignore:11:/docs/research/**/evidence/`
  (exit 0), so these paths are gitignored and were never staged or committed. No image file is part
  of any commit.
- Scratch downloads: `~/.cache/relayable-research/hires/**` (gitignored by nature; outside the repo).
- All frames are analysis-only; the reference footage is not redistributable (`full.md` §2.3, §2.6).

## 2. Capture method (exact commands, exit codes)

Format: **VP9 1080p60** (`format_id 303`, `webm_dash`, direct HTTPS). The earlier pass's `HTTP 403`
on an AV1 720p selector was avoided by selecting the explicit VP9 format id; no 403 occurred in
this pass. Audio was not downloaded (not needed for stills).

```text
# Video 1 section 0:20–2:30 (exit 0; vp9 1170x1080@60; 25,357,498 bytes)
yt-dlp -f 303 --download-sections '*0:20-2:30' \
  -o "$HOME/.cache/relayable-research/hires/v1/%(id)s.%(ext)s" 'https://youtu.be/XyxShCZrGiw'
# Video 4 section 0:25–1:05 (exit 0; vp9 1624x1080@60; 6,257,363 bytes)
yt-dlp -f 303 --download-sections '*0:25-1:05' \
  -o "$HOME/.cache/relayable-research/hires/v4/a-%(id)s.%(ext)s" 'https://youtu.be/PdwTjSBW6Y8'
# Video 4 section 4:00–4:58 (exit 0; vp9 1624x1080@60; 10,317,699 bytes)
yt-dlp -f 303 --download-sections '*4:00-4:58' \
  -o "$HOME/.cache/relayable-research/hires/v4/b-%(id)s.%(ext)s" 'https://youtu.be/PdwTjSBW6Y8'
```

### 2.1 Section-offset calibration (important)

`yt-dlp --download-sections` keeps keyframe pre-roll: the cut file's `t=0` is **earlier** than the
requested start, so a naive `-ss <original time>` extracts the wrong moment. The offset was
measured, not assumed: each 480p full-video file already in the cache
(`~/.cache/relayable-research/1/dl.mp4`, `.../4/dl.mp4`) was compared frame-for-frame (MSE, both
scaled to 260×240, hires decoded at 10 fps) against the hires section files.

| Section | Requested | Measured mapping | Anchors (MSE) |
|---|---|---|---|
| V1 | 0:20–2:30 | `file_t = orig − 17.2 s` | 0:24→6.8, 0:47→29.8, 1:40→82.8, 2:22→124.7 (best MSE 5.3–12.3) |
| V4a | 0:25–1:05 | `file_t = orig − 21.8 s` | 0:30→8.2, 0:58→36.2 (MSE 2.8–3.8) |
| V4b | 4:00–4:58 | `file_t = orig − 238.1 s` | 4:00→1.9, 4:11→12.9, 4:49→50.9 (MSE 2.1–4.3) |

Stills extracted (all `ffmpeg` exits 0):

```text
ffmpeg -hide_banner -loglevel error -ss <file_t> -i <section>.webm -frames:v 1 \
  -compression_level 3 -y docs/research/evidence/hires/<video>/<name>.png
```

### 2.2 Measurement method

- Frame dimensions: `ffprobe -v error -select_streams v -show_entries stream=width,height -of csv=p=0 FRAME`.
- Boundary/colour scans: decoded the PNG to `rawvideo rgb24` via `ffmpeg` into Python/numpy and
  printed per-row/per-column runs or per-row medians (`/tmp/scan.py`, `/tmp/measure.py`,
  `/tmp/rowprof.py`, scratch only, not committed). `p50` below = median of the sampled box.
- 2–3× nearest-neighbour crops (`magick -crop ... -resize 200%/300%`) were used to visually resolve
  boundaries the colour scans could not separate (e.g. the V4 pane-header/body line).
- **Colour caveat:** frames are VP9-compressed, yuv420p/tv-range conversions to RGB. Flat, large
  regions (backgrounds, selection fills) are reliable to ~±3 per channel; 1–2 px lines and thin
  glyph cores are much less reliable and are reported as ranges. Where available the same element
  was sampled in two frames/videos and the variance is stated.

### 2.3 Scale assumption (ASSUMPTION — not measured)

All values are **video pixels of the cited frame**. The recordings are macOS Retina screen captures
downscaled into 1080p exports; the logical window size and the recording scale factor are not
recoverable from the frames. Therefore:

- No logical-pixel/point conversion is performed anywhere in this document.
- V1's frame is 1170×1080 and V4's is 1624×1080, so **absolute pixel values are not comparable
  between the two videos** (different export scale). Within-frame ratios/ramps are comparable with
  care. This differs from the 480p evidence, where both happened to be 480-px tall.

### 2.4 Captured frames of record

| Video | Orig. time | `-ss` used | Frame path (repo-relative) | Dimensions |
|---|---|---|---|---|
| V1 | 0:24 | 6.8 | `docs/research/evidence/hires/video1/v1-t0024.png` | 1170×1080 |
| V1 | 0:42 | 24.8 | `.../video1/v1-t0042.png` | 1170×1080 |
| V1 | 0:47 | 29.8 | `.../video1/v1-t0047.png` | 1170×1080 |
| V1 | 0:55 | 37.8 | `.../video1/v1-t0055.png` | 1170×1080 |
| V1 | 0:58 | 40.8 | `.../video1/v1-t0058.png` | 1170×1080 |
| V1 | 1:33 | 75.8 | `.../video1/v1-t0133.png` | 1170×1080 |
| V1 | 1:40 | 82.8 | `.../video1/v1-t0140.png` | 1170×1080 |
| V1 | 2:03 | 105.8 | `.../video1/v1-t0203.png` | 1170×1080 |
| V1 | 2:22 | 124.8 | `.../video1/v1-t0222.png` | 1170×1080 |
| V4 | 0:30 | 8.2 | `.../video4/v4-t0030.png` | 1624×1080 |
| V4 | 0:58 | 36.2 | `.../video4/v4-t0058.png` | 1624×1080 |
| V4 | 4:11 | 12.9 | `.../video4/v4-t0411.png` | 1624×1080 |
| V4 | 4:49 | 50.9 | `.../video4/v4-t0449.png` | 1624×1080 |

Inspection crops (gitignored, scratch-quality): `.../crops/v1-0047-tabstrip.png` (3×, tab strip),
`.../crops/v1-0140-picker.png` (3×, session picker), `.../crops/v4-0058-paneheader.png` (2×,
pane header/card edge).

## 3. Token measurements

Confidence: **high** = clean boundary/fill in a legible frame, reproducible in a second frame;
**medium** = one frame or compression-limited; **low** = visual judgement. "Video px" = pixels in
the cited 1170- or 1624-wide frame.

### 3.1 Geometry

| Token | Measured value | Source (video + timestamp + frame) | Method | Conf. | Caveats |
|---|---|---|---|---|---|
| V1 window outer bounds | x ≈ 35–1134 (≈1099 px), y ≈ 29–735 (≈706 px) | V1 0:24 `v1-t0024.png` | row scan y=300; col scan x=200 on raw RGB runs | high | macOS rounded corners + shadow blur ±1–2 px at the corners |
| V1 window outer aspect | ≈ 1.56 (14:9) | V1 0:24 `v1-t0024.png` | 1099/706 | medium | video-frame aspect is 1170/1080; the window is not the frame |
| V1 window inner content | x ≈ 43–1124 (≈1081), y ≈ 32–728 (≈697), aspect ≈ 1.55 | V1 0:24 `v1-t0024.png` | same scans (first/last non-wallpaper rows/cols) | medium | 1–2 px border ambiguity |
| V4 window width | x ≈ 34–1590 (≈1556 px) | V4 0:58 `v4-t0058.png` | row scan y=400 | medium | right edge over wallpaper; bottom edge not measured (presenter PiP + rounded corner) |
| Top bar height (V1) | y 32→72 = **40 px**; separator 1–2 px at y 72–73 | V1 0:24/0:47 `v1-t0024.png`, `v1-t0047.png` | col scans x=150/275/600 | high | measured at 1170-px frame scale |
| Header region (V4) | window top y≈25–27 → focused card top y≈84–88 → ≈57–61 px | V4 0:58 `v4-t0058.png` | col scan x=300; visual 2× crop | medium | cannot tell bar height vs bar+outer padding without a V1-equivalent card |
| Active tab pill (V1) | **205 × 30 px** (x ≈ 573–778, y ≈ 36–66) | V1 0:47 `v1-t0047.png` | col scan x=610; row scan y=51 x=500–800 | high | one frame only; pill width depends on tab count/label |
| Tab pill corner radius | ≈ 8–10 px (visual) | V1 0:47 `v1-t0047.png`, crop `v1-0047-tabstrip.png` | 3× nearest-neighbour crop | low | compression softens corners |
| Inactive tab | no pill fill; top-bar background; 1–2 px vertical divider between tabs | V1 0:47 `v1-t0047.png` | row scan y=51 x=140–500 (divider x≈367, rgb(57,73,95)) | medium-high | divider may be a per-item separator; not verified for >2 tabs |
| Pane header height (V4) | ≈ **40–42 px** (card inner top y≈88 → boundary y≈128–130) | V4 0:58 `v4-t0058.png`, crop `v4-0058-paneheader.png` | 2× crop visual + row profiles | medium | no colour step at the boundary; the line is faint |
| Pane header height (V1) | **not measurable** — header and terminal backgrounds are the same colour (p50 rgb(9,30,46) vs rgb(10,29,43)) | V1 0:24 `v1-t0024.png` | row-median profile x=700–1100, y=68–128 | high (that it is not separable) | do not reuse the 480p "30–36 px" estimate; V4 measurement is the better evidence |
| Pane card border / focused | 2–3 px, rgb ≈ (31,82,115) | V4 0:58 `v4-t0058.png` | row scan y=400 x=44–48; col scan x=300 y=84–88 | medium-high | includes antialias |
| Pane card border / unfocused | 2–3 px, rgb ≈ (17,54,71) | V4 0:58 `v4-t0058.png` | row scan y=400 x=816–819 | medium | one frame |
| Inter-pane gap (V4) | ≈ **8–11 px** (card outer x≈808 → x≈816) | V4 0:58 `v4-t0058.png` | row scan y=400 | high | equals gap between card outer edges; not the drag target |
| Outer margin window→card (V4) | ≈ 0–5 px left, ≈ 10–12 px right (y=400) | V4 0:58 `v4-t0058.png` | row scan y=400 | low | asymmetric reading may be window-edge shading/scrollbar |
| Split ratio at creation | pane1 ≈ 757 px vs pane2 ≈ 767 px (≈50/50) | V4 0:58 `v4-t0058.png` | row scan y=400 content spans | medium-high | consistent with the 480p observation and the spoken "split" action; exact ratio depends on padding definition |
| Top-bar→palette offset (V4) | palette top y≈124, window top y≈25 → ≈ **99 px** | V4 0:30 `v4-t0030.png` | col scan x=700, y=100–830 | medium | panel float offset; moveable? |
| Command palette size | ≈ **442 × 667 px** (x 590–1032, y 124–791) | V4 0:30 `v4-t0030.png` | row scan y=400; col scan x=700 | high | outer bounds incl. border |
| Command palette row pitch | ≈ **37.5 px** (11 consecutive text rows; range 36.0–38.0) | V4 0:30 `v4-t0030.png` | bright-row clustering, box x610–990 | high | group header `Pane` breaks the rhythm (43/34 px gaps) |
| Command palette selected row height | ≈ 35 px (y 184–218) | V4 0:30 `v4-t0030.png` | col scan x=700 (mint run) | high | |
| Session picker popup size | ≈ **183 px wide** (x 117–300), ≈ 137–142 px tall (y ≈ 68–210) | V1 1:40 `v1-t0140.png` | row scan y=150; col scan x=200; lit-row clustering | medium | bottom edge soft (shadow) |
| Session picker rows | search field ≈ 27–30 px; list row ≈ 35 px; row pitch ≈ 35–40 px | V1 1:40 `v1-t0140.png` | col x=200 runs; bright rows y 83–205 | medium | cursor position changes row contents frame-to-frame |
| Terminal row pitch (cell height) | ≈ **17.0–17.5 px** (mean ≈17.25; 49 rows sampled) | V1 1:40 `v1-t0140.png` | bright-row clustering of Neovim line numbers, box x62–98, y90–670 | high | native terminal grid, stable across 40+ rows |

### 3.2 Colours

| Token | Measured value (p50 unless noted) | Source | Method | Conf. | Caveats |
|---|---|---|---|---|---|
| Top bar background | rgb(4,27,46); V1 t0047 p50 (3,27,46) | V1 0:24 `v1-t0024.png`; V1 0:47 `v1-t0047.png` | box x500–900 y40–68; box x480–560 y40–62 | high | two frames agree ±1 |
| Active tab pill fill | rgb(19,41,62) | V1 0:47 `v1-t0047.png` | box x640–665 y44–62 p50, p5=(17,41,60) p95=(21,41,63) | high | flat region, low variance |
| Active tab pill edge/highlight | rgb ≈ (50,75,98) top, (54,76,99) bottom | V1 0:47 `v1-t0047.png` | col scan x=610 y36–38/65–66 | medium | 1–2 px line; could be border or inner highlight |
| Pane header/surface bg (V1) | rgb(9,30,46), same as terminal top (rgb(10,29,43)) | V1 0:24 `v1-t0024.png` | box x400–1000 y80–102; band x300–700 y120–200 | high | no visible separation between header and terminal |
| Main terminal background | vertical ramp: top ≈ rgb(10,29,43) → lower ≈ rgb(17,27,35) (smooth, no step) | V1 0:24 `v1-t0024.png` | 4 band p50s along y=120→630 | medium | possibly wallpaper bleed-through (window may be translucent); do **not** hard-code one value without deciding opacity. V4 panes show a similar but not identical ramp |
| Focused pane surface (V4) | header rgb(0,36,53); body ramp rgb(8,34,45)→rgb(7,31,36) | V4 0:58 `v4-t0058.png` | row medians x550–750; band p50s | medium-high | teal-dominant; one frame (4:49/4:11 used only as qualitative cross-check) |
| Unfocused pane surface (V4) | header rgb(6,27,38); body ramp rgb(7,26,34)→rgb(9,25,28) | V4 0:58 `v4-t0058.png` | same boxes on right pane | medium-high | focus shown by ~+7 G / +8–15 B vs unfocused |
| Palette panel background | rgb(10,29,41) | V4 0:30 `v4-t0030.png` | box x650–900 y250–258 | high | flat |
| Palette search field | rgb(31,54,70) | V4 0:30 `v4-t0030.png` | box x660–900 y145–160 | medium | includes placeholder antialias in p95 only |
| Palette border | rgb ≈ (71–102,102–116,113–125); bottom edge (85,109,113) | V4 0:30 `v4-t0030.png` | row scan y=400 x590–594/1030–1033; col x=700 y787–789 | medium | edge antialias; treat as one light neutral border token |
| Session picker background | rgb ≈ (14,28,39); search area p50 (15,27,39) | V1 1:40 `v1-t0140.png` | box x140–280 y75–90 | medium | darker than command palette (different frames/scales) |
| Session picker border | rgb ≈ (80,97,111) | V1 1:40 `v1-t0140.png` | row scan y=150 x117–120 | medium | 1–2 px, glow-like |
| Top-bar separator (V1) | rgb(30,49,66), 1–2 px | V1 0:24/0:47 `v1-t0024.png`, `v1-t0047.png` | col scans x=150/600 y72–73 | high | |
| Tab divider (inactive) | rgb(57,73,95), 1–2 px | V1 0:47 `v1-t0047.png` | row scan y=51 x366–369 | medium | visible between two inactive tabs only |
| Primary text | near-white, bright-glyph samples up to rgb(230,246,255); bulk p95 ≈ (191,219,228) | V1 0:47 `v1-t0047.png` (active tab `~`); V4 0:58 `v4-t0058.png` (table header) | p95/p-max over text boxes | medium | thin-stroke p95 underestimates the true value; exact font colour not isolatable from compression |
| Secondary text | dim blue-gray; clock/date p95 rgb(108,111,159) | V1 0:24 `v1-t0024.png` | p95 over x900–1110 y105–130 | low-medium | antialiased thin text; compression lowers p95 |
| Terminal prompt (V1 local) | light periwinkle-blue accent; p95 rgb(125,128,207) | V1 0:24 `v1-t0024.png` | p95 over x45–120 y105–135 | low-medium | glyph pixels only; small sample |
| Remote prompt (V4) | green/teal; p95 rgb(128,203,167) | V4 0:58 `v4-t0058.png` | p95 over x60–270 y155–185 (`[mitchellh@hill:~]$`) | medium | matches the visible green prompt |
| Active/selected — horizontal tab | fill rgb(19,41,62) vs bar rgb(4,27,46) | V1 0:47 `v1-t0047.png` | see above | high | selection token for tabs |
| Active/selected — session picker row | **blue** fill p50 rgb(68,156,255) | V1 1:40 `v1-t0140.png` | box x135–280 y100–120 p50, p5=(7,18,43) p95=(95,177,255) | high | large flat fill; the `✓ Demo` row |
| Active/selected — command palette row | **mint/green** fill p50 rgb(33,150,137); brighter edge samples (54,171,158) | V4 0:30 `v4-t0030.png` | col scan x=700 y185–218; box p50 | high | confirms V1-blue vs V4-mint difference at 1080p |
| Accent — terminal glyph in tabs/header | mixed green; bright samples rgb(33,94,39)→(208,255,214) | V1 0:47 `v1-t0047.png`; V1 1:40 `v1-t0140.png` | pixel transitions x592–594, x181–185 | low | small glyphs, heavy antialias; not a safe token |
| macOS traffic lights | rgb(254,79,84) / (248,190,0) / (42,190,75) | V1 0:47 `v1-t0047.png` | row scan y=50 x54–105 | high but **not app tokens** | OS chrome; Relayable Linux must not copy these |

## 4. `not measurable from frames`

- **Font family and weight** — cannot be inferred from pixels; no frame resolves the terminal font.
- **Exact type sizes** — only glyph pixel extents are measurable (Neovim digits ≈ 9 px tall at
  V1's 1170-px frame; V4 prompt glyph ≈ 12–13 px cap height in the 2× crop). There is no reliable
  px→pt conversion (see §2.3), and line-height alone does not determine type size.
- **Animation/interaction timings** — no motion analysis was done; hover-only appearances were
  never captured (catalogued as not observed in `ui-catalog.md`), so hover colours/timings are
  `not measurable`.
- **Pane header height in V1** — no colour boundary between header and terminal body (§3.1). The
  V4 measurement (40–42 px) is the evidence to use.
- **V4 window vertical bounds** — window bottom edge not measured this pass (presenter PiP region
  and rounded corners); only width and header-region height were measured.
- **Pixel-exact corner radii** for window, cards, pills, and popups — only visual estimates
  (section 3.1 lists the pill and card figures with low/medium confidence); sub-pixel antialiasing
  over compressed video does not support better.
- **Exact terminal colour values under translucency** — the measured ramp may be partly the desktop
  wallpaper showing through; whether the app's terminal is translucent cannot be decided from a
  single recording.
- **Scrollbar width/colour, cursor style, selection colour, scrollback loading state** — not
  measured from the new stills (scrollbar exists in V1 2:03 but was not scanned here).
- **Video 5 CLI text** — not re-captured in this pass; no new 1080p measurements for `cli-*` entries.
- **Traffic-light-adjacent toolbar button** — a macOS chrome item (seen at V1 0:47 left of the
  tabs) has no confirmed role; not a Relayable token.

## 5. Reconciliation with `ui-catalog.md`

No existing file was edited; this table is for the lead.

| `ui-catalog.md` entry | Tokens this measurement supports | Earlier estimate it replaces/quantifies |
|---|---|---|
| `desktop/workspace-default` | window outer/inner bounds and aspect 1.55–1.56; top bar 40 px; top bar bg rgb(4,27,46) | "top bar height ≈ 3.5–4.5% of window height" → measured 40/706 ≈ 5.7% |
| `desktop/tab-strip` | active pill 205×30 px, fill rgb(19,41,62), edge rgb(50,75,98); inactive flat + 1–2 px divider rgb(57,73,95); radius ≈8–10 px | "tab pill height ≈ 35–40 px at 480p upscale" → **measured 30 px, earlier estimate too large** |
| `desktop/pane-header` | V4 header ≈ 40–42 px; focused card border rgb(31,82,115) and surface rgb(0,36,53)→body ramp; unfocused border rgb(17,54,71), surface rgb(6,27,38) | "header height ≈ 30–36 px at 480p" → **measured ≈40–42 px**; focused/unfocused difference now quantified |
| `desktop/terminal-body` | row pitch 17.0–17.5 px; background ramp rgb(10,29,43)→rgb(17,27,35); primary text near-white; V1 prompt p95 rgb(125,128,207) | "background colours unmeasured" → measured, but with translucency caveat |
| `desktop/session-picker` | popup 183 px wide × ≈140 px tall; row ≈35 px; search field ≈27–30 px; selected fill rgb(68,156,255); bg rgb(14,28,39), border rgb(80,97,111) | "popup width well under half the window" → 183/1081 ≈ 17 % of window width |
| `desktop/command-palette` | panel 442×667 px (28.4 % of window width); top offset ≈99 px; row pitch 37.5 px; selected fill mint rgb(33,150,137); panel bg rgb(10,29,41); border rgb(85–102,109–116,113–125) | "≈28 % width (~390 px in 1448 px capture)" → consistent, now exact at 1624 px; "row pitch ≈29 px" → **wrong: 37.5 px** |
| `desktop/split-layout` | gap 8–11 px; ~50/50 split; focused/unfocused surfaces and borders | "gap ≈ 8–12 px at 480p" → consistent at 1080p (8–11 px) |
| `desktop/remote-host` | same chrome tokens; remote prompt green p95 rgb(128,203,167) | colours previously unmeasured |
| `desktop/vertical-tabs` | no new measurement this pass (frame captured at V1 2:22 but not scanned) | none |
| `desktop/reconnect-after-quit` | 4:11/4:49 frames re-captured; no new visual token beyond split/remote chrome | none |

**GoRex values not used.** No clone-derived number (33 px pane header, 14 px tab radius, 34 px
palette rows, 8 px gaps, 26×26 buttons, `#1e1e22` terminal bg) was adopted. Where video measurement
now disagrees with a clone-derived value (pane header 40–42 px vs GoRex 33 px; palette rows 37.5 px
vs GoRex 34 px), the video values above stand on their own evidence and GoRex is not cited as
corroboration. (Correction: an earlier pass listed "29 px palette rows" here; 29 px was this
project's own video estimate later corrected to 37.5 px in the `desktop/command-palette` row above,
not a clone-derived value — the clone value is GoRex's 34 px.)

## 6. Validation record

- Downloads and stills: all commands in §2, exit codes recorded in the same section; no 403 or
  format failure in this pass.
- This file is the only repository path written; frames and crops live under gitignored
  `docs/research/evidence/` and `~/.cache/relayable-research/`.
- Media check: `git status --porcelain | grep -iE '\.(png|jpg|jpeg)$'` was empty after commit.
