# ADR-003 / terminal-engine spike — live re-run evidence

- Reviewer role: independent evidence gathering (REVIEW-001 prep)
- Repo/worktree: `/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`
- `HEAD` at time of run: `a1091e58f32d363ceb2614ab4915329b5de6a869` (`git rev-parse HEAD`)
- Outside git state (2026-10-15):
  - `rustc 1.98.1 (48a229cea 2026-09-01)`
  - `cargo 1.98.1 (797e8a9bc 2026-08-05)`
  - `Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux`
  - `date -u` → `Thu Oct 15 06:23:38 PM UTC 2026`
- `$SPIKE` = `/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass/spikes/terminal-engine`
- No tracked file was modified. `git status --porcelain` after all runs shows only
  `?? docs/reviews/` (this untracked review directory). `git diff --stat` is empty.
- `scripts/run.sh` was **not** executed (it regenerates tracked `results/raw/*`).
- Only writable output: this file. Scratch crate lives under `/tmp/adr003-resolve/`.

---

## 1. `cargo build`

Command (cwd `$SPIKE`): `cargo build`
Exit code: **0**

```
   Compiling cfg_aliases v0.2.2
   Compiling libc v0.2.190
   Compiling cfg-if v1.0.5
   Compiling bitflags v2.13.2
   Compiling nix v0.31.3
   Compiling relayable-terminal-engine-spike v0.0.0 (/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass/spikes/terminal-engine)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.92s
```

Warnings: **none** (no `warning:` lines in stdout+stderr).
(Fresh build: dependencies compiled because the worktree `target/` was empty at start.)

## 2. `cargo clippy --all-targets`

Command (cwd `$SPIKE`): `cargo clippy --all-targets`
Exit code: **0**

```
    Checking cfg-if v1.0.5
    Checking bitflags v2.13.2
    Checking libc v0.2.190
    Checking nix v0.31.3
    Checking relayable-terminal-engine-spike v0.0.0 (/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass/spikes/terminal-engine)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.40s
```

Warnings/errors: **none** (no `warning:`/`error:` lines in stdout+stderr).

## 3. `cargo fmt --check`

Command (cwd `$SPIKE`): `cargo fmt --check`
Exit code: **0**

```
(no output — empty stdout and stderr)
```

## 4. Real-PTY SIGWINCH probe — live vs committed

Command (cwd `$SPIKE`): `./target/debug/pty_size`
Exit code: **0**; stderr empty.

Live stdout (verbatim):

```
== pty_size: real PTY window-size / SIGWINCH probe ==
TERM=xterm-256color
REQUESTED_INITIAL_WINSIZE=24 rows x 80 cols
CHILD_PID=3342558
SLAVE_DEVICE=/dev/pts/15
CHILD_PS_INITIAL=3342558 3342557 3342558 3342558 3342558 Ss+  sh
MASTER_WINSIZE_INITIAL=24 80
CHILD_STTY_SIZE_INITIAL=24 80
SET_WINSIZE_CALL=ioctl(master, TIOCSWINSZ, {ws_row:40, ws_col:120})
SET_WINSIZE_RC=0
CHILD_PS_AFTER_RESIZE=3342558 3342557 3342558 3342558 3342558 Ss+  sh
MASTER_WINSIZE_AFTER=40 120
CHILD_STTY_SIZE_AFTER=40 120
SIGWINCH_OBSERVED=true
CHILD_EXIT_CODE=Some(0)
CHILD_EXIT_SUCCESS=true

VERDICT=PASS

== raw child output (caret-escaped, 52 bytes) ==
INITIAL_STTY_SIZE=24 80^M^JSIGWINCH_STTY_SIZE=40 120^M^J
```

Committed `results/raw/pty_size.txt` body is byte-identical except for process/host-specific
values (`CHILD_PID`, `SLAVE_DEVICE`, `CHILD_PS_*`, and the `# ---- provenance ----` header).

Reproduction check against the requested fields:

| Field | Committed | Live | Reproduced? |
|---|---|---|---|
| `MASTER_WINSIZE_INITIAL=24 80` | yes | yes | **YES** |
| `SET_WINSIZE_RC=0` | yes | yes | **YES** |
| `CHILD_STTY_SIZE_AFTER=40 120` | yes | yes | **YES** |
| `SIGWINCH_OBSERVED=true` | yes | yes | **YES** |
| `VERDICT=PASS` | yes | yes | **YES** |

Result: **the live run reproduces all five requested fields exactly.** The probe is
self-contained (no external program dependency beyond `sh`/`stty`) and deterministic.

## 5. `esc_selftest` — live vs committed

Command (cwd `$SPIKE`): `./target/debug/esc_selftest`
Exit code: **0**; stderr empty.

Full live stdout (identical to committed `results/raw/esc_selftest.txt` body, modulo the
provenance header):

```
== esc_selftest: detector fixtures ==
SCANNER=relayable_terminal_engine_spike::esc::scan
SELFTEST alt_screen_1049_enter = PASS bytes=[1b 5b 3f 31 30 34 39 68] offset=Some(0)
SELFTEST alt_screen_1049_leave = PASS bytes=[1b 5b 3f 31 30 34 39 6c] offset=Some(0)
SELFTEST alt_screen_47_enter = PASS bytes=[1b 5b 3f 34 37 68] offset=Some(0)
SELFTEST alt_screen_47_leave = PASS bytes=[1b 5b 3f 34 37 6c] offset=Some(0)
SELFTEST alt_screen_1047_enter = PASS bytes=[1b 5b 3f 31 30 34 37 68] offset=Some(0)
SELFTEST alt_screen_1047_leave = PASS bytes=[1b 5b 3f 31 30 34 37 6c] offset=Some(0)
SELFTEST mouse_1000_enable = PASS bytes=[1b 5b 3f 31 30 30 30 68] offset=Some(0)
SELFTEST mouse_1000_disable = PASS bytes=[1b 5b 3f 31 30 30 30 6c] offset=Some(0)
SELFTEST mouse_1002_enable = PASS bytes=[1b 5b 3f 31 30 30 32 68] offset=Some(0)
SELFTEST mouse_1002_disable = PASS bytes=[1b 5b 3f 31 30 30 32 6c] offset=Some(0)
SELFTEST mouse_1003_enable = PASS bytes=[1b 5b 3f 31 30 30 33 68] offset=Some(0)
SELFTEST mouse_1003_disable = PASS bytes=[1b 5b 3f 31 30 30 33 6c] offset=Some(0)
SELFTEST mouse_1005_enable = PASS bytes=[1b 5b 3f 31 30 30 35 68] offset=Some(0)
SELFTEST mouse_1005_disable = PASS bytes=[1b 5b 3f 31 30 30 35 6c] offset=Some(0)
SELFTEST mouse_1006_enable = PASS bytes=[1b 5b 3f 31 30 30 36 68] offset=Some(0)
SELFTEST mouse_1006_disable = PASS bytes=[1b 5b 3f 31 30 30 36 6c] offset=Some(0)
SELFTEST mouse_1015_enable = PASS bytes=[1b 5b 3f 31 30 31 35 68] offset=Some(0)
SELFTEST mouse_1015_disable = PASS bytes=[1b 5b 3f 31 30 31 35 6c] offset=Some(0)
SELFTEST bracketed_paste_enable = PASS bytes=[1b 5b 3f 32 30 30 34 68] offset=Some(0)
SELFTEST bracketed_paste_disable = PASS bytes=[1b 5b 3f 32 30 30 34 6c] offset=Some(0)
SELFTEST osc_title = PASS bytes=[1b 5d 30 3b 68 65 6c 6c 6f 07] offset=Some(0)
SELFTEST osc_title = PASS bytes=[1b 5d 32 3b 68 65 6c 6c 6f 1b 5c] offset=Some(0)
SELFTEST osc_hyperlink = PASS bytes=[1b 5d 38 3b 3b 68 74 74 70 73 3a 2f 2f 65 78 61 6d 70 6c 65 2e 63 6f 6d 1b 5c 74 65 78 74] offset=Some(0)
SELFTEST osc_color = PASS bytes=[1b 5d 31 31 3b 3f 07] offset=Some(0)
SELFTEST cursor_hide = PASS bytes=[1b 5b 3f 32 35 6c] offset=Some(0)
SELFTEST cursor_show = PASS bytes=[1b 5b 3f 32 35 68] offset=Some(0)
SELFTEST decscusr = PASS bytes=[1b 5b 31 20 71] offset=Some(0)
SELFTEST sgr = PASS bytes=[1b 5b 33 31 6d] offset=Some(0)
SELFTEST sgr = PASS bytes=[1b 5b 30 3b 31 3b 33 38 3b 35 3b 31 39 36 6d] offset=Some(0)
SELFTEST combined_1002_1006 = PASS got=["mouse_1002_enable", "mouse_1006_enable"]
SELFTEST negative_control = PASS got=["csi_cursor_or_erase"]
SELFTEST_FAILURES=0
VERDICT=PASS
```

Result: **identical to the committed `results/raw/esc_selftest.txt`** (31 PASS fixture lines +
`SELFTEST_FAILURES=0` + `VERDICT=PASS`). This is the detector self-test, i.e. it validates the
byte-scanning fixture logic, not an engine.

## 6. Committed-evidence integrity / provenance

Every file under `results/raw/` (39 files: 6 `.bin`, 6 `.bin.meta`, 7 `.esc`, 7 `.seq`,
7 `.summary.txt`, `esc_selftest.txt`, `pty_size.txt`, `run_meta.txt`, `tools_available.txt`,
3 `scrollback_*.txt`, and 3 `checks/*`) carries the same 9-line provenance block. Verbatim,
the header of `results/raw/pty_size.txt`:

```
# ---- provenance ----
# command: ./target/debug/pty_size
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:23Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
```

Requested files verbatim:

`results/raw/checks/cargo_build.txt`:
```
# command: cargo build
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:22Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```

`results/raw/checks/cargo_clippy.txt`:
```
# command: cargo clippy --all-targets
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:22Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
    Checking relayable-terminal-engine-spike v0.0.0 (/home/sulthonuladib/.herdr/worktrees/relayable/research-term-001-adr-003/spikes/terminal-engine)
```

`results/raw/checks/cargo_fmt_check.txt`:
```
# command: cargo fmt --check
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:22Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
(empty body)
```

`results/raw/run_meta.txt`:
```
# command: ./scripts/run.sh (aggregate summary)
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:39Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
aggregate_failures=0
```

Sample `*.summary.txt` — `results/raw/less.summary.txt` (first 10 lines):
```
# ---- provenance ----
# command: ./target/debug/capture_stream --name less --out /home/sulthonuladib/.herdr/worktrees/relayable/research-term-001-adr-003/spikes/terminal-engine/results/raw --timeout-ms 4000 --send 1500:q -- less /home/sulthonuladib/.herdr/worktrees/relayable/research-term-001-adr-003/spikes/terminal-engine/results/inputs/less_input.txt
# exit_code: 0
# utc_timestamp: 2026-10-15T17:05:34Z
# git_head: b96da2766e8750a77feee3beec0d84c2ce28f2fc
# rustc: rustc 1.98.1 (48a229cea 2026-09-01)
# cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)
# host: Linux 7.2.5-3-omarchy x86_64
# os: Omarchy
# ---- output ----
```

All 39 files share the identical `git_head`, `rustc`, `cargo`, `host`, `os` values; the
`utc_timestamp` ranges from `2026-10-15T17:05:22Z` (checks, esc_selftest, tools_available)
through `2026-10-15T17:05:39Z` (bin.meta, seq, top.esc, run_meta) — a single ~17-second run
on 2026-10-15. Every file's recorded `exit_code` is `0`.

### What the `git_head`/path mismatch means

- `git_head b96da276…` is **NOT** current HEAD `a1091e58…`. `b96da276` is `Wave 1 close-out`
  (`git log --oneline`), i.e. the pre-integration base the TERM-001 branch started from.
- The `capture_stream` / `clippy` commands embed the **retired worktree path**
  `/home/sulthonuladib/.herdr/worktrees/relayable/research-term-001-adr-003/spikes/terminal-engine`.
  That worktree no longer exists; the current checkout lives at
  `/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`.
- Consequence for reproducibility: the committed evidence was produced on a branch tree at
  `b96da276`, from a directory that no longer exists, with absolute paths baked into the
  commands. Re-running `scripts/run.sh` in the current checkout would: (a) delete and
  regenerate every tracked `results/raw/*` file, (b) stamp `git_head a1091e58…` (or whatever
  HEAD is then), and (c) record the new worktree path — so it would **not** reproduce the
  committed bytes, it would replace them. The committed set is therefore an
  **artifact of a specific historical tree/run, not a reproducible-from-current-HEAD output.**
- Consequence for the ADR's claims: the two self-contained probes (`pty_size`, `esc_selftest`)
  *do* reproduce exactly from the current checkout (sections 4–5), so the PTY/SIGWINCH and
  detector claims are sound. The byte-capture claims (`nvim`/`less`/`top`/`osc8_ls` payloads)
  depend on the external program versions on the host at that time and on the retired paths;
  they are credible as historical observations but are **not independently re-verified here**.
  This is consistent with ADR-003's own framing that phase 1 is engine-independent wire evidence
  and that the engine choice remains provisional.
- The ADR's cited spike stat is accurate: `git show --stat 085375c5fe02e79fbecb6d057a5aea306c7a3565 | tail -1`
  → `57 files changed, 7674 insertions(+)`, matching ADR-003. (Note: `docs/agent-loop/TESTS.md`
  reportedly records "58 files +8300" for the same commit — a documentation discrepancy for the
  review to reconcile; the raw git output confirms ADR-003's figure.)

## 7. Artifact resolution (ADR-003 pinned artifacts)

Scratch crate: `/tmp/adr003-resolve/Cargo.toml` (created for this run; `$SPIKE/Cargo.toml` untouched):

```toml
[package]
name = "adr003-resolve"
version = "0.0.0"
edition = "2021"

[dependencies]
alacritty_terminal = "=0.26.0"
vte = "=0.15.0"
portable-pty = "=0.9.0"
gpui = "=0.2.2"
```

Network first proven:

- `curl -sI https://index.crates.io/config.json` → exit 0, `HTTP/2 200`, `last-modified: Tue, 12 Mar 2024 07:31:15 GMT`, `date: Thu, 15 Oct 2026 18:23:38 GMT`.
- `date -u` → `Thu Oct 15 06:23:38 PM UTC 2026`.

### `cargo metadata --format-version 1 --manifest-path /tmp/adr003-resolve/Cargo.toml`

Exit code: **0**. stderr (verbatim head):

```
    Updating crates.io index
    Blocking waiting for file lock on package cache
     Locking 721 packages to latest compatible versions
      Adding cocoa v0.26.0 (available: v0.26.1)
      Adding cocoa-foundation v0.2.0 (available: v0.2.1)
      Adding core-foundation v0.10.0 (available: v0.10.1)
      Adding generic-array v0.14.7 (available: v0.14.9)
      Adding taffy v0.9.0 (available: v0.9.2)
 Downloading crates ...
  Downloaded arbitrary v1.5.0
  ... (721 packages resolved; full log in /tmp/adr003-meta.err)
  Downloaded gpui v0.2.2
  Downloaded vte v0.15.0
  Downloaded alacritty_terminal v0.26.0
```

Resolved versions parsed from the metadata JSON (exact pins honored):

```
alacritty_terminal = 0.26.0  source=registry+https://github.com/rust-lang/crates.io-index
gpui = 0.2.2                 source=registry+https://github.com/rust-lang/crates.io-index
portable-pty = 0.9.0         source=registry+https://github.com/rust-lang/crates.io-index
vte = 0.15.0                 source=registry+https://github.com/rust-lang/crates.io-index
```

`cargo fetch --manifest-path /tmp/adr003-resolve/Cargo.toml` → exit code **0**, empty stderr.

### Index cross-check (backup method)

`curl -s https://index.crates.io/<path>` per crate. Note: the path given in the task for `vte`
(`vt/e/vte`) is wrong per the crates.io index layout for 3-character names; it returns
`HTTP 404` (279-byte JSON error body). The canonical path is `3/v/vte` (`HTTP 200`). Both are
recorded for accuracy.

| Crate | Index path | HTTP | Versions listed (count) | Exact pin present? |
|---|---|---|---|---|
| `alacritty_terminal` | `al/ac/alacritty_terminal` | 200 | 44, ends `0.26.0-rc1, 0.26.0` | **`0.26.0` present** |
| `vte` | `vt/e/vte` (task-supplied) | 404 | — | wrong path |
| `vte` | `3/v/vte` (canonical) | 200 | 28, ends `0.14.1, 0.15.0` | **`0.15.0` present** |
| `portable-pty` | `po/rt/portable-pty` | 200 | 11, ends `0.8.1, 0.9.0` | **`0.9.0` present** |
| `gpui` | `gp/ui/gpui` | 200 | 7: `0.1.0, 0.1.0-test, 0.2.0-test.4, 0.2.0, 0.2.1-test3, 0.2.1, 0.2.2` | **`0.2.2` present (does exist)** |

Index metadata for the exact pins (from `index.crates.io`, 2026-10-15):

```
alacritty_terminal 0.26.0 cksum=bda177466b9524d59f1b12f0dd30b68696788e9992a7e959021c4a0ed96fcf59 yanked=False rust_version=1.85.0
vte                0.15.0 cksum=a5924018406ce0063cd67f8e008104968b74b563ee1b85dde3ed1f7cb87d3dbd yanked=False rust_version=1.62.1
portable-pty       0.9.0 cksum=b4a596a2b3d2752d94f51fac2d4a96737b8705dddd311a32b9af47211f08671e yanked=False rust_version=None
gpui               0.2.2 cksum=979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707 yanked=False rust_version=None
```

- `alacritty_terminal 0.26.0` checksum matches the value ADR-003 cites
  (`bda177466b9524d59f1b12f0dd30b68696788e9992a7e959021c4a0ed96fcf59`).
- `gpui 0.2.2` checksum matches ADR-003's cited checksum
  (`979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`).
- No pin is yanked.

## 8. Per-pinned-artifact verdict

| Pinned artifact | Verdict | Raw evidence |
|---|---|---|
| `alacritty_terminal = "=0.26.0"` | **RESOLVES** | `cargo metadata` exit 0, resolved `0.26.0`; index 200, cksum matches ADR-003 |
| `vte = "=0.15.0"` | **RESOLVES** | `cargo metadata` exit 0, resolved `0.15.0`; index `3/v/vte` 200 lists `0.15.0`, not yanked |
| `portable-pty = "=0.9.0"` | **RESOLVES** | `cargo metadata` exit 0, resolved `0.9.0`; index 200 lists `0.9.0`, not yanked |
| `gpui = "=0.2.2"` | **RESOLVES** | `cargo metadata` exit 0, resolved `0.2.2`; index `gp/ui/gpui` 200 lists `0.2.2`, not yanked, cksum matches ADR-003 |

No artifact is `DOES NOT RESOLVE` or `BLOCKED`. Network was available throughout.

Scope note: this proves the four exact pins are **published and resolvable/bindable** by cargo
on this host. It does **not** compile or run `alacritty_terminal`/`gpui` — resolution succeeded
for the metadata/fetch step; a full `gpui` build is a separate (UI-001) acceptance item and was
not attempted here (out of scope and not requested).

## 9. Assessment — is PTY-level evidence alone sufficient to accept the engine choice?

**Assessment (my opinion, clearly not a fact): No — PTY-level evidence alone cannot accept the
engine choice; the engine-level spike (TERM-002) must gate acceptance.** Section 4–5 evidence
shows the PTY layer is sound and deterministic: a real PTY is opened, resized via
`ioctl(TIOCSWINSZ, {40,120})` with `SET_WINSIZE_RC=0`, the child observes `40 120` and receives
`SIGWINCH`, and the detector fixtures pass. That validates `portable-pty`-class plumbing and the
wire bytes programs emit, but it never instantiates `alacritty_terminal`, so it cannot test the
things §5.4 actually asks about — rendering/reflow fidelity, selection, Unicode/wide glyphs,
vim/nvim/tmux/htop compatibility, and parser/state-vs-rendering separation.

ADR-003 itself says exactly this, so accepting on PTY evidence alone would contradict the ADR:

- Status line (line 5): *"**Proposed — PTY-level spike evidence available; engine-level spike
  pending**"*.
- Status note (lines 16–17): *"The engine decision is therefore **provisional pending the
  engine-level spike**, and no benchmark, memory target, or rendered-output result is claimed
  anywhere in this ADR."*
- Honest statement (lines ~255–259): *"It is **engine-independent**: it validates neither
  `alacritty_terminal` nor any engine … Those remain `spike-pending`."*
- §5.4 mapping marks engine-level bullets `spike-pending`: *"mechanism evidenced; **engine-level
  rendering spike-pending**"* (VT/ANSI), *"**spike-pending (no evidence)**"* (Unicode/wide
  glyphs, selection), *"**spike-pending (engine-level)**"* (shell/vim/nvim/less/top/tmux/TUI
  compatibility), *"PTY resize evidenced; **engine reflow spike-pending**"* (resize + reflow),
  *"mechanism evidenced; **engine-level separation spike-pending**"* (parser/state vs rendering).
- Decision (line 213): *"**Status: Proposed — PTY-level spike evidence available; engine-level
  spike pending.** … but no engine-level spike has validated the engine choice itself."*

My read: the ADR should stay **Proposed**, and REVIEW-001 should treat the engine pin as
**provisional / pending TERM-002**, not as an accepted decision. The artifact pins themselves
are solidly verified (section 8: all four resolve, two checksums match the ADR), so the
GPUI/engine *version-pin* portion of ADR-003 is on firm footing; it is the *engine behavioral
choice* that lacks evidence. Also worth flagging for the reviewer: the committed byte-capture
evidence is a historical artifact of `b96da276` in a since-retired worktree (section 6), so any
acceptance argument that leans on those raw captures should note that only `pty_size` and
`esc_selftest` were independently reproduced here.

---

## Command log (exact commands, cwd, exit codes)

| # | cwd | command | exit |
|---|---|---|---|
| 1 | `$SPIKE` | `cargo build` | 0 |
| 2 | `$SPIKE` | `cargo clippy --all-targets` | 0 |
| 3 | `$SPIKE` | `cargo fmt --check` | 0 |
| 4 | `$SPIKE` | `./target/debug/pty_size` | 0 |
| 5 | `$SPIKE` | `./target/debug/esc_selftest` | 0 |
| 6 | worktree root | `git status --short --branch` / `git status --porcelain` / `git rev-parse HEAD` / `git diff --stat` | 0 |
| 7 | `$SPIKE` | `find results/raw -type f` + `sed`/`cat` provenance reads | 0 |
| 8 | worktree root | `git show --stat 085375c…` , `git show --stat 2e41da4…` , `git log --oneline -15` | 0 |
| 9 | `/tmp/adr003-resolve` | `cargo metadata --format-version 1 --manifest-path /tmp/adr003-resolve/Cargo.toml` | 0 |
| 10 | `/tmp/adr003-resolve` | `cargo fetch --manifest-path /tmp/adr003-resolve/Cargo.toml` | 0 |
| 11 | `/tmp` | `curl -s https://index.crates.io/{al/ac/alacritty_terminal, 3/v/vte, po/rt/portable-pty, gp/ui/gpui}` | 0 (200s; `vt/e/vte` = 404) |
| 12 | `/tmp` | `curl -sI https://index.crates.io/config.json` ; `date -u` ; `rustc --version` ; `cargo --version` ; `uname -a` | 0 |

Artifacts written: this file only. Scratch (untracked, outside repo): `/tmp/adr003-resolve/**`,
`/tmp/adr003-*.out|.err`.
