//! Deterministic VT/ANSI byte fixtures.
//!
//! Every sequence below is a real ECMA-48 / VT control sequence:
//! * CSI `ESC [ params final` — CUP (`H`), SGR (`m`), EL (`K`), ED (`J`),
//!   DECSET/DECRST private modes (`?n h/l`);
//! * ESC `7` / ESC `8` — DECSC / DECRC (save/restore cursor);
//! * OSC `ESC ] n ; text BEL` — operating-system commands (title, hyperlink);
//! * BEL (`0x07`), and raw UTF-8 text for the Unicode cases.
//!
//! Nothing here is invented: the byte strings were cross-checked against the
//! phase-1 decoded real captures (`spikes/terminal-engine/results/raw/*.seq`)
//! and against the `vte 0.15.0` / `alacritty_terminal 0.26.0` sources named in
//! `notes/engine-api.md`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::sha256::sha256_hex;

/// One named fixture.
pub struct Fixture {
    pub name: &'static str,
    pub description: &'static str,
    pub bytes: Vec<u8>,
}

fn entry(name: &'static str, description: &'static str, bytes: &[u8]) -> Fixture {
    Fixture {
        name,
        description,
        bytes: bytes.to_vec(),
    }
}

/// All fixtures, in a stable order.
#[must_use]
pub fn all() -> Vec<Fixture> {
    vec![
        entry(
            "vt_basics",
            "CUP, SGR bold/italic/underline/reverse, EL/ED erase, DECSC/DECRC save+restore",
            b"AB\x1b[2;5HC\x1b[1mD\x1b[0m\x1b[3mE\x1b[0m\x1b[4mF\x1b[0m\x1b[7mG\x1b[0m\
              \x1b[4;1HKLMNO\x1b[4;3H\x1b[K\x1b[5;1HPQRST\x1b[5;1H\x1b[2K\
              \x1b[6;3H\x1b7\x1b[8;10HZ\x1b8z",
        ),
        entry(
            "alt_enter",
            "DECSET 1049 enter alternate screen, then write 'alt'",
            b"top\x1b[?1049halt",
        ),
        entry(
            "alt_leave",
            "DECRST 1049 leave alternate screen, then write 'back'",
            b"\x1b[?1049lback",
        ),
        entry(
            "title_0",
            "OSC 0 window/icon title 'Fixture Title'",
            b"\x1b]0;Fixture Title\x07",
        ),
        entry(
            "title_2",
            "OSC 2 window title 'Second Title'",
            b"\x1b]2;Second Title\x07",
        ),
        entry(
            "title_reset",
            "OSC 0 with empty payload (title reset/clear)",
            b"\x1b]0;\x07",
        ),
        entry(
            "osc8",
            "OSC 8 hyperlink without id to example.com, then with id=x to example.org",
            b"\x1b]8;;https://example.com/a\x07LINK\x1b]8;;\x07\
              \x1b]8;id=x;https://example.org/b\x07X\x1b]8;;\x07",
        ),
        entry("bell", "BEL between two printable chars", b"A\x07B"),
        entry(
            "unicode_wide",
            "CJK wide glyphs (3 chars, 6 columns)",
            "寬字元".as_bytes(),
        ),
        entry(
            "unicode_combining",
            "combining acute U+0301 over 'e', then 'x'",
            "e\u{0301}x".as_bytes(),
        ),
        entry(
            "unicode_emoji_base",
            "single emoji U+1F600 (wide)",
            "😀".as_bytes(),
        ),
        entry(
            "unicode_vs16",
            "U+2764 + U+FE0F (VS16 emoji-presentation variation selector)",
            "\u{2764}\u{FE0F}".as_bytes(),
        ),
        entry(
            "unicode_zwj",
            "ZWJ family sequence U+1F468 ZWJ U+1F469 ZWJ U+1F467",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}".as_bytes(),
        ),
        entry(
            "colors",
            "SGR named (31,91), 256-index (38;5;196, 48;5;21) and truecolor (38;2, 48;2)",
            b"\x1b[31mr\x1b[0m\x1b[91mR\x1b[0m\x1b[38;5;196mI\x1b[0m\x1b[48;5;21mb\x1b[0m\
              \x1b[38;2;12;34;56mT\x1b[0m\x1b[48;2;210;211;212mU\x1b[0m",
        ),
        entry(
            "cursor_keys_on",
            "DECSET 1 application cursor keys on",
            b"\x1b[?1h",
        ),
        entry(
            "cursor_keys_off",
            "DECRST 1 application cursor keys off",
            b"\x1b[?1l",
        ),
        entry(
            "mouse_1000",
            "DECSET 1000 (click) mouse reporting",
            b"\x1b[?1000h",
        ),
        entry(
            "mouse_1002",
            "DECSET 1002 (drag) mouse reporting",
            b"\x1b[?1002h",
        ),
        entry(
            "mouse_1003",
            "DECSET 1003 (any motion) mouse reporting",
            b"\x1b[?1003h",
        ),
        entry(
            "mouse_1006",
            "DECSET 1006 (SGR) mouse encoding",
            b"\x1b[?1006h",
        ),
        entry(
            "bracketed_paste_on",
            "DECSET 2004 bracketed paste on",
            b"\x1b[?2004h",
        ),
        entry(
            "bracketed_paste_payload",
            "ESC[200~ pasted payload containing ESC and 0x03, then ESC[201~ and 'Z'",
            b"\x1b[200~a\x1b[31mb\x03c\x1b[201~Z",
        ),
        entry(
            "bracketed_paste_off",
            "DECRST 2004 bracketed paste off",
            b"\x1b[?2004l",
        ),
    ]
}

/// Look up a fixture by name.
#[must_use]
pub fn get(name: &str) -> Option<Fixture> {
    all().into_iter().find(|f| f.name == name)
}

/// Write every fixture to `dir` as `<name>.bin` plus `<name>.meta`.
///
/// Returns the list of `.bin` paths written.
pub fn write_all(dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    for fixture in all() {
        let bin = dir.join(format!("{}.bin", fixture.name));
        let meta = dir.join(format!("{}.meta", fixture.name));
        fs::write(&bin, &fixture.bytes)?;
        let text = format!(
            "kind=fixture\nname={}\ndescription={}\nbytes={}\nsha256={}\n",
            fixture.name,
            fixture.description,
            fixture.bytes.len(),
            sha256_hex(&fixture.bytes)
        );
        fs::write(&meta, text)?;
        written.push(bin);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_nonempty() {
        let all = all();
        let mut names: Vec<_> = all.iter().map(|f| f.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate fixture names");
        assert!(all.iter().all(|f| !f.bytes.is_empty()));
    }

    #[test]
    fn write_all_produces_bin_and_meta() {
        let dir = std::env::temp_dir().join(format!("relayable-fx-{}", std::process::id()));
        let written = write_all(&dir).expect("write fixtures");
        assert_eq!(written.len(), all().len());
        for f in all() {
            let bin = fs::read(dir.join(format!("{}.bin", f.name))).expect("bin");
            assert_eq!(bin, f.bytes);
            let meta = fs::read_to_string(dir.join(format!("{}.meta", f.name))).expect("meta");
            assert!(meta.contains("kind=fixture"));
            assert!(meta.contains(&format!("sha256={}", sha256_hex(&f.bytes))));
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
