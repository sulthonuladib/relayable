//! `analyze` — run the engine adapters against fixtures and replayed real
//! captures, and write the per-check artifacts plus `summary.json`/`summary.txt`.
//!
//! ```text
//! analyze [--results-dir DIR] [--repo-root DIR] [--perf] [--perf-mib N]
//! ```
//!
//! It never fabricates state: every assertion comes from a real feed into the
//! pinned engines, and every input is SHA-256 stamped.

use std::fs;
use std::path::{Path, PathBuf};

use relayable_terminal_engine_phase2_spike::checks::{self, Env, PerfRecord, Status, Stream};
use relayable_terminal_engine_phase2_spike::engine;
use relayable_terminal_engine_phase2_spike::fixtures;
use relayable_terminal_engine_phase2_spike::report;

#[derive(Debug)]
struct Args {
    results_dir: PathBuf,
    repo_root: PathBuf,
    cols: u16,
    rows: u16,
    scrollback: usize,
    perf: bool,
    perf_mib: usize,
    /// When set, run ONLY the isolated throughput observation for this engine
    /// and exit (`--perf-only <engine>`); used once per engine by run_all.sh.
    perf_only: Option<String>,
    /// Number of throughput passes per engine (>= 1).
    perf_runs: usize,
}

fn default_repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest)
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        results_dir: PathBuf::from("results/raw"),
        repo_root: default_repo_root(),
        cols: 80,
        rows: 24,
        scrollback: 1000,
        perf: true,
        perf_mib: 8,
        perf_only: None,
        perf_runs: 3,
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0usize;
    while i < argv.len() {
        let val = |i: usize| -> Result<&String, String> {
            argv.get(i)
                .ok_or_else(|| format!("{} needs a value", argv[i - 1]))
        };
        match argv[i].as_str() {
            "--results-dir" => {
                i += 1;
                a.results_dir = PathBuf::from(val(i)?);
            }
            "--repo-root" => {
                i += 1;
                a.repo_root = PathBuf::from(val(i)?);
            }
            "--cols" => {
                i += 1;
                a.cols = val(i)?.parse().map_err(|e| format!("--cols: {e}"))?;
            }
            "--rows" => {
                i += 1;
                a.rows = val(i)?.parse().map_err(|e| format!("--rows: {e}"))?;
            }
            "--scrollback" => {
                i += 1;
                a.scrollback = val(i)?.parse().map_err(|e| format!("--scrollback: {e}"))?;
            }
            "--perf-mib" => {
                i += 1;
                a.perf_mib = val(i)?.parse().map_err(|e| format!("--perf-mib: {e}"))?;
            }
            "--perf-only" => {
                i += 1;
                a.perf_only = Some(val(i)?.clone());
            }
            "--perf-runs" => {
                i += 1;
                a.perf_runs = val(i)?.parse().map_err(|e| format!("--perf-runs: {e}"))?;
            }
            "--no-perf" => a.perf = false,
            "--perf" => a.perf = true,
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok(a)
}

/// Read `resize=MS:COLSxROWS` lines from a capture meta file.
fn parse_resizes(meta_path: &Path) -> Vec<(u64, u16, u16)> {
    let Ok(text) = fs::read_to_string(meta_path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("resize=") else {
            continue;
        };
        let Some((ms, dims)) = rest.split_once(':') else {
            continue;
        };
        let Some((cols, rows)) = dims.split_once('x') else {
            continue;
        };
        if let (Ok(ms), Ok(cols), Ok(rows)) = (ms.parse(), cols.parse(), rows.parse()) {
            out.push((ms, cols, rows));
        }
    }
    out
}

fn load_streams(dir: &Path) -> Vec<Stream> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    let mut bins: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "bin"))
        .collect();
    bins.sort();
    for bin in bins {
        let name = bin
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let bytes = match fs::read(&bin) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("analyze: cannot read {}: {e}", bin.display());
                continue;
            }
        };
        let resizes = parse_resizes(&bin.with_extension("meta"));
        out.push(Stream {
            name,
            path: bin,
            bytes,
            resizes,
        });
    }
    out
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

fn opt_u64(v: Option<u64>) -> String {
    v.map_or_else(|| "null".to_string(), |x| x.to_string())
}

fn opt_i64(v: Option<i64>) -> String {
    v.map_or_else(|| "null".to_string(), |x| x.to_string())
}

fn opt_f64(v: Option<f64>) -> String {
    v.map_or_else(|| "null".to_string(), |x| format!("{x:.3}"))
}

/// One engine's isolated-process perf observation as a JSON object.
fn perf_engine_json(rec: &PerfRecord) -> String {
    let mut runs = Vec::new();
    for run in &rec.runs {
        runs.push(format!(
            "    {{ \"pass\": {}, \"elapsed_ms\": {:.3}, \"bytes_per_sec\": {:.0}, \"rss_before_kb\": {}, \"rss_after_kb\": {}, \"rss_delta_kb\": {}, \"vmhwm_kb\": {} }}",
            run.index,
            run.elapsed_ms,
            run.bytes_per_sec,
            opt_u64(run.rss_before_kb),
            opt_u64(run.rss_after_kb),
            opt_i64(run.rss_delta_kb),
            opt_u64(run.vmhwm_kb)
        ));
    }
    format!(
        "{{\n  \"engine\": \"{}\",\n  \"engine_version\": \"{}\",\n  \"isolation\": \"dedicated process, single engine\",\n  \"description\": \"{}\",\n  \"bytes_fed\": {},\n  \"lines_produced\": {},\n  \"retained_lines\": {},\n  \"configured_bound\": {},\n  \"bytes_per_sec_min\": {},\n  \"bytes_per_sec_median\": {},\n  \"bytes_per_sec_max\": {},\n  \"rss_baseline_kb\": {},\n  \"rss_after_kb\": {},\n  \"rss_delta_kb\": {},\n  \"vmhwm_kb\": {},\n  \"runs\": [\n{}\n  ]\n}}\n",
        json_escape(&rec.engine),
        json_escape(&rec.engine_version),
        json_escape(&rec.description),
        rec.bytes_fed,
        rec.lines_produced,
        rec.retained_lines,
        rec.configured_bound,
        opt_f64(rec.bytes_per_sec_min),
        opt_f64(rec.bytes_per_sec_median),
        opt_f64(rec.bytes_per_sec_max),
        opt_u64(rec.rss_baseline_kb),
        opt_u64(rec.rss_after_kb),
        opt_i64(rec.rss_delta_kb),
        opt_u64(rec.vmhwm_kb),
        runs.join(",\n")
    )
}

/// Human-readable per-engine perf text.
fn perf_engine_txt(rec: &PerfRecord) -> String {
    let mut txt = String::new();
    txt.push_str(&format!(
        "\nengine: {} {}\n",
        rec.engine, rec.engine_version
    ));
    txt.push_str("isolation: measured in a dedicated process that constructed only this engine\n");
    txt.push_str(&format!("description: {}\n", rec.description));
    txt.push_str(&format!("bytes_fed: {}\n", rec.bytes_fed));
    txt.push_str(&format!("lines_produced: {}\n", rec.lines_produced));
    txt.push_str(&format!("retained_lines: {}\n", rec.retained_lines));
    txt.push_str(&format!("configured_bound: {}\n", rec.configured_bound));
    txt.push_str(&format!("passes: {}\n", rec.runs.len()));
    for run in &rec.runs {
        txt.push_str(&format!(
            "  pass {}: elapsed_ms={:.3} bytes_per_sec={:.0} rss_before_kb={} rss_after_kb={} rss_delta_kb={} vmhwm_kb={}\n",
            run.index,
            run.elapsed_ms,
            run.bytes_per_sec,
            opt_u64(run.rss_before_kb),
            opt_u64(run.rss_after_kb),
            opt_i64(run.rss_delta_kb),
            opt_u64(run.vmhwm_kb)
        ));
    }
    txt.push_str(&format!(
        "bytes_per_sec_min: {}\nbytes_per_sec_median: {}\nbytes_per_sec_max: {}\n",
        opt_f64(rec.bytes_per_sec_min),
        opt_f64(rec.bytes_per_sec_median),
        opt_f64(rec.bytes_per_sec_max)
    ));
    txt.push_str(&format!(
        "rss_baseline_kb: {}\nrss_after_kb: {}\nrss_delta_kb: {}\nvmhwm_kb: {}\n",
        opt_u64(rec.rss_baseline_kb),
        opt_u64(rec.rss_after_kb),
        opt_i64(rec.rss_delta_kb),
        opt_u64(rec.vmhwm_kb)
    ));
    txt
}

/// `analyze --perf-only <engine>`: run the throughput observation for exactly
/// one engine in this process, then exit. Called once per engine by
/// `scripts/run_all.sh`, which is what makes the RSS numbers per-engine.
fn run_perf_only(args: &Args, name: &str) -> i32 {
    let Some((static_name, factory)) = engine::engines().into_iter().find(|(n, _)| *n == name)
    else {
        let known: Vec<&str> = engine::engines().iter().map(|(n, _)| *n).collect();
        eprintln!("analyze: --perf-only unknown engine {name:?}; known: {known:?}");
        return 2;
    };
    let name = static_name;
    let perf_dir = args.results_dir.join("perf");
    if let Err(e) = fs::create_dir_all(&perf_dir) {
        eprintln!("analyze: create {}: {e}", perf_dir.display());
        return 2;
    }
    let version = engine::engine_version(name);
    let fixtures: Vec<fixtures::Fixture> = Vec::new();
    let env = Env {
        engine: name,
        version,
        factory,
        fixtures: &fixtures,
        streams: &[],
        cols: args.cols,
        rows: args.rows,
        scrollback: args.scrollback,
        repo_root: args.repo_root.clone(),
    };
    let probe = factory(args.cols, args.rows, args.scrollback);
    if let Some(reason) = probe.implementation_status().reason() {
        let skip_json = format!(
            "  {{ \"engine\": \"{}\", \"reason\": \"{}\" }}",
            json_escape(name),
            json_escape(reason)
        );
        if let Err(e) = fs::write(perf_dir.join(format!("{name}.skip.json")), skip_json) {
            eprintln!("analyze: write skip json: {e}");
            return 2;
        }
        if let Err(e) = fs::write(
            perf_dir.join(format!("{name}.skip.txt")),
            format!("\n# perf skipped for {name} (NOT_IMPLEMENTED stub): {reason}\n"),
        ) {
            eprintln!("analyze: write skip txt: {e}");
            return 2;
        }
        println!("analyze: perf skipped for {name} (NOT_IMPLEMENTED stub)");
        return 0;
    }
    println!(
        "analyze: isolated perf for {name} only ({} passes of {} MiB)",
        args.perf_runs, args.perf_mib
    );
    let rec = checks::run_perf(&env, args.perf_mib * 1024 * 1024, args.perf_runs);
    if let Err(e) = fs::write(
        perf_dir.join(format!("{name}.record.json")),
        perf_engine_json(&rec),
    ) {
        eprintln!("analyze: write perf json: {e}");
        return 2;
    }
    if let Err(e) = fs::write(perf_dir.join(format!("{name}.txt")), perf_engine_txt(&rec)) {
        eprintln!("analyze: write perf txt: {e}");
        return 2;
    }
    println!(
        "  bytes_fed={} passes={} bytes_per_sec min={} median={} max={} rss_baseline_kb={} rss_after_kb={} rss_delta_kb={} vmhwm_kb={}",
        rec.bytes_fed,
        rec.runs.len(),
        opt_f64(rec.bytes_per_sec_min),
        opt_f64(rec.bytes_per_sec_median),
        opt_f64(rec.bytes_per_sec_max),
        opt_u64(rec.rss_baseline_kb),
        opt_u64(rec.rss_after_kb),
        opt_i64(rec.rss_delta_kb),
        opt_u64(rec.vmhwm_kb)
    );
    0
}

/// Merge the per-engine isolated-process perf artifacts into `perf.txt` /
/// `perf.json`. If no per-engine artifact exists the memory item is recorded
/// NOT_TESTED instead of publishing contaminated numbers.
fn merge_perf(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let mut record_json: Vec<String> = Vec::new();
    let mut skip_json: Vec<String> = Vec::new();
    let mut texts: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.ends_with(".record.json") {
                record_json.push(fs::read_to_string(&path)?);
            } else if name.ends_with(".skip.json") {
                skip_json.push(fs::read_to_string(&path)?);
            } else if name.ends_with(".txt") && name != "perf.txt" {
                texts.push((name, fs::read_to_string(&path)?));
            }
        }
    }
    record_json.sort();
    skip_json.sort();
    texts.sort_by(|a, b| a.0.cmp(&b.0));

    let mut txt = String::new();
    txt.push_str("Relayable TERM-002 — throughput / memory OBSERVATION\n");
    txt.push_str("===================================================\n");
    txt.push_str("This is an observation only. No threshold is asserted and no memory\n");
    txt.push_str("target is invented (full.md §0.5 / §12: measure before setting targets).\n");
    txt.push_str(
        "Each engine was measured in its own process (`analyze --perf-only <engine>`),\n\
         so RSS baseline/after/peak are per-engine and not cross-contaminated. Each\n\
         engine ran at least 3 passes; min/median/max throughput and every raw pass\n\
         are reported.\n",
    );
    if record_json.is_empty() && skip_json.is_empty() {
        txt.push_str(
            "\nNOT_TESTED: no per-engine perf artifact found in this directory. Per-engine\n\
             isolation was not achieved, so no memory number is published.\n",
        );
    }
    for (_, text) in &texts {
        txt.push_str(text);
    }
    fs::write(dir.join("perf.txt"), txt)?;

    let json = format!(
        "{{\n  \"observation\": true,\n  \"note\": \"per-engine isolated-process observation; no threshold asserted\",\n  \"records\": [\n{}\n  ],\n  \"skipped\": [\n{}\n  ]\n}}\n",
        record_json.join(",\n"),
        skip_json.join(",\n")
    );
    fs::write(dir.join("perf.json"), json)?;
    Ok(())
}

/// D1/D2(c)/D3 self-check over the *generated* results, run BEFORE any
/// artifact is written so a violating run never writes a `STATUS=PASS` file:
/// * no empty/placeholder `ACTUAL` value (see
///   [`checks::is_placeholder_actual`]);
/// * every cited source path exists on disk;
/// * every replayed-real-capture leg makes a real engine-state assertion.
fn self_check(repo_root: &Path, results: &[checks::CheckResult]) -> Vec<String> {
    let mut violations = Vec::new();
    for result in results {
        if result.kind == "replayed-real-capture"
            && !result
                .assertions
                .iter()
                .any(|a| a.name == "engine_adapter_implemented")
        {
            let has_engine_assertion = result
                .assertions
                .iter()
                .any(|a| a.name != "capture_processed" && a.status != Status::NotTested);
            if result.assertions.len() < 2 || !has_engine_assertion {
                violations.push(format!(
                    "{}/{}: replay leg does not assert engine state (assertions={}, engine_assertions={})",
                    result.engine,
                    result.check,
                    result.assertions.len(),
                    has_engine_assertion
                ));
            }
        }
        for source in &result.sources {
            let path = Path::new(&source.path);
            let exists = path.exists() || repo_root.join(&source.path).exists();
            if !exists {
                violations.push(format!(
                    "{}/{}: cited source path does not exist: {}",
                    result.engine, result.check, source.path
                ));
            }
        }
        for assertion in &result.assertions {
            let value = assertion.actual.trim();
            if checks::is_placeholder_actual(value) {
                violations.push(format!(
                    "{}/{}: assertion {} has empty/placeholder ACTUAL {value:?}",
                    result.engine, result.check, assertion.name
                ));
            }
        }
    }
    violations
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("analyze: {e}");
            std::process::exit(2);
        }
    };

    if let Some(engine_name) = args.perf_only.clone() {
        std::process::exit(run_perf_only(&args, &engine_name));
    }

    let fixtures_dir = args.results_dir.join("fixtures");
    let streams_dir = args.results_dir.join("streams");
    let checks_dir = args.results_dir.join("checks");
    let perf_dir = args.results_dir.join("perf");

    println!(
        "{}",
        relayable_terminal_engine_phase2_spike::version_banner()
    );
    println!("analyze: results_dir={}", args.results_dir.display());
    println!("analyze: repo_root={}", args.repo_root.display());

    if let Err(e) = fixtures::write_all(&fixtures_dir) {
        eprintln!("analyze: write fixtures: {e}");
        std::process::exit(2);
    }
    // D3: persist every engine-input payload so every cited sha256 is
    // recomputable from a real file under results/raw/fixtures/.
    if let Err(e) = checks::write_engine_inputs(&fixtures_dir) {
        eprintln!("analyze: write engine inputs: {e}");
        std::process::exit(2);
    }
    let fixture_list = fixtures::all();
    println!(
        "analyze: wrote {} fixtures + {} engine-input payloads",
        fixture_list.len(),
        checks::engine_inputs().len()
    );

    let streams = load_streams(&streams_dir);
    println!(
        "analyze: loaded {} real-program captures: {:?}",
        streams.len(),
        streams.iter().map(|s| s.name.as_str()).collect::<Vec<_>>()
    );

    let mut all_results = Vec::new();
    // Preserve the registry order: alacritty_terminal, vt100, avt, termwiz.
    for (name, factory) in engine::engines() {
        let version = engine::engine_version(name);
        let env = Env {
            engine: name,
            version,
            factory,
            fixtures: &fixture_list,
            streams: &streams,
            cols: args.cols,
            rows: args.rows,
            scrollback: args.scrollback,
            repo_root: args.repo_root.clone(),
        };
        all_results.extend(checks::run_all(&env));
    }

    // Run the self-check BEFORE writing any report, so a violating run never
    // emits a `STATUS=PASS` artifact.
    let violations = self_check(&args.repo_root, &all_results);
    if !violations.is_empty() {
        eprintln!(
            "\nSELF-CHECK FAILED ({} violation(s)); no per-check artifacts written:",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        std::process::exit(1);
    }

    for result in &all_results {
        match report::write_check(&checks_dir, result) {
            Ok(path) => println!(
                "  {:<26} {:<28} {:<8} {}",
                result.engine,
                result.check,
                result.status().as_str(),
                path.display()
            ),
            Err(e) => eprintln!("analyze: write check: {e}"),
        }
    }

    let summary_json = args.results_dir.join("summary.json");
    let summary_txt = args.results_dir.join("summary.txt");
    if let Err(e) = report::write_summary(&summary_json, &summary_txt, &all_results) {
        eprintln!("analyze: write summary: {e}");
        std::process::exit(2);
    }

    if args.perf {
        if let Err(e) = merge_perf(&perf_dir) {
            eprintln!("analyze: merge perf: {e}");
        }
    }

    let mut counts = std::collections::BTreeMap::new();
    for r in &all_results {
        *counts.entry(r.status().as_str()).or_insert(0usize) += 1;
    }
    println!("\nTALLY: {counts:?}");
    let failed: Vec<&str> = all_results
        .iter()
        .filter(|r| r.status() == Status::Fail)
        .map(|r| r.check.as_str())
        .collect();
    if failed.is_empty() {
        println!("VERDICT=PASS (no engine check FAILed; self-check clean)");
    } else {
        println!("VERDICT=FAIL failing checks: {failed:?}");
        std::process::exit(1);
    }
}
