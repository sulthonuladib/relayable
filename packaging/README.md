# `packaging/`

Installable artifacts for **Relayable** (full.md §7.1–§7.3). These files are the
sources installed by `scripts/install-user.sh`; they are not installed from here
directly.

| Path | Installed to | Notes |
|---|---|---|
| `systemd/user/relayable-server.service` | `${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/relayable-server.service` | `systemd --user` unit. `ExecStart=%h/.local/bin/relayable-server run`; `%h` assumes the default `~/.local/bin` install. |
| `desktop/relayable.desktop` | `${XDG_DATA_HOME:-$HOME/.local/share}/applications/relayable.desktop` | FreeDesktop entry; `Exec=relayable-desktop`. No `Icon=` — no icon asset exists yet. |

Scope and constraints:

- **User install only.** No system unit, no `/usr`, no root. A system-wide service
  is explicitly out of scope until a security review (full.md §7.2).
- **`relayable-server run` is the only implemented foreground mode**
  (`crates/relayable-server/src/main.rs`). Do not edit `ExecStart` to a command
  the binary does not accept.
- Binaries are **not** stored here; the installer builds them from the workspace
  and copies the results.

Install, run, and uninstall instructions live in
[`../docs/operations/user-install.md`](../docs/operations/user-install.md).
