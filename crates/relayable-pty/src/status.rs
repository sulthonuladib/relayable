/// The observed outcome of a PTY child process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyExitStatus {
    /// The process exit code (`1` when the process was terminated by a signal).
    pub exit_code: u32,
    /// The terminating signal name, when the platform reports one.
    pub signal: Option<String>,
    /// Whether the process exited successfully.
    pub success: bool,
}
