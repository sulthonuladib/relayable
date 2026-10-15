//! Engine adapter registry.
//!
//! Phase B implements `alacritty_terminal 0.26.0` end-to-end. Phase C wires the
//! `vt100 0.16.2` and `avt 0.18.0` comparison engines behind the same frozen
//! [`crate::model::EngineAdapter`] trait and capability-gates each check. The
//! `termwiz 0.23.3` adapter remains a `NOT_IMPLEMENTED` stub with a documented
//! exclusion (it has no byte-stream -> screen-state emulator).

pub mod alacritty;
pub mod avt;
pub mod termwiz;
pub mod vt100;

use crate::model::{Cell, Cursor, EngineFactory, MouseMode, ScrollbackStats, Snapshot};

/// The engines exercised by the phase-B/C checks, in report order.
/// `alacritty_terminal` is the gating engine; `vt100` and `avt` are wired
/// comparison engines; `termwiz` is the documented `NOT_IMPLEMENTED`
/// exclusion. Each adapter declares its own [`crate::model::Capabilities`] so a
/// missing feature is reported `UNSUPPORTED` (a capability gap), not `FAIL`.
#[must_use]
pub fn engines() -> Vec<(&'static str, EngineFactory)> {
    vec![
        (alacritty::NAME, alacritty::factory as EngineFactory),
        (vt100::NAME, vt100::factory as EngineFactory),
        (avt::NAME, avt::factory as EngineFactory),
        (termwiz::NAME, termwiz::factory as EngineFactory),
    ]
}

/// Resolved crate version for a registered engine name.
#[must_use]
pub fn engine_version(name: &str) -> &'static str {
    match name {
        alacritty::NAME => alacritty::VERSION,
        vt100::NAME => vt100::VERSION,
        avt::NAME => avt::VERSION,
        termwiz::NAME => termwiz::VERSION,
        _ => "unknown",
    }
}

/// Fully-blank snapshot used by the not-yet-implemented engine stubs.
#[must_use]
pub fn empty_snapshot(cols: u16, rows: u16, scrollback: usize) -> Snapshot {
    Snapshot {
        cols,
        rows,
        cells: vec![Cell::blank(); cols as usize * rows as usize],
        cursor: Cursor {
            row: 0,
            col: 0,
            visible: false,
        },
        alt_screen: false,
        bracketed_paste: false,
        app_cursor_keys: false,
        mouse_mode: MouseMode::Off,
        mouse_sgr: false,
        title: None,
        events: Vec::new(),
        bell_count: 0,
        scrollback: ScrollbackStats {
            retained_lines: 0,
            configured_bound: scrollback,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ImplementationStatus;

    #[test]
    fn registry_lists_four_engines_in_order() {
        let names: Vec<&str> = engines().iter().map(|(n, _)| *n).collect();
        assert_eq!(names, vec!["alacritty_terminal", "vt100", "avt", "termwiz"]);
        assert_eq!(engine_version("alacritty_terminal"), "0.26.0");
        assert_eq!(engine_version("vt100"), "0.16.2");
        assert_eq!(engine_version("avt"), "0.18.0");
        assert_eq!(engine_version("termwiz"), "0.23.3");
    }

    #[test]
    fn alacritty_and_comparison_engines_are_implemented() {
        for (name, factory) in engines() {
            if name == termwiz::NAME {
                continue;
            }
            let adapter = factory(80, 24, 1000);
            assert!(
                adapter.implementation_status().is_implemented(),
                "{name} should be implemented"
            );
        }
    }

    #[test]
    fn termwiz_stub_snapshots_are_blank() {
        let adapter = termwiz::factory(80, 24, 1000);
        let status = adapter.implementation_status();
        assert!(!status.is_implemented());
        assert!(status.reason().is_some_and(|r| !r.is_empty()));
        let s = adapter.snapshot();
        assert_eq!(s.cells.len(), 80 * 24);
        assert!(s
            .cells
            .iter()
            .all(|c| c.text == " " && c.hyperlink.is_none()));
        assert!(!s.cursor.visible && !s.alt_screen && s.title.is_none());
        assert_eq!(s.scrollback.retained_lines, 0);
    }

    #[test]
    fn stub_marker_constant_is_stable() {
        assert_eq!(termwiz::NOT_IMPLEMENTED, "NOT_IMPLEMENTED");
        assert!(termwiz::NOT_IMPLEMENTED_REASON.contains("src/escape/parser/mod.rs:86"));
        assert!(termwiz::NOT_IMPLEMENTED_REASON.contains("src/surface/mod.rs:105"));
        assert!(termwiz::NOT_IMPLEMENTED_REASON.contains("src/terminal/unix.rs:266"));
        assert!(termwiz::NOT_IMPLEMENTED_REASON.contains("src/terminal/unix.rs:171"));
        assert_eq!(
            ImplementationStatus::NotImplemented {
                engine: "x",
                reason: "y"
            }
            .reason(),
            Some("y")
        );
    }
}
