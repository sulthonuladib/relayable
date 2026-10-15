//! Artifact writers: per-check files, machine-readable summary, and provenance.
//!
//! Every file records the UTC timestamp, host, toolchain, git HEAD/status, and
//! the exact input sources (path + SHA-256), so a reviewer can reproduce a
//! result from `scripts/run_all.sh` and the committed raw artifacts.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::checks::{CheckResult, Status};

/// `YYYY-MM-DDTHH:MM:SSZ` (UTC).
#[must_use]
pub fn utc_now() -> String {
    // `date -u +%Y-%m-%dT%H:%M:%SZ` is used by the shell driver; inside the
    // binaries we format SystemTime directly without a chrono dependency.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = civil_from_unix(secs as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

// Howard Hinnant's civil_from_days algorithm.
fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (
        y,
        m as u32,
        d as u32,
        (rem / 3600) as u32,
        ((rem % 3600) / 60) as u32,
        (rem % 60) as u32,
    )
}

fn run_stdout(cmd: &str, args: &[&str]) -> String {
    match Command::new(cmd).args(args).output() {
        Ok(out) => {
            let mut s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                s = String::from_utf8_lossy(&out.stderr).trim().to_string();
            }
            s
        }
        Err(e) => format!("<{cmd} failed: {e}>"),
    }
}

/// Recursively collect regular files under `dir` into `out`.
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

/// Deterministic identity of the exact phase-2 code revision that produced an
/// artifact: `sha256` over a sorted, newline-joined list of
/// `relative_path:file_sha256` for
/// `spikes/terminal-engine-phase2/{Cargo.toml,Cargo.lock,src/**,scripts/run_all.sh}`.
///
/// This exists because the phase-2 tree is untracked during the spike, so the
/// `git_head` stamp is the *base* commit the branch was cut from, not the code
/// revision. `scripts/run_all.sh` computes the same value for
/// `results/raw/run_all_meta.txt`; see README.md for the exact recompute recipe.
#[must_use]
pub fn source_tree_sha256() -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_root
        .parent()
        .and_then(Path::parent)
        .unwrap_or(crate_root);
    let mut files: Vec<PathBuf> = Vec::new();
    for name in ["Cargo.toml", "Cargo.lock", "scripts/run_all.sh"] {
        let path = crate_root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    collect_files(&crate_root.join("src"), &mut files);
    let mut lines: Vec<String> = files
        .iter()
        .map(|path| {
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(path)
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let digest = fs::read(path)
                .map(|bytes| crate::sha256::sha256_hex(&bytes))
                .unwrap_or_default();
            format!("{relative}:{digest}")
        })
        .collect();
    lines.sort();
    let joined = format!("{}\n", lines.join("\n"));
    crate::sha256::sha256_hex(joined.as_bytes())
}

/// Provenance header lines shared by every artifact.
#[must_use]
pub fn provenance_lines() -> Vec<String> {
    vec![
        format!("utc_timestamp: {}", utc_now()),
        format!("hostname: {}", run_stdout("hostname", &[])),
        format!("uname: {}", run_stdout("uname", &["-a"])),
        format!("rustc: {}", run_stdout("rustc", &["--version"])),
        format!("cargo: {}", run_stdout("cargo", &["--version"])),
        format!("git_head: {}", run_stdout("git", &["rev-parse", "HEAD"])),
        format!("source_tree_sha256: {}", source_tree_sha256()),
    ]
}

fn git_status() -> String {
    run_stdout("git", &["status", "--short"])
}

fn write_prefixed(out: &mut String, prefix: &str, text: &str) {
    if text.is_empty() {
        out.push_str(&format!("{prefix}<empty>\n"));
        return;
    }
    for line in text.lines() {
        out.push_str(&format!("{prefix}{line}\n"));
    }
}

/// Path of the per-check artifact for `result`.
#[must_use]
pub fn check_path(dir: &Path, result: &CheckResult) -> PathBuf {
    dir.join(format!("{}__{}.txt", result.engine, result.check))
}

/// Write one per-check artifact and return its path.
pub fn write_check(dir: &Path, result: &CheckResult) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let path = check_path(dir, result);
    let mut out = String::new();
    out.push_str("# ---- provenance ----\n");
    out.push_str(&format!(
        "# engine: {} {}\n",
        result.engine, result.engine_version
    ));
    out.push_str(&format!("# check: {}\n", result.check));
    out.push_str(&format!("# title: {}\n", result.title));
    out.push_str(&format!("# kind: {}\n", result.kind));
    for line in provenance_lines() {
        out.push_str(&format!("# {line}\n"));
    }
    out.push_str("# git_status_short:\n");
    for line in git_status().lines() {
        out.push_str(&format!("#   {line}\n"));
    }
    out.push_str("# ---- inputs ----\n");
    if result.sources.is_empty() {
        out.push_str("# (none)\n");
    }
    for s in &result.sources {
        out.push_str(&format!(
            "# source: {} kind={} sha256={}\n",
            s.path, s.kind, s.sha256
        ));
    }
    out.push_str("# ---- expected vs actual ----\n");
    for a in &result.assertions {
        out.push_str(&format!("ASSERT {}\n", a.name));
        out.push_str(&format!("  EXPECTED {}\n", a.expected));
        out.push_str(&format!("  ACTUAL   {}\n", a.actual));
        out.push_str(&format!("  STATUS   {}\n", a.status.as_str()));
    }
    out.push_str("# ---- screen dump (final visible screen) ----\n");
    if result.screen_dump.is_empty() {
        out.push_str("| <empty>\n");
    } else {
        write_prefixed(&mut out, "| ", &result.screen_dump);
    }
    out.push_str("# ---- notes ----\n");
    if result.notes.is_empty() {
        out.push_str("- (none)\n");
    }
    for n in &result.notes {
        out.push_str(&format!("- {n}\n"));
    }
    out.push_str(&format!("STATUS={}\n", result.status().as_str()));
    fs::write(&path, out)?;
    Ok(path)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Write `summary.json` and `summary.txt`.
pub fn write_summary(json_path: &Path, txt_path: &Path, results: &[CheckResult]) -> io::Result<()> {
    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"provenance\": {\n");
    let prov = provenance_lines();
    for (i, line) in prov.iter().enumerate() {
        let (k, v) = line.split_once(':').unwrap_or((line, ""));
        json.push_str(&format!(
            "    \"{}\": \"{}\"{}\n",
            json_escape(k.trim()),
            json_escape(v.trim()),
            if i + 1 == prov.len() { "" } else { "," }
        ));
    }
    json.push_str("  },\n");
    json.push_str(&format!(
        "  \"git_status_short\": \"{}\",\n",
        json_escape(&git_status())
    ));
    let mut tally: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in results {
        *tally.entry(r.status().as_str()).or_insert(0) += 1;
    }
    json.push_str("  \"tally\": {");
    let tally_items: Vec<String> = tally
        .iter()
        .map(|(k, v)| format!("\"{}\": {}", json_escape(k), v))
        .collect();
    json.push_str(&tally_items.join(", "));
    json.push_str("},\n");
    json.push_str("  \"checks\": [\n");
    for (i, r) in results.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!(
            "      \"engine\": \"{}\",\n",
            json_escape(&r.engine)
        ));
        json.push_str(&format!(
            "      \"engine_version\": \"{}\",\n",
            json_escape(&r.engine_version)
        ));
        json.push_str(&format!(
            "      \"check\": \"{}\",\n",
            json_escape(&r.check)
        ));
        json.push_str(&format!("      \"kind\": \"{}\",\n", json_escape(&r.kind)));
        json.push_str(&format!("      \"status\": \"{}\",\n", r.status().as_str()));
        json.push_str("      \"sources\": [");
        let srcs: Vec<String> = r
            .sources
            .iter()
            .map(|s| {
                format!(
                    "{{\"path\": \"{}\", \"kind\": \"{}\", \"sha256\": \"{}\"}}",
                    json_escape(&s.path),
                    json_escape(&s.kind),
                    json_escape(&s.sha256)
                )
            })
            .collect();
        json.push_str(&srcs.join(", "));
        json.push_str("],\n");
        let fails: Vec<String> = r
            .assertions
            .iter()
            .filter(|a| a.status == Status::Fail)
            .map(|a| json_escape(&a.name))
            .collect();
        json.push_str(&format!(
            "      \"failed_assertions\": [{}]\n",
            fails
                .iter()
                .map(|f| format!("\"{f}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        json.push_str(&format!(
            "    }}{}\n",
            if i + 1 == results.len() { "" } else { "," }
        ));
    }
    json.push_str("  ]\n}\n");
    fs::write(json_path, json)?;

    let mut txt = String::new();
    txt.push_str("Relayable TERM-002 phase B — engine check summary\n");
    txt.push_str("================================================\n\n");
    for line in provenance_lines() {
        txt.push_str(&format!("{line}\n"));
    }
    txt.push_str(&format!("git_status_short: {}\n", git_status()));
    txt.push_str("\nPer-check status:\n");
    txt.push_str(&format!(
        "{:<26} {:<28} {:<24} {}\n",
        "engine", "check", "kind", "status"
    ));
    for r in results {
        txt.push_str(&format!(
            "{:<26} {:<28} {:<24} {}\n",
            r.engine,
            r.check,
            r.kind,
            r.status().as_str()
        ));
    }
    txt.push_str("\nTally:\n");
    for (k, v) in &tally {
        txt.push_str(&format!("  {k}: {v}\n"));
    }
    // Also tally per engine.
    txt.push_str("\nPer-engine tally:\n");
    let mut engines: Vec<&str> = results.iter().map(|r| r.engine.as_str()).collect();
    engines.sort_unstable();
    engines.dedup();
    for engine in engines {
        let mut t: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for r in results.iter().filter(|r| r.engine == engine) {
            *t.entry(r.status().as_str()).or_insert(0) += 1;
        }
        let parts: Vec<String> = t.iter().map(|(k, v)| format!("{k}={v}")).collect();
        txt.push_str(&format!("  {engine}: {}\n", parts.join(" ")));
    }

    // Per-engine implementation status (distinguishes "not yet wired" from
    // "failed"). Stub engines are detected by the `engine_adapter_implemented`
    // placeholder assertion emitted for every catalog check; an implemented
    // engine may still have individual UNSUPPORTED sub-checks.
    txt.push_str("\nEngine implementation status:\n");
    let mut all_engines: Vec<&str> = results.iter().map(|r| r.engine.as_str()).collect();
    all_engines.sort_unstable();
    all_engines.dedup();
    for engine in all_engines {
        let engine_results: Vec<&CheckResult> =
            results.iter().filter(|r| r.engine == engine).collect();
        let mut t: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for r in &engine_results {
            *t.entry(r.status().as_str()).or_insert(0) += 1;
        }
        let parts: Vec<String> = t.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let stub_reason = engine_results
            .iter()
            .flat_map(|r| r.assertions.iter())
            .find(|a| a.name == "engine_adapter_implemented" && a.status == Status::Unsupported)
            .map(|a| a.actual.clone());
        match stub_reason {
            Some(reason) => txt.push_str(&format!(
                "  {engine}: NOT_IMPLEMENTED [{}] - {reason}\n",
                parts.join(" ")
            )),
            None => {
                let unsupported = engine_results
                    .iter()
                    .flat_map(|r| r.assertions.iter())
                    .find(|a| a.status == Status::Unsupported)
                    .map(|a| a.actual.clone());
                match unsupported {
                    Some(reason) => txt.push_str(&format!(
                        "  {engine}: implemented [{}] - one sub-check UNSUPPORTED: {reason}\n",
                        parts.join(" ")
                    )),
                    None => {
                        txt.push_str(&format!("  {engine}: implemented [{}]\n", parts.join(" ")))
                    }
                }
            }
        }
    }
    fs::write(txt_path, txt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats() {
        assert_eq!(civil_from_unix(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(civil_from_unix(1_000_000_000), (2001, 9, 9, 1, 46, 40));
    }

    #[test]
    fn json_escaping() {
        assert_eq!(json_escape("a\"b\nc"), "a\\\"b\\nc");
    }
}
