# STATUS — terminal-engine spike (phase 1)

Bounded phase-1 task only. **Phase 2 (engine adapters) is cancelled** by owner
direction; no terminal-emulation engine dependency is present or planned here.

## State at this commit

- `cargo build` (debug) → **succeeds, exit 0**.
- `cargo fmt --check` → **exit 0**.
- `cargo clippy --all-targets` → **exit 0, no warnings**.
- `./scripts/run.sh` → **exit 0, aggregate_failures=0**; all probes ran on this
  real host and wrote provenance-stamped results under `results/raw/`.
- No step is currently failing. Nothing in this spike is fabricated or guessed:
  every number is copied from a generated result file.

## DONE

- Standalone crate at `spikes/terminal-engine/` with an empty `[workspace]`
  table, edition 2021, `rust-version = "1.98"`, committed `Cargo.lock`, no
  engine dependency, nested `.gitignore` for `/target/`.
- `pty_size`: real pty (`/dev/pts/N`), child sees requested 24×80, real
  `ioctl(TIOCSWINSZ)` to 40×120, `SIGWINCH` delivered and handled, exit 0.
- `capture_stream`: raw byte capture + classification for nvim (default, mouse,
  title), less, top, and `ls --hyperlink` (OSC 8 probe), with `.bin` exact bytes,
  `.esc` caret dump, `.seq` decoded listing, and `.summary.txt` checklist.
- `scrollback_mem`: 64 MiB payload observation under discard / unbounded /
  bounded(1000) retention, with exact N/method and RSS sampling.
- `esc_selftest`: detector fixtures + negative control, all PASS.
- `scripts/run.sh`: idempotent driver with per-file provenance headers.
- `README.md`, `results/README.md`: claims ↔ result mapping and limitations.

## NOT DONE (out of scope for the cancelled phase 2 / not covered by phase 1)

- No engine dependency, no engine adapters, no parser/view separation.
- No rendering, no Unicode/wide-glyph, selection, hyperlink rendering, or
  Android rendering evaluation.
- No tmux/multiplexer capture (`tmux` is present; starting a server was out of
  the bounded phase).
- No benchmark-grade performance measurement and no memory target.
- No X11/Wayland display behavior (pty-only harness).

## Known limitations

See `README.md` "Limitations / unproven" and `results/README.md` "Notes".
`scrollback_mem` is an observation of the probe process, not any engine, and
single-run only.
