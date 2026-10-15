# terminal-engine spike results index

All files below were produced by `./scripts/run.sh` on this host in one run.
Every text result file starts with a `# ---- provenance ----` block containing
the exact command, exit code, UTC timestamp, `git rev-parse HEAD`, rustc/cargo
versions, and host/OS. Binary captures `NAME.bin` hold exact bytes; the matching
`NAME.bin.meta` carries the provenance header.

Raw environment facts (`results/raw/tools_available.txt`): Linux
`7.2.5-3-omarchy` x86_64 (Omarchy); rustc/cargo 1.98.1; `nvim` 0.12.5,
`less` 704, `top` procps-ng 4.0.7, coreutils 9.11, ncurses 6.6; `vim`, `htop`,
`busybox`, `nano` absent.

## Index — file → claim → command → exit code

| File | Claim it supports | Command | Exit |
|---|---|---|---|
| `raw/checks/cargo_build.txt` | Crate compiles (standalone, no engine dep) | `cargo build` | 0 |
| `raw/checks/cargo_fmt_check.txt` | Formatting is clean | `cargo fmt --check` | 0 |
| `raw/checks/cargo_clippy.txt` | No clippy warnings (`--all-targets`) | `cargo clippy --all-targets` | 0 |
| `raw/esc_selftest.txt` | Scanner detects every required sequence; negative control clean | `./target/debug/esc_selftest` | 0 |
| `raw/pty_size.txt` | Real pty; child sees requested size; real `TIOCSWINSZ` 24×80→40×120; `SIGWINCH` delivered and handled | `./target/debug/pty_size` | 0 |
| `raw/nvim.summary.txt` + `.bin/.esc/.seq` | nvim emits `?1049h/l`, `?1002h/l`, `?1006h/l`, `?2004h/l`, `?25l/h`, `ESC[1 SP q`, SGR, `OSC 11;?` | see provenance header (capture `nvim --clean`, send `:qa!`) | 0 |
| `raw/nvim_mouse.summary.txt` + files | Same with `+set mouse=a` (mouse modes still 1002+1006) | capture `nvim --clean '+set mouse=a'` | 0 |
| `raw/nvim_title.summary.txt` + files | `set title` makes nvim emit a real `OSC 0;…BEL` title | capture `nvim --clean '+set title'` | 0 |
| `raw/less.summary.txt` + files | less emits `?1049h/l` and SGR `ESC[7m`; no mouse/paste/OSC | capture `less results/inputs/less_input.txt`, send `q` | 0 |
| `raw/top.summary.txt` + files | top emits `?25l/h` + SGR but **no** alternate screen on this host | capture `top -d 1`, send `q` | 0 |
| `raw/osc8_ls.summary.txt` + files | Real `OSC 8` hyperlinks (no full-screen program here emitted them; `ls --hyperlink=always` is the probe) | capture `ls --hyperlink=always --color=never -1 results/inputs/hyperlink_dir` | 0 |
| `raw/scrollback_discard.txt` | 64 MiB payload through pty, read+drop: RSS stays flat | `./target/debug/scrollback_mem --mib 64 --retain discard --interval-ms 25` | 0 |
| `raw/scrollback_unbounded.txt` | 64 MiB retained unbounded: sampled RSS peak grows by ~retained bytes | `./target/debug/scrollback_mem --mib 64 --retain unbounded --interval-ms 25` | 0 |
| `raw/scrollback_bounded.txt` | 64 MiB with a 1000-line ring: RSS stays flat | `./target/debug/scrollback_mem --mib 64 --retain bounded --cap-lines 1000 --interval-ms 25` | 0 |
| `raw/tools_available.txt` | Real `command -v` + versions for every candidate tool | shell block in `scripts/run.sh` | 0 |
| `raw/run_meta.txt` | Aggregate failures and produced file list | `./scripts/run.sh` | 0 |

## Key numbers

**Resize** (`raw/pty_size.txt`): slave `/dev/pts/9`; requested 24×80;
`MASTER_WINSIZE_INITIAL=24 80`; `CHILD_STTY_SIZE_INITIAL=24 80`;
`ioctl(master, TIOCSWINSZ, {ws_row:40, ws_col:120})` → rc 0;
`MASTER_WINSIZE_AFTER=40 120`; `CHILD_STTY_SIZE_AFTER=40 120`;
`SIGWINCH_OBSERVED=true`; child exit 0. The child is session leader /
foreground pgroup (`Ss+`), and `CHILD_PS_AFTER_RESIZE` shows the same pid/pgid/
sid after the resize.

**Scrollback/RSS** (64 MiB target = 1,220,161 lines × 55 B = 67,108,855 B
payload; 68,329,016 B actually read from the pty because the slave applies
`OPOST|ONLCR`, `+1` byte per `\n`). Method: `/proc/<pid>/status` `VmRSS` every
25 ms + `VmHWM` at end, debug profile, single run. These are observations, **not**
a benchmark and **not** a memory target:

| Mode | Retained bytes | RSS start | RSS sampled peak | VmHWM | Wall |
|---|---|---|---|---|---|
| discard | 0 | 2,732 kB | 2,884 kB | 2,884 kB | 807 ms |
| unbounded | 68,329,016 | 2,752 kB | 68,476 kB | 69,332 kB | 805 ms |
| bounded 1000 lines | 56,000 | 2,728 kB | 3,004 kB | 3,004 kB | 981 ms |

For `unbounded`, `RSS_END_KB` drops back to ~3,032 kB after the reader thread's
retention buffer is dropped; `RSS_PEAK_SAMPLED_KB`/`VmHWM` capture the peak.

## Byte-level escape findings (from `*.seq`)

| Category | nvim | nvim mouse | nvim title | less | top | osc8_ls |
|---|---|---|---|---|---|---|
| alt screen `?1049` enter/leave | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| alt screen `?47` / `?1047` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| mouse `?1000` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| mouse `?1002` | ✅ (2) | ✅ (1) | ✅ (2) | ❌ | ❌ | ❌ |
| mouse `?1006` | ✅ (2) | ✅ (1) | ✅ (2) | ❌ | ❌ | ❌ |
| mouse `?1003` / `?1005` / `?1015` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| bracketed paste `?2004` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| cursor `?25` hide/show | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| DECSCUSR `ESC[1 SP q` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| SGR `ESC[…m` | ✅ (337) | ✅ (337) | ✅ (337) | ✅ (2) | ✅ (374) | ❌ |
| OSC title | ❌ | ❌ | ✅ (1) | ❌ | ❌ | ❌ |
| OSC 8 hyperlink | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (4) |
| OSC color (`11;?`) | ✅ (2) | ✅ (2) | ✅ (2) | ❌ | ❌ | ❌ |

Counts are per capture; first-offset and exact bytes are in each `.seq` /
`.summary.txt`. The detector is proven sound (including a synthetic `?1002;1006`
combination and a negative control) by `raw/esc_selftest.txt`.

## Inputs

- `inputs/less_input.txt` — 400 lines generated by `seq` so `less` has more than
  one screen (generated by `scripts/run.sh`).
- `inputs/hyperlink_dir/{alpha,beta}.txt` — two files for the `ls --hyperlink`
  OSC 8 probe (generated by `scripts/run.sh`).

## Notes / limitations

- `RETAINED_LINES` is only counted in `bounded` mode; for `unbounded` use
  `RETAINED_BYTES`.
- `NOT_OBSERVED` is scoped to these programs/versions/runs, not universal.
- Single-run observations; no variance study, no benchmark claims.
