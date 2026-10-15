# Relayable UI research — open questions and fidelity risks (Phase 0, Wave 2)

Companion to `docs/research/ui-catalog.md`. Sources: `docs/research/video-review.md` (commit
`f067f92`, primary) and `docs/research/gorex-review.md` (commit `a7e14a3`, supplemental,
always `[clone-derived]`). Nothing here is evidence by itself; each question names the artefact
that would close it and the catalogue entry it blocks.

## Unresolved visual/interaction questions

**Q1 — Exact palette/selection colour and terminal theme.**
Current confidence: low. No colour was sampled; selections appear blue in V1, mint/green in V4 and
pink in V5 (the last without a captured frame path). Blocks: `desktop/command-palette`,
`desktop/session-picker`, `desktop/terminal-body`.
Resolver: a lossless/higher-resolution frame or a real-product screenshot with a picker-readable
palette, ideally V1 1:40, V4 0:30 and a V5 palette frame.

**Q2 — Typography, spacing, border and radius values.**
Current confidence: low (`video-review.md` §1.3 explicitly did not measure). Blocks: all 13 entries.
Resolver: 1080p+/lossless frames at a known window size, or an official screenshot; then a token
sheet per `full.md` §9.1. GoRex's numbers (`[clone-derived]`: 44 px title, 33 px pane header,
8 px gaps, 12 px radius, JetBrains Mono 11.6) must not be assumed equal to REX.

**Q3 — Are `desktop/session-picker`, `desktop/command-palette` and the V1 2:22 compact menu the
same component?** Current confidence: medium that they are distinct (placeholders and row sets
differ: `Filter or create...` vs `Search commands...` vs the two-row `Switch to Vertical Tabs`
menu). Blocks: `desktop/session-picker`, `desktop/command-palette`, `desktop/vertical-tabs`.
Resolver: a continuous higher-resolution clip of the V1 1:40 and 2:22 interactions showing the
open action and the full list.

**Q4 — Functional keyboard bindings for non-macOS.**
Current confidence: low. All captured bindings are macOS glyphs; GoRex's Linux remaps are
`[clone-derived]`. Blocks: `desktop/tab-strip`, `desktop/split-layout`, `desktop/command-palette`,
`desktop/vertical-tabs`.
Resolver: official REX docs/help for Linux clients, or a Linux demo capture; otherwise define
Relayable's own bindings and document the deviation.

**Q5 — Prompt badge (`36` in V4, `31` in V5) after `~/D/ghostty`.**
Current confidence: low; meaning unknown (duration/exit code/block id/prompt theme). Blocks:
`desktop/terminal-body`, `desktop/cli-client-terminal`.
Resolver: a legible prompt close-up or official shell-integration docs. Do not recreate until known.

**Q6 — Vertical tabs sidebar geometry and row states.**
Current confidence: low (rows are small at 480p). Blocks: `desktop/vertical-tabs`.
Resolver: higher-resolution V1 2:22 frame (or clip of switching layouts), showing sidebar width,
icon/label layout, active/hover row treatment.

**Q7 — Tab label derivation rules and active-pill styling.**
Current confidence: medium-high for content, medium for rules. `~/Documents/ghostty` duplicates get a
numeric suffix `-1`; program names (`btop`, `htop`, `nvim`) alternate with cwd strings; V4 uses
`mitchellh@hill: ~`. Blocks: `desktop/tab-strip`.
Resolver: a legible tab-bar crop at every transition in V1 0:42–1:40 plus official docs on tab
naming; or a real-product screenshot.

**Q8 — Session chip: role of the leading glyph and the V5 `Ghostty` label.**
Current confidence: medium. V1 `Demo`, V4 `drifting-cedar`/`Demo` + endpoint subtitle,
`cosmic-summit` + `This Mac`; V5 `Ghostty` (role unconfirmed — session name vs window title).
Blocks: `desktop/session-pill`, `desktop/remote-host`.
Resolver: higher-res frames of the chip being clicked/renamed, plus the V4 0:30 `Change Session`
interaction.

**Q9 — In-app connecting/reconnecting/disconnected/error visuals.**
Current confidence: none — these states were never shown; the only "disconnect" evidence is the app
being fully quit (V4 3:59) and restored (V4 4:11). Blocks: `desktop/reconnect-after-quit`,
`desktop/remote-host`.
Resolver: a recorded failure/reconnect moment in any REX demo, official docs, or an explicit
product decision by the lead; otherwise design these as Relayable-original and mark them
non-reference in `apps/linux/README.md`.

**Q10 — Scrollback loading state while history streams in.**
Current confidence: low (`video-review.md` §3.1 records the spoken behaviour at V2 5:00–5:35:
"on a slow link the user can see a loading state while scrolling back"). No frame captured. Blocks:
`desktop/terminal-body`.
Resolver: a captured slow-scrollback frame or official docs; otherwise mark the loading visual
non-reference.

**Q11 — Split orientation, resize affordances and ratio.**
Current confidence: medium for side-by-side 50/50 creation; low for resize/drag/stacked split.
Palette has `Split Pane Left/Right/Up` (V4 0:30) but only horizontal splits were shown. Blocks:
`desktop/split-layout`.
Resolver: a frame of `Split Pane Up`, a divider drag, or the resize shortcuts; GoRex's drag
indicator/clamps are `[clone-derived]`.

**Q12 — Palette shortcut glyph readings.**
Current confidence: low-medium; several strings (`⇧⌘N`, `⌘K`, `⌘1`, `⌘2`, `◇⌘←/→/↑`, `⇧⌘R`) were
read from one 480p frame each and `video-review.md` §8.3 explicitly asks for verification. Blocks:
`desktop/command-palette`, `desktop/tab-strip`.
Resolver: higher-resolution captures of V4 0:30 and V1 2:22; official docs/help.

**Q13 — `Go to Director...` label truncation and its remote/local behaviour.**
Current confidence: medium-high for the action, medium for the exact label (truncated in the frame).
Blocks: `desktop/command-palette`, `desktop/remote-host`.
Resolver: a legible palette frame; the V4 4:49 interaction shows the action working but not the
full label.

**Q14 — V5 palette existence/frame.** `video-review.md` §2.3 states the selection fill is pink in
V5, but no V5 palette frame path is catalogued. Blocks: palette colour decisions (Q1). Resolver:
identify and review the actual V5 palette frame(s) from `docs/research/evidence/video5/**`.

## Top 10 highest-impact fidelity discrepancies/risks (`full.md` §2.7 exit gate)

Ordered by impact on the Relayable UI milestone (geometry first, then behaviour, then colour).

| # | Discrepancy / risk | Video evidence | What would close it |
|---|---|---|---|
| 1 | **No measured geometry anywhere** — all panel widths, header heights, gaps, radii are unmeasured; implementing from 480p estimates risks a wrong shell. | All V1/V4 frames; `video-review.md` §1.3 | Higher-resolution/lossless frames at a known window size; a real-product screenshot; then §9.1 token sheet. Blocks every entry. |
| 2 | **Colour/typography tokens unknown** — no colour sampling, font ID, or spacing measurement was performed. | V1 0:47, V4 0:30, V5 7:04 (visual only) | Picker-readable frames or official screenshots; document unavoidable Linux scale/font differences. Blocks every entry. |
| 3 | **Linux input model unresolved** — all bindings shown are macOS `⌘`; GoRex's `Ctrl+Shift` remaps are clone-derived. | V4 0:30 (`⇧⌘P`), V1 2:22 (`⇧⌘T`, `⇧⌘R`), V4 0:30 palette glyphs | Official docs or a Linux client capture; otherwise an explicit Relayable binding decision. Blocks command palette, tabs, splits. |
| 4 | **In-app disconnected/loading/error states have no reference** — only quit/reopen was shown; a faithful "disconnected" screen cannot be claimed. | V4 3:59 → 4:11 | Any captured reconnect/failure moment, docs, or a lead-approved Relayable-original design labelled non-reference. |
| 5 | **Palette model ambiguity** — session picker vs command palette vs the compact V1 2:22 menu; command list includes truncated labels (`Go to Director...`, `…uments/ghostty`) and one-frame shortcut readings. | V1 1:40; V1 2:22; V4 0:30 | A continuous higher-res clip of each open/filter/execute flow and legible rows. |
| 6 | **Split/focus behaviour is only partially shown** — one horizontal split, no stacked split, no divider drag, no resize indicator, no focus keys. | V4 0:58; V4 4:49; menu text `Split Pane Up` V4 0:30 | A frame/clip of `Split Pane Up`, divider drag, and focus movement; official docs. |
| 7 | **Vertical tabs are nearly illegible** — only two palette rows and a small sidebar were captured; sidebar width/row states unknown. | V1 2:22 | Higher-resolution frame/clip of the vertical-tabs layout and switching. |
| 8 | **Remote workflow visuals beyond success are missing** — no connecting, no host-discovery list, no auth/pairing prompts, no failure UX; Tailscale dependence undetermined. | V4 0:58, 4:11, 4:49; V4 spoken declines protocol detail | Official docs and/or a demo showing the add-host form and a failure; otherwise scope the UI to what is shown. |
| 9 | **Terminal prompt/scrollback details unknown** — prompt badge meaning, theme palette, cursor, selection, and the V2-described scrollback loading state have no frames. | V1 0:47/2:03; V4 `~/D/ghostty 36`; V5 `31`; V2 5:00–5:35 (spoken) | Legible prompt/theme frames and a scrollback-loading capture; official theme docs. |
| 10 | **GoRex-vs-video differences could leak into implementation** — palette placeholder/selection colour, hover-only pane buttons, gradient window background, host popover, light mode, Linux accelerators, and window size are all clone-derived. | GoRex review §4.1–§10 vs V1/V4 frames | Keep the table in `ui-catalog.md` ("Discrepancies video vs. GoRex") authoritative; verify each item against video/docs before coding. |

## Acceptance-gate limits (must remain documented until closed)

1. **Colour, font and spacing values are approximations.** No frame was colour-sampled or measured
   (`video-review.md` §1.3); all estimates in `ui-catalog.md` are `[low-confidence estimate, 480p]`.
2. **Palette shortcut glyph readings are low confidence.** Rows such as `⇧⌘N`, `⇧⌘R`, `⌘K`,
   `◇⌘←/→/↑` were read once at 480p; `video-review.md` §8.3 asks for verification before
   implementation.
3. **Video-5 caption timing and frame content are loosely aligned.** On-screen clocks advance far
   slower than the caption timeline (e.g. captions 4:25/5:35 vs on-screen 15:03:37/15:03:48); cite
   frames for commands and captions for speech, never either as exact for the other.
4. **Video-3 benchmark figures are non-reproducible as published.** REX build/version, toolchain,
   measurement method, idle/active state, output rate and scrollback config are missing; use them as
   directional evidence only, run Relayable's own harness per `full.md` §12, and never use them as
   UI targets, claims, or marketing numbers.
5. **Video 2 is architecture-only and Video 3 is performance-only.** Neither can supply UI evidence;
   Video 2's scrollback-loading and pause/ready claims remain spoken-only.
6. **The exit gate is partially open.** `full.md` §2.7 items 1–2 are satisfied (all five videos
   attempted; V1 workspace and V4 remote flow visually inspected). Items 3–4 are satisfied by
   `ui-catalog.md` + `gorex-review.md` + this top-10 list, **but** the fidelity gate stays open for
   items with no reference frames (disconnected/error/loading, vertical-tab details, measured
   tokens).
