# TERM-002 remedy — `gpui = "=0.2.2"` + `libc` on this host (UI-001, phase 1)

Scope of this file: reproduce, remedy, and prove the TERM-002 compile blocker
for the ADR-003 GPUI pin, **inside the `apps/linux` workspace only**. Every
command below was actually run from `apps/linux` on this host; raw captures are
the sibling files named inline.

- Host: Linux `falcon`, x86_64-unknown-linux-gnu
- Toolchain: `cargo 1.98.1 (797e8a9bc 2026-08-05)`, `rustc 1.98.1 (48a229cea 2026-09-01)`
- Pin under test: `gpui = "=0.2.2"` (ADR-003 Option H)
- Date (UTC): 2026-10-15

## 1. Reproduce — `cargo check` fails

Command:

```
cargo check
```

Exit code: **101** (`EXIT_CODE=101`; raw output: `repro-cargo-check.log`).

Resolved transitive versions at failure (from the freshly generated
`Cargo.lock`): `libc 0.2.190`, `xattr 0.2.3`.

Verbatim error (head of the failure; full block in `repro-cargo-check.log`):

```
error[E0425]: cannot find value `ENOATTR` in crate `::libc`
   --> /home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/xattr-0.2.3/src/sys/mod.rs:12:56
    |
 12 |               pub const ENOATTR: ::libc::c_int = ::libc::ENOATTR;
    |                                                          ^^^^^^^
...
 37 | / platforms! {
 38 | |     "android"; "linux"; "macos" => linux_macos,
 39 | |     "freebsd"; "netbsd" => bsd
 40 | | }
    | |_- in this macro invocation
    |
   ::: /home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.190/src/unix/linux_like/linux/gnu/b64/x86_64/mod.rs:424:1
    |
424 |   pub const ENOSTR: c_int = 60;
    |   ----------------------- similarly named constant `ENOSTR` defined here
    |
    = note: this error originates in the macro `platforms` (in Nightly builds, run with -Z macro-backtrace for more info)
help: a constant with a similar name exists
    |
 12 -             pub const ENOATTR: ::libc::c_int = ::libc::ENOATTR;
 12 +             pub const ENOATTR: ::libc::c_int = ::libc::ENOSTR;
    |

error: could not compile `xattr` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

**The blocker reproduced exactly as documented by TERM-002** (`xattr 0.2.3` ->
`libc 0.2.190` removed `libc::ENOATTR`), with the same `E0425` at
`xattr-0.2.3/src/sys/mod.rs:12:56`.

## 2. Remedy — hold `libc` below 0.2.190 (ADR-003 preferred remedy)

Command (run from `apps/linux`):

```
cargo update -p libc --precise 0.2.189
```

Exit code: **0**. Verbatim output (`remedy-cargo-update.log`):

```
    Updating crates.io index
 Downgrading libc v0.2.190 -> v0.2.189
note: pass `--verbose` to see 5 unchanged dependencies behind latest
```

No `[patch.crates-io]` fallback was needed: `libc 0.2.189` is a valid
`^0.2` resolution for every dependency in this workspace's lockfile (single
`libc` entry, no duplicate). `apps/linux/Cargo.toml` is the only manifest of
this pin, and `apps/linux` is its own workspace (`[workspace]`), so the change
cannot reach the repository-root lockfile or `crates/**`.

## 3. Prove — `cargo check` passes

Command:

```
cargo check
```

Exit code: **0** (`fixed-cargo-check.log`, tail):

```
    Checking oo7 v0.5.0
    Checking relayable-linux v0.0.0 (/home/sulthonuladib/.herdr/worktrees/relayable/ui-ui-001-gpui-window/apps/linux)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.80s
```

## 4. Resulting lockfile versions (`apps/linux/Cargo.lock`)

Exactly one `libc` entry and one `xattr` entry (`grep -c` == 1 each):

```
name = "libc"
version = "0.2.189"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2"

name = "xattr"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6d1526bbe5aaeb5eb06885f4d987bcdfa5e23187055de9b83fe00156a821fabc"
dependencies = [
 "libc",
]
```

`apps/linux/Cargo.lock` **is** committed — it is the remedy artifact that keeps
`libc` at `0.2.189` for this workspace.

## 5. Residual risk

The remedy is a lockfile hold, not a source fix: `xattr 0.2.3` still references
the removed `libc::ENOATTR`, so any future `cargo update` in this workspace that
loosens `libc` past `0.2.189` reintroduces exit 101. Committing the lockfile plus
the exact pin is the containment. Moving to `xattr 1.x` or a `gpui` newer than
`0.2.2` is out of scope for UI-001.
