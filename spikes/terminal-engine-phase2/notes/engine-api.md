# Engine API notes — source-verified interface contract (TERM-002 phase A)

This file is the interface contract for the phase-B engine-adapter implementer.
**Every signature below was read from the fetched crate source on this host**, not
from memory or the web. Anything not confirmed from source is marked `UNVERIFIED`.

Authoritative source roots (fetched by `cargo fetch` in this crate):

```
CARGO_SRC=/home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f
  $CARGO_SRC/alacritty_terminal-0.26.0
  $CARGO_SRC/vte-0.15.0
  $CARGO_SRC/portable-pty-0.9.0
```

Pinned versions (verified in `../Cargo.lock`):
`alacritty_terminal 0.26.0`, `vte 0.15.0`, `portable-pty 0.9.0`
(comparison engines present but not documented here: `vt100 0.16.2`,
`termwiz 0.23.3`, `avt 0.18.0`).

---

## 1. alacritty_terminal 0.26.0

### 1.0 Re-exports and the vte relationship

`/home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/alacritty_terminal-0.26.0/src/lib.rs`

- line 17: `pub use crate::grid::Grid;`
- line 18: `pub use crate::term::Term;`
- line 20: `pub use vte;`
  → `alacritty_terminal::vte` **is** the external `vte` 0.15.0 crate. Use this
  path (`alacritty_terminal::vte::ansi`) or the direct `vte` dependency; both
  resolve to the same pinned 0.15.0 (one copy in `Cargo.lock`).
- Public modules: `event`, `event_loop`, `grid`, `index`, `selection`, `sync`,
  `term`, `thread`, `tty`, `vi_mode`.

**CRITICAL (task question):** alacritty 0.26.0 does **not** use
`vte::Perform`/`advance`. It uses the higher-level **`vte::ansi::Handler`** API
with **`vte::ansi::Processor`**. See §1.5.

### 1.1 Constructing `Term`

`.../alacritty_terminal-0.26.0/src/term/mod.rs`

- line 268: `pub struct Term<T> { ... }`
- line 410:
  ```rust
  pub fn new<D: Dimensions>(config: Config, dimensions: &D, event_proxy: T) -> Term<T>
  ```
  - `config: Config` is taken **by value**.
  - `dimensions: &D` requires the `Default`-free trait `Dimensions` (see §1.4);
    pass your own size type (or `term::test::TermSize`).
  - `event_proxy: T` is your `EventListener` implementation (§1.2).

`Config` — same file:

- line 334: `pub struct Config {`
- line 336: `pub scrolling_history: usize,`
- line 339: `pub default_cursor_style: CursorStyle,`
- line 342: `pub vi_mode_cursor_style: Option<CursorStyle>,`
- line 347: `pub semantic_escape_chars: String,`
- line 350: `pub kitty_keyboard: bool,`
- line 353: `pub osc52: Osc52,`
- `impl Default for Config` begins line 356; `scrolling_history: 10000,` is the
  default at line 359.

So the intended construction (`tests/ref.rs:96-113` is a real in-crate example):

```rust
use alacritty_terminal::event::{EventListener};
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi;

struct Proxy; // impl EventListener
let options = Config { scrolling_history: 10_000, ..Default::default() };
let size = /* your Dimensions type */;
let mut term = Term::new(options, &size, Proxy);
let mut parser: ansi::Processor = ansi::Processor::new();
parser.advance(&mut term, &bytes);
```

`term::test::TermSize` is a real public helper if you do not want to define a
`Dimensions` type (`src/term/mod.rs` line 2416 `pub mod test {`, line 2425
`pub struct TermSize { pub columns: usize, pub screen_lines: usize }`, line 2431
`pub fn new(columns: usize, screen_lines: usize) -> Self`). It is available
because `tests/ref.rs:11` imports `alacritty_terminal::term::test::TermSize` and
`term::test` is a normal `pub mod`, not `#[cfg(test)]`.

### 1.2 `EventListener` trait (events: title / bell / wakeup / …)

`.../alacritty_terminal-0.26.0/src/event.rs` (whole file is 110 lines)

- line 14: `pub enum Event {`
  Variants (exact names): `MouseCursorDirty`, `Title(String)`, `ResetTitle`,
  `ClipboardStore(ClipboardType, String)`,
  `ClipboardLoad(ClipboardType, Arc<dyn Fn(&str) -> String + Sync + Send + 'static>)`,
  `ColorRequest(usize, Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>)`,
  `PtyWrite(String)`,
  `TextAreaSizeRequest(Arc<dyn Fn(WindowSize) -> String + Sync + Send + 'static>)`,
  `CursorBlinkingChange`, `Wakeup`, `Bell`, `Exit`, `ChildExit(ExitStatus)`.
- line 103: `pub trait EventListener {`
- line 104: `fn send_event(&self, _event: Event) {}`  ← **the only method**
- line 108: `pub struct VoidListener;` (line 110 `impl EventListener for VoidListener {}`)

Mapping to the requested escape behaviours:

| Behaviour | How it surfaces | Source |
|---|---|---|
| OSC 0 / OSC 2 window title | `Event::Title(String)` | `src/event.rs:19`; handler `fn set_title` at `src/term/mod.rs:2221` |
| title reset | `Event::ResetTitle` | `src/event.rs:22` |
| bell | `Event::Bell` | `src/event.rs:45`; handler `fn bell` at `src/term/mod.rs:1437` |
| redraw needed | `Event::Wakeup` | `src/event.rs:42` |
| OSC 8 hyperlink | **NOT an `EventListener` event.** `Handler::set_hyperlink` stores it on the cursor template, hence per-cell; read it back via `Cell::hyperlink()` (§1.6). | `src/term/mod.rs:1874`, `src/term/cell.rs:219` |

There is no `fn on_title` / `fn on_bell` / `fn on_wakeup` method: the trait has
exactly one method, and all of the above arrive as `Event` values through it.
(This directly answers the task's "exact method names" question.)

### 1.3 Modes (`TermMode`)

`.../alacritty_terminal-0.26.0/src/term/mod.rs`

- line 55: `pub struct TermMode: u32 {` (a `bitflags!` type; the `bitflags` macro
  block starts line 53)
- Constants (exact names, lines 56-80):
  `NONE=0`, `SHOW_CURSOR=1`, `APP_CURSOR=1<<1`, `APP_KEYPAD=1<<2`,
  `MOUSE_REPORT_CLICK=1<<3`, `BRACKETED_PASTE=1<<4`, `SGR_MOUSE=1<<5`,
  `MOUSE_MOTION=1<<6`, `LINE_WRAP=1<<7`, `LINE_FEED_NEW_LINE=1<<8`,
  `ORIGIN=1<<9`, `INSERT=1<<10`, `FOCUS_IN_OUT=1<<11`, `ALT_SCREEN=1<<12`,
  `MOUSE_DRAG=1<<13`, `UTF8_MOUSE=1<<14`, `ALTERNATE_SCROLL=1<<15`, `VI=1<<16`,
  `URGENCY_HINTS=1<<17`, `DISAMBIGUATE_ESC_CODES=1<<18`,
  `REPORT_EVENT_TYPES=1<<19`, `REPORT_ALTERNATE_KEYS=1<<20`,
  `REPORT_ALL_KEYS_AS_ESC=1<<21`, `REPORT_ASSOCIATED_TEXT=1<<22`,
  plus composed `MOUSE_MODE`, `KITTY_KEYBOARD_PROTOCOL`, `ANY`.
- `impl Default for TermMode` (line 113) defaults to
  `SHOW_CURSOR | LINE_WRAP | ALTERNATE_SCROLL | URGENCY_HINTS`.
- Accessor: line 709 `pub fn mode(&self) -> &TermMode`.

Usage: `term.mode().contains(TermMode::ALT_SCREEN)`,
`term.mode().contains(TermMode::BRACKETED_PASTE)`, etc.

### 1.4 Grid / size / cursor

`.../alacritty_terminal-0.26.0/src/grid/mod.rs`

- line 110: `pub struct Grid<T> {`
- line 113: `pub cursor: Cursor<T>,`  (line 117 `pub saved_cursor: Cursor<T>,`)
- line 141: `pub fn new(lines: usize, columns: usize, max_scroll_limit: usize) -> Grid<T>`
- line 422: `pub fn display_iter(&self) -> GridIterator<'_, T>`
- line 432: `pub fn display_offset(&self) -> usize`
- line 73: `pub enum Scroll { Delta(i32), PageUp, PageDown, Top, Bottom }`
- line 486:
  ```rust
  pub trait Dimensions {
      fn total_lines(&self) -> usize;
      fn screen_lines(&self) -> usize;
      fn columns(&self) -> usize;
      fn last_column(&self) -> Column { ... }      // provided
      fn topmost_line(&self) -> Line { ... }       // provided, Line(-(history_size))
      fn bottommost_line(&self) -> Line { ... }    // provided, Line(screen_lines - 1)
      fn history_size(&self) -> usize { ... }      // provided
  }
  ```

`.../alacritty_terminal-0.26.0/src/term/mod.rs`

- `impl<T> Dimensions for Term<T>` line 1042 (`columns` 1044, `screen_lines`
  1049, `total_lines` 1054).
- line 645: `pub fn grid(&self) -> &Grid<Cell>`
- line 650: `pub fn grid_mut(&mut self) -> &mut Grid<Cell>`
- line 655: `pub fn resize<S: Dimensions>(&mut self, size: S)`
- line 389: `pub fn scroll_display(&mut self, scroll: Scroll) where T: EventListener`
- line 637: `pub fn renderable_content(&self) -> RenderableContent<'_> where T: EventListener`
- line 942: `pub fn cursor_style(&self) -> CursorStyle`
- line 952: `pub fn colors(&self) -> &Colors`
- line 529: `pub fn selection_to_string(&self) -> Option<String>`
- line 558: `pub fn bounds_to_string(&self, start: Point, end: Point) -> String`

**`Term::size()` does NOT exist** — `grep` for `pub fn size`/`fn size(` in
`src/term/mod.rs` returns nothing. Read the size via the `Dimensions` trait
(`term.columns()`, `term.screen_lines()`, `term.total_lines()`) or via
`term.grid().columns()` / `.screen_lines()`. This is a correction to the task's
starting assumption, not an omission.

Cursor position: `term.grid().cursor.point` (a `Point`). `Cursor<T>.point:
Point` is at `src/grid/mod.rs:36`. (For rendering, `renderable_content()` at
line 637 returns `RenderableContent` — defined `src/term/mod.rs:2393` — whose
`cursor: RenderableCursor` field (defined line 2365) already applies vi-mode and
wide-char handling; use it rather than re-deriving.)

`Grid` indexing and scrollback semantics:

- line 453: `impl<T> Index<Line> for Grid<T> { type Output = Row<T>; }`
- line 469: `impl<T> Index<Point> for Grid<T> { type Output = T; }` (i.e. `grid[point]` is `&Cell`)
- `Row<T>` indexes by `Column`: `.../alacritty_terminal-0.26.0/src/grid/row.rs:193`
  `impl<T> Index<Column> for Row<T> { type Output = T; }`
- So `&term.grid()[Line(-1)][Column(0)]` is a cell one line into scrollback, and
  `&term.grid()[term.grid().cursor.point]` is the cursor cell.
- line 554: `pub struct Indexed<T> { pub point: Point, pub cell: T }`
- line 569: `pub struct GridIterator<'a, T>`; line 592
  `impl<'a, T> Iterator for GridIterator<'a, T> { type Item = Indexed<&'a T>; }`

`.../alacritty_terminal-0.26.0/src/index.rs`

- line 136: `pub struct Line(pub i32);`
- line 230: `pub struct Column(pub usize);`
- line 50: `pub struct Point<L = Line, C = Column> { pub line: L, pub column: C }`

Scrollback / viewport sign conventions (from the `Dimensions` provided methods and
`Grid::display_iter`):
- viewport top line = `Line(0)`; bottom viewport line = `Line(screen_lines - 1)`.
- scrollback is **negative**: `Line(-1)` is one line above the viewport; the
  most distant history line is `topmost_line() == Line(-(history_size as i32))`.
- `bottommost_line() == Line(screen_lines as i32 - 1)`.
- `display_offset()` is the number of lines scrolled into history; `display_iter`
  starts at `Line(-(display_offset as i32) - 1)` (`src/grid/mod.rs:426`).
- `point.to_viewport(display_offset)`-style helpers exist as free functions:
  `pub fn point_to_viewport(display_offset: usize, point: Point) -> Option<Point<usize>>`
  at `src/term/mod.rs:124`, and
  `pub fn viewport_to_point(display_offset: usize, point: Point<usize>) -> Point`
  at `src/term/mod.rs:131`.

### 1.5 Feeding bytes into the engine (the exact parser API)

Confirmed from alacritty's own event loop and integration test:

- `.../alacritty_terminal-0.26.0/src/event_loop.rs:21`: `use vte::ansi;`
- `.../src/event_loop.rs:404`: `parser: ansi::Processor,`
- `.../src/event_loop.rs:154`: `state.parser.advance(&mut **terminal, &buf[..unprocessed]);`
- `.../tests/ref.rs:113-116`:
  ```rust
  let mut terminal = Term::new(options, &size, Mock);
  let mut parser: ansi::Processor = ansi::Processor::new();
  parser.advance(&mut terminal, &recording);
  ```

Conclusion: **implement `EventListener` (not `Handler`); alacritty's `Term<T>`
already implements `vte::ansi::Handler`** (`impl<T: EventListener> Handler for
Term<T>` at `src/term/mod.rs:1059`). Create a `vte::ansi::Processor`, then call
`processor.advance(&mut term, &bytes)` for each PTY read chunk.

### 1.6 Cell contents and OSC 8 hyperlinks

`.../alacritty_terminal-0.26.0/src/term/cell.rs`

- line 134: `pub struct Cell {`
- line 135: `pub c: char,`
- line 136: `pub fg: Color,`
- line 137: `pub bg: Color,`
- line 138: `pub flags: Flags,`
- line 139: `pub extra: Option<Arc<CellExtra>>,`
- line 219: `pub fn hyperlink(&self) -> Option<Hyperlink>`
- line 44: `pub struct Hyperlink { ... }` (alacritty's own; private fields)
- line 54: `pub fn id(&self) -> &str`
- line 58: `pub fn uri(&self) -> &str`
- line 15: `pub struct Flags: u16 {` with constants
  `INVERSE`, `BOLD`, `ITALIC`, `BOLD_ITALIC`, `UNDERLINE`, `WRAPLINE`,
  `WIDE_CHAR`, `WIDE_CHAR_SPACER`, `DIM`, `DIM_BOLD`, `HIDDEN`, `STRIKEOUT`,
  `LEADING_WIDE_CHAR_SPACER`, `DOUBLE_UNDERLINE`, `UNDERCURL`,
  `DOTTED_UNDERLINE`, `DASHED_UNDERLINE`, `ALL_UNDERLINES`.
- `Color` / `NamedColor` / `Rgb` live in vte (re-used here via
  `use crate::vte::ansi::{Color, Hyperlink as VteHyperlink, NamedColor};`,
  `src/term/cell.rs:10`); see §2.
- `CellExtra.zerowidth: Vec<char>` and `underline_color: Option<Color>` are
  private; access via `Cell::zerowidth()` (line 165) and
  `Cell::underline_color()` (line 205).

`Hyperlink` in the raw vte sense: `.../vte-0.15.0/src/ansi.rs:51`
```rust
pub struct Hyperlink { pub id: Option<String>, pub uri: String }
```

---

## 2. vte 0.15.0

`.../vte-0.15.0/src/lib.rs` and `.../vte-0.15.0/src/ansi.rs`.
Feature note: `vte/Cargo.toml:50` `default = ["std"]`; the `ansi` module is
behind the `ansi` feature. alacritty enables `features = ["std", "ansi"]` with
`default-features = false` (`alacritty_terminal-0.26.0/Cargo.toml`, `[dependencies.vte]`).

There are **two** layers. `alacritty_terminal` uses layer 2.

### 2.1 Layer 2 — `ansi` (what a terminal frontend should implement)

- line 274: `pub struct Processor<T: Timeout = StdSyncHandler> { ... }`
- line 287: `pub fn new() -> Self`
- line 298:
  ```rust
  pub fn advance<H>(&mut self, handler: &mut H, bytes: &[u8])
  where
      H: Handler,
  ```
- line 315: `pub fn stop_sync<H>(&mut self, handler: &mut H) where H: Handler`
- line 330: `pub fn sync_bytes_count(&self) -> usize`
- line 495: `pub trait Handler {`
  Relevant methods (exact signatures, defaults are no-ops):
  - line 497: `fn set_title(&mut self, _: Option<String>) {}`
  - line 500: `fn set_cursor_style(&mut self, _: Option<CursorStyle>) {}`
  - line 503: `fn set_cursor_shape(&mut self, _shape: CursorShape) {}`
  - line 506: `fn input(&mut self, _c: char) {}`
  - line 559: `fn bell(&mut self) {}`
  - line 701: `fn set_hyperlink(&mut self, _: Option<Hyperlink>) {}`
  - (plus goto/move/scroll/mode/csi/osc methods; the full trait is at lines
    495-707.)
- line 821: `pub struct CursorStyle { pub shape: CursorShape, pub blinking: bool }`
- line 828: `pub enum CursorShape { Block, Underline, Beam, HollowBlock, Hidden }`
  (`Block` is `#[default]`)
- line 51: `pub struct Hyperlink { pub id: Option<String>, pub uri: String }`
- line 1128: `pub enum Color { Named(NamedColor), Spec(Rgb), Indexed(u8) }`
- line 1013: `pub enum NamedColor { Black, Red, Green, Yellow, Blue, Magenta,
  Cyan, White, BrightBlack, BrightGreen, BrightYellow, BrightBlue,
  BrightMagenta, BrightCyan, BrightWhite, Foreground = 256, Background, Cursor, ... }`
- line 56: `pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }`

A custom backend that does not use alacritty would implement `vte::ansi::Handler`
and drive it with `Processor::advance`. With alacritty, `Term` already does this
and you implement only `EventListener` (§1.2).

### 2.2 Layer 1 — raw `Perform` protocol parser (NOT used by alacritty 0.26)

- line 55: `pub struct Parser<const OSC_RAW_BUF_SIZE: usize = MAX_OSC_RAW> { ... }`
- line 74: `pub fn new() -> Parser`
- line 109: `pub fn advance<P: Perform>(&mut self, performer: &mut P, bytes: &[u8])`
- line 140: `pub fn advance_until_terminated<P: Perform>(...)` (exact full signature
  at that line; returns bytes processed)
- line 761: `pub trait Perform {` with methods `print(char)`, `execute(u8)`,
  `hook(&Params, &[u8], bool, char)`, `put(u8)`, `unhook()`,
  `osc_dispatch(&[&[u8]], bool)`, `csi_dispatch(&Params, &[u8], bool, char)`,
  `esc_dispatch(&[u8], bool, u8)`.

Use this only if you want the raw parser; the alacritty path does not.

---

## 3. portable-pty 0.9.0

`.../portable-pty-0.9.0/src/lib.rs`, `.../src/cmdbuilder.rs`, `.../src/unix.rs`.
A complete, compiling usage example ships with the crate:
`.../portable-pty-0.9.0/examples/whoami.rs`.

- line 400: `pub fn native_pty_system() -> Box<dyn PtySystem + Send>`
- line 405: `pub type NativePtySystem = unix::UnixPtySystem;` (Unix)
- line 263: `pub trait PtySystem: Downcast {`
- line 267: `fn openpty(&self, size: PtySize) -> anyhow::Result<PtyPair>;`
- line 253: `pub struct PtyPair {` — fields:
  - `pub slave: Box<dyn SlavePty + Send>,`
  - `pub master: Box<dyn MasterPty + Send>,`
- line 63: `pub struct PtySize {`
  - line 65: `pub rows: u16,`
  - line 67: `pub cols: u16,`
  - line 70: `pub pixel_width: u16,`
  - line 73: `pub pixel_height: u16,`
  - `Default` is rows 24 / cols 80 / 0 / 0.
- line 88: `pub trait MasterPty: Downcast + Send {`
  - line 92: `fn resize(&self, size: PtySize) -> Result<(), Error>;`
  - line 94: `fn get_size(&self) -> Result<PtySize, Error>;`
  - line 97: `fn try_clone_reader(&self) -> Result<Box<dyn std::io::Read + Send>, Error>;`
  - line 102: `fn take_writer(&self) -> Result<Box<dyn std::io::Write + Send>, Error>;`
  - Unix-only extras: `fn process_group_leader(&self) -> Option<libc::pid_t>;`,
    `fn as_raw_fd(&self) -> Option<unix::RawFd>;`,
    `fn tty_name(&self) -> Option<std::path::PathBuf>;`,
    `fn get_termios(&self) -> Option<nix::sys::termios::Termios>;`
- line 163: `pub trait SlavePty {`
- line 165: `fn spawn_command(&self, cmd: CommandBuilder) -> Result<Box<dyn Child + Send + Sync>, Error>;`
- line 130: `pub trait Child: std::fmt::Debug + ChildKiller + Downcast + Send {`
  - line 135: `fn try_wait(&mut self) -> IoResult<Option<ExitStatus>>;`
  - line 138: `fn wait(&mut self) -> IoResult<ExitStatus>;`
  - line 141: `fn process_id(&self) -> Option<u32>;`
- line 150: `pub trait ChildKiller: std::fmt::Debug + Downcast + Send {`
  - line 152: `fn kill(&mut self) -> IoResult<()>;`
  - line 157: `fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync>;`
- line 170: `pub struct ExitStatus {` with `pub fn success(&self) -> bool` (line 190),
  `pub fn exit_code(&self) -> u32` (line 198), `pub fn signal(&self) -> Option<&str>` (line 203).

`CommandBuilder` — `.../portable-pty-0.9.0/src/cmdbuilder.rs`:

- line 203: `pub struct CommandBuilder { ... }`
- line 215: `pub fn new<S: AsRef<OsStr>>(program: S) -> Self`
- line 254: `pub fn new_default_prog() -> Self`
- line 272: `pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S)`
- line 280: `pub fn args<I, S>(&mut self, args: I) where I: IntoIterator<Item = S>, S: AsRef<OsStr>`
- line 299: `pub fn env<K, V>(&mut self, key: K, value: V)`
- line 342: `pub fn cwd<D>(&mut self, dir: D)`
- line 244: `pub fn set_controlling_tty(&mut self, controlling_tty: bool)`

Canonical flow (`examples/whoami.rs`, verbatim structure):

```rust
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

let pty_system = NativePtySystem::default();
let pair = pty_system.openpty(PtySize {
    rows: 24, cols: 80, pixel_width: 0, pixel_height: 0,
})?;
let cmd = CommandBuilder::new("whoami");
let mut child = pair.slave.spawn_command(cmd)?;
drop(pair.slave);                       // release slave handles
let mut reader = pair.master.try_clone_reader()?;
let mut writer = pair.master.take_writer()?;   // take_writer once; drop sends EOF
// ... read loop, resize via pair.master.resize(PtySize{..})? ...
let status = child.wait()?;
```

### Correction: `OpenptyResult` is NOT in portable-pty 0.9.0

`grep -rn OpenptyResult .../portable-pty-0.9.0/` returns **nothing**. The task
brief's `OpenptyResult` name belongs to a different crate/version. In 0.9.0 the
result of `PtySystem::openpty` is `anyhow::Result<PtyPair>` and the pair is
`PtyPair { slave, master }` (above). Do not write code against `OpenptyResult`.

---

## 4. Unverified / not present (do not guess)

- `Term::size()` — **not present** in alacritty_terminal 0.26.0; use `Dimensions`
  methods or `term.grid()`.
- `OpenptyResult` — **not present** in portable-pty 0.9.0; use `PtyPair`.
- `EventListener` method names beyond `send_event` — none exist; title/bell/wakeup
  are `Event` variants, not trait methods.
- OSC 8 hyperlink as an event — not exposed as an event; read per-cell
  `Cell::hyperlink()`.
- `vte::ansi::Processor` generic parameter / `Timeout` behaviour under the
  `std` feature: default `StdSyncHandler` was read from source; concrete
  synchronization timing was **not** exercised here (engine behaviour tests are
  phase B).
- Named/indexed colour → RGB resolution for rendering: alacritty exposes
  `Term::colors() -> &Colors` (`src/term/mod.rs:952`) and `Colors([Option<Rgb>; COUNT])`
  (`src/term/color.rs:22`), but the app palette population path was **not**
  traced end-to-end here; treat as `UNVERIFIED` pending phase-B behaviour tests.
