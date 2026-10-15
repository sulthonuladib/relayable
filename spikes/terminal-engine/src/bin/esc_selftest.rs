//! `esc_selftest` — proves the sequence detector is not silently broken.
//!
//! Real captures report `NOT_OBSERVED` for some categories. That claim is only
//! meaningful if the detector provably recognises those sequences when they are
//! present. This binary feeds synthetic fixtures for every required category
//! through the exact same `esc::scan` used on real PTY captures.
//!
//! Exit 0 iff every fixture is classified as expected and no fixture produces a
//! spurious required-category hit.

use relayable_terminal_engine_spike::esc;

fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    let mut v: Vec<(&'static str, Vec<u8>)> = Vec::new();
    let mut add = |cat: &'static str, bytes: &[u8]| v.push((cat, bytes.to_vec()));
    add("alt_screen_1049_enter", b"\x1b[?1049h");
    add("alt_screen_1049_leave", b"\x1b[?1049l");
    add("alt_screen_47_enter", b"\x1b[?47h");
    add("alt_screen_47_leave", b"\x1b[?47l");
    add("alt_screen_1047_enter", b"\x1b[?1047h");
    add("alt_screen_1047_leave", b"\x1b[?1047l");
    add("mouse_1000_enable", b"\x1b[?1000h");
    add("mouse_1000_disable", b"\x1b[?1000l");
    add("mouse_1002_enable", b"\x1b[?1002h");
    add("mouse_1002_disable", b"\x1b[?1002l");
    add("mouse_1003_enable", b"\x1b[?1003h");
    add("mouse_1003_disable", b"\x1b[?1003l");
    add("mouse_1005_enable", b"\x1b[?1005h");
    add("mouse_1005_disable", b"\x1b[?1005l");
    add("mouse_1006_enable", b"\x1b[?1006h");
    add("mouse_1006_disable", b"\x1b[?1006l");
    add("mouse_1015_enable", b"\x1b[?1015h");
    add("mouse_1015_disable", b"\x1b[?1015l");
    add("bracketed_paste_enable", b"\x1b[?2004h");
    add("bracketed_paste_disable", b"\x1b[?2004l");
    add("osc_title", b"\x1b]0;hello\x07");
    add("osc_title", b"\x1b]2;hello\x1b\\");
    add("osc_hyperlink", b"\x1b]8;;https://example.com\x1b\\text");
    add("osc_color", b"\x1b]11;?\x07");
    add("cursor_hide", b"\x1b[?25l");
    add("cursor_show", b"\x1b[?25h");
    add("decscusr", b"\x1b[1 q");
    add("sgr", b"\x1b[31m");
    add("sgr", b"\x1b[0;1;38;5;196m");
    v
}

fn main() {
    let mut failures = 0usize;
    println!("== esc_selftest: detector fixtures ==");
    println!("SCANNER=relayable_terminal_engine_spike::esc::scan");

    for (expected, bytes) in fixtures() {
        let seqs = esc::scan(&bytes);
        let cats: Vec<&str> = seqs.iter().map(|s| s.category).collect();
        let hit = cats.contains(&expected);
        let extra: Vec<&str> = cats.iter().copied().filter(|c| *c != expected).collect();
        if hit && extra.is_empty() {
            println!(
                "SELFTEST {expected} = PASS bytes=[{}] offset={:?}",
                esc::hex(&bytes),
                seqs.first().map(|s| s.offset)
            );
        } else {
            failures += 1;
            println!(
                "SELFTEST {expected} = FAIL bytes=[{}] got={:?}",
                esc::hex(&bytes),
                cats
            );
        }
    }

    // Combined private-mode list: `?1002;1006h` must yield BOTH categories.
    let combined = b"\x1b[?1002;1006h".to_vec();
    let cats: Vec<&str> = esc::scan(&combined).iter().map(|s| s.category).collect();
    let both = cats.contains(&"mouse_1002_enable") && cats.contains(&"mouse_1006_enable");
    if both {
        println!("SELFTEST combined_1002_1006 = PASS got={cats:?}");
    } else {
        failures += 1;
        println!("SELFTEST combined_1002_1006 = FAIL got={cats:?}");
    }

    // Negative control: plain text and an unrelated CSI must not create any
    // required-category hit.
    let required = [
        "alt_screen_1049_enter",
        "alt_screen_1049_leave",
        "mouse_1000_enable",
        "mouse_1002_enable",
        "mouse_1006_enable",
        "bracketed_paste_enable",
        "osc_title",
        "osc_hyperlink",
        "cursor_hide",
        "cursor_show",
        "decscusr",
        "sgr",
    ];
    let negative = b"plain text \x1b[2J no required sequence here".to_vec();
    let neg_cats: Vec<&str> = esc::scan(&negative).iter().map(|s| s.category).collect();
    let spurious: Vec<&&str> = required.iter().filter(|c| neg_cats.contains(*c)).collect();
    if spurious.is_empty() {
        println!("SELFTEST negative_control = PASS got={neg_cats:?}");
    } else {
        failures += 1;
        println!("SELFTEST negative_control = FAIL spurious={spurious:?} got={neg_cats:?}");
    }

    println!("SELFTEST_FAILURES={failures}");
    println!("VERDICT={}", if failures == 0 { "PASS" } else { "FAIL" });
    if failures != 0 {
        std::process::exit(1);
    }
}
