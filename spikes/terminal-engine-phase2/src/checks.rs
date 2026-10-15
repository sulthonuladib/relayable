//! Check definitions: explicit EXPECTED values, observed ACTUAL values, and a
//! PASS / FAIL / UNSUPPORTED / NOT_TESTED status per engine.
//!
//! All fixture checks run through the neutral [`EngineAdapter`] trait. The
//! scrollback deep-history check additionally uses the alacritty-specific
//! [`AlacrittyAdapter`] history dump because the neutral model intentionally
//! summarises scrollback rather than dumping it; that section is labelled
//! `engine-specific` in the notes.

use std::fmt::Debug;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::engine::alacritty::AlacrittyAdapter;
use crate::engine::vt100::Vt100Adapter;
use crate::fixtures::Fixture;
use crate::model::{
    Capabilities, Cursor, EngineAdapter, EngineEvent, EngineFactory, ImplementationStatus,
    MouseMode, Snapshot,
};
use crate::sha256::sha256_hex;

/// Every check the phase-B suite defines: `(id, title, kind)` in report order.
/// Used to emit one `UNSUPPORTED` placeholder per check for engines that are
/// not yet wired, so the report can show "not yet wired" vs "failed".
pub const CHECK_CATALOG: &[(&str, &str, &str)] = &[
    (
        "vt_basics",
        "VT/ANSI basics (CUP/SGR/EL/ED/DECSC)",
        "fixture",
    ),
    (
        "alt_screen",
        "Alternate screen DECSET/DECRST 1049",
        "fixture",
    ),
    ("osc_title", "OSC 0/2 window title", "fixture"),
    ("osc8_hyperlink", "OSC 8 per-cell hyperlinks", "fixture"),
    ("bell", "BEL bell count and EngineEvent::Bell", "fixture"),
    (
        "unicode_wide",
        "CJK wide glyphs occupy 2 columns",
        "fixture",
    ),
    (
        "unicode_combining",
        "Combining mark U+0301 attaches to base cell",
        "fixture",
    ),
    (
        "unicode_emoji",
        "Emoji base / VS16 / ZWJ per-cell decomposition",
        "fixture",
    ),
    (
        "colors",
        "Named / 256-index / truecolor SGR parsing",
        "fixture",
    ),
    (
        "cursor_keys",
        "DECSET/DECRST 1 application cursor keys",
        "fixture",
    ),
    (
        "bracketed_paste_mode",
        "Bracketed paste mode (DECSET/DECRST 2004) flag",
        "fixture",
    ),
    (
        "bracketed_paste_payload",
        "Bracketed paste payload bytes-as-data (input-path property)",
        "fixture",
    ),
    (
        "mouse_modes",
        "Mouse reporting modes 1000/1002/1003/1006",
        "fixture",
    ),
    (
        "resize_reflow",
        "Resize + logical-line reflow (engine leg)",
        "engine",
    ),
    (
        "scrollback_bound",
        "Scrollback bound at scrolling_history=1000",
        "engine",
    ),
    (
        "selection",
        "Text range selection -> string (engine-specific)",
        "engine",
    ),
    (
        "chunk_boundary_invariance",
        "Feed chunk-boundary invariance (one slice vs one byte at a time)",
        "engine",
    ),
    (
        "alt_screen_real",
        "Alternate screen (real nvim)",
        "real-program",
    ),
    (
        "osc_title_real",
        "OSC 0/2 title (real nvim '+set title')",
        "real-program",
    ),
    (
        "osc8_real",
        "OSC 8 hyperlinks (real ls --hyperlink)",
        "real-program",
    ),
    ("bell_real", "BEL (real sh printf \\a)", "real-program"),
    (
        "unicode_real",
        "Unicode text (real sh printf)",
        "real-program",
    ),
    (
        "cursor_keys_real",
        "Application cursor keys round-trip (real sh + injected arrow)",
        "real-program",
    ),
    (
        "mouse_modes_real",
        "Mouse modes (real nvim '+set mouse=a')",
        "real-program",
    ),
    (
        "resize_real",
        "PTY resize SIGWINCH round-trip (real sh + stty size)",
        "real-program",
    ),
    (
        "scrollback_bound_real",
        "Scrollback bound (real seq 1 20000)",
        "real-program",
    ),
    (
        "tmux_real",
        "Alternate screen + content (real tmux session)",
        "real-program",
    ),
    (
        "replay_less",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
    (
        "replay_nvim",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
    (
        "replay_nvim_mouse",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
    (
        "replay_nvim_title",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
    (
        "replay_osc8_ls",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
    (
        "replay_top",
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    ),
];

/// Result status for one assertion or one whole check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    NotTested,
    Unsupported,
    Fail,
}

impl Status {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::NotTested => "NOT_TESTED",
            Status::Unsupported => "UNSUPPORTED",
            Status::Fail => "FAIL",
        }
    }

    fn severity(self) -> u8 {
        match self {
            Status::Pass => 0,
            Status::NotTested => 1,
            Status::Unsupported => 2,
            Status::Fail => 3,
        }
    }
}

/// One `EXPECTED` vs `ACTUAL` assertion.
#[derive(Debug, Clone)]
pub struct Assertion {
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub status: Status,
}

/// A SHA-256-stamped input source.
#[derive(Debug, Clone)]
pub struct SourceRef {
    pub path: String,
    pub sha256: String,
    pub kind: String,
}

/// A real-program capture loaded from `results/raw/streams/`.
#[derive(Debug, Clone)]
pub struct Stream {
    pub name: String,
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub resizes: Vec<(u64, u16, u16)>,
}

impl Stream {
    /// The stream's recorded resizes as `(ms, cols, rows)`.
    #[must_use]
    pub fn last_resize(&self) -> Option<(u64, u16, u16)> {
        self.resizes.last().copied()
    }
}

/// Environment for one engine's check run.
pub struct Env<'a> {
    pub engine: &'static str,
    pub version: &'static str,
    pub factory: EngineFactory,
    pub fixtures: &'a [Fixture],
    pub streams: &'a [Stream],
    pub cols: u16,
    pub rows: u16,
    pub scrollback: usize,
    /// Repository root, used to locate the read-only phase-1 captures.
    pub repo_root: PathBuf,
}

impl Env<'_> {
    fn fixture(&self, name: &str) -> Option<&Fixture> {
        self.fixtures.iter().find(|f| f.name == name)
    }

    fn stream(&self, name: &str) -> Option<&Stream> {
        self.streams.iter().find(|s| s.name == name)
    }

    fn make(&self, cols: u16, rows: u16) -> Box<dyn EngineAdapter> {
        (self.factory)(cols, rows, self.scrollback)
    }

    fn make_default(&self) -> Box<dyn EngineAdapter> {
        self.make(self.cols, self.rows)
    }

    /// Features declared by this engine's adapter. Derived from a probe
    /// adapter (via the factory) so `Env` keeps its existing constructor shape.
    fn capabilities(&self) -> Capabilities {
        self.make_default().capabilities()
    }
}

/// A `UNSUPPORTED` check result for an engine that does not declare a feature.
///
/// This is a *capability gap*, not a defect: the check is not run and the
/// missing feature is named in the artifact.
fn cap_unsupported(env: &Env, check: &str, title: &str, kind: &str, missing: &str) -> CheckResult {
    let mut r = CheckResult::new(env, check, title, kind);
    r.unsupported(
        "capability_present",
        &format!("engine declares capability '{missing}'"),
        &format!(
            "{} {} does not provide '{missing}' (declared absent in its Capabilities)",
            env.engine, env.version
        ),
    );
    r.note(format!(
        "Capability gap: '{missing}'. Recorded UNSUPPORTED, never FAIL. See results/raw/compare/README.txt for the capability model."
    ));
    r
}

/// Aggregated state observed while feeding a byte stream incrementally.
///
/// Honesty note on `alt_in_chunk_snapshots`: it is a *chunk-sampled*
/// observation, true only when some per-chunk snapshot happened to catch the
/// alternate screen. A capture shorter than one chunk is snapshotted once, so
/// a transient 1049 enter/leave pair can be missed entirely (this is exactly
/// what happened to the 234-byte phase-1 `less` capture before this fix). Do
/// **not** read it as an engine property; use [`scan_alt_1049`] /
/// [`replay_with_alt_watch`] for chunk-independent transition evidence.
#[derive(Debug, Clone)]
pub struct Observed {
    /// Chunk-sampled: see the struct-level note. Renamed from `alt_ever` so no
    /// reader mistakes it for a chunk-independent engine property.
    pub alt_in_chunk_snapshots: bool,
    /// Chunk-sampled final alternate-screen flag.
    pub alt_final_snapshot: bool,
    pub titles_seen: Vec<String>,
    pub title_final: Option<String>,
    pub reset_title_ever: bool,
    pub bell_total: u64,
    pub mouse_mode_max: MouseMode,
    pub mouse_sgr_ever: bool,
    pub bracketed_ever: bool,
    pub cursor_keys_ever: bool,
    pub final_snapshot: Snapshot,
}

fn mouse_rank(mode: MouseMode) -> u8 {
    match mode {
        MouseMode::Off => 0,
        MouseMode::Click => 1,
        MouseMode::Drag => 2,
        MouseMode::Any => 3,
    }
}

/// Feed `bytes` in `chunk`-sized slices, watching mode transitions.
#[must_use]
pub fn watch(adapter: &mut dyn EngineAdapter, bytes: &[u8], chunk: usize) -> Observed {
    let mut obs = Observed {
        alt_in_chunk_snapshots: false,
        alt_final_snapshot: false,
        titles_seen: Vec::new(),
        title_final: None,
        reset_title_ever: false,
        bell_total: 0,
        mouse_mode_max: MouseMode::Off,
        mouse_sgr_ever: false,
        bracketed_ever: false,
        cursor_keys_ever: false,
        final_snapshot: adapter.snapshot(),
    };
    let apply = |obs: &mut Observed, s: Snapshot| {
        obs.alt_in_chunk_snapshots |= s.alt_screen;
        obs.alt_final_snapshot = s.alt_screen;
        if let Some(t) = &s.title {
            if !obs.titles_seen.contains(t) {
                obs.titles_seen.push(t.clone());
            }
            obs.title_final = Some(t.clone());
        }
        if s.events
            .iter()
            .any(|e| matches!(e, EngineEvent::ResetTitle))
        {
            obs.reset_title_ever = true;
        }
        obs.bell_total = s.bell_count;
        if mouse_rank(s.mouse_mode) > mouse_rank(obs.mouse_mode_max) {
            obs.mouse_mode_max = s.mouse_mode;
        }
        obs.mouse_sgr_ever |= s.mouse_sgr;
        obs.bracketed_ever |= s.bracketed_paste;
        obs.cursor_keys_ever |= s.app_cursor_keys;
        obs.final_snapshot = s;
    };
    if bytes.is_empty() {
        let s = adapter.snapshot();
        apply(&mut obs, s);
        return obs;
    }
    for chunk in bytes.chunks(chunk) {
        adapter.feed(chunk);
        let s = adapter.snapshot();
        apply(&mut obs, s);
    }
    obs
}

/// One raw-byte DECSET/DECRST 1049 occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AltSequence {
    /// Offset of the first byte of the escape sequence.
    pub start: usize,
    /// Offset one past the final byte.
    pub end: usize,
    /// `true` for `ESC [ ? 1049 h` (enter), `false` for `ESC [ ? 1049 l`.
    pub enter: bool,
}

/// Scan raw capture bytes for the DECSET/DECRST 1049 sequences (what phase 1
/// did in `CATEGORY_CHECKLIST`). This is chunk-independent: it inspects the
/// bytes, not engine snapshots, so a transient alt-screen transition in a
/// sub-chunk capture is still detected.
#[must_use]
pub fn scan_alt_1049(bytes: &[u8]) -> Vec<AltSequence> {
    const ENTER: &[u8] = b"\x1b[?1049h";
    const LEAVE: &[u8] = b"\x1b[?1049l";
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(ENTER) {
            out.push(AltSequence {
                start: i,
                end: i + ENTER.len(),
                enter: true,
            });
            i += ENTER.len();
        } else if bytes[i..].starts_with(LEAVE) {
            out.push(AltSequence {
                start: i,
                end: i + LEAVE.len(),
                enter: false,
            });
            i += LEAVE.len();
        } else {
            i += 1;
        }
    }
    out
}

/// Engine-observed state immediately after one raw 1049 transition.
#[derive(Debug, Clone)]
pub struct AltTransition {
    /// Offset one past the transition sequence.
    pub offset: usize,
    pub enter: bool,
    /// The engine's `alt_screen` flag after feeding up to `offset`.
    pub engine_alt_screen: bool,
}

/// Chunk-independent replay result.
#[derive(Debug, Clone)]
pub struct AltReplay {
    pub sequences: Vec<AltSequence>,
    pub transitions: Vec<AltTransition>,
    /// Snapshot of the alt screen taken immediately before the last 1049 leave
    /// (or the final snapshot when the capture ends inside the alt screen).
    /// This is where a full-screen program's content actually lives.
    pub last_alt_snapshot: Option<Snapshot>,
    pub final_snapshot: Snapshot,
}

/// Replay `bytes` through `adapter`, splitting the feed at every raw 1049
/// transition so the engine is snapshotted (a) immediately after each
/// transition and (b) immediately before each leave. Unlike [`watch`], the
/// transition evidence does not depend on the chunk size.
#[must_use]
pub fn replay_with_alt_watch(adapter: &mut dyn EngineAdapter, bytes: &[u8]) -> AltReplay {
    let sequences = scan_alt_1049(bytes);
    let mut transitions = Vec::new();
    let mut last_alt_snapshot = None;
    let mut prev = 0usize;
    for seq in &sequences {
        adapter.feed(&bytes[prev..seq.start]);
        if !seq.enter {
            let s = adapter.snapshot();
            if s.alt_screen {
                last_alt_snapshot = Some(s);
            }
        }
        adapter.feed(&bytes[seq.start..seq.end]);
        let s = adapter.snapshot();
        transitions.push(AltTransition {
            offset: seq.end,
            enter: seq.enter,
            engine_alt_screen: s.alt_screen,
        });
        prev = seq.end;
    }
    adapter.feed(&bytes[prev..]);
    let final_snapshot = adapter.snapshot();
    if final_snapshot.alt_screen {
        last_alt_snapshot = Some(final_snapshot.clone());
    }
    AltReplay {
        sequences,
        transitions,
        last_alt_snapshot,
        final_snapshot,
    }
}

// ---------------------------------------------------------------------------
// Engine-input payloads (persisted so every cited sha256 is recomputable)
// ---------------------------------------------------------------------------

/// One deterministic engine-input payload fed directly by a check.
pub struct EngineInput {
    pub name: &'static str,
    pub description: &'static str,
    pub bytes: Vec<u8>,
}

fn numbered_lines(count: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0..count {
        out.extend_from_slice(format!("line {i:04}\r\n").as_bytes());
    }
    out
}

/// Every engine-input payload the checks feed directly (as opposed to
/// `fixtures::all()` or a real-program capture). Persisted by
/// [`write_engine_inputs`] and SHA-256 stamped in the per-check artifacts, so
/// the recorded hash is recomputable from a committed file.
#[must_use]
pub fn engine_inputs() -> Vec<EngineInput> {
    vec![
        EngineInput {
            name: "engine_resize_input",
            description: "100 'A' bytes fed at cols=20 to force a wrapped line before resizing to cols=40",
            bytes: vec![b'A'; 100],
        },
        EngineInput {
            name: "engine_scrollback_input",
            description: "1500 numbered lines 'line 0000'..'line 1499' (CRLF) to overflow scrolling_history=1000",
            bytes: numbered_lines(1500),
        },
        EngineInput {
            name: "selection_input",
            description: "three text rows ('alpha beta gamma' / 'delta epsilon' / 'zeta') for the selection range -> string check",
            bytes: b"alpha beta gamma\r\ndelta epsilon\r\nzeta\r\n".to_vec(),
        },
    ]
}

/// Bytes of one engine-input payload (panics on an unknown name: a bug, not data).
#[must_use]
pub fn engine_input_bytes(name: &str) -> Vec<u8> {
    engine_inputs()
        .into_iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("unknown engine input {name}"))
        .bytes
}

/// Persist every engine-input payload as `<name>.bin` + `<name>.meta`
/// (`kind=engine-input`) under `dir`, so each cited sha256 is recomputable.
pub fn write_engine_inputs(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for input in engine_inputs() {
        fs::write(dir.join(format!("{}.bin", input.name)), &input.bytes)?;
        let meta = format!(
            "kind=engine-input\nname={}\ndescription={}\nbytes={}\nsha256={}\n",
            input.name,
            input.description,
            input.bytes.len(),
            sha256_hex(&input.bytes)
        );
        fs::write(dir.join(format!("{}.meta", input.name)), meta)?;
    }
    Ok(())
}

/// Mutable builder for one check artifact.
pub struct CheckResult {
    pub engine: String,
    pub engine_version: String,
    pub check: String,
    pub title: String,
    pub kind: String,
    pub sources: Vec<SourceRef>,
    pub assertions: Vec<Assertion>,
    pub notes: Vec<String>,
    pub screen_dump: String,
}

impl CheckResult {
    #[must_use]
    pub fn new(env: &Env, check: &str, title: &str, kind: &str) -> Self {
        Self {
            engine: env.engine.to_string(),
            engine_version: env.version.to_string(),
            check: check.to_string(),
            title: title.to_string(),
            kind: kind.to_string(),
            sources: Vec::new(),
            assertions: Vec::new(),
            notes: Vec::new(),
            screen_dump: String::new(),
        }
    }

    pub fn source(&mut self, path: impl Into<String>, bytes: &[u8], kind: &str) {
        self.sources.push(SourceRef {
            path: path.into(),
            sha256: sha256_hex(bytes),
            kind: kind.to_string(),
        });
    }

    pub fn note(&mut self, text: impl Into<String>) {
        self.notes.push(text.into());
    }

    #[must_use]
    pub fn ok(&self) -> bool {
        self.status() != Status::Fail
    }

    /// Aggregate: FAIL beats UNSUPPORTED beats NOT_TESTED beats PASS.
    #[must_use]
    pub fn status(&self) -> Status {
        if self.assertions.is_empty() {
            return Status::NotTested;
        }
        let mut worst = Status::Pass;
        for a in &self.assertions {
            if a.status.severity() > worst.severity() {
                worst = a.status;
            }
        }
        worst
    }

    /// Record an expected/actual comparison.
    pub fn assert<T: PartialEq + Debug>(&mut self, name: &str, expected: T, actual: T) {
        let status = if expected == actual {
            Status::Pass
        } else {
            Status::Fail
        };
        self.push(name, &expected, &actual, status);
    }

    /// Record a boolean assertion with free-form actual text.
    pub fn assert_bool(&mut self, name: &str, condition: bool, expected: &str, actual: &str) {
        let status = if condition {
            Status::Pass
        } else {
            Status::Fail
        };
        self.assertions.push(Assertion {
            name: name.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            status,
        });
    }

    /// Record a not-tested assertion.
    pub fn not_tested(&mut self, name: &str, reason: &str) {
        self.assertions.push(Assertion {
            name: name.to_string(),
            expected: "test executed".to_string(),
            actual: reason.to_string(),
            status: Status::NotTested,
        });
    }

    /// Record an unsupported assertion (field not expressible / not provided).
    pub fn unsupported(&mut self, name: &str, expected: &str, actual: &str) {
        self.assertions.push(Assertion {
            name: name.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            status: Status::Unsupported,
        });
    }

    fn push<T: Debug>(&mut self, name: &str, expected: &T, actual: &T, status: Status) {
        self.assertions.push(Assertion {
            name: name.to_string(),
            expected: format!("{expected:?}"),
            actual: format!("{actual:?}"),
            status,
        });
    }
}

// ---------------------------------------------------------------------------
// Fixture checks
// ---------------------------------------------------------------------------

fn check_vt_basics(env: &Env) -> CheckResult {
    if !env.capabilities().per_cell_style {
        return cap_unsupported(
            env,
            "vt_basics",
            "VT/ANSI basics (CUP/SGR/EL/ED/DECSC)",
            "fixture",
            "per_cell_style",
        );
    }
    let fx = env.fixture("vt_basics").expect("vt_basics fixture");
    let mut r = CheckResult::new(
        env,
        "vt_basics",
        "VT/ANSI basics (CUP/SGR/EL/ED/DECSC)",
        "fixture",
    );
    r.source("results/raw/fixtures/vt_basics.bin", &fx.bytes, "fixture");
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert("row0_text", "AB".to_string(), s.row_text(0));
    r.assert("cell(1,4)_text", "C".to_string(), s.cell(1, 4).text.clone());
    let d = s.cell(1, 5);
    r.assert("cell(1,5)_text", "D".to_string(), d.text.clone());
    r.assert_bool(
        "cell(1,5)_bold",
        d.style.bold,
        "true",
        &d.style.bold.to_string(),
    );
    let e = s.cell(1, 6);
    r.assert("cell(1,6)_text", "E".to_string(), e.text.clone());
    r.assert_bool(
        "cell(1,6)_italic",
        e.style.italic,
        "true",
        &e.style.italic.to_string(),
    );
    let f = s.cell(1, 7);
    r.assert("cell(1,7)_text", "F".to_string(), f.text.clone());
    r.assert_bool(
        "cell(1,7)_underline",
        f.style.underline,
        "true",
        &f.style.underline.to_string(),
    );
    let g = s.cell(1, 8);
    r.assert("cell(1,8)_text", "G".to_string(), g.text.clone());
    r.assert_bool(
        "cell(1,8)_reverse",
        g.style.reverse,
        "true",
        &g.style.reverse.to_string(),
    );
    r.assert("row3_EL_from_col2_text", "KL".to_string(), s.row_text(3));
    r.assert("row4_ED2_cleared_text", String::new(), s.row_text(4));
    r.assert("cell(7,9)_text", "Z".to_string(), s.cell(7, 9).text.clone());
    r.assert(
        "cell(5,2)_after_DECRC_text",
        "z".to_string(),
        s.cell(5, 2).text.clone(),
    );
    r.assert("cursor_row", 5u16, s.cursor.row);
    r.assert("cursor_col", 3u16, s.cursor.col);
    r.assert_bool(
        "cursor_visible",
        s.cursor.visible,
        "true",
        &s.cursor.visible.to_string(),
    );
    r.screen_dump = s.to_plain();
    r
}

fn check_alt_screen_fixture(env: &Env) -> CheckResult {
    if !env.capabilities().alt_screen {
        return cap_unsupported(
            env,
            "alt_screen",
            "Alternate screen DECSET/DECRST 1049",
            "fixture",
            "alt_screen",
        );
    }
    let enter = env.fixture("alt_enter").expect("alt_enter");
    let leave = env.fixture("alt_leave").expect("alt_leave");
    let mut r = CheckResult::new(
        env,
        "alt_screen",
        "Alternate screen DECSET/DECRST 1049",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/alt_enter.bin",
        &enter.bytes,
        "fixture",
    );
    r.source(
        "results/raw/fixtures/alt_leave.bin",
        &leave.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&enter.bytes);
    let in_alt = a.snapshot();
    a.feed(&leave.bytes);
    let out_alt = a.snapshot();
    let alt_row = in_alt.row_text(0);
    r.assert_bool(
        "alt_screen_after_1049h",
        in_alt.alt_screen,
        "true",
        &in_alt.alt_screen.to_string(),
    );
    r.assert_bool(
        "alt_row0_contains_alt",
        alt_row.contains("alt"),
        "alternate-screen row0 contains 'alt'",
        &format!("row0={alt_row:?}"),
    );
    r.assert_bool(
        "primary_text_gone_from_alt",
        !in_alt.to_plain().contains("top"),
        "alternate screen does not show primary 'top' text",
        &format!("alt_plain={:?}", in_alt.to_plain()),
    );
    r.assert_bool(
        "alt_screen_after_1049l",
        !out_alt.alt_screen,
        "false",
        &out_alt.alt_screen.to_string(),
    );
    r.assert(
        "primary_row0_restored_text",
        "topback".to_string(),
        out_alt.row_text(0),
    );
    r.note(format!(
        "DECSET 1049 clears and switches to the alternate screen and does NOT home the cursor when the cursor is shared across screens: alacritty carries the primary cursor (col 3) into the alt grid, so row0 is '   alt'. vt100's alt grid owns a separate cursor, so its row0 is 'alt' (observed row0={alt_row:?}). The exact column is an engine-specific cursor-convention detail; this check asserts alt-screen entry/exit, that the primary text is gone from the alt screen, and that the primary screen/cursor are restored on 1049l."
    ));
    r.screen_dump = out_alt.to_plain();
    r
}

fn check_title_fixture(env: &Env) -> CheckResult {
    if !env.capabilities().title {
        return cap_unsupported(env, "osc_title", "OSC 0/2 window title", "fixture", "title");
    }
    let t0 = env.fixture("title_0").expect("title_0");
    let t2 = env.fixture("title_2").expect("title_2");
    let tr = env.fixture("title_reset").expect("title_reset");
    let mut r = CheckResult::new(env, "osc_title", "OSC 0/2 window title", "fixture");
    r.source("results/raw/fixtures/title_0.bin", &t0.bytes, "fixture");
    r.source("results/raw/fixtures/title_2.bin", &t2.bytes, "fixture");
    r.source("results/raw/fixtures/title_reset.bin", &tr.bytes, "fixture");
    let mut a = env.make_default();
    a.feed(&t0.bytes);
    let s0 = a.snapshot();
    r.assert(
        "title_after_osc0",
        Some("Fixture Title".to_string()),
        s0.title.clone(),
    );
    r.assert_bool(
        "title_osc0_is_event",
        s0.events.iter().any(|e| matches!(e, EngineEvent::Title(_))),
        "Title event present",
        &format!("{:?}", s0.events),
    );
    a.feed(&t2.bytes);
    let s2 = a.snapshot();
    r.assert(
        "title_after_osc2",
        Some("Second Title".to_string()),
        s2.title.clone(),
    );
    a.feed(&tr.bytes);
    let sr = a.snapshot();
    r.assert(
        "title_after_empty_osc0",
        Some(String::new()),
        sr.title.clone(),
    );
    r.screen_dump = sr.to_plain();
    r
}

fn check_osc8_fixture(env: &Env) -> CheckResult {
    if !env.capabilities().osc8_hyperlinks {
        return cap_unsupported(
            env,
            "osc8_hyperlink",
            "OSC 8 per-cell hyperlinks",
            "fixture",
            "osc8_hyperlinks",
        );
    }
    let fx = env.fixture("osc8").expect("osc8");
    let mut r = CheckResult::new(
        env,
        "osc8_hyperlink",
        "OSC 8 per-cell hyperlinks",
        "fixture",
    );
    r.source("results/raw/fixtures/osc8.bin", &fx.bytes, "fixture");
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert("hyperlink_cell_count", 5usize, s.count_hyperlink_cells());
    for (col, expected) in [
        (0u16, "https://example.com/a"),
        (3u16, "https://example.com/a"),
        (4u16, "https://example.org/b"),
    ] {
        r.assert(
            &format!("cell(0,{col})_hyperlink_uri"),
            Some(expected.to_string()),
            s.cell(0, col).hyperlink.clone(),
        );
    }
    r.assert(
        "plain_text_has_no_escape_shown",
        "LINKX".to_string(),
        s.row_text(0),
    );
    r.note(
        "OSC 8 hyperlinks are per-cell in alacritty (Cell::hyperlink().uri()); not an EngineEvent.",
    );
    r.screen_dump = s.to_plain();
    r
}

fn check_bell_fixture(env: &Env) -> CheckResult {
    if !env.capabilities().bell {
        return cap_unsupported(
            env,
            "bell",
            "BEL bell count and EngineEvent::Bell",
            "fixture",
            "bell",
        );
    }
    let fx = env.fixture("bell").expect("bell");
    let mut r = CheckResult::new(
        env,
        "bell",
        "BEL bell count and EngineEvent::Bell",
        "fixture",
    );
    r.source("results/raw/fixtures/bell.bin", &fx.bytes, "fixture");
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert("bell_count", 1u64, s.bell_count);
    r.assert_bool(
        "bell_is_event",
        s.events.iter().any(|e| matches!(e, EngineEvent::Bell)),
        "Bell event present",
        &format!("{:?}", s.events),
    );
    r.assert(
        "row0_text_bel_is_nonprinting",
        "AB".to_string(),
        s.row_text(0),
    );
    r.screen_dump = s.to_plain();
    r
}

fn check_unicode_wide(env: &Env) -> CheckResult {
    if !env.capabilities().wide_glyphs {
        return cap_unsupported(
            env,
            "unicode_wide",
            "CJK wide glyphs occupy 2 columns",
            "fixture",
            "wide_glyphs",
        );
    }
    let fx = env.fixture("unicode_wide").expect("unicode_wide");
    let mut r = CheckResult::new(
        env,
        "unicode_wide",
        "CJK wide glyphs occupy 2 columns",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/unicode_wide.bin",
        &fx.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert("cell(0,0)", "寬".to_string(), s.cell(0, 0).text.clone());
    r.assert(
        "cell(0,1)_continuation",
        String::new(),
        s.cell(0, 1).text.clone(),
    );
    r.assert("cell(0,2)", "字".to_string(), s.cell(0, 2).text.clone());
    r.assert(
        "cell(0,3)_continuation",
        String::new(),
        s.cell(0, 3).text.clone(),
    );
    r.assert("cell(0,4)", "元".to_string(), s.cell(0, 4).text.clone());
    r.assert(
        "cell(0,5)_continuation",
        String::new(),
        s.cell(0, 5).text.clone(),
    );
    r.assert("cursor_advance_cols", 6u16, s.cursor.col);
    r.assert("row0_text", "寬字元".to_string(), s.row_text(0));
    r.note("Wide-glyph continuation cells are represented with empty text in the neutral model.");
    r.screen_dump = s.to_plain();
    r
}

fn check_unicode_combining(env: &Env) -> CheckResult {
    if !env.capabilities().combining_chars {
        return cap_unsupported(
            env,
            "unicode_combining",
            "Combining mark U+0301 attaches to base cell",
            "fixture",
            "combining_chars",
        );
    }
    let fx = env.fixture("unicode_combining").expect("unicode_combining");
    let mut r = CheckResult::new(
        env,
        "unicode_combining",
        "Combining mark U+0301 attaches to base cell",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/unicode_combining.bin",
        &fx.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert(
        "cell(0,0)",
        "e\u{301}".to_string(),
        s.cell(0, 0).text.clone(),
    );
    r.assert("cell(0,1)", "x".to_string(), s.cell(0, 1).text.clone());
    r.assert("cursor_col_after_combining", 2u16, s.cursor.col);
    r.screen_dump = s.to_plain();
    r
}

fn check_unicode_emoji(env: &Env) -> CheckResult {
    if !env.capabilities().combining_chars {
        return cap_unsupported(
            env,
            "unicode_emoji",
            "Emoji base / VS16 / ZWJ per-cell decomposition",
            "fixture",
            "combining_chars",
        );
    }
    let base = env.fixture("unicode_emoji_base").expect("base");
    let vs16 = env.fixture("unicode_vs16").expect("vs16");
    let zwj = env.fixture("unicode_zwj").expect("zwj");
    let mut r = CheckResult::new(
        env,
        "unicode_emoji",
        "Emoji base / VS16 / ZWJ per-cell decomposition",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/unicode_emoji_base.bin",
        &base.bytes,
        "fixture",
    );
    r.source(
        "results/raw/fixtures/unicode_vs16.bin",
        &vs16.bytes,
        "fixture",
    );
    r.source(
        "results/raw/fixtures/unicode_zwj.bin",
        &zwj.bytes,
        "fixture",
    );

    let mut a = env.make_default();
    a.feed(&base.bytes);
    let s = a.snapshot();
    r.assert(
        "emoji_base_cell(0,0)",
        "😀".to_string(),
        s.cell(0, 0).text.clone(),
    );
    r.assert(
        "emoji_base_cell(0,1)_continuation",
        String::new(),
        s.cell(0, 1).text.clone(),
    );
    r.assert("emoji_base_cursor_col", 2u16, s.cursor.col);

    let mut a = env.make_default();
    a.feed(&vs16.bytes);
    let s = a.snapshot();
    r.assert(
        "vs16_cell(0,0)",
        "\u{2764}\u{FE0F}".to_string(),
        s.cell(0, 0).text.clone(),
    );
    r.assert("vs16_cursor_col", 1u16, s.cursor.col);
    r.note("alacritty uses unicode-width per-char width: U+2764 is width 1 and U+FE0F (VS16) is width 0, attached to the base cell. VS16 does not widen the cell to 2 columns in this engine.");

    let mut a = env.make_default();
    a.feed(&zwj.bytes);
    let s = a.snapshot();
    r.assert(
        "zwj_cell(0,0)",
        "👨\u{200D}".to_string(),
        s.cell(0, 0).text.clone(),
    );
    r.assert(
        "zwj_cell(0,1)_continuation",
        String::new(),
        s.cell(0, 1).text.clone(),
    );
    r.assert(
        "zwj_cell(0,2)",
        "👩\u{200D}".to_string(),
        s.cell(0, 2).text.clone(),
    );
    r.assert(
        "zwj_cell(0,3)_continuation",
        String::new(),
        s.cell(0, 3).text.clone(),
    );
    r.assert("zwj_cell(0,4)", "👧".to_string(), s.cell(0, 4).text.clone());
    r.assert(
        "zwj_cell(0,5)_continuation",
        String::new(),
        s.cell(0, 5).text.clone(),
    );
    r.assert("zwj_cursor_col", 6u16, s.cursor.col);
    r.note("alacritty does no grapheme clustering: each emoji is a 2-column wide char and each ZWJ (U+200D) is a width-0 mark attached to the preceding base cell.");
    r.screen_dump = s.to_plain();
    r
}

fn check_colors_fixture(env: &Env) -> CheckResult {
    use crate::model::Color;
    let caps = env.capabilities();
    if !caps.per_cell_style {
        return cap_unsupported(
            env,
            "colors",
            "Named / 256-index / truecolor SGR parsing",
            "fixture",
            "per_cell_style",
        );
    }
    let fx = env.fixture("colors").expect("colors");
    let mut r = CheckResult::new(
        env,
        "colors",
        "Named / 256-index / truecolor SGR parsing",
        "fixture",
    );
    r.source("results/raw/fixtures/colors.bin", &fx.bytes, "fixture");
    let mut a = env.make_default();
    a.feed(&fx.bytes);
    let s = a.snapshot();
    r.assert(
        "cell(0,0)_fg_named_red(31)",
        Color::Indexed(1),
        s.cell(0, 0).style.fg,
    );
    r.assert(
        "cell(0,1)_fg_bright_red(91)",
        Color::Indexed(9),
        s.cell(0, 1).style.fg,
    );
    if !caps.indexed_256 {
        r.unsupported(
            "cell(0,2)_fg_indexed_196",
            "Color::Indexed(196)",
            "engine declares indexed_256 absent",
        );
        r.unsupported(
            "cell(0,3)_bg_indexed_21",
            "Color::Indexed(21)",
            "engine declares indexed_256 absent",
        );
    } else {
        r.assert(
            "cell(0,2)_fg_indexed_196",
            Color::Indexed(196),
            s.cell(0, 2).style.fg,
        );
        r.assert(
            "cell(0,3)_bg_indexed_21",
            Color::Indexed(21),
            s.cell(0, 3).style.bg,
        );
    }
    r.assert(
        "cell(0,3)_fg_default",
        Color::Default,
        s.cell(0, 3).style.fg,
    );
    if !caps.truecolor {
        r.unsupported(
            "cell(0,4)_fg_truecolor",
            "Color::Rgb(12, 34, 56)",
            "engine declares truecolor absent",
        );
        r.unsupported(
            "cell(0,5)_bg_truecolor",
            "Color::Rgb(210, 211, 212)",
            "engine declares truecolor absent",
        );
    } else {
        r.assert(
            "cell(0,4)_fg_truecolor",
            Color::Rgb(12, 34, 56),
            s.cell(0, 4).style.fg,
        );
        r.assert(
            "cell(0,5)_bg_truecolor",
            Color::Rgb(210, 211, 212),
            s.cell(0, 5).style.bg,
        );
    }
    r.note(format!(
        "palette -> RGB resolution is UNVERIFIED: {} (alacritty Term::colors() is empty until a frontend populates it).",
        crate::palette::PALETTE_STATUS
    ));
    r.note("Named special colours (Cursor / Dim* / BrightForeground) have no exact neutral representation and collapse to Color::Default; no check depends on them.");
    r.screen_dump = s.to_plain();
    r
}

fn check_cursor_keys_fixture(env: &Env) -> CheckResult {
    if !env.capabilities().app_cursor_keys {
        return cap_unsupported(
            env,
            "cursor_keys",
            "DECSET/DECRST 1 application cursor keys",
            "fixture",
            "app_cursor_keys",
        );
    }
    let on = env.fixture("cursor_keys_on").expect("cursor_keys_on");
    let off = env.fixture("cursor_keys_off").expect("cursor_keys_off");
    let mut r = CheckResult::new(
        env,
        "cursor_keys",
        "DECSET/DECRST 1 application cursor keys",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/cursor_keys_on.bin",
        &on.bytes,
        "fixture",
    );
    r.source(
        "results/raw/fixtures/cursor_keys_off.bin",
        &off.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&on.bytes);
    let s_on = a.snapshot();
    a.feed(&off.bytes);
    let s_off = a.snapshot();
    r.assert_bool(
        "app_cursor_keys_after_?1h",
        s_on.app_cursor_keys,
        "true",
        &s_on.app_cursor_keys.to_string(),
    );
    r.assert_bool(
        "app_cursor_keys_after_?1l",
        !s_off.app_cursor_keys,
        "false",
        &s_off.app_cursor_keys.to_string(),
    );
    r.note("Encoding the arrow key itself (SS3 vs CSI) is a frontend (UI-001) concern; the engine only reports the mode flag, which is what is asserted here.");
    r.screen_dump = s_off.to_plain();
    r
}

fn check_bracketed_paste_mode(env: &Env) -> CheckResult {
    if !env.capabilities().bracketed_paste_mode {
        return cap_unsupported(
            env,
            "bracketed_paste_mode",
            "Bracketed paste mode (DECSET/DECRST 2004) flag",
            "fixture",
            "bracketed_paste_mode",
        );
    }
    let on = env.fixture("bracketed_paste_on").expect("on");
    let off = env.fixture("bracketed_paste_off").expect("off");
    let mut r = CheckResult::new(
        env,
        "bracketed_paste_mode",
        "Bracketed paste mode (DECSET/DECRST 2004) flag",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/bracketed_paste_on.bin",
        &on.bytes,
        "fixture",
    );
    r.source(
        "results/raw/fixtures/bracketed_paste_off.bin",
        &off.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&on.bytes);
    let s_on = a.snapshot();
    a.feed(&off.bytes);
    let s_off = a.snapshot();
    r.assert_bool(
        "bracketed_paste_after_?2004h",
        s_on.bracketed_paste,
        "true",
        &s_on.bracketed_paste.to_string(),
    );
    r.assert_bool(
        "bracketed_paste_after_?2004l",
        !s_off.bracketed_paste,
        "false",
        &s_off.bracketed_paste.to_string(),
    );
    r.note("This split-off check asserts only the DECSET/DECRST 2004 mode flag, which every engine that tracks it exposes through the neutral model. Payload literal-text insertion is a separate input-path property (bracketed_paste_payload).");
    r.screen_dump = s_off.to_plain();
    r
}

/// The bracketed-paste *payload* leg. Always `UNSUPPORTED`: see the note.
fn check_bracketed_paste_payload(env: &Env) -> CheckResult {
    let payload = env.fixture("bracketed_paste_payload").expect("payload");
    let mut r = CheckResult::new(
        env,
        "bracketed_paste_payload",
        "Bracketed paste payload bytes-as-data (input-path property)",
        "fixture",
    );
    r.source(
        "results/raw/fixtures/bracketed_paste_payload.bin",
        &payload.bytes,
        "fixture",
    );
    let mut a = env.make_default();
    a.feed(&payload.bytes);
    let s = a.snapshot();
    r.unsupported(
        "payload_esc_bytes_are_literal_text",
        "ESC inside ESC[200~..ESC[201~ rendered as literal text",
        &format!(
            "engine parsed ESC[31m as SGR; row0={:?}; cell(0,1).fg={:?}",
            s.row_text(0),
            s.cell(0, 1).style.fg
        ),
    );
    r.note("Bracketed paste is an input-side mode: the program receives the payload, and the frontend wraps it in ESC[200~..ESC[201~ in the terminal->app direction. On the OUTPUT direction this harness feeds, an emulator correctly interprets ESC bytes inside the payload as control sequences rather than literal text; the 'bytes as data' guarantee is an INPUT-path property owned by CORE-001 / app-side work and cannot be asserted from an output-only feed. Recorded UNSUPPORTED for every engine, including alacritty_terminal.");
    r.note(format!(
        "Observed payload screen (output-feed interpretation): row0={:?}; cell(0,1).fg={:?}",
        s.row_text(0),
        s.cell(0, 1).style.fg
    ));
    r.screen_dump = s.to_plain();
    r
}

fn check_mouse_fixture(env: &Env) -> CheckResult {
    let caps = env.capabilities();
    if !caps.mouse_modes {
        return cap_unsupported(
            env,
            "mouse_modes",
            "Mouse reporting modes 1000/1002/1003/1006",
            "fixture",
            "mouse_modes",
        );
    }
    if !caps.mouse_sgr_flag {
        return cap_unsupported(
            env,
            "mouse_modes",
            "Mouse reporting modes 1000/1002/1003/1006",
            "fixture",
            "mouse_sgr_flag",
        );
    }
    let mut r = CheckResult::new(
        env,
        "mouse_modes",
        "Mouse reporting modes 1000/1002/1003/1006",
        "fixture",
    );
    let mut a = env.make_default();
    for (fixture, expect_mode, name) in [
        ("mouse_1000", MouseMode::Click, "1000_click"),
        ("mouse_1002", MouseMode::Drag, "1002_drag"),
        ("mouse_1003", MouseMode::Any, "1003_any"),
    ] {
        let fx = env.fixture(fixture).expect(fixture);
        r.source(
            format!("results/raw/fixtures/{fixture}.bin"),
            &fx.bytes,
            "fixture",
        );
        a.feed(&fx.bytes);
        let s = a.snapshot();
        r.assert(
            &format!("mouse_mode_after_{name}"),
            expect_mode,
            s.mouse_mode,
        );
    }
    let sgr = env.fixture("mouse_1006").expect("mouse_1006");
    r.source("results/raw/fixtures/mouse_1006.bin", &sgr.bytes, "fixture");
    a.feed(&sgr.bytes);
    let s = a.snapshot();
    r.assert_bool(
        "mouse_sgr_after_1006",
        s.mouse_sgr,
        "true",
        &s.mouse_sgr.to_string(),
    );
    r.note("Mouse protocols are mutually exclusive in alacritty; 1003 replaces the prior mode. SGR encoding is independent of the mode and stays enabled.");
    r.screen_dump = s.to_plain();
    r
}

fn check_resize_engine(env: &Env) -> CheckResult {
    if !env.capabilities().reflow_on_resize {
        return cap_unsupported(
            env,
            "resize_reflow",
            "Resize + logical-line reflow (engine leg)",
            "engine",
            "reflow_on_resize",
        );
    }
    let mut r = CheckResult::new(
        env,
        "resize_reflow",
        "Resize + logical-line reflow (engine leg)",
        "engine",
    );
    let payload = engine_input_bytes("engine_resize_input");
    r.source(
        "results/raw/fixtures/engine_resize_input.bin",
        &payload,
        "engine-input",
    );
    let mut a = env.make(20, 10);
    a.feed(&payload);
    let before = a.snapshot();
    r.assert("cols_before_resize", 20u16, before.cols);
    r.assert("rows_before_resize", 10u16, before.rows);
    a.resize(40, 10);
    let after = a.snapshot();
    r.assert("cols_after_resize", 40u16, after.cols);
    r.assert("rows_after_resize", 10u16, after.rows);
    r.assert("reflow_row0", "A".repeat(40), after.row_text(0));
    r.assert("reflow_row1", "A".repeat(40), after.row_text(1));
    r.assert("reflow_row2", "A".repeat(20), after.row_text(2));
    r.assert("reflow_row3_empty", String::new(), after.row_text(3));
    r.note("Fed 100 'A' (results/raw/fixtures/engine_resize_input.bin) at cols=20 (wrapped), then resized to cols=40; expected 40/40/20 reflow.");
    r.screen_dump = after.to_plain();
    r
}

fn check_scrollback_engine(env: &Env) -> CheckResult {
    if !env.capabilities().scrollback_bound {
        return cap_unsupported(
            env,
            "scrollback_bound",
            "Scrollback bound at scrolling_history=1000",
            "engine",
            "scrollback_bound",
        );
    }
    if env.engine == crate::engine::alacritty::NAME {
        check_scrollback_engine_alacritty(env)
    } else {
        check_scrollback_engine_neutral(env)
    }
}

/// Text of the last visible row that is not blank, with its row number.
fn last_nonblank_visible_row(s: &Snapshot) -> Option<(u16, String)> {
    (0..s.rows)
        .rev()
        .map(|row| (row, s.row_text(row)))
        .find(|(_, text)| !text.trim().is_empty())
}

/// Every line of `text` containing `needle` (the `present=[...]` evidence that
/// replaces the old placeholder `checked` literals).
fn matching_lines(text: &str, needle: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.contains(needle))
        .map(str::to_string)
        .collect()
}

/// `alacritty_terminal`-specific leg: uses the adapter's deep history dump to
/// assert the exact oldest retained line (the neutral model deliberately does
/// not expose scrollback text).
fn check_scrollback_engine_alacritty(env: &Env) -> CheckResult {
    let mut r = CheckResult::new(
        env,
        "scrollback_bound",
        "Scrollback bound at scrolling_history=1000",
        "engine",
    );
    let rows = 24u16;
    let cols = 80u16;
    let bound = 1000usize;
    // Concrete adapter so the retained history text itself can be inspected.
    let mut a = AlacrittyAdapter::new(cols, rows, bound);
    let payload = engine_input_bytes("engine_scrollback_input");
    r.source(
        "results/raw/fixtures/engine_scrollback_input.bin",
        &payload,
        "engine-input",
    );
    a.feed(&payload);
    let s = a.snapshot();
    let plain = s.to_plain();
    // D7: `configured_bound` used to compare a harness constant against the
    // adapter echoing its own constructor argument (unfalsifiable). It is now
    // an informational note; the assertions below can actually fail.
    r.assert_bool(
        "retained_lines_le_configured_bound",
        s.scrollback.retained_lines <= s.scrollback.configured_bound,
        "retained_lines <= configured_bound",
        &format!(
            "retained_lines={} configured_bound={}",
            s.scrollback.retained_lines, s.scrollback.configured_bound
        ),
    );
    r.assert_bool(
        "retained_lines_positive",
        s.scrollback.retained_lines > 0,
        "retained_lines > 0 under overflow",
        &s.scrollback.retained_lines.to_string(),
    );
    r.assert("retained_lines_bounded", bound, s.scrollback.retained_lines);
    let newest_needle = "line 1499";
    let newest_present = matching_lines(&plain, newest_needle);
    r.assert_bool(
        "newest_line_present_in_visible",
        plain.contains(newest_needle),
        &format!("visible screen contains {newest_needle:?}"),
        &format!(
            "last_non_blank_visible_row={:?}; present={newest_present:?}",
            last_nonblank_visible_row(&s)
        ),
    );
    let oldest_needle = "line 0000";
    let oldest_present = matching_lines(&plain, oldest_needle);
    r.assert_bool(
        "oldest_line_gone_from_visible",
        oldest_present.is_empty(),
        &format!("visible screen does not contain {oldest_needle:?}"),
        &format!(
            "present={oldest_present:?}; absent={:?}",
            if oldest_present.is_empty() {
                vec![oldest_needle]
            } else {
                Vec::new()
            }
        ),
    );
    let history = a.scrollback_lines();
    let history_text: String = history
        .iter()
        .map(|(_, t)| t.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let oldest_history_present = matching_lines(&history_text, oldest_needle);
    r.assert_bool(
        "oldest_line_gone_from_scrollback",
        oldest_history_present.is_empty(),
        &format!("scrollback history does not contain {oldest_needle:?}"),
        &format!(
            "present={oldest_history_present:?}; oldest_retained={:?}; history_len={}",
            history.last().map(|(_, t)| t.clone()),
            history.len()
        ),
    );
    r.assert_bool(
        "history_len_equals_bound",
        history.len() == bound,
        &bound.to_string(),
        &history.len().to_string(),
    );
    // Exact boundary: with rows=24 + history=1000 the retained window is the
    // last 1024 logical lines, so grid row 0 (oldest, age 1000) is 'line 0477'.
    let oldest = history.last().map(|(_, t)| t.clone()).unwrap_or_default();
    r.assert("oldest_retained_line", "line 0477".to_string(), oldest);
    // Newest scrollback entry (age 1) is the line just above the viewport.
    let newest_scroll = history.first().map(|(_, t)| t.clone()).unwrap_or_default();
    r.note(format!(
        "configured scrolling_history (informational only, not an assertion): {bound}"
    ));
    r.note(format!(
        "engine-specific extension: exact retained window asserted as line 0477 (oldest, age 1000) .. line 1499 (visible). newest scrollback (age 1)={newest_scroll:?}"
    ));
    r.screen_dump = plain;
    r
}

/// Neutral leg for comparison engines: asserts only what the neutral model can
/// express. The exact oldest-retained-line text is engine-specific and is not
/// asserted here (alacritty's deep dump is a separate engine-specific leg).
fn check_scrollback_engine_neutral(env: &Env) -> CheckResult {
    let mut r = CheckResult::new(
        env,
        "scrollback_bound",
        "Scrollback bound at scrolling_history=1000",
        "engine",
    );
    let rows = 24u16;
    let cols = 80u16;
    let bound = 1000usize;
    let mut a = env.make(cols, rows);
    let payload = engine_input_bytes("engine_scrollback_input");
    r.source(
        "results/raw/fixtures/engine_scrollback_input.bin",
        &payload,
        "engine-input",
    );
    a.feed(&payload);
    let s = a.snapshot();
    let plain = s.to_plain();
    r.assert_bool(
        "retained_lines_bounded",
        s.scrollback.retained_lines <= bound,
        &format!("retained_lines <= {bound}"),
        &s.scrollback.retained_lines.to_string(),
    );
    r.assert_bool(
        "retained_lines_positive",
        s.scrollback.retained_lines > 0,
        "retained_lines > 0",
        &s.scrollback.retained_lines.to_string(),
    );
    r.assert_bool(
        "retained_lines_le_configured_bound",
        s.scrollback.retained_lines <= s.scrollback.configured_bound,
        "retained_lines <= configured_bound",
        &format!(
            "retained_lines={} configured_bound={}",
            s.scrollback.retained_lines, s.scrollback.configured_bound
        ),
    );
    let newest_needle = "line 1499";
    let newest_present = matching_lines(&plain, newest_needle);
    r.assert_bool(
        "newest_line_present_in_visible",
        plain.contains(newest_needle),
        &format!("visible screen contains {newest_needle:?}"),
        &format!(
            "last_non_blank_visible_row={:?}; present={newest_present:?}",
            last_nonblank_visible_row(&s)
        ),
    );
    let oldest_needle = "line 0000";
    let oldest_present = matching_lines(&plain, oldest_needle);
    r.assert_bool(
        "oldest_line_gone_from_visible",
        oldest_present.is_empty(),
        &format!("visible screen does not contain {oldest_needle:?}"),
        &format!(
            "present={oldest_present:?}; absent={:?}",
            if oldest_present.is_empty() {
                vec![oldest_needle]
            } else {
                Vec::new()
            }
        ),
    );
    r.note(format!(
        "neutral scrollback leg for {} {}: retained_lines={} (configured_bound={}, informational only). The exact oldest-retained-line text is an engine-specific window-arithmetic detail and is asserted only by the alacritty-specific leg.",
        env.engine, env.version, s.scrollback.retained_lines, s.scrollback.configured_bound
    ));
    r.screen_dump = plain;
    r
}

/// Engine-specific text-range selection check.
///
/// The neutral model has no selection field, so this uses the concrete adapter
/// for engines whose pinned source exposes a range -> string API:
/// * `alacritty_terminal 0.26.0`: `Term::selection_to_string()` (public
///   `Term::selection: Option<Selection>`; `alacritty_terminal-0.26.0/src/
///   term/mod.rs:529` and `src/selection.rs:119`);
/// * `vt100 0.16.2`: `Screen::contents_between(start_row, start_col, end_row,
///   end_col) -> String` (`vt100-0.16.2/src/screen.rs:167`), documented as the
///   clipboard-selection helper.
///
/// `avt 0.18.0` provides no selection/clipboard helper API: its public
/// `Terminal::view()/lines()/line(n)` accessors
/// (`avt-0.18.0/src/terminal.rs:636-644`) expose lines/cells but there is no
/// range -> `String` counterpart to `alacritty_terminal`'s
/// `Term::selection_to_string` or `vt100`'s `Screen::contents_between`.
/// Engines without such a helper are recorded `UNSUPPORTED`, never PASS/FAIL.
fn check_selection(env: &Env) -> CheckResult {
    let title = "Text range selection -> string (engine-specific)";
    if !env.capabilities().selection_to_string {
        return cap_unsupported(env, "selection", title, "engine", "selection_to_string");
    }
    let mut r = CheckResult::new(env, "selection", title, "engine");
    let payload = engine_input_bytes("selection_input");
    r.source(
        "results/raw/fixtures/selection_input.bin",
        &payload,
        "engine-input",
    );
    if env.engine == crate::engine::alacritty::NAME {
        let mut a = AlacrittyAdapter::new(80, 24, env.scrollback);
        a.feed(&payload);
        let got = a.select_range_to_string((0, 6), (2, 3));
        r.assert(
            "selection_to_string_rows_0_6_to_2_3",
            Some("beta gamma\ndelta epsilon\nzeta".to_string()),
            got,
        );
        r.note("alacritty simple selection end column is inclusive (line_length = end_col + 1): (0,6)..(2,3) yields 'zet' + the last cell 'a'. Expectation established by feeding results/raw/fixtures/selection_input.bin into alacritty_terminal 0.26.0 and calling Term::selection_to_string().");
    } else if env.engine == crate::engine::vt100::NAME {
        let mut a = Vt100Adapter::new(80, 24, env.scrollback);
        a.feed(&payload);
        let got = a.contents_between(0, 6, 2, 3);
        r.assert(
            "contents_between_rows_0_6_to_2_3",
            "beta gamma\ndelta epsilon\nzet".to_string(),
            got,
        );
        r.note("vt100 contents_between start column is inclusive and end column is exclusive (up until end_col): (0,6)..(2,3) yields 'zet'. Expectation established by feeding results/raw/fixtures/selection_input.bin into vt100 0.16.2 and calling Screen::contents_between().");
    } else {
        r.not_tested(
            "selection_engine_leg",
            &format!(
                "no selection leg wired for {} despite a true capability declaration",
                env.engine
            ),
        );
    }
    r.screen_dump = payload
        .split(|b| *b == b'\n')
        .map(|l| {
            String::from_utf8_lossy(l)
                .trim_end_matches('\r')
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    r
}

// ---------------------------------------------------------------------------
// Robustness checks
// ---------------------------------------------------------------------------

/// Feed every fixture as one slice and as one byte at a time; the resulting
/// snapshots must be identical. This is genuinely valuable for `avt`, whose
/// feed is char-based and therefore owns an incremental UTF-8 decoder.
/// Compare two snapshots for chunk-boundary invariance and describe every
/// difference. This includes the full per-cell grid (text, fg/bg, bold, italic,
/// underline, reverse, OSC 8 hyperlink) because a parser whose UTF-8 decoder is
/// chunk-sensitive can corrupt a style run or hyperlink even when the plain
/// text matches.
fn snapshot_invariance(a: &Snapshot, b: &Snapshot) -> (bool, String) {
    let mut diffs: Vec<String> = Vec::new();
    if a.to_plain() != b.to_plain() {
        diffs.push(format!(
            "plain_text whole={:?} bytewise={:?}",
            a.to_plain(),
            b.to_plain()
        ));
    }
    if a.cursor != b.cursor {
        diffs.push(format!(
            "cursor whole={:?} bytewise={:?}",
            a.cursor, b.cursor
        ));
    }
    if a.alt_screen != b.alt_screen {
        diffs.push(format!(
            "alt_screen whole={} bytewise={}",
            a.alt_screen, b.alt_screen
        ));
    }
    if a.bracketed_paste != b.bracketed_paste {
        diffs.push(format!(
            "bracketed_paste whole={} bytewise={}",
            a.bracketed_paste, b.bracketed_paste
        ));
    }
    if a.app_cursor_keys != b.app_cursor_keys {
        diffs.push(format!(
            "app_cursor_keys whole={} bytewise={}",
            a.app_cursor_keys, b.app_cursor_keys
        ));
    }
    if a.mouse_mode != b.mouse_mode {
        diffs.push(format!(
            "mouse_mode whole={:?} bytewise={:?}",
            a.mouse_mode, b.mouse_mode
        ));
    }
    if a.mouse_sgr != b.mouse_sgr {
        diffs.push(format!(
            "mouse_sgr whole={} bytewise={}",
            a.mouse_sgr, b.mouse_sgr
        ));
    }
    if a.title != b.title {
        diffs.push(format!("title whole={:?} bytewise={:?}", a.title, b.title));
    }
    if a.bell_count != b.bell_count {
        diffs.push(format!(
            "bell_count whole={} bytewise={}",
            a.bell_count, b.bell_count
        ));
    }
    let cols = usize::from(a.cols.max(1));
    let mut differing_cells = 0usize;
    let mut first_cell: Option<String> = None;
    for (i, (ca, cb)) in a.cells.iter().zip(b.cells.iter()).enumerate() {
        if ca != cb {
            differing_cells += 1;
            if first_cell.is_none() {
                let row = i / cols;
                let col = i % cols;
                first_cell = Some(format!(
                    "cell(row={row},col={col}) whole={:?} bytewise={:?}",
                    ca, cb
                ));
            }
        }
    }
    if differing_cells > 0 {
        diffs.push(format!(
            "differing_cells={differing_cells} first={}",
            first_cell.unwrap_or_default()
        ));
    }
    if a.cells.len() != b.cells.len() {
        diffs.push(format!(
            "cells_len whole={} bytewise={}",
            a.cells.len(),
            b.cells.len()
        ));
    }
    if diffs.is_empty() {
        (true, "no differences".to_string())
    } else {
        (false, diffs.join("; "))
    }
}

/// Flag names that this engine declares absent, so they are constant for it and
/// cannot discriminate a chunk-boundary difference.
fn constant_by_declaration(env: &Env) -> Vec<&'static str> {
    let caps = env.capabilities();
    let mut out = Vec::new();
    if !caps.alt_screen {
        out.push("alt_screen");
    }
    if !caps.bracketed_paste_mode {
        out.push("bracketed_paste");
    }
    if !caps.app_cursor_keys {
        out.push("app_cursor_keys");
    }
    if !caps.mouse_modes {
        out.push("mouse_mode");
    }
    if !caps.mouse_sgr_flag {
        out.push("mouse_sgr");
    }
    if !caps.title {
        out.push("title");
    }
    if !caps.bell {
        out.push("bell_count");
    }
    out
}

/// Expected-no-op classification for one fixture/engine pair.
///
/// A fixture is a *legitimate* no-op only for one of two auditable reasons:
/// (1) it is a DECRST/clear sequence whose target state is already the engine
/// default, or (2) it exercises exactly one capability this engine declares
/// absent, so the neutral snapshot cannot change. Every no-op is named here and
/// re-asserted in the artifact; an unlisted no-op is a failure, never a silent
/// skip.
fn noop_fixture_reason(env: &Env, name: &str) -> Option<&'static str> {
    let caps = env.capabilities();
    match name {
        // DECRST/off fixtures when the default state already matches.
        "cursor_keys_off" => {
            Some("DECRST 1 when the engine default is already application-cursor-keys off")
        }
        "bracketed_paste_off" => {
            Some("DECRST 2004 when the engine default is already bracketed-paste off")
        }
        // Fixtures whose only observable effect is a capability declared absent.
        "title_0" | "title_2" | "title_reset" if !caps.title => {
            Some("OSC 0/2 title fixture, but this engine declares `title` absent")
        }
        "mouse_1000" | "mouse_1002" | "mouse_1003" if !caps.mouse_modes => {
            Some("mouse-reporting fixture, but this engine declares `mouse_modes` absent")
        }
        "mouse_1006" if !caps.mouse_sgr_flag => {
            Some("SGR mouse-encoding fixture, but this engine declares `mouse_sgr_flag` absent")
        }
        "bracketed_paste_on" if !caps.bracketed_paste_mode => {
            Some("bracketed-paste fixture, but this engine declares `bracketed_paste_mode` absent")
        }
        _ => None,
    }
}

/// Non-blank visible rows of `s` as `(row, text)`.
fn nonblank_rows(s: &Snapshot) -> Vec<(u16, String)> {
    (0..s.rows)
        .map(|row| (row, s.row_text(row)))
        .filter(|(_, text)| !text.trim().is_empty())
        .collect()
}

/// Compact but complete comparison evidence emitted on every PASS: the
/// plain-text dump digest + length, first/last non-blank rows, cursor, cell
/// count, and the constant-by-declaration list. The digest (rather than the
/// full dump) keeps the artifact small while remaining recomputable from a real
/// feed.
fn snapshot_evidence(env: &Env, s: &Snapshot) -> String {
    let plain = s.to_plain();
    let rows = nonblank_rows(s);
    format!(
        "plain_sha256={} plain_len={} cursor=(row={},col={},visible={}) cells={} first_nonblank_row={:?} last_nonblank_row={:?} constant_by_declaration={:?}",
        sha256_hex(plain.as_bytes()),
        plain.len(),
        s.cursor.row,
        s.cursor.col,
        s.cursor.visible,
        s.cells.len(),
        rows.first(),
        rows.last(),
        constant_by_declaration(env),
    )
}

fn check_chunk_boundary_invariance(env: &Env) -> CheckResult {
    let mut r = CheckResult::new(
        env,
        "chunk_boundary_invariance",
        "Feed chunk-boundary invariance (one slice vs one byte at a time)",
        "engine",
    );
    if env.fixtures.is_empty() {
        r.not_tested("fixtures_available", "no fixtures were loaded");
        return r;
    }
    let pristine = env.make_default().snapshot();
    let constant = constant_by_declaration(env);
    let mut expected_noops: Vec<String> = Vec::new();
    for fx in env.fixtures {
        let mut whole = env.make_default();
        whole.feed(&fx.bytes);
        let a = whole.snapshot();
        let mut bytewise = env.make_default();
        for b in &fx.bytes {
            bytewise.feed(std::slice::from_ref(b));
        }
        let b = bytewise.snapshot();
        let (same, detail) = snapshot_invariance(&a, &b);
        // Falsifiability term: feeding the fixture must move the screen away
        // from a pristine engine. An engine that ignores its input compares
        // equal to pristine for every fixture and therefore fails here.
        let changed = a != pristine;
        let evidence = snapshot_evidence(env, &a);
        match noop_fixture_reason(env, fx.name) {
            // Auditable exception: a named, reasoned no-op is excluded from the
            // changed-vs-pristine term, but must actually be a no-op.
            Some(reason) => {
                expected_noops.push(format!("{}: {reason}", fx.name));
                r.assert_bool(
                    &format!("fixture_{}_expected_noop", fx.name),
                    !changed,
                    &format!("expected no-op ({reason}); snapshot identical to pristine"),
                    &format!("changed_vs_pristine={changed}; reason={reason}; {evidence}"),
                );
                r.assert_bool(
                    &format!("fixture_{}_chunk_invariant", fx.name),
                    same,
                    "identical snapshot for 1-slice and byte-at-a-time feeds",
                    &format!(
                        "same={same}; expected_noop=true; reason={reason}; {detail}; {evidence}"
                    ),
                );
            }
            None => {
                r.assert_bool(
                    &format!("fixture_{}_chunk_invariant", fx.name),
                    same,
                    "identical snapshot for 1-slice and byte-at-a-time feeds",
                    &format!("same={same}; changed_vs_pristine={changed}; {detail}; {evidence}"),
                );
                r.assert_bool(
                    &format!("fixture_{}_changed_vs_pristine", fx.name),
                    changed,
                    "feeding the fixture changes the snapshot relative to a pristine engine (non-no-op)",
                    &format!(
                        "changed_vs_pristine={changed}; fixture_sha256={}; {evidence}",
                        sha256_hex(&fx.bytes)
                    ),
                );
            }
        }
    }
    r.note(format!(
        "Compares, for every fixture, the full visible grid (per-cell text, fg/bg, bold/italic/underline/reverse and OSC 8 hyperlink), the cursor, and every neutral-model flag (alt screen, bracketed paste, app cursor keys, mouse mode, mouse SGR, title, bell count). Terms that are constant-by-declaration for {} and therefore cannot discriminate: {:?}. Every PASS also emits the compared snapshot evidence (plain-text sha256 + length + first/last non-blank rows, cursor, cell count, constant-by-declaration list) and asserts each non-no-op fixture actually changes the screen relative to a pristine engine, so an engine that ignores its input cannot PASS. Expected no-ops (excluded from the changed-vs-pristine term by name, each re-asserted as a no-op above): {:?}.",
        env.engine, constant, expected_noops
    ));
    r
}

// ---------------------------------------------------------------------------
// Real-program checks
// ---------------------------------------------------------------------------

fn alt_screen_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().alt_screen {
        return cap_unsupported(
            env,
            "alt_screen_real",
            "Alternate screen (real nvim)",
            "real-program",
            "alt_screen",
        );
    }
    let mut r = CheckResult::new(
        env,
        "alt_screen_real",
        "Alternate screen (real nvim)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    let replay = replay_with_alt_watch(a.as_mut(), &stream.bytes);
    let enters: Vec<usize> = replay
        .sequences
        .iter()
        .filter(|s| s.enter)
        .map(|s| s.start)
        .collect();
    let leaves: Vec<usize> = replay
        .sequences
        .iter()
        .filter(|s| !s.enter)
        .map(|s| s.start)
        .collect();
    r.assert_bool(
        "raw_bytes_1049h_present",
        !enters.is_empty(),
        "capture bytes contain at least one ESC[?1049h",
        &format!("{:?}", enters),
    );
    r.assert_bool(
        "raw_bytes_1049l_present",
        !leaves.is_empty(),
        "capture bytes contain at least one ESC[?1049l",
        &format!("{:?}", leaves),
    );
    let enter_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    let leave_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| !t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    r.assert_bool(
        "engine_alt_true_after_every_1049h",
        !enter_states.is_empty() && enter_states.iter().all(|v| *v),
        "engine alt_screen == true after every 1049h",
        &format!("{enter_states:?}"),
    );
    r.assert_bool(
        "engine_alt_false_after_every_1049l",
        !leave_states.is_empty() && leave_states.iter().all(|v| !*v),
        "engine alt_screen == false after every 1049l",
        &format!("{leave_states:?}"),
    );
    r.assert_bool(
        "alt_screen_had_content",
        replay
            .last_alt_snapshot
            .as_ref()
            .is_some_and(|s| !s.to_plain().trim().is_empty()),
        "a snapshot taken while the engine was in the alt screen is non-blank",
        &format!(
            "alt_screen_nonblank={}",
            replay
                .last_alt_snapshot
                .as_ref()
                .is_some_and(|s| !s.to_plain().trim().is_empty())
        ),
    );
    r.note(
        "Alt-screen transitions are detected two ways: (1) chunk-independent raw-byte scan for ESC[?1049h/l (the phase-1 method) and (2) the engine's own alt_screen flag snapshotted at each transition by replay_with_alt_watch. The old chunk-sampled `watch` field is no longer used as alt-screen evidence.",
    );
    // Dump the snapshot the `alt_screen_had_content` assertion was actually
    // evaluated on (the last alt-screen snapshot), not the blank final primary
    // screen, so the asserted content is visible. Matches replay_phase1.
    r.screen_dump = replay
        .last_alt_snapshot
        .as_ref()
        .unwrap_or(&replay.final_snapshot)
        .to_plain();
    r
}

/// Real `tmux` session: asserts the engine entered the alternate screen for the
/// capture and that the alt-screen snapshot carries a deterministic content
/// marker printed by the tmux pane. The capture uses a private tmux socket so
/// the user's default tmux server is never touched (see README and
/// `results/raw/streams/tmux.cleanup.txt`).
fn tmux_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().alt_screen {
        return cap_unsupported(
            env,
            "tmux_real",
            "Alternate screen + content (real tmux session)",
            "real-program",
            "alt_screen",
        );
    }
    let mut r = CheckResult::new(
        env,
        "tmux_real",
        "Alternate screen + content (real tmux session)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    const MARKER: &str = "RELAYABLE_TMUX_MARKER";
    let mut a = env.make_default();
    let replay = replay_with_alt_watch(a.as_mut(), &stream.bytes);
    let enters = replay.sequences.iter().filter(|s| s.enter).count();
    let leaves = replay.sequences.iter().filter(|s| !s.enter).count();
    r.assert_bool(
        "raw_bytes_1049h_present",
        enters > 0,
        "capture bytes contain at least one ESC[?1049h",
        &format!("1049h_count={enters}"),
    );
    r.assert_bool(
        "raw_bytes_1049l_present",
        leaves > 0,
        "capture bytes contain at least one ESC[?1049l",
        &format!("1049l_count={leaves}"),
    );
    let enter_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    let leave_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| !t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    r.assert_bool(
        "engine_alt_true_after_every_1049h",
        !enter_states.is_empty() && enter_states.iter().all(|v| *v),
        "engine alt_screen == true after every 1049h",
        &format!("engine_alt_after_each_1049h={enter_states:?}"),
    );
    r.assert_bool(
        "engine_alt_false_after_every_1049l",
        !leave_states.is_empty() && leave_states.iter().all(|v| !*v),
        "engine alt_screen == false after every 1049l",
        &format!("engine_alt_after_each_1049l={leave_states:?}"),
    );
    let marker_count = stream
        .bytes
        .windows(MARKER.len())
        .filter(|w| w == &MARKER.as_bytes())
        .count();
    let marker_offset = stream
        .bytes
        .windows(MARKER.len())
        .position(|w| w == MARKER.as_bytes());
    r.assert_bool(
        "raw_bytes_marker_present",
        marker_count > 0,
        &format!("capture bytes contain the deterministic tmux pane marker {MARKER:?}"),
        &format!("marker_count={marker_count} marker_offset={marker_offset:?}"),
    );
    // Snapshot the engine exactly where the pane marker was written. tmux
    // clears the screen (ESC[H ESC[2J) on detach before it leaves the alternate
    // screen, so the last snapshot before 1049l is blank; the marker is only
    // visible mid-replay while the engine is still on the alternate screen.
    let content_snapshot = marker_offset.map(|offset| {
        let mut c = env.make_default();
        c.feed(&stream.bytes[..offset + MARKER.len()]);
        c.snapshot()
    });
    let alt_active = content_snapshot.as_ref().is_some_and(|s| s.alt_screen);
    let alt_text = content_snapshot
        .as_ref()
        .map(Snapshot::to_plain)
        .unwrap_or_default();
    r.assert_bool(
        "engine_alt_screen_active_at_marker",
        alt_active,
        "engine alt_screen == true at the byte offset where the pane marker is written",
        &format!("alt_screen={alt_active} marker_offset={marker_offset:?}"),
    );
    r.assert_bool(
        "engine_alt_screen_contains_marker",
        alt_text.contains(MARKER),
        &format!(
            "engine alt-screen snapshot at the marker offset contains {MARKER:?} (while alt_screen == true)"
        ),
        &format!(
            "alt_nonblank={} present={:?} first_nonblank_rows={:?}",
            !alt_text.trim().is_empty(),
            matching_lines(&alt_text, MARKER),
            alt_text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(4)
                .collect::<Vec<_>>()
        ),
    );
    r.note("The marker is asserted on an engine snapshot taken at the marker byte offset (mid-replay, while the engine is on the alternate screen), because tmux clears the screen before detaching: the snapshot immediately before 1049l is blank even though the pane content was non-blank.");
    r.note(
        "Real tmux session captured through the PTY on the private socket `relayable-term002-spike` (capture command: `tmux -L relayable-term002-spike -f /dev/null new-session 'printf \"RELAYABLE_TMUX_MARKER\\n\"; sleep 4'`). The user's default tmux server is never referenced. scripts/run_all.sh kills the private server after the capture and records the cleanup command, its exit code, and the verification that no server remains in results/raw/streams/tmux.cleanup.txt.",
    );
    r.screen_dump = if alt_text.is_empty() {
        replay
            .last_alt_snapshot
            .as_ref()
            .unwrap_or(&replay.final_snapshot)
            .to_plain()
    } else {
        alt_text
    };
    r
}

fn title_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().title {
        return cap_unsupported(
            env,
            "osc_title_real",
            "OSC 0/2 title (real nvim '+set title')",
            "real-program",
            "title",
        );
    }
    let mut r = CheckResult::new(
        env,
        "osc_title_real",
        "OSC 0/2 title (real nvim '+set title')",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    let obs = watch(a.as_mut(), &stream.bytes, 512);
    let non_empty = obs.titles_seen.iter().find(|t| !t.is_empty()).cloned();
    r.assert_bool(
        "title_set_observed",
        non_empty.is_some(),
        "a non-empty engine title observed",
        &format!("{:?}", obs.titles_seen),
    );
    r.note(format!("titles observed: {:?}", obs.titles_seen));
    r.note(format!("engine title at end: {:?}", obs.title_final));
    r.screen_dump = obs.final_snapshot.to_plain();
    r
}

fn osc8_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().osc8_hyperlinks {
        return cap_unsupported(
            env,
            "osc8_real",
            "OSC 8 hyperlinks (real ls --hyperlink)",
            "real-program",
            "osc8_hyperlinks",
        );
    }
    let mut r = CheckResult::new(
        env,
        "osc8_real",
        "OSC 8 hyperlinks (real ls --hyperlink)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    a.feed(&stream.bytes);
    let s = a.snapshot();
    let mut uris: Vec<String> = s.cells.iter().filter_map(|c| c.hyperlink.clone()).collect();
    uris.sort();
    uris.dedup();
    r.assert_bool(
        "hyperlink_cells_present",
        s.count_hyperlink_cells() > 0,
        "> 0 hyperlink cells",
        &s.count_hyperlink_cells().to_string(),
    );
    r.assert_bool(
        "hyperlink_uris_are_file_urls",
        !uris.is_empty()
            && uris
                .iter()
                .all(|u| u.starts_with("file://") || u.starts_with("http")),
        "file:// or http(s) URIs",
        &format!("{uris:?}"),
    );
    r.note(format!("unique hyperlink URIs ({}): {uris:?}", uris.len()));
    r.screen_dump = s.to_plain();
    r
}

fn bell_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().bell {
        return cap_unsupported(
            env,
            "bell_real",
            "BEL (real sh printf \\a)",
            "real-program",
            "bell",
        );
    }
    let mut r = CheckResult::new(env, "bell_real", "BEL (real sh printf \\a)", "real-program");
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    let obs = watch(a.as_mut(), &stream.bytes, 64);
    r.assert_bool(
        "bell_count_ge_1",
        obs.bell_total >= 1,
        "bell_count >= 1",
        &obs.bell_total.to_string(),
    );
    r.assert_bool(
        "bell_event_present",
        obs.final_snapshot
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::Bell)),
        "Bell event present",
        &format!("{:?}", obs.final_snapshot.events),
    );
    r.screen_dump = obs.final_snapshot.to_plain();
    r
}

fn unicode_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().wide_glyphs {
        return cap_unsupported(
            env,
            "unicode_real",
            "Unicode text (real sh printf)",
            "real-program",
            "wide_glyphs",
        );
    }
    let mut r = CheckResult::new(
        env,
        "unicode_real",
        "Unicode text (real sh printf)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    a.feed(&stream.bytes);
    let s = a.snapshot();
    let plain = s.to_plain();
    for needle in ["寬字元", "日本語", "😀"] {
        r.assert_bool(
            &format!("plain_contains_{needle}"),
            plain.contains(needle),
            &format!("to_plain() contains {needle:?}"),
            &format!("plain={plain:?}"),
        );
    }
    r.screen_dump = plain;
    r
}

fn cursor_keys_real(env: &Env, stream: &Stream) -> CheckResult {
    if !env.capabilities().app_cursor_keys {
        return cap_unsupported(
            env,
            "cursor_keys_real",
            "Application cursor keys round-trip (real sh + injected arrow)",
            "real-program",
            "app_cursor_keys",
        );
    }
    let mut r = CheckResult::new(
        env,
        "cursor_keys_real",
        "Application cursor keys round-trip (real sh + injected arrow)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let raw_arrow = stream.bytes.windows(3).any(|w| w == b"\x1b[A");
    let echoed = stream.bytes.windows(4).any(|w| w == b"^[[A");
    // D6: the primary evidence is a real engine assertion, not a byte grep.
    // (a) Feed the whole committed capture into the engine at a known cursor
    //     position (CUP row 5 col 10) and assert the resulting cursor and mode.
    let mut a = env.make_default();
    a.feed(b"\x1b[5;10H");
    let cursor_at_cup = a.snapshot().cursor;
    a.feed(&stream.bytes);
    let after_capture = a.snapshot();
    r.assert(
        "engine_cursor_at_known_cup",
        Cursor {
            row: 4,
            col: 9,
            visible: true,
        },
        cursor_at_cup,
    );
    r.assert(
        "engine_cursor_after_capture",
        Cursor {
            row: 5,
            col: 0,
            visible: true,
        },
        after_capture.cursor,
    );
    r.assert_bool(
        "engine_app_cursor_keys_mode_after_capture",
        !after_capture.app_cursor_keys,
        "false (the capture never sends DECSET 1)",
        &after_capture.app_cursor_keys.to_string(),
    );
    // (b) The injected arrow bytes themselves, fed at another known position,
    //     must resolve to CUU: row 9 -> row 8 with the column unchanged.
    let mut b = env.make_default();
    b.feed(b"\x1b[10;5H");
    let before_arrow = b.snapshot().cursor;
    b.feed(b"\x1b[A");
    let after_arrow = b.snapshot().cursor;
    r.assert(
        "engine_cursor_before_arrow",
        Cursor {
            row: 9,
            col: 4,
            visible: true,
        },
        before_arrow,
    );
    r.assert(
        "engine_cursor_after_esc_A_is_cuu",
        Cursor {
            row: 8,
            col: 4,
            visible: true,
        },
        after_arrow,
    );
    // Secondary capture-bytes observation (kept as a note, per the brief).
    r.note(format!(
        "secondary capture-bytes observation: raw ESC[A present={raw_arrow}, line-discipline echo ^[[A present={echoed}, capture bytes={}",
        stream.bytes.len()
    ));
    if !raw_arrow && !echoed {
        r.note("The line discipline did not echo the injected arrow bytes; the app-cursor mode itself is covered by the fixture check cursor_keys.");
    }
    r.note(
        "Engine expectations were established by feeding this committed capture (results/raw/streams/cursor_keys.bin) into alacritty_terminal 0.26.0 and reading the resulting Cursor point; the arrow leg feeds the literal 3-byte ESC[A present in the capture at CUP(10,5).",
    );
    r.screen_dump = after_capture.to_plain();
    r
}

fn mouse_real(env: &Env, stream: &Stream) -> CheckResult {
    let caps = env.capabilities();
    if !caps.mouse_modes {
        return cap_unsupported(
            env,
            "mouse_modes_real",
            "Mouse modes (real nvim '+set mouse=a')",
            "real-program",
            "mouse_modes",
        );
    }
    if !caps.mouse_sgr_flag {
        return cap_unsupported(
            env,
            "mouse_modes_real",
            "Mouse modes (real nvim '+set mouse=a')",
            "real-program",
            "mouse_sgr_flag",
        );
    }
    let mut r = CheckResult::new(
        env,
        "mouse_modes_real",
        "Mouse modes (real nvim '+set mouse=a')",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make_default();
    let obs = watch(a.as_mut(), &stream.bytes, 512);
    r.assert_bool(
        "mouse_mode_enabled",
        obs.mouse_mode_max != MouseMode::Off,
        "mouse mode != Off",
        &format!("{:?}", obs.mouse_mode_max),
    );
    r.assert_bool(
        "mouse_sgr_enabled",
        obs.mouse_sgr_ever,
        "SGR mouse encoding enabled",
        &obs.mouse_sgr_ever.to_string(),
    );
    r.note(format!("max mouse mode observed: {:?}", obs.mouse_mode_max));
    r.screen_dump = obs.final_snapshot.to_plain();
    r
}

fn resize_real(env: &Env, stream: &Stream) -> CheckResult {
    let mut r = CheckResult::new(
        env,
        "resize_real",
        "PTY resize SIGWINCH round-trip (real sh + stty size)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let Some((_ms, cols, rows)) = stream.last_resize() else {
        r.not_tested(
            "sigwinch_propagated",
            "stream meta contains no resize=... line (capture was not run with --resize-ms)",
        );
        return r;
    };
    let needle = format!("{rows} {cols}");
    let text = String::from_utf8_lossy(&stream.bytes).to_string();
    r.assert_bool(
        "child_reported_new_size",
        text.contains(&needle),
        &format!("captured stream contains {needle:?} (stty size output)"),
        &format!("contains={}", text.contains(&needle)),
    );
    r.note(format!(
        "requested resize sequence: {:?}; pre-size={}x{}, post-size={}x{}",
        stream.resizes, env.cols, env.rows, cols, rows
    ));
    r.note(
        "real command used: /bin/sh -c 'trap \"echo WINCHED; stty size\" WINCH; echo READY; while :; do sleep 0.1; done'. The brief's `sleep 5` form is NOT usable on this host: /bin/sh is bash and a WINCH trap that arrives during a single long foreground `sleep` is not run until that sleep exits (verified: no trap output). The 0.1s sleep loop keeps the same trap/stty mechanism but makes the trap run promptly; the captured stream shows WINCHED and the new '100 30'.",
    );
    // Engine leg: resize the grid and confirm it matches the PTY's new size.
    let mut a = env.make(env.cols, env.rows);
    a.feed(&stream.bytes);
    a.resize(cols, rows);
    let s = a.snapshot();
    r.assert("engine_grid_cols_after_resize", cols, s.cols);
    r.assert("engine_grid_rows_after_resize", rows, s.rows);
    r.screen_dump = s.to_plain();
    r
}

fn scrollback_real(env: &Env, stream: &Stream, produced_lines: usize) -> CheckResult {
    if !env.capabilities().scrollback_bound {
        return cap_unsupported(
            env,
            "scrollback_bound_real",
            "Scrollback bound (real seq 1 20000)",
            "real-program",
            "scrollback_bound",
        );
    }
    let mut r = CheckResult::new(
        env,
        "scrollback_bound_real",
        "Scrollback bound (real seq 1 20000)",
        "real-program",
    );
    r.source(
        stream.path.display().to_string(),
        &stream.bytes,
        "real-program",
    );
    let mut a = env.make(env.cols, env.rows);
    a.feed(&stream.bytes);
    let s = a.snapshot();
    let plain = s.to_plain();
    // D7: `configured_bound` is informational only (it compared a harness
    // constant to the adapter echoing its own constructor argument).
    r.assert_bool(
        "retained_lines_le_configured_bound",
        s.scrollback.retained_lines <= s.scrollback.configured_bound,
        "retained_lines <= configured_bound",
        &format!(
            "retained_lines={} configured_bound={}",
            s.scrollback.retained_lines, s.scrollback.configured_bound
        ),
    );
    r.assert_bool(
        "retained_lines_bounded",
        s.scrollback.retained_lines <= env.scrollback,
        &format!("retained_lines <= {}", env.scrollback),
        &s.scrollback.retained_lines.to_string(),
    );
    r.assert_bool(
        "retained_lines_positive",
        s.scrollback.retained_lines > 0,
        "retained_lines > 0 under real overflow",
        &s.scrollback.retained_lines.to_string(),
    );
    let newest_needle = produced_lines.to_string();
    let newest_present = matching_lines(&plain, &newest_needle);
    r.assert_bool(
        "newest_present",
        plain.contains(&newest_needle),
        &format!("visible screen contains {newest_needle}"),
        &format!(
            "last_non_blank_visible_row={:?}; present={newest_present:?}",
            last_nonblank_visible_row(&s)
        ),
    );
    let oldest_matches: Vec<String> = plain
        .lines()
        .filter(|l| l.trim() == "1")
        .map(str::to_string)
        .collect();
    r.assert_bool(
        "oldest_gone",
        oldest_matches.is_empty(),
        "visible screen does not show the first line '1'",
        &format!(
            "present={oldest_matches:?} (lines whose trimmed text == \"1\"); first_visible_non_blank_row={:?}",
            (0..s.rows)
                .map(|row| (row, s.row_text(row)))
                .find(|(_, text)| !text.trim().is_empty())
        ),
    );
    r.note(format!(
        "lines produced by child = {produced_lines}; retained_lines = {} (configured_bound = {}, informational only)",
        s.scrollback.retained_lines, env.scrollback
    ));
    r.note(format!(
        "unique client lines in capture = {}",
        count_newlines(&stream.bytes)
    ));
    r.screen_dump = plain;
    r
}

fn count_newlines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

#[must_use]
pub fn count_stream_lines(stream: &Stream) -> usize {
    count_newlines(&stream.bytes)
}

// ---------------------------------------------------------------------------
// Orchestration
// ---------------------------------------------------------------------------

/// Run every check that can run for this engine. Missing real captures are
/// recorded as NOT_TESTED with the reason. Not-yet-wired engines short-circuit
/// to one `UNSUPPORTED` result per catalog entry.
#[must_use]
pub fn run_all(env: &Env) -> Vec<CheckResult> {
    let probe = env.make_default();
    if let ImplementationStatus::NotImplemented { engine, reason } = probe.implementation_status() {
        return not_implemented_results(env, engine, reason);
    }
    let mut out = vec![
        check_vt_basics(env),
        check_alt_screen_fixture(env),
        check_title_fixture(env),
        check_osc8_fixture(env),
        check_bell_fixture(env),
        check_unicode_wide(env),
        check_unicode_combining(env),
        check_unicode_emoji(env),
        check_colors_fixture(env),
        check_cursor_keys_fixture(env),
        check_bracketed_paste_mode(env),
        check_bracketed_paste_payload(env),
        check_mouse_fixture(env),
        check_resize_engine(env),
        check_scrollback_engine(env),
        check_selection(env),
        check_chunk_boundary_invariance(env),
    ];

    // Real-program legs (streams produced by the `capture` binary).
    out.push(real_or_missing(
        env,
        "nvim_alt",
        "alt_screen_real",
        alt_screen_real,
    ));
    out.push(real_or_missing(
        env,
        "nvim_title",
        "osc_title_real",
        title_real,
    ));
    out.push(real_or_missing(env, "osc8_ls", "osc8_real", osc8_real));
    out.push(real_or_missing(env, "bell", "bell_real", bell_real));
    out.push(real_or_missing(
        env,
        "unicode",
        "unicode_real",
        unicode_real,
    ));
    out.push(real_or_missing(
        env,
        "cursor_keys",
        "cursor_keys_real",
        cursor_keys_real,
    ));
    out.push(real_or_missing(
        env,
        "nvim_mouse",
        "mouse_modes_real",
        mouse_real,
    ));
    out.push(real_or_missing(
        env,
        "resize_winch",
        "resize_real",
        resize_real,
    ));
    let seq = env.stream("seq_20000");
    out.push(match seq {
        Some(stream) => scrollback_real(env, stream, count_stream_lines(stream)),
        None => missing_stream(
            env,
            "scrollback_bound_real",
            "Scrollback bound (real seq)",
            "seq_20000",
        ),
    });
    out.push(real_or_missing(env, "tmux", "tmux_real", tmux_real));

    // Secondary cross-check: phase-1 committed real captures.
    for name in ["less", "nvim", "nvim_mouse", "nvim_title", "osc8_ls", "top"] {
        out.push(replay_phase1(env, name));
    }
    out
}

/// True when an artifact `ACTUAL` value is a placeholder rather than evidence.
///
/// Rejects an empty value and any value whose first whitespace-delimited token
/// is on the denylist, so the single-token forms (`checked`, `todo`, `n/a`, `-`,
/// `placeholder`) and the multi-token form `checked (see dump)` are all caught.
/// Kept conservative so legitimate engine values are never rejected: the first
/// token must be exactly a denylisted word, so `true`, `false`, `Click`,
/// `Default`, `[]`, `Some("checked")` and `-1` all remain valid.
#[must_use]
pub fn is_placeholder_actual(value: &str) -> bool {
    const DENY: &[&str] = &["checked", "placeholder", "todo", "n/a", "-"];
    let v = value.trim();
    if v.is_empty() {
        return true;
    }
    let first = v.split_whitespace().next().unwrap_or(v);
    DENY.iter().any(|d| *d == first.to_ascii_lowercase())
}

/// One `UNSUPPORTED` result per catalog entry for a `NOT_IMPLEMENTED` engine.
fn not_implemented_results(env: &Env, engine: &str, reason: &str) -> Vec<CheckResult> {
    CHECK_CATALOG
        .iter()
        .map(|(id, title, kind)| {
            let mut r = CheckResult::new(env, id, title, kind);
            r.unsupported(
                "engine_adapter_implemented",
                &format!("{engine} adapter implements the neutral model"),
                reason,
            );
            r.note(format!(
                "{engine} adapter is a NOT_IMPLEMENTED stub; recorded UNSUPPORTED, never PASS/FAIL. Reason: {reason}"
            ));
            r
        })
        .collect()
}

fn real_or_missing(
    env: &Env,
    stream_name: &str,
    check: &str,
    run: fn(&Env, &Stream) -> CheckResult,
) -> CheckResult {
    match env.stream(stream_name) {
        Some(stream) => run(env, stream),
        None => missing_stream(
            env,
            check,
            &format!("real-program leg ({stream_name})"),
            stream_name,
        ),
    }
}

fn missing_stream(env: &Env, check: &str, title: &str, stream_name: &str) -> CheckResult {
    let mut r = CheckResult::new(env, check, title, "real-program");
    r.not_tested(
        "stream_available",
        &format!(
            "results/raw/streams/{stream_name}.bin missing (program absent or capture not run); see run_all_meta.txt"
        ),
    );
    r
}

/// Per-capture expectations for the phase-1 replay legs. The signature strings
/// and 1049 counts were established by feeding the committed
/// `spikes/terminal-engine/results/raw/<name>.bin` captures into
/// `alacritty_terminal 0.26.0` and reading the engine's own screen/title/mode
/// state (see the notes inside each generated artifact).
struct ReplayExpectation {
    phase1_1049h: usize,
    phase1_1049l: usize,
    /// Substring that must be present in the alt screen snapshotted just before
    /// the final 1049l (where a full-screen program's content lives).
    alt_signature: Option<&'static str>,
    /// Substring that must be present in the final (post-replay) screen.
    final_signature: Option<&'static str>,
    /// Mouse mode/SGR expectation, `None` when this capture's distinguishing
    /// feature is not mouse (mouse is then covered by `replay_nvim_mouse`).
    expect_mouse: Option<(MouseMode, bool)>,
    title_suffix: Option<&'static str>,
    hyperlink_cells: bool,
}

fn replay_expectation(name: &str) -> ReplayExpectation {
    let nvim = || ReplayExpectation {
        phase1_1049h: 1,
        phase1_1049l: 1,
        alt_signature: Some("NVIM v0.12.5"),
        final_signature: None,
        expect_mouse: None,
        title_suffix: None,
        hyperlink_cells: false,
    };
    match name {
        "less" => ReplayExpectation {
            phase1_1049h: 1,
            phase1_1049l: 1,
            alt_signature: Some("23"),
            final_signature: None,
            expect_mouse: None,
            title_suffix: None,
            hyperlink_cells: false,
        },
        "nvim" => nvim(),
        "nvim_mouse" => ReplayExpectation {
            expect_mouse: Some((MouseMode::Drag, true)),
            ..nvim()
        },
        "nvim_title" => ReplayExpectation {
            title_suffix: Some("- Nvim"),
            ..nvim()
        },
        "osc8_ls" => ReplayExpectation {
            phase1_1049h: 0,
            phase1_1049l: 0,
            alt_signature: None,
            final_signature: Some("alpha.txt"),
            expect_mouse: None,
            title_suffix: None,
            hyperlink_cells: true,
        },
        "top" => ReplayExpectation {
            phase1_1049h: 0,
            phase1_1049l: 0,
            alt_signature: None,
            final_signature: Some("Tasks:"),
            expect_mouse: None,
            title_suffix: None,
            hyperlink_cells: false,
        },
        other => panic!("no replay expectation defined for {other}"),
    }
}

fn replay_phase1(env: &Env, name: &str) -> CheckResult {
    let spec = replay_expectation(name);
    let rel = format!("spikes/terminal-engine/results/raw/{name}.bin");
    let path = env.repo_root.join(&rel);
    let mut r = CheckResult::new(
        env,
        &format!("replay_{name}"),
        "Secondary cross-check of phase-1 committed capture",
        "replayed-real-capture",
    );
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            r.not_tested(
                "phase1_capture_readable",
                &format!("{}: {e}", path.display()),
            );
            return r;
        }
    };
    r.source(rel.clone(), &bytes, "replayed-real-capture");
    let commit = std::process::Command::new("git")
        .current_dir(&env.repo_root)
        .args([
            "log",
            "-1",
            "--format=%H",
            "--",
            "spikes/terminal-engine/results/raw",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    r.note(format!(
        "phase-1 source commit (last change to spikes/terminal-engine/results/raw): {commit}"
    ));
    r.assert_bool(
        "capture_processed",
        !bytes.is_empty(),
        "non-empty capture processed",
        &format!("{} bytes", bytes.len()),
    );

    // Chunk-independent alt-screen evidence: raw-byte 1049 scan + engine flag at
    // each transition. `watch` is only used for accumulating mode/title state.
    let mut a = env.make_default();
    let replay = replay_with_alt_watch(a.as_mut(), &bytes);
    let actual_h = replay.sequences.iter().filter(|s| s.enter).count();
    let actual_l = replay.sequences.iter().filter(|s| !s.enter).count();
    r.assert("raw_bytes_1049h_count", spec.phase1_1049h, actual_h);
    r.assert("raw_bytes_1049l_count", spec.phase1_1049l, actual_l);
    let enter_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    let leave_states: Vec<bool> = replay
        .transitions
        .iter()
        .filter(|t| !t.enter)
        .map(|t| t.engine_alt_screen)
        .collect();
    if spec.phase1_1049h > 0 {
        if env.capabilities().alt_screen {
            r.assert_bool(
                "engine_alt_true_after_every_1049h",
                !enter_states.is_empty() && enter_states.iter().all(|v| *v),
                "engine alt_screen == true after every observed 1049h",
                &format!("engine_alt_after_each_1049h={enter_states:?}"),
            );
        } else {
            r.unsupported(
                "engine_alt_true_after_every_1049h",
                "engine alt_screen == true after every observed 1049h",
                &format!("{} declares alt_screen absent", env.engine),
            );
        }
    }
    if spec.phase1_1049l > 0 {
        if env.capabilities().alt_screen {
            r.assert_bool(
                "engine_alt_false_after_every_1049l",
                !leave_states.is_empty() && leave_states.iter().all(|v| !*v),
                "engine alt_screen == false after every observed 1049l",
                &format!("engine_alt_after_each_1049l={leave_states:?}"),
            );
        } else {
            r.unsupported(
                "engine_alt_false_after_every_1049l",
                "engine alt_screen == false after every observed 1049l",
                &format!("{} declares alt_screen absent", env.engine),
            );
        }
    }
    if let Some(sig) = spec.alt_signature {
        let text = replay
            .last_alt_snapshot
            .as_ref()
            .map(Snapshot::to_plain)
            .unwrap_or_default();
        r.assert_bool(
            "alt_screen_signature",
            text.contains(sig),
            &format!("alt screen snapshot before the final 1049l contains {sig:?}"),
            &format!(
                "alt_nonblank={} present={:?} first_nonblank_rows={:?}",
                !text.trim().is_empty(),
                matching_lines(&text, sig),
                text.lines()
                    .filter(|l| !l.trim().is_empty())
                    .take(4)
                    .collect::<Vec<_>>()
            ),
        );
    }
    if let Some(sig) = spec.final_signature {
        let text = replay.final_snapshot.to_plain();
        r.assert_bool(
            "final_screen_signature",
            text.contains(sig),
            &format!("final engine screen contains {sig:?}"),
            &format!(
                "present={:?} final_first_rows={:?}",
                matching_lines(&text, sig),
                text.lines()
                    .filter(|l| !l.trim().is_empty())
                    .take(4)
                    .collect::<Vec<_>>()
            ),
        );
    }

    // Mode/title accumulation over per-chunk engine snapshots. This is
    // chunk-sampled but only used for state that is held for a long stretch;
    // it is labelled as such in the artifact note.
    let mut w = env.make_default();
    let obs = watch(w.as_mut(), &bytes, 512);
    let caps = env.capabilities();
    if let Some((mode, sgr)) = spec.expect_mouse {
        if caps.mouse_modes {
            r.assert("mouse_mode_max_chunk_sampled", mode, obs.mouse_mode_max);
        } else {
            r.unsupported(
                "mouse_mode_max_chunk_sampled",
                &format!("{mode:?}"),
                &format!("{} declares mouse_modes absent", env.engine),
            );
        }
        if caps.mouse_sgr_flag {
            r.assert_bool(
                "mouse_sgr_ever_chunk_sampled",
                obs.mouse_sgr_ever == sgr,
                &sgr.to_string(),
                &obs.mouse_sgr_ever.to_string(),
            );
        } else {
            r.unsupported(
                "mouse_sgr_ever_chunk_sampled",
                &sgr.to_string(),
                &format!("{} declares mouse_sgr_flag absent", env.engine),
            );
        }
    }
    if let Some(suffix) = spec.title_suffix {
        if caps.title {
            let ok = obs
                .titles_seen
                .iter()
                .any(|t| !t.is_empty() && t.ends_with(suffix));
            r.assert_bool(
                "title_seen_ends_with",
                ok,
                &format!("a non-empty engine title ends with {suffix:?}"),
                &format!("{:?}", obs.titles_seen),
            );
        } else {
            r.unsupported(
                "title_seen_ends_with",
                &format!("a non-empty engine title ends with {suffix:?}"),
                &format!("{} declares title absent", env.engine),
            );
        }
    }
    if spec.hyperlink_cells {
        if caps.osc8_hyperlinks {
            let mut uris: Vec<String> = replay
                .final_snapshot
                .cells
                .iter()
                .filter_map(|c| c.hyperlink.clone())
                .collect();
            uris.sort();
            uris.dedup();
            r.assert_bool(
                "hyperlink_cells_present",
                !uris.is_empty() && uris.iter().all(|u| u.starts_with("file://")),
                "> 0 cells carry a file:// OSC 8 hyperlink",
                &format!("uri_count={} uris={uris:?}", uris.len()),
            );
        } else {
            r.unsupported(
                "hyperlink_cells_present",
                "> 0 cells carry a file:// OSC 8 hyperlink",
                &format!("{} declares osc8_hyperlinks absent", env.engine),
            );
        }
    }
    r.note(
        "Expectations were established by feeding the committed phase-1 capture into alacritty_terminal 0.26.0 and reading the engine's own screen/title/mode state; the 1049 counts match phase-1's CATEGORY_CHECKLIST (spikes/terminal-engine/results/raw/<name>.summary.txt).",
    );
    r.note(
        "Alt-screen transitions are chunk-independent (raw-byte scan + engine flag per transition). Mouse state is asserted only for the mouse-specific capture (replay_nvim_mouse) and is accumulated from per-chunk snapshots, which is labelled as chunk-sampled.",
    );
    r.screen_dump = replay
        .last_alt_snapshot
        .as_ref()
        .unwrap_or(&replay.final_snapshot)
        .to_plain();
    r
}

// ---------------------------------------------------------------------------
// Performance / memory observation (no thresholds, no fabrication)
// ---------------------------------------------------------------------------

/// One throughput pass.
#[derive(Debug, Clone)]
pub struct PerfRun {
    /// 1-based pass number.
    pub index: usize,
    pub elapsed_ms: f64,
    pub bytes_per_sec: f64,
    pub rss_before_kb: Option<u64>,
    pub rss_after_kb: Option<u64>,
    pub rss_delta_kb: Option<i64>,
    pub vmhwm_kb: Option<u64>,
}

/// A per-engine throughput/memory observation, measured in an isolated process
/// that runs exactly one engine (see `analyze --perf-only`). OBSERVATION ONLY:
/// no threshold is asserted.
#[derive(Debug, Clone)]
pub struct PerfRecord {
    pub engine: String,
    pub engine_version: String,
    pub description: String,
    pub bytes_fed: usize,
    pub lines_produced: usize,
    pub retained_lines: usize,
    pub configured_bound: usize,
    pub runs: Vec<PerfRun>,
    pub bytes_per_sec_min: Option<f64>,
    pub bytes_per_sec_median: Option<f64>,
    pub bytes_per_sec_max: Option<f64>,
    /// VmRSS before the first pass (this process's baseline).
    pub rss_baseline_kb: Option<u64>,
    /// VmRSS after the last pass.
    pub rss_after_kb: Option<u64>,
    pub rss_delta_kb: Option<i64>,
    /// This process's VmHWM after all passes (peak RSS).
    pub vmhwm_kb: Option<u64>,
}

fn read_status_kb(field: &str) -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            let v = rest.trim().trim_end_matches("kB").trim();
            return v.parse().ok();
        }
    }
    None
}

/// Read this process's current RSS and peak RSS in kB.
#[must_use]
pub fn rss_probe() -> (Option<u64>, Option<u64>) {
    (read_status_kb("VmRSS:"), read_status_kb("VmHWM:"))
}

fn median_of_sorted(sorted: &[f64]) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let n = sorted.len();
    Some(if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    })
}

fn perf_payload(target_bytes: usize) -> (Vec<u8>, usize) {
    let mut payload = Vec::with_capacity(target_bytes + 128);
    let mut lines = 0usize;
    while payload.len() < target_bytes {
        payload.extend_from_slice(
            format!("line {lines:08} abcdefghijklmnopqrstuvwxyz0123456789ABCD\r\n").as_bytes(),
        );
        lines += 1;
    }
    (payload, lines)
}

/// Feed a large sustained output `runs` times and record real per-pass
/// timings/RSS plus min/median/max throughput. OBSERVATION ONLY.
///
/// Call this from a process that constructs exactly one engine; see
/// `analyze --perf-only <engine>`, which `scripts/run_all.sh` invokes once per
/// engine so the RSS numbers are per-engine and not cross-contaminated.
#[must_use]
pub fn run_perf(env: &Env, target_bytes: usize, runs: usize) -> PerfRecord {
    let (payload, lines) = perf_payload(target_bytes);
    let mut a = env.make(env.cols, env.rows);
    let baseline = rss_probe().0;
    let mut pass_records = Vec::new();
    let mut rss_after = baseline;
    for index in 1..=runs.max(1) {
        let before = rss_probe().0;
        let start = Instant::now();
        for chunk in payload.chunks(65536) {
            a.feed(chunk);
        }
        let elapsed = start.elapsed();
        let (after, hwm) = rss_probe();
        rss_after = after;
        let secs = elapsed.as_secs_f64();
        pass_records.push(PerfRun {
            index,
            elapsed_ms: secs * 1000.0,
            bytes_per_sec: if secs > 0.0 {
                payload.len() as f64 / secs
            } else {
                0.0
            },
            rss_before_kb: before,
            rss_after_kb: after,
            rss_delta_kb: match (before, after) {
                (Some(b), Some(x)) => Some(x as i64 - b as i64),
                _ => None,
            },
            vmhwm_kb: hwm,
        });
    }
    let s = a.snapshot();
    let mut speeds: Vec<f64> = pass_records.iter().map(|r| r.bytes_per_sec).collect();
    speeds.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let vmhwm = pass_records.iter().filter_map(|r| r.vmhwm_kb).max();
    PerfRecord {
        engine: env.engine.to_string(),
        engine_version: env.version.to_string(),
        description: format!(
            "generated numbered lines, fed in 64 KiB chunks, cols={} rows={} scrolling_history={}, passes={}",
            env.cols,
            env.rows,
            env.scrollback,
            pass_records.len()
        ),
        bytes_fed: payload.len(),
        lines_produced: lines,
        retained_lines: s.scrollback.retained_lines,
        configured_bound: s.scrollback.configured_bound,
        runs: pass_records,
        bytes_per_sec_min: speeds.first().copied(),
        bytes_per_sec_median: median_of_sorted(&speeds),
        bytes_per_sec_max: speeds.last().copied(),
        rss_baseline_kb: baseline,
        rss_after_kb: rss_after,
        rss_delta_kb: match (baseline, rss_after) {
            (Some(b), Some(a)) => Some(a as i64 - b as i64),
            _ => None,
        },
        vmhwm_kb: vmhwm,
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use crate::engine;

    fn env_for<'a>(name: &'static str, factory: EngineFactory, fixtures: &'a [Fixture]) -> Env<'a> {
        Env {
            engine: name,
            version: engine::engine_version(name),
            factory,
            fixtures,
            streams: &[],
            cols: 80,
            rows: 24,
            scrollback: 1000,
            repo_root: std::env::temp_dir(),
        }
    }

    #[test]
    fn catalog_matches_implemented_run_all() {
        let fixtures = crate::fixtures::all();
        let env = env_for(
            engine::alacritty::NAME,
            engine::alacritty::factory,
            &fixtures,
        );
        let results = run_all(&env);
        assert_eq!(
            results.len(),
            CHECK_CATALOG.len(),
            "CHECK_CATALOG is out of sync with run_all"
        );
        for (result, (id, _, _)) in results.iter().zip(CHECK_CATALOG) {
            assert_eq!(result.check, *id);
        }
    }

    #[test]
    fn termwiz_stub_check_is_unsupported_never_pass_or_fail() {
        let fixtures = crate::fixtures::all();
        // `termwiz` is the only remaining NOT_IMPLEMENTED stub (documented
        // exclusion); `vt100` and `avt` are implemented comparison engines.
        let env = env_for(engine::termwiz::NAME, engine::termwiz::factory, &fixtures);
        let results = run_all(&env);
        assert_eq!(results.len(), CHECK_CATALOG.len());
        for result in &results {
            assert_eq!(
                result.status(),
                Status::Unsupported,
                "termwiz/{} should be UNSUPPORTED",
                result.check
            );
            assert!(!result.assertions.is_empty());
            assert!(result
                .assertions
                .iter()
                .all(|a| a.status == Status::Unsupported));
        }
    }

    #[test]
    fn comparison_engines_are_implemented_not_stub_short_circuited() {
        let fixtures = crate::fixtures::all();
        for name in [engine::vt100::NAME, engine::avt::NAME] {
            let (_, factory) = engine::engines()
                .into_iter()
                .find(|(n, _)| *n == name)
                .expect("registered engine");
            let env = env_for(name, factory, &fixtures);
            let results = run_all(&env);
            assert_eq!(results.len(), CHECK_CATALOG.len());
            for result in &results {
                let stub_short_circuit = result.assertions.len() == 1
                    && result.assertions[0].name == "engine_adapter_implemented";
                assert!(
                    !stub_short_circuit,
                    "{name}/{} was short-circuited as a stub",
                    result.check
                );
            }
        }
    }

    #[test]
    fn capability_gaps_are_unsupported_never_fail() {
        let fixtures = crate::fixtures::all();
        fn status_of(results: &[CheckResult], id: &str) -> Status {
            results
                .iter()
                .find(|r| r.check == id)
                .unwrap_or_else(|| panic!("missing check {id}"))
                .status()
        }

        // avt: no OSC 8, no combining chars, no mouse, no bracketed-paste mode.
        let env = env_for(engine::avt::NAME, engine::avt::factory, &fixtures);
        let avt = run_all(&env);
        for missing in [
            "osc8_hyperlink",
            "unicode_combining",
            "mouse_modes",
            "bracketed_paste_mode",
        ] {
            assert_eq!(
                status_of(&avt, missing),
                Status::Unsupported,
                "avt/{missing}"
            );
        }
        // A feature avt does provide is still asserted for real.
        assert_eq!(status_of(&avt, "vt_basics"), Status::Pass);
        // The payload leg is UNSUPPORTED for every engine by design.
        assert_eq!(
            status_of(&avt, "bracketed_paste_payload"),
            Status::Unsupported
        );

        // vt100: OSC 8 absent, resize does not reflow; 2004 mode is present.
        let env = env_for(engine::vt100::NAME, engine::vt100::factory, &fixtures);
        let vt100 = run_all(&env);
        assert_eq!(status_of(&vt100, "osc8_hyperlink"), Status::Unsupported);
        assert_eq!(status_of(&vt100, "resize_reflow"), Status::Unsupported);
        assert_eq!(status_of(&vt100, "bracketed_paste_mode"), Status::Pass);
        assert_eq!(status_of(&vt100, "vt_basics"), Status::Pass);
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::engine;
    use crate::model::Cell;

    fn env_for<'a>(name: &'static str, factory: EngineFactory, fixtures: &'a [Fixture]) -> Env<'a> {
        Env {
            engine: name,
            version: engine::engine_version(name),
            factory,
            fixtures,
            streams: &[],
            cols: 80,
            rows: 24,
            scrollback: 1000,
            repo_root: std::env::temp_dir(),
        }
    }

    #[test]
    fn engine_inputs_are_persisted_with_recomputable_sha256() {
        let dir = std::env::temp_dir().join(format!("relayable-ei-{}", std::process::id()));
        write_engine_inputs(&dir).expect("write engine inputs");
        for input in engine_inputs() {
            let bin = std::fs::read(dir.join(format!("{}.bin", input.name))).expect("bin");
            assert_eq!(bin, input.bytes, "{} payload", input.name);
            let meta =
                std::fs::read_to_string(dir.join(format!("{}.meta", input.name))).expect("meta");
            assert!(meta.contains("kind=engine-input"), "{} meta", input.name);
            assert!(
                meta.contains(&format!("sha256={}", sha256_hex(&input.bytes))),
                "{} sha256",
                input.name
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_alt_1049_is_chunk_independent_and_ordered() {
        let bytes = b"x\x1b[?1049habc\x1b[?1049ly";
        let seq = scan_alt_1049(bytes);
        assert_eq!(seq.len(), 2);
        assert!(seq[0].enter);
        assert_eq!(seq[0].start, 1);
        assert!(!seq[1].enter);
        assert_eq!(seq[1].start, 1 + 8 + 3);
        assert!(scan_alt_1049(b"no alt here").is_empty());
    }

    #[test]
    fn replay_with_alt_watch_is_independent_of_chunk_size() {
        // The 234-byte phase-1 `less` capture is smaller than the old 512-byte
        // chunk, which is why `watch` missed its transient alt screen.
        let payload = b"\x1b[?1049hhello\x1b[?1049lbye";
        let mut a = crate::engine::alacritty::AlacrittyAdapter::new(20, 3, 10);
        let replay = replay_with_alt_watch(&mut a, payload);
        assert_eq!(replay.transitions.len(), 2);
        assert!(replay.transitions[0].engine_alt_screen);
        assert!(!replay.transitions[1].engine_alt_screen);
        let alt = replay.last_alt_snapshot.expect("alt snapshot");
        assert!(alt.to_plain().contains("hello"));

        // `watch` with a chunk larger than the capture cannot see the transition.
        let mut b = crate::engine::alacritty::AlacrittyAdapter::new(20, 3, 10);
        let obs = watch(&mut b, payload, 512);
        assert!(
            !obs.alt_in_chunk_snapshots,
            "the chunk-sampled field is expected to miss a sub-chunk transition"
        );
    }

    #[test]
    fn snapshot_invariance_detects_style_and_hyperlink_differences() {
        let a = Snapshot {
            cols: 1,
            rows: 1,
            cells: vec![Cell::blank()],
            cursor: Cursor {
                row: 0,
                col: 0,
                visible: true,
            },
            alt_screen: false,
            bracketed_paste: false,
            app_cursor_keys: false,
            mouse_mode: MouseMode::Off,
            mouse_sgr: false,
            title: None,
            events: Vec::new(),
            bell_count: 0,
            scrollback: crate::model::ScrollbackStats {
                retained_lines: 0,
                configured_bound: 0,
            },
        };
        let mut b = a.clone();
        assert!(snapshot_invariance(&a, &b).0);
        b.cells[0].style.bold = true;
        let (same, detail) = snapshot_invariance(&a, &b);
        assert!(!same, "bold-only difference must be detected");
        assert!(detail.contains("differing_cells=1"), "detail={detail}");
        let mut c = a.clone();
        c.cells[0].hyperlink = Some("https://example.com".to_string());
        assert!(!snapshot_invariance(&a, &c).0);
    }

    #[test]
    fn replay_expectations_cover_every_replay_name_in_run_all() {
        for name in ["less", "nvim", "nvim_mouse", "nvim_title", "osc8_ls", "top"] {
            let _ = replay_expectation(name);
        }
    }

    /// An adapter that ignores every `feed` must FAIL
    /// `chunk_boundary_invariance`: its snapshots never move away from
    /// pristine, so the changed-vs-pristine falsifiability term fails.
    struct InertAdapter {
        cols: u16,
        rows: u16,
        scrollback: usize,
    }

    impl EngineAdapter for InertAdapter {
        fn name(&self) -> &'static str {
            "inert"
        }
        fn feed(&mut self, _bytes: &[u8]) {}
        fn resize(&mut self, cols: u16, rows: u16) {
            self.cols = cols;
            self.rows = rows;
        }
        fn snapshot(&self) -> Snapshot {
            crate::engine::empty_snapshot(self.cols, self.rows, self.scrollback)
        }
    }

    fn inert_factory(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter> {
        Box::new(InertAdapter {
            cols,
            rows,
            scrollback,
        })
    }

    #[test]
    fn input_ignoring_engine_fails_chunk_boundary_invariance() {
        let fixtures = crate::fixtures::all();
        let env = env_for("inert", inert_factory, &fixtures);
        let result = check_chunk_boundary_invariance(&env);
        assert_eq!(
            result.status(),
            Status::Fail,
            "an engine that ignores input must FAIL the falsifiability term; assertions: {:?}",
            result.assertions
        );
        assert!(
            result
                .assertions
                .iter()
                .any(|a| a.name == "fixture_vt_basics_changed_vs_pristine"
                    && a.status == Status::Fail),
            "vt_basics must be reported as not changing the screen"
        );
    }

    #[test]
    fn placeholder_actual_rule_is_conservative() {
        for bad in [
            "",
            "   ",
            "checked",
            "TODO",
            "n/a",
            "-",
            "placeholder",
            "checked (see dump)",
            "todo - fill in",
        ] {
            assert!(is_placeholder_actual(bad), "{bad:?} must be rejected");
        }
        for good in [
            "true",
            "false",
            "Click",
            "Default",
            "[]",
            "Some(\"x\")",
            "Some(\"checked\")",
            "[true, false]",
            "0",
            "-1",
        ] {
            assert!(!is_placeholder_actual(good), "{good:?} must be accepted");
        }
    }

    #[test]
    fn generated_assertions_never_use_placeholder_actuals() {
        let fixtures = crate::fixtures::all();
        for (name, factory) in engine::engines() {
            let env = env_for(name, factory, &fixtures);
            for result in run_all(&env) {
                for assertion in &result.assertions {
                    let value = assertion.actual.trim();
                    assert!(
                        !is_placeholder_actual(value),
                        "{}/{} assertion {} has placeholder ACTUAL {value:?}",
                        result.engine,
                        result.check,
                        assertion.name
                    );
                }
            }
        }
    }
}
