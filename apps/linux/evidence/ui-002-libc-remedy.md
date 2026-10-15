# UI-002 stage A — `libc` / `xattr` resolution (contained remedy)

Scope: the `apps/linux` **separate workspace only**. Nothing outside
`apps/linux` is touched, and the core crates are unaffected (root
`cargo check --workspace --locked` exits 0; see `ui-002-stage-a.md`).

Host: Linux `falcon`, x86_64-unknown-linux-gnu. Toolchain: `cargo 1.98.1`.
Date (UTC): 2026-10-16.

## What was added (why this file exists)

UI-002 stage A adds these dependencies to `apps/linux/Cargo.toml`:

- path deps to `../../crates/{relayable-client,relayable-model,relayable-protocol}`
  (`relayable-protocol` with feature `transport-unix`);
- `alacritty_terminal = "=0.26.0"` and `vte = "=0.15.0"` (ADR-003 pins).

ADR-003 Consequences warn that if `apps/linux` ever path-depends on a core
crate that pins `libc = "=0.2.190"` (i.e. `relayable-server`), the resolved
`libc` is forced back up and `xattr 0.2.3` fails with
`cannot find value ENOATTR`.

## Inspection before resolving

- The only crate in the repository that pins `libc = "=0.2.190"` is
  `crates/relayable-server` (it is **not** a dependency of
  `relayable-model`, `relayable-protocol`, or `relayable-client`).
- No core crate mentions `libc` or `xattr` in its own manifest.
- The reachable `libc` requirements are caret (`^0.2`):
  `uuid → getrandom` (already present for `relayable-model`) and, newly,
  `alacritty_terminal`'s published `libc = "0.2"`.

## Outcome after `cargo check` (unlocked) in `apps/linux`

`cargo check` was run **without** `--locked` to force a re-resolve of the
lockfile with the new dependencies. It exited 0 and the lockfile **still
resolved**:

```
[[package]]
name = "libc"
version = "0.2.189"
checksum = "3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2"

[[package]]
name = "xattr"
version = "0.2.3"
checksum = "6d1526bbe5aaeb5eb06885f4d987bcdfa5e23187055de9b83fe00156a821fabc"
```

`0.2.189` satisfies every caret requirement, so no bump occurred. The
`ENOATTR` failure did **not** return: with `libc 0.2.189` the
`[patch.crates-io]` vendoring of `xattr` was **not needed**.

## Remedy used

The remedy in effect is the UI-001 lockfile hold, applied with:

```
cargo update -p libc --precise 0.2.189
```

(from `apps/linux`). It survives the new dependencies because none of them
requires `libc` above `0.2.189`. No `[patch.crates-io]` table and no
`apps/linux/vendor/xattr` directory were added.

Evidence that the hold is live: `apps/linux/Cargo.lock` contains exactly one
`libc` package at `0.2.189` and one `xattr` at `0.2.3`, and
`cargo check --locked` / `cargo clippy --all-targets --locked` /
`cargo test --locked` all exit 0 (see `ui-002-stage-a.md`).

## Diagnostics actually run (exit codes)

| Command (from `apps/linux`) | Exit |
| --- | --- |
| `cargo check` (unlocked, resolves the lock) | 0 |
| `cargo check --locked` | 0 |
| `cargo build --bin live-probe --locked` | 0 |
| `cargo test --locked` | 0 (35 tests) |
| `cargo clippy --all-targets --locked` | 0 |
| `cargo fmt --check` | 0 |

Raw check output for the resolving run: the re-resolve added 183 lockfile
lines and did not change the `libc`/`xattr` versions (see
`ui-002-live-probe.log` for the live proof and `ui-002-stage-a.md` for the
summary).
