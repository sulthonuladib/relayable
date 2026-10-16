//! The `relayable` CLI test harness.

use std::process::Command;

use serde_json::Value;

use super::binary::bin;
use super::common::child_path;

/// Captured result of one `relayable` CLI invocation.
pub struct CliOutput {
    /// Process exit code (`-1` if killed by a signal).
    pub status: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}

impl CliOutput {
    /// The exit code as `i64` for equality checks against literals.
    #[must_use]
    pub fn code(&self) -> i64 {
        i64::from(self.status)
    }
}

/// Run the `relayable` binary with `args` and `env` (for example
/// `[("RELAYABLE_SERVER", sock)]`).
///
/// The child environment is cleared first, so a developer's `RELAYABLE_SESSION`
/// or `XDG_RUNTIME_DIR` cannot leak in; only the caller's explicit pairs (plus
/// `PATH`) reach the CLI.
pub fn cli(args: &[&str], env: &[(&str, &str)]) -> CliOutput {
    let mut command = Command::new(bin("relayable"));
    command.args(args).env_clear().env("PATH", child_path());
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("run relayable");
    CliOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Run `relayable` with `args`/`env`, assert exit 0, and parse stdout as JSON.
pub fn cli_json(args: &[&str], env: &[(&str, &str)]) -> Value {
    let output = cli(args, env);
    assert_eq!(
        output.status, 0,
        "relayable {args:?} failed: status={} stdout={:?} stderr={:?}",
        output.status, output.stdout, output.stderr
    );
    serde_json::from_str(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "relayable {args:?} stdout is not JSON ({error}): {:?}",
            output.stdout
        )
    })
}
