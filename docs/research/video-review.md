# Reference video review — Relayable Phase 0 (Wave 1)

Owner: research worker `ref-video`. Branch: `research/ui-reference`.
Scope: all five user-provided reference videos and this file only. Sibling worker `gorex` owns
`docs/research/gorex-review.md` and `docs/research/evidence/gorex/**`; this file does not touch those.

> **Name rule (`full.md` §0.1).** The reference product in the footage is branded **Superlogical** in
> the UI and is called **REX** (or "Rex") in speech. Those names appear below only as research
> evidence, per §0.1. Relayable is the product being built; nothing here authorizes using "REX" in
> Relayable UI, packages, or docs.

---

## 1. Method, tooling, and access routes

### 1.1 Access route per video

| # | Video ID | Actual title (from `yt-dlp` metadata) | Channel | Published | Duration | Access route achieved |
|---|---|---|---|---|---|---|
| 1 | `XyxShCZrGiw` | Superlogical Pre-Alpha Demo | Mitchell Hashimoto | 2026-09-02 | 2:53 (173 s) | **Metadata + auto-captions + downloaded 480p frames (vision worked)** |
| 2 | `Y6nFMmUPzXM` | Superlogical Terminal Multiplexer High-Level Architecture | Mitchell Hashimoto | 2026-09-02 | 10:40 (640 s) | **Metadata + auto-captions + selected 480p frame sections (vision worked)** |
| 3 | `T5gV6anSt-4` | Superlogical Server Memory Optimizations Overview | Mitchell Hashimoto | 2026-09-02 | 11:43 (703 s) | **Metadata + auto-captions + selected 720p/480p frame sections (vision worked)** |
| 4 | `PdwTjSBW6Y8` | Superlogical Remote Sessions Pre-Alpha Demo | Mitchell Hashimoto | 2026-09-08 | 6:16 (376 s) | **Metadata + auto-captions + downloaded 480p frames (vision worked)** |
| 5 | `9fcfDF8SBnc` | Superlogical Pre-Alpha Demo: CLI | Mitchell Hashimoto | 2026-09-14 | 7:31 (451 s) | **Metadata + auto-captions + downloaded 480p frames (vision worked)** |

All five videos were accessible. No video was blocked.

### 1.2 Exact commands and exit codes (representative)

Metadata and captions, per video (all five succeeded):

```text
yt-dlp --skip-download --write-info-json --no-playlist -o '<n>/%(id)s.%(ext)s' 'https://youtu.be/<id>'
# exit 0 for all five. stderr excerpt (all five):
#   WARNING: The extractor specified to use impersonation for this download, but no
#   impersonate target is available. ...
yt-dlp --skip-download --write-auto-subs --write-subs --sub-langs 'en.*,en' \
       --sub-format vtt --no-playlist -o '<n>/%(id)s.%(ext)s' 'https://youtu.be/<id>'
# exit 0 for all five; en-orig.vtt and en.vtt written per video.
```

Captions are **auto-generated** (`info.json` has `subtitles: []`, only
`automatic_captions`), and `en-orig.vtt`/`en.vtt` are byte-identical. See §7 for ASR caveats.

Frame extraction downloads (only files needed for frames were downloaded):

```text
# Video 1, first attempt with 'bv*[height<=720]+ba' picked AV1 stream 398:
#   ERROR: unable to download video data: HTTP Error 403: Forbidden   (exit 1)
# Retried with 'bv*[height<=480]+ba' -> 1/dl.mp4 (4.8 MB), exit 0.
# Video 3 chart section with same 720p selector: 'ERROR: ffmpeg exited with code 8' (merge failure).
# Retried '--download-sections "*2:35-3:45"' with 'bv*[height<=480]+ba' -> 3/chart.mp4, exit 0.
# Videos 4 and 5 full downloads with 'bv*[height<=480]+ba':
#   4/dl.mp4 (8.1 MB, AV1 722x480), 5/dl.mp4 (11 MB, AV1 786x480), both exit 0.
# Video 2 sections 0:25-1:20, 2:00-3:25, 4:50-5:45 -> three files, total 6.4 MB, all exit 0.
```

Frames (representative):

```text
ffmpeg -i 1/dl.mp4 -vf fps=1/4 -q:v 3 evidence/video1/f%03d.jpg          # exit 0, 43 frames
ffmpeg -ss 00:47 -i 1/dl.mp4 -frames:v 1 -q:v 2 evidence/video1/still-0047.jpg   # exit 0
ffmpeg -i 3/chart.mp4 -vf fps=1/2 -q:v 3 evidence/video3/chart-%03d.jpg  # exit 0, 35 frames
ffmpeg -i 5/dl.mp4 -vf fps=1/5 -q:v 3 evidence/video5/f%03d.jpg          # exit 0, 90 frames
# plus targeted `-ss` stills and 2x/3x lanczos upscales/crops for legibility.
```

Total media downloaded ≈ **34 MB** (cache) plus ≈ 15 MB extracted stills — within the
operator-imposed 1.5 GB capture budget for this run (a per-run capture budget, not a `full.md` or
repository requirement).
No video or image file was committed (see §1.4).

### 1.3 Audio and vision inspection statement

- **Audio:** inspected via auto-generated captions only; no human transcription exists for these
  videos. Raw VTT and deduplicated transcripts were produced (`~/.cache/relayable-research/<n>/transcript.txt`).
- **Frames:** actually extracted and **visually interpreted**. The reviewing tool routed extracted
  JPEGs to a vision-capable model and the UI content was legible, including console text, window
  chrome, and palette rows (see the frame citations below). Where a label was not confidently
  legible, it is marked `unknown` rather than guessed.
- **Not done:** frame-by-frame pixel measurement, colour sampling with a colour picker, font
  identification, or any motion analysis. Geometry statements are relative/qualitative, not measured.

### 1.4 Evidence paths (local, gitignored, not committed)

`git check-ignore -v docs/research/evidence/` → `.gitignore:11:/docs/research/**/evidence/` (exit 0);
a dummy `docs/research/evidence/video1-frame-0001.png` also ignored. Frames were therefore written
under:

```text
docs/research/evidence/video1/   # fps frames + stills + tab-bar crops
docs/research/evidence/video2/   # targeted diagram/connect frames from 3 sections
docs/research/evidence/video3/   # chart/post frames
docs/research/evidence/video4/   # fps frames + stills + 2x upscales
docs/research/evidence/video5/   # fps frames + stills + 2x upscales
```

Raw metadata, transcripts, and download logs: `~/.cache/relayable-research/<n>/`.
These are analysis-only artifacts. They are not shipped assets, not licensed for redistribution,
and must never be committed or bundled (`full.md` §2.3, §2.6).

---

## 2. Video 1 — “Superlogical Pre-Alpha Demo”

- URL: https://youtu.be/XyxShCZrGiw
- Creator: Mitchell Hashimoto · published 2026-09-02 · duration 2:53 · no chapters (metadata)
- Access route: metadata + auto-captions + downloaded 480p video, frames visually inspected.
- Frame evidence: `docs/research/evidence/video1/` (`f001`–`f043` at 4 s spacing, 14 targeted
  stills, 5 tab-bar crops (`topbar-*`), 2 tab crops (`tabs-*`); no upscale files in this directory).

### 2.1 Spoken timeline (caption timestamps)

| Time | Statement / action |
|---|---|
| 0:00–0:12 | Demo of "the real basics"; intentionally under 3 minutes. |
| 0:13–0:20 | App launches from Dock; "showing how fast it loads. Less than half a bounce." (exact spoken phrase is an auto-caption; see §7 — do not treat as a numeric claim). "And we have an interactive application." |
| 0:24–0:40 | "when you come in, it just feels like a normal terminal. And it should… You shouldn't be entering a different mode… it shouldn't be scary… because you're starting to multiplex terminals across different machines and… persistence." |
| 0:42–0:55 | Starts workloads: "Btop, Htop", goes to "the ghosty folder", opens a file in there. |
| 0:52–1:10 | Starts a counter; quits the client "around 50" and restarts: "it's at almost 80. These things keep running… even though the… client isn't running." |
| 1:17–1:30 | Re-opens: "how fast we get back in. Again, still half a bounce even though we have multiple state. We're connecting to a server. We're rendering. We're syncing these streams and rendering." "this is a persistent session that supports remote hosts. Although, this one is local." |
| 1:36–1:52 | "we have sessions… let's go ahead and create another one. We can name it work… And we're in the work session." Returns to the ghosty session; re-runs the counter: "We're still in that session… Everything just keeps working as you'd expect." |
| 1:57–2:17 | Stops the program: "look at the scroll bar. Native scrolling. Just native scrolling… for people coming from tmux… that's going to feel pretty cool… feels like a native terminal… But, you are getting that persistence." |
| 2:20–2:30 | "there are vertical tabs. I'm not converted yet to vertical tabs, so I still prefer horizontal, but they are there. People ask a lot." |
| 2:31–2:53 | Closing: "I'm purposely showing this minimal slice that is just a terminal because that's how it should feel." |

### 2.2 Visual observations (frame-cited)

| Video time | Observed (frame) | Confidence |
|---|---|---|
| 0:13 | Desktop; app launched from macOS Dock; a "Superlogical" app icon is visible in the Dock. | high (frame) |
| 0:24 | App window on screen, terminal shows macOS login banner and date; workspace appears as a single full-window terminal. | high (frame) |
| 0:42–0:47 | **Title/tab bar:** macOS traffic lights, a small window/sidebar control, then a horizontal row of tabs, each with a `>_` terminal glyph and a label; `+` control at the right end. Visible labels: `btop`, `htop`, `~` (frame `tabs-0047`). Active tab has a lighter pill/segment background. Below the tab row a thin **pane header** row shows a small icon and the pane's cwd/program (`~`). | high (frame) |
| 0:47 | Terminal content: `Last login: Fri 28 Aug 14:23:18 on ttys003`, prompt, typed `cd ~/Documents/ghostty`, and a right-aligned `Fri 28 Aug 14:23:19 -07:00` line. | high (frame) |
| 0:55–0:58 | Counter block: a single column of incrementing numbers (…24, 25, 26…) in a plain shell pane; tab strip now `btop`, `htop`, `~/Documents/ghostty`-style labels (`still-0058`). | high (frame) |
| 1:33–1:40 | Four tabs visible: `btop`, `htop`, two `~/Documents/ghostty`-label shells distinguished by a numeric suffix (frame `tabs-0133`, e.g. `~/Documents/ghostty-1`). One tab runs Neovim on a Zig source file (mouse_state/gesture code; status line shows NORMAL mode and a file path). Pane header shows cwd+program (`~/Documents/ghostty-nvim`). | medium-high (label suffix read from a 3x crop; medium for exact directory string) |
| 1:40 | **Session switcher palette** opened over the terminal: rounded dark popup with a search field placeholder `Filter or create...`, first row highlighted with a checkmark: `✓ Demo`; further rows include `New Session`, an `Add Remote Host...` entry (full list partly occluded by the cursor). | high (frame `topbar-0140`) |
| 2:03 | Scrollback view: long column of counter output, prompt `degn@r:~`-style, and a **native scrollbar** visible at the right edge. | medium-high (frame) |
| 2:22 | **Vertical tabs** state and palette: left vertical tab sidebar with `>_`-icon tab rows; palette rows visible `Switch to Vertical Tabs` with shortcut `⇧⌘T` and `Add Remote Host...` with shortcut `⇧⌘R`. | medium-high (frame `still-0222`, palette rows legible; vertical-sidebar rows small) |

### 2.3 UI structure (as observed)

- Outer window: macOS window chrome, traffic lights top-left; a **single top bar** containing window
  control + horizontal tabs + `+`. No menu bar or toolbar was shown.
- Tab = one terminal block; label is the running program name (`btop`, `htop`, `nvim`) or the shell
  cwd (`~/Documents/ghostty`, with `-1` suffix for a duplicate). Every tab carries a `>_` glyph.
- Active tab is a lighter rounded segment; inactive tabs are flat/darker.
- A per-pane header strip (icon + cwd/program) sits between the tab strip and terminal content; in
  Video 4 this same strip reads `shell`, so this is the block/pane header (§5).
- Terminal rendering: monospace, dark navy background, standard ANSI prompt colours, block cursor;
  no custom status bar or footer was visible in this video.
- Command palette: dark rounded popup, search field at top, single highlighted row (checkmark for
  the current session; blue fill in V1, pink fill in V5), shortcut glyphs right-aligned.
- Native scrollback scrollbar on the right edge.

### 2.4 Observed / inferred / unknown

- **Observed:** session creation and naming ("work"); session switching; per-session tab sets;
  persistent processes after client quit/relaunch; native scrollback; horizontal and vertical tabs;
  command palette with session list; `Switch to Vertical Tabs` and `Add Remote Host...` actions.
- **Inferred:** tab label strings are cwd paths/program names (from legible frames); selected-tab
  visual treatment is a lighter pill; `⇧⌘R` opens Add Remote Host and `⇧⌘T` switches to vertical
  tabs (visible beside the palette rows).
- **Unknown:** window/panel minimum sizes; exact colours and fonts (no colour switching — JPEG 480p
  samples only); split/pane controls in Video 1 (none observed); whether the `>` control next to
  the traffic lights toggles a sidebar (V4/V5 evidence suggests it relates to vertical tabs —
  not confirmed).

---

## 3. Video 2 — “Superlogical Terminal Multiplexer High-Level Architecture”

- URL: https://youtu.be/Y6nFMmUPzXM
- Creator: Mitchell Hashimoto · published 2026-09-02 · duration 10:40 · no chapters
- Access route: metadata + auto-captions + selected frames from 3 sections (0:25–1:20, 2:00–3:25,
  4:50–5:45) at 480p; frames visually inspected. It is a talking-head architecture video; the
  useful evidence is spoken, with occasional on-screen diagrams.
- Frame evidence: `docs/research/evidence/video2/` (8 stills).
- **This video contains no product UI and no CLI commands.** It must not be used as UI evidence.

### 3.1 Architecture claims (spoken; timestamps from captions)

| Time | Claim (paraphrase; direct quotes where short) | Confidence |
|---|---|---|
| 0:33–1:08 | Traditional multiplexer stack: terminal emulator → multiplexer → PTY/shell; this introduces "double parsing, double state processing, just duplicated work". | high (spoken, consistent) |
| 1:08–1:42 | "tmux, Zellij, screen… their terminal emulator like IO part is fairly slow" compared to modern terminals (Ghostty, Kitty, Alacritty) "sometimes 100 plus times faster", so a multiplexer in front delivers a slow experience in a fast terminal. | medium (spoken opinion/claim; "100 plus times" not measured on screen) |
| 1:48–2:00 | The project goes "unapologetically all in" on **libghostty**. (Captions render this as "libgo C"/"libgoasty" — see §7; treated as libghostty, consistent with later mentions.) | high for intent; medium for spelling (ASR) |
| 2:00–3:10 | On client connect, the server's PTY-reader **pauses processing**; it sends a custom binary protocol with "just enough terminal state, screen state… dimensions, mouse cursor state" so the client renders immediately; a **ready frame** ends the initial sync. Scrollback history is deferred and streamed in the background. | high |
| 3:12–3:55 | After the ready frame, the server un-pauses and **tees raw PTY bytes to all clients, like SSH**, instead of sending screen diffs (which tmux/zellij do). The client is expected to run a compliant, performant terminal emulator ("we assume you're running libghostty everywhere"). | high |
| 4:04–4:45 | Consequence: the attached client is "basically like an SSH client… connected directly to the PTY stream. There is no internal parsing." Even if the *server* parses slowly, the client still parses at full speed. | high |
| 4:45–5:30 | Input is sent to the authoritative server and **serialized**: "one writer, multiple readers", a "distributed system of synchronized finite state machines". If one client's emulator processes incorrectly, only that client sees wrong data; the server stays correct. | high |
| 5:00–5:35 | Scrollback is streamed in the background "newest to oldest… chunk by chunk"; on a slow link the user can see a loading state while scrolling back. | high |
| 5:35–5:55 | Each client "fully owns the viewport state" — independent scroll/selection per client, contrasted with tmux where one client's scroll moves everyone. | high |
| 5:55–6:15 | Out-of-sync client can "restart this process… reset the terminal frames". | high |
| 6:20–7:00 | Native apps are planned (macOS today; browser, mobile, multiple OSes later) so the multiplexer is not "trapped inside a terminal emulator". Each **split is represented as a native tab/window/split**, each with **its own connection** to the binary protocol and 1:1 to a PTY. | high |
| 7:50–8:50 | Legacy/compatibility mode: a non-native terminal (e.g. Kitty) can attach to a single stream by ID; when the terminal cannot speak the binary protocol, a **libghostty terminal sits in the middle**, which is "the same trade-off as other multiplexers" (explicitly compared to tmux), with the same architecture but a faster core. | high |
| 8:50–9:10 | The binary protocol is "predominantly part of libghostty", described as an **open protocol**; native clients avoid the middleman. | medium-high (spoken) |
| 9:06–10:05 | Claim that tmux-style tools are slow to adopt terminal features (e.g. Kitty graphics protocol workarounds) because terminal features are bound to the server-side terminal implementation (captions say "libvte"; see §7); the team claims faster adoption with libghostty, features toggleable by the embedder. | medium (spoken, partly ASR-ambiguous) |

### 3.2 Observed / inferred / unknown

- **Observed (spoken + on-screen):** the client/server split, pause-and-snapshot handshake, `ready`
  frame, raw PTY tee, one-writer input serialization, per-client viewport, background scrollback
  streaming, native splits each with their own connection, libghostty in the middle for legacy
  terminals, native client roadmap (browser/mobile).
- **Inferred:** the "ready frame" is a frame type within a binary protocol (spoken). The exact
  protocol name, framing, and encoding are not shown and remain unknown. "Open protocol" implies
  documentation may exist but none was shown.
- **Unknown (do not invent):** wire format, message names other than `ready`, compression, sequence
  numbering, flow-control mechanism, authentication, and whether the browser/mobile clients are
  implemented. Video 2 does not describe the remote transport at all.

---

## 4. Video 3 — “Superlogical Server Memory Optimizations Overview”

- URL: https://youtu.be/T5gV6anSt-4
- Creator: Mitchell Hashimoto · published 2026-09-02 · duration 11:43 · no chapters
- Access route: metadata + auto-captions + frames from the on-screen X post (~1:46) and its charts
  (2:35–3:45) at up to 720p/480p; frames visually inspected. The session context page and chart
  text are legible.
- Frame evidence: `docs/research/evidence/video3/` (`post-*`, `chart-*`).

### 4.1 Performance figures — verbatim, with all stated context

Spoken (auto-caption text):

- 3:13–3:26: *"The memory cost per full terminal is an order of magnitude in favor of superlogical.
  We're at the 400 kilobyte, they're almost 10 times more, more than 10 times more at the four uh
  near 5 megabyte um level."*
- 3:27–3:35: *"For empty terminals, they are a little bit less, but but I know how what that is and
  we'll be able to match that."*
- 3:36–3:42: *"the per client cost of us being cheaper per connected client. Again, need to get that
  lower, too"*.
- 4:21–4:35: *"if a terminal is idle for 60 seconds, what we do… is we park it. And what that means
  is we take a binary snapshot of the entire terminal terminal emulator state and put it to disk."*
- 6:02–6:33: *"even for a 64 MB, which is much bigger than 10,000 lines by the way, even for a 64 MB
  full compressed scrollback, um it takes us about um 200 microseconds to de- to unpark that, not
  counting the disk speed… it only takes about 20 20 microseconds."* (The referent of "20
  microseconds" is not established in the captions — likely snapshot/park rather than unpark; treat
  as ambiguous.)
- 8:48–8:58: moving an unobserved PTY into the shared poller: *"you get about a 5 to 10% hit in IO
  throughput, but that's worth it when you're not looking at it."*
- 9:27–9:29: client buffer parking: *"It adds up. It's kilobytes of buffers."*
- 5:59–6:22 and 7:00–7:30: per-PTY dedicated OS thread blocked on `read` (from Ghostty) is claimed
  faster than putting PTY FDs in `kqueue`/`epoll`/`io_uring`, which the speaker says causes "a very
  noticeable… hit to latency, to IO throughput".
- 10:36–11:00: parked terminals can stream their binary snapshot from disk directly to a connecting
  client without unparking.

On-screen charts (frames; these are the numbers the speaker tells viewers to look at):

| Chart (frame) | Series | Value |
|---|---|---|
| **“Server Start, One Session”** — “Total server physical footprint” (`chart-016`, ~3:05) | tmux 3.5a | **2.50 MiB** |
| | Superlogical | **10.6 MiB** |
| | zellij 0.45.0 – plugin-free | **35.3 MiB** |
| | zellij 0.45.0 – default | **53.6 MiB** |
| **“Memory Cost Per Filled 10,000 Line Terminal”** — “Additional server memory footprint per single 80×24 terminal filled with 10,000 numbered 72-column prose lines” (`chart-026`, ~3:20) | Superlogical | **407 KiB** |
| | tmux 3.5a | **4.89 MiB** |
| | zellij 0.45.0 – plugin-free | **22.7 MiB** |
| **“Memory Cost Per Client Connection”** — “Additional server memory footprint per single client connection, with 50 filled terminals” (`chart-031`, ~3:30) | Superlogical | **85 KiB** |
| | tmux 3.5a | **157 KiB** |
| | zellij 0.45.0 – plugin-free | **21.7 MiB** |
| | zellij 0.45.0 – default | **1.59 GiB** |

Every chart carries the footer: *“Zellij note: Default is out of the box. Plugin-free explicitly
removes all plugins.”*, *“LOWER = BETTER.”*, and the environment line
*“macOS 26.2 on an Apple M4 Max, phys, footprint numbers”*. The per-client chart uses a logarithmic
axis (64 KiB → 1 GiB).

Also shown on screen in the quoted post (frame `post-008`, ~1:46; written source accompanying the
talk): *"The Superlogical server is built to run anywhere and everywhere and serve a level of scale
(terminals+clients) unmatched w/ prior solutions."*; scale rationale attributed to AI-driven agent
terminal counts; *"while tmux's empty states are excellent, Superlogical makes it up at scale"*;
Zellij critique (wasm plugins per tab, memory not reclaimed after closing tabs).

### 4.2 Reproducibility assessment (mandatory labelling)

- **Stated environment:** macOS 26.2, Apple M4 Max, "phys, footprint numbers".
- **Stated workloads:** chart 1 = server start with one session; chart 2 = one 80×24 terminal
  filled with 10,000 numbered 72-column prose lines; chart 3 = one client connection with 50 filled
  terminals.
- **Missing context (not stated anywhere in the video or on the charts):** the Superlogical build
  or version, tmux/zellij build flags beyond the version number, Rust/toolchain, measurement command
  or profiler, idle vs active duration, output rate, and scrollback configuration for tmux/zellij.
  The 400 KB / ~5 MB spoken figures are rounded and have no separate measurement setup.
- **Conclusion:** **every published figure here is non-reproducible as stated** for Relayable
  decisions. They are usable only as directional evidence that per-terminal and per-client memory
  are treated as first-class server concerns. Per `full.md` §12, Relayable must run its own
  benchmark harness with recorded OS/CPU/RAM/toolchain/commit/workload/units rather than copying
  these numbers as targets. Nothing may be extrapolated from them (e.g. do not claim "Relayable
  will use 400 KB per terminal").

### 4.3 Observed / inferred / unknown

- **Observed:** parking design (idle ≥60 s → encrypted binary snapshot to disk; unpark on PTY read;
  works while clients are attached), parked PTY moved to a shared evented poller when idle or
  unobserved (5–10% throughput trade-off), client buffer parking, disk-streamed snapshots to new
  clients.
- **Inferred:** "parking" is the project's own term; equivalent industry terms cited by the speaker:
  hydrate/dehydrate.
- **Unknown:** encryption scheme for snapshots ("encryption and other security involved… we'll talk
  about that another time"), compression algorithm, disk format, snapshot size limits, unpark
  latency measurement method, and how the 20 µs/200 µs figures were measured.

---

## 5. Video 4 — “Superlogical Remote Sessions Pre-Alpha Demo”

- URL: https://youtu.be/PdwTjSBW6Y8
- Creator: Mitchell Hashimoto · published 2026-09-08 · duration 6:16
- YouTube chapters (metadata): 0:00 Introduction to Superlogical · 1:36 Session and user management
  · 3:00 Performance and responsiveness · 4:49 Advanced navigation features
- Access route: metadata + auto-captions + downloaded 480p video, frames visually inspected.
- Frame evidence: `docs/research/evidence/video4/` (`f001`–`f075` at 5 s spacing, 11 stills, 6 2x upscales).

### 5.1 Connection lifecycle — as shown (frame-cited)

| Step | Evidence |
|---|---|
| 1. Start local | Local session, persistent ("even though it's local… I could reopen the application and that's saved"). |
| 2. Add remote host | Command palette → `Add Remote Host...`. Palette frame at 0:30 shows the full **command list**: `Change Session` (current value `cosmic-summit`), `Rename Session`, `Rename Tab` (value `~/Documents/ghostty`), `Add Remote Host...`, `New Session`, `New Tab`, `Go to Director...` (value `~/Documents/ghostty`), `New Window`, `Close Tab`, `Refresh Sessions`, `Close Session`, `Close Window`, then a **`Pane`** section: `Split Pane Left`, `Split Pane Right`, `Split Pane Up`. Search placeholder `Search commands...`; demo key overlay shows `⇧⌘P`. |
| 3. Host reachability | Host is a remote VPS "connected to my Tailnet". Window subtitle shows the host endpoint **`hill.hamster-dinosaur.ts.net`** (Tailscale MagicDNS name); tab label reads **`mitchellh@hill: ~`**. |
| 4. Connect | After connecting the remote session appears in the session list and is auto-named **`drifting-cedar`** ("every new session gets a unique name"); he renames it to `demo`. Second generated name in the transcript sounds like "Chris Sierra" (ASR of a two-word name). |
| 5. Split over one connection | "I could open a split and I have two now, and this is over the same connection." Split frame at 0:58: two side-by-side panes, each with a `shell` pane header; left pane runs `loginctl list-sessions`. |
| 6. Real system login | `loginctl list-sessions` output (left pane, frame `up-0058`): a `user` class session with leader `rex` (the REX server's login session) exists in logind. Spoken: "a full SSH-style system login… all of your login shell stuff is loaded, all of your per-user limits are honored." |
| 7. Identity / auth | CLI on the local machine (frame `up-0411`, session pill **`cosmic-summit`** with subtitle **`This Mac`**): `rex -S hill.hamster-dinosaur.ts.net whoami` → `you: mitchellh@github over tailscale via tailnet-serve/login-server from mitchellh-mbp, acting as mitchellh`; `rex -S root@hill.hamster-dinosaur.ts.net whoami` → same prefix, `acting as root`; `rex -S hill.hamster-dinosaur.ts.net session new` → `session:01a082b8-f7a2-78cd-bc19-fb524efd8bad`. |
| 8. Quit / reconnect | Key overlay `⌘Q` at 3:59 with the app gone (desktop visible). Spoken: "if I quit, quit my local app, it'll be a little slower to load this time, but we reconnect and we go back into the last session we were in… all our sessions are still here." Frame `up-0411` (after reconnect) still shows the earlier `whoami`/`session new` scrollback in the terminal, i.e. server-side session state was restored on reattach. |
| 9. Session kill | Spoken only ("grab this ID… do a session kill… if I hit go and reopen, it's gone already… propagates to all the clients really quickly"). No clear frame of the exact interaction was captured; the underlying CLI verb is `kill` ("Kill sessions", Video 5 help). |
| 10. Go to directory | New shortcut shown: **`command shift G`** → "a go to directory feature… opens a new terminal in that folder". Works locally and remotely: demo navigates `/proc`, `/home`, back to a user directory on the remote host (frame `up-0449` two-pane remote: `uname -a` → `Linux hill 6.18.48 #1-NixOS SMP PREEMPT_DYNAMIC Fri Aug 28 06:22:54 UTC 2026 x86_64 GNU/Linux`; other pane `cd /`). |

### 5.2 Transport, auth, and VPN dependency (evidence vs inference)

- **Stated:** "this is a real login… Our server does a full SSH style system login"; "Rex is a full
  SSH replacement… I actually don't have SSH on this machine at all. This is how I get in"; there is
  "a mapping on the server side of who can act as who and how they authenticate and… we respect
  pre-existing SSH keys and everything"; he declines to detail "the full protocol and transport";
  "we bring in a lot of the architectural similarities as Mosh into this. And… we don't do
  everything Mosh does."
- **Observed:** the demo host is reached by its **Tailscale** name (`*.ts.net`), `whoami` reports
  `over tailscale via tailnet-serve/login-server`; the demo machine has no SSH server (spoken).
- **Inferred:** the demo's connectivity depends on a Tailscale tailnet; the application protocol
  itself is a native login/RPC channel, not SSH tunneling (root and mapped-identity `whoami` output
  indicates server-side account mapping rather than a raw SSH shell). **This is inference from the
  demo, not a protocol statement by the speaker.**
- **Unknown:** whether a Tailscale-style overlay network is *required* or merely used in the demo;
  wire protocol, encryption, pairing/credential storage, port/bind defaults; failure/error handling
  (unreachable host, expired credential, protocol mismatch) — none demonstrated; session-kill UI
  geometry; whether reconnection replays full scrollback or only the visible screen (frame suggests
  at least visible scrollback is restored).

---

## 6. Video 5 — “Superlogical Pre-Alpha Demo: CLI”

- URL: https://youtu.be/9fcfDF8SBnc
- Creator: Mitchell Hashimoto · published 2026-09-14 · duration 7:31
- YouTube chapters (metadata): 0:00 Introduction to CLI · 0:47 Managing sessions and blocks ·
  1:44 Scripting and synchronization · 2:26 Inspecting session state · 3:48 Working with the API and
  events · 5:03 Advanced block manipulation · 6:34 Conclusion and automation
- Access route: metadata + auto-captions + downloaded 480p video, frames visually inspected.
- Frame evidence: `docs/research/evidence/video5/`.
- **Important alignment caveat:** in this video the on-screen terminal clock advances far more slowly
  than the caption timeline in the middle section (e.g. `up-0425` at caption 4:25 shows on-screen
  clock 15:02:01; `up-0535` at caption 5:35 shows 15:03:37; the `up-0600` JSON output at caption
  ≈6:00 shows 15:03:48). Caption timing and frame content are therefore only loosely
  aligned; command evidence below cites the frame when a command is visible and the caption when a
  statement is heard. Do not treat either timestamp as exact for this video.

### 6.1 Commands and outputs captured verbatim (legible frames)

Session/block creation and layout (frame ~1:04–2:28 video; on-screen 14:59–15:00):

```text
$ rex new Video -- $(which htop)
session:01a0a1ee-3d84-779d-8fe4-bbdb65f194fc
$ rex split -s Video --label nvim -- $(which nvim)
block:01a0a1ee-7ccc-7822-8b75-a25f3cb05292
$ rex move -s Video nvim below shell
$ rex split -s Video --keep-open --label sleep -- $(which sleep) 10
block:01a0a1ef-3928-74ed-b166-776d332fcdee
$ rex wait -s Video sleep
$ rex session inspect Video
```

`$(which …)` note: the `$` is rendered with an unusual glyph at this resolution; the command
substitution is **inferred** from the `(which htop)` shape and the shell context — marked
medium confidence. The `rex new` form is `rex new <label> -- <command>`, and `rex split` uses
`-s <session> --label <label> -- <command> [args]`.

`rex session inspect Video` output (frame `up-0348`, on-screen 15:01:39):

```text
session: session:01a0a1ee-3d84-779d-8fe4-bbdb65f194fc
label: Video
active-window: window:01a0a1ee-3d85-7904-abfa-403677cb416e
windows: 2
clients:
  uid 501 over unix 2m ago  client:01a0a1b0-e93c-7177-85f8-0525157b261d
  uid 501 over unix 0s ago  client:01a0a1f0-7a7f-70de-a548-03f0d1c354ca
WINDOW                                       LABEL              ACTIVE FOCUSED BLOCK                                        BLOCKS
window:01a0a1ee-3d85-7904-abfa-403677cb416e  Primary            true   block:01a0a1ef-3928-74ed-b166-776d332fcdee   3
window:01a0a1ef-db63-77ac-912b-b37ee64939f0  Window e64939f0     false  block:01a0a1ef-db61-772b-8c8d-8cc76919144e   1
```

`rex block list -s Video` output (same frame):

```text
WINDOW ID  FLAVOR                            LABEL  PLACEMENT
1     42020f  com.superlogical.terminal.shell  shell  placed
1     b05292  com.superlogical.terminal.shell  nvim   placed
1     2fcdee  com.superlogical.terminal.shell  sleep  placed
2     19144e  com.superlogical.terminal.shell  shell  placed
```

Method call and JSON output (`rex block inspect …` typed with blank output in `up-0425`,
caption ~4:25, clock 15:02:01; `rex block call …` being typed with no output in `up-0535`,
caption ~5:35, clock 15:03:37; JSON output plus `rex block close -s Video sleep` in `up-0600`,
caption ≈6:00, clock 15:03:48):

```text
$ rex block call -s Video -b sleep process
{
  "last_exit": {
    "exit_code": 0
  },
  "home": "/Users/mitchellh"
}
$ rex block close -s Video sleep
```

(For a running block the speaker says the same method returns child + foreground process
information; no frame captured of that variant.)

`rex` with no arguments — full help text (frame ~7:04, fully legible):

```text
The multiplexer for all work.

Usage:
  rex [command] [flags]

Common commands:
  new      Create a session
  attach   Attach to a session
  ls, list List sessions
  run      Run a command in a new block
  split    Split the focused block
  send     Send input to a terminal
  send-key Send key presses to a terminal
  capture  Print a terminal's screen
  kill     Kill sessions

Layout:
  focus    Focus a block or window
  zoom     Zoom a block to fill its window
  move     Move a block next to another
  swap     Swap two blocks
  resize   Resize a block

Resources:
  session  Manage sessions
  window   Manage windows
  block    Manage blocks
  client   Manage attached clients

Automation:
  events   Stream server events
  wait     Wait for a block, window, or session to end
  api      Inspect and call the server API

Server:
  server       Manage the Rex server
  login-server Run the multi-account login server
  whoami       Show your identity and the account running the server
```

Remote targeting syntax visible in Video 4 (see §5.1): `rex -S <host>` and
`rex -S root@<host>` (server target `-S`), with `session new`, `whoami` as subcommands.

### 6.2 Spoken behavior not fully captured in frames

- Transcript ~1:44–2:26: for a scripted `sleep`, `wait` blocks until completion; with `--keep-open`
  the block remains after exit, otherwise "normally it would disappear". `move` can move a block
  relative to another label (e.g. below `shell`) while a script runs, without interruption.
- Transcript ~2:26–3:48: `session inspect` shows "its ID, its label, the currently focused window…
  the number of windows… the connected clients"; he explains two clients are attached (the CLI and
  the Mac app). `block list` shows 4 blocks, 3 in window 1 (`Primary`), all `terminal` type.
- Transcript ~3:48–5:00: `block inspect` shows "block ID, session ID, label… metadata… methods…
  events"; "every block within the Rex Multiplexer exposes its own custom API", terminal blocks
  "emit events"; an **event stream** (`rex events`, per help) is started and events observed:
  `client connected`, `block size changed`, `block closed`, a view/layout change triggered by the
  close, a session change after a new split, and a custom `child exited` event from the terminal
  block. Exact event payloads are not legible in the captured frames → **unknown**.
- Transcript ~5:03–6:34: `block call` can invoke a method on a block ("one of the methods name was
  process"); closing via `rex block close` (shown) vs UI; `rex kill video`-style session kill
  ("I could just go ahead and kill video").
- Transcript ~6:34–7:31: help demonstrates more capability: "send input, simulate key releases,
  simulate mouse movement… capture the terminal screen as text or HTML or other formats". Flags for
  those operations were not legible → **unknown**.

### 6.3 CLI observations

- All commands in this demo are run **inside a client terminal** (window shows a session/terminal
  UI; speaker says at 0:56 "Since I'm in a Rex client"), and changes propagate live to the GUI —
  "any changes you make through the CLI are immediately visible on all connected clients".
- Output style: short confirmation lines (`session:<uuid>`, `block:<uuid>`), aligned table output
  for `inspect`/`list`, JSON for typed method calls (`block call`).
- IDs are type-prefixed (`session:`, `window:`, `block:`, `client:`) with UUID-style payloads;
  window/block tables show shortened hex IDs as well; auto-generated window label is
  `Window e64939f0`.
- CLI examples reference sessions/windows/blocks by label (`Video`, `nvim`, `sleep`, `shell`,
  `Primary`); ambiguity/failure behavior was not exercised.
- Not observed in this video: `--json` output for regular commands (only method-call JSON), exit
  codes, environment variables (`REX_SESSION`/`REX_BLOCK`/`REX_SERVER` are documented officially but
  were not shown), Lua calls, `api list/describe/call` output. These remain **unknown from video**.

---

## 7. Caption/ASR caveats (apply to all spoken quotes)

Auto-captions are the only transcript available, and they demonstrably mangle proper nouns:
"Superlogic"/"Superlogical"/"Super Logical", "Rex"/"Rexas", "libghostty" appears as "libgo C",
"libgoasty", "libgo see" and possibly "libvte"; "Ghostty" appears as "Ghost T"/"goasty"; "btop" may
appear as "Btop". Direct quotes above preserve the caption text where a quote is given, with the
normalized name added in brackets. Architecture and CLI claims were cross-checked against legible
frames (help text, commands, chart titles) wherever possible; anything that rests on captions alone
is marked accordingly.

---

## 8. Cross-cutting findings and open items

1. **Branding:** the footage uses the Superlogical wordmark/app name and speech says "Rex". Relayable
   must use its own branding while reproducing observable layout and behavior (`full.md` §0.2).
2. **Highest-value UI evidence** is Video 1 (main workspace) and Videos 4–5 (sessions, remote host,
   command palette, pane headers, CLI). Video 2 is architecture only; Video 3 is performance only.
3. **Palette vocabulary observed once each (candidate command names):** `Filter or create...`
   (session switcher), `Search commands...` (command palette), `Change Session`, `Rename Session`,
   `Rename Tab`, `Add Remote Host...`, `New Session`, `New Tab`, `Go to Directory...`, `New Window`,
   `Close Tab`, `Refresh Sessions`, `Close Session`, `Close Window`, `Switch to Vertical Tabs`,
   `Split Pane Left/Right/Up`. Shortcuts seen on screen: `⇧⌘T`, `⇧⌘R`, `⇧⌘P`, `⌘K`, `⌘1`, `⌘2`,
   `◇⌘←/→/↑`, `⇧⌘N`, `⌘T`, `⌘N`, `⌘W`, `⌘R`. **Several of these were read from one frame at 480p —
   verify against a higher-resolution capture before implementing.**
4. **Unresolved questions** (owner: lead/UI catalogue; not answered by this footage):
   - exact colours, fonts, and spacing metrics (no measurement was possible at these resolutions);
   - split keyboard shortcut semantics on Linux (all demos are macOS `⌘` bindings);
   - whether the keycast overlay shown in the demos is product UI or a recording aid (assumed aid);
   - event payload schema, `api` command output, `--json` flag, exit codes, environment variables;
   - remote failure/reconnect-error states; whether Tailscale is required;
   - memory benchmark methodology needed to reproduce Video 3 claims (see §4.2).
5. **Per `full.md` §2.7 exit gate:** all five sections are attempted with honest labels; Videos 1
   and 4 (main workspace and remote workflow) were visually inspected; remaining video-sourced
   details are marked inferred/unknown rather than invented.

## 9. Evidence inventory

| Video | Files | What |
|---|---|---|
| 1 | `video1/` — 43 fps frames, 14 stills, 5 tab-bar crops, 2 tab crops | workspace, tabs, palette, scroll, vertical tabs |
| 2 | `video2/` — 8 section stills | talking-head/diagram sections only |
| 3 | `video3/` — 35 chart frames, 20 post-section frames | X post text and all three memory charts (legible) |
| 4 | `video4/` — 75 fps frames, 11 stills, 6 upscales | command palette, remote login, split, reconnect, go-to-directory |
| 5 | `video5/` — 90 fps frames, 14 stills, 9 upscales | CLI commands, outputs, help, palette |

All under `docs/research/evidence/` (gitignored). Raw downloads/logs/transcripts under
`~/.cache/relayable-research/`.
