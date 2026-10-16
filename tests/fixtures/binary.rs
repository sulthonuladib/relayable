//! Workspace binary location and the staleness freshness guard.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Resolve a workspace binary next to the running test executable.
///
/// **The acceptance tests require a `cargo build --workspace` first.** This
/// package intentionally does not depend on `relayable-server`/`relayable-cli`
/// (ADR-001: the tests package links only model/protocol/client), so `cargo test
/// -p relayable-integration-tests` does not build those binaries. To keep the
/// evidence honest, this function therefore fails loudly when the binary is
/// older than the newest crate source instead of silently running a stale
/// build (a stale-server run already produced misleading output once).
///
/// The test exe lives in `<target>/debug/deps/`, so pop the exe, pop `deps` when
/// present, and append `name`. Never hard-codes a target directory.
///
/// Panics if the binary is missing, not executable, or stale relative to the
/// newest `*.rs`/`*.toml` of the crates it is built from (see
/// [`dependency_crates`]).
#[must_use]
pub fn bin(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current test executable path");
    let mut dir = exe
        .parent()
        .expect("test executable has a parent directory")
        .to_path_buf();
    if dir.file_name().and_then(|value| value.to_str()) == Some("deps") {
        dir = dir
            .parent()
            .expect("deps directory has a parent")
            .to_path_buf();
    }
    let path = dir.join(name);
    assert!(
        path.exists(),
        "binary {name:?} not found at {path:?}; run `cargo build --workspace` first"
    );
    let metadata =
        fs::metadata(&path).unwrap_or_else(|error| panic!("cannot stat binary {path:?}: {error}"));
    let mode = metadata.permissions().mode();
    assert!(
        mode & 0o111 != 0,
        "binary {path:?} is not executable (mode {:o})",
        mode & 0o777
    );
    guard_freshness(name, &path, &metadata);
    path
}

/// Pure staleness decision: `Some(reason)` when the newest source is strictly
/// newer than the binary, `None` when the binary is fresh. Kept free of I/O so
/// it can be unit-tested directly.
fn staleness(binary_mtime: SystemTime, newest_source_mtime: SystemTime) -> Option<String> {
    if newest_source_mtime > binary_mtime {
        Some(format!(
            "it is older (mtime {binary_mtime:?}) than the newest crate source (mtime {newest_source_mtime:?})"
        ))
    } else {
        None
    }
}

/// Fail loudly if `path` is older than the newest source it is built from,
/// naming that source.
fn guard_freshness(name: &str, path: &Path, metadata: &fs::Metadata) {
    let Ok(binary_mtime) = metadata.modified() else {
        return; // Cannot compare: do not block the run on an unsupported timestamp.
    };
    let Some((newest_mtime, newest_path)) = newest_dependency_source(name) else {
        return; // No sources found: nothing to compare against.
    };
    if let Some(reason) = staleness(binary_mtime, newest_mtime) {
        panic!(
            "refusing to run a stale {name:?} binary at {path:?}: {reason}. \
             The newest source it is built from is {newest_path:?}; run `cargo build --workspace` first."
        );
    }
}

/// The in-repo crates each workspace binary is built from. This mirrors the
/// frozen ADR-001 dependency graph; keep it in step with the manifests.
///
/// The freshness comparison must be scoped to a binary's real dependencies:
/// Cargo does **not** rebuild `relayable-cli` when only `relayable-server`
/// changes (and vice versa), so comparing against every crate under `crates/**`
/// would falsely flag a freshly built binary after an unrelated crate edit and
/// make the `cargo build --workspace` guidance useless.
fn dependency_crates(binary: &str) -> &'static [&'static str] {
    const SERVER: &[&str] = &[
        "relayable-server",
        "relayable-pty",
        "relayable-protocol",
        "relayable-model",
    ];
    const CLI: &[&str] = &[
        "relayable-cli",
        "relayable-client",
        "relayable-protocol",
        "relayable-model",
    ];
    const ALL: &[&str] = &[
        "relayable-model",
        "relayable-protocol",
        "relayable-pty",
        "relayable-server",
        "relayable-client",
        "relayable-cli",
    ];
    match binary {
        "relayable-server" => SERVER,
        "relayable" => CLI,
        // Conservative fallback for an unknown binary: compare against all crates.
        _ => ALL,
    }
}

/// The newest `*.rs`/`*.toml` mtime across the crates `binary` is built from.
fn newest_dependency_source(binary: &str) -> Option<(SystemTime, PathBuf)> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let crates = repo.join("crates");
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for crate_name in dependency_crates(binary) {
        if let Some((time, path)) = newest_source_mtime_in(&crates.join(crate_name)) {
            if newest.as_ref().is_none_or(|(current, _)| time > *current) {
                newest = Some((time, path));
            }
        }
    }
    newest
}

/// Walk `root` recursively and return the newest `(mtime, path)` of a `*.rs` or
/// `*.toml` file, skipping any `target` directory. Bounded and tiny (the crate
/// tree only).
fn newest_source_mtime_in(root: &Path) -> Option<(SystemTime, PathBuf)> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
            } else if is_source_file(&path) {
                if let Ok(modified) = entry.metadata().and_then(|meta| meta.modified()) {
                    if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
                        newest = Some((modified, path));
                    }
                }
            }
        }
    }
    newest
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("rs" | "toml")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn staleness_flags_only_an_older_binary() {
        let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let newer_source = base + Duration::from_secs(1);

        assert!(staleness(base, base).is_none(), "equal mtimes are fresh");
        assert!(
            staleness(newer_source, base).is_none(),
            "a binary newer than its sources is fresh"
        );
        let reason = staleness(base, newer_source).expect("an older binary is stale");
        assert!(
            reason.contains("older"),
            "the staleness reason should explain the condition: {reason}"
        );
    }

    #[test]
    fn source_scan_tolerates_a_missing_root() {
        assert!(newest_source_mtime_in(Path::new("/nonexistent-relayable-root")).is_none());
    }
}
