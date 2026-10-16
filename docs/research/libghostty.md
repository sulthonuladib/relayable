# libghostty / libghostty-vt / Rust bindings evaluation — license, build, API, Android, and vs the ADR-003 pins

Status: **research artifact, evidence only.** This file answers task `RESEARCH-002`
(`docs/agent-loop/TASKS.md`) for `libghostty` / `libghostty-rs` (license, build, API, Android, vs
the ADR-003 pins). It does **not** change any pin, ADR, or code, and it makes **no REX wire/binary
compatibility claim**. It feeds TERM-003 (a possible libghostty engine spike + ADR-003 amendment,
reconsideration trigger #10).

**Retrieval window:** all HTTP fetches in this file were made `2026-10-16T04:52:43Z`–`05:09:28Z` (UTC).
**Session start (network probe):** `2026-10-16T04:52:33Z`.

This pass had working read-only HTTP egress, unlike the prior session quoted in `result:ghostty#1`.
That changes the answer materially: the upstream repository, its license and cadence, the real Rust
bindings, and both bindings' `build.rs` are now **verified from fetched source**, not inferred.

## Evidence basis and conventions

Three evidence classes are kept separate and never blurred:

- **(a) HTTP-fetched** — fetched in this session (2026-10-16 UTC) with `curl`/`wget`, read-only.
  Every claim carries the URL and is listed in §18. Raw bodies are saved under
  `/tmp/libghostty-evidence/` (§19).
- **(b) Local primary artifacts read from disk** — the host's installed Ghostty package and the
  extracted published crate tarballs. Exact paths are named inline.
- **(c) Inference** — labelled `(inference)`; never presented as fact.

Confidence markers: **high** (fetched primary source: repo API, `LICENSE`, `build.rs`, published
`.crate`), **medium** (single fetched source or date taken from a non-API endpoint), **low**
(second-hand), **UNVERIFIED** (could not be established), **not observed** (absence of evidence,
not evidence of absence), **inferred**.

Where upstream text is quoted it is quoted verbatim with its file/URL. Nothing below invents a
symbol, version, commit, flag or date.

**Version terms used throughout (high, this matters):**

- **"release libghostty-vt"** = the C library as shipped in the latest tagged Ghostty release,
  **v1.3.1** — this is exactly what the host has installed (`ghostty 1.3.1-2`).
- **"main libghostty-vt"** = the C library on the `main` branch today (`build.zig.zon` says
  `version = "1.3.2-dev"`), which is what both Rust bindings pin. It is **not in any tag yet**.

Both expose the **same two stability warnings**; only `main` exposes the terminal/grid API (§5, §8).

---

## 1. Bottom line

- **The `Uzaaft/libghostty-rs` lead is CONFIRMED** (it was only a browser-tab title in the prior
  evidence set). It is a real, active, 394-star Rust project publishing the crates
  **`libghostty-vt`** and **`libghostty-vt-sys`** on crates.io. A **second** binding,
  **`ghostty-vt`** / **`ghostty-vt-sys`**, is published from `prabirshrestha/gpui-cn` (an **adaptation**
  of the first — *not* a GitHub fork: the repo HTML reports `repository_is_fork = "false"`; the
  adaptation is supported by the binding's own comments, `Adapted from libghostty-vt-sys 0.2.2
  (MIT OR Apache-2.0)` and `Adapted from libghostty-vt 0.2.2`). A third crates.io name, **`libghostty`**, is an unrelated **yanked placeholder**.
- **Upstream license is MIT**, verified from the fetched `/ghostty-org/ghostty/LICENSE` (sha256
  `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1`). No `NOTICE` file exists
  upstream (three probes → HTTP 404).
- **The decisive build blocker is real: every maintained Rust binding requires a system Zig 0.16.x
  toolchain** in its default build (`build.rs` quoted in §6). Zig is **not installed** on this host
  (`command -v zig` → not found; `mise which zig` → not a mise bin). Per the project's
  no-global-install/no-sudo rule this is a **BLOCKER for the default (vendored) build**, and the
  opt-in `pkg-config` "use the system library" path does **not** unblock it here, because the
  installed C library (`libghostty-vt.so.0.1.0`) is an **API-incompatible, utility-only build** —
  see §5.3 and §6.4.
- **The full terminal API exists only on `main`.** The release the host runs (`v1.3.1`) exposes
  only OSC/SGR/key/paste/color/wasm utilities, exactly as the prior session reported. Upstream
  `main` adds `Terminal`, `RenderState`, scrollback/reflow, selection, mouse, kitty graphics, search,
  snapshot and more — but carries "this API … is not yet stable and is definitely going to change".
- **Trigger #10 is NOT met on present evidence:** the license sub-item becomes **VERIFIED**
  (upstream MIT, verified from the fetched file); the *maintained Rust binding* sub-item is now
  **VERIFIED-WITH-CAVEATS** (real, active, licensed crates exist — but the leading crate ships no
  license file in its tarball, and both need Zig); the *passing build/spike* sub-item stays
  **STILL UNVERIFIED** (no build was attempted; Zig is absent by project rule). Recommendation:
  **DEFER / PARTIAL** — keep the ADR-003 pins; the ADR should **not** be reopened on this evidence
  alone.

---

## 2. What `libghostty` / `libghostty-vt` is, and who owns it

**(a) Fetched, high.** `https://api.github.com/repos/ghostty-org/ghostty` (2026-10-16):

| Field | Value |
|---|---|
| `full_name` | `ghostty-org/ghostty` |
| `description` | 👻 Ghostty is a fast, feature-rich, and cross-platform terminal emulator that uses platform-native UI and GPU acceleration. |
| `homepage` | `https://ghostty.org` |
| `default_branch` | `main` |
| `language` | Zig |
| `license.spdx_id` | `MIT` |
| `stargazers_count` | 62266 |
| `forks_count` | 3656 |
| `open_issues_count` | 258 |
| `created_at` | `2022-03-29T17:38:26Z` |
| `pushed_at` | `2026-10-15T21:55:51Z` |
| `archived` | false |

**(a) Fetched, high.** `raw.githubusercontent.com/ghostty-org/ghostty/main/README.md` describes the
three-layer relationship:

> **`libghostty`** is a cross-platform, zero-dependency C and Zig library for building terminal
> emulators or utilizing terminal functionality (such as style parsing).

and, in the "Cross-platform `libghostty` for Embeddable Terminals" section:

> Due to the scope of this project, we're breaking libghostty down into separate libraries, starting
> with `libghostty-vt`. The goal of this project is to focus on parsing terminal sequences and
> maintaining terminal state.

So the relationship is:

- **Ghostty the app** — the native GUI terminal. **(inference, medium)**: a Zig core with
  platform-native front-ends. This is grounded on the fetched repo `languages` breakdown
  (`ghostty-languages.json`: Zig 13.1 MB, then Swift 1.46 MB, C, C++, …) and the README's
  *"platform-native UI and GPU acceleration"*; the README does **not** name the front-end toolkits.
- **`libghostty`** — the umbrella C/Zig API for embedding a terminal, being split into pieces.
- **`libghostty-vt`** — the first (and currently only shipped) piece: the VT/terminal-state library
  extracted from the app's core.
- **Rust bindings** — *third-party*: not owned by `ghostty-org`. §4 names the real ones.

**Official docs.** `https://ghostty.org` (repo `homepage`, also linked from the `libghostty-vt.pc`
`URL:` field read locally) and the C-API Doxygen site
`https://libghostty.tip.ghostty.org/` (fetched, HTTP 200; the README points to it). The installed
header `/usr/include/ghostty/vt.h` also names `https://ghostty.org` in its `@mainpage` block.

**(b) Local, high.** The host's own package metadata agrees with the fetched repo:
`/var/lib/pacman/local/ghostty-1.3.1-2/desc` → `%URL% https://github.com/ghostty-org/ghostty`,
`%LICENSE% MIT`; `/usr/include/ghostty/vt.h` and `/usr/share/pkgconfig/libghostty-vt.pc`
(`URL: https://github.com/ghostty-org/ghostty`).

---

## 3. Licenses

### 3.1 Upstream Ghostty / libghostty-vt — **MIT** (verified)

**(a) Fetched, high.** `https://raw.githubusercontent.com/ghostty-org/ghostty/main/LICENSE`
(HTTP 200, 2026-10-16):

```
MIT License

Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors
```

sha256 `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1` (1097 bytes).

`NOTICE`, `NOTICE.md`, `COPYING` at the repo root → **HTTP 404 each** (fetched 2026-10-16). So the
upstream distribution carries **no `NOTICE` file**, consistent with ADR-003's finding for the other
pinned crates.

**(b) Local, high.** The installed package agrees: `/usr/share/licenses/ghostty/LICENSE` begins
`MIT License / Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors`;
`/var/lib/pacman/local/ghostty-1.3.1-2/desc` → `%LICENSE% MIT`.

**Verdict:** upstream Ghostty **including libghostty-vt** is **MIT**. This closes the "verified
license" sub-item of ADR-003 Option E / trigger #10 for the *library*.

### 3.2 Rust bindings — bindings have separate licenses, and one ships no license file

| Binding crate | Declared `license` (crates.io) | License file **inside the published `.crate`** | `NOTICE` |
|---|---|---|---|
| `libghostty-vt` 0.2.2 (Uzaaft) | `MIT OR Apache-2.0` (high) | **NONE** — `find` over the extracted tarball → no `LICENSE*`/`COPYING*`/`NOTICE*` | none |
| `libghostty-vt-sys` 0.2.2 (Uzaaft) | `MIT OR Apache-2.0` (high) | **NONE** | none |
| `ghostty-vt` 0.7.2+ghostty.1.3.2-main.b40acce | `Apache-2.0` (high) | `LICENSE` (Apache-2.0) + `LICENSE-GHOSTTY` (MIT, upstream Ghostty) | none |
| `ghostty-vt-sys` 0.7.2+… | `Apache-2.0` (high) | `LICENSE` + `LICENSE-GHOSTTY` | none |
| `libghostty` 0.1.0 (remissio) | `MIT` (high) | none; **crate is yanked**, 906 bytes, placeholder | none |

**(b) Local, high** — extracted from the fetched `.crate` tarballs under
`/tmp/libghostty-evidence/extracted/`:

- `libghostty-vt-0.2.2/` file tree lists only `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`,
  `.cargo_vcs_info.json`, `README.md`, `src/**` — **no license file**. Same for
  `libghostty-vt-sys-0.2.2/`.
- `ghostty-vt-0.7.2+…/LICENSE` is the standard Apache License 2.0 text (11358 bytes);
  `LICENSE-GHOSTTY` is **byte-identical** to the upstream Ghostty MIT file
  (both sha256 `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1`).

**(a) Fetched, high.** The Uzaaft repository *root* does contain an MIT license:
`https://raw.githubusercontent.com/Uzaaft/libghostty-rs/master/LICENSE` (HTTP 200) begins
`MIT License / Copyright (c) 2026 Uzair Aftab, Leah Amelia Chen`. `LICENSE-MIT` and `LICENSE-APACHE`
at the repo root → **HTTP 404**. So the project declares **`MIT OR Apache-2.0`** but only the MIT
text exists in the repo, and **the published crate artifacts bundle no license file at all**.

**Licensing risk (inference, medium).** ADR-003 rejected `Glass-HQ/gpui` because a pinned tree with
no license file makes Apache-2.0 §4(a) ("give any other recipients … a copy of this License")
unsatisfiable from the artifact alone. The **leading** Rust binding (`libghostty-vt` +
`libghostty-vt-sys`, the crates most downstream projects use) reproduces that exact pattern:
declared license, **no license text in the crate**. Because the declaration is a **disjunction**
(`MIT OR Apache-2.0`), a consumer may elect MIT, whose notice obligation is satisfiable from the
repo's `LICENSE`; but the crate artifact itself does not carry the notice. This is a **material
finding** the lead must weigh, not a settled legal conclusion. The `ghostty-vt` (prabirshrestha)
crates are cleaner: they bundle both licenses. Note also (inference) that `ghostty-vt`'s source says
it is *"Adapted from libghostty-vt 0.2.2 (MIT OR Apache-2.0)"* while its bundled `LICENSE` is plain
Apache-2.0 with no upstream copyright line in the file itself; the attribution survives only as a
source-comment (`src/lib.rs:37`, `build.rs` header). Whether Apache-2.0 §4(b)/(c) are fully satisfied
is **not adjudicated here**.

**Not a binding.** `libghostty` 0.1.0 (publisher `remissio`) is a `cargo new` placeholder — its
`src/lib.rs` is literally `pub fn add(left: u64, right: u64) -> u64` — it has **no repository**,
**no dependencies**, and is **yanked** (`crates.io/api/v1/crates/libghostty` →
`"yanked": true`, `"max_version": "0.0.0"`). It must not be confused with the real bindings.

---

## 4. The real Rust bindings (identity resolved)

### 4.1 `Uzaaft/libghostty-rs` → crates `libghostty-vt` + `libghostty-vt-sys` — **the lead, CONFIRMED**

**(a) Fetched, high** (`https://api.github.com/repos/Uzaaft/libghostty-rs`):

| Field | Value |
|---|---|
| `full_name` | `Uzaaft/libghostty-rs` |
| `description` | (none) |
| `default_branch` | `master` (not `main`) |
| `language` | Rust |
| `license.spdx_id` | `MIT` |
| `stargazers_count` | 394 |
| `forks_count` | 44 |
| `open_issues_count` | 13 |
| `created_at` | `2026-03-21T21:25:55Z` |
| `pushed_at` | `2026-10-15T15:17:42Z` |
| `archived` | false |

**(a) Fetched, high** (`https://crates.io/api/v1/crates/libghostty-vt` + `/versions` + `/owners`):
owner `Uzaaft` ("Uzair Aftab") plus `pluiedev` ("Leah Amelia Chen"); `repository =
https://github.com/uzaaft/libghostty-rs`; description *"Safe Rust API for libghostty-vt, the Ghostty
terminal emulation library"*; 167125 total downloads. Versions (all non-yanked):

| Version | Published (UTC) | `license` | `rust-version` | crate size |
|---|---|---|---|---|
| 0.2.2 | 2026-09-28 | MIT OR Apache-2.0 | 1.90 | 82491 |
| 0.2.1 | 2026-07-18 | MIT OR Apache-2.0 | 1.90 | 81007 |
| 0.2.0 | 2026-06-16 | MIT OR Apache-2.0 | 1.90 | 72287 |
| 0.1.1 | 2026-03-28 | MIT OR Apache-2.0 | 1.93 | 37937 |
| 0.1.0 | 2026-03-28 | MIT OR Apache-2.0 | 1.93 | 37938 |

Tags: `v0.2.0`, `v0.2.1`, `v0.2.2`; GitHub releases: `v0.2.2` (2026-09-28), `v0.2.0` (2026-06-16).
Latest commits on `master` (fetched): `d2036baeb605` (2026-10-09, *"Merge pull request #91 …
ghostty-formatter"*), with recent work on snapshot, paste, search and formatter modules.
`libghostty-vt-sys` versions track the safe crate (0.1.0 … 0.2.2), declared `MIT OR Apache-2.0`,
`links = "ghostty-vt"`.

**Workspace layout (high, from the fetched repo README):**

> - `crates/libghostty-vt-sys` — raw FFI bindings generated from `ghostty/vt.h`
> - `crates/libghostty-vt` — safe Rust wrappers (Terminal, RenderState, KeyEncoder, MouseEncoder, etc.)

### 4.2 `prabirshrestha/gpui-cn` → crates `ghostty-vt` + `ghostty-vt-sys` — **a second, newer binding**

**(a) Fetched, high** (`https://crates.io/api/v1/crates/ghostty-vt`): `repository =
https://github.com/prabirshrestha/gpui-cn`; description *"Safe Rust API for libghostty-vt, the
Ghostty terminal emulation library, built from Ghostty's official source tarball"*; declared
`license = Apache-2.0`; 106 total downloads. Versions:

| Version | Published (UTC) | `license` | `rust-version` |
|---|---|---|---|
| 0.7.2+ghostty.1.3.2-main.b40acce | 2026-10-14 | Apache-2.0 | 1.96 |
| 0.7.0+ghostty.1.3.2-main.b40acce | 2026-10-10 | Apache-2.0 | 1.96 |
| 0.6.0+ghostty.1.3.2-main.b40acce | 2026-10-08 | Apache-2.0 | 1.96 |
| 0.0.1 | 2026-10-08 | Apache-2.0 | 1.85 |

**(a) Fetched, high.** The repo `https://github.com/prabirshrestha/gpui-cn` (HTML, 2026-10-16)
has **1 star** — a personal monorepo, very low adoption. Its published crates are a **superset**
adaptation of the Uzaaft project: the sys `build.rs` header says *"Adapted from libghostty-vt-sys
0.2.2 (MIT OR Apache-2.0)"* and `ghostty-vt/src/lib.rs:37` says *"Adapted from libghostty-vt 0.2.2
(MIT OR Apache-2.0)"*. It pins a **newer** Ghostty commit and adds `search`, `snapshot`,
`terminfo`, `shell_integration`, `resources` and `io` modules the Uzaaft crate does not have.

### 4.3 Community GPUI + libghostty precedents (relevant to `full.md` §5.3)

**(a) Fetched, high** (GitHub repo search + one repo tarball):

- **`behzade/gpui-libghostty`** (MIT, Rust) — *"Embed a terminal or neovim in a gpui app using
  libghostty"*. Extracted its repo tarball: it **vendors Ghostty source**
  (`crates/gpui-ghostty/vendor/ghostty/`), **builds `libghostty-internal.a` with Zig**, and its
  `flake.nix` pins `zig-overlay` `0.16.0`. Its README Requirements list **"Zig 0.16"** and it
  explicitly supports **"macOS and Linux with Wayland"**. Workspace lints set `unsafe_code =
  "forbid"`. This is the clearest existing proof that a **GPUI + libghostty** integration is
  buildable — and that it does **not** avoid Zig even when Ghostty is vendored.
- `modu-ai/moai-studio` and `nowledge-co/con-terminal` (both Rust) are listed by
  `Uzaaft/awesome-libghostty/master/README.md` under **"AI Tools & Agent Orchestration"**
  (lines 124 and 118), and `Uzaaft/libghostty-rs` itself is listed under **"Core & Libraries"**
  (line 31) as *"Rust FFI bindings and safe API for libghostty-vt, with a Rust port of Ghostling"*.
  `sadiksaifi/SpaceTerm` (Rust) — *"A native, keyboard-first desktop terminal multiplexer built with
  Rust, GPUI, and libghostty-vt"* — comes from the GitHub repository search `?q=libghostty-vt`
  (saved as `gh-search-libghostty-vt.json`, 4★); it is **not** in the awesome-libghostty list.

**(inference, medium):** the Rust ecosystem around libghostty is real and active, and GPUI hosts
already exist, but **all of them pay the Zig 0.16 build cost** (§6).

---

## 5. Version, cadence and stability posture

### 5.1 Releases and cadence

**(a) Fetched, high/medium.** Upstream ships **tags**, not GitHub releases (the only release object
is the 2022 `tip` nightly). Tag commit dates (2026-10-16):

| Tag | Commit date (UTC) | Source |
|---|---|---|
| v1.0.0 | 2024-12-26 | commits API (high) |
| v1.0.1 | 2024-12-31 | commits API (high) |
| v1.1.0 | 2025-01-30 | commits API (high) |
| v1.1.1 | 2025-02-13 | commits API (high) |
| v1.1.2 | 2025-02-13 | commits API (high) |
| v1.1.3 | 2025-03-24 | commits API (high) |
| v1.2.0 | 2025-09-15 | commits API (high) |
| v1.2.1 | 2025-10-06 | commits API (high) |
| v1.2.2 | 2025-10-08 | `commits/v1.2.2.atom` (medium) |
| v1.2.3 | 2025-10-23 | `commits/v1.2.3.atom` (medium) |
| v1.3.0 | 2026-03-09 | `commits/v1.3.0.atom` (medium) |
| v1.3.1 | 2026-03-13 | `commits/v1.3.1.atom` (medium) |

Latest default-branch commit: **`b3dc3d794513`** (2026-10-15T21:12:20Z, *"benchmark: convert prints
to log (#14718)"*). `build.zig.zon` on `main` declares `version = "1.3.2-dev"`, i.e. the next
release is in development. **Cadence (inference, medium):** roughly 6–10 weeks between minor
releases, patch releases within days; last release (v1.3.1, 2026-03) is ~7 months behind `main`
today.

### 5.2 Stability posture — verbatim

The prior session's quote is **confirmed against live upstream source** (and is present in *both*
the `v1.3.1` release header and `main`). From
`https://raw.githubusercontent.com/ghostty-org/ghostty/main/include/ghostty/vt.h` (and the identical
text at the `v1.3.1` tag, `.../v1.3.1/include/ghostty/vt.h`):

> `WARNING: This is an incomplete, work-in-progress API. It is not yet stable and is definitely going to change.`

and in the `@mainpage` block:

> `@warning This library is currently in development and the API is not yet stable. Breaking changes are expected in future versions. Use with caution in production code.`

From the upstream README (`main`):

> `libghostty-vt` is already available and usable today for Zig and C and is compatible for macOS, Linux, Windows, and WebAssembly. The functionality is extremely stable (since its been proven in Ghostty GUI for a long time), but the API signatures are still in flux.

> We haven't tagged libghostty with a version yet and we're still working on a better docs experience, but our [Doxygen website](https://libghostty.tip.ghostty.org/) is a good resource for the C API.

**Read together (inference, high confidence):** upstream distinguishes **functional stability** of
the engine (proven inside the Ghostty GUI) from **API-signature stability** (explicitly *not*
stable). Note the README's own compatibility list — **"macOS, Linux, Windows, and WebAssembly"** —
does **not** name Android.

### 5.3 The version split that matters for this project

**(b) Local, high.** The host runs `ghostty 1.3.1-2` (`/var/lib/pacman/local/ghostty-1.3.1-2/`).
Its `libghostty-vt` is the **release 0.1.0 ABI only**:

```
$ cat /usr/share/pkgconfig/libghostty-vt.pc          # Version: 0.1.0
$ ls /usr/include/ghostty/vt/                          # allocator color key key.h osc paste result sgr wasm
$ grep '#include' /usr/include/ghostty/vt.h            # result allocator osc sgr key paste wasm
$ nm -D /usr/lib/libghostty-vt.so.0.1.0 | grep -oE 'ghostty_[a-z0-9_]+' | sort -u | wc -l   # 43
```

Those **43 exported symbols** are only `ghostty_osc_*`, `ghostty_sgr_*`, `ghostty_key_*`,
`ghostty_paste_is_safe`, `ghostty_color_rgb_get`, and SIMD helpers — **no `ghostty_terminal_*` and
no `ghostty_render_state_*`**. This exactly matches the fetched `v1.3.1` `vt.h`, whose include list
is only `result/allocator/osc/sgr/key/paste/wasm`.

**(a) Fetched, high.** `main`'s `vt.h` include list, and the directory listing of
`include/ghostty/vt/` at `main`, contain the full terminal surface:

```
terminal.h render.h selection.h mouse.h mouse/event.h mouse/encoder.h
screen.h modes.h snapshot.h search.h formatter.h grid_ref.h grid_ref_tracked.h
style.h unicode.h kitty_graphics.h focus.h size_report.h io.h sys.h device.h
color_scheme.h point.h build_info.h types.h
```

i.e. the **terminal/grid/scrollback/selection/mouse API is a post-1.3.1, `main`-only development**.
The prior session's "the packaged version exposes only utility sub-parsers" is **correct for
v1.3.1**, and must be **refined**: upstream `main` (and both Rust bindings) expose the full API.

---

## 6. Build requirements and the Zig blocker (CRITICAL)

### 6.1 Upstream requires Zig 0.16

**(a) Fetched, high.** `https://raw.githubusercontent.com/ghostty-org/ghostty/main/build.zig.zon`:

```
.version = "1.3.2-dev",
.minimum_zig_version = "0.16.0",
```

Upstream is built with `zig build` (Zig 0.16.x). It also declares an Android NDK path dependency
`.android_ndk = .{ .path = "./pkg/android-ndk" }` (§7).

### 6.2 Rust binding A — `libghostty-vt-sys` (Uzaaft) requires Zig on `PATH`

**(b) Local, high** — extracted from the fetched `.crate`. `libghostty-vt-sys-0.2.2/build.rs`:

```rust
const GHOSTTY_REPO: &str = "https://github.com/ghostty-org/ghostty.git";
const GHOSTTY_COMMIT: &str = "a887df42c56f6de86c0fe6da9c4eeca37931e083";
...
let mut build = Command::new("zig");
build.arg("build").arg("-Demit-lib-vt=true") ... .arg("-Dapp-runtime=none")
    .arg("--prefix").arg(&install_prefix) ...
```

- **Default feature `vendored`** (`Cargo.toml`: `default = ["vendored"]`). The vendored path
  **`git clone`s Ghostty** at the pinned commit into `OUT_DIR` and runs **`zig build`**.
- `features` (high): `bindgen-tool`, `default=["vendored"]`, `kitty-graphics`, `link-dynamic`,
  `pkg-config`, `vendored`. `[lib] links = "ghostty-vt"`.
- Env knobs (from `build.rs`): `GHOSTTY_SOURCE_DIR`, `GHOSTTY_ZIG_SYSTEM_DIR`,
  `LIBGHOSTTY_VT_SYS_CPU`, `LIBGHOSTTY_VT_SYS_OPTIMIZE`. **No `ZIG` env override and no `mise`
  fallback** — it calls bare `zig`.
- `zig_target()` maps `aarch64-linux-android` and `x86_64-linux-android` alongside desktop targets
  (Android evidence, §7).
- `pkg-config` path (opt-in feature): probes `libghostty-vt` (dynamic) or
  `libghostty-vt-static` (static) and **prefers an installed library over fetching Ghostty**. The
  README warns: *"libghostty is pre-1.0, so this crate intentionally does not promise compatibility
  with every installed C API revision."*

### 6.3 Rust binding B — `ghostty-vt-sys` (prabirshrestha) requires Zig 0.16.x explicitly

**(b) Local, high.** `ghostty-vt-sys-0.7.2+…/build.rs` resolves Zig and **hard-requires 0.16.x**:

```rust
fn resolve_zig() -> PathBuf {
    if let Some(zig) = env::var_os("ZIG") ... { check_zig_version(&zig); return zig; }
    let path_zig = PathBuf::from("zig");
    if matches!(&zig_version(&path_zig), Ok(v) if v.starts_with("0.16.")) { return path_zig; }
    // then: `mise which zig`
    ...
    panic!("ghostty-vt-sys needs Zig 0.16.x, found `{version}` at `zig`.\n\
            Install it with `mise install`, activate mise, or set ZIG to the Zig 0.16 binary.");
}
```

- Source lookup order — the `build.rs` **header comment** lists 1 `GHOSTTY_SOURCE_DIR` → 2
  `third_party/ghostty` submodule → 3 **download of `GHOSTTY_SOURCE_URL`** → 4 `pkg-config`
  (`libghostty-vt-static`); the **code order** in `main()` is source-dir → submodule → **pkg-config**
  → download. The pinned source lives in `GHOSTTY.lock` (`commit =
  b40acce58dcf77df52231c3798ea58e924647c89`, `url =
  https://tip.files.ghostty.org/…/libghostty-vt-source.tar.gz`, `sha256` pinned).
- Features (high): `default = []`; `pkg-config`, `bindgen-tool`. So the **default build downloads
  source and runs Zig**.
- `zig_target()` supports linux-gnu/musl, macos, windows-msvc, **and `aarch64-apple-ios` /
  `aarch64-apple-ios-sim`** — but **no Android target** (an explicit, documented panic for other
  targets). It has a `ZIG` env override and a `mise which zig` fallback.
- It supports the same `GHOSTTY_SOURCE_DIR` / `GHOSTTY_ZIG_SYSTEM_DIR` pre-seeding as A.

### 6.4 The no-global-install verdict — **BLOCKER (default build), and the system-lib path does not rescue it here**

**(b) Local, high.** This host: `command -v zig` → **not found**; no `zig` binary under
`/usr/bin`/`/usr/local/bin`; `mise which zig` → *"mise ERROR zig is not a mise bin. Perhaps you need
to install it first."*; `mise ls` shows no zig. So **neither binding's default (vendored) build can
run without installing Zig 0.16.x**, which is a global-ish toolchain install.

**What a `mise`-managed Zig pin would and would not solve (inference, high):**

- **Solves:** providing Zig 0.16.x **without `sudo` and without a system package** — `mise install
  zig@0.16.0` is a per-user install. This satisfies the letter of the project's "no global installs /
  no sudo" rule if the lead accepts a downloaded toolchain under `~/.local/share/mise`. It is the
  remedy the optional note in the task and `docs/architecture/next-phase-plan.md` anticipate.
- **Does not solve, by itself:**
  1. **Build-time network.** Both vendored paths fetch Ghostty (A `git clone`s; B downloads a
     tarball) **and** Zig's own package dependencies. A fully offline/hermetic build needs
     `GHOSTTY_SOURCE_DIR` + `GHOSTTY_ZIG_SYSTEM_DIR` (or a vendored tree), i.e. extra provisioning.
  2. **Consumer burden.** Only binding B has a `mise` fallback and a `ZIG` env override; binding A
     calls bare `zig` with **no `ZIG` override**, so a consumer of A must have `zig` on `PATH`.
  3. **Reproducibility.** Zig must be pinned exactly (0.16.x); A does not even version-check it.
  4. **The `pkg-config`/system-library escape hatch is unusable on this host.** The only installed
     C library is the release `0.1.0` utility-only build (§5.3), which lacks every
     `ghostty_terminal_*` / `ghostty_render_state_*` symbol the safe crates bind. A `pkg-config`
     build would therefore **link-fail**, not succeed. The README's own warning says the checked-in
     bindings "do not guarantee compatibility with arbitrary installed C API revisions."

**Plain statement:** *the maintained Rust bindings need a Zig 0.16.x toolchain; there is no working
Zig-free path on this host; this is a BLOCKER under the current project constraints.* This is a
**build-outcome statement made without building** — it is derived from the fetched `build.rs` files
plus the local absence of Zig and the local library's symbol set; **no build, link, or run was
performed** (forbidden in this task).

---

## 7. Cross-platform / Android (DEC-010)

**(a) Fetched, high** — evidence *for* Android target support upstream:

- `build.zig.zon` declares `.android_ndk = .{ .path = "./pkg/android-ndk" }`.
- `https://raw.githubusercontent.com/ghostty-org/ghostty/main/pkg/android-ndk/build.zig` exists and
  provides `pub fn addPaths(...)`, whose comment reads: *"Configure the step to point to the Android
  NDK for libc and include paths. This requires the Android NDK installed in the system and setting
  the appropriate environment variables … `ANDROID_NDK_HOME` … `ANDROID_HOME` or `ANDROID_SDK_ROOT`
  … NB: This is a workaround until zig natively supports bionic cross-compilation (ziglang/zig#23906)."*
  That is an **Android/bionic cross-compilation helper** for Zig builds.

**Evidence that is NOT present:**

- The upstream README's compatibility list is **"macOS, Linux, Windows, and WebAssembly"** — no
  Android. **(a) fetched, high.**
- No `NOTICE` of an Android *app/UI* in the upstream repo was found; the README frames libghostty as
  an embeddable C library. **(inference, medium).**
- The Uzaaft sys `build.rs` maps Android Rust/Zig targets (§6.2), but the **prabirshrestha** sys
  `build.rs` explicitly does **not** (it panics for unsupported targets, listing only linux/macos/
  windows/iOS). **(b) local, high.**
- Community Android work exists but is **third-party**: `SagerNet/libghostty-android`
  (*"libghostty-vt bindings and terminal components for Android"*), `jossephus/chuchu`
  (*"ssh client for android based on libghostty"*, Kotlin), `arcboxlabs/expo-libghostty`
  (*"Ghostty terminal view for Expo / React Native on iOS and Android, powered by libghostty"*).
  **(a) fetched, high** (GitHub repo search + the curated `awesome-libghostty` list).

**Verdict for DEC-010:** *Evidence found* that upstream has an **Android NDK cross-compilation
helper** for Zig builds, and that one binding (Uzaaft) declares Android target mappings; *no
evidence found* of an upstream Android app, of an Android rendering path, or of a tested Android
ABI build for either Rust binding. DEC-010 is **unaffected**: nothing here justifies starting
Android work, and nothing here closes TERM-002's "no Android rendering" gap on its own.

---

## 8. C API surface (real symbols, by version)

All symbol names below are taken from **fetched upstream `main` headers**
(`raw.githubusercontent.com/ghostty-org/ghostty/main/include/ghostty/vt/*.h`) and from the two
bindings' generated `bindings.rs` (extracted from the published `.crate` files). None are invented.

### 8.1 `main` (ghostty 1.3.2-dev) — full terminal engine

| Category | Exposed? | Real symbols / evidence |
|---|---|---|
| VT/ANSI stream parse → state | **Yes** | `ghostty_terminal_new`, `ghostty_terminal_vt_write`, `ghostty_terminal_vt_write_until_ground` (`terminal.h`) |
| Parser-vs-state separation | **Yes, by design** | `RenderState` is an incremental render model over `Terminal`; `ghostty_render_state_new/update/begin_update/end_update` (`render.h`) |
| Screen/grid/cell/style types | **Yes** | `ghostty_cell_get`, `ghostty_cell_get_multi`, `ghostty_row_get`, `ghostty_row_get_multi` (`screen.h`); grid refs via `ghostty_grid_ref_*` (`grid_ref.h`); styles via `GhosttyStyle` (`style.h`) |
| Scrollback buffer + history limits | **Yes** | `ghostty_terminal_scroll_viewport`, `GHOSTTY_TERMINAL_DATA_SCROLLBACK_ROWS`, `GHOSTTY_TERMINAL_OPT_SCROLLBACK_MAX_BYTES`, `…_MAX_LINES`, `ghostty_terminal_compress` (scrollback compression) |
| Reflow on resize | **Yes** | `ghostty_terminal_resize`; `GHOSTTY_TERMINAL_OPT_RESIZE_PULL_SCROLLBACK` |
| OSC general | **Yes** | `ghostty_osc_new/next/end/reset/free`, `ghostty_osc_command_type`, `ghostty_osc_command_data` |
| OSC 8 (hyperlinks) | **Yes** | `GHOSTTY_OSC_COMMAND_HYPERLINK_START/_END`; URI on cells via `ghostty_grid_ref_hyperlink_uri` (`grid_ref.h`) / `has_hyperlink`+`hyperlink_uri` in bindings |
| OSC 52 (clipboard) | **Yes** | `GHOSTTY_OSC_COMMAND_CLIPBOARD_CONTENTS`; callbacks `GHOSTTY_TERMINAL_OPT_CLIPBOARD_WRITE`/`CLIPBOARD_READ` (`GhosttyTerminalClipboardWriteFn`/`ReadFn`) |
| OSC 7 (cwd) | **Yes** | `GHOSTTY_OSC_COMMAND_REPORT_PWD`; `GHOSTTY_TERMINAL_OPT_PWD_CHANGED`; `GHOSTTY_TERMINAL_DATA_PWD` |
| OSC 133 (semantic prompt) | **Yes** | `GHOSTTY_OSC_COMMAND_SEMANTIC_PROMPT`; `GHOSTTY_TERMINAL_OPT_SEMANTIC_PROMPT` (`GhosttyTerminalSemanticPromptFn`) |
| OSC 7501 (program status) | **Yes** | `GHOSTTY_OSC_COMMAND_PROGRAM_STATUS`; `GHOSTTY_TERMINAL_OPT_PROGRAM_STATUS`; docs quote `ESC ] 7501 ; state=blocked:kind=permission:…` |
| Mouse modes / SGR mouse | **Yes** | Encoders + modes; `mouse.h` docstring: *"supporting X10, UTF-8, SGR, URxvt, and SGR-Pixels mouse protocols"*; `GhosttyMouse…`, `set_tracking_mode`, `set_format` |
| Bracketed paste — **mode** | **Yes** | `GHOSTTY_MODE_BRACKETED_PASTE` (mode 2004); `GHOSTTY_MODE_PASTE_EVENTS` (mode 5522) |
| Bracketed paste — **payload encoding** | **Yes** | `ghostty_terminal_paste`, `ghostty_paste_encode(..., bool bracketed, ...)`, `ghostty_paste_is_safe` (`paste.h`) |
| Keyboard/cursor modes | **Yes** | DECCKM (`GHOSTTY_KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION`), keypad, `GHOSTTY_KEY_ENCODER_OPT_KITTY_FLAGS` + `GHOSTTY_KITTY_KEY_*` (`key/encoder.h`); `CursorStyle` (bar/block/underline, DECSCUSR) in `terminal.h`/`render.h` |
| Unicode wide/grapheme/emoji/combining | **Yes** | `ghostty_unicode_codepoint_width`, `ghostty_unicode_grapheme_width`; grapheme iteration via `ghostty_grid_ref_graphemes`; mode 2027 grapheme clustering |
| Kitty graphics / sixel | Kitty **yes**, sixel **no decoder** | `kitty_graphics.h` (`ghostty_kitty_graphics_*`, placements, PNG decoder callback); **sixel** appears only as a DA feature flag `GHOSTTY_DA_FEATURE_SIXEL` — **no sixel decode/placement API observed** |
| Selection | **Yes** | `selection.h` (`ghostty_terminal_select_word/_line/_output/_all`, `…selection_adjust`, `…selection_format_*`, `selection/gesture.h`) |
| Parser/state separate from rendering | **Yes** | No renderer in the library; `RenderState` is explicitly *"Incremental render state updates for custom renderers"* |
| Callback vs polled model | **Both** | Options install C callbacks (`GhosttyTerminalWritePtyFn`, `…BellFn`, `…TitleChangedFn`, `…PwdChangedFn`, `…ClipboardWriteFn`, `…SemanticPromptFn`, `…ProgramStatusFn`, `…DesktopNotificationFn`); data is also polled via `ghostty_terminal_get` + `GHOSTTY_TERMINAL_DATA_*` |
| Memory introspection | **Yes** | `GHOSTTY_TERMINAL_DATA_MEMORY_USAGE` (relevant to `full.md` §6.4 scrollback bound) |

Module inventory on `main` is additionally `formatter.h`, `snapshot.h`, `search.h`, `io.h`,
`sys.h`, `device.h`, `color_scheme.h`, `build_info.h`, `size_report.h` (terminal snapshot
encode/restore, terminal search, byte-stream I/O callbacks, system hookup).

### 8.2 Release `v1.3.1` (the installed ABI) — utilities only

**(b) Local, high** and **(a) fetched, high** (the `v1.3.1` `vt.h` include list). Exposed symbols
(43): `ghostty_osc_*`, `ghostty_sgr_*`, `ghostty_key_encoder_*` / `ghostty_key_event_*`,
`ghostty_paste_is_safe`, `ghostty_color_rgb_get`, and `ghostty_simd_*`. **Absent at v1.3.1:** the
entire screen-state capability — no terminal handle (`ghostty_terminal_*`), no render state
(`ghostty_render_state_*`), no grid/cell/row getters (`ghostty_cell_get`, `ghostty_row_get`,
`ghostty_grid_ref_*`), and no selection, mouse, search, snapshot or kitty-graphics API. (There is no
`ghostty_screen_*` family in any version — `screen.h` defines the value handles `GhosttyCell` and
`GhosttyRow` (both `uint64_t`) plus the `ghostty_cell_*`/`ghostty_row_*` getters, which are
themselves absent at v1.3.1.) This is
the version the prior session read from `/usr/include/ghostty/vt.h` and reported correctly.

### 8.3 Rust binding safe-API completeness

**(b) Local, high** — from the extracted crate sources:

| Module | `libghostty-vt` 0.2.2 (Uzaaft) | `ghostty-vt` 0.7.2 (prabirshrestha) |
|---|---|---|
| `terminal` (Terminal, modes, cursor, pwd, title, callbacks) | yes | yes (+ `ProgressState`, continuation) |
| `render` (RenderState, Row/Cell iterators, colors, cursor) | yes | yes |
| `screen`/`grid_ref`/`tracked` | yes | yes |
| `selection` + `selection/gesture` | yes | yes |
| `mouse` (encoder, event, formats) | yes | yes |
| `key` (encoder, event, kitty flags) | yes | yes |
| `paste`, `osc`, `sgr`, `style`, `color`, `unicode`, `focus` | yes | yes |
| `kitty/graphics` (+ PNG decoder) | yes | yes |
| `formatter` | yes | yes |
| `search` | **no** | yes |
| `snapshot` (encode/restore) | **no** | yes |
| `terminfo`, `shell_integration`, `resources`, `io` | no | yes |
| `build_info`, `log`, `alloc`, `error` | yes | yes |

So **the leading binding (`libghostty-vt` 0.2.2) is missing search + snapshot**, which the newer
`ghostty-vt` 0.7.2 adds.

**Threading (high, from `libghostty-vt/src/lib.rs`):** the safe crate documents that *"all
`libghostty-vt` types are `!Send` … they are also `!Sync`"*, because the C API may use thread-local
state and is not mutex-guarded. The recommended pattern is to own the terminal on **one thread/task**
and communicate over channels. This is directly relevant to the `apps/linux` model (§10), where the
`TerminalModel` is currently `Send`-friendly safe Rust.

---

## 9. Comparison vs the ADR-003 pins

ADR-003 pins (verified in that ADR): `alacritty_terminal =0.26.0` (Apache-2.0) + `vte =0.15.0`
(Apache-2.0 OR MIT) + `portable-pty =0.9.0` (MIT). `libghostty-vt` here means **`main`** (the only
version with a terminal API); the `v1.3.1` release is strictly a utilities subset.

| Category | ADR-003 pins (today) | `libghostty-vt` (`main`) | Disposition |
|---|---|---|---|
| VT parse → screen state | `vte::ansi::Processor` → `Term` | `ghostty_terminal_*` | equivalent |
| Grid/cells/styles for rendering | `renderable_content()`, `grid().iter_from()`, `Cell` | `RenderState`, `grid_ref`, `cell_get`, `GhosttyStyle` | equivalent |
| Scrollback + viewport + **measured bound** | `Config::scrolling_history` default 10000; bound still undefined | `SCROLLBACK_MAX_BYTES`/`SCROLLBACK_MAX_LINES` + `MEMORY_USAGE` + compression | **slightly better** (byte- and memory-aware bounds) |
| Resize/reflow | `Term::resize` (reflow) | `ghostty_terminal_resize` + `RESIZE_PULL_SCROLLBACK` | equivalent |
| OSC 8 hyperlinks on cells | `Cell::hyperlink` | `ghostty_grid_ref_hyperlink_uri` | equivalent |
| OSC 52 / 133 / 7 | via `vte::osc_dispatch` | callbacks + data getters | equivalent (libghostty surfaces them as first-class callbacks) |
| OSC 7501 | not in the pins | `PROGRAM_STATUS` callback | **better (new capability)** |
| Mouse modes | `TermMode::MOUSE_*`, `SGR_MOUSE` | mouse encoder + mode-derived options | equivalent (encoder is input-side; alacritty is state-side) |
| Bracketed paste **mode** | `TermMode::BRACKETED_PASTE` | mode 2004 const | equivalent |
| Bracketed paste **payload/input** | **not provided** (TERM-002 UNSUPPORTED) | `ghostty_paste_encode(bracketed)`, `is_safe`, `terminal_paste` | **better — would close a TERM-002 gap (unverified in our flow)** |
| Keyboard/input encoding | **not provided** by the pins | full encoder (DECCKM, keypad, kitty, modifyOtherKeys) | **better (net-new)** |
| Unicode wide/grapheme/emoji | `WIDE_CHAR` + `unicode-width` | `unicode_codepoint_width` / `grapheme_width`, mode 2027 | equivalent |
| Parser/state vs rendering split | library, no renderer | library, no renderer (`RenderState` for hosts) | equivalent |
| Kitty graphics / sixel | not provided | kitty graphics yes; sixel no decoder | **better for kitty; no change for sixel** |
| Selection | `Selection`/`SelectionRange` | `selection.h` + gestures | equivalent |
| Android rendering later | `tty` gated `#[cfg(not(windows))]`, untested | Android NDK cross-compile helper; binding A maps Android targets | **unverified** (see §7) |
| License | Apache-2.0 / Apache-2.0-OR-MIT (verified) | upstream MIT (verified); **bindings: see §3.2** | upstream equivalent; **binding artifacts weaker (A ships no license file)** |
| Stability | 0.x, README "beta", exact pin mandatory | *"not yet stable and is definitely going to change"*, `main`-only, no tag for the terminal API | **equivalent-to-worse** (API in flux; no released version of the terminal API) |
| Language / integration | pure safe Rust | C/Zig FFI (see §10) | **worse for safety/build**, better for reference-product parity |
| Maturity/provenance for **this repo** | TERM-002 committed spike (32 PASS / 1 UNSUPPORTED / 0 FAIL) | **no spike for this repo; no build attempted** | **worse** |
| Windows/desktop targets | crates.io BSD/Linux/macOS/Windows | binding A: linux/macos/windows (+android); binding B: linux/macos/windows/iOS | equivalent |

`full.md` §5.4 tension (recorded at ADR-003 line 38) **remains**: §5.4 asks for "maintained
**Rust-compatible** terminal engines", while libghostty is a C/Zig library used only through a
third-party FFI binding. The reference product uses `libghostty-vt`
(`docs/research/gorex-review.md:19`), which is the sole reason it is on the table.

---

## 10. TERM-002 §8 gap-by-gap disposition

TERM-002 §8 gaps are quoted from `spikes/terminal-engine-phase2/results/REPORT.md` §8 (loaded on
disk; matches `result:ghostty#1`). "libghostty status" means **`main`** unless stated.

| TERM-002 §8 recorded gap | libghostty status | Confidence |
|---|---|---|
| `vte` not independently exercised | N/A — different stack; libghostty-vt is a set of independently-callable parsers, but **Relayable has no libghostty spike** | unverified |
| Palette → RGB UNVERIFIED | `ghostty_color_parse` / `ghostty_terminal_get(COLOR_PALETTE)` exist; whether they match the TERM-002 expectation harness is **untested** — no spike | **unverified / likely equivalent** |
| Mouse model approximates (no Press vs PressRelease; DECSET 9/1000 collapse) | `main` exposes an explicit mouse encoder + tracking/format options; the exact `Press`/`PressRelease` distinction is **not verified against the harness** | **unverified** |
| Bracketed-paste payload/input semantics untested | **Potentially closed** by `ghostty_paste_encode`, `ghostty_paste_is_safe`, `ghostty_terminal_paste` (mode 2004 + payload encoding are both present) | **better (unverified in our flow)** |
| Selection golden-value tested only | `selection.h` provides the model; no libghostty spike exists | **unverified** |
| No Android rendering / no GPU-renderer path | libghostty-vt is deliberately headless (host renders) → same as the requirement; **Android has an NDK helper but no tested rendering path** | **equivalent at best; Android unverified** |
| §5.4 Android-rendering bullet untested | see §7 | **unverified** |
| `vim`/`htop` absent (only `tmux` covered) | a **spike** gap, not a library property; no libghostty spike exists | **unverified** |
| Wire protocol untouched | unchanged by any engine choice | **unverified** |

**Differential worth recording:** the **input side** — bracketed-paste payload encoding and
keyboard/cursor-mode encoding — is a category the current pins provide **nothing** for and
libghostty-vt provides a documented API for. TERM-002 did not measure that side at all, and no Rust
call path exists in this repo. That is the single strongest technical argument for libghostty over
the current stack; it is also **unverified by any run**.

---

## 11. Integration cost against `apps/linux/src/term/mod.rs`

**(b) Local, high.** Current module (498 lines) is headless safe Rust:

- Public contract: `TerminalModel::{new, process(&[u8]), resize, snapshot, snapshot_at, history_len,
  size}` plus `ScreenSnapshot`/`ScreenCell`/`CellStyle`/`TermColor` (GPUI-free).
- It uses `VoidListener`, so **no OSC/selection/mouse/hyperlink exposure** today; `Config` keeps the
  engine default history (10 000).

**A libghostty swap would require (inference, high):**

1. **A new `unsafe` FFI boundary.** Today the module has **zero** `unsafe` (pure safe
   `alacritty_terminal`). A C library reached either through a hand-written `bindgen` layer or a
   third-party binding adds that boundary. The third-party safe wrapper removes most direct `unsafe`
   from *our* code, but `ghostty-vt-sys`/`libghostty-vt-sys` are themselves `unsafe` FFI, and the
   safe crate documents that **all its types are `!Send`/`!Sync`** — a real constraint for the
   current `TerminalModel` usage.
2. **Handle/lifetime/error mapping.** New→free lifetimes, `GhosttyResult` (`GHOSTTY_SUCCESS = 0`,
   out-of-memory = −1, invalid-value = −2) → `Result`, allocator ownership. Straightforward but new.
3. **A host-owned cell/style struct survives.** `ScreenSnapshot`/`ScreenCell`/`CellStyle`/`TermColor`
   are already engine-neutral, so the **rendering/painting layer, palette resolution, PTY ownership
   (server-side `portable-pty`), and protocol framing do not need to change**. That is a genuine plus.
4. **Callbacks replace `VoidListener`.** libghostty uses C function-pointer options for
   pty-write/bell/title/pwd/clipboard/semantic-prompt/program-status. OSC-driven client behaviour
   (title, clipboard, cwd, prompt marks) that `crate::api` currently handles **outside** the model
   would be re-plumbed through these callbacks.
5. **Threading model.** The C API is not thread-safe; the safe crate is `!Send`. Relayable would own
   the terminal on one thread/task with channel hand-off. This is a design decision, not a drop-in.
6. **New host-side work if `alacritty_terminal` is dropped.** Selection, mouse-reporting state,
   bracketed-paste mode, and OSC 8 URI surfacing each need either the (now-present) libghostty APIs
   or Relayable code — precisely the areas TERM-002 *did* evidence for `alacritty_terminal`
   (OSC 8 replay, mouse modes, selection golden value).
7. **Build cost** (§6): Zig 0.16.x + a source fetch, on every clean build.

**GPUI precedent (a, high).** `behzade/gpui-libghostty` shows the shape of a working integration
(vendored Ghostty, Zig 0.16, `libghostty-internal.a`, a `Terminal::spawn` entity, wayland surface,
live theming). It confirms feasibility on Linux/Wayland and confirms **Zig is not avoided**.

**Bottom line:** the swap adds an FFI/`unsafe` boundary, a Zig build dependency, a `!Send` threading
constraint, and a re-verify-every-bump tax, while leaving the host owning the **same** client
surface (rendering/widget, palette, PTY, protocol, and most selection/mouse/hyperlink surfacing) —
minus input encoding, plus libghostty's paste/key encoders. `full.md` §5.4's "maintained
Rust-compatible terminal engine" wording stays in tension exactly as ADR-003:38 records.

---

## 12. ADR-003 Option E and trigger #10 — adjudication

**Option E, verbatim** (`docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md`, Option E):

> ### Option E — Bind `libghostty-vt` (C/Zig), matching the reference product
> Considered and **not chosen now**. … no license text, Rust binding availability, or build evidence
> for libghostty was fetched in this evidence set. **UNVERIFIED** — outstanding check: fetch the
> Ghostty/libghostty license and any maintained Rust binding, and spike a build. This remains a
> legitimate reconsideration path (see below).

**Trigger #10, verbatim** (`## What Evidence Would Cause Reconsideration`):

> 10. **`libghostty-vt` turns out to be the right boundary** (a verified license, a maintained Rust
>     binding, and a passing build/spike) — this would reopen Option E and the reference-product
>     alignment.

| Trigger-#10 condition | Status now | Basis |
|---|---|---|
| 1. **A verified license** | **VERIFIED** for upstream Ghostty/libghostty-vt (**MIT**, fetched `LICENSE`, sha256 `3862…e6f1`; no `NOTICE`). **CAVEAT:** the leading Rust crate (`libghostty-vt` 0.2.2) declares `MIT OR Apache-2.0` but **ships no license file in its published tarball**; the second binding bundles its licenses (§3.2). | (a)+(b) |
| 2. **A maintained Rust binding** | **VERIFIED-WITH-CAVEATS.** `Uzaaft/libghostty-rs` → `libghostty-vt` 0.2.2 (2026-09-28), 394★, 13 open issues, active (last push 2026-10-15); rc 1.90/edition 2024; safe API covers terminal/render/selection/mouse/key/paste/osc/kitty-graphics. A **second** binding (`ghostty-vt` 0.7.2, 2026-10-14) adds search/snapshot/terminfo. **Caveats:** leading crate's missing license file; both need Zig 0.16.x; safe types are `!Send`/`!Sync`. | (a)+(b) |
| 3. **A passing build/spike** | **STILL UNVERIFIED / NOT MET.** This was a documentation/registry task; **no build, link or run was performed**. Zig is absent on this host and is **not** installed here. The default `build.rs` paths require Zig 0.16.x and (by default) a network source fetch. | (b) local env |

**Adjudication:** **trigger #10 is NOT satisfied on present evidence.** One of its three
sub-conditions (license of upstream) is now closed; the second is substantially met but with a
material license-file caveat and a hard build prerequisite; the third (build/spike) is untouched and
is the gate that actually decides. **ADR-003 should not be reopened on this alone.** Any reopening
should be gated on a real TERM-003 spike (which will itself be blocked until a Zig-provisioning
decision is made).

---

## 13. Recommendation

**DEFER / PARTIAL — keep the ADR-003 pins; do not amend the ADR yet; do not adopt a libghostty
engine in this phase.** A **research-only PARTIAL**: the ecosystem is real and the input-side
capabilities are genuinely valuable, but adoption is blocked by an unresolved toolchain decision and
an unreleased, explicitly unstable API.

**Evidence for adopting (partial):**

- Upstream is **MIT** (verified), headless by design (host renders — matches §5.4's parser/state-vs-
  rendering requirement), zero-dependency C/Zig.
- **Input-side capabilities the current pins lack entirely:** key/cursor-mode encoding (DECCKM,
  keypad, kitty keyboard, modifyOtherKeys), bracketed-paste payload encoding + safety validation,
  mouse encoders (X10/UTF-8/SGR/URxvt/SGR-Pixels), OSC 7501 program status.
- `main` has a complete, documented terminal API with real symbols (`terminal.h`, `render.h`,
  `selection.h`, `screen.h`, `grid_ref.h`, `kitty_graphics.h`, `search.h`, `snapshot.h`).
- A **maintained Rust binding** exists (394★, active) and a second, newer one bundles clean licenses
  and adds search/snapshot.
- **`behzade/gpui-libghostty`** demonstrates that a GPUI + libghostty integration is buildable
  (artifact-reading inference — **no build was performed here**) **(Linux/Wayland)**.
- Reference-product alignment (`gorex-review.md:19`) and byte/-memory-aware scrollback bounds
  (`SCROLLBACK_MAX_BYTES`, `MEMORY_USAGE`) help `full.md` §6.4.

**Evidence against adopting now (decisive):**

1. **The release the host runs (v1.3.1) has no terminal/grid/scrollback/selection/mouse API** — it
   is a utilities-only library (43 symbols). The full API exists only on `main`, i.e. in **no tag**.
2. Upstream declares the API *"not yet stable and is definitely going to change"*.
3. **Zig 0.16.x is a build BLOCKER** under the no-global-install rule, and the `pkg-config`/system-lib
   escape hatch is unusable here (installed lib is API-incompatible).
4. The **leading Rust crate ships no license file** in its published tarball — the exact pattern
   ADR-003 rejected for `Glass-HQ/gpui`.
5. `alacritty_terminal 0.26.0` already has **committed repo-local spike evidence**
   (32 PASS / 1 UNSUPPORTED / 0 FAIL) and OSC 8/mouse/selection coverage that no libghostty spike has
   matched, and the swap would *add* `unsafe`/build/threading cost while leaving the host owning the
   same client logic.
6. **Android** is unverified (NDK helper exists; no tested rendering path).

**If adoption is ever pursued, it should be *additive*, not a replacement:** bind libghostty for the
**input-encoding gap** (key/paste/mouse encoders) while keeping `alacritty_terminal` for screen state
— but that requires the same Zig build dependency, so it is not free.

---

## 14. Open questions / unverified

1. **Whether the terminal API will ship in a tagged release** and under what ABI/soname; today it is
   `main`-only (`1.3.2-dev`). **UNVERIFIED.**
2. **Which libghostty-vt version/ABI the two bindings actually require** versus arbitrary installed
   libraries; the READMEs warn they are not compatible with "arbitrary installed C API revisions."
   **UNVERIFIED** beyond that warning.
3. **Exact dependency/license surface of the bindings** (transitive crates, whether any needs a
   `NOTICE`); only the two crates' own license files were inspected. **UNVERIFIED.**
4. **Android**: whether either Rust binding actually cross-compiles and runs on an Android ABI;
   whether upstream's `pkg/android-ndk` helper is used by a shipped product. **UNVERIFIED.** No
   Android build was attempted (forbidden).
5. **Runtime behavior versus the TERM-002 check matrix** (OSC 8, mouse modes, resize/reflow,
   scrollback bound, Unicode/wide/emoji, chunk-boundary invariance, `vim`/`htop`, `tmux`): entirely
   **unverified** — no libghostty spike exists for this repo.
6. **Performance/memory** of libghostty-vt `main` on this host: **UNVERIFIED** (no measurement; a
   benchmark would need a build, which is out of scope).
7. **Whether `ghostty-vt` (prabirshrestha) is intended as an upstream successor** to
   `libghostty-vt` or is a personal fork; it is a 1★ monorepo adapted from Uzaaft 0.2.2.
   **UNVERIFIED.**
8. **Upstream contributor count** was not resolved (GitHub API rate limit); not material to the
   decision.
9. **Whether upstream `main`'s OSC 8 URI / palette→RGB values match the proxy/reference behavior
   matrix**: **not verified.**

---

## 15. Concrete evidence that would trigger ADR-003 reopening

Each item is a sourced flip-condition. All are **research/experiment** actions, not commitments made
here.

1. **A tagged Ghostty release that ships the full terminal API** (e.g. `1.4.0` with `terminal.h`,
   `render.h`, `selection.h`, `screen.h` and the symbol set in §8.1), making the C API a versioned,
   pinned artifact rather than a `main`-only development. → then Option E becomes costable against a
   released ABI.
2. **A Zig-provisioning decision** that satisfies the no-global-install rule (a `mise`-pinned Zig
   0.16.x accepted by the lead, plus `GHOSTTY_SOURCE_DIR`/`GHOSTTY_ZIG_SYSTEM_DIR` hermetic
   provisioning). Without this, no spike can run.
3. **A license-file fix in the leading crate** (a `LICENSE`/`LICENSE-MIT`/`LICENSE-APACHE` added to
   the published `libghostty-vt`/`libghostty-vt-sys` tarballs), closing the §3.2 caveat — or explicit
   lead acceptance of the `MIT OR Apache-2.0` declaration with the repo's MIT text as the notice.
4. **A passing TERM-003 spike** that runs the TERM-002 check matrix against libghostty-vt (OSC 8,
   mouse modes, resize/reflow, scrollback bound, Unicode/wide/emoji, chunk-boundary invariance,
   `vim`/`htop`, `tmux`) with a **measured** memory number for §6.4.
5. **Android evidence**: a documented, tested Android ABI build of the chosen binding plus a
   rendering path (upstream or a maintained third party), which would speak to DEC-010 without
   starting Android work.
6. **Any measured defect** in `alacritty_terminal 0.26.0` on the §5.4 bullets (ADR-003 trigger #1/#2)
   that libghostty-vt is demonstrably better at, *verified by a run*.

---

## 16. Proposed dependency/ordering for adoption (if a future phase proceeds)

Research output only — **nothing here is approved work**; it is a proposed ordering for the lead to
schedule if trigger conditions are met.

1. **Decision gate (lead):** choose the Zig provisioning path (`mise` pin + hermetic Ghostty source)
   or keep deferring. No code until this is decided. `docs/architecture/next-phase-plan.md` already
   says "if libghostty needs Zig, report it — do not install; propose a `mise` pin or defer."
2. **Boundary spike (TERM-003):** a standalone `spikes/**` crate (exempt from file-size caps) that
   links one binding (prefer `libghostty-vt` with a **vetted fork/pin or a vendored build**) and
   runs the TERM-002 check matrix. Record the real Zig version, source pin, and build time.
3. **Pin & license audit:** record the exact crate version + checksum + source commit, and complete a
   license-file audit for the crate and its transitive crates (the §3.2 caveat must be resolved or
   explicitly accepted).
4. **Only then** consider an ADR-003 amendment (Option E) — as a **lead-owned** change, gated on the
   spike, and only if it beats `alacritty_terminal` on the verified matrix.
5. **Possible additive adoption:** reuse libghostty's key/paste/mouse encoders in
   `crate::input`/protocol without replacing the engine — but only after the build blocker is
   resolved, since it carries the same Zig cost.

---

## 17. Plain-language answers to the task's mandated questions

- **Does the network work?** Yes — §0/§18. `curl`/`wget` reached GitHub API, crates.io, ghostty.org
  and `raw.githubusercontent.com`.
- **What is the real Rust binding?** `Uzaaft/libghostty-rs` → crates **`libghostty-vt`** +
  **`libghostty-vt-sys`** (the prior "browser tab title" lead is **confirmed**). A second binding is
  `prabirshrestha/gpui-cn` → **`ghostty-vt`** + **`ghostty-vt-sys`**. The crate **`libghostty`** is an
  unrelated **yanked placeholder**.
- **License?** Upstream **MIT** (verified, no `NOTICE`). Bindings: `libghostty-vt` declares
  `MIT OR Apache-2.0` but **ships no license file** in the crate; `ghostty-vt` is `Apache-2.0` and
  bundles `LICENSE` + `LICENSE-GHOSTTY` (MIT).
- **Zig blocker?** **Yes, a blocker.** Every maintained binding's default build needs **Zig 0.16.x**;
  Zig is absent here; the `pkg-config` system-lib path is unusable because the installed library is
  API-incompatible. A `mise` pin would remove the *global-install* objection but not the network,
  reproducibility, or consumer-path issues.
- **Android?** Upstream has an **Android NDK cross-compilation helper** and Uzaaft maps Android
  targets; the README's compatibility list omits Android; **no tested Android rendering path** was
  found. DEC-010 unaffected.
- **Trigger #10?** **Not met.** License → VERIFIED; maintained binding → VERIFIED-WITH-CAVEATS;
  passing build/spike → STILL UNVERIFIED. Recommend **DEFER/PARTIAL**.
- **No REX claim.** No wire/binary compatibility with the reference product is claimed anywhere.

---

## 18. Search trail (URLs, status codes, retrieval date)

All fetches **2026-10-16 (UTC)**, read-only, User-Agent `relayable-research/0.1`. Full raw log:
`/tmp/libghostty-evidence/FETCH_LOG.tsv` (111 entries). Selected entries (label → HTTP status):

**Identity / search (GitHub API, `api.github.com`):**

| URL | Status |
|---|---|
| `api.github.com/repos/ghostty-org/ghostty` | 200 |
| `api.github.com/repos/Uzaaft/libghostty-rs` | 200 |
| `api.github.com/search/repositories?q=libghostty&per_page=50` | 200 (280 results) |
| `api.github.com/search/repositories?q=libghostty-rs&per_page=50` | 200 (3 results) |
| `api.github.com/search/repositories?q=libghostty-vt&per_page=50` | 200 (91 results) |
| `api.github.com/repos/ghostty-org/ghostty/releases?per_page=20` | 200 (1 result: `tip`) |
| `api.github.com/repos/ghostty-org/ghostty/tags?per_page=30` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/commits?sha=main&per_page=10` | 200 |
| `api.github.com/repos/Uzaaft/libghostty-rs/contents/` | 200 |
| `api.github.com/repos/Uzaaft/libghostty-rs/commits?sha=master&per_page=10` | 200 |
| `api.github.com/repos/Uzaaft/libghostty-rs/releases?per_page=20` | 200 |
| `api.github.com/repos/Uzaaft/libghostty-rs/tags?per_page=20` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/contributors?per_page=1` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/languages` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/contents/include/ghostty/vt?ref=main` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/contents/include/ghostty/vt?ref=v1.3.1` | 200 |
| `api.github.com/repos/ghostty-org/ghostty/git/ref/tags/v1.{0.0,0.1,1.0,1.1,1.2,1.3,2.0,2.1,2.2,2.3,3.0,3.1}` | 200 (12) |
| `api.github.com/rate_limit` | 200 (limit 60; later exhausted — see §19) |

**crates.io API:**

| URL | Status |
|---|---|
| `crates.io/api/v1/crates/libghostty` | 200 (yanked placeholder) |
| `crates.io/api/v1/crates/libghostty-rs` | **404** (`libghostty-rs` is not a crate name) |
| `crates.io/api/v1/crates/libghostty-vt` + `/versions` + `/owners` + `.../0.2.2/dependencies` | 200 |
| `crates.io/api/v1/crates/libghostty-vt-sys` | 200 |
| `crates.io/api/v1/crates/ghostty-vt` + `/versions` + `/owners` + `.../0.7.2+…/dependencies` | 200 |
| `crates.io/api/v1/crates/ghostty-vt-sys` | 200 |
| `crates.io/api/v1/crates/ghostty` | **404** (no such crate) |
| `crates.io/api/v1/crates/{libghostty-vt/0.2.2,ghostty-vt/0.7.2+…,libghostty-vt-sys/0.2.2,ghostty-vt-sys/0.7.2+…,libghostty/0.1.0}/download` | 200 (`.crate` tarballs) |

**raw.githubusercontent.com (source/docs):**

| URL | Status |
|---|---|
| `.../ghostty-org/ghostty/main/LICENSE` | 200 |
| `.../ghostty-org/ghostty/main/README.md` | 200 |
| `.../ghostty-org/ghostty/main/build.zig.zon` | 200 |
| `.../ghostty-org/ghostty/main/build.zig` | 200 |
| `.../ghostty-org/ghostty/main/vt.h` and `main/include/ghostty/vt/*.h` (32 headers) | 200 |
| `.../ghostty-org/ghostty/v1.3.1/include/ghostty/vt.h` | 200 |
| `.../ghostty-org/ghostty/main/pkg/android-ndk/build.zig` | 200 |
| `.../ghostty-org/ghostty/main/HACKING.md` | 200 |
| `.../ghostty-org/ghostty/main/{NOTICE,NOTICE.md,COPYING}` | **404 ×3** |
| `.../Uzaaft/libghostty-rs/master/LICENSE` | 200 |
| `.../Uzaaft/libghostty-rs/master/{LICENSE-MIT,LICENSE-APACHE,README.md}` | 200/200/200 (README) — `LICENSE-MIT`/`LICENSE-APACHE` **404** |
| `.../Uzaaft/awesome-libghostty/master/README.md` | 200 |
| `.../Uzaaft/libghostty-rs/main/{LICENSE,README}` (wrong branch `main`) | **404** (default branch is `master`) |

**Non-API sources (no rate limit):**

| URL | Status |
|---|---|
| `github.com/ghostty-org/ghostty/commits/v1.{2.2,2.3,3.0,3.1}.atom` | 200 (tag dates) |
| `github.com/prabirshrestha/gpui-cn` (HTML) | 200 |
| `codeload.github.com/behzade/gpui-libghostty/tar.gz/refs/heads/main` | 200 |
| `ghostty.org/docs` | 200 |
| `ghostty.org/docs/install/build` | 200 |
| `libghostty.tip.ghostty.org/` (Doxygen) | 200 |
| `wget https://api.github.com/repos/ghostty-org/ghostty` (probe) | ok |

**Could not fetch / negative:** GitHub API rate limit (60/hr) was exhausted during the tag-date pass
(remaining 0); the remaining four tag dates came from the rate-free `commits/<tag>.atom` feed
(marked medium). The `prabirshrestha/gpui-cn` star count came from the HTML page. No proxy, install,
or config change was used.

---

## 19. Validation record

- **Output written:** the only file this task wrote is `docs/research/libghostty.md`. (A sibling
  task, RESEARCH-001, legitimately created `docs/research/gpui-kit.md` in the same worktree; this task
  did not touch it.) No `docs/adr/**`, `docs/agent-loop/**`, `apps/**`, `Cargo.toml`/`Cargo.lock`, or
  other file was touched by this task.
- **Raw evidence bundle (not in the repo):** `/tmp/libghostty-evidence/` (outside the worktree). All
  fetched bodies, extracted `.crate` trees, and `FETCH_LOG.tsv` live there. Selected checksums:
  `libghostty-vt-0.2.2.crate` = `16c7287a8dc35c3436224b39358af81ab9ea54bafa0f7ea2e068bd2a47bdcb2d`
  (matches the crates.io `version.checksum`); `ghostty-vt-0.7.2.crate` =
  `c056e7b0393b6516369c62508cdf243fc3c9b3acb711f3dd14f0b17eb9660915` (matches). Upstream Ghostty
  `LICENSE` = `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1`.
- **Build/run claim:** **none.** No `cargo`, `zig`, or build/link/run command was executed. Every
  build conclusion is derived from fetched `build.rs`/`build.zig.zon` plus the local absence of Zig
  and the local library's symbol set, and is labelled "unverified-by-build".
- **Local host facts re-verified:** `ghostty 1.3.1-2`; `libghostty-vt.so.0.1.0`; pkg-config
  `Version: 0.1.0`; installed `vt.h` include list = utilities only; `nm -D` → 43 `ghostty_*`
  symbols; `command -v zig` → not found; `mise which zig` → not a mise bin; rustc/cargo 1.98.1.
- **Branch base caveat:** this worktree is `research/gpui-kit-libghostty` at base `f4d59f8dd813…`,
  which predates the plan/TASKS ledger that authorizes RESEARCH-002. Citations in this file to
  `docs/architecture/next-phase-plan.md` and `docs/agent-loop/TASKS.md` (RESEARCH-002) therefore
  resolve on `master` (`ba92c517…`, where this file is destined), **not** on the current branch base.
  No file was copied into the branch to work around this.
- **Provenance of inference:** all `(inference)`/`(c)` statements are marked; §12's trigger
  adjudication quotes ADR-003/TERM-002 verbatim and separates verified from unverified.
