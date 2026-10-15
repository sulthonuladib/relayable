//! Escape-sequence decoding/scanning for raw PTY byte streams.
//!
//! The scanner is a deliberately small CSI/OSC/DCS recogniser: it exists to
//! answer "did this real program actually emit X, and at what byte offset" —
//! not to be a terminal emulator. Anything it does not recognise is reported as
//! `csi_other` / `esc_other`.

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Seq {
    pub offset: usize,
    pub len: usize,
    pub category: &'static str,
    pub detail: String,
}

/// `^[`-style caret escaping for raw control bytes, with wrapping. Non-ASCII
/// bytes are rendered as `\xHH`. The accompanying `.bin` file holds the exact
/// bytes; this is a human-readable companion.
pub fn caret(bytes: &[u8]) -> String {
    let mut s = String::new();
    let mut col = 0usize;
    for &b in bytes {
        let piece: String = match b {
            0x1b => "^[".to_string(),
            0x0a => "^J".to_string(),
            0x0d => "^M".to_string(),
            0x09 => "^I".to_string(),
            0x07 => "^G".to_string(),
            b if b < 0x20 => format!("^{}", (b + 0x40) as char),
            0x7f => "^?".to_string(),
            b if b < 0x7f => (b as char).to_string(),
            b => format!("\\x{:02x}", b),
        };
        col += piece.len();
        s.push_str(&piece);
        if col >= 200 {
            s.push('\n');
            col = 0;
        }
    }
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn printable(bytes: &[u8], max: usize) -> String {
    let mut s = String::new();
    for &b in bytes.iter().take(max) {
        if (0x20..0x7f).contains(&b) {
            s.push(b as char);
        } else {
            s.push_str(&format!("\\x{:02x}", b));
        }
    }
    if bytes.len() > max {
        s.push_str("...");
    }
    s
}

fn raw_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) {
                b as char
            } else {
                '?'
            }
        })
        .collect()
}

/// Returns (private prefix char, parsed numeric parameter list).
fn csi_params(params: &[u8]) -> (Option<u8>, Vec<u32>) {
    let (prefix, rest) = match params.first() {
        Some(&p) if p == b'?' || p == b'>' || p == b'=' || p == b'<' => (Some(p), &params[1..]),
        _ => (None, params),
    };
    let nums: Vec<u32> = rest
        .split(|&b| b == b';')
        .map(|chunk| {
            let digits: Vec<u8> = chunk
                .iter()
                .copied()
                .filter(|b| b.is_ascii_digit())
                .collect();
            if digits.is_empty() {
                0
            } else {
                std::str::from_utf8(&digits)
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(u32::MAX)
            }
        })
        .collect();
    (prefix, nums)
}

fn classify_csi(
    offset: usize,
    raw: &[u8],
    params: &[u8],
    intermediates: &[u8],
    final_byte: u8,
    out: &mut Vec<Seq>,
) {
    let (priv_prefix, nums) = csi_params(params);
    let prefix_text = match priv_prefix {
        Some(p) => (p as char).to_string(),
        None => String::new(),
    };
    let detail = format!(
        "CSI prefix={:?} params={} inters={:?} final={:?}",
        prefix_text,
        raw_ascii(params),
        raw_ascii(intermediates),
        final_byte as char
    );

    if (final_byte == b'h' || final_byte == b'l') && priv_prefix == Some(b'?') {
        let set = final_byte == b'h';
        for p in &nums {
            let category: &'static str = match *p {
                1049 => {
                    if set {
                        "alt_screen_1049_enter"
                    } else {
                        "alt_screen_1049_leave"
                    }
                }
                1047 => {
                    if set {
                        "alt_screen_1047_enter"
                    } else {
                        "alt_screen_1047_leave"
                    }
                }
                47 => {
                    if set {
                        "alt_screen_47_enter"
                    } else {
                        "alt_screen_47_leave"
                    }
                }
                1000 => {
                    if set {
                        "mouse_1000_enable"
                    } else {
                        "mouse_1000_disable"
                    }
                }
                1002 => {
                    if set {
                        "mouse_1002_enable"
                    } else {
                        "mouse_1002_disable"
                    }
                }
                1003 => {
                    if set {
                        "mouse_1003_enable"
                    } else {
                        "mouse_1003_disable"
                    }
                }
                1005 => {
                    if set {
                        "mouse_1005_enable"
                    } else {
                        "mouse_1005_disable"
                    }
                }
                1006 => {
                    if set {
                        "mouse_1006_enable"
                    } else {
                        "mouse_1006_disable"
                    }
                }
                1015 => {
                    if set {
                        "mouse_1015_enable"
                    } else {
                        "mouse_1015_disable"
                    }
                }
                2004 => {
                    if set {
                        "bracketed_paste_enable"
                    } else {
                        "bracketed_paste_disable"
                    }
                }
                25 => {
                    if set {
                        "cursor_show"
                    } else {
                        "cursor_hide"
                    }
                }
                1 => {
                    if set {
                        "cursor_keys_application"
                    } else {
                        "cursor_keys_normal"
                    }
                }
                1004 => {
                    if set {
                        "focus_events_enable"
                    } else {
                        "focus_events_disable"
                    }
                }
                1048 => "save_restore_cursor",
                _ => "private_mode_other",
            };
            out.push(Seq {
                offset,
                len: raw.len(),
                category,
                detail: format!("{} [mode {}]", detail, p),
            });
        }
        return;
    }

    if final_byte == b'q' && intermediates == b" " {
        out.push(Seq {
            offset,
            len: raw.len(),
            category: "decscusr",
            detail: format!("{} [DECSCUSR cursor style]", detail),
        });
        return;
    }

    if final_byte == b'm' {
        out.push(Seq {
            offset,
            len: raw.len(),
            category: "sgr",
            detail,
        });
        return;
    }

    let category: &'static str = match final_byte {
        b'H' | b'f' | b'A' | b'B' | b'C' | b'D' | b'G' | b'd' | b'J' | b'K' | b'r' | b's'
        | b'u' => "csi_cursor_or_erase",
        _ => "csi_other",
    };
    out.push(Seq {
        offset,
        len: raw.len(),
        category,
        detail,
    });
}

fn classify_osc(offset: usize, raw: &[u8], content: &[u8], out: &mut Vec<Seq>) {
    let head = content
        .iter()
        .take_while(|&&b| b != b';')
        .copied()
        .collect::<Vec<u8>>();
    let head_str = raw_ascii(&head);
    let category: &'static str = match head_str.as_str() {
        "0" | "1" | "2" => "osc_title",
        "8" => "osc_hyperlink",
        "4" | "10" | "11" | "12" => "osc_color",
        "9" | "777" => "osc_notify",
        _ => "osc_other",
    };
    out.push(Seq {
        offset,
        len: raw.len(),
        category,
        detail: format!(
            "OSC {} content=\"{}\" raw={}",
            if head_str.is_empty() {
                "<empty>"
            } else {
                &head_str
            },
            printable(content, 96),
            hex(&raw[..raw.len().min(48)])
        ),
    });
}

/// Scan a raw child stream for every ESC-introduced sequence and classify it.
pub fn scan(bytes: &[u8]) -> Vec<Seq> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        if i + 1 >= bytes.len() {
            out.push(Seq {
                offset: i,
                len: 1,
                category: "esc_other",
                detail: "trailing ESC".to_string(),
            });
            break;
        }
        match bytes[i + 1] {
            b'[' => {
                let mut j = i + 2;
                let pstart = j;
                while j < bytes.len() && (0x30..=0x3f).contains(&bytes[j]) {
                    j += 1;
                }
                let params_end = j;
                while j < bytes.len() && (0x20..=0x2f).contains(&bytes[j]) {
                    j += 1;
                }
                let inters_end = j;
                if j < bytes.len() {
                    let end = j + 1;
                    classify_csi(
                        i,
                        &bytes[i..end],
                        &bytes[pstart..params_end],
                        &bytes[params_end..inters_end],
                        bytes[j],
                        &mut out,
                    );
                    i = end;
                } else {
                    i = bytes.len();
                }
            }
            b']' => {
                let mut j = i + 2;
                let mut term_len = 0usize;
                while j < bytes.len() {
                    if bytes[j] == 0x07 {
                        term_len = 1;
                        break;
                    }
                    if bytes[j] == 0x1b && j + 1 < bytes.len() && bytes[j + 1] == b'\\' {
                        term_len = 2;
                        break;
                    }
                    j += 1;
                }
                let content_end = if term_len == 0 { bytes.len() } else { j };
                let end = if term_len == 0 {
                    bytes.len()
                } else {
                    j + term_len
                };
                let content_start = (i + 2).min(content_end);
                classify_osc(
                    i,
                    &bytes[i..end],
                    &bytes[content_start..content_end],
                    &mut out,
                );
                i = end;
            }
            b'P' | b'^' | b'_' => {
                let mut j = i + 2;
                while j < bytes.len() {
                    if bytes[j] == 0x1b && j + 1 < bytes.len() && bytes[j + 1] == b'\\' {
                        j += 2;
                        break;
                    }
                    j += 1;
                }
                let end = j.min(bytes.len());
                out.push(Seq {
                    offset: i,
                    len: end - i,
                    category: "dcs_other",
                    detail: format!("DCS/PM/APC {:?}", bytes[i + 1] as char),
                });
                i = end;
            }
            _ => {
                let end = (i + 2).min(bytes.len());
                out.push(Seq {
                    offset: i,
                    len: end - i,
                    category: "esc_other",
                    detail: format!("ESC {:?} {}", bytes[i + 1] as char, hex(&bytes[i..end])),
                });
                i = end;
            }
        }
    }
    out
}

/// category -> (count, first_offset, first_len, first_detail)
pub fn summary(seqs: &[Seq]) -> BTreeMap<&'static str, (usize, usize, usize, String)> {
    let mut map: BTreeMap<&'static str, (usize, usize, usize, String)> = BTreeMap::new();
    for s in seqs {
        map.entry(s.category).and_modify(|e| e.0 += 1).or_insert((
            1,
            s.offset,
            s.len,
            s.detail.clone(),
        ));
    }
    map
}

/// Render a `.seq` listing.
pub fn listing(seqs: &[Seq]) -> String {
    let mut s = String::new();
    for q in seqs {
        s.push_str(&format!(
            "offset={:>8} len={:>3} category={:<28} {}\n",
            q.offset, q.len, q.category, q.detail
        ));
    }
    s
}
