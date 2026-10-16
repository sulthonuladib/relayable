//! POSIX process-group/session helpers used to terminate a block's tree.
//!
//! An interactive shell enables job control, so a background descendant such as
//! `sleep 30 &` gets its own process group but shares the child's session.
//! `BlockRuntime::close` therefore signals the child's process group (when it is
//! its own group leader, as `portable-pty`'s `setsid()` guarantees) and every
//! process in the child's session.

/// Whether `pid` is its own process-group leader (portable-pty's `setsid()`).
pub(super) fn owns_process_group(pid: u32) -> bool {
    // SAFETY: `getpgid` is a read-only query on a pid this server created.
    let pgid = unsafe { libc::getpgid(pid as libc::pid_t) };
    pgid >= 0 && pgid == pid as libc::pid_t
}

pub(super) fn session_of(pid: u32) -> Option<libc::pid_t> {
    // SAFETY: `getsid` is a read-only query on a pid this server created.
    let sid = unsafe { libc::getsid(pid as libc::pid_t) };
    if sid >= 0 {
        Some(sid)
    } else {
        None
    }
}

/// Signal one process group (`kill(-pgid, ...)`).
pub(super) fn signal_group(group: libc::pid_t, signal: libc::c_int) {
    // SAFETY: best-effort signal to a process group created by this server.
    unsafe {
        libc::kill(-group, signal);
    }
}

/// Signal every process whose session id is `session`.
///
/// Job control puts background jobs in their own process groups but keeps them
/// in the child's session, so a group kill alone can miss a descendant holding
/// the PTY slave.
pub(super) fn signal_session(session: libc::pid_t, signal: libc::c_int) {
    for pid in session_members(session) {
        // SAFETY: best-effort signal to a process in this server's session tree.
        unsafe {
            libc::kill(pid, signal);
        }
    }
}

/// Enumerate the pids whose session id is `session`, by reading `/proc`.
fn session_members(session: libc::pid_t) -> Vec<libc::pid_t> {
    let mut members = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return members;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            if stat_session(&stat) == Some(session) {
                members.push(pid);
            }
        }
    }
    members
}

/// Parse the session id from a `/proc/<pid>/stat` line.
///
/// The `comm` field can contain spaces and parentheses, so parsing resumes
/// after the final `)`: `state ppid pgrp session ...`.
fn stat_session(stat: &str) -> Option<libc::pid_t> {
    let rest = stat.rsplit_once(") ")?.1;
    let mut fields = rest.split_whitespace();
    let _state = fields.next()?;
    let _ppid = fields.next()?;
    let _pgrp = fields.next()?;
    fields.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_session_parses_after_comm_with_spaces() {
        let stat = "42 (weird ) name) S 1 42 42 0 -1 4194560";
        assert_eq!(stat_session(stat), Some(42));
    }
}
