# Relayable

**Relayable** (by Superlogical) is a terminal session manager: a local server owns
PTY-backed sessions that survive client disconnects, and clients (a CLI and a Linux
desktop app) connect over a user-owned Unix-domain socket.

This repository is at **Phase 1–2**: the local Linux path is implemented and tested —
`relayable-server` (foreground server), `relayable` (CLI), and a GPUI desktop app
under `apps/linux`. Remote transport and the Android client are not implemented yet
(Android is deferred per DEC-010).

> New to the project? This quickstart covers the local user install. The detailed
> reference (paths, environment variables, systemd commands, uninstall,
> troubleshooting) is [`docs/operations/user-install.md`](docs/operations/user-install.md).

## Prerequisites

- **Linux** with a user `systemd` manager (for the optional background service).
- **Rust toolchain** 1.98 or newer (`rustc`/`cargo`; the repo pins
  `rust-version = "1.98"`). Install via <https://rustup.rs>:
  ```sh
  rustc --version && cargo --version
  ```
- **`protoc`** (Protocol Buffers compiler) on `PATH` — required by `prost-build` to
  compile the protocol crate.
  ```sh
  protoc --version    # e.g. libprotoc 36.1
  ```
- No root access and no system-wide packages are required.

## Build

```sh
cargo build --release --offline --bin relayable --bin relayable-server
```

Binaries land in `target/release/relayable` (CLI) and
`target/release/relayable-server` (server).

## Install (per user, no root)

```sh
scripts/install-user.sh
```

This builds the two binaries offline and installs them, plus a `systemd --user`
unit and a desktop entry, entirely under your XDG user directories. It never uses
`sudo`, never enables or starts the service, and only runs
`systemctl --user daemon-reload` (unless you pass `--skip-daemon-reload`).

To also build and install the GPUI desktop app as `relayable-desktop`:

```sh
scripts/install-user.sh --with-desktop
```

The desktop build is a heavy release build (the `gpui` dependency tree); it is
skipped by default.

### Where files land

| Artifact | Path |
|---|---|
| `relayable` (CLI) | `${XDG_BIN_HOME:-$HOME/.local/bin}/relayable` |
| `relayable-server` | `${XDG_BIN_HOME:-$HOME/.local/bin}/relayable-server` |
| `relayable-desktop` (GPUI, `--with-desktop`) | `${XDG_BIN_HOME:-$HOME/.local/bin}/relayable-desktop` |
| `systemd --user` unit | `${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/relayable-server.service` |
| Desktop entry | `${XDG_DATA_HOME:-$HOME/.local/share}/applications/relayable.desktop` |
| Default Unix socket | `$XDG_RUNTIME_DIR/relayable/relayable.sock` |

If `~/.local/bin` is not on your `PATH`, add it:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

## Run the server (foreground)

The only implemented foreground mode is `relayable-server run`:

```sh
relayable-server run
# relayable-server: listening on /run/user/1000/relayable/relayable.sock (protocol 1.0)
```

It runs in the foreground until `Ctrl-C`/`SIGTERM`, then exits cleanly and removes
its socket. Configuration is via environment variables (see the reference doc);
the socket path can be overridden with `RELAYABLE_SERVER=/path/to.sock`.

## Use the CLI

The CLI (`relayable`) talks to the server over the same socket. In another
terminal:

```sh
relayable ls                 # list sessions (empty output when there are none)
relayable new                # create a session
# session:01a142bd-... 
# window:01a142bd-...
# block:01a142bd-...
# pty:01a142bd-...
# pid:507405

relayable session inspect -s 01a142bd-...     # inspect one session
relayable session close   -s 01a142bd-...     # close it

relayable --help             # frozen command surface
```

Session targets accepted by `-s/--session` are the full ID, a case-insensitive
label, a 1-based position, or an ID suffix. The default target can be set with
`RELAYABLE_SESSION`. Add `--json` to any of `ls`, `new`, `session inspect`, and
`session close` for machine-readable output.

## Run the desktop app

Build and launch the GPUI app (from `apps/linux`, its own Cargo workspace):

```sh
cd apps/linux
cargo run --bin relayable -- --mock      # offline UI with mock data
cargo run --bin relayable                 # connect to a running server
```

After `scripts/install-user.sh --with-desktop`, launch it from your application
menu (**Relayable**) or run `relayable-desktop`. See
[`apps/linux/README.md`](apps/linux/README.md) for the app's fidelity notes and
build caveats.

## Install and enable the user service

The installer places the unit but deliberately does not touch service state:

```sh
systemctl --user daemon-reload
systemctl --user enable --now relayable-server
systemctl --user status relayable-server
journalctl --user -u relayable-server -f
```

The unit runs `relayable-server run` as your user (`WantedBy=default.target`,
`Restart=on-failure`, `TimeoutStopSec=15s`, no `User=`). To persist across logout,
opt in to lingering with `loginctl enable-linger "$USER"` (this changes your login
behaviour — do it only if you want the server to keep running after logout).

## Security notes

- The server creates its socket directory with mode **0700** and the socket file
  with mode **0600**, so only your user can connect. There is no network listener
  and no authentication layer beyond filesystem ownership.
- Sessions run with **your** user's permissions; the server never elevates.
- Installing/enabling the systemd unit is a user action; this repo's scripts never
  enable, start, or stop services for you.

## Documentation

- [`docs/operations/user-install.md`](docs/operations/user-install.md) — install /
  uninstall / env-var / systemd reference and troubleshooting.
- [`packaging/README.md`](packaging/README.md) — what `packaging/` ships.
- [`apps/linux/README.md`](apps/linux/README.md) — the desktop app.
- [`full.md`](full.md) — the product and architecture specification.
