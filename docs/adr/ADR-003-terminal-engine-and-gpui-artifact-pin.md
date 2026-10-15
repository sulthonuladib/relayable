# ADR-003 — Terminal engine and GPUI artifact pin

| Field | Value |
|---|---|
| Status | **Proposed — PTY-level spike evidence available; engine-level spike pending** |
| Date | 2026-10-15 |
| Deciders | Lead (Relayable / Superlogical) |
| Supersedes | — (new ADR; this file creates `docs/adr/`) |
| Related | DEC-004 (Terminal engine selection, Pending, this ADR); DEC-002 / ADR-001; DEC-005 / ADR-004; `full.md` §5.2, §5.3, §5.4, §6.1, §6.4, §12, §13.5, §15.2; `docs/agent-loop/DECISIONS.md:96–97` |

> **Status note.** The TERM-001 spike (`spikes/terminal-engine`) has a **committed, engine-independent
> phase 1** (commit `085375c5fe02e79fbecb6d057a5aea306c7a3565`): a real-PTY resize/SIGWINCH probe, raw
> byte captures from real full-screen programs, and an RSS observation — all provenance-stamped under
> `spikes/terminal-engine/results/raw/`. Its **phase 2 (per-engine adapters) was cancelled by
> wind-down and has no results**. Phase 1 validates PTY plumbing and what programs put on the wire; it
> does **not** validate `alacritty_terminal` or any engine. The engine decision is therefore
> **provisional pending the engine-level spike**, and no benchmark, memory target, or rendered-output
> result is claimed anywhere in this ADR. Every version, license, date, checksum and source fact below
> comes from published crates.io artifacts, published sources, and the committed spike artifacts
> (see `## Evidence`).

---

## Context

`full.md` §5.4 requires Relayable to avoid writing a terminal emulator from scratch unless the
maintained Rust-compatible candidates fail a documented spike, and to evaluate them on VT/ANSI
coverage, alternate screen, OSC, mouse modes, bracketed paste, Unicode/wide glyphs, selection,
hyperlinks, resize; scrollback limits and memory behavior; compatibility with common shells,
`vim`/`nvim`, `less`, `top`/`htop`, multiplexers and full-screen TUIs; parser/state vs rendering
separation; how state can render on Android later; and platform support, licensing, release cadence
and test quality. §5.3 additionally requires GPUI for the Linux app with a pinned version/revision,
GPUI-specific code isolated in `apps/linux`, and no GPUI types leaked into shared crates. §5.2
requires choices to be documented in ADRs with context, options, decision, consequences and
reconsideration evidence.

The reference product's own runtime is recorded as **`MyGo native UI + mygo/plugins/terminal`
(Ghostty libghostty-vt), no webview, no cgo** (`docs/research/gorex-review.md:19`). That is a C/Zig
engine, so the §5.4 phrase "maintained Rust-compatible terminal engines" and the reference product's
engine are in tension; this ADR records that as an option rather than silently ignoring it.

`docs/agent-loop/DECISIONS.md:96` records the open terminal-engine question and `:97` records the
GPUI artifact/license pin question (`crates.io gpui 0.2.2` vs `Glass-HQ/gpui`) as needing this ADR.

Two facts constrain the choice mechanically:

- **`wezterm-term` does not exist on crates.io.** `GET https://crates.io/api/v1/crates/wezterm-term`
  → HTTP 404, body `{"errors":[{"detail":"crate \`wezterm-term\` does not exist"}]}` (2026-10-15).
  `libvterm` and `crosspty` return the same 404. They are workspace/source names, not published
  crates, so they cannot be pinned as artifacts.
- **The reference UI is not a generic redesign** (`full.md` §0.2/§2.4); engine selection must support
  the observed terminal behaviors in `docs/compatibility/reference-behavior-matrix.md`
  (`terminal.*` events at lines 387–400; terminal block methods at lines 320–322), not redefine them.

---

## Options Considered

### Option A — `alacritty_terminal` 0.26.0 (parser + grid/screen state) with `vte` 0.15.0 as the parser boundary and `portable-pty` 0.9.0 for PTY ownership

- `alacritty_terminal` 0.26.0, version-level `license = "Apache-2.0"`
  (`https://crates.io/api/v1/crates/alacritty_terminal/0.26.0`, 2026-10-15); published tarball
  `alacritty_terminal-0.26.0.crate` sha256 `bda177466b9524d59f1b12f0dd30b68696788e9992a7e959021c4a0ed96fcf59`
  ships only `LICENSE-APACHE` (sha256 `8884289e3969c089cd556c53fa1d3f51d86b29e52a8c1d660a55ce8292986910`); no `NOTICE`.
- `vte` 0.15.0, version-level `license = "Apache-2.0 OR MIT"`; tarball ships `LICENSE-APACHE`
  (`62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`) and `LICENSE-MIT`
  (`e4c9b06fa850cb9b540a5e400e9f6394cf15efcf4098144de477d1d3dae10150`).
- `portable-pty` 0.9.0, version-level `license = "MIT"`; tarball ships `LICENSE.md`
  (`191c46fcf52061382b1c51a70311eb9081381cc158e5899f3739473a9432185b`, `Copyright (c) 2018 Wez Furlong`).

Published-source verification of the §5.4 bullets (all greps reproduced in `## Evidence`):

| Capability | Published-source mechanism | Status |
|---|---|---|
| Scrollback + bounded history | `term::Config { pub scrolling_history: usize }` (`src/term/mod.rs:334,336`), default `10000` (`:359`); `Grid::update_history` (`src/grid/mod.rs:154`) | evidenced |
| Reflow on resize | `Grid::resize(reflow, lines, columns)` in `src/grid/resize.rs` (file header `//! Grid resize and reflow.`, `:1`; `grow_columns` `:101`, `shrink_columns` `:245`) | evidenced |
| Selection model | `pub struct Selection` (`src/selection.rs:119`), `SelectionRange` (`:33`) | evidenced |
| OSC 8 hyperlinks on cells | `pub struct Hyperlink` (`src/term/cell.rs:44`), `Cell::set_hyperlink` (`:202`), `Cell::hyperlink` (`:219`); OSC 8 dispatch in `vte` `src/ansi.rs:1392` | evidenced |
| Bracketed paste | `TermMode::BRACKETED_PASTE` (`src/term/mod.rs:61`), set/removed on mode 2004 (`:1970`, `:2029`) | evidenced |
| Mouse modes | `MOUSE_REPORT_CLICK`/`MOUSE_MOTION`/`MOUSE_DRAG`/`SGR_MOUSE`/`UTF8_MOUSE` (`src/term/mod.rs:60,63,69,71`; `MOUSE_MODE` mask `:80`) | evidenced |
| Alternate screen | `TermMode::ALT_SCREEN` (`src/term/mod.rs:69`), `Term::swap_alt` (`:714`) | evidenced |
| Wide/Unicode glyphs | `Flags::WIDE_CHAR`/`WIDE_CHAR_SPACER` handling (`src/term/mod.rs:584,605,626`), `unicode-width` dependency | evidenced |
| Parser/state vs rendering split | Library exports `term`, `grid`, `selection`, `tty` etc. with no renderer (`src/lib.rs`: `pub mod grid; pub mod term; pub mod tty;`), and re-exports the parser (`pub use vte;`) | evidenced |
| PTY ownership | `alacritty_terminal::tty` (`src/tty/mod.rs`), unix backend `mod unix` gated `#[cfg(not(windows))]` (`:10–13`) | evidenced; Android build not tested |

### Option B — `termwiz` 0.23.3 (MIT)

Rejected. Published source shows:

- No reflow: `grep -rln "reflow" src/` → no output (exit 1).
- No scrollback/history model: `grep -rn "history\|scrollback" src/terminal/` returns a single comment
  (`src/terminal/windows.rs:782`, "changes the size of everything including scrollback"); `Surface`
  has no history API.
- No user selection model: `grep -rn "selection" src/escape/osc.rs` matches OSC selection-data parsing
  only (`:167 parse_selection`), not a selection model; no `Selection` type exists.
- Focus mode is declared but no focus event is delivered: `FocusTracking = 1004`
  (`src/escape/csi.rs:824`), while `grep -rn "FocusGained\|FocusLost" src/` → no output.
- Self-declared unstable API: README (`README.md:7`) — *"It is currently in active development and
  subject to fairly wild sweeping changes."*
- Tight sibling coupling: `Cargo.toml` has 34 direct dependencies, 5 of them direct `wezterm-*`
  workspace siblings (`wezterm-bidi`, `wezterm-blob-leases`, `wezterm-color-types`,
  `wezterm-dynamic`, `wezterm-input-types`) and `vtparse 0.6.2`, all pinned inside the WezTerm tree.

### Option C — `vt100` 0.16.2 (MIT)

Rejected. Published source shows a real screen model with scrollback (`src/grid.rs:13–19`,
`scrollback_len`), alternate screen (`src/screen.rs:548`), a bracketed-paste flag (`:572`) and mouse
protocol mode/encoding getters (`:578,584`) — but:

- No reflow: `grep -rln "reflow" src/` → no output; `Screen::set_size` (`:88`) resizes the grid without
  reflow logic.
- No hyperlink support: `grep -rn "hyperlink\|Hyperlink" src/` → no output.
- No selection model / no input handling: the only "selection" hit is a comment
  (`src/screen.rs:165`); no selection type; the crate only reports input *modes*
  (`input_mode_formatted`, `:379`).

### Option D — `avt` 0.18.0 (Apache-2.0)

Rejected, but **not for the reason the pre-inspection assumption expected**. The published source
contradicts "parser/performer only, no screen state model": `avt` ships a `Terminal` with primary and
alternate buffers, a bounded scrollback (`scrollback_limit: Option<usize>`, `src/terminal.rs:26`;
`Terminal::gc`, `:343`), resize (`:525`) and reflow (`fn reflow`, `:565`). What it lacks is the
interactive surface §5.4 requires:

- No selection (`grep -rln "selection" src/` → no output).
- No hyperlinks / OSC 8 (`grep -rln "hyperlink" src/` → no output).
- No mouse modes and no bracketed paste (`grep -rln "mouse\|bracketed" src/` → no output).
- Its README (`README.md:24`) places **"input handling"** out of scope; it is an asciinema
  record/playback engine, not an interactive client engine.

### Option E — Bind `libghostty-vt` (C/Zig), matching the reference product

Considered and **not chosen now**. The reference product uses it
(`docs/research/gorex-review.md:19`), but §5.4 asks for maintained Rust-compatible engines, and no
license text, Rust binding availability, or build evidence for libghostty was fetched in this evidence
set. **UNVERIFIED** — outstanding check: fetch the Ghostty/libghostty license and any maintained Rust
binding, and spike a build. This remains a legitimate reconsideration path (see below).

### Option F — `termina` 0.4.0 (`MIT OR MPL-2.0`)

Not an engine candidate. It is a "virtual terminal (VT) manipulation library" (README), i.e. typed
escape-sequence writing, styling and input parsing for TUI applications: `pub trait Terminal: io::Write`
(`src/terminal.rs:95`), and no `Screen`/`Grid` type (`grep -rln "pub struct Screen\|struct Grid" src/`
→ no output). It is PTY/I-O adjacent, not a screen-state engine.

### Option G — `wezterm-term`, `libvterm`, `vterm-sys`, `crosspty`

- `wezterm-term`, `libvterm`, `crosspty`: **do not exist on crates.io** (HTTP 404, 2026-10-15). Cannot
  be pinned; the `wezterm-term` name refers to a WezTerm workspace crate, not a published artifact.
- `vterm-sys` 0.1.0 (MIT) *does* exist, but its last publication is **2016-04-23**
  (`PER-VERSION.tsv`), i.e. unmaintained for ~10 years, failing the §5.4 "maintained" requirement;
  it is a C `libvterm` binding, so it carries Option E's build/license questions as well.

### Option H (GPUI) — `gpui = "=0.2.2"` from crates.io

Chosen pin. Version-level `license = "Apache-2.0"`, `repository = "https://github.com/zed-industries/zed"`,
`created_at = "2025-10-22T03:56:29.835700Z"`, `crate_size = 5291873`, `published_by.login = "mikayla-maki"`,
`checksum = "979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707"`
(`https://crates.io/api/v1/crates/gpui/0.2.2`, 2026-10-15). The downloaded tarball's checksum matches
and it ships `LICENSE-APACHE` sha256 `752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a`,
**byte-identical** to `https://raw.githubusercontent.com/zed-industries/zed/main/LICENSE-APACHE`
(same sha256). The published `Cargo.toml.orig` declares `license = "Apache-2.0"`, `publish = true`.
Caveat: `.cargo_vcs_info.json` pins `sha1 = 69e2130295c2649963eb639fc70b4f2ee8ea1624` with
**`"dirty": true`** — the published tree is not a clean tagged revision. This dirty-flag/sha1 pair is
this ADR's own quote of the extracted tarball's file (evidence set not repo-tracked); independent
review verified artifact resolution and checksum match only, not the dirty flag or sha1
(`docs/reviews/evidence/adr-003-spike-rerun.md` §7–8). Note also that the crates.io
**crate-level** object carries **no `license` field** (absent/unset) while the **version-level**
`version.license` is `"Apache-2.0"`; the two are recorded separately and not collapsed.

### Option I (GPUI) — `Glass-HQ/gpui` (revision pin)

Rejected. `GET https://api.github.com/repos/Glass-HQ/gpui` (2026-10-15) → `fork: false`,
`license: null`, `created_at: 2026-02-04T15:19:02Z`, `pushed_at: 2026-05-16T02:19:19Z`, default-branch
HEAD `650aab59d54f4f6f85d42c965831935568ac1162`. `GET .../license` → **HTTP 404**
(`{"message":"Not Found",...,"status":"404"}`), and **7/7 root license filename probes 404**:
`LICENSE`, `LICENSE.md`, `LICENSE-APACHE`, `LICENSE-MIT`, `LICENSE-GPL`, `COPYING`, `NOTICE`. The only
license statement is `crates/gpui/Cargo.toml` → `license = "Apache-2.0"`, `publish = true`. That
working-tree manifest is a **newer tree** than published 0.2.2 (it references `gpui_platform`,
`gpui_web`, `scheduler`, `ttf-parser 0.25`, `windows 0.61`, none in the 0.2.2 published manifest), so
its `version = "0.2.2"` string is **not byte-equivalent** to the crates.io artifact.

Rejection reason: Apache License 2.0 §4(a) requires that a redistributor *"give any other recipients
of the Work or Derivative Works a copy of this License"* (`https://www.apache.org/licenses/LICENSE-2.0.txt`,
2026-10-15). A revision whose tree contains no license file makes that obligation unsatisfiable from
the pinned artifact alone. §4(b) and §4(d) add: modified files must carry prominent change notices
(*"You must cause any modified files to carry prominent notices stating that You changed the files"*),
and if the Work includes a `NOTICE` file, derivative distributions must include a readable copy of its
attribution notices. **No terminal- or GPUI-related crate inspected ships a `NOTICE` file** (see
`## Evidence`: `find . -iname 'NOTICE*'` → 0 matches across all 14 extracted artifacts), and
`Glass-HQ/gpui` itself has no `NOTICE`.

### Option J (GPUI) — renamed/republished artifacts `gpui-pre` 0.3.9, `gpui-ce` 0.2.2, and the `gpui-base`/`gpui-component`/`gpui-kit` 0.7.3 family

Rejected as the pin:

- `gpui-pre` 0.3.9 (Apache-2.0): a republish of Zed's `gpui`. Its `.orig` metadata states
  `zed-crate = "gpui"`, `zed-version = "0.2.2"`, `zed-rev = "93d745668805029476e6d7fbb19652116fed51de"`;
  it ships Zed's `LICENSE-APACHE` (same `752daf2f…` sha256). Its Zed revision existence was not
  confirmed. **UNVERIFIED** — outstanding check: resolve `93d7456…` in `zed-industries/zed`.
- `gpui-ce` 0.2.2 (Apache-2.0): "A community fork of Zed's GPU-accelerated UI framework"; ships the
  same `752daf2f…` `LICENSE-APACHE`; no semver guarantee and no verified fork base.
- `gpui-base`/`gpui-component` 0.7.3 (Apache-2.0, Longbridge `LICENSE-APACHE`
  `d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811`) depend on Zed's `gpui` only via
  the alias `package = "gpui-pre"`; `gpui-kit` 0.7.3 declares `license = "Apache-2.0"` but ships
  **no** license file at all.

These are third-party republishes behind a rename chain with an unverified fork base. `full.md` §15.2
makes an external API/license question a human stop condition, so the pin stays on the upstream
crates.io artifact (Option H) and these remain fallbacks.

---

## Decision

**Status: Accepted — Manager acceptance 2026-10-15 (DEC-004).** Engine pins validated by TERM-002: `alacritty_terminal` 0.26.0 + `vte` 0.15.0 + `portable-pty` 0.9.0 pass **32 PASS / 1 documented UNSUPPORTED / 0 FAIL / 0 NOT_TESTED** on real PTYs, and no engine change is warranted (`vt100` lacks OSC 8 + resize reflow; `avt` lacks title/mouse/bracketed-paste/OSC 8). The `gpui = "=0.2.2"` pin **builds and runs** via the UI-001 remedy (hold `libc` at 0.2.189 in the `apps/linux` workspace lockfile); a real Wayland window (`class=relayable`, `title=Relayable`) was launched and screenshotted. Reconsideration trigger 4 is discharged.

1. **Terminal engine:** pin `alacritty_terminal` **0.26.0** (`Apache-2.0`, version-level field) as the
   parser + screen/grid state engine.
2. **Parser boundary** (used if Relayable ever owns its own grid, e.g. for Android): pin `vte`
   **0.15.0** (`Apache-2.0 OR MIT`).
3. **PTY ownership:** pin `portable-pty` **0.9.0** (`MIT`), owned by the server-side process layer,
   not by any GUI view (`full.md` §5.4: "a GUI view does not own the shell").
4. **GPUI artifact:** pin `gpui` **exactly** as `gpui = "=0.2.2"` (`Apache-2.0`, version-level field;
   crate-level field absent), sourced from crates.io, with GPUI code isolated in `apps/linux` and no
   GPUI types in shared crates (`full.md` §5.3). **TERM-002 caveat (blocking for the pin):** on this
   host `cargo check` for `gpui 0.2.2` exits 101 at `xattr 0.2.3` (`libc::ENOATTR` was removed in
   `libc 0.2.190`); 0.2.2 is the latest crates.io `gpui` and a 7-variant feature matrix does not help.
   UI-001 applied and proved the remedy: `cargo update -p libc --precise 0.2.189` in the separate `apps/linux` workspace downgrades `libc 0.2.190 → 0.2.189` (`cargo check --locked`/`build`/`test`/`clippy` all exit 0) and a real window launches. `[patch.crates-io]` was not needed. Evidence: `apps/linux/evidence/TERM-002-remedy.md`, `spikes/terminal-engine-phase2/results/raw/gpui-followup/DIAGNOSIS.md`.
5. **License obligation mitigation:** vendor the applicable `LICENSE-APACHE`/`LICENSE-MIT`/`LICENSE.md`
   texts into any Relayable distribution, and ship no `NOTICE` (none of the inspected artifacts ships
   one). For `alacritty_terminal` that means `LICENSE-APACHE` (Apache-2.0); for `vte` both
   `LICENSE-APACHE` and `LICENSE-MIT`; for `portable-pty` its `LICENSE.md` (MIT); for `gpui`
   `LICENSE-APACHE`.
6. **Neither `wezterm-term` nor `libvterm`/`crosspty` may be referenced as crates.io artifacts** — they
   do not exist.

### §5.4 requirement mapping

**Phase-2 update (TERM-002).** The `alacritty_terminal` 0.26.0 engine-level results are committed in
`spikes/terminal-engine-phase2/results/REPORT.md` §3–§4 (32 PASS / 1 documented UNSUPPORTED / 0 FAIL
on real PTYs); the phase-1 column below is retained as the engine-independent record.

| §5.4 bullet | Chosen-engine mechanism (published source) | Spike evidence — TERM-001 phase 1, engine-independent (`spikes/terminal-engine`) | Status |
|---|---|---|---|
| VT/ANSI coverage | consumes `vte` 0.15.0 (`pub use vte;`, `ansi` feature) | real byte streams captured from nvim 0.12.5, less 704, top 4.0.7; SGR counts 337 / 2 / 335 (`results/raw/*.summary.txt`) — proves the sequences are emitted on the wire | mechanism evidenced; **engine-level rendering spike-pending** |
| Alternate screen | `TermMode::ALT_SCREEN`, `Term::swap_alt` (`src/term/mod.rs:69,714`) | `?1049h/l` OBSERVED for nvim and less; `?47`/`?1047` and top's alt screen `NOT_OBSERVED` (`results/raw/{nvim,less,top}.summary.txt`); detector proven sound by `results/raw/esc_selftest.txt` | evidenced (mechanism + wire) |
| OSC | `vte::osc_dispatch` (`src/ansi.rs:1329`): title 0/2 (`:1350`), color 4/10/11/12 (`:1366,1422`), hyperlink 8 (`:1392`), clipboard 52 (`:1483`) | `OSC 0;…` title OBSERVED (`results/raw/nvim_title.seq`), `OSC 11;?` color query OBSERVED (`results/raw/nvim.seq`), `OSC 8` OBSERVED (`results/raw/osc8_ls.seq`) | evidenced (mechanism + wire) |
| Mouse modes | `MOUSE_REPORT_CLICK`/`MOUSE_MOTION`/`MOUSE_DRAG`/`SGR_MOUSE`/`UTF8_MOUSE` (`src/term/mod.rs:60–80`) | `?1002h/l` + `?1006h/l` OBSERVED for nvim; `?1000`/`?1003`/`?1005`/`?1015` `NOT_OBSERVED` by these programs (`results/raw/nvim*.seq`) | evidenced (mechanism + wire) |
| Bracketed paste | `TermMode::BRACKETED_PASTE` (`:61`, mode 2004 `:1970,2029`) | `?2004h/l` OBSERVED for nvim (`results/raw/nvim.seq`) | evidenced (mechanism + wire) |
| Unicode / wide glyphs | `WIDE_CHAR`/`WIDE_CHAR_SPACER` flags (`:584,605,626`); `unicode-width` 0.2 dep | none — phase 1 captured SGR/ASCII payloads only; Unicode/wide-glyph behavior was phase 2 | **spike-pending (no evidence)** |
| Selection | `Selection`/`SelectionRange` (`src/selection.rs:33,119`) | none — selection was phase 2 | **spike-pending (no evidence)** |
| Hyperlinks | `Cell::hyperlink`/`set_hyperlink` (`src/term/cell.rs:202,219`), OSC 8 (`vte src/ansi.rs:1392`) | OSC 8 OBSERVED, `count=4` incl. `8;;` close markers, from `ls --hyperlink=always` (`results/raw/osc8_ls.seq` lines 35, 42–45) | evidenced (mechanism + wire) |
| Resize + reflow | `Grid::resize(reflow, …)` (`src/grid/resize.rs:1,14,101,245`) | real resize `ioctl(master, TIOCSWINSZ, {ws_row:40, ws_col:120})` rc 0 → child `stty size` = `40 120`; `SIGWINCH_OBSERVED=true`; child stays `Ss+` (`results/raw/pty_size.txt`) | PTY resize evidenced; **engine reflow spike-pending** |
| Scrollback limits + memory | `Config::scrolling_history` (`:336`, default `10000`), `Grid::update_history` (`grid/mod.rs:154`); §6.4 requires Relayable to *choose* the bound | observation only (not a benchmark, no target): 64 MiB payload, `68,329,016 B` read; `VmHWM` discard `2,884 kB` / unbounded `69,332 kB` (retained `68,329,016 B`) / bounded-1000 `3,004 kB` (retained `56,000 B`); debug profile, single run (`results/raw/scrollback_*.txt`) | mechanism evidenced; **Relayable must set the bound from §12 measurement** |
| Shell/vim/nvim/less/top/htop/multiplexer/TUI compatibility | no engine-selected mechanism | nvim, less, top ran under a real PTY and were captured; `vim`, `htop`, `busybox`, `nano` absent; `tmux` present but **not captured** (`results/raw/tools_available.txt`; `results/README.md` "Notes / limitations") | **spike-pending (engine-level)** |
| Parser/state vs rendering separation | library exports `term`/`grid`/`selection`/`tty`, no renderer (`src/lib.rs`); `EventListener` (`src/event.rs:103`) | phase 2 (engine adapters) CANCELLED; phase 1 is a pty-only harness with no view separation (spike README: "No engine, no rendering, no view separation is evaluated") | mechanism evidenced; **engine-level separation spike-pending** |
| Android rendering later | no renderer in the engine; `tty` unix backend `#[cfg(not(windows))]` (`src/tty/mod.rs:10–13`) | none | **UNVERIFIED / spike-pending** |
| Platform support / licensing / cadence / tests | crates.io: BSD/Linux/macOS/Windows; Apache-2.0; 0.26.0 published 2026-04-06, active cadence (`PER-VERSION.tsv`); `tests/ref.rs` | none | evidenced (metadata; tests not executed) |

**Honest statement.** The phase-1 spike (engine-independent) proved a real PTY can be owned, resized
and signalled and that candidate programs emit alt-screen / mouse / bracketed-paste / OSC / cursor /
SGR sequences on the wire. **TERM-002 (phase 2) now supplies the engine-level evidence for
`alacritty_terminal` 0.26.0** (`spikes/terminal-engine-phase2/results/REPORT.md` §3–§4, §7): 32 PASS /
1 UNSUPPORTED / 0 FAIL / 0 NOT_TESTED, with real-program captures (nvim, less, top, tmux) and
chunk-boundary invariance. Surviving limits (REPORT.md §8): `vte` is exercised only through
`alacritty_terminal`'s re-export, not as an independent parser boundary; palette→RGB is unverified
(no frontend); the mouse model approximates (`Press` vs `PressRelease`); bracketed-paste payload is an
input-path property and UNSUPPORTED here; selection is golden-value tested only; there is no Android
rendering, no GPU/renderer path, and no `TerminalOutput`/`TerminalSnapshot` protocol mapping yet; and
`gpui` was not built or run by this spike (UI-001 later built and ran the pin; see decision item 4).
Nothing here claims otherwise.

---

## Consequences

- **Transitive dependency/license surface (partially UNVERIFIED).** `alacritty_terminal` 0.26.0's
  published, cargo-normalized `Cargo.toml` lists direct dependencies `base64 0.22.0`, `bitflags 2.4.1`,
  `home 0.5.5`, `libc 0.2`, `log 0.4`, `parking_lot 0.12.0`, `polling 3.8.0`, `regex-automata 0.4.3`,
  `serde 1` (optional), `unicode-width 0.2.0`, `vte 0.15.0`; unix: `rustix 1.0.0`,
  `rustix-openpty 0.2.0`, `signal-hook 0.4.3`; windows: `miow 0.6.0`, `piper 0.2.1`,
  `windows-sys 0.59.0`. **There are no `alacritty_config`/`alacritty_config_derive` siblings in
  0.26.0** (`grep -c "alacritty_config" Cargo.toml` → 0). Licenses of the transitive crates were
  **not** fetched in this evidence set: **UNVERIFIED** — outstanding check: run a license audit
  (e.g. `cargo deny`/`cargo metadata` + license fetch) over the lockfile before distributing.
  Only `vte` (`Apache-2.0 OR MIT`) and `unicode-width` (`MIT OR Apache-2.0`) are directly evidenced
  here.
- **GPUI `libc` lockfile-hold (forward constraint).** UI-001 resolves the `gpui 0.2.2` build by holding
  `libc` at 0.2.189 in the `apps/linux` workspace lockfile (`cargo update -p libc --precise 0.2.189`;
  `apps/linux` is a separate workspace, so the core crates are unaffected). This is a **lockfile hold,
  not a source fix**: if a future UI slice makes `apps/linux` path-depend on a core crate that pins
  `libc = "=0.2.190"` (e.g. `relayable-server`), `libc` is forced back to 0.2.190 and the `ENOATTR`
  failure returns (exit 101) — that slice must then add the `[patch.crates-io]` `xattr` fix, or depend
  only on `libc`-neutral crates. Evidence: `apps/linux/evidence/TERM-002-remedy.md`.
- **API-stability risk.** `alacritty_terminal` is a 0.x library with no semver guarantee for library
  consumers: its own `CHANGELOG.md` marks breaking changes in bold (*"**Breaking changes are written in
  bold style.**"*) and the README calls the software *"at a **beta** level of readiness"*. Pinning an
  exact version is therefore mandatory, not optional.
- **Integration work Relayable must own.** The engine exposes an `EventListener` trait
  (`src/event.rs:103`, `fn send_event(&self, _event: Event) {}`) and a `notify` writer
  (`:86`); Relayable must implement a listener that maps `Event` values (`Event::Title`,
  `ClipboardStore/Load`, `ColorRequest`, `PtyWrite`, `TextAreaSizeRequest`, `Bell`, `Wakeup`,
  `ChildExit`, `MouseCursorDirty`, …) into the server/protocol layer. One obvious win:
  `Event::Title`, `ClipboardStore`, `Bell`, `ChildExit` map onto documented reference events
  `terminal.title_changed`, `terminal.clipboard_written`, `terminal.bell`, `terminal.child_exited`
  (`docs/compatibility/reference-behavior-matrix.md:391–398`).
- **Ownership boundaries.** The engine owns grid, reflow and selection *state*
  (`src/grid/resize.rs`, `src/selection.rs`), but Relayable owns the **bounds**: `full.md` §6.4
  requires *"Bound per-terminal scrollback and outgoing queues"* and §12 requires measuring memory per
  idle and per active terminal plus retained scrollback bytes **before** hard targets are set.
  `Config::scrolling_history` defaults to 10000 lines; that default is not a Relayable decision and
  must be replaced by a measured bound. The phase-1 observation (`results/raw/scrollback_*.txt`,
  single-run debug profile, no target) is an input to that measurement, not the measurement.
- **GPUI isolation.** `full.md` §5.3 requires GPUI-specific code under `apps/linux` and forbids GPUI
  types in shared crates; this ADR does not relax that. The GPUI pin is a build-input decision only.
- **GPUI dependency risk.** `gpui` 0.2.2's published manifest has 48 direct dependencies, default
  features `font-kit`, `wayland`, `x11`, `windows-manifest`, and it reaches Zed workspace siblings
  through package renames (dependency keys `collections`, `http_client`, `util`, `refineable`, … map to
  packages `gpui_collections`, `gpui_http_client`, `gpui_util`, `gpui_refineable`,
  `gpui_semantic_version`, `gpui_sum_tree`, `gpui_util_macros`, `gpui-macros`). The exact pins must be
  recorded in `Cargo.lock`; the `.cargo_vcs_info` `dirty: true` flag means the published tree is not a
  clean commit and the crate should always be consumed at the pinned version/checksum. That flag is
  self-quoted by this ADR from the extracted tarball (see Option H); independent verification covered
  resolution and checksum only (`docs/reviews/evidence/adr-003-spike-rerun.md` §7–8).
- **Pin-verification method for the future workspace:**
  `cargo update -p gpui --precise 0.2.2` (and equivalent for `alacritty_terminal`, `vte`,
  `portable-pty`), then commit `Cargo.lock` so the `checksum`
  `979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707` for `gpui` 0.2.2 (and the
  corresponding checksums for the other three) are enforced. `full.md` §5.3 still requires a minimal
  GPUI window to build and run in this repository/toolchain (UI-001) before the pin is relied on.
- **Residual uncertainty.** Only the engine-independent phase-1 spike exists; the engine-adapter
  spike (phase 2) was cancelled, so the *choice* is provisional and every engine-level §5.4 bullet is
  `spike-pending`. The `## What Evidence Would Cause Reconsideration` items are live.

---

## What Evidence Would Cause Reconsideration

Each trigger below is concrete and falsifiable.

1. **Spike failure on a named §5.4 bullet.** If `spikes/terminal-engine` shows `alacritty_terminal`
   0.26.0 fails a specific bullet (e.g. a `vim`/`nvim`/`tmux`/`htop` rendering or mode mismatch, or a
   bracketed-paste/mouse-mode round-trip failure observed in the reference behavior matrix), the engine
   decision is reopened.
2. **Reflow / selection / hyperlink gaps.** If the published mechanisms above do not actually cover the
   reference behaviors (e.g. reflow breaks scrollback at the tested sizes; copy-on-select cannot be
   reconstructed; OSC 8 link targets are not retrievable through the public API), reconsider.
3. **Measured memory growth violating §6.4.** If measurement (§12) shows unbounded or unacceptably
   large retained scrollback bytes with the chosen bound, reconsider the bound or the engine.
4. **`gpui` 0.2.2 does not build or run on this toolchain.** TERM-002 confirms it does not compile
   (`xattr`/`libc::ENOATTR`). Keep the pin if UI-001's contained remedy (hold `libc` ≤0.2.189 or patch
   `xattr`) makes it build and run; otherwise a revision move (a newer `gpui`, or a fork/republish that
   first satisfies the license check) is required.
5. **GPUI API churn.** Breaking API changes that force a revision move away from 0.2.2 constitute
   evidence to revisit the pin and possibly move to a different published artifact.
6. **`Glass-HQ/gpui` (or any fork) gains a license file + (if applicable) `NOTICE`,** making a
   revision pin legally viable and byte-reproducible; that would reopen Option I.
7. **A maintained Rust engine adds the missing scrollback/reflow/selection features.** If `termwiz` or
   `vt100` gains those features and stabilizes, re-evaluate Options B/C.
8. **`gpui-pre` / `gpui-ce` lineage stabilizes with a verified fork base** (e.g. `gpui-pre`'s
   `zed-rev = 93d7456…` is confirmed against upstream and a stable pin policy exists); that would
   reopen Option J.
9. **Android rendering proves the parser/state split unusable** (e.g. `alacritty_terminal`'s grid or
   `tty` path cannot compile/run on the target Android ABI), which would force the `vte`-only
   parser-boundary path and a Relayable-owned grid.
10. **`libghostty-vt` turns out to be the right boundary** (a verified license, a maintained Rust
    binding, and a passing build/spike) — this would reopen Option E and the reference-product
    alignment.

---

## Evidence

All fetches 2026-10-15 (UTC; window 16:59:15Z–17:01:21Z), User-Agent
`relayable-research/0.1 (read-only; contact: repo checkout)`; full manifest with URL, HTTP status,
retrieval time, sha256 and byte count is `/tmp/term001-evidence/MANIFEST.tsv` (103 fetches; 86× HTTP
200, 17× HTTP 404). Per-version data: `/tmp/term001-evidence/license-artifacts/PER-VERSION.tsv`
(331 rows). Extracted published sources: `/tmp/term001-evidence/extracted/<crate>-<ver>/`.

**crates.io API — version-level license/identity fields (verbatim):**

```
crates.io/api/v1/crates/vte/0.15.0                     -> version.license = "Apache-2.0 OR MIT"
crates.io/api/v1/crates/alacritty_terminal/0.26.0      -> version.license = "Apache-2.0"
crates.io/api/v1/crates/portable-pty/0.9.0             -> version.license = "MIT"
crates.io/api/v1/crates/gpui/0.2.2                     -> version.license = "Apache-2.0"
                                                          version.created_at = "2025-10-22T03:56:29.835700Z"
                                                          version.crate_size = 5291873
                                                          version.checksum = "979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707"
                                                          version.published_by.login = "mikayla-maki"
                                                          version.edition = "2024"
```

**crate-level vs version-level (not collapsed):**

```
$ python3 -c "import json; c=json.load(open('crates-io-gpui'))['crate']; print('license key present:', 'license' in c)"
crates-io-gpui -> license key present: False | value: '<absent>'
```

i.e. the crates.io **crate** object for `gpui` (and for `vte`, `alacritty_terminal`, `portable-pty`,
`termwiz`, `vt100`, `avt`) carries **no `license` field**, while the **version** object carries
`version.license = "Apache-2.0"`. (The evidence set's prose records this as "crate-level `license` =
`null`"; the raw JSON shows the key is absent.)

**Tarball checksums vs crates.io-recorded checksums:** all 14 downloaded `.crate` files MATCH their
recorded checksum (see `license-artifacts/CHECKSUM-CROSSCHECK.txt`), including
`gpui-0.2.2.crate` = `979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`.

**`gpui` 0.2.2 tarball facts:**

```
$ cat gpui-0.2.2/.cargo_vcs_info.json
{ "git": { "sha1": "69e2130295c2649963eb639fc70b4f2ee8ea1624", "dirty": true }, "path_in_vcs": "crates/gpui" }
# self-quoted by this ADR from the extracted tarball (evidence set not repo-tracked); independent
#   verification covered pin resolution + checksum only (docs/reviews/evidence/adr-003-spike-rerun.md §7–8)
$ sha256sum gpui-0.2.2/LICENSE-APACHE raw/zed-main-LICENSE-APACHE
752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a  gpui-0.2.2/LICENSE-APACHE
752daf2fb234ca4a1fa372c073fe127f44b7b90fd2529ae44273a64f9d53da7a  zed-main-LICENSE-APACHE
```

`gpui` `Cargo.toml` (published): `license = "Apache-2.0"`, `repository = "https://github.com/zed-industries/zed"`,
`publish = true`, `edition = "2024"`, 48 direct dependencies; sibling workspace crates are reached via
renames, e.g.:

```
[dependencies.collections]
version = "0.2.2"
package = "gpui_collections"
[dependencies.http_client]
version = "0.2.2"
package = "gpui_http_client"
[dependencies.util]
version = "0.2.2"
package = "gpui_util"
```

**Zed repo-level license (GitHub API, 2026-10-15):**

```
GET https://api.github.com/repos/zed-industries/zed            -> license = {"key":"other","name":"Other","spdx_id":"NOASSERTION","url":null,...}
GET https://api.github.com/repos/zed-industries/zed/license    -> license.spdx_id = "NOASSERTION"; path = "LICENSE-APACHE"; sha = "461a0fe5ba20a096c281a2686da08fef79f534a3"; size = 10768
```

The upstream root `LICENSE-APACHE` carries a `Copyright 2022 - 2025 Zed Industries, Inc.` header and
is **not** byte-identical to the canonical text at `https://www.apache.org/licenses/LICENSE-2.0.txt`
(`cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`); canonical has 1 occurrence of
`APPENDIX: How to apply`, `vte`'s shipped `LICENSE-APACHE` has 0. Whether each is legally complete was
not adjudicated (out of scope).

**Apache-2.0 §4 obligations (verbatim, `https://www.apache.org/licenses/LICENSE-2.0.txt`, 2026-10-15):**

```text
 4. Redistribution. You may reproduce and distribute copies of the
    Work or Derivative Works thereof in any medium, with or without
    modifications, and in Source or Object form, provided that You
    meet the following conditions:

    (a) You must give any other recipients of the Work or
        Derivative Works a copy of this License; and

    (b) You must cause any modified files to carry prominent notices
        stating that You changed the files; and

    (c) You must retain, in the Source form of any Derivative Works
        that You distribute, all copyright, patent, trademark, and
        attribution notices from the Source form of the Work, ...

    (d) If the Work includes a "NOTICE" text file as part of its
        distribution, then any Derivative Works that You distribute must
        include a readable copy of the attribution notices contained
        within such NOTICE file, ...
```

**No `NOTICE` files in any inspected artifact:**

```
$ find /tmp/term001-evidence/extracted -iname 'NOTICE*' -type f | wc -l
0
```

**Registry existence checks (negative evidence):**

```
GET https://crates.io/api/v1/crates/wezterm-term -> 404 {"errors":[{"detail":"crate `wezterm-term` does not exist"}]}
GET https://crates.io/api/v1/crates/libvterm     -> 404 {"errors":[{"detail":"crate `libvterm` does not exist"}]}
GET https://crates.io/api/v1/crates/crosspty     -> 404 {"errors":[{"detail":"crate `crosspty` does not exist"}]}
GET https://api.github.com/repos/Glass-HQ/gpui/license -> 404 {"message":"Not Found",...,"status":"404"}
GET https://raw.githubusercontent.com/Glass-HQ/gpui/main/{LICENSE,LICENSE.md,LICENSE-APACHE,LICENSE-MIT,LICENSE-GPL,COPYING,NOTICE} -> all 404
```

**Chosen-engine published-source checks (exact commands and real output):**

```
$ cd /tmp/term001-evidence/extracted/alacritty_terminal-0.26.0/alacritty_terminal-0.26.0
$ grep -n "pub struct Config\|pub scrolling_history\|pub fn new<D: Dimensions>\|pub fn swap_alt" src/term/mod.rs
334:pub struct Config {
336:    pub scrolling_history: usize,
410:    pub fn new<D: Dimensions>(config: Config, dimensions: &D, event_proxy: T) -> Term<T> {
714:    pub fn swap_alt(&mut self) {
$ sed -n '356,362p' src/term/mod.rs
impl Default for Config {
    fn default() -> Self {
        Self {
            scrolling_history: 10000,
...
$ sed -n '1,16p' src/grid/resize.rs
//! Grid resize and reflow.
...
    pub fn resize<D>(&mut self, reflow: bool, lines: usize, columns: usize)
$ grep -n "pub struct Selection" src/selection.rs
119:pub struct Selection {
$ grep -n "pub struct Hyperlink\|pub fn set_hyperlink\|pub fn hyperlink" src/term/cell.rs
44:pub struct Hyperlink {
202:    pub fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>) {
219:    pub fn hyperlink(&self) -> Option<Hyperlink> {
$ grep -n "BRACKETED_PASTE\|ALT_SCREEN\|MOUSE_MODE" src/term/mod.rs | head -4
61:        const BRACKETED_PASTE         = 1 << 4;
69:        const ALT_SCREEN              = 1 << 12;
80:        const MOUSE_MODE              = Self::MOUSE_REPORT_CLICK.bits() | Self::MOUSE_MOTION.bits() | Self::MOUSE_DRAG.bits();
$ sed -n '10,13p' src/tty/mod.rs
#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use self::unix::*;
$ grep -n "pub trait EventListener\|pub enum Event" src/event.rs
14:pub enum Event {
103:pub trait EventListener {
$ grep -c "alacritty_config" Cargo.toml
0
```

**Rejected-candidate published-source checks:**

```
### termwiz 0.23.3
$ grep -rln "reflow" src/                                        -> (no output; exit 1)
$ grep -n "wild sweeping" README.md
7:It is currently in active development and subject to fairly wild sweeping
$ grep -n "FocusTracking" src/escape/csi.rs
824:    FocusTracking = 1004,
$ grep -rn "FocusGained\|FocusLost" src/                         -> (no output; exit 1)
$ grep -c "wezterm-" Cargo.toml
8
$ grep -c "^\[dependencies\." Cargo.toml
34

### vt100 0.16.2
$ grep -rln "reflow" src/                                        -> (no output; exit 1)
$ grep -rln "hyperlink\|Hyperlink" src/                          -> (no output; exit 1)
$ grep -n "scrollback_len" src/grid.rs
14:    scrollback_len: usize,
19:    pub fn new(size: Size, scrollback_len: usize) -> Self {
$ grep -n "pub fn bracketed_paste\|pub fn alternate_screen" src/screen.rs
548:    pub fn alternate_screen(&self) -> bool {
572:    pub fn bracketed_paste(&self) -> bool {

### avt 0.18.0
$ grep -n "pub struct Terminal\|scrollback_limit\|fn reflow\|pub fn gc\|pub fn resize" src/terminal.rs
20:pub struct Terminal {
26:    scrollback_limit: Option<usize>,
94:    pub fn new((cols, rows): (usize, usize), scrollback_limit: Option<usize>) -> Self {
343:    pub fn gc(&mut self) -> Box<dyn Iterator<Item = Line> + '_> {
525:    pub fn resize(&mut self, cols: usize, rows: usize) -> bool {
565:    fn reflow(&mut self) {
$ grep -rln "selection\|hyperlink\|mouse\|bracketed" src/         -> (no output; exit 1)
$ grep -n "input handling" README.md
24:- input handling,

### termina 0.4.0
$ grep -n "pub trait Terminal" src/terminal.rs
95:pub trait Terminal: io::Write {
$ grep -rln "pub struct Screen\|struct Grid" src/                -> (no output; exit 1)
```

**vte 0.15.0 parser/OSC checks:**

```
$ grep -n "pub struct Hyperlink\|osc_dispatch\|BracketedPaste" src/ansi.rs
51:pub struct Hyperlink {
701:    fn set_hyperlink(&mut self, _: Option<Hyperlink>) {}
915:            2004 => Self::Named(NamedPrivateMode::BracketedPaste),
965:    BracketedPaste = 2004,
1329:    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
$ grep -n "Hyperlink\|b\"52\"" src/ansi.rs
1392:            // Hyperlink.
1418:                self.handler.set_hyperlink(Some(Hyperlink { id, uri }));
1483:            b"52" => {
```

**TERM-001 phase-1 spike artifacts (committed, engine-independent) — cited by this ADR:**

```
$ git show --stat 085375c5fe02e79fbecb6d057a5aea306c7a3565 | tail -1
 57 files changed, 7674 insertions(+)
# every file under spikes/terminal-engine/** (target/ is gitignored)
# provenance stamped in each result: git_head b96da2766e8750a77feee3beec0d84c2ce28f2fc
#   (the pre-commit tree the harness ran against), rustc/cargo 1.98.1, Linux 7.2.5-3-omarchy x86_64

$ sed -n '/---- output ----/,$p' spikes/terminal-engine/results/raw/pty_size.txt
REQUESTED_INITIAL_WINSIZE=24 rows x 80 cols
SET_WINSIZE_CALL=ioctl(master, TIOCSWINSZ, {ws_row:40, ws_col:120})
SET_WINSIZE_RC=0
CHILD_STTY_SIZE_AFTER=40 120
SIGWINCH_OBSERVED=true
CHILD_EXIT_CODE=Some(0)
VERDICT=PASS

$ grep -n "CHECK alt_screen_1049_enter\|CHECK mouse_1002_enable\|CHECK mouse_1006_enable\|CHECK bracketed_paste_enable\|CHECK osc_color\|CHECK decscusr\|CHECK sgr\|CHECK mouse_1000_enable" spikes/terminal-engine/results/raw/nvim.summary.txt
CHECK alt_screen_1049_enter = OBSERVED count=1 first_bytes=[1b 5b 3f 31 30 34 39 68]
CHECK mouse_1000_enable = NOT_OBSERVED
CHECK mouse_1002_enable = OBSERVED count=2 first_bytes=[1b 5b 3f 31 30 30 32 68]
CHECK mouse_1006_enable = OBSERVED count=2 first_bytes=[1b 5b 3f 31 30 30 36 68]
CHECK bracketed_paste_enable = OBSERVED count=1 first_bytes=[1b 5b 3f 32 30 30 34 68]
CHECK osc_color = OBSERVED count=2 first_bytes=[1b 5d 31 31 3b 3f 07]
CHECK decscusr = OBSERVED count=7 first_bytes=[1b 5b 31 20 71]
CHECK sgr = OBSERVED count=337 first_bytes=[1b 5b 30 6d]

$ grep -n "osc_title\|osc_hyperlink" spikes/terminal-engine/results/raw/nvim_title.seq spikes/terminal-engine/results/raw/osc8_ls.seq
# nvim_title.seq: OSC 0 OBSERVED (real window title from `nvim '+set title'`)
# osc8_ls.seq:35:  # CHECK osc_hyperlink = OBSERVED count=4 first_offset=0 first_bytes=[1b 5d 38 3b 3b ...]
# osc8_ls.seq:42-45: offset=0 len=154 / offset=163 len=7 / offset=172 len=153 / offset=333 len=7  (OSC 8 ... ST, incl. 8;; close)

$ grep -H "RETENTION_MODE\|RETAINED_BYTES\|RSS_VMHWM_KB" spikes/terminal-engine/results/raw/scrollback_*.txt
scrollback_bounded.txt:RETENTION_MODE=bounded cap_lines=1000
scrollback_bounded.txt:RETAINED_BYTES=56000
scrollback_bounded.txt:RSS_VMHWM_KB=3004
scrollback_discard.txt:RETENTION_MODE=discard
scrollback_discard.txt:RETAINED_BYTES=0
scrollback_discard.txt:RSS_VMHWM_KB=2884
scrollback_unbounded.txt:RETENTION_MODE=unbounded
scrollback_unbounded.txt:RETAINED_BYTES=68329016
scrollback_unbounded.txt:RSS_VMHWM_KB=69332
```

Method and limitations (from the spike's own provenance headers and `results/README.md`): payload
`N = 64 MiB = 1,220,161 lines × 55 B = 67,108,855 B`, `68,329,016 B` actually read from the pty because
the slave applies `OPOST|ONLCR` (`+1 byte/line`); RSS sampled from `/proc/<pid>/status` `VmRSS` every
25 ms plus `VmHWM` at end; **debug profile, single run each, this probe process only — an
observation, not a benchmark and not a memory target**. Phase 1 ran `cargo build`, `cargo fmt --check`
and `cargo clippy --all-targets` at exit 0 and `./scripts/run.sh` at `aggregate_failures=0`
(`results/raw/checks/*`, `results/raw/run_meta.txt`); these are the spike crate's own checks, **not**
checks of any engine or of the Relayable workspace. Phase 2 (per-engine adapters) was cancelled and
produced nothing (`spikes/terminal-engine/STATUS.md`, `README.md`, `results/README.md`).

**Reproducibility limit of the committed raw captures.** The `results/raw/*` set is a historical
artifact of a specific tree/run, not a reproducible-from-current-HEAD output. All files carry the
same provenance `git_head b96da276…` (the pre-integration base the TERM-001 branch started from, not
current HEAD), and the `capture_stream`/`clippy` commands embed the since-retired worktree path
`…/worktrees/relayable/research-term-001-adr-003/…` (e.g. `results/raw/checks/cargo_clippy.txt:11`),
which also leaks into the OSC 8 `file://` payload (`results/raw/osc8_ls.seq` lines 35, 42, 44).
`scripts/run.sh` deletes and regenerates `results/raw/` and re-stamps `git_head`/paths, so re-running
it **replaces** the committed bytes rather than reproducing them. Only the self-contained `pty_size`
and `esc_selftest` probes were independently reproduced from the current checkout; the byte-capture
claims depend on the host program versions at capture time and are historical observations, not
independently re-verified (`docs/reviews/evidence/adr-003-spike-rerun.md` §6).

**Worktree fact recorded by this ADR:** `docs/adr/` did not exist before this file was written
(`ls docs/adr` → `No such file or directory`). `full.md` §5.2 defines the required ADR sections; no
existing ADR template was present, so the five headings below (`Context`, `Options Considered`,
`Decision`, `Consequences`, `What Evidence Would Cause Reconsideration`) plus this appendix are the
adopted format.
