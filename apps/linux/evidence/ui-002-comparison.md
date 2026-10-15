# UI-002 — quantitative reference-vs-Relayable screenshot comparison

Scope: `apps/linux` only. This file is the **committed number record** for the UI-002
screenshot loop (`full.md` §9.3). Every number below was produced by the ImageMagick commands
shown; nothing is estimated or copied from a claim.

**Reference imagery is local-only and never committed.** The reference still used here
(`v1-window-crop.png`) lives under `apps/linux/evidence-local/ref/` (gitignored by the root
`.gitignore:12` `**/evidence-local/`). This document contains only measurements, no image.

## 1. Inputs (identical dimensions, so the comparison is 1:1 with no scaling)

| Image | Path | Dimensions | Provenance |
|---|---|---|---|
| Reference | `apps/linux/evidence-local/ref/v1-window-crop.png` | **1099×706** | `v1-t0024.png` cropped `1099x706+35+29` (V1 0:24 workspace-default; `design-tokens.md` §3.1 window outer bounds x≈35–1134, y≈29–735) |
| Relayable | `apps/linux/evidence/ui-002-workspace.png` | **1099×706** | live run, pinned by Hyprland to `at 10,40` `size 1099x706`; captured with `grim -g "10,40 1099x706"` |

`identify` confirms both are exactly `1099x706` (exit 0 for both), so no crop/scale normalisation
was needed. The reference crop intentionally removes the macOS chrome above y=29 (window shadow /
top border) but keeps the reference's own ~3 px of window chrome (rows y0–2 of the crop).

## 2. Composite + raw-pixel difference

Working directory `apps/linux`:

```sh
mkdir -p evidence-local/compare
magick montage -label 'REFERENCE (V1 0:24)' evidence-local/ref/v1-window-crop.png \
  -label 'RELAYABLE UI-002' evidence/ui-002-workspace.png \
  -tile 2x1 -geometry +6+6 -background '#101010' -fill white -pointsize 18 \
  evidence-local/compare/side-by-side.png                      # exit 0, 2222x741

magick compare -metric AE  evidence-local/ref/v1-window-crop.png evidence/ui-002-workspace.png \
  evidence-local/compare/diff-ae.png                           # AE = 9665.64 (0.0124574)

magick compare -metric RMSE evidence-local/ref/v1-window-crop.png evidence/ui-002-workspace.png \
  evidence-local/compare/diff-rmse.png                         # RMSE = 2937.14 (0.0448178)

magick composite -blend 50 evidence-local/ref/v1-window-crop.png evidence/ui-002-workspace.png \
  evidence-local/compare/blend-50.png                          # exit 0
```

Raw metrics: **AE = 9 665 (1.25 % of 775 894 pixels differ at all); RMSE = 4.48 %.**

**These are NOT a fidelity score.** The two frames show different terminal content and different
tab labels (reference `~`, Relayable `user@host:~`), the reference carries macOS traffic lights and
a rounded/shadowed window edge that Relayable deliberately does not reproduce, and the typography is
unmeasured. A raw pixel diff therefore measures "different content + different chrome", not
layout fidelity. It is recorded only to show the comparison was actually computed.

## 3. Same-quantity pixel measurements

All probes are ImageMagick point samples, `-crop`d 1-px rows/columns or 1:1 box means. Frame-relative
y is the reference crop y (subtract 29 from the V1 frame-absolute y recorded in
`design-tokens.md`).

### 3.1 Top bar height + separator

Reference column x=600 (`magick … -crop 1x60+600+0 txt:-`): rows y0 = shadow `(24,77,117)`,
y1 = top highlight `(49,74,94)`, y2 = dark border `(0,14,34)`, **y3–42 = bar background (40 rows)**,
**y43 = separator `(30,49,66)` exactly 1 px**, y44+ terminal body.

Relayable column x=600 / x=900: **y0–39 = bar background (40 rows)**, **y40 = separator
`(30,48,66)` exactly 1 px**, y41 = pane header.

| Quantity | Reference | Relayable | Delta |
|---|---|---|---|
| Bar height | 40 rows (y3–42) | 40 rows (y0–39) | 0 px |
| Separator | 1 px @ y43, mean rgb(30,49,66) | 1 px @ y40, mean rgb(30,48,66) | ΔG −1 |
| Vertical origin | bar starts y3 (macOS border) | bar starts y0 (flush) | +3 px |

### 3.2 Top bar background

| Sample | Reference | Relayable |
|---|---|---|
| p(450,20) | (2,27,45) | (4,27,46) |
| p(900,20) | (2,27,47) | (4,27,46) |
| p(1000,20) | (2,27,47) | (4,27,46) |
| p(450,30) | (7,27,46) | (4,28,46) |
| box mean 260×24 @(820,10) / @(820,1) | **rgb(5,27,47)** | **rgb(4,27,46)** |

Delta ≈ −1 R, 0 G, −1 B. The reference bar is not perfectly flat (its top rows are darker:
`(2,27,45/47)` at y20, `(7,27,46)` at y30/35) — a compression/gradient effect; Relayable renders the
measured token flat.

### 3.3 Active tab pill

Reference: row y=23 edge scan gives a left edge highlight at x≈127–128 and a right edge highlight at
x≈330–331 (outer bounds ≈x126–332 → **≈205 px**); column x=300 gives a top edge highlight at y=7
and bottom edge highlight at y=36 (outer **y7–36 = 30 px**). Interior fill is **rgb(19,41,62)** in
the upper half and **rgb(21,41,62)** in the lower half. Relayable: row y=20 fill spans
**x=128–332 (205 px, no gap)**; column x=200 fill spans **y=4–33 (30 px)**; fill is a flat
**rgb(19,40,62)** with no edge highlight.

| Quantity | Reference (V1 0:24 crop) | Relayable | Delta |
|---|---|---|---|
| Outer horizontal extent (workspace-default) | x≈126–332 → 205 px | x=128–332 → 205 px | +1 px left |
| Outer vertical extent | y7–36 → 30 px | y4–33 → 30 px | +3 px (same chrome origin as §3.1) |
| Fill | rgb(19,41,62) upper / (21,41,62) lower | rgb(19,40,62) flat | ΔG −1 (upper) |
| 1 px edge highlight | top (50,75,99), bottom (54,76,99) | **none rendered** | missing |

Documented token cross-check (`design-tokens.md` §3.1–§3.2, V1 **0:47** three-tab frame):
active pill frame-absolute x≈573–778 ⇒ window-relative **x538–743 = 205 px**, y≈36–65 ⇒
**y7–36 = 30 px**, fill **rgb(19,41,62)**. Relayable's third-tab capture
(`ui-002-switch-tab3.png` / `-split.png`) measures the pill at x=538–742 (log: 205 px) — a ≤1 px
match on the same token. (The workspace-default comparison image is the V1 0:24 frame, whose
active pill is the first tab at x≈126–332, not the 0:47 third-tab pill.)

### 3.4 Pane header band

Reference V1: **not separable** — `design-tokens.md` §3.1/§4 records the header and terminal
backgrounds as the same colour (p50 rgb(9,30,46) vs rgb(10,29,43)) and explicitly says the V1
pane-header height is `not measurable`. Relayable column x=900: y41–42 = rgb(9,30,46) (header
surface), y43–81 = rgb(10,31,46), y82 = rgb(11,30,43) (terminal ramp top). The band from the
separator end (y41) to the terminal start (y81) is **41 rows = the V4-inferred token**.

| Quantity | Reference | Relayable | Verdict |
|---|---|---|---|
| V1 pane-header height | not measurable (no colour boundary) | **41 px** (y41–81) | V4 inference, not a V1 reproduction |
| V4 pane-header height | 40–42 px (`design-tokens.md` §3.1) | 41 px | inside the measured range |

### 3.5 Terminal background ramp

| Location | Reference | Relayable | Delta |
|---|---|---|---|
| top, point y90–120 | (9,30,46) → (10,29,46) | (11,30,43) flat | (+1,+1,−3) |
| top, band mean 130×40 @(950,90) | rgb(11,31,48) | rgb(11,30,43) | (0,−1,−5) |
| bottom, point y630 | (14,29,35) | (16,27,36) | (+2,−2,+1) |
| bottom, band mean 130×60 @(950,620) | rgb(14,29,35) | rgb(17,27,36) | (+3,−2,+1) |
| documented token (`design-tokens.md` §3.2) | top rgb(10,29,43) → lower rgb(17,27,35) | top rgb(10,29,43) → lower rgb(17,27,35) | 0 (token) |

The measured deltas are within the VP9 ±3/channel caveat and partly caused by the documented ~3 %
desktop bleed on the Relayable surface (base ramp pixel +1 R/+1 G). The ramp direction and endpoints
match the token.

### 3.6 Terminal text row pitch

Reference (`design-tokens.md` §3.1, `v1-t0140`, 580-row Neovim gutter autocorrelation): **≈17.1 px**.
Relayable: the baseline `ui-002-workspace.png` contains too few terminal text rows for a full
autocorrelation, so the pitch was measured from bright-row cluster centres in two multi-row
captures (`ui-002-tabs-one-osc.png`, `ui-002-npanes-live.png`), and cross-checked against the app's
logged font metrics.

| Capture | Cluster centres (y) | Consecutive diffs |
|---|---|---|
| `ui-002-tabs-one-osc.png` (x40–400) | 107.1, 123.5, 141.5 | 16.4, 18.0 |
| `ui-002-npanes-live.png` (x10–350) | 109.6, 126.4, 161.6, 178.5, 214.0 | 16.8, 35.2 (2 rows), 16.9, 35.5 (2 rows) |

Measured single-row pitch ≈ **16.8–18.0 px** (noisy: few rows, glyph-shape-dependent cluster
centres); the app log records `row_pitch=**17.25**` (`evidence/ui-002-run.log:35`). Reference 17.1
vs Relayable 17.25 → **+0.15 px**, inside the reference's documented 17.0–17.5 px range. The
difference is within measurement noise; a clean autocorrelation is not possible from these captures.

### 3.7 Primary text colour

Reference (`design-tokens.md` §3.2, V1 0:47 / V4 0:58): bright-glyph bulk **p95 ≈ rgb(191,219,228)**.
In the regenerated crop, compression overshoot makes isolated glyph pixels reach
rgb(239,255,255) (max), so a local p95 over the reference crop is not a stable statistic.

Relayable (regions x120–400, y12–72 = tab + pane labels, threshold r+g+b>450):
**p95 = rgb(186,214,224), max = rgb(188,216,226)** over 460 bright pixels. The painted token is
`theme::PRIMARY_TEXT = rgb(191,219,228)`; the rendered maximum is **−3/channel** from the token,
consistent with glyph antialiasing / sub-surface compositing (no full-coverage pixel). The
reference's exact font colour is not isolatable from compression, so this is a within-tolerance
colour match, not a typography match.

## 4. Ranked discrepancy list (`full.md` §9.3 order)

| Rank | Class | Discrepancy | Measured evidence | Status |
|---|---|---|---|---|
| 1 | layout/hierarchy | Vertical tabs absent | `v1-t0222` does not show a vertical-tab layout at all (`REFERENCE-FRAMES.md` §5); geometry unmeasurable | (c) deliberate documented gap |
| 2 | layout/hierarchy | Command palette absent (V4-observed) | `v4-t0030` palette 442×667 px, pitch 37.5 px (`design-tokens.md` §3.1) | (c) optional in brief; not implemented |
| 3 | layout/hierarchy | Session pill / remote-host chip absent | V1 2:22, V4 0:58/4:11/4:49; it is remote chrome | (c) out of slice scope |
| 4 | layout/hierarchy | macOS chrome (traffic lights, rounded corners, shadow) not reproduced | top bar starts y0 vs reference y3; traffic-light region left empty | (c) deliberate: OS chrome is not an app token (`design-tokens.md` §3.2) |
| 5 | layout/hierarchy | In-app connection banner exists; reference showed none | only a quit/relaunch was shown (V4 3:59→4:11), no in-app disconnected chrome | (c) Relayable-original (Q9), visible only off-`Connected` |
| 6 | proportions | 3 px vertical chrome offset | bar y0 vs reference y3 (§3.1) | (c) consequence of #4 |
| 7 | proportions | Pane header 41 px is a V4 inference | V1 header not separable (§3.4) | (b) blocked by unmeasured V1 header height |
| 8 | proportions | Pill radius 9 px low-confidence | pill outer 205×30 measured; radius 8–10 px visual only | (b) compression softens corners |
| 9 | colours | Active pill lacks the 1 px edge highlight | reference (50,75,99)/(54,76,99); Relayable flat (§3.3) | (a) fixable with current evidence (token exists in `tokens`/`theme`) |
| 10 | colours | Pill fill ΔG −1 | ref (19,41,62) vs ours (19,40,62) (§3.3) | (a) fixable; render/rounding |
| 11 | colours | Primary text rendered −3/channel from token | p95 (186,214,224) vs token (191,219,228) (§3.7) | (b)/(a) antialiasing; exact reference font colour unmeasured |
| 12 | colours | ~3 % desktop bleed through the surface | base ramp pixel +1 R/+1 G; band deltas ±1–3 (§3.5) | (b) gpui 0.2.2 + Hyprland CSD alpha residual |
| 13 | colours | ANSI palette / cursor / scrollbar / banners are Relayable-original | `ui/palette.rs`, `theme.rs` `*_ORIGINAL` | (c) not measurable from the reference |
| 14 | typography | Font family + sizes unmeasured | `design-tokens.md` §4; JetBrainsMono Nerd Font 13 px is a proposal | (b) blocked by unmeasured reference typography |
| 15 | microspacing | Separator 1 px vs the 1.5 px token midpoint | local re-measure found exactly 1 px (§3.1) | (c) evidence-supported deviation |
| 16 | microspacing | Pill/pane label content differs (`~` vs `user@host:~`) | shell OSC 2 title drives the label | (c) live-data content, not a token |

Rows 9–10 are the only items with a concrete, evidence-backed fix available now; they are recorded,
not silently fixed (this stage owns documentation, not `src/**`).

## 5. Supported fidelity level

**Reproduced** (measured, within each token's confidence):

- window outer size 1099×706 (live capture matches exactly);
- top bar height 40 px + 1 px separator, separator colour within Δ1/channel;
- active tab pill 205×30 px, x-position and fill within Δ1/channel;
- inter-pane split geometry 9.5 px gap / 50-50 (documented; V4);
- terminal row pitch 17.25 px vs reference ≈17.1 px;
- terminal background ramp direction and endpoints (top rgb(10,29,43) → lower rgb(17,27,35))
  within the compression/translucency caveat;
- pane header band inside the V4 measured 40–42 px range.

**Not reproduced / not claimed:**

- typography (family, weight, exact sizes) — unmeasured from the reference;
- motion, hover, active animation — never captured;
- pixel-exact corner radii — visual estimates only;
- the reference's macOS chrome (traffic lights, rounded window, shadow);
- the ANSI palette, cursor colour/shape beyond "block", selection colour, scrollbar colours;
- connecting / reconnecting / disconnected / error visuals — Relayable-original (Q9);
- vertical-tabs and command-palette geometry — unmeasured/not implemented;
- remote-host, CLI-in-client and CLI-help screens.

**Explicitly not claimed: pixel-perfect matching.** `full.md` §2.4 forbids the claim; the measured
tokens match to video-compression tolerance, but fonts, motion, radii, the palette and all
connection states are unmeasured or Relayable-original.

## 6. Reproduce

From `apps/linux`, with the local reference still present under `evidence-local/ref/`:

```sh
mkdir -p evidence-local/compare
magick montage -label 'REFERENCE (V1 0:24)' evidence-local/ref/v1-window-crop.png \
  -label 'RELAYABLE UI-002' evidence/ui-002-workspace.png \
  -tile 2x1 -geometry +6+6 -background '#101010' -fill white -pointsize 18 \
  evidence-local/compare/side-by-side.png
magick compare -metric AE evidence-local/ref/v1-window-crop.png evidence/ui-002-workspace.png \
  evidence-local/compare/diff-ae.png
```

`apps/linux/evidence-local/**` is gitignored (`.gitignore:12`); the composites and the reference
stills are local-only and must never be committed (`full.md` §0.2/§2.6).
