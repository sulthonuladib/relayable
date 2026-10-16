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
/// Only a *production* source counts as "newer" (see [`is_production_source`]).
/// Cargo does not relink the binary for a test-only edit, so counting those
/// mtimes would fail a freshly built binary after an unrelated `#[cfg(test)]`
/// edit and make the `cargo build --workspace` guidance useless.
///
/// Panics if the binary is missing, not executable, or older than the newest
/// production source of the crates it is built from (see [`dependency_crates`]).
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
/// make the `cargo build --workspace` guidance useless. The lists below are the
/// `[dependencies]` of each binary's manifest, including `relayable-store`,
/// which `relayable-server` links (see `crates/relayable-server/Cargo.toml`).
fn dependency_crates(binary: &str) -> &'static [&'static str] {
    const SERVER: &[&str] = &[
        "relayable-server",
        "relayable-pty",
        "relayable-store",
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
        "relayable-store",
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

/// The newest *production* `*.rs`/`*.toml` mtime across the crates `binary` is
/// built from.
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

/// Walk `root` recursively and return the newest `(mtime, path)` of a
/// production `*.rs`/`*.toml` file, skipping any `target` directory. Bounded and
/// tiny (the crate tree only).
fn newest_source_mtime_in(root: &Path) -> Option<(SystemTime, PathBuf)> {
    let test_only = test_only_module_roots(root);
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for path in collect_files(root, is_source_file) {
        if !is_production_source(root, &path, &test_only) {
            continue;
        }
        if let Ok(modified) = fs::metadata(&path).and_then(|meta| meta.modified()) {
            if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
                newest = Some((modified, path));
            }
        }
    }
    newest
}

/// Pure classification: could editing this file change the compiled binary?
///
/// `path` is a `*.rs`/`*.toml` file inside the crate at `root`; `test_only`
/// lists the `#[cfg(test)]` module roots of that crate, relative to `root` (see
/// [`test_only_module_roots`]). Returns `false` only for files rustc provably
/// leaves out of a non-test build:
///
/// * `tests/`, `benches/`, and `examples/` trees at the crate root are separate
///   Cargo targets (autodiscovered integration tests, benches, examples) that
///   are never linked into the crate's bin/lib artifact;
/// * a `#[cfg(test)]` file module and the directory it owns. `#[cfg(test)]` is
///   stripped before a production build, so rustc's own dep-info for the
///   production target omits those files entirely — verified in this checkout
///   by `target/debug/.fingerprint/relayable-model-*/dep-lib-relayable_model`
///   listing `src/ids.rs` but not `src/tests.rs`, while the `dep-test-lib-*`
///   counterpart lists both. Cargo therefore does not relink the binary, and
///   the file's mtime must not count as staleness.
///
/// Anything the classifier does not recognize stays production-relevant, so an
/// unknown layout keeps the guard strict rather than silently skipping sources.
fn is_production_source(root: &Path, path: &Path, test_only: &[PathBuf]) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    if is_separate_test_target(relative) {
        return false;
    }
    !test_only
        .iter()
        .any(|module| relative == module.as_path() || relative.starts_with(module))
}

/// `tests/`, `benches/`, and `examples/` under a crate root are separate Cargo
/// targets, never linked into the crate's own bin/lib artifact.
fn is_separate_test_target(relative: &Path) -> bool {
    matches!(
        relative
            .components()
            .next()
            .and_then(|part| part.as_os_str().to_str()),
        Some("tests" | "benches" | "examples")
    )
}

/// The paths (relative to `crate_root`) that a `#[cfg(test)]` module declaration
/// owns, so their mtimes can be ignored.
///
/// Only file modules (`mod name;`) can be pruned by path. An inline
/// `#[cfg(test)] mod tests { .. }` block lives inside an otherwise production
/// file and is deliberately left alone: pruning it would require parsing Rust,
/// and leaving it counted can only make the guard stricter, never weaker (see
/// [`is_production_source`]).
fn test_only_module_roots(crate_root: &Path) -> Vec<PathBuf> {
    let src = crate_root.join("src");
    let mut roots = Vec::new();
    for file in collect_files(&src, is_rust_file) {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        let dir = file.parent().unwrap_or(&src);
        for name in test_only_file_modules(&source) {
            let module = dir.join(&name);
            let owned = if module.join("mod.rs").exists() {
                module // a directory module also owns everything below it
            } else if dir.join(format!("{name}.rs")).exists() {
                dir.join(format!("{name}.rs"))
            } else {
                continue;
            };
            if let Ok(relative) = owned.strip_prefix(crate_root) {
                roots.push(relative.to_path_buf());
            }
        }
    }
    roots
}

/// Module names declared as `#[cfg(test)] mod <name>;` in one source file.
fn test_only_file_modules(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut names = Vec::new();
    for (index, &line) in lines.iter().enumerate() {
        if let Some(name) = file_module_name(line) {
            if has_cfg_test_attribute(&lines[..index]) {
                names.push(name);
            }
        }
    }
    names
}

/// The name of a `mod <name>;` declaration line, ignoring a visibility prefix.
/// Returns `None` for everything else, including an inline `mod <name> { .. }`
/// block and a `use`/`include` line.
fn file_module_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.ends_with(';') || line.contains('{') {
        return None;
    }
    let mut tokens = line.trim_end_matches(';').split_whitespace();
    let mut token = tokens.next()?;
    while is_visibility(token) {
        token = tokens.next()?;
    }
    if token != "mod" {
        return None;
    }
    let name = tokens.next()?;
    if tokens.next().is_some() || !is_identifier(name) {
        return None;
    }
    Some(name.to_string())
}

/// Whether the attribute block immediately above a declaration line contains
/// `#[cfg(test)]`. Blank lines and `//` comments are skipped; other attribute
/// lines are collected until a non-attribute line ends the block.
///
/// Conservative: an unrecognized form is reported as *not* test-gated, so the
/// file stays production-relevant and the guard stays strict.
fn has_cfg_test_attribute(lines_before: &[&str]) -> bool {
    for line in lines_before.iter().rev() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with("#[") {
            if is_cfg_test_attribute(line) {
                return true;
            }
            continue;
        }
        return false;
    }
    false
}

/// Whether one attribute line is a `cfg(test)` gate, ignoring whitespace.
fn is_cfg_test_attribute(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    let Some(inner) = compact.strip_prefix("#[cfg(") else {
        return false;
    };
    let Some(inner) = inner.strip_suffix(")]") else {
        return false;
    };
    // `not(test)` means "production only": never prune on it.
    if inner.starts_with("not(") || inner.contains("not(test)") {
        return false;
    }
    inner == "test"
        || inner.starts_with("test,")
        || inner.ends_with(",test")
        || inner.contains(",test,")
        || inner.contains("(test")
}

/// Rust visibility modifier that may precede a `mod` declaration.
fn is_visibility(token: &str) -> bool {
    matches!(token, "pub" | "pub(crate)" | "pub(super)" | "pub(self)")
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_alphabetic() || first == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Every file under `root` (skipping `target` directories) that `wanted`
/// accepts. Bounded and tiny: the walk is confined to one crate tree.
fn collect_files(root: &Path, wanted: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
            } else if wanted(&path) {
                found.push(path);
            }
        }
    }
    found
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("rs" | "toml")
    )
}

fn is_rust_file(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("rs")
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

    #[test]
    fn file_module_names_are_read_only_from_declarations() {
        assert_eq!(file_module_name("mod tests;").as_deref(), Some("tests"));
        assert_eq!(
            file_module_name("    mod events;").as_deref(),
            Some("events")
        );
        assert_eq!(
            file_module_name("pub(crate) mod harness;").as_deref(),
            Some("harness")
        );
        // Inline blocks, `use` lines, and non-declarations are not modules.
        assert_eq!(file_module_name("mod tests {"), None);
        assert_eq!(file_module_name("mod tests; // trailing"), None);
        assert_eq!(file_module_name("use error::CliError;"), None);
        assert_eq!(file_module_name("// mod tests;"), None);
        assert_eq!(file_module_name("mod tests extra;"), None);
    }

    #[test]
    fn cfg_test_is_recognized_only_as_a_gate() {
        assert!(has_cfg_test_attribute(&["#[cfg(test)]"]));
        assert!(has_cfg_test_attribute(&[
            "// a comment",
            "",
            "#[cfg(all(test, feature = \"x\"))]"
        ]));
        assert!(has_cfg_test_attribute(&[
            "#[allow(dead_code)]",
            "#[cfg(test)]"
        ]));
        assert!(!has_cfg_test_attribute(&["#[cfg(unix)]"]));
        assert!(!has_cfg_test_attribute(&["#[cfg(not(test))]"]));
        assert!(!has_cfg_test_attribute(&["#[cfg(test)]", "let x = 1;"]));
        assert!(!has_cfg_test_attribute(&[]));
    }

    #[test]
    fn test_only_file_modules_tracks_gated_declarations() {
        let source = "\
mod production;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod harness;

#[cfg(unix)]
mod platform;
";
        assert_eq!(test_only_file_modules(source), vec!["tests", "harness"]);
    }

    #[test]
    fn production_classification_ignores_test_only_paths() {
        let root = Path::new("crates/relayable-server");
        let test_only = vec![PathBuf::from("src/tests.rs")];

        // Production sources always count.
        assert!(is_production_source(
            root,
            &root.join("src/session.rs"),
            &test_only
        ));
        assert!(is_production_source(
            root,
            &root.join("build.rs"),
            &test_only
        ));
        assert!(is_production_source(
            root,
            &root.join("Cargo.toml"),
            &test_only
        ));

        // A `#[cfg(test)]` file module does not.
        assert!(!is_production_source(
            root,
            &root.join("src/tests.rs"),
            &test_only
        ));

        // Integration tests, benches, and examples are separate targets.
        for path in [
            "tests/server_e2e.rs",
            "benches/latency.rs",
            "examples/demo.rs",
        ] {
            assert!(
                !is_production_source(root, &root.join(path), &test_only),
                "{path} is a separate Cargo target"
            );
        }
    }

    #[test]
    fn production_classification_prunes_a_directory_module_and_its_children() {
        let root = Path::new("crates/relayable-client");
        let test_only = vec![PathBuf::from("src/tests")];

        assert!(!is_production_source(
            root,
            &root.join("src/tests/harness.rs"),
            &test_only
        ));
        assert!(is_production_source(
            root,
            &root.join("src/client.rs"),
            &test_only
        ));
    }

    #[test]
    fn this_checkout_cfg_test_modules_are_classified_as_test_only() {
        // Guards the regression QA-HARNESS-001 records: in this repo every
        // `tests` module under a crate's `src/` is `#[cfg(test)]`-gated, so an
        // edit there must not count as staleness.
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for crate_name in dependency_crates("relayable-server") {
            let root = repo.join("crates").join(crate_name);
            let roots = test_only_module_roots(&root);
            for module in &roots {
                assert!(
                    module.starts_with("src"),
                    "{crate_name}: {module:?} should be a src module"
                );
                assert!(
                    !is_production_source(&root, &root.join(module), &roots),
                    "{crate_name}: {module:?} should be pruned"
                );
            }
        }

        let model = repo.join("crates/relayable-model");
        let roots = test_only_module_roots(&model);
        assert!(
            roots.contains(&PathBuf::from("src/tests.rs")),
            "relayable-model's `#[cfg(test)] mod tests;` should be pruned, got {roots:?}"
        );
        assert!(is_production_source(
            &model,
            &model.join("src/ids.rs"),
            &roots
        ));
        assert!(!is_production_source(
            &model,
            &model.join("src/tests.rs"),
            &roots
        ));
    }
}
