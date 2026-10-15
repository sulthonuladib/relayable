# TERM-002 Phase 2 — Terminal Engine Validation: Results Report

Standalone engineering report for the engine-level validation spike that supports
**ADR-003** ("terminal engine and gpui artifact pin"). Audience: the Manager and the next
implementation agents (**UI-001** on the GPUI/rendering side, **CORE-001** on the server/PTY
side). Every claim below is traceable to a committed artifact under
`spikes/terminal-engine-phase2/results/raw/`; every number is copied from an artifact that a
real command produced on host `falcon` during the run recorded in the provenance header.
The harness was re-run from scratch for this report (see §1.4); no result here is inherited
from a prior author's session.

Repo path for this report: `spikes/terminal-engine-phase2/results/REPORT.md`.
Reproduction entry point: `spikes/terminal-engine-phase2/scripts/run_all.sh`.

---

## 1. Provenance

### 1.1 Toolchain and host

Source: `results/raw/run_all_meta.txt`, `results/raw/summary.txt`, and every per-check artifact
(all carry the same header fields).

| Field | Value |
|---|---|
| `rustc` | `rustc 1.98.1 (48a229cea 2026-09-01)`, host `x86_64-unknown-linux-gnu`, LLVM 22.1.8 |
| `cargo` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| hostname | `falcon` |
| uname | `Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux` |
| run window (UTC) | started `2026-10-15T20:12:05Z` (`run_all_meta.txt`); finalized `2026-10-15T20:12:36Z` (`run_all_exit.txt`) |
| TERM / COLORTERM | `xterm-256color` / `truecolor` (`run_all_meta.txt`) |

### 1.2 Code identity (no commit — untracked tree)

`git_head: a1091e58f32d363ceb2614ab4915329b5de6a869` appears in every artifact, and it is
**the BASE commit the spike branch was cut from — not the code revision that produced these
artifacts**. The phase-2 tree was untracked at run time (`git_status_short: ?? ./` in the
crate-relative header; `?? spikes/terminal-engine-phase2/` from the repo root), so `HEAD` does
not identify the code. The code-identity guarantee is the deterministic
`source_tree_sha256`:

```
source_tree_sha256 = cc5529d4584f054e531f0a0eca4a1981b3ba0d6172f5dd5e20e4047a9a9f521d
```

Stamped in `results/raw/run_all_meta.txt`, `results/raw/summary.txt`, `results/raw/summary.json`,
and every `results/raw/checks/*.txt` artifact. It is defined as the SHA-256 over the
`LC_ALL=C`-sorted, newline-joined list of `<relative_path>:<file_sha256>` for
`spikes/terminal-engine-phase2/{Cargo.toml, Cargo.lock, scripts/run_all.sh, src/**}`.

**Recompute** (this is the exact recipe in `README.md`, and the same algorithm is implemented by
`report::source_tree_sha256()` and by the `source_tree_sha256()` shell function in
`scripts/run_all.sh`):

```sh
cd <repo-root>
{
  for f in \
    spikes/terminal-engine-phase2/Cargo.toml \
    spikes/terminal-engine-phase2/Cargo.lock \
    spikes/terminal-engine-phase2/scripts/run_all.sh; do
    printf '%s:%s\n' "$f" "$(sha256sum -- "$f" | cut -d' ' -f1)"
  done
  find spikes/terminal-engine-phase2/src -type f -print0 \
    | xargs -0 -r sha256sum \
    | while read -r sum path; do printf '%s:%s\n' "$path" "$sum"; done
} | LC_ALL=C sort | sha256sum | cut -d' ' -f1
```

I ran this recipe independently on the fresh tree and it printed
`cc5529d4584f054e531f0a0eca4a1981b3ba0d6172f5dd5e20e4047a9a9f521d`, **equal to the stamped
value** in `run_all_meta.txt` / `summary.json`.

### 1.3 Exact dependency pins (as resolved in `Cargo.lock`)

From the `pinned_crates` block in `results/raw/run_all_meta.txt`, cross-checked against
`Cargo.lock`:

| Crate | Pin | Role |
|---|---|---|
| `portable-pty` | `0.9.0` | Real-PTY ownership (gating) |
| `alacritty_terminal` | `0.26.0` | Candidate engine: parser + grid/screen state (gating) |
| `vte` | `0.15.0` | Parser boundary; `alacritty_terminal 0.26.0` requires `vte ^0.15.0`, one shared copy |
| `vt100` | `0.16.2` | Comparison engine |
| `termwiz` | `0.23.3` | Comparison engine — deliberately unimplemented stub (see §3.4) |
| `avt` | `0.18.0` | Comparison engine |

No `[profile]` override; debug profile only (declared in `Cargo.toml`).

### 1.4 Harness commands and observed exit codes

I ran each gate directly from `spikes/terminal-engine-phase2/` and then ran the one-shot
harness; all output and exit codes are recorded verbatim.

| Command | Exit code | Notes |
|---|---|---|
| `cargo fmt --check` | `0` | no output |
| `cargo build` | `0` | |
| `cargo clippy --all-targets` | `0` | |
| `cargo test` | `0` | `42 passed; 0 failed; 0 ignored` (lib) + 0 in the two bins + 0 doc-tests |
| `./scripts/run_all.sh` | `0` | printed `OVERALL=PASS`; own exit code also written to `results/raw/run_all_exit.txt` as `exit_code=0` at `2026-10-15T20:12:36Z` |

Internal gate exit codes recorded by `run_all.sh` into `results/raw/run_all_meta.txt`
("end of run summary") and all `0`:

```
capture_failures=0
analyze_exit=0
perf_isolated_exit=0
matrix_generation_exit=0
gating_engine_not_tested_gate_exit=0
cargo_fmt_check_exit=0
cargo_build_exit=0
cargo_clippy_exit=0
cargo_test_exit=0
```

### 1.5 Fresh-run acceptance checks (all held)

* `OVERALL=PASS`; tallies `PASS: 77 / UNSUPPORTED: 55`; **0 FAIL, 0 NOT_TESTED**
  (`results/raw/summary.txt`, `results/raw/summary.json`).
* `alacritty_terminal = 32 PASS / 1 UNSUPPORTED`; the single UNSUPPORTED is exactly
  `bracketed_paste_payload` (`results/raw/compare/matrix.txt`, per-check artifact).
* `source_tree_sha256` recomputed = stamped (above).
* Five independent spot-checks on the fresh artifacts (below) all held.

Spot-verification of the fresh artifacts:

1. `results/raw/checks/alacritty_terminal__chunk_boundary_invariance.txt` — `STATUS=PASS`, and
   every `PASS` line carries the compared evidence (`plain_sha256`, `plain_len`, `cursor`,
   `cells`, `first_nonblank_row`/`last_nonblank_row`) **and** `changed_vs_pristine=true`; the two
   named expected no-ops are re-asserted as no-ops.
2. `results/raw/checks/alacritty_terminal__alt_screen_real.txt` — the check code sets the screen
   dump to `last_alt_snapshot` (the snapshot the `alt_screen_had_content` assertion was
   evaluated on; `src/checks.rs:2124`), and the artifact's screen dump contains real nvim
   content (`NVIM v0.12.5`, `:qa!`, `[No Name]`) plus `alt_screen_nonblank=true`.
3. `results/raw/checks/alacritty_terminal__tmux_real.txt` — asserts raw `1049h_count=1`,
   `1049l_count=1`, the engine alt flag transitions
   (`engine_alt_after_each_1049h=[true]`, `engine_alt_after_each_1049l=[false]`), and the marker
   (`marker_count=1 marker_offset=Some(681)`; alt snapshot present `["RELAYABLE_TMUX_MARKER"]`).
4. `results/raw/streams/tmux.cleanup.txt` — `kill_server_exit=0`, `verify_no_server_exit=1`,
   `leftover_server=none`, private socket only.
5. No `ACTUAL` line under `results/raw/checks/` is a placeholder (scanned 607 `ACTUAL` lines for
   `TODO/TBD/PLACEHOLDER/XXX/FIXME/<unset>/unknown/N/A` and empty values; no matches). `analyze`
   additionally performs this self-check **before** writing any per-check artifact and exits
   non-zero on a violation (README.md; `results/raw/checks/_analyze_stdout.txt`).

---

## 2. Method

### 2.1 End-to-end shape

Source: `scripts/run_all.sh`, `src/pty.rs`, `src/bin/capture.rs`, `src/bin/analyze.rs`,
`src/checks.rs`, `src/engine/*.rs`, `src/model.rs`.

1. **Provenance stamp.** `run_all.sh` writes `results/raw/run_all_meta.txt` (UTC, host, rustc/cargo,
   `git_head`, `source_tree_sha256`, pinned crate versions, tool presence) and registers an `EXIT`
   trap that writes the script's real exit code to `results/raw/run_all_exit.txt`.
2. **Build.** `cargo build` produces the two binaries `capture` and `analyze`; failure aborts with
   `exit 2`.
3. **Real-PTY captures.** `capture` opens a real `portable-pty 0.9.0` PTY
   (`PtySize { rows, cols }`), spawns a real program, schedules optional byte injection
   (`--send ms:hex`) and resize events (`--resize ms:ROWSxCOLS`), reads the master fd, and writes
   the exact captured bytes to `results/raw/streams/<name>.bin` plus `<name>.meta`
   (`kind=real-program`, exact byte count, SHA-256, child exit, slave tty, pid).
4. **Deterministic fixtures.** For cases that need a byte-exact deterministic input, `analyze`
   writes `results/raw/fixtures/<name>.{bin,meta}` with `kind=fixture` (and 3
   `kind=engine-input` payloads for the resize/scrollback/selection engine legs), each with a
   recomputable SHA-256.
5. **Engine adapters.** Each engine is adapted behind one neutral trait (`src/model.rs`
   `EngineAdapter { feed, resize, snapshot, implementation_status, capabilities }`) and produces a
   neutral `Snapshot`: visible cells with `(text, fg/bg, bold/italic/underline/reverse, OSC 8
   URI)`, cursor, mode flags (alt screen, bracketed paste, app cursor keys, mouse mode/SGR),
   title, event list/bell count, and `ScrollbackStats`. For `alacritty_terminal`, the adapter
   implements only `EventListener` and drives the pinned `vte::ansi::Processor`; `Term` itself
   implements `vte::ansi::Handler` (`notes/engine-api.md` §1.5).
6. **Checks.** `analyze` feeds each fixture/capture into each engine and asserts on
   grid/cursor/modes/history. Each `(engine, check)` writes one artifact
   `results/raw/checks/<engine>__<check>.txt` with provenance, cited inputs + SHA-256,
   `EXPECTED`/`ACTUAL`/`STATUS` per assertion, and a screen dump.
7. **Comparison matrix.** `run_all.sh` derives `results/raw/compare/matrix.{txt,json}` from
   `summary.json` (never hand-written).
8. **Per-engine perf observation.** `analyze --perf-only <engine>` runs once per engine in its own
   process, then the merged observation is written to `results/raw/perf/`.
9. **Quality gates** and the gating-engine `NOT_TESTED` gate run last; the script prints
   `OVERALL=PASS`/`OVERALL=FAIL` and exits accordingly.

### 2.2 Evidence labeling: `kind=`

Every cited input is labeled, and the check suite records the check `kind` in
`summary.json`/`matrix.txt` (132 rows = 33 checks × 4 engines): `fixture` (52),
`real-program` (40), `replayed-real-capture` (24), `engine` (16). Input `kind`s additionally
include `engine-input` (3 persisted payloads). The rule applied:

* **`fixture`** — a deterministic byte stream, used only where byte-exact determinism is required
  (SGR/OSC/mode/Unicode/mouse/alt-screen basics).
* **`real-program`** — a real program captured through a real PTY; used for the interactive/TUI
  claims (nvim, tmux, ls, seq, bell, cursor-key round trip, resize).
* **`replayed-real-capture`** — a real capture produced by phase 1 (`spikes/terminal-engine/`,
  read-only) replayed into the engine (`less`, `nvim`, `nvim_mouse`, `nvim_title`, `osc8_ls`,
  `top`); every citation shows the phase-1 source path and SHA-256.
* **`engine`** — an engine-level property exercised directly (resize/reflow, scrollback bound,
  selection, chunk-boundary invariance) using the 3 `engine-input` payloads.

Assertions are never satisfied by an engine that ignores its input: each non-no-op fixture also
asserts `changed_vs_pristine=true`, and the two named expected no-ops are re-asserted as no-ops.

---

## 3. Per-engine results

Tallies from `results/raw/summary.txt`/`summary.json`; the per-check statuses are the generated
matrix in `results/raw/compare/matrix.txt` (also `matrix.json`).

| Engine | Version | PASS | UNSUPPORTED | FAIL | NOT_TESTED | Artifact |
|---|---|---|---|---|---|---|
| `alacritty_terminal` | 0.26.0 (+ `vte` 0.15.0) | **32** | **1** | 0 | 0 | `summary.txt`, `compare/matrix.txt` |
| `vt100` | 0.16.2 | 28 | 5 | 0 | 0 | `summary.txt` |
| `avt` | 0.18.0 | 17 | 16 | 0 | 0 | `summary.txt` |
| `termwiz` (stub) | 0.23.3 | 0 | 33 | 0 | 0 | `summary.txt` |
| **Total (132)** | — | **77** | **55** | **0** | **0** | `summary.txt` |

### 3.1 Full check × engine status matrix (33 checks)

Reproduced from `results/raw/compare/matrix.txt`.

| # | check | kind | alacritty | vt100 | avt | termwiz |
|---|---|---|---|---|---|---|
| 1 | `vt_basics` | fixture | PASS | PASS | PASS | UNSUPPORTED |
| 2 | `alt_screen` | fixture | PASS | PASS | PASS | UNSUPPORTED |
| 3 | `osc_title` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 4 | `osc8_hyperlink` | fixture | PASS | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED |
| 5 | `bell` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 6 | `unicode_wide` | fixture | PASS | PASS | PASS | UNSUPPORTED |
| 7 | `unicode_combining` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 8 | `unicode_emoji` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 9 | `colors` | fixture | PASS | PASS | PASS | UNSUPPORTED |
| 10 | `cursor_keys` | fixture | PASS | PASS | PASS | UNSUPPORTED |
| 11 | `bracketed_paste_mode` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 12 | `bracketed_paste_payload` | fixture | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED |
| 13 | `mouse_modes` | fixture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 14 | `resize_reflow` | engine | PASS | UNSUPPORTED | PASS | UNSUPPORTED |
| 15 | `scrollback_bound` | engine | PASS | PASS | PASS | UNSUPPORTED |
| 16 | `selection` | engine | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 17 | `chunk_boundary_invariance` | engine | PASS | PASS | PASS | UNSUPPORTED |
| 18 | `alt_screen_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 19 | `osc_title_real` | real-program | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 20 | `osc8_real` | real-program | PASS | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED |
| 21 | `bell_real` | real-program | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 22 | `unicode_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 23 | `cursor_keys_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 24 | `mouse_modes_real` | real-program | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 25 | `resize_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 26 | `scrollback_bound_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 27 | `tmux_real` | real-program | PASS | PASS | PASS | UNSUPPORTED |
| 28 | `replay_less` | replayed-real-capture | PASS | PASS | PASS | UNSUPPORTED |
| 29 | `replay_nvim` | replayed-real-capture | PASS | PASS | PASS | UNSUPPORTED |
| 30 | `replay_nvim_mouse` | replayed-real-capture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 31 | `replay_nvim_title` | replayed-real-capture | PASS | PASS | UNSUPPORTED | UNSUPPORTED |
| 32 | `replay_osc8_ls` | replayed-real-capture | PASS | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED |
| 33 | `replay_top` | replayed-real-capture | PASS | PASS | PASS | UNSUPPORTED |

### 3.2 UNSUPPORTED reasons and classification

`alacritty_terminal` (1): `bracketed_paste_payload` — **out-of-scope / input-path property**, not
a capability gap. The output-only feed correctly parses `ESC` inside
`ESC[200~ .. ESC[201~` as control sequences (`ACTUAL engine parsed ESC[31m as SGR;
row0="abcZ"; cell(0,1).fg=Indexed(1)`). The "bytes as data" guarantee is the terminal→app
direction, owned by CORE-001; see §4 item 9.

`vt100 0.16.2` (5):

| Check | Reason | Class |
|---|---|---|
| `osc8_hyperlink`, `osc8_real`, `replay_osc8_ls` | vt100 0.16.2 does not implement OSC 8 | capability gap |
| `resize_reflow` | `Grid::set_size` clears row wrapping instead of reflowing logical lines | capability gap |
| `bracketed_paste_payload` | input-path property | out-of-scope |

`avt 0.18.0` (16):

| Check(s) | Reason | Class |
|---|---|---|
| `osc_title`, `osc_title_real`, `replay_nvim_title` | no OSC 0/2 handling (`parser::osc_put` is a no-op) | capability gap |
| `bell`, `bell_real` | no BEL action in `parser::Function` | capability gap |
| `osc8_hyperlink`, `osc8_real`, `replay_osc8_ls` | no OSC 8 | capability gap |
| `mouse_modes`, `mouse_modes_real`, `replay_nvim_mouse` | no 1000/1002/1003/1006 tracking (`DecMode` lists only 1/6/7/25/1047/1048/1049) | capability gap |
| `bracketed_paste_mode` | `DecMode` does not list 2004 | capability gap |
| `unicode_combining`, `unicode_emoji` | one char per cell; combining/VS16/ZWJ overwrite instead of attach | capability gap |
| `selection` | no selection-range API | capability gap |
| `bracketed_paste_payload` | input-path property | out-of-scope |

`termwiz 0.23.3` (33): every check is UNSUPPORTED because the adapter is deliberately
`NOT_IMPLEMENTED`. Source-cited exclusion reason (verbatim in every artifact and in
`compare/README.txt`): termwiz has no byte-stream → screen-state emulator —
`escape::parser::Parser::parse` (`termwiz-0.23.3/src/escape/parser/mod.rs:86`) only decodes bytes
into `escape::Action` (`src/escape/mod.rs:28`), and nothing maps `Action` → `surface::Surface`
(`src/surface/mod.rs:105` has no `Action` reference); meanwhile
`terminal::UnixTerminal::new` opens `/dev/tty` (`src/terminal/unix.rs:266`) and captures the tty
termios via `Termios::from_fd` (`src/terminal/unix.rs:171`), so it cannot be driven headlessly
from captured bytes. This is an explicit "not wired yet" exclusion, not a behavioural failure —
`termwiz` is never reported PASS or FAIL.

### 3.3 Why the gating engine declaration cannot hide a result

`alacritty_terminal` never overrides `capabilities()`, so it keeps the `Capabilities::ALL`
default; no alacritty check can be suppressed by a capability declaration. A check whose required
capability is declared absent becomes UNSUPPORTED (capability gap) with the missing capability
named; a declared-present capability that disagrees with the engine's behaviour still produces a
FAIL. This is documented in `results/raw/compare/README.txt` and enforced by unit tests
(`checks::catalog_tests::*`).

### 3.4 Deliberately-unimplemented `termwiz`

See §3.2. `results/raw/perf/termwiz.skip.{txt,json}` records the perf skip with the same reason;
`src/engine/termwiz.rs` is a compiling stub whose `implementation_status()` is
`NotImplemented`.

---

## 4. Assignment coverage matrix (the 13 required items)

Each row maps a required item to the artifact(s) that evidence it and to a coverage verdict.
"COVERED" means a committed artifact asserts the item on `alacritty_terminal`; "PARTIAL" and
"GAP" carry a one-line reason.

| # | Required item | Evidence artifact(s) | Verdict |
|---|---|---|---|
| 1 | VT/ANSI basics | `checks/alacritty_terminal__vt_basics.txt`, `fixtures/vt_basics.bin` (+ `chunk_boundary_invariance`) | **COVERED** |
| 2 | Alt screen with `vim`/`less` | `checks/alacritty_terminal__alt_screen_real.txt` (real nvim), `replay_less.txt`, `tmux_real.txt` | **PARTIAL** — `vim` is absent on this host; `nvim v0.12.5` is used instead (`run_all_meta.txt`), `less` is replayed from phase 1. |
| 3 | OSC 0/2 title | `checks/alacritty_terminal__osc_title.txt`, `osc_title_real.txt`, `replay_nvim_title.txt` | **COVERED** |
| 4 | OSC 8 hyperlinks | `checks/alacritty_terminal__osc8_hyperlink.txt`, `osc8_real.txt`, `replay_osc8_ls.txt` | **COVERED** (vt100/avt lack OSC 8 → UNSUPPORTED; see §3.2) |
| 5 | Bell | `checks/alacritty_terminal__bell.txt`, `bell_real.txt` | **COVERED** |
| 6 | Unicode + wide glyphs incl. CJK/emoji | `checks/alacritty_terminal__unicode_wide.txt`, `unicode_combining.txt`, `unicode_emoji.txt`, `unicode_real.txt` | **COVERED** |
| 7 | 256/truecolor | `checks/alacritty_terminal__colors.txt` | **COVERED** — indexed and truecolor values asserted; palette→RGB resolution is UNVERIFIED (see §8) |
| 8 | Cursor keys/modes | `checks/alacritty_terminal__cursor_keys.txt`, `cursor_keys_real.txt` | **COVERED** |
| 9 | Bracketed paste bytes as data | `checks/alacritty_terminal__bracketed_paste_mode.txt` (mode asserted); `bracketed_paste_payload.txt` (UNSUPPORTED) | **PARTIAL / GAP by design** — the `2004h/l` **mode** is asserted; the **payload** guarantee is an input-path (terminal→app) property that cannot be asserted from an output-only feed and belongs to CORE-001. On the output direction, `ESC` inside the payload is *correctly* parsed as a control sequence (`ESC[31m` → SGR), i.e. the emulator must not treat it as literal text. |
| 10 | Mouse-mode enable sequences | `checks/alacritty_terminal__mouse_modes.txt`, `mouse_modes_real.txt`, `replay_nvim_mouse.txt` | **COVERED** |
| 11 | Resize/reflow | `checks/alacritty_terminal__resize_reflow.txt` (engine), `resize_real.txt` (real PTY + SIGWINCH) | **COVERED** |
| 12 | Scrollback bound | `checks/alacritty_terminal__scrollback_bound.txt`, `scrollback_bound_real.txt` | **COVERED** |
| 13 | Rough RSS/throughput | `perf/perf.txt`, `perf/alacritty_terminal.txt`, `checks/_perf_alacritty_terminal.txt` | **COVERED** — observation only, no threshold asserted |

Additional precision the reader should not lose:

* **Bracketed paste** is split as above. The mode check asserts `bracketed_paste` is `true` after
  `?2004h` and `false` after `?2004l`; the payload check is UNSUPPORTED for **every** engine by
  design.
* **Selection** (`checks/alacritty_terminal__selection.txt`,
  `checks/vt100__selection.txt`) is asserted for alacritty and vt100, but with **golden-value
  expectations derived from each engine's own output** (alacritty: `"beta gamma\ndelta
  epsilon\nzeta"`; vt100: `"beta gamma\ndelta epsilon\nzet"`). Because the expected string was
  taken from the engine, the check cannot detect wrong-but-stable range semantics; it only
  detects drift. `avt` has no selection API → UNSUPPORTED.
* **`vim` absent / `htop` absent.** `run_all_meta.txt` records
  `vim_presence: absent (nvim NVIM v0.12.5 is used instead)` and `htop_presence: absent`. `less`,
  `top`, `tmux`, `ls`, `seq`, `/bin/sh`, `stty` are present.
* **tmux** is a real private-socket capture with a real assertion (§7 and
  `checks/alacritty_terminal__tmux_real.txt`), not a recorded skip.
* **"Rough RSS"** is isolated per-engine process measurement, observation only, no threshold
  (`perf/perf.txt`).
* The §5.4 bullets "how state can be rendered on Android later" and shell/`vim`/`htop`
  compatibility are only **partially** covered: this spike proves the engine + neutral snapshot
  (the rendering input), not an Android renderer; `vim`/`htop` are absent; `tmux` is covered.

---

## 5. Measured observations (real numbers only)

### 5.1 Scrollback bound

Artifacts: `checks/alacritty_terminal__scrollback_bound.txt` (deterministic engine leg) and
`checks/alacritty_terminal__scrollback_bound_real.txt` (real `seq 1 20000`).

| Observation | Bound | Retained | Oldest/newest evidence |
|---|---|---|---|
| Engine fixture leg | 1000 | exactly 1000 (`history_len=1000`, `retained_lines=1000`) | oldest retained `"line 0477"`; newest visible `"line 1499"`; `"line 0000"` absent from visible and history |
| Real `seq 1 20000` | 1000 | 1000 (`retained_lines=1000`, `<= 1000`) | newest visible `"20000"`; first visible non-blank row `Some((0, "19978"))`; the first line `"1"` absent |

Configured bound is `scrolling_history = 1000` (`fixtures/engine_scrollback_input.meta`,
`perf/*`).

### 5.2 Throughput and per-engine RSS (isolated processes)

Artifacts: `perf/perf.txt`, `perf/perf.json`, `perf/<engine>.txt`, `checks/_perf_<engine>.txt`.
Each engine ran in its own `analyze --perf-only` process (so RSS is not cross-contaminated),
inputs `bytes_fed=8,388,632`, `lines_produced=149,797`, `cols=80 rows=24 scrolling_history=1000`,
`passes=3`. Observation only — no threshold.

| Engine | bytes/s min | bytes/s median | bytes/s max | RSS baseline | RSS after | RSS delta | vmhwm |
|---|---|---|---|---|---|---|---|
| `alacritty_terminal 0.26.0` | 14,204,097.699 | 14,267,095.787 | 14,687,419.127 | 12,480 kB | 14,476 kB | **1,996 kB** | 14,476 kB |
| `avt 0.18.0` | 16,895,429.145 | 16,942,944.171 | 17,497,790.867 | 11,976 kB | 15,176 kB | 3,200 kB | 15,176 kB |
| `vt100 0.16.2` | 6,909,884.787 | 6,915,145.655 | 6,941,463.988 | 12,204 kB | 14,896 kB | 2,692 kB | 14,896 kB |
| `termwiz 0.23.3` | — | — | — | — | — | — | skipped (stub) |

Per-pass detail is in `perf/<engine>.txt`; e.g. alacritty pass 1
`elapsed_ms=571.144 bytes_per_sec=14687419 rss_delta_kb=1988 vmhwm_kb=14476`, passes 2–3 show
`rss_delta_kb=0`. This is the first data point for CORE-001's scrollback-bound decision
(§10).

### 5.3 Resize (real PTY + SIGWINCH) and reflow

Artifacts: `checks/alacritty_terminal__resize_real.txt`,
`results/raw/streams/resize_winch.{bin,meta,summary.txt}`,
`checks/alacritty_terminal__resize_reflow.txt`.

* Real PTY: pre-size `80x24`, resized to `30x100` at ~1000 ms; the child's `trap ... WINCH`
  handler printed `WINCHED` and `stty size` output `100 30`; engine post-resize grid
  `cols=30 rows=100` (assertions `child_reported_new_size=true`,
  `engine_grid_cols_after_resize=30`, `engine_grid_rows_after_resize=100`).
* Engine reflow leg: `20x10` → `40x10`; row reflow asserted
  (`row0="A"×40`, `row1="A"×40`, `row2="A"×20`, `row3=""`).

### 5.4 Chunk-boundary invariance

Artifact: `checks/alacritty_terminal__chunk_boundary_invariance.txt`. 23 fixtures are fed twice
(one slice vs one byte at a time) and the full neutral snapshot is compared; all `same=true`, all
non-no-op fixtures `changed_vs_pristine=true`, and the two expected no-ops
(`cursor_keys_off`, `bracketed_paste_off`) are no-ops by declaration. Example compared evidence:
`plain_sha256=03b2edb491dbc66350a97d81d864cb5b8b9f20a1a1b347109d693477e28783c5 plain_len=49
cursor=(row=5,col=3,visible=true) cells=1920 first_nonblank_row=Some((0, "AB"))
last_nonblank_row=Some((7, "         Z"))`.

### 5.5 Real-program captures available to the checks

`results/raw/checks/_analyze_stdout.txt`: `loaded 10 real-program captures: ["bell",
"cursor_keys", "nvim_alt", "nvim_mouse", "nvim_title", "osc8_ls", "resize_winch", "seq_20000",
"tmux", "unicode"]`. Phase-1 replay sources: `spikes/terminal-engine/results/raw/{less,nvim,
nvim_mouse,nvim_title,osc8_ls,top}.bin` (read-only).

---

## 6. `gpui = "=0.2.2"` pin sub-result

All facts below come from `results/raw/deps/gpui_*` and `results/raw/gpui-followup/*`; no gpui
binary was ever run.

* **Resolves.** `cargo fetch` resolves `gpui = "=0.2.2"` from crates.io with **exit 0**
  (`deps/gpui_cargo_fetch.txt`, `Locking 708 packages`).
* **Does not compile.** `cargo check` **FAILS with exit 101**
  (`deps/gpui_cargo_check.txt`). Exact error:

  ```
  error[E0425]: cannot find value `ENOATTR` in crate `::libc`
     --> .../xattr-0.2.3/src/sys/mod.rs:12:56
      |
   12 |               pub const ENOATTR: ::libc::c_int = ::libc::ENOATTR;
      |                                                          ^^^^^^^
  ...
  error: could not compile `xattr` (lib) due to 1 previous error
  ```

* **Dependency chain** (confirmed by `cargo tree -i xattr`,
  `results/raw/gpui-followup/tree_invert_xattr.txt`):

  ```
  xattr v0.2.3
  └── zed-async-tar v0.5.0-zed
      └── gpui_http_client v0.2.2
          └── gpui v0.2.2
  ```

  i.e. `gpui 0.2.2 → gpui_http_client 0.2.2 → zed-async-tar 0.5.0-zed → xattr 0.2.3 → libc
  0.2.190` (`lock_entries.txt`: xattr `0.2.3`, libc `0.2.190`, exactly one libc).
* **Root cause.** `xattr-0.2.3/src/sys/mod.rs:12` references `libc::ENOATTR` unconditionally for
  Linux; libc 0.2.190 (released 2026-10-02) removed the long-deprecated `ENOATTR` (breaking change
  in its changelog). xattr 0.2.3's manifest allows `libc = "0.2"` with no upper bound, so Cargo
  selects 0.2.190 and the build breaks.
* **Feature matrix does not help.** A 7-variant matrix (root `--no-default-features`, gpui
  `default-features=false`, x11-only, wayland-only, `-p gpui_http_client`, isolated `-p xattr`,
  and one cargo-CLI-invalid isolated variant) **all exit 101 at the same error**
  (`gpui-followup/matrix_summary.txt`; `matrix_x11_only.txt`, `matrix_wayland_only.txt`, …).
  gpui always depends on `gpui_http_client`, which always enables `zed-async-tar`'s default
  `xattr` feature.
* **Cold check.** A true cold `cargo check --locked` with a fresh empty target dir: **exit 101,
  20.8 s wall, 392 crates attempted (146 `Compiling` + 246 `Checking`), 20 CPUs**
  (`gpui-followup/matrix_cold_full_check.txt`; `DIAGNOSIS.md`). It aborts at `xattr` before
  `xkbcommon` or `blade-graphics` (or even `zed-async-tar`/`gpui_http_client`) are ever reached;
  the Linux display backend is therefore **UNPROVEN**.
* **No newer pin available.** crates.io latest `gpui` is `0.2.2` (`gpui-followup/search_gpui.txt`);
  `zed-async-tar` requires `xattr = "0.2"` (`registry_zed_async_tar_manifest.txt`), and there is
  no `0.2.4` (`update_dryrun_xattr.txt`: `Locking 0 packages`; `index_xattr_versions.txt`: 0.2.0
  … 0.2.3 then 1.x).
* **Candidate remedies (NOT applied).** (a) hold libc below 0.2.190, e.g.
  `cargo update -p libc --precise 0.2.189` (0.2.189 defines the Linux `ENOATTR`); or
  (b) `[patch.crates-io]` `xattr` to a 0.2.3 fork that drops the `ENOATTR` const (mirrors the
  real 1.x fix). Both change a pin/workspace dependency and are an ADR/Manager decision, so
  **neither was applied** in this spike.

No window/run proof exists for gpui; that work is deferred to UI-001 (§10).

---

## 7. Verdict

The verdict below is the lead's technical decision, reproduced faithfully.

* **`alacritty_terminal 0.26.0` + `vte 0.15.0` (parser/grid) + `portable-pty 0.9.0`: ACCEPT.**
  The engine-level spike passes on real PTYs with **32 PASS / 1 documented UNSUPPORTED / 0 FAIL /
  0 NOT_TESTED**. All 13 assigned items were exercised and have committed artifacts; **11 of
  the 13 are fully asserted**. Two carry deliberate, documented gaps that the lead accepts:
  **item 2** uses `nvim v0.12.5` + `less` because `vim` is not installed on this host (a
  substitution, not a missing capability), and **item 9**'s bracketed-paste "bytes as data"
  semantics is an input-path (terminal→app) property that an output-feed engine spike cannot
  assert — the mode toggle itself **is** asserted (`bracketed_paste_mode`) and the payload leg is
  recorded `UNSUPPORTED` with its reason. **Item 13**'s RSS figures are per-engine isolated
  observations only, with no threshold asserted.
* **`gpui = "=0.2.2"`: ACCEPT WITH CAVEATS — not build-verified.** It resolves but does not
  compile on this host. ADR-003 should be amended to record the blocker and a mandatory remedy
  before UI-001 relies on the pin — **not re-pinned**, because 0.2.2 is the latest published
  `gpui`.
* **Net:** accept ADR-003's engine pins as-is; accept the gpui pin only with the recorded caveat
  and remedy. **No engine change is warranted.** `vt100 0.16.2` lacks OSC 8 and has no resize
  reflow, and `avt 0.18.0` lacks title/mouse/bracketed-paste/OSC 8, so both are strictly weaker
  for §5.4 coverage; they served only as sanity-check comparisons.

---

## 8. Caveats and what this report does NOT establish

Stated explicitly and without softening:

* **vte is not independently exercised.** `vte 0.15.0` is reached through
  `alacritty_terminal`'s re-export (`alacritty_terminal::vte`), so it is not driven as a separate
  parser boundary; the pin is validated only as part of the alacritty adapter.
* **Palette → RGB is UNVERIFIED.** `Term::colors()` is all-`None` without a frontend, so the
  engine cannot resolve named/indexed colours to RGB in this harness. The colors check asserts the
  engine-neutral `Indexed(n)`/`Rgb(r,g,b)` values only; UI-001 must supply the palette. Recorded
  in `checks/alacritty_terminal__colors.txt` and `checks/vt100__colors.txt`.
* **Mouse model approximates.** The neutral model has no `Press` vs `PressRelease` distinction, so
  vt100's DECSET 9 (`Press`) and DECSET 1000 (`PressRelease`) both collapse to `MouseMode::Click`
  (documented in `compare/README.txt`).
* **Bracketed-paste payload/input semantics are untested** (`bracketed_paste_payload` UNSUPPORTED);
  belongs to CORE-001.
* **Selection is golden-value tested only** (expected values taken from each engine's own output;
  cannot detect wrong-but-stable range semantics).
* **No Android rendering and no GPU/renderer path.** The spike proves a neutral `Snapshot`, not a
  renderer.
* **§5.4 partials.** "How state can be rendered on Android later" is not tested; shell/`vim`/`htop`
  compatibility is partial (`vim`, `htop` absent; `tmux` covered).
* **Wire protocol untouched.** The `TerminalOutput` / `TerminalInput` / `TerminalResize` /
  `TerminalSnapshot` messages of `docs/architecture/interface-contract.md` §5 were **not
  exercised** (no protocol work in this spike), and the mapping from the harness's neutral
  `Snapshot` to `TerminalSnapshot` (`cols`/`rows`/`screen_bytes`/`scrollback_bytes`) remains for
  CORE-001.
* **Provenance cannot be tied to a git commit.** The phase-2 tree was untracked (`HEAD` is the
  base commit `a1091e58`); `source_tree_sha256` is the code-identity guarantee.
* **gpui remains unbuilt and unrun** (§6); no window proof exists.

---

## 9. Independent verification

A separate **read-only verifier** audited the phase-2 code and evidence during internal
development; its findings (tracked internally as **C1–C11**) were fixed and the fixes re-verified
by re-running the harness. This is **INTERNAL team verification — not user or supervisor
approval**. **REVIEW-001 / Manager review is still outstanding.**

Surviving caveats that verification did not (and cannot) remove, and which are carried forward
above:

* selection expectations are golden values taken from each engine's own output (drift detection
  only);
* `termwiz` is unasserted by design (a documented `NOT_IMPLEMENTED` stub, never PASS/FAIL);
* provenance is not commit-tied because the tree was untracked at run time.

No verification claim here should be read as acceptance of the underlying engine/product
decision; that is the Manager's call.

---

## 10. Remaining work

**UI-001**

1. Resolve the `gpui` / `xattr` blocker (§6), then produce the minimal Linux window
   **build + run** proof (this spike never built or ran a gpui binary).
2. Render from the harness's neutral `Snapshot` (`src/model.rs`) rather than engine-specific
   types.
3. Supply the color palette (named/indexed → RGB) that the engine cannot resolve headlessly.
4. Visual resize/reflow + selection check on the rendered surface.

**CORE-001**

1. Server-owned PTY integration using `portable-pty 0.9.0` (the harness proves the ownership
   shape).
2. Credit-based bounded output buffering + detach semantics (`full.md` §6.1/§11), and
   snapshot-recovery semantics (`TerminalSnapshot`).
3. Input-path bracketed paste (`bracketed_paste_payload`, the UNSUPPORTED item here).
4. The scrollback bound as a measured decision — first data point: bound 1000, exactly 1000
   retained, with observed per-engine RSS delta (alacritty **+1,996 kB**, vt100 +2,692 kB, avt
   +3,200 kB; `perf/perf.txt`).

---

## 11. Raw evidence index

Everything under `spikes/terminal-engine-phase2/results/raw/`:

| Path | What it is |
|---|---|
| `run_all_meta.txt` | Provenance + pinned crate versions + tool presence + end-of-run gate exit codes. |
| `run_all_exit.txt` | The harness's real exit code (`exit_code=0`) stamped by its `EXIT` trap. |
| `summary.txt` / `summary.json` | Provenance, the 132 per-check statuses, tallies, engine implementation status. |
| `checks/*.txt` | One artifact per `(engine, check)`: provenance, cited inputs + SHA-256, `EXPECTED`/`ACTUAL`/`STATUS`, screen dump, notes. Includes `_analyze_stdout.txt` (self-check + run log), `_harness_build.txt`, `_perf_<engine>.txt`, and the four quality-gate captures `cargo_{fmt_check,build,clippy,test}.txt`. |
| `streams/*.{bin,meta,summary.txt}` | Raw real-PTY captures (`kind=real-program`) with exact bytes, SHA-256, child exit, PTY size. |
| `streams/tmux.cleanup.txt` | Private-socket tmux cleanup: `kill-server` exit 0 and the leftover-server check. |
| `fixtures/*.{bin,meta}` | Deterministic byte streams (`kind=fixture`, 23) and persisted engine inputs (`kind=engine-input`, 3), each with SHA-256. |
| `perf/*.txt`, `perf/*.json` | Per-engine isolated throughput/RSS observation (3 passes, min/median/max) and the `termwiz.skip`. |
| `compare/matrix.{txt,json}` | Generated per-check × per-engine matrix; `compare/README.txt` explains the capability model and every UNSUPPORTED reason. |
| `deps/*` | Phase-A dependency probe (fetch/build/clippy/fmt/metadata, gpui fetch/check/tree, display-stack and platform greps, Linux lib probe). Preserved by `run_all.sh`. |
| `gpui-followup/*` | The gpui `=0.2.2` compile-blocker diagnosis: dependency chain, root cause, 7-variant feature matrix, cold check, registry evidence, and `DIAGNOSIS.md`. Preserved by `run_all.sh`. |
