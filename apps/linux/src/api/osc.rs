use vte::Perform;

// --- OSC title/cwd tracking ------------------------------------------------

/// The latest OSC 0/2 title and OSC 7 cwd seen on the output stream.
#[derive(Debug, Default, Clone)]
pub(super) struct TitleTracker {
    pub(super) title: Option<String>,
    pub(super) cwd: Option<String>,
}

/// A `vte` performer that only records OSC 0/2 (title) and OSC 7 (cwd).
pub(super) struct OscPerformer<'a> {
    pub(super) tracker: &'a mut TitleTracker,
    pub(super) changed: bool,
}

impl Perform for OscPerformer<'_> {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() < 2 {
            return;
        }
        match params[0] {
            b"0" | b"2" => {
                let title = params[1..]
                    .iter()
                    .map(|part| String::from_utf8_lossy(part))
                    .collect::<Vec<_>>()
                    .join(";");
                self.tracker.title = Some(title.trim().to_string());
                self.changed = true;
            }
            b"7" => {
                let raw = String::from_utf8_lossy(params[1]).to_string();
                self.tracker.cwd = Some(strip_file_uri(&raw));
                self.changed = true;
            }
            _ => {}
        }
    }
}

/// Turn `file://host/path` into `/path`; leave anything else untouched.
pub(super) fn strip_file_uri(raw: &str) -> String {
    let Some(rest) = raw.strip_prefix("file://") else {
        return raw.to_string();
    };
    match rest.find('/') {
        Some(index) => rest[index..].to_string(),
        None => rest.to_string(),
    }
}
