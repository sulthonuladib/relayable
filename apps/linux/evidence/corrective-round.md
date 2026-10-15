# UI-001 corrective round — review findings fixed

Scope: `apps/linux/**` only. No dependency changed; `apps/linux/Cargo.lock` still
holds `libc` 0.2.189 (TERM-002 remedy artifact).

## Fix 1 (MEDIUM) — two measured colours added

`docs/research/design-tokens.md` measures the pane-card border colours that the
phase-2 sheet had dropped (only their 2–3 px thickness was encoded):

| Document line | Row | Value | Source stamp | Confidence | Caveat (verbatim) |
|---|---|---|---|---|---|
| `:127` | Pane card border / focused | rgb ≈ (31,82,115) | `V4 0:58 `v4-t0058.png`` | medium-high | `includes antialias` |
| `:128` | Pane card border / unfocused | rgb ≈ (17,54,71) | `V4 0:58 `v4-t0058.png`` | medium | `one frame` |

Both are restated as measured at `design-tokens.md:201`. Added to the V4-scoped
colour set in `src/tokens.rs`:

- `colors::PANE_CARD_BORDER_FOCUSED: Measured<Rgb8>` = `Rgb8::new(31, 82, 115)`,
  `Confidence::MediumHigh`, caveat `"includes antialias"`.
- `colors::PANE_CARD_BORDER_UNFOCUSED: Measured<Rgb8>` = `Rgb8::new(17, 54, 71)`,
  `Confidence::Medium`, caveat `"one frame"`.

The existing thickness tokens (`geometry::v4::PANE_CARD_BORDER_FOCUSED_PX`,
`PANE_CARD_BORDER_UNFOCUSED_PX`) are unchanged. Test coverage added to
`documented_colour_equals_exact_triple` (exact triples) and both tokens were added
to the `all_provenance()` list checked by
`every_measured_token_has_non_empty_source_and_caveat`.

## Sweep for other dropped rows

Every row of `design-tokens.md` §3.1 (geometry) and §3.2 (colours) was compared
against `src/tokens.rs`:

- **No others.** Every other §3.1/§3.2 row maps to an existing token (geometry,
  colour, ramp, or `not_app_tokens`), and the only explicitly `not measurable`
  row (`Pane header height (V1)`) is already in `unmeasured::PANE_HEADER_HEIGHT_V1`.
  All §4 `not measurable` items are represented in `unmeasured`.

## Fix 2 (LOW) — unverifiable lint suppression removed

`Rgb8::to_u32_0xRRGGBB` had `#[allow(non_snake_case)]` justified as "fixed by the
UI-001 spec"; no repository content mandates that name, so the suppression was
removed and the method renamed:

- `to_u32_0xRRGGBB` → **`to_packed_rgb_u32`** (`src/tokens.rs`), call sites updated
  in `src/main.rs`, doc links and tests updated.
- `grep -rn '#\[allow' apps/linux/src` → **no matches**: no `#[allow(...)]`
  remains anywhere in `apps/linux/**`.

## No screenshot re-capture

The rendered colours are unchanged:
`colors::PANE_HEADER_SURFACE_V1` is still rgb(9,30,46) and
`colors::PRIMARY_TEXT` is still rgb(191,219,228); `git diff src/main.rs` shows
only the method rename plus rustfmt line wrapping. The committed phase-2
screenshots therefore still reflect the committed code and were not re-captured.
