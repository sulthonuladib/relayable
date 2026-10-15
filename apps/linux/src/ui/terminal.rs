//! Terminal body renderer: paints a [`ScreenSnapshot`] onto a GPUI canvas.
//!
//! The grid is painted with GPUI's own shaping path (not per-glyph
//! `paint_glyph`): each row becomes one `ShapedLine` with per-style `TextRun`s,
//! which keeps a whole 140×38 screen to ~40 shapes per frame instead of ~5 000.
//! Verified API: `Window::text_system()` → `WindowTextSystem` (derefs to
//! `TextSystem`), `TextSystem::shape_line(SharedString, Pixels, &[TextRun],
//! Option<Pixels>)`, `ShapedLine::paint`/`paint_background`, `Window::paint_quad`
//! and `fill` (`gpui-0.2.2/src/text_system.rs`, `.../text_system/line.rs`,
//! `.../window.rs:2839`/`:5078`).
//!
//! # Faithfulness notes
//!
//! * The measured background ramps (`theme::TERMINAL_BG_*`, `FOCUSED_*`,
//!   `UNFOCUSED_*`) are painted as vertical bands so default-background cells
//!   stay transparent and show the ramp.
//! * Non-default ANSI/RGB cell colours use the **non-reference** standard xterm
//!   palette ([`super::palette`]).
//! * The block cursor and the scrollbar colours are **Relayable-original**
//!   (both unmeasured — `docs/research/design-tokens.md` §4).

use gpui::{
    canvas, div, fill, point, px, size, App, Bounds, Font, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Rgba, ScrollDelta, ShapedLine, SharedString, Styled, TextRun,
    UnderlineStyle, Window,
};

use crate::term::{CellStyle, ScreenSnapshot, TermColor};
use crate::theme;
use crate::ui::lerp_packed;

/// Everything needed to paint one pane's terminal body.
pub struct TerminalCanvas {
    /// The viewport (or scrollback) snapshot to render.
    pub snapshot: ScreenSnapshot,
    /// Resolved monospace font.
    pub font: Font,
    /// Terminal font size (logical px).
    pub font_size: Pixels,
    /// Measured monospace cell advance.
    pub cell_width: f32,
    /// Measured 17.25 px row pitch.
    pub row_pitch: f32,
    /// Top of the measured background ramp (`0xRRGGBB`).
    pub ramp_top: u32,
    /// Bottom of the measured background ramp (`0xRRGGBB`).
    pub ramp_bottom: u32,
    /// Terminal default foreground (`0xRRGGBB`).
    pub default_fg: u32,
    /// Terminal default background (`0xRRGGBB`); used only for inverse cells.
    pub default_bg: u32,
    /// Block-cursor colour (`0xRRGGBB`, Relayable-original).
    pub cursor_fill: u32,
    /// Block-cursor alpha (Relayable-original).
    pub cursor_alpha: f32,
    /// Scrollback offset in lines (`0` = live view).
    pub history_offset: isize,
    /// Retained scrollback length.
    pub history_len: usize,
    /// Scrollbar track colour (`0xRRGGBB`, Relayable-original).
    pub scrollbar_track: u32,
    /// Scrollbar thumb colour (`0xRRGGBB`, Relayable-original).
    pub scrollbar_thumb: u32,
    /// Scrollbar width in logical px (Relayable-original).
    pub scrollbar_width: f32,
}

/// Pre-shaped rows plus the small bits the paint phase needs.
struct TerminalPrepaint {
    lines: Vec<ShapedLine>,
    cursor: Option<(u16, u16)>,
    cursor_visible: bool,
    rows: u16,
}

impl TerminalCanvas {
    /// Build the scrollable terminal element.
    ///
    /// The caller supplies a unique `ElementId` (panes must not share ids) and a
    /// callback that receives scroll-wheel deltas.
    pub fn into_element(
        self,
        id: gpui::ElementId,
        on_scroll: impl Fn(ScrollDelta, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let TerminalCanvas {
            snapshot,
            font,
            font_size,
            cell_width,
            row_pitch,
            ramp_top,
            ramp_bottom,
            default_fg,
            default_bg,
            cursor_fill,
            cursor_alpha,
            history_offset,
            history_len,
            scrollbar_track,
            scrollbar_thumb,
            scrollbar_width,
        } = self;

        let prepaint_snapshot = snapshot.clone();

        div()
            .id(id)
            .flex_1()
            .w_full()
            .min_h(px(0.))
            .overflow_hidden()
            .on_scroll_wheel(move |event, window, cx| {
                on_scroll(event.delta, window, cx);
            })
            .child(
                canvas(
                    move |_bounds, window, _cx| {
                        let text_system = window.text_system();
                        let rows = prepaint_snapshot.rows;
                        let mut lines = Vec::with_capacity(usize::from(rows));
                        for row in 0..rows {
                            let (text, runs) =
                                shape_row(&prepaint_snapshot, row, &font, default_fg, default_bg);
                            lines.push(text_system.shape_line(text, font_size, &runs, None));
                        }
                        TerminalPrepaint {
                            lines,
                            cursor: prepaint_snapshot.cursor,
                            cursor_visible: prepaint_snapshot.cursor_visible,
                            rows,
                        }
                    },
                    move |bounds, state, window, cx| {
                        paint_ramp(window, bounds, ramp_top, ramp_bottom);
                        for (row, line) in state.lines.iter().enumerate() {
                            let origin = point(
                                bounds.origin.x,
                                bounds.origin.y + px(row_pitch * row as f32),
                            );
                            let line_height = px(row_pitch);
                            let _ = line.paint_background(origin, line_height, window, cx);
                            let _ = line.paint(origin, line_height, window, cx);
                        }
                        if state.cursor_visible {
                            if let Some((row, col)) = state.cursor {
                                paint_cursor(
                                    window,
                                    bounds,
                                    row,
                                    col,
                                    cell_width,
                                    row_pitch,
                                    cursor_fill,
                                    cursor_alpha,
                                );
                            }
                        }
                        paint_scrollbar(
                            window,
                            bounds,
                            state.rows,
                            history_len,
                            history_offset,
                            scrollbar_width,
                            scrollbar_track,
                            scrollbar_thumb,
                        );
                    },
                )
                .size_full(),
            )
    }
}

/// Paint the measured vertical background ramp as horizontal bands.
fn paint_ramp(window: &mut Window, bounds: Bounds<Pixels>, top: u32, bottom: u32) {
    // The ramp endpoints are measured; the band count is a Relayable-original
    // renderer choice (`theme::TERMINAL_RAMP_BANDS_ORIGINAL`), smooth and cheap.
    let bands = theme::TERMINAL_RAMP_BANDS_ORIGINAL;
    let height = f32::from(bounds.size.height);
    if height <= 0.0 {
        return;
    }
    let step = height / bands as f32;
    for band in 0..bands {
        let y0 = f32::from(bounds.origin.y) + step * band as f32;
        let y1 = if band + 1 == bands {
            f32::from(bounds.origin.y) + height
        } else {
            f32::from(bounds.origin.y) + step * (band + 1) as f32
        };
        let t = band as f32 / (bands - 1) as f32;
        let color = lerp_packed(top, bottom, t);
        let rect = Bounds::new(
            point(px(f32::from(bounds.origin.x)), px(y0)),
            size(px(f32::from(bounds.size.width)), px((y1 - y0).max(0.0))),
        );
        window.paint_quad(fill(rect, color));
    }
}

/// Paint the observed block cursor (Relayable-original colour/alpha).
#[allow(clippy::too_many_arguments)]
fn paint_cursor(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    row: u16,
    col: u16,
    cell_width: f32,
    row_pitch: f32,
    cursor_fill: u32,
    cursor_alpha: f32,
) {
    let origin = point(
        px(f32::from(bounds.origin.x) + cell_width * f32::from(col)),
        px(f32::from(bounds.origin.y) + row_pitch * f32::from(row)),
    );
    let rect = Bounds::new(origin, size(px(cell_width), px(row_pitch)));
    let mut color: Rgba = gpui::rgb(cursor_fill);
    color.a = cursor_alpha.clamp(0.0, 1.0);
    window.paint_quad(fill(rect, color));
}

/// Paint a thin scrollback scrollbar when there is history (Relayable-original).
#[allow(clippy::too_many_arguments)]
fn paint_scrollbar(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    rows: u16,
    history_len: usize,
    history_offset: isize,
    width: f32,
    track: u32,
    thumb: u32,
) {
    if history_len == 0 {
        return;
    }
    let total = history_len as f32 + f32::from(rows);
    if total <= 0.0 {
        return;
    }
    let track_height = f32::from(bounds.size.height);
    let x = f32::from(bounds.origin.x) + f32::from(bounds.size.width) - width;
    let track_rect = Bounds::new(
        point(px(x), px(f32::from(bounds.origin.y))),
        size(px(width), px(track_height)),
    );
    let mut track_color: Rgba = gpui::rgb(track);
    track_color.a = theme::SCROLLBAR_TRACK_ALPHA_ORIGINAL;
    window.paint_quad(fill(track_rect, track_color));

    let visible = f32::from(rows);
    let thumb_height =
        (visible / total * track_height).max(theme::SCROLLBAR_THUMB_MIN_LENGTH_ORIGINAL);
    let offset = history_offset.max(0) as f32;
    let view_top = (history_len as f32 - offset).clamp(0.0, history_len as f32);
    let thumb_y = f32::from(bounds.origin.y) + (view_top / total) * track_height;
    let thumb_rect = Bounds::new(
        point(px(x), px(thumb_y)),
        size(px(width), px(thumb_height.min(track_height))),
    );
    let mut thumb_color: Rgba = gpui::rgb(thumb);
    thumb_color.a = theme::SCROLLBAR_THUMB_ALPHA_ORIGINAL;
    window.paint_quad(fill(thumb_rect, thumb_color));
}

/// Shape one row into a `SharedString` plus per-style runs.
fn shape_row(
    snapshot: &ScreenSnapshot,
    row: u16,
    font: &Font,
    default_fg: u32,
    default_bg: u32,
) -> (SharedString, Vec<TextRun>) {
    let cols = usize::from(snapshot.cols);
    let start = usize::from(row) * cols;
    let cells = &snapshot.cells[start..start + cols];

    let mut text = String::with_capacity(cols);
    let mut runs: Vec<TextRun> = Vec::new();

    for cell in cells {
        if cell.wide_continuation {
            continue;
        }
        let ch = if cell.ch == '\0' || cell.ch.is_control() {
            ' '
        } else {
            cell.ch
        };
        let (color, background, run_font) = resolve_style(cell.style, font, default_fg, default_bg);
        let underline = cell.style.underline.then_some(UnderlineStyle {
            thickness: px(theme::UNDERLINE_THICKNESS_ORIGINAL),
            color: Some(color),
            wavy: false,
        });

        let before = text.len();
        text.push(ch);
        let len = text.len() - before;

        if let Some(last) = runs.last_mut() {
            if last.color == color
                && last.background_color == background
                && last.font == run_font
                && last.underline == underline
            {
                last.len += len;
                continue;
            }
        }
        runs.push(TextRun {
            len,
            font: run_font,
            color,
            background_color: background,
            underline,
            strikethrough: None,
        });
    }

    (SharedString::new(text), runs)
}

/// Resolve a cell's colours/font against the terminal defaults and the palette.
fn resolve_style(
    style: CellStyle,
    font: &Font,
    default_fg: u32,
    default_bg: u32,
) -> (Hsla, Option<Hsla>, Font) {
    let fg_rgb = resolve_rgb(style.fg, default_fg);
    let mut color: Hsla = gpui::rgb(fg_rgb).into();
    if style.dim {
        color = color.opacity(theme::DIM_OPACITY_ORIGINAL);
    }

    let mut background = style.bg.map(|bg| -> Hsla {
        let rgb = resolve_rgb(bg, default_bg);
        gpui::rgb(rgb).into()
    });

    if style.inverse {
        let implied_bg: Hsla = gpui::rgb(default_bg).into();
        let new_fg = background.unwrap_or(implied_bg);
        let new_bg: Hsla = gpui::rgb(fg_rgb).into();
        color = new_fg;
        background = Some(new_bg);
    }

    let run_font = if style.bold {
        font.clone().bold()
    } else if style.italic {
        font.clone().italic()
    } else {
        font.clone()
    };

    (color, background, run_font)
}

/// A `TermColor` to `0xRRGGBB`, falling back to `default` for terminal defaults.
fn resolve_rgb(color: TermColor, default: u32) -> u32 {
    match color {
        TermColor::Default => default,
        TermColor::Indexed(index) => u32::from(super::palette::ansi(index)) >> 8,
        TermColor::Rgb(r, g, b) => ((u32::from(r)) << 16) | ((u32::from(g)) << 8) | u32::from(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::{ScreenCell, TerminalModel, TerminalSize};

    fn one_row_snapshot() -> ScreenSnapshot {
        let mut term = TerminalModel::new(TerminalSize::cells(8, 1));
        term.process(b"\x1b[31mre\x1b[0md");
        term.snapshot()
    }

    fn a_font() -> Font {
        gpui::font("monospace")
    }

    #[test]
    fn runs_cover_the_whole_row_and_split_on_style_change() {
        let snapshot = one_row_snapshot();
        let font = a_font();
        let (text, runs) = shape_row(&snapshot, 0, &font, 0xBFDBE4, 0x0A1D2B);

        assert_eq!(runs.iter().map(|run| run.len).sum::<usize>(), text.len());
        // "re" is one red run, "d" + trailing blanks are another.
        assert_eq!(runs.len(), 2, "a style change must split the run");
        assert_eq!(runs[0].len, 2);
        assert!(runs[0].background_color.is_none(), "default bg stays None");
    }

    #[test]
    fn wide_continuation_cells_are_skipped() {
        let snapshot = ScreenSnapshot {
            cols: 4,
            rows: 1,
            cells: vec![
                ScreenCell {
                    ch: '世',
                    style: CellStyle {
                        fg: TermColor::Default,
                        bg: None,
                        bold: false,
                        italic: false,
                        underline: false,
                        inverse: false,
                        dim: false,
                    },
                    wide_continuation: false,
                },
                ScreenCell {
                    ch: ' ',
                    style: CellStyle {
                        fg: TermColor::Default,
                        bg: None,
                        bold: false,
                        italic: false,
                        underline: false,
                        inverse: false,
                        dim: false,
                    },
                    wide_continuation: true,
                },
                ScreenCell {
                    ch: 'x',
                    style: CellStyle {
                        fg: TermColor::Default,
                        bg: None,
                        bold: false,
                        italic: false,
                        underline: false,
                        inverse: false,
                        dim: false,
                    },
                    wide_continuation: false,
                },
                ScreenCell {
                    ch: ' ',
                    style: CellStyle {
                        fg: TermColor::Default,
                        bg: None,
                        bold: false,
                        italic: false,
                        underline: false,
                        inverse: false,
                        dim: false,
                    },
                    wide_continuation: false,
                },
            ],
            cursor: None,
            cursor_visible: false,
        };
        let font = a_font();
        let (text, runs) = shape_row(&snapshot, 0, &font, 0xFFFFFF, 0x000000);
        assert_eq!(text.as_ref(), "世x ");
        assert_eq!(runs.iter().map(|r| r.len).sum::<usize>(), text.len());
    }

    #[test]
    fn default_background_never_paints_a_run() {
        let snapshot = one_row_snapshot();
        let font = a_font();
        let (_, runs) = shape_row(&snapshot, 0, &font, 0xFFFFFF, 0x000000);
        assert!(runs.iter().all(|run| run.background_color.is_none()));
    }
}
