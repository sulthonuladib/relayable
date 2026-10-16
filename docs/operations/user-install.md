# Relayable — user install and operations

How to install, run, and remove the **Relayable** Linux client tools and
per-user server service without root and without system-wide changes.

- Audience: a single Linux user installing Relayable on their own workstation.
- Scope: local Unix-socket server, `relayable` CLI, optional GPUI desktop app,
  `systemd --user` unit.
- Related: [`../../README.md`](../../README.md) (quickstart),
  [`../../packaging/README.md`](../../packaging/README.md),
  [`../../full.md`](../../full.md) §7.1–§7.3.

---

## 1. Support matrix

| Component | Source | Notes |
|---|---|---|
| `relayable-server` | `crates/relayable-server` | Foreground server. Only `relayable-server run` is implemented. |
| `relayable` (CLI) | `crates/relayable-cli` | Frozen command surface (see §8). |
| `relayable-desktop` | `apps/linux` (GPUI, separate workspace) | Installed from a binary named `relayable`; renamed on install to avoid the CLI collision. |
| User unit | `packaging/systemd/user/relayable-server.service` | `systemd --user` only. |
| Desktop entry | `packaging/desktop/relayable.desktop` | `Exec=relayable-desktop`, `Terminal=false`, no `Icon=`. |

No root. No `/usr`, `/etc`, `/run`, or system service writes. No network
listener.

## 2. Prerequisites

| Requirement | Check |
|---|---|
| Rust toolchain ≥ 1.98 | `rustc --version` |
| `protoc` (Protocol Buffers compiler) | `protoc --version` |
| `systemd` user manager (optional, for the service) | `systemctl --user is-system-running` |
| `install` (coreutils) | `command -v install` |

`protoc` is required by `prost-build` at build time (see
[`toolchain-audit.md`](toolchain-audit.md)). The build is offline
(`cargo build --offline`); the crates must already be in the local registry cache.

## 3. Install

```sh
scripts/install-user.sh                 # binaries + unit + desktop entry
scripts/install-user.sh --with-desktop  # also build/install the GPUI app
scripts/install-user.sh --skip-daemon-reload
```

Flags:

| Flag | Effect |
|---|---|
| `--with-desktop` | Build `apps/linux` in release (`cargo build --release --locked --offline`) and install its binary as `relayable-desktop`. Heavy; skipped by default. |
| `--skip-daemon-reload` (alias `--no-daemon-reload`) | Do not run `systemctl --user daemon-reload`. |
| `-h`, `--help` | Print usage. |

What the script does, in order:

1. Resolves `XDG_BIN_HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME` (see §5) and
   refuses non-absolute or `/` targets.
2. Builds the root binaries offline:
   `cargo build --release --offline --bin relayable --bin relayable-server`.
3. Installs `relayable` and `relayable-server` (mode 0755) into `XDG_BIN_HOME`.
4. Installs `relayable-server.service` (mode 0644) into
   `XDG_CONFIG_HOME/systemd/user/`.
5. Installs `relayable.desktop` (mode 0644) into `XDG_DATA_HOME/applications/`.
6. With `--with-desktop`, builds `apps/linux` and installs
   `apps/linux/target/release/relayable` as `relayable-desktop`.
7. Runs `systemctl --user daemon-reload` (unless skipped) — **never**
   `enable`/`start`.
8. Prints next steps.

The script is idempotent: re-running it replaces the installed files with
freshly built ones.

> Note: `apps/linux` is its own Cargo workspace with its own `Cargo.lock`. Its
> release build is independent of the root workspace target directory.

## 4. Where files land

Paths honour XDG overrides. With defaults:

| Artifact | Path | Mode |
|---|---|---|
| CLI | `~/.local/bin/relayable` | 0755 |
| Server | `~/.local/bin/relayable-server` | 0755 |
| Desktop app (optional) | `~/.local/bin/relayable-desktop` | 0755 |
| User unit | `~/.config/systemd/user/relayable-server.service` | 0644 |
| Desktop entry | `~/.local/share/applications/relayable.desktop` | 0644 |
| Runtime socket dir | `$XDG_RUNTIME_DIR/relayable/` | 0700 |
| Runtime socket | `$XDG_RUNTIME_DIR/relayable/relayable.sock` | 0600 |

Build output stays in `target/release/` (root workspace) and
`apps/linux/target/release/` (desktop workspace); neither is installed.

## 5. Environment variables

### 5.1 Installer / XDG overrides

| Variable | Default | Purpose |
|---|---|---|
| `HOME` | (required) | Base for the defaults below. |
| `XDG_BIN_HOME` | `$HOME/.local/bin` | Where binaries are installed. |
| `XDG_CONFIG_HOME` | `$HOME/.config` | Base for the systemd user unit. |
| `XDG_DATA_HOME` | `$HOME/.local/share` | Base for the desktop entry. |
| `CARGO` | `cargo` | Cargo executable the installer invokes. |

The shipped unit hard-codes `ExecStart=%h/.local/bin/relayable-server run`.
`%h` is the systemd specifier for the user's home directory. If you installed
with `XDG_BIN_HOME` pointing elsewhere, edit the unit's `ExecStart` to an
absolute path before enabling it.

### 5.2 Server (`relayable-server run`)

| Variable | Default | Meaning |
|---|---|---|
| `RELAYABLE_SERVER` | derived socket path | Socket path override. |
| `RELAYABLE_EVENT_RETENTION_EVENTS` | `4096` | Retained control events per session. Must be a positive integer. |
| `RELAYABLE_IDLE_TIMEOUT_MILLIS` | `30000` | Connection idle timeout (ms). Positive integer. |
| `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS` | `5000` | Heartbeat interval (ms). Positive and strictly less than the idle timeout. |
| `SHELL` | `/bin/sh` | Shell spawned for new sessions. |

Invalid/empty/zero values are startup errors (non-zero exit). The default
socket path is resolved in this order:

1. `$XDG_RUNTIME_DIR/relayable/relayable.sock` when `XDG_RUNTIME_DIR` is a
   non-empty existing directory;
2. `/run/user/<uid>/relayable/relayable.sock` when that root exists;
3. `$TMPDIR/relayable-<uid>/relayable.sock`, falling back to
   `/tmp/relayable-<uid>/relayable.sock`.

### 5.3 CLI (`relayable`)

| Variable | Default | Meaning |
|---|---|---|
| `RELAYABLE_SERVER` | derived socket path | Socket to connect to. |
| `RELAYABLE_SESSION` | (none) | Default session target for `-s/--session`. |

## 6. Running

### 6.1 Foreground (try it out)

```sh
relayable-server run
# relayable-server: listening on /run/user/1000/relayable/relayable.sock (protocol 1.0)
```

Stop with `Ctrl-C`. On `SIGTERM`/`SIGINT` the server exits 0 and unlinks its
socket.

### 6.2 As a `systemd --user` service

```sh
systemctl --user daemon-reload
systemctl --user enable --now relayable-server
systemctl --user status relayable-server
journalctl --user -u relayable-server -f          # follow logs
systemctl --user restart relayable-server
systemctl --user stop relayable-server
```

The unit is `WantedBy=default.target`, `Restart=on-failure`, `RestartSec=2s`,
`TimeoutStopSec=15s`, and has **no** `User=` key (it always runs as you).

To keep the service running after logout, opt in to lingering:

```sh
loginctl enable-linger "$USER"      # changes login behaviour; opt-in only
```

## 7. Desktop app

Build/run from source (`apps/linux` is its own workspace):

```sh
cd apps/linux
cargo run --bin relayable -- --mock     # offline mock UI
cargo run --bin relayable               # connect to a running server
```

Installed via `scripts/install-user.sh --with-desktop`, it appears in the
application menu as **Relayable**, or can be launched with `relayable-desktop`.
There is no `Icon=` in the desktop entry because no icon asset exists in the
repository yet.

The GPUI release build is heavy and can be slow; see
[`../../apps/linux/README.md`](../../apps/linux/README.md) §12 for the pinned
`gpui`/`libc` build caveat.

## 8. CLI reference

```
relayable --help | -h
relayable --version | -V
relayable version [--json]
relayable ls [--json]
relayable new [--json]
relayable session inspect [-s|--session <target>] [--json]
relayable session close   [-s|--session <target>] [--json]
```

`-s/--session <target>` accepts a full session ID, a case-insensitive label, a
1-based position, or an ID suffix. `--json` produces stable machine-readable
output.

## 9. Security

- Socket directory mode **0700**, socket file mode **0600** — only the owning
  user can connect.
- No network transport and no listener beyond the local Unix socket.
- Sessions execute with the invoking user's normal permissions; the server does
  not elevate.
- Installing or enabling the service is always an explicit user action; the
  installer never enables, starts, or stops services.

## 10. Uninstall

Stop and disable any service first, then remove the installed files:

```sh
# 1. Stop/disable the service (ignore "not loaded" if you never enabled it)
systemctl --user disable --now relayable-server 2>/dev/null || true
rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/relayable-server.service"
systemctl --user daemon-reload

# 2. Remove binaries and the desktop entry
rm -f "${XDG_BIN_HOME:-$HOME/.local/bin}/relayable" \
      "${XDG_BIN_HOME:-$HOME/.local/bin}/relayable-server" \
      "${XDG_BIN_HOME:-$HOME/.local/bin}/relayable-desktop" \
      "${XDG_DATA_HOME:-$HOME/.local/share}/applications/relayable.desktop"

# 3. Optionally remove the runtime socket directory
rm -rf "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/relayable"
```

Build output can be removed with `rm -rf target apps/linux/target`. If you
enabled lingering and no longer want it, `loginctl disable-linger "$USER"`.

## 11. Troubleshooting

| Symptom | Likely cause / fix |
|---|---|
| `cargo: command not found` | Install Rust via rustup and ensure `~/.cargo/bin` is on `PATH`. |
| Build fails in `prost-build` | `protoc` missing from `PATH`; install it and re-run. |
| `cargo build --offline` cannot resolve crates | The registry cache is incomplete. Build once online, or pre-populate the cache. |
| `relayable: command not found` after install | `~/.local/bin` is not on `PATH`; `export PATH="$HOME/.local/bin:$PATH"`. |
| `relayable ls` prints nothing | No sessions exist yet; that is normal. Create one with `relayable new`. |
| CLI reports a connection error | No server is running on the derived socket. Start `relayable-server run` or the user service, or set `RELAYABLE_SERVER` to the right path. |
| `server: invalid configuration` | A numeric env var is empty/zero/non-numeric, or heartbeat ≥ idle timeout. Fix the variable or unset it. |
| `Address already in use` on start | A live server already owns the socket. Stop the old server/service. |
| Service exits immediately | Check `journalctl --user -u relayable-server -n 50`. Verify the unit's `ExecStart` path matches where you installed the binary (relevant if `XDG_BIN_HOME` was overridden). |
| `systemctl --user daemon-reload` fails | Run it from a logged-in session with a user manager; it is harmless to skip with `--skip-daemon-reload`. |
| Desktop app fails to build | See `apps/linux/README.md` §12 (pinned `gpui`/`libc` build caveat). |
