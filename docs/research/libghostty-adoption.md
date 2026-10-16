# libghostty adoption evidence — TERM-003 stage A

Status: **research artifact, evidence only.** This file answers stage A of task `TERM-003`
(`docs/agent-loop/TASKS.md:40`) under the user directive **DEC-021** (`docs/agent-loop/DECISIONS.md:444`):
*"adopt libghostty"*, with explicit permission to clone upstream for research. It turns
`docs/research/libghostty.md` (RESEARCH-002, integrated INT-021) from a `DEFER / PARTIAL` recommendation
into either an **executable adoption path with evidence** or a **precise blocker list**.

It does **not**: change any pin, edit any ADR, edit any code, install a toolchain, or make any REX
wire/binary compatibility claim. It names what an ADR-003 amendment *would* have to revisit; it does not
draft one (`docs/adr/**` is lead-owned).

**Retrieval window:** `2026-10-16T16:51Z`–`2026-10-16T17:10Z` (UTC). All upstream study below is from
**local Git clones** fetched in that window, plus live HTTP probes; each claim carries the ref/file:line
or URL it comes from.

---

## 0. Bottom line (read this first)

**An executable adoption path exists, and it is materially better-evidenced than RESEARCH-002 could
establish** (RESEARCH-002 had to infer from served HTML and published tarballs; this pass read the real
repository across its whole tag/branch history). Three findings change the shape of the problem:

1. **The terminal core is in the released tag** — the *C header* at `v1.3.1` is utilities-only, but the
   same tag ships the terminal engine as a **Zig module** (`src/lib_vt.zig:59`
   `pub const Terminal = terminal.Terminal;`). RESEARCH-002's "the release has no terminal API" is true
   only at the **C boundary**. Consuming the tagged engine therefore means writing our own thin Zig build
   + C ABI (exactly what two shipped projects do — §1.4, §7.3), or consuming `main`'s C API.
2. **The C-side build cost is small and can be made offline.** A lib-vt-only build
   (`zig build -Demit-lib-vt -Dapp-runtime=none`) pulls exactly **one** non-lazy external Zig package
   (`uucode`) plus the in-tree C shims; the SIMD dependency vendors its C++ in-tree with **no** external
   URL; every heavy dependency (GTK, Wayland, fonts, libxev, vaxis, z2d) is `.lazy = true` and is not
   built. Upstream also supports `--system <dir>` for a fully network-free build (§3.4).
3. **A pinned, checksummed, immutable source tarball exists per upstream commit**, produced by upstream's
   own CI: `https://tip.files.ghostty.org/<40-char-sha>/libghostty-vt-source.tar.gz`. I downloaded one and
   its sha256 matched a third-party pin byte-for-byte (§3.4). Caveat: it does **not** exist for every
   commit (§3.4, observed 3× HTTP 200 / 3× HTTP 404 out of six probes).

**What each question resolves to:**

| # | Question | Answer |
|---|---|---|
| 1 | Released vs `main` C API | **Split, and worse than "main-only".** Newest tag `v1.3.1` (2026-03-13): 12 headers, 50 declared `ghostty_*` symbols, **no** `terminal.h`. `main` @ `949149d` (2026-10-16): 35 headers, 207 declared symbols, full terminal API (36 vs 13 files under `include/` including `module.modulemap`). The rolling `tip` tag carries main's API but is a **moving prerelease tag** with no version contract. `libghostty-vt`'s own semver is **`0.1.0`** at `v1.3.1` and **`0.1.0-dev`** on `main` — the terminal API has no version of its own at all. |
| 2 | Rust binding | Two real candidates, both **need Zig 0.16.x**, both bind a **pinned Ghostty commit**, neither ships a prebuilt binary. `libghostty-vt` 0.2.2 is the maintained one and is a complete safe wrapper for our needs (feed bytes, grid/cells, resize+reflow, scrollback bounds, cursor, title/OSC, OSC 8 hyperlink URI, callbacks for bell/pty-write/clipboard/notification/progress). It ships **no license file** in the published tarball (confirmed independently, §5.2). |
| 3 | Build on this host | `minimum_zig_version = "0.16.0"` (`build.zig.zon:6` on `main`), hard-enforced at `build.zig:12-14`. **`mise` can supply exactly `0.16.0`** (`mise ls-remote zig` → `0.16.0`; `mise registry` → `zig core:zig`). Static `.a` (default) or shared `.so` with soname major `0`; C ABI. Time/disk: **unmeasured — inferred only** (§3.5). Needs two approvals (§3.6). |
| 4 | Adoption shape | **Not contained to `term/**`.** `apps/linux/src/api/osc.rs:1` uses `vte::Perform` directly, so the `vte` pin is load-bearing outside the engine. Files that change are named in §4. The public `ScreenSnapshot` shape survives, so the GPUI painter and theme do not. |
| 5 | Portability | Upstream's lib-vt build path has **explicit Android support** (`src/build/GhosttyLibVt.zig:273-278`, 16 KB pages + NDK paths) — stronger than RESEARCH-002's "helper exists". `ghostty-vt` (binding B) has **no Android target**; `libghostty-vt` (A) maps two Android triples. `relayable-model`/`relayable-protocol` are untouched either way. |
| 6 | Licence | Upstream is **MIT** with **no `NOTICE`**. Relayable must ship the MIT text + copyright line (`Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors`) in any distribution. Static-vs-dynamic changes *nothing* about MIT's notice obligation. The binding's missing licence file is a **real but severable** defect (§6.3) — severable because the *upstream source path* needs no third-party licence at all. |
| 7 | Stage B | Smallest verifiable spike defined in §7: build lib-vt at one pinned commit, bind it through the sys crate behind the existing `TerminalModel` interface, and re-run the eight existing `apps/linux/src/term/tests.rs` assertions plus a named subset of the TERM-002 matrix. Fallback in §7.6. |

**Decision the lead must make now:** approve the two stage-A→B prerequisites in §3.6, and choose the
adoption shape in §4.4. Until then ADR-003's pins stay authoritative; **no evidence collected here shows
the current stack failing**, so adoption is not forced by a defect — it is a reference-product-alignment
and capability decision (`full.md` §5.4, `docs/research/gorex-review.md:19`).

---

## 1. Corpus studied (exact refs)

All three clones live **outside the Relayable tree**, under `/home/sulthonuladib/work/vendor/`, per the
DEC-021 bound. Nothing was copied into the repository; `.gitignore` was not touched (`git status` in the
Relayable checkout is unchanged: only the pre-existing `docs/agent-loop/{DECISIONS,TASKS}.md` edits that
pre-date this task, plus the two files this task writes).

| Clone | Remote | Ref studied | Commit | Date (UTC) | Size |
|---|---|---|---|---|---|
| `/home/sulthonuladib/work/vendor/ghostty` | `github.com/ghostty-org/ghostty.git` | `HEAD` (`main`) | `949149d5791a0eb08412ad05cd8fd484ad134481` | 2026-10-16T16:19:44Z | 201 MB |
| ″ | ″ | tag `v1.3.1` (newest release tag) | `22efb0be2bbea73e5339f5426fa3b20edabcaa11` | 2026-03-13T09:03:38−07:00 | — |
| ″ | ″ | tag `tip` (rolling prerelease) | `877dda73124de3eaf8ff94672ffa2412c63d6d3e` | 2026-10-15T21:55:50Z | — |
| `/home/sulthonuladib/work/vendor/libghostty-rs` | `github.com/Uzaaft/libghostty-rs.git` | `HEAD` (`master`) | `d2036baeb605d31d09a9e2ba6d0c923e04c34d54` | 2026-10-09T16:16:44+02:00 | 3.1 MB |
| ″ | ″ | tag `v0.2.2` (published release) | `a3a967698be27fde0077f8559ca36f852f5885ac` | 2026-09-28T14:01:36+02:00 | — |
| `/home/sulthonuladib/work/vendor/gpui-cn` | `github.com/prabirshrestha/gpui-cn.git` | `HEAD`, tag `v0.7.2` | `f066027b77d3dd2e2ba4ac343529cebcd2626fdd` | 2026-10-14T15:33:35−07:00 | 6.2 MB |

Cloning commands (re-derivable):

```shell-session
$ mkdir -p /home/sulthonuladib/work/vendor && cd /home/sulthonuladib/work/vendor
$ git clone https://github.com/ghostty-org/ghostty.git ghostty
$ git clone https://github.com/Uzaaft/libghostty-rs.git libghostty-rs
$ git clone https://github.com/prabirshrestha/gpui-cn.git gpui-cn
```

Raw fetched artifacts are under `/tmp/relayable-research/libghostty-evidence/` (outside the repo).

### 1.1 Evidence-class labels (used on every claim)

`observed-by-running` (a command on this host) · `read-in-source` (a file:line in one of the clones above)
· `verified-official-docs` (a live fetch of upstream's own repo/docs) · `inferred` · `unknown`.
Confidence: **high** / **medium** / **low** / **UNVERIFIED** / **not observed**.

### 1.2 Corrections to RESEARCH-002 (`docs/research/libghostty.md`)

RESEARCH-002 was accurate on almost everything it claimed, and its §3.2 licence finding, §5.3
release/`main` split, §6.2/§6.3 `build.rs` behaviour and §8.3 `!Send`/`!Sync` claim are all
**independently reproduced** here. Four things are wrong, stale, or incomplete:

| # | RESEARCH-002 claim | Correction | Anchor |
|---|---|---|---|
| C1 | §5.1: *"Upstream ships **tags**, not GitHub releases (the only release object is the 2022 `tip` nightly)."* | **Stale.** The `tip` **release object is live and rolling**: `name: 'Ghostty Tip ("Nightly")'`, `prerelease: true`, `tag_name: tip`, `target_commitish: ${{ github.sha }}` — it is re-created on every `main` push and carries real assets. `git rev-parse tip` → `877dda7…`, dated **2026-10-15**, not 2022. Assets include `libghostty-vt-source.tar.gz`, `ghostty-vt.wasm`, `ghostty-vt-small.wasm`, `ghostty-vt.xcframework.zip`, plus the macOS app zips. | `.github/workflows/release-tip.yml` (all jobs, `tag_name: tip`); `git for-each-ref --sort=-creatordate refs/tags` |
| C2 | §5.3/§13: *"The release the host runs (v1.3.1) has no terminal/grid/scrollback/selection/mouse API — it is a utilities-only library."* | **Correct at the C boundary, incomplete overall.** The same tag ships the engine as a **Zig module**: `src/lib_vt.zig` at `v1.3.1` exports `Terminal` (:59), `RenderState` (:52), `Screen`/`ScreenSet` (:53-54), `Selection` (:55), `Style` (:58), `Parser` (:49), `Pin` (:50), plus `search` (:34), `formatter` (:28), `kitty` (:30), `page`/`Cell` (:32, :42). Only the flat C header is missing. (`v1.2.3` has **no** `src/lib_vt.zig` — the module form arrived between `v1.2.3` and `v1.3.1`.) | `git show v1.3.1:src/lib_vt.zig`; `git show v1.2.3:src/lib_vt.zig` → *"fatal: path 'src/lib_vt.zig' exists on disk, but not in 'v1.2.3'"* |
| C3 | §6.1 implies Zig `0.16.0` is *the* upstream requirement. | **Split by ref.** `v1.3.1` requires **`0.15.2`** (`build.zig.zon:6` at that tag). `0.16.0` is a **`main`-only** requirement. Relevant because a "use the tagged engine as a Zig module" path (§7.3) would need 0.15.2, not 0.16.0. | `git show v1.3.1:build.zig.zon` → `.minimum_zig_version = "0.15.2"`; `git show HEAD:build.zig.zon` → `"0.16.0"` |
| C4 | §5.3/§8.2 treat the `Version: 0.1.0` in the installed `.pc` as evidence of an old build only. | **Add the mechanism, which matters for ABI.** `v1.3.1` **hardcodes** the lib version to `0.1.0` (`src/build/GhosttyLibVt.zig:30` and `:57`, `std.SemanticVersion{ .major = 0, .minor = 1, .patch = 0 }`; the `.pc` template at `:110` is the literal string `Version: 0.1.0`). `main` uses a **separate** `const lib_version = "0.1.0-dev";` (`build.zig:11`) threaded through `Config.init(b, appVersion, lib_version)`. So the full-terminal API on `main` is versioned **`0.1.0-dev`** — the same `0.x` major as the utilities-only release. Consequence: the `major` component of the versioned library is `0` in both; the host's v1.3.1 installation is `libghostty-vt.so.0 → libghostty-vt.so.0.1.0` (**observed**). The `libghostty-vt.so.0` soname therefore does **not** separate the utilities-only ABI from a full-terminal ABI. | `git show v1.3.1:src/build/GhosttyLibVt.zig`; `build.zig:11`; `src/build/Config.zig:343-353`; `ls -l /usr/lib/libghostty-vt*` |

**One upstream doc defect found (not RESEARCH-002's fault):** `PACKAGING.md` instructs maintainers to
*"check the `required_zig` constant in `build.zig`"*. That constant **does not exist** on `main`
(`grep -n required_zig build.zig` → no output). The real mechanism is
`build.zig.zon.minimum_zig_version`, read at `build.zig:10` and enforced by
`buildpkg.requireZig(...)` at `build.zig:12-14`. Reported as an upstream inconsistency, not acted on.

### 1.3 Local host facts re-verified this session (all `observed-by-running`)

```shell-session
$ pacman -Q ghostty                        # ghostty 1.3.1-2
$ ls -l /usr/lib/libghostty-vt*
lrwxrwxrwx  libghostty-vt.so       -> libghostty-vt.so.0
lrwxrwxrwx  libghostty-vt.so.0     -> libghostty-vt.so.0.1.0
-rwxr-xr-x  libghostty-vt.so.0.1.0        842192 bytes
$ cat /usr/share/pkgconfig/libghostty-vt.pc | grep Version    # Version: 0.1.0
$ command -v zig                                             # (no output — Zig absent)
$ mise --version                                             # 2026.10.10 linux-x64
$ mise registry | grep '^zig'                                # zig  core:zig
$ mise ls-remote zig | grep -E '^0\.1[3-9]'
0.13.0 / 0.14.0 / 0.14.1 / 0.15.1 / 0.15.2 / 0.16.0 / 0.17.0
$ git --version                                              # /usr/bin/git
$ df -h /home | tail -1                                      # 235 G available
```

One discrepancy with `AGENTS.md`'s environment table, recorded for the lead (not repaired — I do not own
that file): the table lists the Android CLI at `/home/sulthonuladib/.local/bin/android`; **that path does
not exist** on 2026-10-16, `command -v android` fails, and no `ANDROID_NDK_HOME` / `ANDROID_HOME` /
`ANDROID_SDK_ROOT` is set, with no `~/Android/Sdk/ndk/` present. **Consequence: no Android build can be
attempted on this host in stage B** (§5).

### 1.4 Adoption precedents found (all `verified-official-docs`, fetched 2026-10-16)

| Project | What it is | Ghostty coupling | Zig | Licence |
|---|---|---|---|---|
| `Xuanwo/gpui-ghostty` | *"GPUI + Ghostty (VT) Embedded Terminal"* — `crates/ghostty_vt_sys` (Zig build + **its own** C ABI), `crates/ghostty_vt` (safe wrapper), `crates/gpui_ghostty_terminal` (`TerminalView`), four examples incl. a PTY login shell and split panes | Ghostty **vendored as a git submodule pinned to tag `v1.2.3`** | **pinned 0.14.1**, installed project-locally by `./scripts/bootstrap-zig.sh` into `.context/zig/zig`; `ZIG=` env override honoured | Apache-2.0 |
| `xipeng-jin/libghostty-vt-prebuilt` | Prebuilt `libghostty-vt.a` static archives, tag `ghostty-<commit10>`, for a *fork's* terminal backend. *"consumed by Zed's terminal backend (`crates/ghostty_vt_sys/build.rs` in the Zed tree)"* | built from ghostty commits by CI | zig 0.15.2 on the publishing runner; consumers need **no Zig** | ships ghostty's MIT `LICENSE` inside each asset; assets carry GitHub artifact attestations |
| `xipeng-jin/zed` branch `migration/libghostty` | A **personal fork** of Zed attempting the migration; has `docs/ghostty-migration/artifact-pipeline.md`, `ghostty_pin.toml`. Last branch commit `e537270dacae…` *"Close the terminal P8 performance gate"*, **2026-07-22** (≈3 months stale) | libghostty-vt via the prebuilt archives | not needed | — |
| `behzade/gpui-libghostty` | RESEARCH-002 §4.3's precedent | **vendors Ghostty source** in-tree | Zig **0.16.0** (flake `zig-overlay`) | MIT |

Two important non-findings recorded against wishful thinking:

- **Upstream Zed has not migrated.** `GET api.github.com/repos/zed-industries/zed/contents/crates/ghostty_vt_sys`
  → **HTTP 404** (2026-10-16). The migration lives only in the individual's fork. A January-2026
  third-party write-up still states *"alacritty_terminal (Zed uses this for their terminal)"*
  (https://xuanwo.io/2026/01-gpui-ghostty/, fetched 2026-10-16). **`inferred`: Relayable cannot cite
  "Zed did it" as precedent for the *product*, only as precedent for the *build shape*.**
- **No upstream-published Linux binary.** The `tip` release publishes a source tarball, a wasm module and
  an Apple xcframework — **no Linux `.a`/`.so`**. Distro-side, third parties package `libghostty-vt-tip`
  RPMs versioned by build date (e.g. `1:202610120605`) and a release `libghostty-vt0-1.3.1`. **Confidence
  low / not re-verified in detail** (pkgs.org pages surfaced via search; the Fedora terra package page
  returned HTTP 402 to this fetch). Their practical value to Relayable is limited: they are dated
  snapshots with no pinning contract, and **`not observed`** as adopter-verified.

---

## 2. Q1 — Released vs `main`: what the tagged library actually exports

### 2.1 Tag inventory

```shell-session
$ cd /home/sulthonuladib/work/vendor/ghostty
$ git for-each-ref --sort=-creatordate --format='%(refname:short) %(creatordate:iso-strict)' refs/tags | head -3
tip     2026-10-15T21:55:50Z
v1.3.1  2026-03-13T09:03:38-07:00
v1.3.0  2026-03-09T08:52:26-07:00
$ git tag --list 'v*' --sort=-v:refname | head -1        # v1.3.1
```

**The newest release tag is `v1.3.1` (2026-03-13). `tip` is not a release tag in any versioning sense**
— it is a rolling pointer (`target_commitish: ${{ github.sha }}`); today it is `877dda7…` (2026-10-15),
one main-push behind `HEAD`. `build.zig.zon` on `main` says `.version = "1.3.2-dev"`, so **1.3.1 is
~7 months behind `main`** and no `1.3.2`/`1.4.0` tag exists.

### 2.2 Header inventory, by ref (`read-in-source`, reproduced)

```shell-session
$ git ls-tree -r --name-only v1.3.1 -- include/ | wc -l
13                                     # 12 headers + include/module.modulemap
$ git ls-tree -r --name-only v1.3.1 -- include/ghostty
include/ghostty.h  include/ghostty/vt.h
include/ghostty/vt/{allocator,color,key,osc,paste,result,sgr,wasm}.h
include/ghostty/vt/key/{encoder,event}.h
$ git ls-tree -r --name-only HEAD -- include/ | wc -l
36                                     # 35 headers + include/module.modulemap
$ git ls-tree -r --name-only HEAD -- include/ghostty/vt/ | sed -n '1,10p'
include/ghostty/vt/{allocator,build_info,color,color_scheme,device,focus,formatter}.h
include/ghostty/vt/{grid_ref,grid_ref_tracked,io}.h
# … and additionally, present on main but not at v1.3.1:
#   terminal.h  render.h  screen.h  selection.h  search.h  snapshot.h  style.h
#   mouse.h  mouse/encoder.h  mouse/event.h  modes.h  unicode.h  kitty_graphics.h
#   point.h  size_report.h  sys.h  types.h
$ git show v1.3.1:include/ghostty/vt.h | grep -c '#include'
7                                      # result allocator osc sgr key paste wasm
$ git show HEAD:include/ghostty/vt.h | grep -c '#include'
30                                     # incl. <ghostty/vt/terminal.h>, render.h, screen.h,
                                       # grid_ref.h, selection.h, …
```

(The `include/ghostty/vt/` file list is printed abbreviated above for readability; the count
`git ls-tree -r --name-only <ref> -- include/ | wc -l` is the exact, re-derivable number.)

Declared-symbol count (method: `git grep -h -o -E '^[a-zA-Z_].*\bghostty_[a-z0-9_]+\(' <ref> -- include/ghostty
| grep -oE 'ghostty_[a-z0-9_]+' | sort -u | wc -l` — a **header-declaration** count, not a symbol table):

| Ref | distinct declared `ghostty_*` names |
|---|---|
| `v1.3.1` | **50** |
| `tip` (`877dda7…`) | **207** |
| `HEAD` (`949149d…`) | **207** |

`v1.3.1`'s 50 names are exactly `ghostty_osc_*` (8), `ghostty_sgr_*` (9), `ghostty_key_event_*` (16),
`ghostty_key_encoder_*` (4), `ghostty_color_rgb_get`, `ghostty_paste_is_safe`, `ghostty_vt_osc_free`, and
13 `ghostty_wasm_*` allocators — **no `ghostty_terminal_*`, no `ghostty_render_state_*`, no
`ghostty_cell_*`/`ghostty_row_*`, no selection/mouse/search/snapshot**. This matches both RESEARCH-002's
`nm -D` count on the installed `.so` (43) and the header, so **RESEARCH-002 §5.3/§8.2 are confirmed**.

### 2.3 The decisive nuance: the engine is in the tag, the *C ABI* is not

`read-in-source`, `git show v1.3.1:src/lib_vt.zig`:

```
//! This is the public API of the ghostty-vt Zig module.
//!
//! WARNING: The API is not guaranteed to be stable.
```

with `:59 pub const Terminal = terminal.Terminal;` and neighbours `RenderState`, `Screen`, `ScreenSet`,
`Selection`, `Style`, `Parser`, `Pin`, `page`/`Cell`, `search`, `formatter`, `kitty`, `size`,
`device_status`, `modes`. So at the **released** tag:

- the **C library** (`libghostty-vt.so`) is a utilities subset — RESEARCH-002 is right;
- the **Zig module** contains the full terminal/page/render-state engine.

This is not a hypothetical: `Xuanwo/gpui-ghostty` (§1.4) pins `v1.2.3` and ships **its own** Zig build +
C ABI over the vendored core, precisely because the tagged C ABI did not expose the terminal.

**Consequence for pinning.** Two distinguishable pins exist:

| Pin | Engine | C ABI stability | Zig required | Release contract |
|---|---|---|---|---|
| tag `v1.3.1` + own Zig/C-ABI shim | full engine (Zig module) | *ours* — we own the boundary | **0.15.2** | tag exists; the *engine* API is still self-declared unstable (`lib_vt.zig:3`) |
| `main`/`tip` + upstream's C ABI | full engine | upstream's, explicitly unstable | **0.16.0** | **none** — no tag for the terminal API |
| tag `v1.3.1` + upstream's C ABI | **utilities only** | stable-ish | 0.15.2 | **unusable** for us |

### 2.4 What pinning `main` would actually mean (stated plainly)

1. **No released version of the API we would use.** `lib_version` is `0.1.0-dev` (`build.zig:11`) and
   upstream says, in its own README: *"We haven't tagged libghostty with a version yet and we're still
   working on a better docs experience"* (`README.md:168-170`).
2. **The API contract is explicitly self-declared unstable**, in both places we would depend on:
   `include/ghostty/vt.h:10-12` — *"WARNING: This is an incomplete, work-in-progress API. It is not yet
   stable and is definitely going to change."*; `:25-26` — *"Breaking changes are expected in future
   versions. Use with caution in production code."*; `src/lib_vt.zig:3` — *"The API is not guaranteed to
   be stable."*
3. **No soname protection.** Version major is `0` for both the utilities release and `main` (C4). A
   `dynamic` link resolves by `libghostty-vt.so.0`, so ABI drift is invisible to the dynamic linker.
   **`inferred` with high confidence, observed for v1.3.1.** Practical rule: pin an exact commit and link
   **statically**; treat dynamic linking against a system library as unsafe.
4. **Availability of "the commit" is itself not guaranteed.** The per-commit tarball is produced by CI
   from `main` pushes, but three of six probes 404'd (§3.4).
5. **Cost of a bump is a re-verify, not a semver bump.** `include/ghostty/vt.h` declares sized structs
   (`sized_field!` in the binding reads fields only when they fit `size`) and a `GhosttyResult` code
   (`0` success, negative errors), which the binding maps at
   `crates/libghostty-vt/src/error.rs:51 from_result`. Growing a struct is tolerated; renaming or removing
   a symbol is not. **`read-in-source`.**

**Honest counterpoint:** the *functional* stability claim is unusually strong and upstream states it
repeatedly: *"The functionality is extremely stable (since its been proven in Ghostty GUI for a long
time), but the API signatures are still in flux."* (`README.md:157-160`), repeated in `lib_vt.zig:5-8` and
in the binding's crate docs. **The risk is the boundary, not the parser.** That is exactly the risk
profile the "own thin C ABI over a tagged engine" path in §7.3 removes — at the cost of owning the shim.

---

## 3. Q3 — Building on this host

### 3.1 Exact Zig version required (`read-in-source`)

```shell-session
$ git show HEAD:build.zig.zon | sed -n '1,8p'
.{
    .name = .ghostty,
    .version = "1.3.2-dev",
    .paths = .{""},
    .fingerprint = 0x64407a2a0b4147e5,
    .minimum_zig_version = "0.16.0",
$ sed -n '10,17p' build.zig
const minimum_zig_version = @import("build.zig.zon").minimum_zig_version;
comptime {
    buildpkg.requireZig(minimum_zig_version);
}
$ git show v1.3.1:build.zig.zon | grep minimum_zig_version
.minimum_zig_version = "0.15.2",
```

So: **Zig `0.16.0` for `main`/`tip`; `0.15.2` for the `v1.3.1` tag.** Upstream pins the same version in its
own dev shell: `flake.nix:80` — `zig = zig.packages.${pkgs.stdenv.hostPlatform.system}."0.16.0";`.
Enforcement is a `comptime` assertion run by the build system itself, i.e. **the wrong Zig fails
immediately and loudly**, not silently.

### 3.2 Is a `mise`-managed Zig pin available? (`observed-by-running`)

```shell-session
$ mise registry | grep '^zig'
zig    core:zig
$ mise ls-remote zig | grep -E '^(0\.15\.2|0\.16\.0|0\.17\.0)$'
0.15.2
0.16.0
0.17.0
```

**Yes.** The `core:zig` backend lists **exactly `0.16.0`** (and `0.15.2`, and `0.17.0`). No third-party
plugin is needed. Note that upstream itself refuses to be driven by a floating Zig: the requirement is a
minimum *and* the upstream packaging guidance says Ghostty *"always uses some released version of Zig"*
(`PACKAGING.md` §Zig Version), so a future `main` may require `0.17.0` — **the Zig pin must be re-checked
at every engine bump**, and that is one more item on the re-verify tax (§2.4.5).

### 3.3 Build/link shape (`read-in-source`)

Produced by `zig build -Demit-lib-vt -Dapp-runtime=none` (the exact flag set both Rust bindings use —
`libghostty-vt-sys/build.rs:186-211`; `ghostty-vt-sys/build.rs:340-366`):

| Artifact | Evidence |
|---|---|
| `libghostty-vt.a` (static, default in both bindings) | `src/build/GhosttyLibVt.zig:172` `.name = "ghostty-vt"`, static name `libghostty-vt.a` (`:560`); bundles `compiler_rt` + `ubsan_rt` and sets PIC (`:239-249`) |
| `libghostty-vt.so` (opt-in: `link-dynamic` feature) | `src/build/GhosttyLibVt.zig:226-232` `addLibrary(.{ .linkage = linkage, .version = zig.version })`; installed via `addInstallArtifact` (`:342`, `:365`) |
| Headers | `installHeadersDirectory(b.path("include/ghostty"), "ghostty", .{ .include_extensions = &.{".h"} })` (`:232-236`) → `<prefix>/include/ghostty/*.h` |
| pkg-config | `<prefix>/share/pkgconfig/libghostty-vt.pc` and `libghostty-vt-static.pc` (`:645-656`), templates at `:519-556` (`Name: libghostty-vt`, `Version: {zig.version}`, `Libs: -L${libdir} -lghostty-vt`) |
| Zig-module consumer artifacts | as a *dependency*, `dep.artifact("ghostty-vt")` (shared) and `dep.artifact("ghostty-vt-static")` (`src/build/GhosttyLibVt.zig:228`; `build.zig:166-170`) |
| ABI | **C ABI**, deliberately restricted exports (`.exports` allowlist, `GhosttyLibVt.zig:389-391`,`:420`); `src/terminal/c/types.schema.json` + `types-schema-verify.py` validate the ABI type manifest against the built `.so` (`build.zig:145-152`) |

Two build knobs matter for us:

- **`-Dvt-features`** (`src/build/Config.zig:429-449`; docs `PACKAGING.md` §WebAssembly): comma-separated
  modifications to the all-enabled set, `-Dcpu` style. The ten gates are
  `snapshot, formatter, selection, search, render_state, input_encode, color, grid_introspection,
  glyph_protocol, kitty_graphics` (`src/terminal/build_options.zig`, `pub const Features = packed struct`).
  A terminal viewer needs `-all,+render-state`; an interactive terminal wants
  `-all,+render-state,+input-encode,+selection,+color`. **Neither Rust binding passes this flag**, so both
  build the full set today.
- **`--system <dir>`** with `GHOSTTY_ZIG_SYSTEM_DIR` → Zig resolves packages from an immutable pre-fetched
  store instead of downloading (`libghostty-vt-sys/build.rs:216-232`; `ghostty-vt-sys/build.rs:357-364`;
  upstream's own recipe `nix/build-support/fetch-zig-cache.sh`, which loops `zig fetch` over
  `build.zig.zon.txt`).

### 3.4 The dependency/network surface is smaller than expected (`read-in-source` + `observed-by-running`)

I parsed `build.zig.zon`'s dependency table: **only `uucode` is non-lazy**; `libxev`, `vaxis`, `z2d`,
`zig_objc`, `zig_js`, `zig_wayland`, `zf`, `gobject`, `wayland`, `wayland_protocols`,
`plasma_wayland_protocols`, `jetbrains_mono`, `nerd_fonts_symbols_only`, `iterm2_themes` are all
`.lazy = true` (app-side only, not built for lib-vt). The C shims in `pkg/` are path dependencies, and the
SIMD one vendors its sources in-tree with **zero** external URLs:

```shell-session
$ grep -c "url =" pkg/simdutf/build.zig.zon    # 0
$ ls pkg/simdutf/vendor                         # simdutf.cpp  simdutf.h
```

The remaining external fetches for a lib-vt build are small and pinned by hash in-tree:
`pkg/translate-c/build.zig.zon:15` (codeberg tarball), `pkg/oniguruma/build.zig.zon:9` and
`pkg/wuffs/build.zig.zon:10,17` (both `.lazy = true`, pulled only for the feature set that needs them —
`src/build/SharedDeps.zig:337`, `:523`).

**The per-commit source tarball is a real, checksummed artifact** (upstream `release-tip.yml`, job
`source-tarball-lib-vt`):

```shell-session
$ curl -sSI https://tip.files.ghostty.org/b40acce58dcf77df52231c3798ea58e924647c89/libghostty-vt-source.tar.gz
HTTP/2 200   content-length: 4032640   last-modified: Sun, 27 Sep 2026 02:56:20 GMT
$ curl -sS -o t.tgz <same url>; sha256sum t.tgz
a623aa0b1267594d9a1bb76a5fef535f198004e78730894f494a5bdc522a4850  t.tgz
```

`a623aa0b…` is **byte-identical to the sha256 pinned in `gpui-cn/third_party/GHOSTTY.lock`** — an
independent, cross-vendor confirmation of the artifact. The tarball root is
`libghostty-vt-1.3.2-main+b40acce/` and it contains the build inputs (1514 entries).

**Honest caveat (`observed-by-running`):** availability is **not uniform** across commits.

| commit | HTTP status of the per-commit tarball |
|---|---|
| `b40acce58dcf…` (2026-09-26) | **200** |
| `a887df42c56f…` (pinned by `libghostty-vt` 0.2.2) | **200** |
| `b3dc3d794513…` (old `tip`) | **200** |
| `877dda73124d…` (current `tip`) | **404** |
| `949149d5791a…` (`main` HEAD today) | **404** |
| `3425025e585a…` (pinned by `libghostty-rs` HEAD) | **404** |

So a stage-B pin must **first verify the artifact exists**, or provision from a git clone (201 MB, which I
already have locally) instead.

### 3.5 Time and disk — an **estimate**, explicitly unmeasured

RESEARCH-002 could not build and neither can stage A (no Zig installed; installing is an approval item).
**No build was run. Every number below is `inferred` and must be replaced by measurements in stage B.**

| Item | Estimate | Basis |
|---|---|---|
| Zig 0.16.0 install (mise, per-user) | ~45–60 MB download, under `~/.local/share/mise/installs/zig/0.16.0` | standard ziglang release tarball size; **unmeasured here** |
| Engine source | 4 MB (tarball) or 201 MB (full clone, already present) | `observed-by-running` |
| Zig package cache (`ZIG_GLOBAL_CACHE_DIR`) | ~50–300 MB | one non-lazy external package (`uucode`) + `translate-c`; the heavy deps are lazy and not resolved |
| Build output/cache (`--cache-dir`, `OUT_DIR`) | ~0.2–1 GB | Zig local cache incl. intermediate objects; `observed-by-running` for the code size (166 `.zig` files, 6.2 MB) |
| `libghostty-vt.a` (ReleaseFast) | tens of MB, DWARF-dominated | upstream's own wasm datapoint: ReleaseFast *"over 5 MB"* → *"roughly 1.1 MB"* stripped vs *"roughly 0.8 MB"* ReleaseSmall (`PACKAGING.md`); the installed **utilities-only** v1.3.1 `.so` is 842 KB |
| First build wall-clock | **order of 10–30 min** | inference from code volume + upstream CI shape; **UNVERIFIED** |
| Incremental rebuild | seconds to a few minutes | `inferred` |

Host capacity is not a constraint: 235 GB free on `/home` (`observed-by-running`).

### 3.6 The exact approvals I need (nothing was installed)

Two approvals, both narrow:

**A1 — acquire Zig `0.16.0` for this user, no `sudo`, no system change.**
Two acceptable shapes; the lead should pick one and record it:

- **(preferred) `mise install zig@0.16.0`.** `mise` is already installed (`2026.10.10 linux-x64`), the
  backend is the official `core:zig`, and `mise ls-remote zig` lists `0.16.0`. Lands under the user's
  `~/.local/share/mise/`. One wrinkle to decide: **where the version pin is recorded** so a fresh clone
  reproduces it — a repo-level `mise.toml`/`.mise.toml` is a root manifest-style file the lead owns. Third
  parties solve exactly this with `mise.toml` (`gpui-cn/mise.toml`: `[tools] zig = "0.16.0"`).
- **(alternative) project-local toolchain**, mirroring `Xuanwo/gpui-ghostty`: a `scripts/bootstrap-zig.sh`
  that unpacks the official ziglang tarball into `.context/zig/zig`, with `ZIG=` honoured by the binding's
  `build.rs`. Nothing touches a shared prefix at all. Costs a bespoke script (we own it).

**A2 — one-time networked provisioning of engine source + Zig packages**, per the spirit of DEC-018
(*"Do not run a networked `cargo fetch` at agent initiative; the DEC-012 pattern requires user approval"*).
The analogous step here is:
`curl` the pinned `libghostty-vt-source.tar.gz` (or use the existing clone) **and** `zig fetch`/`--system`
provisioning of `uucode`, `translate-c` (+ feature-dependent `oniguruma`/`wuffs`) into a fixed directory.
After it succeeds, `zig build --system <dir>` is network-free, which is what a hermetic/CI story needs.

**Also needed (a decision, not a permission):** confirmation that stage B may add a spike crate with its
own workspace + provisioning scripts under `spikes/**` (exempt from the file-size caps,
`docs/architecture/code-organization.md`; precedent `spikes/terminal-engine-phase2`), and that the
non-lazy-network rule does not forbid the one-time provisioning in A2.

**Explicitly not requested:** no `sudo`, no distro package, no global install, no change to
`/usr`, no repository-tracked copy of upstream source, no edit to `.gitignore`/`Cargo.toml`/`AGENTS.md`.

---

## 4. Q2 + Q4 — the Rust binding, and the adoption shape

### 4.1 Binding A — `libghostty-vt` / `libghostty-vt-sys` (Uzaaft) — the maintained candidate

`read-in-source` at tag `v0.2.2` (`a3a96769…`) and at `master` HEAD (`d2036ba…`):

| Property | Value | Anchor |
|---|---|---|
| Bindings target | a **pinned Ghostty git commit**, cloned at build time: `a887df42c56f6de86c0fe6da9c4eeca37931e083` at v0.2.2, `3425025e585a3403340d4dc6d65132f42e05e605` at HEAD | `crates/libghostty-vt-sys/build.rs:6-7`; `git show v0.2.2:crates/libghostty-vt-sys/build.rs:7` |
| Default build | `default = ["vendored"]` → `git clone --filter=blob:none` of Ghostty into `OUT_DIR/ghostty-src`, then **bare `zig`** | `crates/libghostty-vt-sys/Cargo.toml`; `build.rs:417-460`; `build.rs:186` |
| Zig discovery | **bare `zig` on `PATH` only** — no `ZIG` env override, no `mise` fallback (the prabirshrestha binding has both; this one does not) | `build.rs:186` |
| Zig invocation | `zig build -Demit-lib-vt=true -Doptimize=… -Dcpu=… -Demit-xcframework=false -Dapp-runtime=none --prefix …` | `build.rs:186-211` |
| Optimize mode | from Cargo profile: `Debug` for dev, `ReleaseSmall` for `opt-level=s/z`, else `ReleaseFast`; `LIBGHOSTTY_VT_SYS_OPTIMIZE` overrides | `build.rs:383-413` |
| CPU | `-Dcpu=baseline` by default (portable binaries); `LIBGHOSTTY_VT_SYS_CPU=native` opt-in | `build.rs:152-156` |
| Link | **static** `libghostty-vt.a` by default; `link-dynamic` feature switches to `dylib=ghostty-vt` | `build.rs:287-309` |
| Target map | linux gnu/musl ×2 arch, macos ×2, windows-msvc ×2, **`aarch64-linux-android`, `x86_64-linux-android`** | `build.rs:558-575` |
| `pkg-config` path | opt-in feature; probes `libghostty-vt` (dynamic) / `libghostty-vt-static` (static); `GHOSTTY_SOURCE_DIR` always wins | `build.rs:99-115`, `:321-341` |
| Offline provisioning | `GHOSTTY_SOURCE_DIR` + `GHOSTTY_ZIG_SYSTEM_DIR` → `--system` | `build.rs:104`, `:216-232`; README §Building |
| docs.rs / Miri | build.rs early-returns (no Zig needed for docs) | `build.rs:71-82` |
| Zig requirement stated | *"Requires [Zig](https://ziglang.org/) 0.16.x on PATH."* | `README.md:54` |
| Licence metadata | workspace `license = "MIT OR Apache-2.0"`; **repo root ships only a `LICENSE` (MIT)** — `LICENSE-MIT`/`LICENSE-APACHE` do **not** exist | `Cargo.toml` `[workspace.package]`; `git ls-tree -r --name-only v0.2.2 \| grep -iE 'license\|notice\|copying'` → `LICENSE` only |
| Published tarball | **no licence file at all** (independently reproduced) | see §5.2 |

### 4.2 Binding B — `ghostty-vt` / `ghostty-vt-sys` (prabirshrestha/gpui-cn) — the cleaner-licensed, less-adopted one

| Property | Value | Anchor |
|---|---|---|
| Bindings target | an **upstream-published per-commit source tarball**, not a git clone: `commit = b40acce58dcf77df52231c3798ea58e924647c89`, `url = https://tip.files.ghostty.org/b40acce…/libghostty-vt-source.tar.gz`, `sha256 = a623aa0b1267594d9a1bb76a5fef535f198004e78730894f494a5bdc522a4850` | `third_party/GHOSTTY.lock`; `crates/ghostty-vt-sys/GHOSTTY.lock` |
| Default build | `default = []` → **downloads** the tarball (sha256-verified), then runs Zig | `crates/ghostty-vt-sys/Cargo.toml`; `build.rs:209-230` |
| Zig discovery | `ZIG` env → `zig` on `PATH` (if `0.16.x`) → **`mise which zig`** → explicit panic telling the user to `mise install` | `crates/ghostty-vt-sys/build.rs:396-432` |
| Zig requirement | hard-asserted `version.starts_with("0.16.")` | `build.rs:434-443` |
| Zig invocation | `zig build -Demit-lib-vt -Doptimize=ReleaseFast -Dsimd=true -Dtarget=… -Dcpu=… -Dversion-string=… -Demit-xcframework=false -Demit-themes=false --prefix …` | `build.rs:340-366` |
| Project Zig pin | **`mise.toml`: `[tools] zig = "0.16.0"`** (also `rust-toolchain.toml` → 1.98.1) | `mise.toml`, `rust-toolchain.toml` |
| Target map | linux gnu/musl ×2 arch, macos ×2, windows-msvc ×2, `aarch64-apple-ios`, `aarch64-apple-ios-sim` — **no Android**; unsupported targets `panic!` with a target list | `build.rs:467-485` |
| Licence files in the **published** crate | `LICENSE` (Apache-2.0 text) **and** `LICENSE-GHOSTTY` (MIT, ghostty) — verified by extraction | see §5.2 |
| Download hardening | `download_failure()` tells the user to set `GHOSTTY_SOURCE_URL` to a mirror; sha256 mismatch is a hard panic | `build.rs:299-320`, `:230` |

**Adoption implication.** Binding B has the better licence story and the better Zig-discovery story, and
its source provisioning matches an upstream-immutable artifact — but it has **106 total crates.io
downloads**, 1 GitHub star, **no Android target**, and it pins a **2026-09-26** commit (older than binding
A's HEAD pin). Binding A has 167 k downloads and Android targets but no `ZIG` override and no licence file.
**Neither is a clean winner; the choice is a §4.4/§7 item for the lead, not a research conclusion.**

### 4.3 Safe-API coverage for Relayable's needs (`read-in-source`, binding A)

Every capability the existing `apps/linux/src/term/**` interface needs has a documented safe call:

| `term/model.rs` need (today) | libghostty-vt safe API | Anchor |
|---|---|---|
| `TerminalModel::new(size)` | `Terminal::new(cols, rows) -> Result<Self>` | `crates/libghostty-vt/src/terminal.rs:261` |
| `process(&[u8])` | `Terminal::vt_write(&mut self, data: &[u8])` (also `vt_write_until_ground`, `is_vt_ground`) | `terminal.rs:312`, `:333`, `:350` |
| `resize(size)` | `Terminal::resize(cols, rows, cell_w_px, cell_h_px) -> Result<()>` — reflows the primary screen, no reflow on alt screen | `terminal.rs:368` |
| `snapshot()` grid/cells | `RenderState::update/begin_update/end`, then `RowIterator::next`, `CellIterator::next`, `Cell::fg_color`/`bg_color`/`graphemes`/`graphemes_utf8`, `Row::is_wrapped` | `render.rs:462`, `:491`, `:552`, `:870`, `:1028`, `:1086`, `:1104`, `:1116`, `:1154` |
| cursor position/visibility/style | `RenderState::cursor`, `cursor_visible`, `cursor_blinking`, `cursor_visual_style`, `cursor_password_input`, `cursor_viewport`; `Terminal::cursor_x/cursor_y/is_cursor_visible/cursor_style` | `render.rs:609-660`; `terminal.rs:864-891` |
| `history_len()` | `Terminal::scrollback_rows()`, `total_rows()`, `Scrollbar` | `terminal.rs:983`, `:979`, `:923` |
| `snapshot_at(offset)` | `Terminal::scroll_viewport(ScrollViewport::{Top,Bottom,Delta,Row})` (+ `viewport_active`) | `terminal.rs:400`, `:1270-1290` |
| scrollback **bound** (`full.md` §6.4) | `scrollback_max_bytes`/`set_scrollback_max_bytes(Option<usize>)`, `scrollback_max_lines`/`set_scrollback_max_lines`, plus `compression_activity` | `terminal.rs:805-858`, `:587` |
| OSC 0/2 title, OSC 7 cwd | `Terminal::title() -> Result<&str>`, `Terminal::pwd() -> Result<&str>`, and callbacks `on_title_changed`, `on_pwd_changed` | `terminal.rs:959`, `:971`, `:2607`, `:2621` |
| OSC 8 hyperlink URI per cell | `GridRef::hyperlink_uri(&mut [u8]) -> Result<usize>`, `Row::has_hyperlink`, `Cell::has_hyperlink` | `screen.rs:116`, `:312`, `:374` |
| palette / colour resolution | `Terminal::color_palette()`, `default_color_palette()`, `Cell::bg_color_palette()`, `GridRef::style()` | `terminal.rs:1076`, `screen.rs:390`, `:75` |
| bell / clipboard / notification / progress | callbacks `on_bell`, `on_clipboard_write` (`ClipboardWrite` + `reply`), `on_clipboard_read`, `on_desktop_notification`, `on_progress_report`, `on_pty_write`, `on_size`, `on_render_hold` | `terminal.rs:2401-2781` |
| selection | `selection.rs`, `selection/gesture.rs` | crate tree |

### 4.4 Adoption shape — three options, and what each touches

**(a) Full replacement of `alacritty_terminal` + `vte` in the desktop app.**
**(b) Additive** — keep `alacritty_terminal` for screen state and bind libghostty only for the
input-encoding gap (key/paste/mouse encoders). RESEARCH-002 §13 already proposed this; note it **carries
the same Zig build cost**, so it buys capability without removing a dependency.
**(c) "Own thin C ABI over a pinned tagged engine"** (the `Xuanwo/gpui-ghostty` shape, §1.4): vendor/pin
`v1.3.1` (Zig **0.15.2**), write our own Zig build + C ABI over `src/lib_vt.zig`, bind that. Removes the
third-party binding (and its licence gap) and removes dependence on `main`, at the cost of owning a shim
and being a tag behind `main`'s API.

**Critical scoping fact — the swap is *not* contained to `term/**`.** The `vte` pin is used directly by the
client-side OSC tracker:

```
apps/linux/src/api/osc.rs:1   use vte::Perform;
apps/linux/src/api/osc.rs:12  /// A `vte` performer that only records OSC 0/2 (title) and OSC 7 (cwd).
apps/linux/src/api/osc.rs:18  impl Perform for OscPerformer<'_> { fn osc_dispatch(...) }
```

That file feeds `Pane::program` / `Pane::cwd` (`apps/linux/src/app/state.rs:52-59`), which map onto the
reference events `terminal.title_changed` / `terminal.pwd_changed`
(`docs/compatibility/reference-behavior-matrix.md:396-397`). libghostty can supply both
(`Terminal::title()`, `Terminal::pwd()`, `on_title_changed`, `on_pwd_changed`), so the data is available —
but the ownership question (**engine owns OSC title/cwd, or the `api` layer keeps doing it?**) is a design
decision the ADR amendment must answer, and option (a) silently moves that responsibility if not stated.

Files that would change under option (a), with anchors:

| File | Change | Note |
|---|---|---|
| `apps/linux/Cargo.toml` | swap `alacritty_terminal = "=0.26.0"` + `vte = "=0.15.0"` for the chosen binding; add the Zig provisioning contract | **lead-owned manifest** |
| `apps/linux/src/term/model.rs` (247 lines) | **entire backend rewritten** — `Dims`/`Dimensions`, `TerminalModel::{new,process,resize,snapshot,snapshot_at,history_len,size}`, `collect_cells`, `cursor_of`, `convert_cell`, `convert_color`, `convert_bg`, `named_index` | engine-specific by construction |
| `apps/linux/src/term/mod.rs` | the doc block listing "Verified upstream API (file:line in the local registry source)" at `:9-34` is now false and must be replaced; its two deliberate non-decisions (`Config::scrolling_history` default; no measured bound) carry over as *different* API | `BLANK_CELL` stays |
| `apps/linux/src/term/snapshot.rs` | **unchanged** — `TerminalSize`, `TermColor`, `CellStyle`, `ScreenCell`, `ScreenSnapshot` are engine-neutral | this is the reason the swap is feasible at all |
| `apps/linux/src/term/tests.rs` (8 tests) | assertion bodies mostly survive; the two colour-semantics tests need re-verification (below) | see "semantics" note |
| `apps/linux/src/api/osc.rs` | only if responsibility for title/cwd moves (above) | |
| Call sites (compile-only if the snapshot types hold): `app/state.rs:1,58,83,93`; `app/events.rs:2,89`; `app/resize.rs:3`; `bin/live-probe.rs:28,60,88,115`; `ui/terminal.rs:28,376,379` | | `ui/terminal.rs` consumes `CellStyle`/`ScreenSnapshot`/`TermColor` only |

**Does the public `apps/linux` behaviour change?** The **rendering** contract does not:
`ui/terminal.rs` paints `ScreenSnapshot` (`apps/linux/src/ui/terminal.rs:33-60` `TerminalCanvas`,
`:81 pub fn into_element`), and `TermColor::Default` deliberately means "the measured ramp shows through"
(`apps/linux/src/term/snapshot.rs:29-31`). Behaviour **can** change in the engine-semantics corners the
existing tests pin:

- `tests.rs:19-33` asserts `\x1b[31m` yields `TermColor::Indexed(1)` and that `\x1b[0m` returns
  `TermColor::Default`. libghostty resolves styles through `Style` handles plus a palette
  (`screen.rs:390 bg_color_palette()`, `terminal.rs:1076 color_palette()`), so an **indexed** colour may
  only be recoverable as RGB + palette, or not at all — this is the single most likely test-level change.
- `tests.rs:93-102` asserts `history_len() >= 2` after `"one\r\ntwo\r\nthree\r\nfour"` — semantics differ
  (`scrollback_rows()` vs `history_size()`), so the assertion threshold may need recalibration (a *test*
  edit, not a behaviour change).
- `snapshot.rs:69-71` `wide_continuation` maps `WIDE_CHAR_SPACER|LEADING_WIDE_CHAR_SPACER`; libghostty
  exposes `Cell::wide() -> CellWide` — mapping must be re-verified.

**Threading constraint (`read-in-source`, high).** The safe crate documents: *"all `libghostty-vt` types
are `!Send` … they are also `!Sync`"*, recommending one owning thread plus channels
(`crates/libghostty-vt/src/lib.rs:55-76`). **I found no `Send` bound on the term model today**:
`grep -rn "Send\b" apps/linux/src` hits only `api/control.rs:16,164` and `api/tests.rs:9`, which concern the
API handle; `TerminalModel` is a plain field of `Pane` (`app/state.rs:58`) inside `Tab { panes: Vec<Pane> }`
(`app/state.rs:136-137`), and the one `cx.spawn` (`app/lifecycle.rs:92`) captures a weak entity, not the
model. **`inferred, medium`: the `!Send` constraint is probably compatible — but stage B must prove it**
(the `live-probe` binary owns a `TerminalModel` in a struct and drives it from one thread,
`bin/live-probe.rs:60,88,115`).

**Which ADR-003 pins an amendment would have to revisit** (naming only — `docs/adr/**` is lead-owned):

| ADR-003 element | Why it is touched |
|---|---|
| Decision item 1 — `alacritty_terminal 0.26.0` the parser + screen/grid engine | replaced under (a) or `main`-only-under-`0.1.0-dev`-under (b)/(c) |
| Decision item 2 — `vte 0.15.0` as the parser boundary | **also used by `apps/linux/src/api/osc.rs:1`** — cannot be retired without deciding title/cwd ownership |
| Decision item 3 — `portable-pty 0.9.0` PTY ownership | **unaffected**: libghostty has no PTY layer; the PTY stays in `crates/relayable-pty` |
| Decision item 4 — `gpui = "=0.2.2"` + `libc 0.2.189` hold | **unaffected by the engine**; but if a Zig→`cc`/`ld` path entered `apps/linux`'s build, the `libc`/`xattr` hold interacts (worth an explicit statement) |
| Decision item 5 — licence-obligation mitigation list | gains a fifth artefact: ghostty's MIT text + `Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors` |
| "Status: Accepted — engine pins validated by TERM-002 (32 PASS / 1 UNSUPPORTED / 0 FAIL)" | the sentence "no engine change is warranted" is the claim a swap contradicts; it must be re-stated against new evidence, not deleted |
| §5.4 requirement-mapping table | every row is engine-specific |
| Reconsideration trigger #10 (sub-conditions 1–3) | sub-condition 3 ("a passing build/spike") is what stage B would discharge; sub-condition 2's licence caveat is refined in §5 |
| Consequences → "`libc` lockfile-hold (forward constraint)" | unchanged, but see above |
| `docs/agent-loop/DECISIONS.md` DEC-004 (engine pins) and DEC-014 (superseded for libghostty only, per DEC-021) | decision-ledger edits, lead-owned |

---

## 5. Q6 — Licence and attribution

### 5.1 Upstream Ghostty — MIT, verified from the clone (`verified-official-docs`)

`LICENSE` at the repo root of the clone (and at every tag checked): `MIT License` /
`Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors` (sha256 `386211873e5b7a02f663ae4d7adf96285999f91608f8f9f31fecfd0f4095e6f1`, matching RESEARCH-002 §3.1 exactly). **No `NOTICE` file exists upstream**
(confirmed: `find` over the clone root and over the extracted per-commit tarball).

### 5.2 The bindings' licence files — RESEARCH-002's finding **reproduced independently**

```shell-session
$ curl -sSL -o libghostty-vt-0.2.2.crate  https://crates.io/api/v1/crates/libghostty-vt/0.2.2/download
$ curl -sSL -o libghostty-vt-sys-0.2.2.crate https://crates.io/api/v1/crates/libghostty-vt-sys/0.2.2/download
libghostty-vt-0.2.2.crate      16c7287a8dc35c3436224b39358af81ab9ea54bafa0f7ea2e068bd2a47bdcb2d   82491 B
libghostty-vt-sys-0.2.2.crate  df233d9b287cded4bbebba3463c6df7a8bdb516c0341ed984e870ef5594c2e6f   65701 B
$ find ext -iname 'LICENSE*' -o -iname 'NOTICE*' -o -iname 'COPYING*'     # (no output)
$ find ext/libghostty-vt-0.2.2 -type f | sort   # Cargo.toml(.orig), .cargo_vcs_info.json,
                                                # README.md, src/*.rs  — and nothing else
```

The checksum `16c7287a…` **matches RESEARCH-002's recorded checksum**, so this is the same artifact,
re-fetched and re-inspected. `Cargo.toml` declares `license = "MIT OR Apache-2.0"` while the repo root
ships only the MIT text (`LICENSE`, 1086 bytes). By contrast, binding B's published crate **does** bundle
both: `ghostty-vt-0.7.2+ghostty.1.3.2-main.b40acce.crate` ships `LICENSE` (Apache-2.0, 10305 B) and
`LICENSE-GHOSTTY` (MIT, first line `MIT License`, second `Copyright (c) 2024 Mitchell Hashimoto, Ghostty
contributors`); its `.crate` sha256 is `c056e7b0393b6516369c62508cdf243fc3c9b3acb711f3dd14f0b17eb9660915`,
matching RESEARCH-002. **So §3.2 of RESEARCH-002 stands, strengthened by one more independent artifact
fetch.**

### 5.3 What Relayable must actually ship (MIT, in practice)

MIT's operative clause requires that *"the above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software."* Practical consequences, unchanged by
whether we link statically or dynamically:

1. **Ship the MIT text plus the copyright line** for ghostty's code in whatever the distribution's
   third-party-notice channel is. That is `Copyright (c) 2024 Mitchell Hashimoto, Ghostty contributors`.
2. **Static vs dynamic does not change the obligation.** MIT is not copyleft and has no
   derived-work/source-disclosure terms, so linking shape changes nothing here (unlike, say, LGPL). A
   statically-linked `.a` does mean the ghostty code is *embedded* in Relayable's binary, which makes the
   notice requirement apply to the binary distribution rather than only to a library package — worth
   stating explicitly, but it is the same sentence of the licence.
3. **No `NOTICE` obligation exists**, because upstream ships no `NOTICE` (`Apache-2.0 §4(d)`'s
   notice-propagation duty has no object). ADR-003 decision item 5 already records the same for the four
   pinned crates and ships no `NOTICE`; ghostty is consistent with that pattern.
4. **If we bind through binding A, its licence file is the gap**, not ghostty's. `MIT OR Apache-2.0`
   permits electing MIT, so a consumer who can read the copyright line from the upstream repository can
   satisfy MIT — but the *crate artifact we would depend on* contains **no licence text**, so the notice
   is not obtainable from the artifact alone. This is precisely the pattern ADR-003 rejected for
   `Glass-HQ/gpui` (Option I). **It is a real defect and I am not adjudicating it**; I am reporting that it
   is **severable**, by any of: (i) explicit lead acceptance of the declaration plus the upstream MIT text
   as the notice; (ii) using binding B, which bundles both texts; (iii) **option (c)** (§4.4), which
   consumes upstream ghostty directly and therefore needs only ghostty's own MIT text and **no third-party
   licence at all**.

Attribution is *not* clean-room territory: DEC-021 already records *"upstream ghostty is MIT, so licence
compliance is attribution, not clean-room"*, and `full.md` §2.6's clean-room rules continue to apply to
GoRex only.

---

## 6. Q5 — Portability (Android, and the shared crates)

### 6.1 Upstream's lib-vt build path has **explicit Android support** (`read-in-source`, high)

```
src/build/GhosttyLibVt.zig:273-278
    if (lib.rootModuleTarget().abi.isAndroid()) {
        // Support 16kb page sizes, required for Android 15+.
        lib.link_z_max_page_size = 16384; // 16kb
        try @import("android_ndk").addPaths(b, lib);
    }
```

`pkg/android-ndk/build.zig` `pub fn addPaths(b, step)` resolves the NDK from `ANDROID_NDK_HOME`, else
`ANDROID_HOME`/`ANDROID_SDK_ROOT`, else `$HOME/Android/Sdk`, and writes a `libc.txt` for Zig's bionic
cross-compilation ("*This is a workaround until zig natively supports bionic cross-compilation
(ziglang/zig#23906)*"). The SIMD dependency does the same (`pkg/simdutf/build.zig:35-36`). `build.zig.zon`
declares the helper as a path dependency (`.android_ndk`).

This is **materially stronger** than RESEARCH-002's §7 verdict ("an Android NDK helper exists; unknown
whether it is used by a shipped product"): the helper is now shown to be **wired into the lib-vt library
target itself**, which is the target we would build.

**But nothing can be verified on this host** (§1.3): **no NDK, no Android SDK, no `android` CLI at the
documented path**, and `ANDROID_*` unset. So Android remains **`UNVERIFIED`** and, unlike the desktop case,
cannot even reach a build attempt in stage B without separate provisioning (which I am not requesting).

Binding-level portability differences (both `read-in-source`):

| | binding A `libghostty-vt-sys` | binding B `ghostty-vt-sys` |
|---|---|---|
| Android triples | `aarch64-linux-android`, `x86_64-linux-android` (`build.rs:570-571`) | **none** — explicit `panic!` listing only linux/macos/windows/iOS (`build.rs:467-485`) |
| iOS | via upstream xcframework (`build.rs:158-184`, `:484-533`) | `aarch64-apple-ios`, `aarch64-apple-ios-sim` |
| Windows | gnu + msvc, both arches | msvc, both arches |

### 6.2 `relayable-model` / `relayable-protocol` / off-Linux (assessment)

- **Unaffected either way.** Both crates are pure Rust and depend on nothing in `apps/linux`; ADR-001's
  dependency direction forbids the reverse. The engine sits behind the desktop app's `term/**`, whose
  public types (`ScreenSnapshot`/`ScreenCell`/`CellStyle`/`TermColor`/`TerminalSize`,
  `apps/linux/src/term/snapshot.rs`) are engine-neutral and GPUI-free.
- **PTY ownership is untouched.** libghostty, like `alacritty_terminal`, has **no** PTY layer: no
  `forkpty`/`openpty`/child-process API appears in `include/ghostty/vt.h`. `crates/relayable-pty` (and
  `portable-pty` behind its opaque handles, `crates/relayable-pty/src/lib.rs:1-16`) remains the only place
  that allocates a PTY — so ADR-001's server-side boundary and ADR-003 decision item 3 are unaffected.
- **The engine is *already* headless.** No renderer, no font stack, no GPU path in the library
  (`RenderState` is documented as *"Incremental render state updates for custom renderers"*). That matches
  `full.md` §5.4's parser/state-vs-rendering requirement and the Android-rendering-later goal equally well
  before and after a swap. Windows/WebAssembly are upstream-supported (`README.md:157-160`) — **but "usable
  off-Linux" is a build/packaging question we would inherit, and neither binding ships a prebuilt, so it is
  `inferred`, not verified, for any target beyond this host.**

---

## 7. Q7 — Stage B: the smallest verifiable spike

### 7.1 What stage B must prove (and nothing more)

**One question:** *can a libghostty-vt-backed model drive the existing `apps/linux` `term/**` interface,
with the existing tests plus a named slice of the TERM-002 matrix, on this host?*
It is **not** a rendering-fidelity task, **not** an ADR change, and **not** a production integration.

### 7.2 Shape (recommended)

A throwaway spike crate with its own workspace under `spikes/**` (exempt from the file-size caps;
precedent `spikes/terminal-engine-phase2`), containing:

1. **`model.rs`** — a `GhosttyModel` exposing the **identical** interface as
   `apps/linux/src/term/model.rs`: `new(TerminalSize)`, `process(&[u8])`, `resize(TerminalSize)`,
   `snapshot() -> ScreenSnapshot`, `snapshot_at(isize) -> ScreenSnapshot`, `history_len() -> usize`,
   `size() -> TerminalSize`, built on the existing engine-neutral types (`snapshot.rs` is copied verbatim
   as a spike-local module, or the spike depends on `relayable-linux`'s `term` module **read-only**).
2. **A dependency choice, recorded as a one-line decision in the spike README:** binding A
   (`libghostty-vt 0.2.2`) vs binding B (`ghostty-vt 0.7.2`). Binding A is the default because it is the
   maintained one; binding B is the fallback if A's missing licence file or lack of `ZIG`/`mise`
   discovery becomes the blocking issue (its `mise` fallback and bundled licences make it the
   lower-friction build).
3. **`tests.rs`** — the eight existing `apps/linux/src/term/tests.rs` assertions, copied, plus:
   - alt-screen enter/leave (feed real captured bytes from `spikes/terminal-engine-phase2/results/raw/`)
   - OSC 8 hyperlink URI retrieval (`GridRef::hyperlink_uri`) against the captured `osc8_ls.seq`
   - bracketed-paste **mode** (2004) and — new capability — `ghostty_paste_encode(bracketed=true)`
   - chunk-boundary invariance (the same bytes fed 1 byte at a time vs whole)
   - a **measured** scrollback bound: set `set_scrollback_max_lines`/`_max_bytes`, feed a bounded large
     stream, and record RSS — the number `full.md` §6.4/§12 requires and that ADR-003 currently leaves
     open.
4. **`provision.sh`** — the A1+A2 steps (fetch Zig, fetch source, `zig fetch`/`--system` seed), idempotent,
   printing every version and sha256 it used.

### 7.3 Variant worth costing but not required first

**Option (c)** — vendor the tagged `v1.3.1` engine (Zig **0.15.2**) and write our own ~100-line Zig build +
C-ABI shim, the `Xuanwo/gpui-ghostty` shape. Cost: we own the boundary and get a tag instead of `main`.
Benefit: no third-party binding, no third-party licence question, and the ability to withhold parts of
`terminal/main.zig` (`src/lib_vt.zig:11-14` says the public module exists precisely for that). **Also adds a
fourth Zig version to track.** Cost it in stage B only if binding A fails.

### 7.4 Exact commands, acceptance evidence, timebox

```shell-session
# -- provisioning (needs approval A1 + A2) --------------------------------
mise install zig@0.16.0 && zig version                       # expect 0.16.0
mkdir -p /tmp/relayable-term003/{src,zig-system}
curl -sSL -o /tmp/relayable-term003/libghostty-vt-source.tar.gz \
  https://tip.files.ghostty.org/<40-char-sha>/libghostty-vt-source.tar.gz
sha256sum /tmp/relayable-term003/libghostty-vt-source.tar.gz  # record it; verify it exists FIRST
tar xzf ... -C /tmp/relayable-term003/src
# -- offline build --------------------------------------------------------
ZIG_GLOBAL_CACHE_DIR=… zig build -Demit-lib-vt -Dapp-runtime=none \
  -Dtarget=x86_64-linux-gnu -Dcpu=baseline -Doptimize=ReleaseFast \
  --prefix …/install --system …/zig-system
# -- bind + measure -------------------------------------------------------
GHOSTTY_SOURCE_DIR=…/src GHOSTTY_ZIG_SYSTEM_DIR=…/zig-system \
  CARGO_NET_OFFLINE=true cargo test -p term003-spike --offline
/usr/bin/time -v cargo build -p term003-spike 2>&1 | tail -20   # wall clock + peak RSS
du -sh ~/.local/share/mise/installs/zig/0.16.0 …/target …/zig-cache …/zig-system
nm -D …/install/lib/libghostty-vt.so | grep -c ghostty_terminal_   # prove the terminal API is present
```

**Acceptance evidence the spike must produce** (all as raw, committed-under-`spikes/**` output):

1. `zig version` = `0.16.0`; the exact **40-char** ghostty commit; the source tarball **sha256**; and the
   crate version + `.crate` sha256 of the binding.
2. First-build and cached-build **wall clock**, **peak RSS**, and disk usage of every cache directory
   (replaces §3.5's estimates).
3. Test output: the 8 existing assertions + the new ones, with a **count**, a pass/fail line, and the
   **exact diff** for any assertion that had to be recalibrated (especially the two colour-semantics tests
   in `apps/linux/src/term/tests.rs:19-53`).
4. A **measured** scrollback RSS number with the bound set (feeds `full.md` §6.4).
5. `nm` evidence that the linked artifact exports the terminal/grid/render-state symbols (and, for binding
   A, that the pinned commit's headers match the pinned commit's library — the crate's `build.rs` already
   asserts the header exists at `build.rs:278-282`).
6. A `cargo tree`/licence audit of the spike's transitive crates (ADR-003's Consequences already flags this
   as `UNVERIFIED` for the current stack).
7. An explicit statement of what was **not** proven (below).

**Timebox:** one implementer session for provisioning + first build (measure it — that is the deliverable),
one for the model + tests. **Stop and report** if either approval is refused, if the pinned commit has no
per-commit tarball and a clone is unavailable, or if the safe crate's `!Send` types conflict with the
`apps/linux` state shape in a way that needs a design change rather than a spike-local workaround.

### 7.5 What stays unproven even after a successful spike

- **Rendering fidelity** to the reference videos — the engine does not render; the GPUI painter is
  unchanged and unmeasured by this spike (`full.md` §0.2/§2.4).
- **Real-program behaviour** (`vim`, `htop`, `tmux`) under the new engine — the phase-2 captures can be
  *replayed*, but live `stty`/SIGWINCH/reflow behaviour is a separate run.
- **Input encoding** end-to-end (key/paste/mouse → PTY → program), which is where libghostty is strongest
  and where TERM-002 measured nothing.
- **Android**, for the host reasons in §6.1.
- **Performance** against `alacritty_terminal` (no benchmark, and a benchmark is a *different* task).
- **Whether the terminal API ever ships in a tag** — upstream's own words: *"We haven't tagged libghostty
  with a version yet"*.

### 7.6 Fallback if the spike fails, or if the approvals are refused

**Keep ADR-003's pins unchanged.** Concretely: no ADR amendment, `alacritty_terminal 0.26.0` +
`vte 0.15.0` + `portable-pty 0.9.0` continue to be the engine, the committed TERM-002 evidence
(32 PASS / 1 UNSUPPORTED / 0 FAIL) remains the only engine-level evidence in the repository, and
reconsideration trigger #10 sub-condition 3 stays **unmet** with a *measured* reason recorded instead of an
inferred one. The capability gap that remains unserved is the **input side** (key/cursor-mode encoding,
bracketed-paste payload encoding, mouse encoders) — and, critically, **there is no cheap additive path
either**, because any libghostty binding carries the same Zig 0.16.0 build requirement (§4.4b). A second
fallback, if the *build* is the only blocker, is option (c) with `v1.3.1`/Zig 0.15.2 — same engine, one tag
older, no third-party binding.

---

## 8. Search trail (URLs, status, retrieval date)

All HTTP `2026-10-16` (UTC), read-only, no proxy, no install, no system change.

| URL | Status | Purpose |
|---|---|---|
| `crates.io/api/v1/crates/libghostty-vt/0.2.2/download` | 200 | binding A tarball (sha256 `16c7287a…`) |
| `crates.io/api/v1/crates/libghostty-vt-sys/0.2.2/download` | 200 | binding A sys tarball (sha256 `df233d9b…`) |
| `static.crates.io/crates/ghostty-vt/ghostty-vt-0.7.2%2Bghostty.1.3.2-main.b40acce.crate` | 200 | binding B tarball (sha256 `c056e7b0…`) |
| `tip.files.ghostty.org/b40acce…/libghostty-vt-source.tar.gz` | **200**, 4,032,640 B, sha256 `a623aa0b…` | immutable per-commit source artifact |
| `tip.files.ghostty.org/{877dda7…,949149d…,3425025e…}/libghostty-vt-source.tar.gz` | **404** ×3 | availability is **not** uniform |
| `tip.files.ghostty.org/{a887df42…,b3dc3d79…}/libghostty-vt-source.tar.gz` | 200 ×2 | more availability data |
| `github.com/xipeng-jin/libghostty-vt-prebuilt` | 200 | third-party prebuilt precedent (§1.4) |
| `api.github.com/repos/xipeng-jin/zed/branches/migration%2Flibghostty` | 200 | fork branch, last commit 2026-07-22 |
| `api.github.com/repos/zed-industries/zed/contents/crates/ghostty_vt_sys` | **404** | upstream Zed has **not** adopted it |
| `xuanwo.io/2026/01-gpui-ghostty/` | 200 | GPUI + libghostty precedent (§1.4) |
| `raw.githubusercontent.com/Xuanwo/gpui-ghostty/main/README.md` | 200 | its pinned Zig 0.14.1 / ghostty v1.2.3 |
| `fedora.pkgs.org/rawhide/terra/x86_64/libghostty-vt-tip-static-…rpm.html` | **402** | third-party RPM — **not** re-verified (§1.4) |

Git-level evidence is cited inline as `file:line` at the refs in §1 and is re-derivable from the three
clones.

---

## 9. Validation record and limitations

- **Files written by this task:** `docs/research/libghostty-adoption.md` (this file) and a dated
  corrections block at the top of `docs/research/libghostty.md` (§1.2 items C1–C4). No code, no ADR, no
  `docs/agent-loop/**`, no `AGENTS.md`, no manifest, no `.gitignore` was touched. No upstream source was
  copied into the repository.
- **No build, link, or run of any engine was performed.** No Zig was installed. Every build conclusion is
  `inferred` from upstream build sources plus this host's observed state and is **labelled** as such
  (§3.5). No benchmark, memory number, screenshot or rendering claim is made anywhere in this file.
- **Clones are outside the tree** at `/home/sulthonuladib/work/vendor/{ghostty,libghostty-rs,gpui-cn}`
  (201 MB / 3.1 MB / 6.2 MB), fetched read-only; they are not tracked and must not be
  committed (`.gitignore` intentionally untouched — I do not own it).
- **Known limitations:** (1) the symbol counts in §2.2 are header-declaration counts from a regex, not a
  symbol table; (2) no prebuilt/distro artifact was verified as trustworthy, only catalogued (§1.4, low
  confidence); (3) the `!Send` compatibility conclusion in §4.4 is `inferred`, not compiled; (4) the
  "no Android SDK/NDK on this host" observation contradicts `AGENTS.md`'s environment table and I did not
  re-run ENV-001 to reconcile it; (5) `tip.files.ghostty.org` upload cadence was probed with six samples
  only — the *reason* for the 404s is **unknown**; (6) no Rust code was compiled, so no claim is made about
  binding A vs B compiling.
- **What this file deliberately does not do:** recommend a pin, amend ADR-003, or decide replace-vs-additive.
  It reports the evidence, the adoption path, the blocker list, the controls an amendment would touch, and
  the single decision the lead must make.
