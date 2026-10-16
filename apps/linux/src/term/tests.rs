use super::*;

fn model(cols: u16, rows: u16) -> TerminalModel {
    TerminalModel::new(TerminalSize::cells(cols, rows))
}

fn row_text(snapshot: &ScreenSnapshot, row: u16) -> String {
    let cols = usize::from(snapshot.cols);
    let start = usize::from(row) * cols;
    snapshot.cells[start..start + cols]
        .iter()
        .map(|cell| cell.ch)
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[test]
fn sgr_red_sets_indexed_foreground_and_reset_clears_it() {
    let mut term = model(20, 2);
    term.process(b"\x1b[31mred\x1b[0m");

    let snapshot = term.snapshot();
    assert_eq!(row_text(&snapshot, 0), "red");

    let r = snapshot.cells[0].style;
    assert_eq!(r.fg, TermColor::Indexed(1));
    assert_eq!(r.bg, None);

    let after_reset = snapshot.cells[3].style;
    assert_eq!(after_reset.fg, TermColor::Default);
    assert_eq!(after_reset.bg, None);
}

#[test]
fn default_background_cells_report_bg_none() {
    let mut term = model(8, 2);
    term.process(b"hello");

    let snapshot = term.snapshot();
    for cell in &snapshot.cells {
        assert_eq!(cell.style.bg, None, "no invented default background");
    }
}

#[test]
fn explicit_background_is_reported() {
    let mut term = model(8, 2);
    term.process(b"\x1b[41mX\x1b[0m");

    let snapshot = term.snapshot();
    assert_eq!(snapshot.cells[0].style.bg, Some(TermColor::Indexed(1)));
}

#[test]
fn resize_changes_dimensions() {
    let mut term = model(80, 24);
    assert_eq!(term.snapshot().cols, 80);
    assert_eq!(term.snapshot().rows, 24);

    term.resize(TerminalSize::cells(120, 40));
    let snapshot = term.snapshot();
    assert_eq!(snapshot.cols, 120);
    assert_eq!(snapshot.rows, 40);
    assert_eq!(snapshot.cells.len(), 120 * 40);
}

#[test]
fn process_handles_partial_escape_and_invalid_utf8_without_panicking() {
    let mut term = model(40, 4);
    // A truncated CSI ...
    term.process(b"\x1b[38;5;");
    // ... completed later.
    term.process(b"196mZ");
    // Invalid UTF-8 continuation bytes.
    term.process(&[0xFF, 0xFE, 0x80, b'?']);
    // Truncated OSC that never terminates.
    term.process(b"\x1b]0;partial-title");
    // Must still snapshot.
    let snapshot = term.snapshot();
    assert_eq!(snapshot.cells.len(), 40 * 4);
}

#[test]
fn truecolor_foreground_is_rgb() {
    let mut term = model(8, 2);
    term.process(b"\x1b[38;2;10;20;30mQ");
    let snapshot = term.snapshot();
    assert_eq!(snapshot.cells[0].style.fg, TermColor::Rgb(10, 20, 30));
}

#[test]
fn scrollback_grows_and_snapshot_at_reads_history() {
    let mut term = model(10, 2);
    term.process(b"one\r\ntwo\r\nthree\r\nfour");
    assert!(term.history_len() >= 2, "history should have grown");

    let live = term.snapshot();
    assert_eq!(row_text(&live, 1), "four");
    let older = term.snapshot_at(2);
    assert_eq!(older.cells.len(), 10 * 2);
}

#[test]
fn snapshot_at_clamps_beyond_history() {
    let mut term = model(10, 2);
    term.process(b"only");
    // A large offset must clamp, not panic.
    let snapshot = term.snapshot_at(100_000);
    assert_eq!(snapshot.cells.len(), 10 * 2);
}
