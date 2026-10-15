# ADR-002 local evidence — unverified items #1, #2, #9

Scope of this file (only these three items; items #3–#8 and #10 are owned by another agent):

| ADR item | Question |
|---|---|
| #1 | Whether `prost` retains **unknown fields** (not just unknown enum values) |
| #2 | Whether `rustls` is "pure Rust"/avoids OpenSSL, and its default crypto provider (`ring` vs `aws-lc-rs`) |
| #9 | Presence and version of `protoc` on the build host |

Evidence produced by live, reproducible local runs on this host. No file in the repository was
modified except this evidence file. All scratch work is under `/tmp`.

- Repo worktree: `/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`
- ADR reviewed: `docs/adr/ADR-002-protocol-schema-and-transport.md`
  `sha256 = 09bb21d3537924f1f84ed1e1c0a9736221c6b03a389e4cfdfab7f42587369469`
- Repo Git state before this file was written: `?? docs/reviews/` (no tracked file changed).

---

## Environment baseline

### Commands (exact)

```bash
date -u
uname -a
which protoc
protoc --version
cargo --version
rustc --version
```

### Exit codes

All six commands exited `0` (`EXIT:0`).

### Raw output

```
Thu Oct 15 06:23:08 PM UTC 2026
Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux
/usr/bin/protoc
libprotoc 36.1
cargo 1.98.1 (797e8a9bc 2026-08-05)
rustc 1.98.1 (48a229cea 2026-09-01)
```

Supplementary host checks:

```bash
ls -l /usr/bin/protoc
find "$HOME/.cargo" /usr/local /opt -maxdepth 4 -iname 'protoc*' -type f 2>/dev/null
ls /usr/bin/protoc*
env | grep -i -E 'PROTOC|CARGO_NET|http_proxy|https_proxy'
timeout 20 curl -sS -o /dev/null -w '%{http_code}\n' https://index.crates.io/config.json
```

Raw output / exit codes:

```
lrwxrwxrwx 1 root root 13 Sep  5 04:11 /usr/bin/protoc -> protoc-36.1.0
/usr/bin/protoc
/usr/bin/protoc-36.1.0
/usr/bin/protoc-c
/usr/bin/protoc-gen-c
/usr/bin/protoc-gen-upb
/usr/bin/protoc-gen-upb-36.1.0
/usr/bin/protoc-gen-upbdefs
/usr/bin/protoc-gen-upbdefs-36.1.0
/usr/bin/protoc-gen-upb_minitable
/usr/bin/protoc-gen-upb_minitable-36.1.0
```
- `find` over `~/.cargo`, `/usr/local`, `/opt` returned **no** `protoc*` files (no vendored/`protoc-bin` copy there); exit 0 with empty output.
- `env | grep -i -E 'PROTOC|...'` printed nothing, grep exit `1` (no `PROTOC`, `PROTOC_INCLUDE`, or proxy variables set).
- Network reachable: `https://index.crates.io/config.json` → HTTP `200`, curl exit `0`.

### Verdict

Baseline established. rustc/cargo 1.98.1 as recorded in `AGENTS.md`; `protoc` **is** present
(symlink to a real binary), and crates.io is reachable.

---

## Item #9 — `protoc` presence and version

### Commands (exact)

```bash
which protoc; echo "EXIT:$?"
protoc --version; echo "EXIT:$?"
ls -l /usr/bin/protoc
find "$HOME/.cargo" /usr/local /opt -maxdepth 4 -iname 'protoc*' -type f 2>/dev/null
# plus the live prost-build run in Item #1, which invokes protoc implicitly
```

### Exit codes

`which protoc` → `0`; `protoc --version` → `0`; `find` → `0` (empty).

### Raw output

```
/usr/bin/protoc
EXIT:0
libprotoc 36.1
EXIT:0
lrwxrwxrwx 1 root root 13 Sep  5 04:11 /usr/bin/protoc -> protoc-36.1.0
```

Can prost-build use it without extra config? Proven yes: the Item #1 crate declares only
`prost-build = "=0.14.4"` with **no** `PROTOC`/`PROTOC_INCLUDE` env vars and **no**
`protoc_executable(...)` call, and `cargo build` succeeded (see Item #1). `prost-build` 0.14.4
resolves `protoc` as follows (source, version-pinned):

`~/.cargo/registry/src/index.crates.io-.../prost-build-0.14.4/src/config.rs:1272`:

```rust
/// Returns the path to the `protoc` binary.
pub fn protoc_from_env() -> PathBuf {
    env::var_os("PROTOC")
        .map(PathBuf::from)
        .unwrap_or(PathBuf::from("protoc"))
}
```

`.../prost-build-0.14.4/src/lib.rs:104–113`:

```
//! `prost-build` depends on the Protocol Buffers compiler, `protoc`, to parse `.proto` files into
//! a representation that can be transformed into Rust.
//!
//! The easiest way for `prost-build` to find `protoc` is to install it in your `PATH`.
//! ... `prost-build` will search
//! the current path for `protoc` or `protoc.exe`.
```

`.../prost-build-0.14.4/src/config.rs:938` runs `Command::new(&self.protoc_executable)`; the
default is `protoc_from_env()`, i.e. bare `protoc` resolved via `PATH`. There is no bundled
`protoc` in `prost-build`; the documented alternative is the `protobuf-src` crate building
`protoc` from source (`prost-build-0.14.4/src/lib.rs:126–131`), or `skip_protoc_run()` with a
pre-generated `FileDescriptorSet` (`config.rs:598`, `config.rs:927–937`). No vendored
`protoc`/`protoc-bin` tree exists on this host; `/usr/bin/protoc-36.1.0` is the system copy.

### Verdict

**SETTLED.** `protoc` exists at `/usr/bin/protoc` (real binary `/usr/bin/protoc-36.1.0`),
version **`libprotoc 36.1`** (Protobuf 36.1). `prost-build` 0.14.4 uses it with **no extra
configuration** beyond having it on `PATH`; no vendored/`protoc-bin` fallback is present.
This resolves ADR-002 unverified item #9 and ADR-002 line 408 ("Whether it is present on this
host is unverified") — it is present.

---

## Item #1 — does `prost` retain unknown fields?

### Setup

Scratch crate `/tmp/adr002-unknown` (prost/prost-build pinned to the ADR's 0.14.4 with `=`):

`Cargo.toml`:

```toml
[package]
name = "adr002-unknown"
version = "0.1.0"
edition = "2021"

[dependencies]
prost = "=0.14.4"

[build-dependencies]
prost-build = "=0.14.4"
```

`proto/v1.proto`:

```proto
syntax = "proto3";
package experiment.v1;

message Msg {
  uint32 id = 1;
  string name = 2;
}
```

`proto/v2.proto` (adds field 3; fields 1–2 keep the same numbers):

```proto
syntax = "proto3";
package experiment.v2;

message Msg {
  uint32 id = 1;
  string name = 2;
  string extra = 3;
}
```

`build.rs`:

```rust
fn main() {
    prost_build::compile_protos(&["proto/v1.proto", "proto/v2.proto"], &["proto"])
        .expect("prost-build compile failed");
}
```

`src/main.rs` encodes a `v2::Msg` with field 3 set, decodes with `v1::Msg::decode`, re-encodes
the `v1::Msg`, then decodes that with `v2::Msg` and checks whether field 3 survived.

### Commands (exact)

```bash
cd /tmp/adr002-unknown
cargo generate-lockfile
cargo build
cargo run --quiet
```

### Exit codes

`cargo generate-lockfile` → `0`; `cargo build` → `0` (`BUILD_EXIT:0`); `cargo run --quiet` → `0`
(`RUN_EXIT:0`).

### Raw output

`cargo generate-lockfile`:

```
    Updating crates.io index
     Locking 38 packages to latest compatible versions
```

Resolved versions (from `Cargo.lock`):

```
name = "prost"
version = "0.14.4"
name = "prost-build"
version = "0.14.4"
name = "prost-derive"
version = "0.14.4"
name = "prost-types"
version = "0.14.4"
```

`cargo build` tail:

```
   Compiling prost-derive v0.14.4
   Compiling prost v0.14.4
   Compiling prost-types v0.14.4
   Compiling prost-build v0.14.4
   Compiling adr002-unknown v0.1.0 (/tmp/adr002-unknown)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.11s
BUILD_EXIT:0
```

`cargo run --quiet` (the decisive experiment):

```
v2 wire bytes (24): 08 07 12 05 68 65 6c 6c 6f 1a 0d 45 58 54 52 41 5f 50 41 59 4c 4f 41 44
v1 decoded: id=7 name="hello"
v1 re-encoded bytes (9): 08 07 12 05 68 65 6c 6c 6f
v2 decode of re-encoded: id=7 name="hello" extra=""
v2 decode of original: id=7 name="hello" extra="EXTRA_PAYLOAD"
RESULT: DROPPED - unknown field 3 did NOT survive v1 decode + v1 re-encode
RUN_EXIT:0
```

The v2 wire form is 24 bytes including field 3 (`1a 0d 45 58 54 52 41 5f 50 41 59 4c 4f 41 44` =
tag `0x1a`, len `0x0d`, `"EXTRA_PAYLOAD"`). After decoding as v1 and re-encoding, the output is
9 bytes with field 3 gone; decoding that with v2 yields `extra=""`.

### prost version dependence

Tested the previous minor line `prost 0.13.5` in `/tmp/adr002-unknown-013` (same protos and
program). `cargo run --quiet` exit `0`, resolved `prost`/`prost-build` `0.13.5`, raw output:

```
v2 wire bytes (24): 08 07 12 05 68 65 6c 6c 6f 1a 0d 45 58 54 52 41 5f 50 41 59 4c 4f 41 44
v1 decoded: id=7 name="hello"
v1 re-encoded bytes (9): 08 07 12 05 68 65 6c 6c 6f
v2 decode of re-encoded: id=7 name="hello" extra=""
v2 decode of original: id=7 name="hello" extra="EXTRA_PAYLOAD"
RESULT: DROPPED - unknown field 3 did NOT survive v1 decode + v1 re-encode
```

`0.14.4` is the newest published `prost` (crates.io sparse index `.../pr/os/prost`, last entries:
`0.13.5`, `0.14.0` yanked, `0.14.1`, `0.14.2` yanked, `0.14.3`, `0.14.4`), so the result is not
a transient of an old release: **both 0.13.x and 0.14.x drop unknown fields**, and there is no
newer release that could behave differently.

### Source evidence (version-pinned, `prost 0.14.4`)

The generated `Message::merge_field` implementation ends with a catch-all that **skips and
discards** any unrecognized tag —
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/prost-derive-0.14.4/src/lib.rs:176–192`:

```rust
        impl #impl_generics #prost_path::Message for #ident #ty_generics #where_clause {
            ...
            fn merge_field(
                &mut self,
                tag: u32,
                wire_type: #prost_path::encoding::wire_type::WireType,
                buf: &mut impl #prost_path::bytes::Buf,
                ctx: #prost_path::encoding::DecodeContext,
            ) -> ::core::result::Result<(), #prost_path::DecodeError>
            {
                #struct_name
                match tag {
                    #(#merge)*
                    _ => #prost_path::encoding::skip_field(wire_type, tag, buf, ctx),
                }
```

`skip_field` advances past the bytes and retains nothing —
`.../prost-0.14.4/src/encoding.rs:166–197`:

```rust
pub fn skip_field(
    wire_type: WireType,
    tag: u32,
    buf: &mut impl Buf,
    ctx: DecodeContext,
) -> Result<(), DecodeError> {
    ctx.limit_reached()?;
    let len = match wire_type {
        WireType::Varint => decode_varint(buf).map(|_| 0)?,
        WireType::ThirtyTwoBit => 4,
        WireType::SixtyFourBit => 8,
        WireType::LengthDelimited => decode_varint(buf)?,
        WireType::StartGroup => loop { ... },
        WireType::EndGroup => return Err(DecodeErrorKind::UnexpectedEndGroupTag.into()),
    };

    if len > buf.remaining() as u64 {
        return Err(DecodeErrorKind::BufferUnderflow.into());
    }

    buf.advance(len as usize);
    Ok(())
}
```

`grep -rn -E 'unknown|skip_field' .../prost-0.14.4/src` returns only:
`error.rs:229` / `error.rs:239` (the `UnknownEnumValue` error type) and the `skip_field` call
sites in `encoding.rs`/`types.rs`. There is **no** `unknown_fields` field, type, or `Message`
trait method anywhere in `prost 0.14.4`. The `Message` trait
(`.../prost-0.14.4/src/message.rs:11–44`) exposes only `encode_raw`, `merge_field`,
`encoded_len`, `encode*`, `decode*`, `clear`, and the `Name` type-url helpers.

The version-pinned `README.md` for `prost 0.14.4` claims only
`* Preserves unknown enum values during deserialization.` (line 25) — it makes no unknown-field
retention claim, matching ADR-002 line 210.

### Verdict

**SETTLED: prost DROPS unknown fields on decode.** A v2 message carrying an added field (field 3)
decoded by a v1-only `prost` type and re-encoded loses those bytes
(`24 bytes → 9 bytes`, `extra=""` after round-trip). This holds for `prost` `0.14.4` (ADR's
pinned version) and `0.13.5`; `0.14.4` is the newest release. Source confirms the mechanism:
the generated catch-all calls `encoding::skip_field`, and there is no unknown-field storage or
`Message` API. This **resolves** ADR-002 unverified item #1 and validates D4's caution: the
"no relay/proxy/store-and-forward hop reserializes decoded messages" rule is necessary, because
such a hop under `prost` would silently drop forward-compatible fields. ADR-002 does **not**
assert retention anywhere; it explicitly flags the gap (line 210), so there is no contradicted
claim — but any future text asserting `prost` "preserves unknown fields" would be false.

---

## Item #2 — `rustls` pure Rust / OpenSSL / default crypto provider

### Setup

Scratch crate `/tmp/adr002-rustls` depending on `rustls` with default features:

`Cargo.toml`:

```toml
[package]
name = "adr002-rustls"
version = "0.1.0"
edition = "2021"

[dependencies]
rustls = "0.23.46"
```

### Commands (exact)

```bash
cd /tmp/adr002-rustls
cargo generate-lockfile
cargo tree -e features
cargo tree | grep -i -E 'openssl|ring|aws-lc'
grep -i 'openssl' Cargo.lock
which cmake; which nasm; which cc gcc
cargo build
```

### Exit codes

`cargo generate-lockfile` → `0`; `cargo tree -e features` → `0` (`TREE_EXIT:0`);
`cargo tree | grep ...` → `0` (`GREP_EXIT:0`, matches found); `grep -i 'openssl' Cargo.lock` →
`1` (no match); `which cmake` → `0` (`/usr/bin/cmake`), `which nasm` → `1` (not installed),
`which cc gcc` → `0`; `cargo build` → `0` (`BUILD_EXIT:0`).

### Raw output

Resolved versions of interest (`Cargo.lock`):

```
name = "rustls"
version = "0.23.46"
---
name = "aws-lc-rs"
version = "1.18.2"
---
name = "aws-lc-sys"
version = "0.46.0"
---
name = "rustls-pki-types"
version = "1.15.1"
---
name = "rustls-webpki"
version = "0.103.15"
```

`cargo tree | grep -i -E 'openssl|ring|aws-lc'`:

```
    ├── aws-lc-rs v1.18.2
    │   ├── aws-lc-sys v0.46.0
    │   ├── aws-lc-rs v1.18.2 (*)
GREP_EXIT:0
```

`cargo tree` top of the default tree:

```
adr002-rustls v0.1.0 (/tmp/adr002-rustls)
└── rustls v0.23.46
    ├── aws-lc-rs v1.18.2
    │   ├── aws-lc-sys v0.46.0
    │   │   [build-dependencies]
```

`cargo tree -e features` relevant branches:

```
    ├── rustls feature "aws_lc_rs"
    │   ├── rustls v0.23.46 (*)
    │   ├── rustls feature "aws-lc-rs"
    │   │   └── rustls v0.23.46 (*)
    │   ├── aws-lc-rs feature "aws-lc-sys"
    │   │   └── aws-lc-rs v1.18.2 (*)
    │   ├── aws-lc-rs feature "prebuilt-nasm"
    │   │   ├── aws-lc-rs v1.18.2 (*)
    │   │   └── aws-lc-sys feature "prebuilt-nasm"
    │   │       └── aws-lc-sys v0.46.0 (*)
    │   └── rustls-webpki feature "aws-lc-rs"
    │       └── rustls-webpki v0.103.15 (*)
    ...
    ├── rustls feature "logging"
    ├── rustls feature "prefer-post-quantum"
    ├── rustls feature "std"
    └── rustls feature "tls12"
```

OpenSSL check: `grep -i 'openssl' Cargo.lock` printed nothing and exited `1`; `cargo tree |
grep -i -E 'openssl|ring|aws-lc'` matched **only** `aws-lc-rs`/`aws-lc-sys`, **no** `openssl*`
and **no** `ring` node. The default tree uses feature `aws_lc_rs` → `aws-lc-rs` **without** the
`ring` feature. (`ring 0.17.14` does appear as a package stanza in `Cargo.lock`, but it is the
un-activated optional `rustls` `ring` dependency: `cargo tree | grep -i ring` exits `1`, i.e.
`ring` is not in the built default tree. `Cargo.lock` records optional packages regardless of
feature activation.)

`cargo build` tail (compiles the native provider successfully):

```
   Compiling aws-lc-rs v1.18.2
   Compiling rustls v0.23.46
   ...
   Compiling cmake v0.1.58
   Compiling aws-lc-sys v0.46.0
   Compiling rustls-webpki v0.103.15
   Compiling adr002-rustls v0.1.0 (/tmp/adr002-rustls)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.12s
BUILD_EXIT:0
```

`aws-lc-sys` is not pure Rust, so "rustls default is pure Rust" is false even though the build
succeeded here. Counts in the vendored `aws-lc-sys-0.46.0` source tree:

```
C files: 423
S/asm files: 951
Rust files: 39
```

### Vendor statements (version-pinned, `rustls 0.23.46`)

`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustls-0.23.46/README.md`.

Lines 46–49:

```
While Rustls itself is platform independent, by default it uses [`aws-lc-rs`] for implementing
the cryptography in TLS.  See [the aws-lc-rs FAQ][aws-lc-rs-platforms-faq] for more details of the
platform/architecture support constraints in aws-lc-rs.
```

Line 50–51:

```
[`ring`] is also available via the `ring` crate feature: see
[the supported `ring` target platforms][ring-target-platforms].
```

Line 59:

```
Specifying `default-features = false` when depending on rustls will remove the implicit
dependency on aws-lc-rs.
```

Lines 81–84:

```
Rustls ships with two built-in providers controlled by associated crate features:

* [`aws-lc-rs`] - enabled by default, available with the `aws_lc_rs` crate feature enabled.
* [`ring`] - available with the `ring` crate feature enabled.
```

The README also lists `rustls-openssl` only as a **third-party** provider (lines 89–108), not the
default. The `aws-lc-rs` README (`aws-lc-rs-1.18.2/README.md:7–10`) describes it as wrapping
AWS-LC through the `aws-lc-sys` FFI crate, i.e. native C/assembly crypto.

### Verdict

**SETTLED.**
- **OpenSSL: no.** The default `rustls 0.23.46` dependency tree contains no `openssl`,
  `openssl-sys`, or any OpenSSL binding (`grep -i openssl Cargo.lock` → no match, exit `1`;
  `cargo tree | grep` shows none).
- **Default crypto provider: `aws-lc-rs`, not `ring`.** Default feature `aws_lc_rs` pulls
  `aws-lc-rs 1.18.2` → `aws-lc-sys 0.46.0` (README: "by default it uses `aws-lc-rs`"; "enabled
  by default"). `ring` is only pulled with the non-default `ring` feature.
- **"Pure Rust": false as stated for the default configuration.** The `rustls` crate itself is
  written in Rust, but its default provider `aws-lc-sys` contains 423 `.c` files and 951
  assembly files, so the default build is **not** pure Rust (it needs a C toolchain/CMake; it
  built here with `/usr/bin/cc` and `/usr/bin/cmake`, no `nasm` needed because the
  `prebuilt-nasm` feature was active). A truly pure-Rust claim would require
  `default-features = false` plus a custom `CryptoProvider`.

This **resolves** ADR-002 unverified item #2. ADR-002's own text (line 260) makes no pure-Rust
assertion and flags the item as unverified, so nothing in the ADR is contradicted — but the
evidence warns against writing "pure Rust" into the ADR as a settled fact: only "TLS 1.3
implemented with rustls, no OpenSSL, default provider aws-lc-rs (native C/assembly via
aws-lc-sys)" is supported.

---

## ADR-002 statement reconciliation

| ADR location | ADR statement | Evidence outcome |
|---|---|---|
| Line 210 (D4) | "what `prost` does with unknown **fields** is **unverified**" | Now verified: prost **drops** unknown fields; the ADR's no-relay stance is required and correct. Not contradicted. |
| Line 208 (D4) | "Proto3 messages preserve unknown fields…" (quoted about the protobuf model) | The quoted claim is about the protobuf model; the `prost` implementation does not implement it. No contradiction because the ADR distinguishes model from implementation. |
| Line 171 (D1) | `prost`/`prost-build`/`prost-types` `0.14.4`; "`protoc` is therefore a recorded build-time dependency whose presence on this host is unverified" | Versions reproduce exactly; `protoc` **is** present (`libprotoc 36.1`) and works with default prost-build config. Unverified claim now settled. |
| Line 260 (D7) | "no staged file supports a 'pure Rust / avoids OpenSSL' claim about rustls. The default crypto-provider choice (`ring` vs `aws-lc-rs`) … is not visible" | Verified: default provider is `aws-lc-rs`; no OpenSSL; **not** pure Rust due to native `aws-lc-sys`. No ADR assertion contradicted; item resolved. |
| Line 408 (Consequences) | "`protoc` is not listed in the verified local environment table… Whether it is present on this host is unverified." | `protoc` is present at `/usr/bin/protoc`, `libprotoc 36.1`; the shared `AGENTS.md` environment table could be updated to list it. |
| Lines 433, 434, 441 (Unverified table) | Items #1, #2, #9 | All three settle as above. |

No statement in ADR-002 is factually contradicted by this evidence. The three items should move
from "Unverified / outstanding verification" to verified, with the prost result recorded as
"DROPS unknown fields" and the rustls result recorded as "default provider aws-lc-rs; no
OpenSSL; not pure Rust by default."

---

## Return summary

| Item | Verdict | One-line answer | Proving command |
|---|---|---|---|
| #1 | **SETTLED (drops)** | `prost` 0.14.4 (and 0.13.5) **drops** unknown fields on decode; a v1 decode+re-encode of a v2 message loses field 3 (`24 → 9` bytes, `extra=""`). | `cd /tmp/adr002-unknown && cargo run --quiet` |
| #2 | **SETTLED** | `rustls` 0.23.46 default features pull **aws-lc-rs 1.18.2 / aws-lc-sys 0.46.0** (not `ring`), **no OpenSSL**; default build is **not** pure Rust (native C/asm in aws-lc-sys). | `cd /tmp/adr002-rustls && cargo tree \| grep -i -E 'openssl\|ring\|aws-lc'` |
| #9 | **SETTLED** | `protoc` exists: `/usr/bin/protoc -> protoc-36.1.0`, **`libprotoc 36.1`**; `prost-build` 0.14.4 finds it on `PATH` with no extra config and no vendored fallback. | `protoc --version` (plus the successful `/tmp/adr002-unknown` build) |

Nothing in this scope was BLOCKED; network was available and all commands exited `0`.
