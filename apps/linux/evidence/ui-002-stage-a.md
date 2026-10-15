# UI-002 stage A — foundation summary (implementer:stage-a-foundation)

Worktree: `/home/sulthonuladib/.herdr/worktrees/relayable/ui-ui-002-reference-workspace`
Scope: `apps/linux` **only** (the separate Cargo workspace). Reviewed with
`git status --short`: only the owned paths below changed; `src/main.rs`,
`src/tokens.rs`, `README.md`, `evidence/run-window.sh`, `evidence/validate.sh`,
all of `crates/**`, `docs/**`, `tests/**`, `protocol/**`, and the repository-root
`Cargo.toml`/`Cargo.lock` are untouched.

Host: Linux `falcon` x86_64, rustc/cargo 1.98.1, single monitor 1920×1080 at
scale 1 (so reference video px were treated as 1:1 logical px, per instruction).

## 1. Files changed / added

| Path | Purpose |
| --- | --- |
| `apps/linux/Cargo.toml` | Path deps on `relayable-client`/`relayable-model`/`relayable-protocol` (`transport-unix`); ADR-003 pins `alacritty_terminal = "=0.26.0"`, `vte = "=0.15.0"` (`ansi`); new `[[bin]] live-probe`. `gpui = "=0.2.2"` and the empty `[workspace]` table kept. |
| `apps/linux/Cargo.lock` | Re-resolved lock. `libc 0.2.189` + `xattr 0.2.3` hold survived (see `ui-002-libc-remedy.md`). |
| `apps/linux/src/lib.rs` | Declares `api`, `app`, `geometry`, `term`, `theme`, `tokens`, `ui`. GPUI-free. |
| `apps/linux/src/api/mod.rs` | Live two-plane client (private data plane + `relayable_client` control plane), the frozen `ApiHandle`/`ApiEvent`/`ApiCommand`/`ConnectionState` shapes, bounded reconnect policy, clearly-labelled mock, OSC title/cwd tracker. |
| `apps/linux/src/term/mod.rs` | GPUI-free headless terminal model on `alacritty_terminal 0.26.0` + `vte 0.15.0`; `bg: None` for default backgrounds. |
| `apps/linux/src/geometry.rs` | Measured reference geometry (reuses `tokens::Measured`/`Confidence`), `BASELINE_CANVAS_*`, `workspace_columns`, `split_panes`. |
| `apps/linux/src/theme.rs` | Packed `0xRRGGBB` `u32` colour consts + `rgb_u32`, all mapped from `tokens`; Relayable-original font proposals. |
| `apps/linux/src/app.rs` | Frozen placeholder `pub struct WorkspaceModel;` (gpui-free). |
| `apps/linux/src/ui/mod.rs` | Frozen placeholder UI module (doc comment only, gpui-free). |
| `apps/linux/src/bin/live-probe.rs` | GPUI-free live proof against a real server. |
| `apps/linux/evidence/ui-002-live-probe.log` | Raw live-probe transcript. |
| `apps/linux/evidence/ui-002-libc-remedy.md` | `libc`/`xattr` resolution note. |
| `apps/linux/evidence/ui-002-stage-a.md` | This summary. |

## 2. Validation — exact commands and exit codes

From `apps/linux`:

```
cargo fmt --check && cargo clippy --all-targets --locked && cargo test --locked && cargo check --locked
```

| Command | Exit |
| --- | --- |
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets --locked` | 0 (only a transitive `proc-macro-error2` future-incompat note; no crate warnings) |
| `cargo test --locked` | 0 — **35 passed / 0 failed**, including the pre-existing 9 token-sheet tests |
| `cargo check --locked` | 0 |

From the repository root (proves `apps/linux` stays a separate workspace and the
core is unaffected):

| Command | Exit |
| --- | --- |
| `cargo check --workspace --locked` | 0 |
| `cargo build --workspace --locked` | 0 (needed first: the integration harness locates the `relayable` CLI binary next to the test executable) |
| `cargo test --workspace --locked` | 0 |

`scripts/validate.sh` was **not** run separately because it is the slower
full-suite wrapper (fmt + `clippy -D warnings` + build + test + acceptance); the
equivalent root build+test above was run instead, per the brief's fallback.

New tests added: API `Send + Sync`, mock labelling/non-blocking/round-trip, OSC
tracker, tail bound, `strip_file_uri`; terminal SGR/default-bg/explicit-bg/
resize/partial-escape+invalid-UTF-8/truecolor/scrollback/`snapshot_at` clamping;
geometry layout sums and tiny-width clamping, midpoint-in-range, baseline-caveat;
theme colour-equals-token for every theme colour + `rgb_u32`.

## 3. Live probe — observed result (raw transcript in `ui-002-live-probe.log`)

Server built at the repo root and run **in the foreground** with a scratch socket
`/tmp/relayable-ui002-probe/relayable.sock`, `SHELL=/bin/bash`; probe run under
`timeout 60`; server killed, waited, socket removed. `PROBE_EXIT=0`.

Observed stdout (verbatim excerpt; the byte count varies slightly run to run
with shell-prompt timing — the raw final run is in the log):

```
live-probe: socket = /tmp/relayable-ui002-probe/relayable.sock
live-probe: connected
live-probe: created block block:01a14194-e9a9-73b5-be07-5847a1c64fad
live-probe: attached stream_id = 1
live-probe: observed RELAYABLE-PROBE-OK and initial size 24 80
live-probe: received output bytes = 230
live-probe: stream_id = 1
live-probe: TerminalResizeAccepted = stream_id=1 cols=120 rows=40
live-probe: rendered screen (120x40), first 24 rows:
||
|~ ❯ printf 'RELAYABLE-PROBE-%s\n' OK; stty size|
|RELAYABLE-PROBE-OK|
|24 80|
||
|~ ❯ stty size|
|40 120|
...
PROBE_EXIT=0
```

Proof: the child's own `stty size` was `24 80` before and `40 120` after the
`terminal.resize`; the server's `TerminalResizeAccepted` was observed. No output
was fabricated.

## 4. `libc` / `xattr`

After `cargo check` (**unlocked**) in `apps/linux`, the lock still resolved
**`libc 0.2.189` + `xattr 0.2.3`**; the `ENOATTR` failure did **not** return.
The remedy in effect is the UI-001 lockfile hold
`cargo update -p libc --precise 0.2.189`, which survives the new deps because
`relayable-server` (the only `libc = "=0.2.190"` pin) is not reachable and every
reachable requirement is caret `^0.2`. **No `[patch.crates-io] xattr` vendoring
was needed.** Details in `ui-002-libc-remedy.md`.

## 5. Verified upstream API entry points (local registry source, file:line)

`alacritty_terminal-0.26.0`:

- `src/term/mod.rs:410` — `Term::new(config, &dimensions, event_proxy)`
- `src/term/mod.rs:655` — `Term::resize`
- `src/term/mod.rs:637` — `Term::renderable_content`
- `src/term/mod.rs:645` — `Term::grid`; `:709` `Term::mode`
- `src/term/mod.rs:2365` — `RenderableCursor { shape, point }`; `:2393`
  `RenderableContent { display_iter, cursor, display_offset, … }`
- `src/event.rs:103` — `EventListener::send_event`; `src/event.rs:108`
  `VoidListener` (used as the model's proxy)
- `src/grid/mod.rs:486` — `Dimensions` (`columns`, `screen_lines`, `total_lines`,
  default `history_size`/`last_column`/`topmost_line`/`bottommost_line`)
- `src/grid/mod.rs:412` — `Grid::iter_from`; `:422` `Grid::display_iter`
  (row-major viewport iterator)
- `src/term/cell.rs:134` — `Cell { c, fg, bg, flags }`; `:15` `Flags`
  (`BOLD`, `ITALIC`, `DIM`, `ALL_UNDERLINES`, `WIDE_CHAR_SPACER`, …)
- `src/term/color.rs` — `Colors` is **not** needed (cell `fg`/`bg` already carry
  `vte::ansi::Color`)

`vte-0.15.0`:

- `src/ansi.rs:287` — `Processor::new`; `:298` `Processor::advance(&mut handler, bytes)`
- `src/ansi.rs:1128` — `Color`; `:1013` `NamedColor`; `:60` `Rgb`; `:828`
  `CursorShape` (`Hidden` ⇒ no cursor)
- `src/lib.rs:109` — `Parser::advance(&mut performer, bytes)`; `src/lib.rs:761`
  `Perform` trait (`osc_dispatch` at `:792`, all methods defaulted) — used for the
  OSC 0/2 title and OSC 7 cwd tracker

## 6. Interface notes / deliberate deviations

1. **`lib.rs` freezing succeeded.** `pub mod api; pub mod app; pub mod geometry;
   pub mod term; pub mod theme; pub mod tokens; pub mod ui;` are all declared and
   every one is **GPUI-free** (`app.rs` = `pub struct WorkspaceModel;`, `ui/mod.rs`
   = doc comment only). `src/main.rs` is the only file that imports `gpui`, and it
   was not touched.
2. **Flow-control ack unit.** The brief said "ack_bytes: exact frame byte len".
   The server charges its credit window per **output payload byte**
   (`crates/relayable-server/src/session.rs` `Subscriber::try_consume(bytes)`,
   `add_credit(ack_bytes)`), and the reference client acks
   `output.bytes.len()` (`crates/relayable-client/src/lib.rs`,
   `wait_for_output_inner`). The data plane therefore acks exactly
   `TerminalOutput.bytes.len()`. Acking the whole envelope would permanently pin
   the window at its cap. This is a documented interpretation, not a new number.
3. **Reconnect policy** is implemented client-side only (bounded, 3 attempts,
   fixed 500 ms backoff), emitting `Reconnecting{attempt}` then
   `Connected`/`Error`, and is labelled as Relayable-original — core-002 §G leaves
   reconnect policy unimplemented.
4. **`PaneTitleChanged`** is driven by real output bytes: the data plane runs a
   defaulted `vte::Perform` that records OSC 0/2 (title → `program`) and OSC 7
   (cwd) directly from the live stream. The wire event set has no title/pwd event
   (core-002 §D).
5. **`Attached.screen_bytes`** carries the server's `TerminalSnapshot.screen_bytes`
   clamped to the last 32 KiB (the server's `output_tail_bytes` default is 32 768
   and `scrollback_bytes` is always empty). The consumer seeds its `TerminalModel`
   from it; the API does not embed a model of its own.
6. **`ApiHandle::send`** uses an unbounded `std::sync::mpsc` with error-ignoring
   sends, so it never blocks and never panics after shutdown; the receiver is
   behind a `Mutex` so the handle is `Send + Sync`. Events are not dropped by a
   queue bound; the UI stage must drain each frame.
7. **`vte` declaration** is `vte = { version = "=0.15.0", features = ["ansi"] }`
   (the exact ADR-003 pin plus the `ansi` feature required to name `vte::ansi`;
   `alacritty_terminal` already enables it, so no new crates are pulled).
8. **`session.view`** is wired into the reconnect path (`resume_after_connect`
   calls `session_snapshot` and emits `ServerEvent{kind:"session.view"}`, then
   `session.resume`), so the control plane genuinely uses create/list/view/resume.
9. **Font list** in `theme.rs` is the verified host list
   `["JetBrains Mono", "JetBrainsMono Nerd Font", "Noto Sans Mono",
   "Liberation Mono"]`, marked Relayable-original / non-reference.

## 7. Remaining risks / not done

- `Config::scrolling_history` keeps the engine default (10 000 lines). ADR-003 /
  `full.md` §6.4/§12 require a *measured* Relayable bound; no number is invented
  here.
- The per-plane read loop uses a 100 ms read timeout (matching the existing
  `relayable-client` pattern) to stay responsive to commands. A read deadline
  expiring between a frame's 4-byte prefix and its body would desync the stream;
  this is the same residual caveat the core client documents, mitigated by the
  server writing each frame's prefix and body back-to-back.
- Mock output is synthetic and only reachable through `ApiHandle::connect_mock`;
  the UI stage must surface `is_mock()` per the brief.
- No GPUI/UI rendering, no window sizing, and no `src/main.rs` change were made.
