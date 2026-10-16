# Relayable — Full Product Specification and Persistent Codex Agent Loop

> **Product:** Relayable  
> **Primary platform:** Linux desktop + Linux session server  
> **Desktop UI:** Rust + GPUI  
> **CLI/server/protocol:** Rust  
> **Future mobile client:** Android, Kotlin + Jetpack Compose  
> **Android build workflow:** Android CLI, Gradle wrapper, SDK command-line tools, ADB/emulator; Android Studio is not a required tool  
> **Status:** Living specification. Evidence and implementation status must be maintained in-repository.

This file is both the product specification and the operating prompt for a long-running Codex engineering effort. Read it completely before making architectural decisions. Work in small, verifiable increments. Do not merely produce a plan: implement, test, record evidence, and continue through the task loop until a real blocker or explicit stop condition is reached.

---

# 0. Operating prompt for the Codex lead agent

You are the engineering lead for **Relayable**, a persistent session platform for Linux machines. You are responsible for coordinating research, architecture, implementation, UI fidelity, testing, Linux packaging, and the future Android client.

## 0.1 Product-name rules

- The product name is **Relayable**.
- Relayable is an **independent project**; do not attribute it to Superlogical. Superlogical is the maker of the reference product (REX) only, and that name appears in this repository only as research/compatibility evidence, never on a Relayable surface.
- Do not label the product “REX”, “REX Superlogical”, or “Superlogical Terminal” in the new UI, package metadata, application title, README, docs, CLI help, or marketing copy except when describing the reference product during research.
- The existing reference product in the videos and docs is called REX. Keep that name only in source-evidence notes and compatibility discussions.
- Preferred public CLI: `relayable`.
- Proposed Rust crate prefix: `relayable-`.
- Proposed systemd unit names: `relayable-server.service`.
- Proposed app title: `Relayable`.
- An Android application ID (for example `com.relayable.app`) is only a provisional identifier. Check repository convention and identifier conflicts before locking it.

## 0.2 The user’s explicit UI requirement

**The user wants the UI in the five reference videos, not a newly invented redesign. Treat fidelity to the visible reference UI and workflows as a primary acceptance criterion.** Recreate the observable screen structure, proportions, hierarchy, panel placement, split behavior, tab/window/session navigation, control locations, typography scale, spacing, colors, borders, focus/selection states, command palette, and interaction patterns as closely as evidence permits. Do not replace the interface with a generic terminal app, generic dashboard, or arbitrary “modern” redesign.

The product branding is Relayable, but the goal is to reproduce the reference product’s observable interface and behavior. Use new Relayable wordmarks/icons and original assets where the reference uses a proprietary logo or asset. Do not copy source code, extract private assets, or use copyrighted screenshots as shipped UI textures. Publicly accessible reference footage may be used as visual evidence for implementation and local visual comparison; don't redistribute reference footage/assets in releases.

**Do not claim a faithful recreation before inspecting the videos.** The first research task is to retrieve and inspect the actual footage/captions/available keyframes and build a timestamped visual reference catalogue. If you cannot access a video or its relevant visual moments, say so in the research log, continue with verified official documentation, and clearly mark unverified details. Do not invent a UI and then call it a match.

## 0.3 Rules for working in a real repository

1. Inspect before editing. Preserve all pre-existing user changes, local files, branches, and architecture that makes sense.
2. Read existing `AGENTS.md`, nested agent instructions, README, CI configuration, lockfiles, and code before choosing a plan.
3. Start with a cleanly documented inventory of what exists, what builds, and which dependencies/toolchains are present.
4. Keep the code compiling at every meaningful checkpoint. Prefer vertical slices to placeholder scaffolding.
5. Use actual current tool help and versioned documentation. Never invent CLI flags, GPUI methods, Android CLI commands, library APIs, or protocol behavior.
6. Every non-trivial feature requires tests or a written, specific reason why it cannot yet be tested.
7. Never fake tests, screenshots, video review, measurements, benchmark results, or successful device/emulator execution.
8. Keep unrelated formatting changes out of commits. Do not discard or overwrite user work.
9. Use safe defaults: least privilege, authenticated/encrypted remote transport, restrictive local socket permissions, bounded resource usage, no hardcoded secrets.
10. Do not install global packages, change system configuration, elevate privileges, open firewall ports, deploy remotely, publish releases, or incur paid cloud/device costs without explicit user approval.
11. Do not automatically `sudo`, modify global shell startup files, or replace the user’s selected toolchain. If a required dependency is missing, explain exactly what is missing and suggest a compatible `mise` tool only when appropriate; don't force-install it.
12. Do not assume the Codex runtime has browser access, a particular subagent API, a specific Android emulator image, or internet access. Probe actual capabilities first.
13. The coordinator must use subagents when the active Codex environment supports them. If not, execute the same role plan sequentially and say that agents were not actually spawned.
14. Independent agents must not write overlapping files. Delegate by bounded task and file ownership; the coordinator integrates all changes and owns cross-cutting architecture.
15. Work in bounded loops with durable checkpoints. Continue while there are actionable, authorized tasks. Stop only for an explicit approval gate, unavailable required access, a true external blocker, or an exhausted configured work budget.
16. Do not send user secrets, tokens, SSH keys, credentials, or private machine data to a subagent or remote service unless necessary and explicitly authorized. Redact logs.

## 0.4 First-run environment discovery

Before code edits, run and record the results of the equivalent commands available on the host:

```bash
pwd
git status --short --branch
git log -5 --oneline
find .. \( -name AGENTS.md -o -name AGENTS.override.md \) -print
find . -maxdepth 2 -type f | sort | head -300
rustc --version
cargo --version
rustup show active-toolchain
command -v mise || true
command -v android || true
command -v adb || true
command -v java || true
java -version 2>&1 || true
command -v gradle || true
command -v ffmpeg || true
command -v yt-dlp || true
command -v chromium || command -v google-chrome || command -v firefox || true
uname -a
```

These are discovery checks, not a request to change the system. The user says Rust, Android CLI, and possibly JDK are already installed. **Check before installing.** If Java/JDK or another required tool is absent, print what is missing and propose a `mise` configuration/pin that fits the project rather than trying to modify the machine silently. Respect the repo’s existing toolchain pins.

If the repository is empty, establish a minimal workspace, `AGENTS.md`, CI, docs/decision log, and smoke tests. If the repo already contains code, map it to this plan rather than blindly replacing it.

---

# 1. Product definition

## 1.1 What Relayable is

Relayable is a persistent session platform for working on Linux machines from multiple clients. It includes:

- A server that owns sessions and PTYs independently from any viewer.
- A versioned native client/server protocol, designed for remote use rather than treating SSH as the fundamental session protocol.
- A command-line interface for all core operations and automation.
- A Linux desktop client built with Rust and GPUI.
- A planned Android client built with Kotlin/Jetpack Compose and developed/tested without requiring Android Studio.
- A typed server API, event stream, Lua automation compatibility layer, structured program statuses, and Linux-native packaging.

Do not reduce Relayable to a terminal emulator, a GUI wrapper around SSH, or a single desktop process. Its essential concept is that the session belongs to the server and clients can attach/detach independently.

## 1.2 Core product promise

A user starts work on a local or remote Linux host, disconnects a client, and later reconnects from the same or another device without that disconnect implicitly terminating the process. The desktop and Android clients share a common domain model and server behavior.

## 1.3 Product principles

1. **Session lifetime is independent of client lifetime.** Closing the app or losing network must not kill a server-owned PTY.
2. **Local and remote are the same logical product.** A local server is accessed via a Unix domain socket; a remote server uses an authenticated encrypted connection. UI code should not implement separate session models.
3. **UI fidelity matters.** The five reference videos are the primary visual/interaction benchmark. Don't invent a divergent design.
4. **CLI is first-class.** Every core session operation must be accessible without the GUI.
5. **Protocol-first architecture.** UI frameworks must not leak into the protocol/model crates.
6. **Android readiness from day one.** Model reconnects, slow links, client lifecycle, device authorization, and event resynchronization before implementing Android.
7. **Linux-native operations.** Use systemd, XDG paths, journald, native packages, and least privilege.
8. **Evidence-based performance.** Measure before setting hard memory targets or claiming an optimization works.
9. **Observable and automatable.** Stable IDs, structured errors/events, diagnostics, API discovery, Lua automation.
10. **No fake persistence.** SQLite metadata alone cannot restore a live PTY after a server crash. State guarantees must reflect implemented supervision and tested recovery.
11. **Forward-compatible APIs.** Older clients ignore unknown event fields/types where safe; incompatible protocol versions fail clearly.
12. **Security is a product feature.** No unauthenticated public listeners, no default root shells, no custom cryptography.

## 1.4 First-release scope

Deliver a real end-to-end Linux vertical slice:

- Rust server process owning PTYs and session lifecycle.
- Local Unix socket secured by OS permissions.
- Rust CLI to create, list, inspect, attach to, send input to, and close sessions/windows/blocks.
- GPUI client that shows the reference UI structure and can render/interact with a terminal.
- Persistent sessions after closing the desktop client and disconnecting the client from a remote server.
- Explicit protocol version negotiation, structured errors, snapshots/events, flow control, and reconnect behavior.
- `systemd --user` local installation mode.
- Remote authenticated/encrypted connection design and working implementation before exposing a network listener.
- Linux integration tests and package groundwork.
- Program-status parsing if validated against the reference documentation.

## 1.5 Defer until the foundation works

- Hosted account service/cloud backend.
- Mandatory cloud relay or mandatory mesh VPN.
- Hosted multi-tenant infrastructure.
- Windows/macOS client.
- Full Android implementation before protocol/test foundations stabilize.
- Marketplace/plugin ecosystem.
- General distributed job scheduler.
- Collaborative terminal editing.
- Claims of preserving all running processes across server crash/reboot until tested.

Design extension points without allowing deferred scope to block a real Linux product.

---

# 2. Video research and UI reverse-engineering — mandatory early gate

Five user-provided videos are reference material:

1. **Pre-alpha product demo** — https://youtu.be/XyxShCZrGiw?si=YbQqAJ7xH62eGnHe
2. **Terminal multiplexer architecture** — https://youtu.be/Y6nFMmUPzXM?si=BgjaBfs8jC43QHo5
3. **Server memory optimization** — https://youtu.be/T5gV6anSt-4?si=60tv3fmAbn1iu3OQ
4. **Remote sessions pre-alpha demo** — https://youtu.be/PdwTjSBW6Y8?si=m9GAagbhNl0IFpQS
5. **Pre-alpha CLI demo** — https://youtu.be/9fcfDF8SBnc?si=SEC1FMxrXnEGK8eR

These titles/topics are working leads, not permission to fabricate precise details. The actual videos must be inspected. Build `docs/research/video-review.md`, `docs/research/ui-catalog.md`, and `docs/research/open-questions.md`.

## 2.1 Permitted research workflow

Try, in order:

1. Open the public video pages in an available browser/research tool.
2. Read the public captions/transcript if available.
3. Inspect actual video frames and timestamped UI states using the capabilities available in the environment.
4. If local frame extraction is possible from a permitted media source, capture representative stills for local development comparison. Do not bypass access controls or rely on unauthorised/private media sources.
5. If no actual video frames can be inspected, record that limitation explicitly and use available official docs only for documented behavior. Do not assert pixel fidelity in parts that remain unknown.

Do not spend an unlimited amount of time trying inaccessible content. Time-box each access strategy, record the failure and continue with the remaining evidence. Mark all assumptions with a confidence level.

## 2.2 Video research record template

For every video, record:

- URL, visible title, creator, date/duration if available.
- Access route: actual viewing, captions/transcript only, metadata only, or inaccessible.
- Whether the audio and actual frames were inspected.
- Timestamped timeline of each meaningful workflow.
- Screen layouts, key component bounds, panel locations, session/window/block hierarchy, colors, text scale, spacing, borders, icon style, state indicators, and focus behavior.
- Actions performed and observed transitions.
- Keyboard shortcuts and CLI commands verbatim only when shown/spoken clearly.
- Terminal rendering and behavior.
- Local/remote distinction and connection lifecycle.
- Memory/CPU numbers with the workload and test environment stated by the speaker.
- What is observed, what is inferred, and what is unknown.
- Confidence level for every significant claim.

Never make up benchmark values, command flags, shortcut keys, layouts, architecture boundaries, or network behavior.

## 2.3 Visual reference catalogue

`docs/research/ui-catalog.md` must include, for each identified screen/state:

- Stable name, e.g. `desktop/workspace-default`, `desktop/session-picker`, `desktop/command-palette`, `desktop/remote-host`, `desktop/disconnected`, `desktop/split-layout`, `desktop/settings`, `desktop/inspector`.
- Source video plus timestamp.
- A screenshot reference path if the frame can be lawfully/technically captured; keep raw reference captures local or in a gitignored directory unless there is a clear right to redistribute them.
- A written description of component hierarchy and geometry.
- Interaction notes and state transitions.
- Exact labels/text when readable.
- Expected empty/loading/error/disconnected/focused/selected states.
- Confidence and unresolved details.
- Corresponding Relayable implementation path and screenshot-test name.

The catalogue must represent actual observed screens, not a list of screens we merely wish the product to have. Cross-reference the independent GoRex review where it clarifies a component or interaction.

## 2.4 UI fidelity requirement

Implement the visible reference design with high fidelity:

- Match the outer window layout, major panel widths, central terminal geometry, header and footer structure, tab/session controls, panel separators, hierarchy, and information density.
- Match typography scale/weight, line spacing, padding, border treatment, color relationships, icons' visual weight, terminal font and cursor styling as closely as evidence permits.
- Match hover, selected, active, unfocused, connected, reconnecting, running, completed, blocked, and error appearances wherever those states are observed.
- Reproduce the observed navigation and actions, not just a static screenshot.
- Ensure keyboard-driven actions and terminal input behave as shown in references when shortcuts are verified.
- Prefer a small set of reference-derived design tokens to scattered arbitrary styles.
- Build screenshot tests or visual diff capture for stable screens. Document unavoidable rendering differences across Linux scale factors/fonts.
- Run screenshots on the actual Linux app when possible and compare to reference stills side-by-side. Record discrepancies as issues and fix high-impact differences first.
- Do not claim pixel-perfect matching when exact reference dimensions/fonts/assets are unavailable. State the supported fidelity level and what is unverified.
- Do not use a generic dashboard or over-designed card UI unless the footage shows that structure.
- Brand it as Relayable rather than copying a reference company's logo/wordmark. A new product name does not justify changing the established screen geometry or workflows.

## 2.5 Per-video analysis

### Video 1: pre-alpha product demo

Identify the primary workspace, session chooser, windows/tabs, block/pane split, command palette, host selection, settings, status badges/markers, keyboard navigation, focus handling, and visual transitions. Note all states that are actually shown.

### Video 2: terminal multiplexer architecture

Find evidence about server/client/PTY ownership, session lifetime, reconnection, terminal state, layout storage, multi-client interaction, and transport. Separate the native REX wire protocol from SSH usage and from terminal-level sequences such as OSC. Do not infer a wire format from a diagram or from the UI alone.

### Video 3: server memory optimization

Record all figures and units verbatim. Include machine, OS, build/version, session count, idle/active state, scrollback, output rate, and the measurement method if given. If the context is incomplete, label the metric as non-reproducible and create Relayable benchmarks rather than copying it as a target.

### Video 4: remote sessions

Record connection setup, host discovery, authentication/pairing, the transport shown or stated, connection state, reconnect workflow, failure cases, and any VPN/tunnel dependencies. Distinguish SSH as bootstrap/tunnel from a native application protocol only when evidence supports that distinction.

### Video 5: CLI demo

Capture commands, arguments, output, target resolution, contextual defaults, JSON output, exit codes, environment variables, Lua calls, events, waiting/capture, and API inspection. Reconcile against official REX documentation and document differences between reference behavior and Relayable design.

## 2.6 Supplemental reference implementation: GoRex

Inspect the public repository https://github.com/egoist/gorex as an additional, unusually useful UI/behavior reference. Its README explicitly describes it as a clone of Superlogical's REX terminal. As of the initial research pass, its README describes tabs and split panes, pane headers with program/icon/directory, resizing and zoom controls, persistent sessions, program activity indicators, notifications, host information, a command palette, appearance settings, text sizing, and keyboard shortcuts.

**This repository is a supplemental reference, not the product's source of truth.** Use:

- The five original videos to ground what was actually shown in the original product.
- Official REX documentation to ground documented CLI/API/protocol-visible behavior.
- GoRex to inspect concrete UI composition, interaction details, layout behavior, shortcut conventions, persistence UX, and potential edge cases not easy to see in brief video footage.
- Relayable's own tests and architecture decisions to determine what Relayable actually supports.

Create `docs/research/gorex-review.md` containing:

- Repository URL and commit hash reviewed.
- Screenshots or screenshot descriptions for each useful UI state, with a local-only evidence folder if needed.
- Component hierarchy, relative measurements, typography/color observations, window/tab/pane behaviors, menus/popovers, keyboard shortcuts, focus, resizing, empty/error states, and accessibility observations.
- What behavior is confirmed by running the application versus merely described in the README or inferred by reading code.
- A feature comparison table: original videos/docs, GoRex, Relayable implementation, discrepancies, and confidence.
- Explicit limitations: GoRex README states that remote server connections are not replicated and that its server listens on a local socket. Do not use GoRex as evidence for the remote protocol or remote-host workflows.

### Intellectual property and clean-room rules

At the initial research pass, a root `LICENSE` file was not found at the expected GitHub path. Re-check the repository's current license and any license notices before reusing any code or assets. Public availability is not permission to copy source code, icons, images, branding, or other assets.

Unless a verified license expressly permits reuse, do **not** copy, vendor, translate, or mechanically port GoRex source code or bundle its non-system assets. It is okay to study the running application's user-visible behavior and create an independent implementation from a written behavior/layout specification. Keep the observation and implementation work separated: the research agent writes the behavior catalogue; the GPUI agent implements against that catalogue and screenshots rather than copying source files. Use Relayable branding and independently created assets. Reuse third-party libraries only under their own verified licenses and notices.

The requested UI fidelity remains high: recreate the same visible information architecture, relative geometry, density, visual hierarchy, pane/tab interactions, command palette behavior, keyboard conventions, and state transitions to the extent supported by evidence. Do not redesign it into a generic dashboard. If original video frames and the GoRex implementation differ, record the discrepancy and prefer direct original-product evidence for the intended original behavior.

## 2.7 Exit gate

Do not finish the first UI milestone until:

- All five video review sections have been attempted and honestly labeled.
- At least the main visible workspace in Video 1 and the remote workflow in Video 4 have been visually inspected, or a concrete access blocker is documented.
- The UI catalogue and `docs/research/gorex-review.md` have implementable screen/state specifications, with original-product evidence distinguished from clone-derived observation.
- The top 10 highest-impact visual/interaction discrepancies are tracked.

If the videos remain inaccessible, implement only what is supported, avoid claims of full fidelity, and leave the fidelity gate explicitly open.

---

# 3. Official behavior research and compatibility matrix

Use official live docs as the source of truth for documented behavior. Initial entry points:

- CLI: https://www.superlogical.com/rex/docs/automate/cli
- Server API: https://www.superlogical.com/rex/docs/automate/server-api
- Lua API: https://www.superlogical.com/rex/docs/reference/lua
- Events: https://www.superlogical.com/rex/docs/reference/events
- Lua events: https://www.superlogical.com/rex/docs/automate/lua-events
- Lua scripts: https://www.superlogical.com/rex/docs/automate/lua-scripts
- Program Status Protocol: https://www.superlogical.com/rex/docs/build/program-status
- GPUI: https://github.com/Glass-HQ/gpui
- Zed terminal-view architectural reference: https://github.com/zed-industries/zed/tree/main/crates/terminal_view
- Android CLI: https://developer.android.com/tools/agents/android-cli
- Android CLI Journeys: https://developer.android.com/tools/agents/android-cli/journeys

Create `docs/compatibility/reference-behavior-matrix.md`. For every behavior, label one of:

- `observed-video`
- `verified-official-docs`
- `verified-by-test`
- `implemented`
- `intentionally-different`
- `inferred`
- `unknown`

Track domain hierarchy, target resolution, stable IDs, CLI flags, JSON output, exit codes, environment variables, server API method names/schemas, Lua functions, event names/payloads, block methods, status protocol semantics, multi-client behavior, and connection lifecycle.

The official REX CLI documentation describes sessions, windows, blocks, and clients; stable IDs; contextual variables such as `REX_SESSION`, `REX_BLOCK`, and `REX_SERVER`; and target resolution by full ID, label, position, or ID suffix. The server API documentation describes dynamically discoverable methods such as `rex api list`, `rex api describe`, and `rex api call`, with JSON schemas and examples. Verify all details against current docs before implementing them.

Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it. Replicating documented user-visible behavior and implementing an equivalent native protocol are distinct goals.

---

# 4. Domain model

Use stable, collision-resistant IDs that are never reused. Labels may change and must not serve as permanent identities.

- **Server:** A REX/Relayable service instance, version, protocol capabilities, endpoint, policies, and status.
- **Host profile:** Saved client-side connection parameters and user-visible host label.
- **Session:** A persistent group of related work, often associated with a project.
- **Window:** A tab/workspace within a session.
- **Block:** An application region within a window. First implementation is a terminal; do not build future block plugins prematurely.
- **Client:** Connected GPUI, CLI, or Android application.
- **PTY/process:** A server-owned pseudoterminal and its process tree/shell.
- **Layout:** The tree of blocks and split ratios in a window. Separate shared layout state from local client UI preferences where the reference behavior implies that distinction.
- **Event:** A sequenced/revisioned change to a server/session/window/block/client/status.
- **Program status:** Current structured status from terminal programs, including OSC 7501 support if confirmed.
- **Automation script:** A Lua script using the same typed server API and authorization model.

Use tagged IDs or explicit entity type fields to prevent cross-entity ambiguity. Keep stable server IDs and human labels separate. Document when deletion is permanent and whether a session close leaves audit history.

---

# 5. Architecture and repository layout

## 5.1 Proposed repository structure

Adapt to existing repo conventions instead of moving code solely to match this sketch.

```text
relayable/
├── AGENTS.md
├── full.md
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── mise.toml                 # Only if project pins are useful and agreed
├── crates/
│   ├── relayable-model/      # Domain types, IDs, revisions, snapshots
│   ├── relayable-protocol/   # Schema, messages, codec, version negotiation
│   ├── relayable-client/     # Connection lifecycle, RPC, state sync
│   ├── relayable-server/     # API, authorization, lifecycle, persistence
│   ├── relayable-pty/        # PTY/process ownership and supervisor boundary
│   ├── relayable-terminal/   # Terminal backend abstraction and scrollback
│   ├── relayable-status/     # Terminal status protocol parsing
│   ├── relayable-automation/ # Lua integration, after typed API stabilizes
│   └── relayable-cli/        # `relayable` binary
├── apps/
│   ├── linux/                # GPUI desktop client
│   └── android/              # Kotlin/Jetpack Compose app and Gradle wrapper
├── protocol/
│   ├── schema/
│   ├── examples/
│   └── compatibility/
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── agent-loop/
│   ├── compatibility/
│   ├── operations/
│   └── research/
├── packaging/
│   ├── systemd/user/
│   ├── systemd/system/
│   ├── desktop/
│   ├── deb/
│   └── rpm/
├── scripts/
└── tests/
    ├── protocol/
    ├── lifecycle/
    ├── reconnect/
    ├── multi-client/
    ├── terminal-compat/
    └── fixtures/
```

Don't create a crate for every label on day one. Split only when it establishes a useful dependency boundary. Android can be an independent Gradle project and must not make default Cargo tests require Android tooling.

## 5.2 Dependency direction

- Model types must not depend on GPUI, Android, OS views, or transport.
- Protocol maps model operations to versioned requests/events.
- PTY/process management is OS-specific and server-side.
- Server composes persistence, authz, PTY, model, protocol, and event dispatch.
- Client-core owns connection, RPC, resynchronization, and event sequencing.
- GPUI/Android/CLI are independent clients of the API.
- Lua must call the typed API and obey the same authorization/invariants; no hidden bypass.
- No cycles between crates.

Document choices in ADRs under `docs/adr/`; every ADR includes context, options, decision, consequences, and what evidence would cause reconsideration.

## 5.3 GPUI desktop

Use GPUI for the Linux app, but first prove a minimal window builds and runs in the current repository/toolchain. GPUI is under active development and can have breaking API changes: pin the version/revision, isolate GPUI-specific code within `apps/linux`, and do not leak GPUI types into shared crates.

Test Wayland and X11 on the target host where feasible. Verify keyboard input, IME/paste paths, clipboard, resizing, display scaling, font fallback, accessibility labels, context menus, and terminal performance. If GPUI cannot meet a required behavior, write an ADR and an evidence-backed tradeoff before switching UI framework.

## 5.4 Terminal engine selection

Do not write a terminal emulator from scratch unless the candidates fail a documented spike. Evaluate maintained Rust-compatible terminal engines and their licensing/API stability against:

- VT/ANSI coverage, alternate screen, OSC, mouse modes, bracketed paste, Unicode/wide glyphs, selection, hyperlinks, resize.
- Scrollback limits and memory behavior.
- Compatibility with common shells, `vim`/`nvim`, `less`, `top`/`htop`, multiplexers, full-screen TUIs, color, Unicode, and cursor modes.
- Separation between terminal parser/state and rendering view.
- How state can be rendered on Android later.
- Platform support, licensing, release cadence, and test quality.

Create a short benchmark/test app before choosing. Use Zed’s terminal-view design as a boundary reference, not as permission to copy source wholesale: the terminal backend/event loop should be isolated from GPUI view integration. PTY ownership is a server-side responsibility; a GUI view does not own the shell.

---

# 6. Native protocol and session behavior

## 6.1 Requirements

Choose a documented schema-driven protocol. Protobuf is a candidate; compare viable alternatives and record why the selected one fits. The protocol must have explicit boundaries for control requests, event sync, terminal streaming, auth, and admin/diagnostic operations.

Required capabilities:

- Handshake and protocol-version negotiation.
- Client kind/platform/version/capability declaration.
- Authentication and per-principal authorization.
- Request IDs, structured errors, clear timeout behavior, and cancellation where applicable.
- Stable entity IDs and monotonically increasing revisions where state changes.
- Snapshot request plus incremental event stream.
- Terminal input/output with bounded flow control and backpressure.
- Terminal resize, dimensions, relevant capabilities, and client reconnect behavior.
- Multiple clients attached to the same session with documented input arbitration.
- Heartbeats/timeouts that mark a client disconnected without killing the session.
- Resume cursor/token semantics where safe; full snapshot fallback when events/output cannot be replayed.
- Clear response for incompatible client/server versions.
- Redaction-safe diagnostics and optional tracing IDs.
- Tests for malformed, oversized, truncated, unknown, stale, and replayed messages.

Don't claim exact recovery of terminal byte streams unless the server guarantees it. A terminal screen snapshot and scrollback recovery are different capabilities. Document exactly what is retained and what is lost after a client disconnect or server crash.

## 6.2 Transport

- **Local:** Unix domain socket with restrictive ownership/mode; no network port required.
- **Remote:** authenticated encrypted native protocol. Choose a maintained TLS/QUIC/WebSocket-over-TLS/etc. approach based on requirements and practical cross-platform support. Do not roll custom cryptography.
- Bind remote listeners conservatively, off by default until secure configuration is complete.
- SSH may optionally be a tunnel/bootstrap path, but don't treat SSH as the fundamental application protocol if remote session functionality is intended to replace an SSH-centric workflow.
- Don't require Tailscale/other vendor infrastructure unless the evidence and product requirements explicitly justify it.
- Document NAT/traversal limitations rather than pretending direct remote connectivity always works.

## 6.3 Session lifecycle

First-release guarantees:

- Closing a client window does not destroy the server-owned session or PTY.
- A remote network disconnect does not destroy the server-owned session.
- Reopening the client requests a current snapshot and can reattach.
- A session can be explicitly stopped/destroyed through authorized API calls.

Do not promise survival after server crash or host reboot unless one of these is implemented and tested:

- a separate supervisor owns PTYs and allows server restarts to reattach;
- per-session supervisor services/processes preserve PTY descriptors;
- or another proven OS-level arrangement.

Host reboot necessarily ends ordinary processes unless a job is deliberately restored/restarted. Persist session definitions separately from live process state and offer clear restart policy instead of false guarantees.

## 6.4 State and persistence

Use SQLite or another small transactional store for durable metadata after evaluating migration, locking, and test needs. Persist identities, labels, layouts, host-independent session metadata, preferences, and selected audit/event data. Do not store unbounded PTY output as ordinary rows.

Define:
- DB schema and migrations.
- Retention and pruning.
- Crash-consistent writes.
- Event revisions and snapshot consistency.
- What is restored after a process/server restart.
- Backup/export behavior if needed.

Bound per-terminal scrollback and outgoing queues. Handle high-volume terminal output without unbounded memory.

---

# 7. Linux native operation and packaging

## 7.1 Filesystem convention

Follow XDG conventions rather than inventing paths:

- Executables: packaged `/usr/bin/relayable` and an appropriate desktop launcher, or user install under `~/.local/bin/`.
- Per-user config: `$XDG_CONFIG_HOME/relayable`, default `~/.config/relayable/`.
- Machine config: `/etc/relayable/` only for system deployment.
- Per-user persistent state: `$XDG_STATE_HOME/relayable`, default `~/.local/state/relayable/`.
- User runtime/socket: `$XDG_RUNTIME_DIR/relayable/` or another compliant path derived at runtime.
- System runtime: `/run/relayable/` only for a system service.
- User cache: `$XDG_CACHE_HOME/relayable`, default `~/.cache/relayable/`.
- Desktop entry/icon under standard system or user application directories.

Use absolute paths only where required by systemd or packaging. Never assume `$HOME` or `$XDG_RUNTIME_DIR` exists before checking service context.

## 7.2 systemd modes

**Workstation default:** `systemd --user` server, running under the logged-in user’s identity. It owns that user’s sessions and executes processes with that user’s normal permissions. The local client connects over a user-owned Unix socket.

**Remote host:** a system service is an optional deployment mode for a dedicated, headless machine, but choose the execution identity model deliberately. A daemon running as one `relayable` account must not silently impersonate all remote users or run everybody’s commands under that account. Per-user workers/services may be required.

Create and test:
- User service unit and documentation.
- Optional system unit only after security review.
- Graceful shutdown policy and `TimeoutStopSec` choice based on lifecycle behavior.
- `Restart=on-failure` and backoff without restart loops.
- Runtime/state directory permissions.
- Journald diagnostics.
- Socket activation only if justified and correctly implemented.
- User lingering instructions as an opt-in choice for post-logout persistence.

Do not install or enable a service system-wide during autonomous development. Tests should use temporary user units or a foreground test server unless the user approves otherwise.

Before unit generation, inspect real server command-line options. `ExecStart` must use an implemented foreground mode; don't invent `relayable server start --foreground` unless the CLI implements it and tests it.

## 7.3 Packaging

Prioritize:
1. reproducible debug/release build and local user install;
2. `.deb` package for a Debian/Ubuntu-based target if that is the user's distribution;
3. `.rpm` if relevant to target environments;
4. other distro packaging after the base path is correct.

Include binary, units, desktop entry, icons, docs, config examples, and migration rules. Never ship reference screenshots as app resources. Do not build Flatpak/Snap unless their sandbox/device/PTY integration has a tested plan; record tradeoffs.

## 7.4 Operations and diagnostics

Implement a diagnostics command (`relayable doctor` or the documented equivalent) that reports:
- server/client versions and protocol compatibility;
- service state and journal log hint;
- endpoint/socket permissions;
- config parse result;
- PTY availability and shell path;
- network transport/listener status without printing secrets;
- DB schema version and migration state;
- Android toolchain status only when requested.

Use structured logs, sensible redaction, trace IDs, and log levels. A malformed client request must not crash the server.

---

# 8. CLI, API, Lua, events, and program status

## 8.1 CLI design principles

The exact command set must be reconciled with observed video commands and current official reference docs. Create a command matrix before locking syntax.

Likely command families, subject to verification:

```text
relayable version
relayable doctor
relayable server status|run|inspect
relayable ls
relayable new
relayable session inspect|rename|close
relayable window list|new|close
relayable split
relayable run
relayable block inspect|close
relayable attach|detach
relayable send|send-key|capture|wait
relayable events
relayable hosts list|add|remove|inspect
relayable clients list|disconnect
relayable api list|describe|call
relayable config check|reload
relayable do
```

Do not implement every command superficially. Implement a small, coherent command set and expand from actual requirements. Prefer exact reference command behavior where documented and a compatibility matrix where deliberate differences remain.

CLI requirements:
- Human-readable output by default; machine-readable `--json` with stable schemas.
- Stable exit code semantics documented and tested.
- Useful `--help` and subcommand help.
- Context-aware defaults when executed inside a managed terminal.
- Explicit host/session/window/block/client targeting.
- Stable IDs returned by creation commands.
- Ambiguous targets fail before mutation; never guess silently.
- Errors are actionable and don't print secrets.
- CLI works without the GUI.

Reference documentation currently describes `REX_SESSION`, `REX_BLOCK`, and `REX_SERVER` as context variables. Relayable should choose whether to use `RELAYABLE_*`, support aliases for compatibility, or set both; document the decision in an ADR and test it. Do not mix namespaces accidentally.

## 8.2 Server API

Build a typed API with request/response schemas and stable error codes. Provide a discovery mechanism equivalent in spirit to listing methods, describing schemas, and calling methods from the CLI. Use one source of truth for validation, docs, Lua bindings, and CLI where practical.

Do not dynamically expose arbitrary internal methods. Methods need an explicit registration/authorization policy. Add integration tests for malformed inputs, missing fields, unknown methods, target ambiguity, authorization failure, and backwards/forwards compatibility.

## 8.3 Lua

Lua 5.1 compatibility is a candidate because the reference docs describe Lua 5.1 plus standard library and a `rex` API. Confirm scope and licensing/embedding requirements before implementation. The initial goal is not to embed a full automation runtime before the typed API and event model stabilize.

When implemented, Lua calls must use the same typed API, context, structured errors, and permission model. Event handlers should not deadlock the server. Long-running `do` scripts need cancellation, timeouts, output/log policy, and clear failure handling.

## 8.4 Events

Server/session/terminal events need a versioned envelope with event type, payload, entity/session context, sequence or revision, and timestamp where useful. Ignore unknown optional fields safely. Define subscription scope, replay behavior, and event retention explicitly.

Potential events inspired by documented reference behavior include session creation/destruction, client join/leave/change, window/block lifecycle, terminal title/PWD/process changes, bell, child exit, and program-status changes. Implement only events with defined semantics and tests.

## 8.5 Program Status Protocol (`OSC 7501`)

Treat OSC 7501 as a terminal-level integration, not the native Relayable wire protocol. Verify the current specification before implementation: https://www.superlogical.com/rex/docs/build/program-status

The current reference describes states including `idle`, `working`, `done`, `blocked`, `error`, and `clear`, with possible progress, app identity, title/message, blocked kind, and nested IDs. Exact encoding, base64 requirements, lifetime semantics, feature detection, malformed sequence handling, and unknown-state behavior must follow the spec, not these shorthand notes.

Parse statuses without corrupting ordinary terminal output. The model should be available to desktop, CLI, and future Android notification surfaces. Unknown OSC sequences must not crash the emulator. Add byte-level parser fixtures.

---

# 9. Linux UI implementation — reproduce the references

## 9.1 Design process

Do not begin by inventing design tokens. First build the video UI catalogue. Then extract a token sheet from observed examples: background levels, text colors, border/divider colors, accent colors, spacing increments, typography sizes/weights, corner radii, icon rules, panel widths, row heights, terminal font, and animation timing if visible.

Create `apps/linux/README.md` describing:
- reference screens/states being implemented;
- source timestamp per screen;
- component mapping;
- design tokens;
- known fidelity differences;
- screenshot-test procedure.

## 9.2 Implementation order

1. App shell/window geometry matching the main video workspace.
2. Session/window/block navigation and the terminal area.
3. Split layout and focus behavior.
4. Session switcher and command palette if observed.
5. Connection/remote host visuals.
6. Inspector/status panels and secondary interactions actually shown.
7. Empty/loading/error/disconnected states.
8. Refinement via screenshot comparisons and interaction tests.

The app must be usable during each stage. Use realistic data, but label sample/mock data clearly. Avoid polishing non-reference features while core geometry remains wrong.

## 9.3 Screenshot loop

For every reference screen:
1. capture a reference still with source timestamp where accessible;
2. implement the state using live UI components;
3. capture Relayable's window at matching dimensions and scale;
4. compare side-by-side or with an image-difference tool;
5. file mismatches by impact: layout/hierarchy, proportions, colors/contrast, typography, icons, microspacing;
6. fix the highest-impact mismatches first;
7. rerun the screenshot and relevant interaction tests.

Don't overfit to one screenshot and break resize behavior. Verify at a baseline reference size and at a smaller/larger desktop size. Document fonts/platform differences.

## 9.4 UX functionality

- Keyboard-first navigation; shortcuts are defined only when verified or deliberately specified.
- Clear active, selected, focused, hovered, connected, reconnecting, and error states.
- Session/window/block labels and stable IDs are separate concepts.
- Focus moves predictably after split/close/navigation.
- Closing a terminal view is distinct from destroying its process/session.
- Host context is always apparent before issuing remote commands.
- Clipboard, paste, IME text, terminal control keys, mouse reporting, selection, and resize each have correct separate paths.
- Accessibility labels and keyboard focus behavior must be tested.

---

# 10. Android client — Android CLI only as the standard workflow

The user has installed Rust and Android CLI, and may already have a JDK. **Do not require Android Studio.** Use Google's official Android CLI, Gradle wrapper, Android SDK command-line tools, ADB, and an emulator or authorized physical device. Official entry points:

- https://developer.android.com/tools/agents/android-cli
- https://developer.android.com/tools/agents/android-cli/journeys
- https://developer.android.com/agents/skills/devtools/android-cli/skill

Android CLI exposes tools for environment/SDK setup, project creation, emulator/device interactions, APK install/run, UI hierarchy (`android layout`), screenshots (`android screen capture`), and Journeys. The exact commands may change: inspect the installed version's help and read the official installed skill references, especially interaction and Journey guidance, before device interaction.

## 10.1 First run

1. Run `command -v android`, `android -V`, `android -h`, and `android info`.
2. If available and appropriate, run `android init` to install/use the Android CLI agent skill. Inspect the changes it makes.
3. Use `android <subcommand> -h` before every unfamiliar operation. Do not assume flags based on examples from a different CLI version.
4. Check Java with `java -version`, `JAVA_HOME`, Gradle wrapper, and SDK paths. If JDK is missing/incompatible, report it and suggest a suitable `mise` pin/config; do not silently install or globally change Java.
5. Check `adb devices`, installed SDK packages, available emulators, storage, and permissions. Don't create/download a large emulator image without confirming it is needed.
6. Create or adapt the project using supported Android CLI templates or the Gradle/Kotlin project conventions found in the repository.
7. Commit the Gradle wrapper and pin compatible plugins/dependencies; don't depend on a system Gradle version.
8. Build and run using CLI. Do not launch Android Studio, depend on its running process, or make Compose preview rendering through Android Studio part of the build loop.
9. If Android CLI offers optional usage metrics settings, inspect support and opt out using the documented flag/config when feasible; do not make unsupported command assumptions.
10. Record versions in `docs/operations/android-toolchain.md` and provide a `scripts/android-check.sh` that reports actionable missing pieces without auto-modifying the host.

If a tool is missing, the agent must stop short of machine-wide installation, report what is missing, and continue unrelated tasks. Since the user has said these tools are installed, prefer discovering and using them.

## 10.2 Android architecture

- UI: Kotlin + Jetpack Compose.
- Transport/core: share schema and compatibility tests with Rust; evaluate generated Kotlin bindings or a compatible Kotlin protocol implementation. Avoid a large FFI surface unless real reuse benefits are proven.
- Terminal session/process: always server-owned on the Linux host.
- Local mobile UI state: selected host/session, keyboard state, font scale, scroll position, local preferences.
- Remote domain state: server/session/window/block IDs, revisions, statuses, authorized operations.
- Use Android lifecycle-aware connection handling; app suspension must not terminate server processes.

## 10.3 Android MVP

- Device pairing and revocation.
- Host list and connection state.
- Session/window/block overview.
- Session status and recent activity.
- Attach, detach, reconnect, resynchronize.
- Terminal output/input, scrollback, selection, copy/paste, font size.
- Mobile key toolbar for Ctrl, Alt, Esc, Tab, arrows and other frequently used terminal keys.
- Correct IME composition and keyboard show/hide behavior.
- Network transitions (Wi-Fi/mobile/offline) and app foreground/background transitions.
- Explicit errors for unreachable host, expired/revoked credentials, protocol mismatch, and unauthorized operation.
- Notifications only after core state synchronization works and only with an explicit user privacy preference.

## 10.4 Android tests

Use ordinary unit/instrumentation tests where possible and Android CLI screenshot/layout/Journey functionality for end-to-end flows. Follow the installed official `interact.md` and `journeys.md` guidance before using those capabilities.

Minimum automated journey:
1. launch app;
2. connect to a test Relayable server;
3. view session list;
4. attach to an existing session;
5. send text through the terminal;
6. background/resume or simulate connection loss;
7. reconnect and verify server-side session still runs;
8. revoke/disconnect the device and verify access is rejected.

Capture screenshots and layout hierarchy for major screens. Never claim a journey passed unless it actually ran on a device/emulator and the assertions were checked.

---

# 11. Security, identity, and multi-client safety

Security review is a release gate, not a later cleanup task.

- Local socket: restrictive filesystem permissions, peer identity where available, no world-readable credentials.
- Remote endpoint: encryption, client authentication, authorization scope, rate limiting, explicit pairing/revocation, conservative bind defaults.
- Credentials: OS keyring/secure store when feasible; avoid plaintext long-lived tokens; redact diagnostics.
- Server process: least privilege; do not run remote shells as root by default.
- Multi-user model: one dedicated daemon account cannot casually launch commands with arbitrary users’ permissions. Implement a deliberate per-user execution model or scope initial release to one user per server.
- Input: all attached clients share a terminal input stream only according to documented semantics; display connected clients and avoid surprising simultaneous writes.
- API: validate request sizes, IDs, path handling, command options, schema fields, and authorization before performing actions.
- IPC: enforce message and queue size limits; protect against slow-client memory growth.
- Logging: never emit private terminal output, tokens, keys, or full command arguments by default.
- Updates: package upgrades must not silently kill sessions; define how the service drains/restarts or preserves session supervision.
- Dependencies: audit licenses and known vulnerabilities using available approved tools; don't send private code to third-party analyzers without approval.
- Tests: fuzz parsers/protocol boundaries where practical; test unauthorized calls and malformed payloads.

Do not open firewall ports or deploy to a remote server as part of this agent loop without explicit human approval.

---

# 12. Performance and memory engineering

Performance is based on measurements, not guesses or marketing targets. Video 3 is a key reference and must be investigated before choosing comparable benchmarks.

Create `docs/performance/benchmark-plan.md` and an automated benchmark/test harness that records:

- idle server RSS and CPU;
- memory per idle terminal and per active terminal;
- scrollback size and retained bytes;
- output throughput and UI latency under sustained output;
- many mostly idle sessions;
- many connected clients;
- attach/reconnect time;
- terminal resize/render cost;
- event queue and DB growth;
- behavior under one slow or suspended client.

Every result must record OS/kernel, CPU/RAM, Rust toolchain, commit, build profile, terminal engine, number of sessions, test workload, duration, units, measurement command, and raw output. Don't compare unlike workloads. Add regression thresholds only after a stable baseline exists. Avoid flaky hard performance gates before measurement variance is understood.

Resource rules:
- bounded scrollback and queues;
- avoid one unbounded task per client/event;
- cancel/clean up disconnected client tasks;
- avoid polling loops when event-driven notification is available;
- benchmark before adding caching/complexity;
- keep terminal byte streams out of metadata DB;
- test idle and high-output sessions separately.

---

# 13. Testing and acceptance criteria

## 13.1 Unit tests

Cover ID serialization, target matching, command/context resolution, protocol encoding/decoding, API schema validation, event revision logic, layout tree transforms, status parser, config parsing, migrations, and error mapping.

## 13.2 Integration tests

At minimum:

1. start server in test mode;
2. connect CLI client;
3. create a session and verify stable IDs;
4. create window/split/block;
5. send and receive terminal input/output;
6. resize and verify `stty size` or equivalent;
7. disconnect the client and verify the child remains alive;
8. reconnect and recover current session state;
9. create a second client and verify multi-client semantics;
10. terminate session explicitly and verify cleanup;
11. restart server and assert only the recovery properties actually implemented;
12. reject unauthorized client and invalid request;
13. stress malformed/oversized messages;
14. run a terminal compatibility smoke set on available programs;
15. verify OSC 7501 behavior using fixtures if implemented.

## 13.3 UI tests

- Reference-based screenshot captures for every catalogued key screen.
- Keyboard navigation and command palette tests.
- Split/focus/close interaction tests.
- Disconnected/reconnecting/error states.
- Local vs remote host indication.
- Close client, reopen, and verify process persists.

For video fidelity, keep a discrepancy list and compare screenshots at matching dimensions/scale. Store test output and describe any unavoidable platform differences.

## 13.4 Linux smoke test

A script should build, launch a test server, create a session, send input, verify output, disconnect the client, reconnect, verify continued process execution, shut down cleanly, and print concise logs. It must run without installing a system service or requiring root.

## 13.5 Build matrix

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` (adapt feature scope if platform libraries require it; document skips)
- `cargo test --workspace`
- Linux desktop build with actual GPUI dependencies
- Linux UI smoke/screenshot tests where display backend is available
- `./gradlew test` and Android assemble/test tasks when Android project exists and SDK is available
- protocol compatibility/fixtures
- package build tests when relevant

Only report commands that actually ran. If a check cannot run because a dependency/display/device is missing, report the exact blocker and keep the task open.

## 13.6 User-value acceptance checklist

- [ ] Product consistently branded Relayable; no reference-product vendor attribution on Relayable surfaces.
- [ ] UI reference review has honest evidence records for all five videos.
- [ ] Main Linux workspace is visibly based on the reference UI, not a generic redesign.
- [ ] Session/window/block hierarchy and behavior are implemented.
- [ ] Server owns PTY; GUI closure doesn't kill it.
- [ ] Remote disconnect doesn't kill server-owned sessions.
- [ ] Reconnect resynchronizes state or clearly states limitations.
- [ ] CLI can perform core operations without GUI.
- [ ] Local Unix socket has safe permissions.
- [ ] Remote mode cannot be accidentally exposed unauthenticated.
- [ ] systemd user-service path works in tests/docs.
- [ ] Android project can build from CLI without Android Studio.
- [ ] Protocol test client can simulate mobile suspend/reconnect before Android UI is complete.
- [ ] Memory/CPU benchmark harness exists before performance claims.
- [ ] Docs explain which persistence guarantees are real and tested.

---

# 14. Multi-agent orchestration

The lead agent is responsible for architecture, ordering, task assignments, integration, tests, and user-facing checkpoints. Subagents are workers, not independent product owners.

## 14.1 Agent roles

Use the roles below when Codex multi-agent spawning is supported. Keep the number of simultaneous workers modest (usually 3–5) and only spawn work that can proceed independently. Too many agents editing shared surfaces is a failure mode.

### A. Lead / Integrator (always active)

Owns the overall plan, repository, dependency boundaries, task board, architecture decision records, shared models/protocol decisions, merge/integration, and the final validation. Assigns file ownership before delegating. Resolves conflicts. Does not delegate the responsibility for integration or test truthfulness.

### B. Reference Research and UI Audit agent

Owns only:
- `docs/research/video-review.md`
- `docs/research/ui-catalog.md`
- `docs/research/open-questions.md`
- `docs/compatibility/reference-behavior-matrix.md` (research portion only; coordinate before shared edits)

Tasks: inspect the five videos and official docs, review GoRex as a supplementary reference, record timestamped/source-specific evidence, extract visible layouts/workflows/CLI behavior, identify UI tokens and unknowns, and verify license status before proposing any reuse. Do not write production UI code or alter architecture. Report evidence with URLs/timestamps and confidence. Do not copy GoRex source or assets into the project.

### C. Protocol / Server agent

Owns assigned protocol/server files after the lead fixes the core ADR and interfaces. Implements a minimal usable control protocol, server lifecycle, session state, authorization boundaries, and integration tests. Must not alter GPUI UI files or Android app.

### D. Terminal / PTY agent

Owns `relayable-pty`, terminal backend investigation/spike, terminal process tests, flow control, and terminal compatibility fixtures. Must coordinate interface changes with the protocol/server agent through a written contract; no overlapping edits to server API files without approval.

### E. Linux UI / GPUI agent

Owns `apps/linux` and `docs` specific to UI implementation. Must consume the approved client API; must not redefine domain/protocol models independently. UI priorities come from the reference catalogue. No substitute design without a documented evidence gap and lead approval.

### F. Android agent (later parallel work)

Owns `apps/android`, Android build scripts, Journey/UI tests, and `docs/operations/android-toolchain.md`. Starts with a protocol simulator/test client and mobile UI shell; no dependency on Android Studio. Does not change the Rust protocol schema without an interface review.

### G. QA / Performance agent

Owns tests/benchmarking docs and test harnesses assigned by the lead. Reviews other agents’ claims with commands, logs, and reproducible evidence. Must never “fix” failing tests by deleting assertions or weakening acceptance criteria without a reviewed ADR.

## 14.2 Assignment format

Every delegation must include:

- **Goal:** one measurable outcome.
- **Owned paths:** files/directories the worker may edit.
- **Read-only context:** files/specs to consult.
- **Interfaces:** exact agreed types/APIs; if undecided, return a proposed interface before coding.
- **Acceptance criteria:** concrete tests/outputs.
- **Out of scope:** what must not be changed.
- **Validation commands:** the commands to run.
- **Return report:** files changed, decisions, commands/results, risks, unfinished items.

Example:

> Goal: Produce the video-based workspace UI catalogue. Owned paths: `docs/research/video-review.md`, `docs/research/ui-catalog.md`, `docs/research/open-questions.md`. Do not edit production code, protocol, or branding. Each observed screen must include source URL and timestamp. Label inaccessible footage honestly. Return a list of high-confidence UI requirements and unresolved questions.

## 14.3 Parallelism policy

Parallelize research and independent spikes early. Don't parallelize tightly coupled implementation until API contracts and file ownership are explicit.

Good parallel tasks:
- Video/document research vs toolchain inventory.
- Terminal engine spike vs protocol serialization comparison.
- Linux packaging research vs Android CLI environment check.
- Android project smoke test after the protocol schema/interface is frozen enough.
- Separate test/benchmark harnesses that do not share files.

Bad parallel tasks:
- Multiple agents editing the same root manifest or protocol schema.
- GPUI agent inventing its own session model while server agent creates another.
- UI agent implementing mock semantics that conflict with server API.
- Two agents changing the CLI command tree concurrently.
- Agent asking another agent to invent proof or claim screenshots passed without running them.

## 14.4 Interface-first workflow

Before parallel coding, the lead writes an interface note or ADR containing:
- entity types and IDs;
- session snapshot schema;
- request/event envelope;
- connection state model;
- terminal stream interface;
- error representation;
- crate ownership and dependency direction.

Workers can propose changes, but cross-cutting changes go through the lead. Update schemas and contract tests before changing all consumers.

## 14.5 Agent review protocol

When a worker returns:
1. inspect the diff and `git status`;
2. run targeted tests independently when feasible;
3. check changes for accidental unrelated files/secrets;
4. review public API and dependency direction;
5. request a revision if criteria aren't met;
6. integrate only after the worker report matches the observed diff;
7. update the ledger and worklog;
8. only then assign dependent work.

If subagents work on shared filesystem branches, do not let the lead reset/clean their work. Prefer distinct worktrees/branches when supported and beneficial. Never overwrite another agent's uncommitted changes.

---

# 15. Persistent autonomous work loop

Codex must use a durable loop, not an open-ended stream of untracked changes. Set up these files:

```text
docs/agent-loop/STATUS.md       # current phase, objective, blockers
 docs/agent-loop/TASKS.md       # ordered task ledger, owners, status
 docs/agent-loop/DECISIONS.md   # index of decisions; detailed ADRs under docs/adr
 docs/agent-loop/WORKLOG.md     # timestamp/checkpoint log and commands
 docs/agent-loop/NEXT.md        # precise instructions for the next resumed run
```

Use statuses `BACKLOG`, `READY`, `IN_PROGRESS`, `BLOCKED`, `REVIEW`, `DONE`. Every task has an ID, dependencies, owner, acceptance criteria, test commands, and evidence path. Avoid adding a task without a reason or measurable result.

## 15.1 Loop algorithm

Repeat the following steps until there are no safe/actionable tasks or a true stop condition is met:

1. **Recover context:** read `full.md`, `AGENTS.md`, current status, task ledger, worklog tail, open issues, current branch/worktree state, and latest test reports.
2. **Inspect reality:** check `git status`, pending changes and running tests; don't assume the last agent completed correctly.
3. **Select the next vertical slice:** choose the highest-value unblocked task that advances a real end-to-end path or closes a release gate. Respect dependencies; avoid broad speculative refactors.
4. **Split bounded work:** launch independent subagents for research, tests, UI, or subsystems with explicit path ownership. Keep one integrator responsible for shared interface files.
5. **Define the contract:** record API/schema changes before parallel implementation.
6. **Implement:** make the smallest useful change that can be tested. Do not leave the app permanently as mock data or TODO stubs.
7. **Validate:** format, compile, targeted tests, integration smoke, and relevant UI/device tests. Run the exact commands; record exit codes and limitations.
8. **Review:** inspect diff, dependencies, security implications, visual fidelity, and code quality. Ensure no agent has silently weakened acceptance tests.
9. **Fix regressions:** do not proceed to dependent tasks while the current increment is broken. Reproduce and isolate failures, then repair.
10. **Checkpoint:** update task status, evidence, worklog, next steps, and decisions. Commit only if repository policy and user environment make commits appropriate; never amend/rewrite user history without approval.
11. **Select next work:** prioritize in this order unless research justifies an adjustment:
    - build/compilation blockers;
    - server/CLI vertical slice;
    - reference UI fidelity blockers;
    - protocol correctness/security;
    - lifecycle/reconnect;
    - terminal compatibility;
    - Linux service/packaging;
    - Android protocol simulator and CLI build path;
    - Android client MVP;
    - perf/observability/docs/polish.
12. **Continue:** do not stop just because one feature works. Move to the next unblocked, high-value task until a stop condition applies.

## 15.2 Stop conditions

Stop and clearly report only when:
- human approval is required for a destructive/privileged/external action;
- a required credential or inaccessible reference is essential to the next step;
- a hardware/display/emulator dependency cannot be used and no valid fallback exists;
- an external API/license/security question needs human decision;
- repeated attempts demonstrate a genuine blocker and a new approach requires user input;
- the user-configured time/token/iteration budget is exhausted;
- the requested milestone's acceptance criteria are all met and the next phase requires approval.

Do not stop merely because a phase checklist was written, because an agent returned, or because the initial answer is complete. If blocked, continue with other independent tasks and label the blocker.

## 15.3 Bounded loops and checkpoints

Work in a series of bounded iterations, usually one meaningful task or small vertical slice per checkpoint. Before an expensive test/build, estimate the cost from prior run time if known. Avoid running the full matrix after every one-line edit; run targeted tests first, then full validation at integration checkpoints.

After each iteration write:
- objective and files changed;
- exact commands and exit codes;
- tests passed/failed/not run and why;
- screenshot/video evidence paths if applicable;
- known regressions and risks;
- current blockers;
- next 3–8 actionable tasks ordered by dependency/value.

At the start of a resumed session, trust repository artifacts—not prior chat messages—as the source of work state.

## 15.4 Long-running safety

- Never spawn an unbounded number of agents.
- Never run destructive commands to “clean” the repo without inspecting consequences.
- Never install a service or open a port automatically.
- Never start an unbounded load test on the user's machine.
- Always time-box video-access attempts and long tests.
- Avoid duplicate tasks across agents; use the ledger as source of truth.
- Avoid concurrent Cargo builds that compete for the same build directory unless isolated.
- Do not claim to have “run for days” or keep working in the background when the current tool/session has ended. The loop only continues while Codex is actively executing; each resume must read the checkpoint.

---

# 16. Initial roadmap and dependency graph

## Phase 0 — Repository and evidence audit

Tasks:
- inspect repository/toolchain and existing changes;
- create/update `AGENTS.md` with concise repo conventions (refer to this file rather than duplicating everything);
- inspect all five video references and official docs;
- create UI catalogue and compatibility matrix;
- GPUI minimal Linux spike;
- terminal engine comparison and spike;
- protocol serialization/transport ADR;
- Android CLI/JDK/SDK check, without unnecessary installs;
- create task board and CI/smoke scripts.

Exit gate: evidence status clear, architecture ADRs approved internally, and a buildable minimal shell/workspace exists if repository conditions permit.

## Phase 1 — Server + CLI vertical slice

Tasks:
- model and IDs;
- local foreground server and local Unix socket;
- session/window/block basics;
- PTY ownership and terminal I/O;
- create/list/inspect/close CLI commands;
- integration smoke test;
- client disconnect leaves PTY/session alive;
- snapshot/reconnect minimal flow.

Exit gate: automated test proves session survives client disconnect and can be reattached.

## Phase 2 — Reference-faithful GPUI client

Tasks:
- match main workspace geometry based on video catalogue;
- connect to live server, no core mock behavior;
- render terminal, session/window/block hierarchy;
- split/focus/resize/close operations;
- session switcher/command palette/other screens supported by evidence;
- screenshot and keyboard interaction suite;
- error/disconnect/reconnect states.

Exit gate: screenshots compare against the highest-confidence reference screens; discrepancies documented; core actions use real API.

## Phase 3 — Remote protocol and auth

Tasks:
- versioned network transport and encryption;
- pairing/trust/revocation;
- host connection profiles;
- state snapshot/event stream;
- flow control and reconnect;
- multi-client semantics;
- security integration tests;
- remote systemd deployment documentation.

Exit gate: remote clients cannot attach without authorization, and dropping network doesn't kill sessions.

## Phase 4 — Linux operations and performance

Tasks:
- systemd user-service unit and tests;
- optional system-unit/least-privilege design;
- user install/package basics;
- XDG filesystem conventions;
- journald, doctor, diagnostics;
- performance harness based on research evidence;
- TUI/OSC compatibility.

Exit gate: fresh-environment instructions work without root for local mode; system-service security reviewed before enabled use.

## Phase 5 — Android protocol/client groundwork

Can start earlier as a separate small agent task after protocol contract stabilizes.

Tasks:
- install/use Android CLI skill only if necessary;
- record SDK/JDK state;
- build minimal Compose app via CLI/Gradle without Android Studio;
- protocol test client/simulator for mobile reconnect;
- create Kotlin protocol models/bindings;
- host/session dashboard UI;
- UI hierarchy/screenshot/Journey tests.

Exit gate: Android build and automated main journey run successfully from shell; client can list and reconnect to a test session.

## Phase 6 — Android terminal MVP and shared product polish

Tasks:
- terminal rendering/input/key toolbar;
- lifecycle and network state transitions;
- pairing/revocation UX;
- copy/paste, selection, scrollback, font-size;
- status indicators and optional notifications;
- multi-client conflict behavior;
- release checklist and docs.

Exit gate: end-to-end Android test proves the same Linux session continues through background/foreground and network disconnect/reconnect.

---

# 17. Initial work queue

Create the actual ledger and refine priorities after repository/video inspection. Suggested first tasks:

| ID | Task | Depends on | Owner | Done when |
|---|---|---|---|---|
| BOOT-001 | Inspect repo, toolchain, instructions, working tree | None | Lead | Inventory recorded; user changes preserved |
| REF-001 | Review video 1 and catalogue main UI | BOOT-001 | Research | Timestamped findings + confidence |
| REF-002 | Review videos 2–5 | BOOT-001 | Research | Each review honestly reports evidence/access |
| DOC-001 | Verify official CLI/API/Lua/events/status docs | BOOT-001 | Research | Compatibility matrix created |
| ADR-001 | Domain and dependency boundaries | BOOT-001, DOC-001 | Lead | Decision recorded |
| ADR-002 | Protocol schema/transport alternatives | DOC-001 | Lead + protocol agent | Testable interface decision |
| UI-001 | GPUI minimal Linux build/run proof | BOOT-001 | UI agent | Window launches or blocker captured |
| TERM-001 | Terminal engine candidates and prototype | BOOT-001 | Terminal agent | TUI/input/resize spike report |
| AND-001 | Android CLI/JDK/SDK inventory | BOOT-001 | Android agent or lead | Toolchain report; no global changes |
| CORE-001 | Implement model/server/CLI vertical slice | ADR-001, ADR-002 | Server agent | Integration lifecycle test passes |
| UI-002 | Implement main reference workspace | REF-001, UI-001, CORE-001 API | UI agent | Screenshot + live terminal flow |
| QA-001 | Establish smoke/CI tests | BOOT-001 | QA agent | CI commands documented and run |

The coordinator may split or re-sequence tasks once evidence is in hand. Do not blindly follow this table when actual dependencies differ.

---

# 18. Definition of done for any feature

A task is not `DONE` just because code exists. It must satisfy all applicable items:

- Required behavior exists in working code rather than a stub/mock.
- Relevant tests exist and run.
- Compile/lint output is clean or exceptions are documented.
- Public API/docs/config are updated.
- Error paths have meaningful behavior.
- Security and resource limits are reviewed.
- UI changes are compared to reference footage when visual fidelity applies.
- At least one agent other than the original implementer or the lead has reviewed high-risk protocol/security work when agent support exists.
- Exact validation commands/results are stored in the worklog.
- Remaining shortcomings are explicit and tracked.

## 18.1 Final-report format

When a milestone completes or a true blocker occurs, report:

1. What now works end-to-end.
2. UI fidelity evidence and the screens that remain unverified.
3. Exact validation commands with pass/fail/not-run states.
4. Toolchain requirements found missing; do not say the user must install tools that are already present.
5. Security/persistence guarantees and non-guarantees.
6. Files/decisions changed.
7. Current blockers and next 3–8 tasks.
8. Whether agents were truly spawned or work was done sequentially.

Do not summarize unimplemented ideas as completed features.

---

# 19. First execution instructions for Codex

Begin immediately; don't answer with another plan only.

1. Inspect the repository and toolchain using §0.4. Record results before code changes.
2. Read all existing agent instructions and preserve user modifications.
3. Create or update `AGENTS.md` to be short and useful; point it to `full.md` for product-level requirements.
4. Create `docs/agent-loop/STATUS.md`, `TASKS.md`, `DECISIONS.md`, `WORKLOG.md`, and `NEXT.md`.
5. Spawn bounded agents for video/UI research, official behavior docs, GPUI/terminal spike, and Android environment check if Codex supports subagents. Give each separate owned paths. If unsupported, do the tasks sequentially.
6. Prioritize actual inspection of the five videos. Don't pretend this specification already contains a full frame-by-frame analysis.
7. Write ADRs for domain model, terminal engine, protocol serialization/transport, and local/remote process ownership based on evidence.
8. Build the smallest useful vertical slice: foreground Rust server owns a PTY; CLI creates a session; a client can attach/send/read; disconnecting the client leaves the session running; reconnect retrieves current state.
9. In parallel only after the UI catalogue and API contract exist, implement the main reference workspace in GPUI. Keep visual fidelity as an acceptance gate.
10. Add Linux smoke tests and CI before broad feature work.
11. Check Android CLI, JDK, SDK, and `adb`; use only the installed toolchain and official Android CLI workflow. Do not use or require Android Studio.
12. Continue the task loop (§15), checkpointing after each bounded iteration. Do not stop at documentation, scaffolding, or a static mock UI.
13. If a decision requires the user, continue other safe tasks and write a specific question with the options/consequences to `docs/agent-loop/NEXT.md`.
14. At the end of each active run, ensure `NEXT.md` tells the next run exactly what to inspect and do next, including the first command to run. The project must always be resumable from files alone.

**First concrete deliverable:** repository inventory, task ledger, and `docs/research/video-review.md` with all five videos accounted for honestly. **First engineering deliverable:** a tested local server/CLI/PTY lifecycle slice. **First visual deliverable:** a high-fidelity Relayable GPUI workspace derived from the reference catalogue, not an invented generic UI.

---

# 20. Reference index

Use these URLs as research entry points and verify current content when implementation depends on it:

- Video 1, pre-alpha demo: https://youtu.be/XyxShCZrGiw?si=YbQqAJ7xH62eGnHe
- Video 2, high-level terminal multiplexer architecture: https://youtu.be/Y6nFMmUPzXM?si=BgjaBfs8jC43QHo5
- Video 3, server memory optimizations: https://youtu.be/T5gV6anSt-4?si=60tv3fmAbn1iu3OQ
- Video 4, remote sessions demo: https://youtu.be/PdwTjSBW6Y8?si=m9GAagbhNl0IFpQS
- Video 5, CLI demo: https://youtu.be/9fcfDF8SBnc?si=SEC1FMxrXnEGK8eR
- Supplemental public UI/behavior reference: GoRex (Rex clone): https://github.com/egoist/gorex
- Official REX CLI: https://www.superlogical.com/rex/docs/automate/cli
- Official REX server API: https://www.superlogical.com/rex/docs/automate/server-api
- Official REX Lua API: https://www.superlogical.com/rex/docs/reference/lua
- Official REX events: https://www.superlogical.com/rex/docs/reference/events
- Official REX Lua events: https://www.superlogical.com/rex/docs/automate/lua-events
- Official REX Lua scripts: https://www.superlogical.com/rex/docs/automate/lua-scripts
- Official Program Status Protocol: https://www.superlogical.com/rex/docs/build/program-status
- GPUI project: https://github.com/Glass-HQ/gpui
- Zed terminal-view design reference: https://github.com/zed-industries/zed/tree/main/crates/terminal_view
- Android CLI: https://developer.android.com/tools/agents/android-cli
- Android CLI Journeys: https://developer.android.com/tools/agents/android-cli/journeys
- Android CLI specialist skill: https://developer.android.com/agents/skills/devtools/android-cli/skill

---

# 21. Copy/paste starter message for Codex

Use this message when starting Codex in the repository:

> Read `full.md` completely and treat it as the durable product specification and agent-loop instruction for Relayable. Begin executing §19 now. Inspect the current repository and toolchain before editing. Use supported Codex subagents for bounded independent tasks if available, with strict path ownership and one lead integrator; otherwise work sequentially. I have Rust and Android CLI installed, and may already have a JDK. Discover what's installed and use it; don't install or change global tools without asking. Do not require Android Studio. The Linux desktop uses GPUI. Most importantly, I want a high-fidelity recreation of the visible UI and workflows in the five linked reference videos—not a generic redesign. Inspect the videos, create timestamped evidence and a UI catalogue, and be honest if footage is inaccessible. Keep working in checkpointed iterations through the tasks, implementing and testing real vertical slices rather than stopping at a plan or mockup. Preserve my existing work. Update `docs/agent-loop/NEXT.md` and the task/work logs after every bounded iteration so another Codex run can resume from repository state alone. Ask only when an actual approval gate or external blocker requires me; otherwise continue with safe, testable work.
