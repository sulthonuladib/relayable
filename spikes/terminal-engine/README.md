# terminal-engine spike (TERM-001 / ADR-003, phase 1)

Engine-INDEPENDENT ground truth for terminal-engine selection (DEC-004, ADR-003).
This crate does **not** depend on a terminal-emulation engine and does **not**
render anything. It exists to produce the recyclable PTY evidence that any
engine choice must be judged against (`full.md` §5.4, §12).

Phase 2 (engine adapters) is **cancelled**; nothing here depends on an engine
crate (`alacritty_terminal`, `vte`, `termwiz`, `vt100`, …).

## Build and run

All commands run from this directory (`spikes/terminal-engine/`):

```bash
cargo build                 # standalone crate; empty [workspace] table
./scripts/run.sh            # builds + runs every probe and writes results/raw/
cargo fmt --check
cargo clippy --all-targets
```

`./scripts/run.sh` is idempotent (it deletes and regenerates `results/raw/`),
`set -euo pipefail`, prints each command, and stamps every result file with a
provenance header (exact command, exit code, UTC timestamp, `git rev-parse HEAD`,
rustc/cargo versions, host/OS). It exits non-zero if any step fails.

## Binaries

| Binary | What it proves |
|---|---|
| `pty_size` | Real pty opened with a requested window size; child sees it via `stty size`; real `ioctl(TIOCSWINSZ)` changes the child's view; kernel delivers `SIGWINCH` and the child reacts. |
| `capture_stream` | Runs a real full-screen program on a real pty, captures the exact raw byte stream, and byte-level classifies alt-screen / mouse / bracketed-paste / OSC / cursor-mode / DECSCUSR / SGR sequences. |
| `scrollback_mem` | Feeds a precisely quantified byte volume (`--mib M` bytes of fixed-size numbered lines) through a real pty and observes this process's RSS under `discard` / `unbounded` / `bounded` scrollback retention. |
| `esc_selftest` | Feeds synthetic fixtures for every classified sequence through the same scanner, so `NOT_OBSERVED` in a real capture means "absent on the wire", not "detector broken". Negative control included. |

## Claims each result supports (mapping to ADR-003 / §5.4)

| ADR-003 / §5.4 item | Evidence | Result files |
|---|---|---|
| Resize is testable, engine-independent | `pty_size` exit 0; real resize 24×80 → 40×120 | `results/raw/pty_size.txt` |
| VT/ANSI + alternate screen | `?1049h/l` observed for nvim, less | `results/raw/{nvim,less}.{bin,esc,seq,summary.txt}` |
| Mouse modes (1000/1002/1003/1006) | `?1002h/l` + `?1006h/l` observed for nvim; 1000/1003/1005/1015 not observed by these programs | `results/raw/nvim*.seq` |
| Bracketed paste | `?2004h/l` observed for nvim | `results/raw/nvim*.seq` |
| OSC title | `OSC 0;…` observed when `nvim '+set title'` | `results/raw/nvim_title.*` |
| OSC 8 hyperlinks | observed from `ls --hyperlink=always` (no full-screen program emitted them) | `results/raw/osc8_ls.*` |
| OSC color queries | `OSC 11;?` background-color query observed from nvim | `results/raw/nvim*.seq` |
| Cursor modes (`?25`, DECSCUSR) | `?25l/h` and `ESC [ 1 SP q` observed | `results/raw/{nvim,top}.*` |
| SGR color | observed for nvim, less, top | `results/raw/{nvim,less,top}.seq` |
| Scrollback limits / memory behavior | `discard` flat; `unbounded` peak ≈ retained bytes; `bounded:1000` flat | `results/raw/scrollback_*.txt` |
| Detector is sound | all fixtures PASS + negative control PASS | `results/raw/esc_selftest.txt` |
| Real programs available on this host | `command -v` + versions recorded | `results/raw/tools_available.txt` |

Toolchain/static checks live in `results/raw/checks/`.

## How the PTY is owned (API verification)

Every API used was read from the real crate source before use and then verified
by compiling and running:

- `nix` 0.31.3 `src/pty.rs`, `src/fcntl.rs`, `src/unistd.rs`, `src/sys/signal.rs`
  (`posix_openpt`, `grantpt`, `unlockpt`, `ptsname_r`, `open`, `Winsize`).
- `libc` `TIOCGWINSZ`, `TIOCSWINSZ`, `TIOCSCTTY`, `setsid`, `killpg`, `ioctl`.

`src/pty.rs` opens the master with `posix_openpt(O_RDWR|O_NOCTTY)`, unlocks the
slave, opens it, sets the initial window size, then spawns `Command` with a
`pre_exec` hook that calls `setsid()` and `ioctl(0, TIOCSCTTY)` so the pty is
the child's controlling terminal (this is what routes `SIGWINCH` to it). This is
the same ownership model the eventual server-side PTY owner will need
(`full.md`: the GUI view does not own the shell).

## Limitations / unproven (phase 1 stops here)

- **No engine, no rendering, no view separation** is evaluated. §5.4 items such
  as Unicode/wide glyphs, selection, hyperlink rendering, parser/view
  separation, Android rendering, license/release-cadence/test-quality, and
  shell/multiplexer compatibility are **not** covered.
- `scrollback_mem` is an **observation, not a benchmark** and asserts **no
  memory target** (§0.3.7, §12). It measures this debug-profile probe process,
  not any engine. RSS is sampled from `/proc/<pid>/status` `VmRSS` at a fixed
  interval plus `VmHWM` at the end; transient peaks between samples can be
  missed.
- Captures are point-in-time for the versions in `tools_available.txt`
  (nvim 0.12.5, less 704, procps-ng top 4.0.7, coreutils 9.11, ncurses 6.6).
  `NOT_OBSERVED` is scoped to those programs/versions/run, not universal.
- `top` on this host did **not** enter the alternate screen; the multiplexer
  candidate `tmux` is present but was not captured (starting a tmux server is
  out of this bounded phase).
- The byte-stream classifier covers the categories ADR-003 names; it is not a
  full VT parser and does not decode all CSI/DCS semantics.
- No X11-vs-Wayland display behavior is exercised (no rendering in phase 1);
  this is a pty-only harness.
- Timings (e.g. `WALL_MS` in `scrollback_*`) are single-run observations with no
  variance study.
