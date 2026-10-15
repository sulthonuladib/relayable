# TERM-002 gpui =0.2.2 compile-blocker diagnosis (follow-up)

Scope: read-only w.r.t. source. No version pin changed, no `[patch]`, no lockfile edit.
All evidence under `results/raw/gpui-followup/`; every file is a verbatim command capture with
provenance + real exit code. Reproduce with `scripts/probe_gpui_followup.sh static|matrix|cold`.

## 1. Exact dependency chain to `xattr`

`cargo tree -i xattr` (exit 0) — `tree_invert_xattr.txt`:

```
xattr v0.2.3
└── zed-async-tar v0.5.0-zed
    └── gpui_http_client v0.2.2
        └── gpui v0.2.2
            └── relayable-gpui-pin-probe v0.0.0
```

Recorded chain CONFIRMED. `gpui v0.2.2 -> gpui_http_client v0.2.2 -> zed-async-tar v0.5.0-zed -> xattr v0.2.3`.

## 2. Why it fails (root cause)

`registry_xattr_sys_mod.txt` — `xattr-0.2.3/src/sys/mod.rs`:

```
 1  macro_rules! platforms {
...
11              #[cfg(any($(target_os = $platform),*))]
12              pub const ENOATTR: ::libc::c_int = ::libc::ENOATTR;   <-- line 12, unconditional for linux
...
37  platforms! {
38      "android"; "linux"; "macos" => linux_macos,
```

`xattr-0.2.3/src/lib.rs` (`registry_xattr_lib_rs.txt`) has `mod sys;` with no cfg/feature gate;
the crate's only feature is `unsupported`, which does NOT gate the `platforms!` ENOATTR const.
So on `x86_64-unknown-linux-gnu` the const is compiled unconditionally.

`registry_libc_enoattr_matrix.txt`:
- libc 0.2.147 and 0.2.189 define, under `src/unix/linux_like/linux/mod.rs`, a
  `#[deprecated(since = "0.2.55", note = "ENOATTR is not available on Linux; use ENODATA instead")]`
  `pub const ENOATTR: c_int = ENODATA;`.
- libc 0.2.190 (the lock's resolved version) has NO `ENOATTR` anywhere under
  `linux_like/linux` or `linux_like/android`.

`registry_libc_0_2_190_changelog.txt` (libc 0.2.190, released 2026-10-02):
```
### Removed
- **breaking** Remove most items that have been deprecated since before 0.2.100 ([#5401]...)
```

Verdict: **libc-version drift on top of a latent xattr defect.** xattr 0.2.3 references a libc
constant that was deprecated since libc 0.2.55; libc 0.2.190 finally removed it. The reference is
unconditional for Linux, so there is no cfg mistake to fix here — it is a genuine
`xattr 0.2.3 + libc >= 0.2.190` incompatibility. xattr 0.2.3's manifest (`libc = "0.2"`, no upper
bound) permits the newer libc, so Cargo selects 0.2.190 and the build breaks.
xattr 1.x removed the const entirely (`registry_xattr_1_6_1_sys_mod.txt`), but is not reachable
from `zed-async-tar 0.5.0-zed`.

## 3. Other libc versions / newer xattr

- Lockfile has exactly **one** libc: `0.2.190` (`lock_entries.txt`; `grep -c '^name = "libc"' == 1`).
- `cargo tree -d` (`tree_duplicates.txt`) shows no duplicate libc.
- `cargo update --dry-run --verbose -p xattr` -> `Locking 0 packages to latest compatible versions`
  (`update_dryrun_xattr.txt`): 0.2.3 is already the latest `^0.2`.
- Local index `index_xattr_versions.txt`: `0.2.0, 0.2.1, 0.2.2, 0.2.3` then jumps to `1.0.x..1.6.1`.
  **No `0.2.4`; no newer compatible xattr exists.**
- `zed-async-tar 0.5.0-zed/Cargo.toml.orig` line 34: `xattr = { version = "0.2", optional = true }`,
  `default = [ "xattr" ]` (`registry_zed_async_tar_manifest.txt`). Requirement `>=0.2.0,<0.3.0`.
- crates.io latest xattr is `1.6.1` (`search_xattr.txt`); crates.io latest gpui is `0.2.2`
  (`search_gpui.txt`) — the pinned version is current, there is no newer gpui to move to.

## 4. Bounded retry matrix (`matrix_summary.txt`; each `timeout 900`, real exit codes)

| attempt | feature selection | exit | wall | last error |
|---|---|---|---|---|
| isolated_xattr | `-p xattr --locked` | 101 | 0.3s | could not compile `xattr` |
| isolated_xattr_no_default_features | `-p xattr --no-default-features` | 101 | 0.0s | cargo: cannot specify features for packages outside of workspace (inconclusive) |
| gpui_http_client | `-p gpui_http_client --locked` | 101 | 0.5s | could not compile `xattr` |
| no_default_features | root pkg `--no-default-features --locked` | 101 | 2.0s | could not compile `xattr` |
| no_default_features_gpui | `gpui` default-features=false | 101 | 7.5s | could not compile `xattr` |
| x11_only | `gpui` features=["x11"] | 101 | 6.3s | could not compile `xattr` |
| wayland_only | `gpui` features=["wayland"] | 101 | 4.8s | could not compile `xattr` |

Every variant reproduces `error[E0425]: cannot find value ENOATTR in crate ::libc` at
`xattr-0.2.3/src/sys/mod.rs:12:56`. `matrix_x11_only.txt`, `matrix_wayland_only.txt`.
Step (e) skipped — no feature avoided the xattr path (see `matrix_summary.txt`).
`isolated_xattr_no_default_features` is a cargo CLI rejection, not a real check; xattr's only
feature (`unsupported`) does not gate the failing const anyway.

True cold full check (`matrix_cold_full_check.txt`, fresh empty target dir): exit 101,
**20.8s wall**, **392 crates** attempted (146 `Compiling` + 246 `Checking`), 20 CPUs.
This matches the earlier `results/raw/deps/gpui_cargo_check.txt` (exit 101 at line 406).

## 5. Did the Linux backend get reached?

No — the build aborts at xattr first. In the cold run, `xkbcommon`, `blade-graphics`,
`blade-macros`, `blade-util`, `cosmic-text`, `font-kit`, `x11rb`, and even
`zed-async-tar`/`gpui_http_client` never appeared. Only parallel leaf deps
(`ash`, `x11rb-protocol`, `wayland-client`, `wayland-backend`) were compiled/checked before
the abort. Therefore **"gpui builds on this host" remains UNPROVEN**; UI-001 window/display
proof cannot start until this is resolved.

## 6. Recommendation for UI-001 (no remedy applied)

1. Treat this as a **hard blocker** for `gpui = "=0.2.2"` on Linux as of libc 0.2.190; the pin
   resolves (fetch exit 0) but `cargo check` deterministically fails (exit 101).
2. Cheapest evidence-backed remedy: hold libc below 0.2.190, e.g.
   `cargo update -p libc --precise 0.2.189` (0.2.189 defines Linux `ENOATTR`;
   `registry_libc_enoattr_matrix.txt`). Blast radius is global to the workspace; Manager/ADR decision.
3. Alternative: `[patch.crates-io]` `xattr` to a 0.2.3 fork that drops the `ENOATTR` const
   (mirrors the real xattr 1.x fix in `registry_xattr_1_6_1_sys_mod.txt`). Localized but vendored.
4. Do NOT expect feature selection to help: gpui always depends on gpui_http_client
   (`gpui-0.2.2/Cargo.toml.orig:99`) and gpui_http_client always depends on zed-async-tar with
   default feature `xattr` (`gpui_http_client-0.2.2/Cargo.toml:49-51`); no gpui feature disables it.
5. The gpui pin is not "wrong"; it is newly broken by a libc release after the pin. There is no
   newer crates.io gpui (latest is 0.2.2), so moving the pin is not currently an option.
