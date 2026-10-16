# GPUI Kit evaluation — license, `gpui = "=0.2.2"` compatibility, components, and fidelity impact

Status: **research artifact, evidence only.** This file answers task `RESEARCH-001`
(`docs/agent-loop/TASKS.md`) for `longbridge/gpui-kit` (docs site `gpui-kit.com`). It does **not**
change any ADR, pin, `Cargo.toml`, or `apps/linux` code; it supplies evidence for the lead and
relates to ADR-003 "Option J", which already rejected this family. Every claim below carries an
evidence class and a confidence marker. Nothing here is a recommendation to reverse an accepted
ADR.

**Retrieval window:** all HTTP fetches in this file were made `2026-10-16T04:52Z`–`04:56Z` (UTC).
**Session start (network probe):** `2026-10-16T04:52:30Z`.

---

## 1. Evidence basis and conventions

### 1.1 Evidence classes (kept separate throughout)

- **`[HTTP]`** — fetched live over read-only HTTP(S) this session (crates.io API, `static.crates.io`
  `.crate` tarballs, `api.github.com`, `raw.githubusercontent.com`, `gpui-kit.com`). URL + retrieval
  date + status code are recorded in §15.
- **`[LOCAL]`** — read from this machine's disk, from either (a) this worktree's repository files, or
  (b) the prior agent's published-crate cache under
  `/home/sulthonuladib/.cargo/registry/src/index.crates.io-*/…` and
  `/home/sulthonuladib/.cargo/registry/index/index.crates.io-*/.cache/gp/ui/`. Exact paths named.
- **`[INF]`** — inference by this author, explicitly labelled.

Raw fetched bodies and extracted crates are saved under `/tmp/gpui-kit-evidence/` (outside the repo);
the directory listing is in §16.2.

### 1.2 Confidence labels

`high` / `medium-high` / `medium` / `low`; `UNVERIFIED` (no acceptable evidence);
`not observed`; `inferred` (a conclusion drawn by this author, not stated by a source);
`unverified-by-build` (would require compiling/linking, which this task forbids).

### 1.3 Relationship to the prior brief (`result:gpui#1`)

A prior read-only agent (`researcher:gpui`, `/tmp`-scoped) produced a brief from local artifacts
only, because its session lacked network tools. Its central finding — every `gpui-kit` version
aliases the dependency key `gpui` to package `gpui-pre` with an exact pin — is **independently
confirmed here against live registries at the current version (0.7.3 → `gpui-pre = "=0.3.9"`)**.
Where this file corrects or extends that brief, it is marked. The most important extensions: live
crates.io version data to 0.7.3 (prior snapshot ended at 0.7.0), the GitHub repo/release/issue data,
the docs-site content, the `gpui-fast` optional backend, and confirmation that both `gpui-pre` Zed
revisions exist upstream.

### 1.4 Network capability (probe result — this session succeeded)

Contrary to `result:gpui#1` §0, this session **does** have read-only network egress via `bash` +
`curl`/`chromium`. Probe results (all `2026-10-16T04:52Z`):

| Probe | Result |
|---|---|
| `curl https://crates.io/api/v1/crates/gpui-kit` (no UA) | `403` |
| same with a `User-Agent` header | `200` |
| `curl https://api.github.com/repos/longbridge/gpui-kit` | `200` |
| `curl https://gpui-kit.com/docs` | `301` → `/docs/`; `200` after `-L` |
| `curl https://static.crates.io/crates/gpui-kit/gpui-kit-0.6.6.crate` | `200` |
| `curl https://raw.githubusercontent.com/longbridge/gpui-kit/main/README.md` | `200` |

`chromium 152.0.7977.82` is present. `gpui-kit.com/docs` is **server-rendered** (a plain `curl`
already returns the full ~100 KB HTML; headless `chromium --dump-dom` returned the same content
plus a `dark` class). No JS-only rendering problem existed.

---

## 2. What GPUI Kit is

### 2.1 Identity (`high`)

- **Product name:** GPUI Kit.
- **Vendor/owner:** **Longbridge** (`github.com/longbridge/gpui-kit`). License headers read
  `Copyright 2024 - 2026 Longbridge <https://longbridge.com>`.
- **Homepage:** `https://gpui-kit.com`. **Docs:** `https://gpui-kit.com/docs`.
  **Repository:** `https://github.com/longbridge/gpui-kit` (default branch `main`).
- **crates.io description (`high`):** *"GPUI Kit: one dependency for building desktop applications
  with GPUI, GPUI Base, GPUI Component and default assets."*
- **What it is (`high`, docs):** *"GPUI Kit is a Rust desktop application framework built on GPUI."*
  It is an umbrella facade over Zed's GPUI plus the vendor's own crates. The docs describe five
  layers: `gpui`, `gpui-kit`, `gpui-base`, `gpui-component`, `gpui-shell`.
- **Prior brand (`high`):** the project was previously **GPUI Component**; the long-lived crates are
  `gpui-component` / `gpui-component-macros` (first published `2025-02-06` and `2025-10-09`
  respectively), while the umbrella `gpui-kit` first appeared `2026-09-03`.

### 2.2 The crate family (`high`)

`[HTTP]` crates.io API, `2026-10-16`:

| Crate | Latest | Latest published (UTC) | Repository | License field |
|---|---|---|---|---|
| `gpui-kit` | **0.7.3** | 2026-10-12T07:27:07Z | longbridge/gpui-kit | Apache-2.0 |
| `gpui-base` | **0.7.3** | 2026-10-12T07:24:03Z | longbridge/gpui-kit | Apache-2.0 |
| `gpui-component` | **0.7.3** | 2026-10-12T07:25:17Z | longbridge/gpui-kit | Apache-2.0 |
| `gpui-component-macros` | **0.7.3** | 2026-10-12T07:23:22Z | none in metadata | Apache-2.0 |
| `gpui-kit-assets` | **0.7.3** | 2026-10-12T07:23:18Z | longbridge/gpui-kit | Apache-2.0 |
| `gpui-pre` | **0.3.9** | 2026-10-12T00:44:57Z | zed-industries/zed | Apache-2.0 |
| `gpui-omarchy` (3rd party) | **0.2.1** | 2026-10-05T03:08:54Z | huacnlee/gpui-omarchy | **MIT** |

Two further crates are named by the vendor and are relevant but **were not on the mandated list**:
`gpui-fast` (`[HTTP]` latest **0.1.8**, 2026-10-14, `longbridge/gpui-fast`, Apache-2.0) — an
optional alternative GPUI backend, see §2.3 — and `gpui-shell`, named in the docs but **not queried
here** (`UNVERIFIED`).

**Publisher (`high`):** every family version's `published_by` is **`huacnlee`** (the top contributor,
1488 commits). `gpui-pre`, `gpui-fast`, and `gpui-omarchy` are all also published by `huacnlee`.

### 2.3 The optional `gpui-fast` backend (`high` for existence, `medium` for implications)

`[HTTP]` the docs and the `gpui-kit` 0.7.3 README/manifest document an optional alternative backend:
enabling the `gpui-fast` Cargo feature switches the core, platform, and web backends to
`gpui-fast` / `gpui-fast-platform` / `gpui-fast-web` (`longbridge/gpui-fast`, a Longbridge fork
with *"rendering and layout optimizations"*). `gpui` (the `gpui-pre` alias) remains a build
dependency; the Kit exposes only Fast types. This is a **second** vendor-controlled GPUI lineage on
top of `gpui-pre`. It is relevant to ADR-003 reconsideration trigger #8 (lineage stability) because
it increases the number of artifacts a consumer would need to trust.

---

## 3. Licensing

### 3.1 crates.io license metadata (`high`)

`[HTTP]` `https://crates.io/api/v1/crates/<name>` and `/versions`, `2026-10-16`:

- Every version of `gpui-kit`, `gpui-base`, `gpui-component`, `gpui-component-macros`,
  `gpui-kit-assets`, and `gpui-pre` carries per-version `license = "Apache-2.0"`. `gpui-omarchy`
  carries `"MIT"`. `gpui-fast` carries `"Apache-2.0"`.
- `rust_version` is **`null`** for every version of every family crate (crates.io has no recorded
  MSRV). See §7 for the docs' informal statement.
- The crates.io JSON API per-version objects in this sample did **not** include a
  `published_by` user object on the `/versions` list rows; publisher identity was read from the
  top-level `.crate` object (`versions[].published_by` was populated there as `huacnlee`).

### 3.2 License-file presence per published crate at the CURRENT version (`high`)

`[HTTP]` contents of the actual `static.crates.io` `.crate` tarballs for **0.7.3 / 0.3.9**,
extracted and inspected `2026-10-16`:

| Crate @ current version | Declared `license` | License file(s) actually shipped | Copyright line |
|---|---|---|---|
| `gpui-kit` 0.7.3 | Apache-2.0 | **NONE** (no `LICENSE*`, no `NOTICE*` in the tarball) | — |
| `gpui-base` 0.7.3 | Apache-2.0 | `LICENSE-APACHE` | `Copyright 2024 - 2026 Longbridge` |
| `gpui-component` 0.7.3 | Apache-2.0 | `LICENSE-APACHE` | Longbridge |
| `gpui-component-macros` 0.7.3 | Apache-2.0 | `LICENSE-APACHE` | Longbridge |
| `gpui-kit-assets` 0.7.3 | Apache-2.0 | **only `LICENSE-LUCIDE`** (no Apache text) | — |
| `gpui-pre` 0.3.9 | Apache-2.0 | `LICENSE-APACHE` | `Copyright 2022 - 2025 Zed Industries, Inc.` |
| `gpui-fast` 0.1.8 | Apache-2.0 | `LICENSE` | `Copyright 2022 - 2025 Zed Industries, Inc.` + `Copyright 2026 Longbridge.` |
| `gpui-omarchy` 0.2.1 | MIT | (*not extracted this session*) — prior `[LOCAL]` read at 0.1.3 found `LICENSE` | — |

**The per-crate license-file gap is CONFIRMED at the current version:** `gpui-kit` itself declares
Apache-2.0 in its manifest but ships **no** license file at all, while its four sibling family crates
do. `gpui-kit-assets` ships only the Lucide icon license, not the Apache text it declares.

The repository **root** *does* carry licensing documents (`[HTTP]` root listing, 2026-10-16):
`LICENSE-APACHE` (Longbridge copyright), `LICENSE-DOCS.md` (CC BY 4.0 prose license), and
`CONTRIBUTING.md`. There is **no** root `LICENSE` file — a `raw.githubusercontent.com/.../LICENSE`
fetch returned `404`, which is why the GitHub API reports `license.spdx_id = "NOASSERTION"`. The
discrepancy is therefore between the *repository* (has `LICENSE-APACHE`) and the *published
`gpui-kit` crate* (ships nothing); the crate tarball does not embed the repo-root license file.

### 3.3 The `gpui-pre` / Zed Apache-2.0 lineage (`high`)

`[HTTP]` `gpui-pre-0.3.9/Cargo.toml.orig`: `repository = "https://github.com/zed-industries/zed"`,
`license = "Apache-2.0"`, `authors = ["Nathan Sobo <nathan@zed.dev>"]`, and
`[package.metadata.gpui-pre]` with `zed-crate = "gpui"`, `zed-version = "0.2.2"`,
`zed-rev = "93d745668805029476e6d7fbb19652116fed51de"`. The crate ships Zed's `LICENSE-APACHE`
intact. The vendor's `gpui-component` README states verbatim: *"The `gpui-pre-*` crates are snapshots
of it, published with Zed's license and notices intact."*

### 3.4 `NOTICE` presence (`high`)

`[HTTP]` recursive search of all extracted current-version tarballs (`find … -iname 'NOTICE*'`):
**zero `NOTICE` files** in `gpui-kit` 0.7.3, `gpui-base` 0.7.3, `gpui-component` 0.7.3,
`gpui-component-macros` 0.7.3, `gpui-kit-assets` 0.7.3, `gpui-pre` 0.3.9, or `gpui-fast` 0.1.8.
(`gpui-fast`'s `include` list names `/NOTICE`, but no `NOTICE` file exists in the published tarball —
a packaging inconsistency, `medium`.) This matters because ADR-003's Option H chain treats
`NOTICE` absence as part of the pin decision.

### 3.5 License assessment (`medium-high`)

The **declared** license story is clean: Apache-2.0 across the family, with Zed's Apache-2.0
inherited through the `gpui-pre` snapshot and Lucide's terms for icons. The **distribution** story
has a real gap for the umbrella crate: `gpui-kit` declares Apache-2.0 but the artifact a cargo
consumer actually receives contains no license text. Whether that is a legal defect or merely an
inconsistent packaging choice is a question for the lead, not this research artifact; but it is a
verifiable fact and it is the same fact ADR-003 recorded at 0.7.3 (now reproduced independently).

---

## 4. Versions, release cadence, recency, contributors

### 4.1 `gpui-kit` version history (`high`)

`[HTTP]` crates.io, `2026-10-16`. (Prior brief's snapshot ended at 0.7.0; these are current.)

| Version | Published (UTC) | Yanked |
|---|---|---|
| 0.1.0 (empty placeholder, no deps) | 2026-09-03T02:32:19Z | no |
| 0.6.0 | 2026-09-03T13:10:34Z | no |
| 0.6.1 | 2026-09-09T09:52:31Z | no |
| 0.6.2 | 2026-09-18T09:20:06Z | no |
| 0.6.4 | 2026-09-18T16:37:38Z | no |
| 0.6.5 | 2026-09-21T13:06:13Z | **yes** |
| 0.6.6 | 2026-09-21T13:27:45Z | no |
| 0.7.0 | 2026-09-28T03:14:47Z | no |
| 0.7.1 | 2026-10-05T02:38:26Z | no |
| 0.7.2 | 2026-10-12T04:08:49Z | no |
| **0.7.3** | **2026-10-12T07:27:07Z** | no |

There is no 0.6.3. `gpui-kit-assets` and `gpui-base` mirror the same ladder; `gpui-component-macros`
also yanks none of its 0.6.5. **Yanked versions (`high`):** `0.6.5` for `gpui-kit`, `gpui-base`,
`gpui-component`, and `gpui-kit-assets` (but **not** `gpui-component-macros`); `gpui-component` also
yanks its old `0.2.0-test*`/`0.3.0-test*` line. `gpui-pre` has 10 versions (0.3.0–0.3.9), none yanked.

### 4.2 Cadence and recency (`high`)

- `gpui-kit` published **11 versions in 39 days** (2026-09-03 → 2026-10-12); `gpui-pre` published
  **10 versions in 39 days** (0.3.0 → 0.3.9). The 0.6.5 yank was 21 minutes before the 0.6.6
  republish (2026-09-21T13:06 → 13:27) — an active hotfix cadence.
- The minor number tracks the `gpui-pre` generation (0.6.x → `gpui-pre` 0.3.x; 0.7.x → 0.3.7+), not
  a per-crate semver maturity signal (`inferred`, `medium-high`).
- **GitHub (`high`, `[HTTP]` api.github.com, 2026-10-16):** `created_at` 2024-06-13,
  `updated_at` 2026-10-16T04:40Z, `pushed_at` **2026-10-16T03:54Z** (i.e. pushed within the hour
  before retrieval), `stargazers_count` **17,664**, `forks_count` 1,074, `open_issues_count` 115,
  `archived` false.
- **Recent commits (`high`):** most recent HEAD-ward commits are dated 2026-10-15 (e.g.
  `1b6059c133` "window_border: Don't offer resizing for a window that isn't resizable (#3525)",
  2026-10-15T11:28:49Z). The project is **very active**.
- **Releases/tags (`high`):** 16 GitHub releases; latest `v0.7.3` @ 2026-10-12T07:27:58Z. 27 tags,
  including `-preview*` tags back to the old line. Release tags track crates.io versions closely.
- **Contributors (`high`):** the API returned the 100-per-page cap. Top: `huacnlee` 1488,
  `madcodelife` 469, `ihavecoke` 37, `ylinwind` 37, `zanmato` 28, … `Copilot` 13,
  `dependabot[bot]` 11. Contributor count is effectively ≥100 (`low` on the true total because the
  API page was capped).

### 4.3 `gpui-pre` / upstream base (`high`)

- `gpui-pre` `description` in current metadata: *"Zed's GPU-accelerated UI framework
  (gpui-pre snapshot of zed@93d7456)"*. Its manifest names `zed-version = "0.2.2"` and
  `zed-rev = "93d745668805029476e6d7fbb19652116fed51de"`.
- `[HTTP]` **the rev exists upstream:** `api.github.com/repos/zed-industries/zed/commits/93d7456…`
  → `200`, committed 2026-10-10T22:13:37Z ("git_ui: Fix commit editor button hover colors (#65448)").
- `[HTTP]` **the claimed upstream version matches:** `crates/gpui/Cargo.toml` at that rev has
  `name = "gpui"`, `version = "0.2.2"`, `license = "Apache-2.0"`.
- The prior version line's rev also checks out: `gpui-pre` 0.3.6 declared
  `zed-rev = "bcf6582…"`, and that commit exists upstream (2026-09-21T02:25:54Z).
- **What remains unverified:** whether `gpui-pre` 0.3.9's source is **byte-identical** to upstream
  `gpui 0.2.2` at `93d7456` modulo the documented rename/manifest adjustments. The vendor documents
  *"The publication process adjusts package names and dependency manifests for crates.io"*. A source
  diff was not performed (`UNVERIFIED`).

### 4.4 The repo's own `gpui = "=0.2.2"` pin, independently re-verified (`high`)

`[HTTP]` crates.io `gpui` 0.2.2: `checksum =
979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`, which **exactly matches the
checksum recorded in ADR-003**. The extracted upstream 0.2.2 crate has `[lib] name = "gpui"`,
`path = "src/gpui.rs"`. This confirms the repo's pinned artifact identity and the `apps/linux`
`libc 0.2.189` remedy context in `apps/linux/README.md` §12.

---

## 5. The decisive question: can GPUI Kit build against `gpui = "=0.2.2"`?

### 5.1 Literal dependency strings at the CURRENT version (`high`)

`[HTTP]` extracted from `static.crates.io/crates/gpui-kit/gpui-kit-0.7.3.crate`, file
`gpui-kit-0.7.3/Cargo.toml` (the **normalized** manifest cargo actually resolves, not the
workspace-inheriting `.orig`), quoted verbatim:

```toml
[dependencies.gpui]
version = "=0.3.9"
package = "gpui-pre"

[dependencies.gpui-base]
version = "0.7.1"

[dependencies.gpui-component]
version = "0.7.1"
optional = true

[dependencies.gpui_macros]
version = "=0.3.9"
package = "gpui-pre-macros"

[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies.gpui_platform]
version = "=0.3.9"
features = ["font-kit", "x11", "wayland", "runtime_shaders"]
package = "gpui-pre-platform"

[target.'cfg(target_family = "wasm")'.dependencies.gpui_web]
version = "=0.3.9"
package = "gpui-pre-web"
```

`gpui-base` 0.7.3 and `gpui-component` 0.7.3 are identical in form: `gpui = { version = "=0.3.9",
package = "gpui-pre" }`, plus `gpui_macros → gpui-pre-macros =0.3.9` and
`sum-tree → gpui-pre-sum-tree =0.3.9`. `gpui-base` also carries `reqwest_client →
gpui-pre-reqwest-client` (non-optional in `gpui-component`, dev-only in `gpui-base`).

**Requirement over time (`high`, `[HTTP]` crates.io + `[LOCAL]` prior cache):**

| `gpui-kit` | gpui requirement (all `package = "gpui-pre"`) | Form |
|---|---|---|
| 0.6.0, 0.6.1 | `^0.3.1` | caret on `gpui-pre` |
| 0.6.2, 0.6.4 | `^0.3.5` | caret on `gpui-pre` |
| 0.6.5, 0.6.6 | `=0.3.6` | **exact** |
| 0.7.0 | `=0.3.7` | exact |
| 0.7.1, 0.7.2 | `=0.3.8` | exact |
| current **0.7.3** | **`=0.3.9`** | exact |

Each Kit release exact-pins the matching `gpui-pre` snapshot (`[HTTP]` manifest of every version:
0.7.1 and 0.7.2 both fetched and confirmed `=0.3.8`; 0.7.0 `=0.3.7` and 0.7.3 `=0.3.9`). The
requirement is never a range at the 0.7.x line.

### 5.2 No gpui-kit version depends on the package `gpui` (`high`)

`[HTTP]` a recursive `grep -rn 'package = "gpui"'` over all extracted current-version manifests
returns **only** `package = "gpui-pre"` matches. The dependency **key** is `gpui`; the **package**
is `gpui-pre`. There is no version, at any point in the family history, whose `gpui` dependency
resolves to the real crates.io `gpui` package. (The prior agent found the same at 0.6.6; this
extends it to 0.7.3.)

### 5.3 The artifact identity problem (`high`)

Both packages declare `[lib] name = "gpui"` and `path = "src/gpui.rs"`:

- `[HTTP]` upstream crates.io `gpui` 0.2.2 → package `gpui`, `[lib] name = "gpui"`.
- `[HTTP]` `gpui-pre` 0.3.9 → package `gpui-pre`, `[lib] name = "gpui"`.

So both compile to an extern crate named `gpui`, but from two **distinct package identities and two
distinct compilations**. `gpui-kit` re-exports the `gpui-pre` compilation as `gpui_kit::*`
(`gpui-kit-0.7.3/src/lib.rs`: `pub use ::gpui::*;`). Types from the two compilations — `App`,
`Window`, `Entity`, `Context`, elements, actions — are unrelated, even though both descend from Zed
`gpui` 0.2.2.

### 5.4 The vendor documents this exact failure mode (`high`)

`[HTTP]` the docs-site "Mobile" page (in `/llms-full.txt`) states, verbatim:

> `gpui = { package = "gpui-pre", version = "=0.3.4", default-features = false }`
> … The current GPUI Kit checkout uses `gpui-pre 0.3.9`, while this pinned mobile platform and
> renderer use `0.3.4`. **Cargo can select both versions, producing incompatible GPUI types**; …
> | Rust reports two versions of GPUI or mismatched `App`/`Window` types | Check the resolved
> `gpui-pre` packages. |

`[HTTP]` the installed `gpui-kit` 0.7.3 README states: *"`gpui-kit` depends on the matching set of
GPUI crates, so an application never lists GPUI itself."* And `gpui-kit-0.7.3/src/lib.rs` ships a
bespoke `actions!` macro with the comment: *"GPUI's original macro spells its derive as
`gpui::Action`, which does not resolve when GPUI is consumed solely through this facade."*

### 5.5 Verdict (`high` for the manifest/type-level conclusion)

**No `gpui-kit` version can be used alongside the repository's pinned direct dependency
`gpui = "=0.2.2"`.** Cargo would resolve two distinct packages — `gpui` 0.2.2 and `gpui-pre`
0.3.9 — that both export an extern crate called `gpui`; the Kit's public API is the `gpui-pre`
compilation; and the vendor documents that mixing them yields incompatible GPUI types. Adopting
GPUI Kit therefore means **replacing** the GPUI artifact (ADR-003 Option H) with the `gpui-pre`
facade family — an ADR-level artifact decision, not a worker-level change. The underlying source
base is the same upstream version (`zed gpui 0.2.2`), so this is a package-identity/lineage change,
not an upstream API-version jump — but the artifacts, checksums, and license files differ.

### 5.6 Build-level questions (`unverified-by-build`)

None of the following were executed (builds are forbidden by this task):

1. Whether a manifest that lists **both** `gpui = "=0.2.2"` and `gpui-kit = "=0.7.3"` even resolves,
   or fails earlier at Cargo's extern-name disambiguation. `[INF]` predicted: it resolves as two
   packages and fails at the type level (and possibly at link time with duplicate crate `gpui`).
2. Whether `gpui-pre 0.3.9` is API-identical to `gpui 0.2.2` such that Relayable's existing
   `apps/linux` source compiles unchanged after a pin swap.
3. Whether the Kit builds on this host (Wayland/Omarchy, rustc 1.98.1) under the `libc 0.2.189`
   hold, and whether any `xattr`/`ENOATTR` analogue returns.
4. The real consumer `cargo tree -i libc` / lockfile resolution for a Relayable app.
5. Any rendered-fidelity comparison of Kit widgets against the reference stills.

---

## 6. Component / API inventory relevant to `apps/linux`

### 6.1 What exists (`high`, from `[HTTP]` 0.7.3 crate source + `[HTTP]` docs)

`gpui-component` 0.7.3 ships ~75+ documented components. Module directories actually present in
`gpui-component-0.7.3/src/`:

`accordion, alert, attachment, avatar, badge, breadcrumb, bubble, button, carousel, chart, checkbox,
clipboard, collapsible, color_picker, combobox, command, description_list, dialog, diff, dock,
empty, form, group_box, highlighter, history, hover_card, icon, input, inspector, kbd, label, link,
list, marker, menu, message, message_scroller, native_menu, notification, pagination, plot,
popover, progress, questionnaire, radio, rating, resizable, root, scroll, searchable_list, select,
separator, setting, sheet, shimmer, sidebar, skeleton, slider, speech, spinner, status_bar, stepper,
switch, tab, table, tag, text, theme, time, title_bar, toolbar, tooltip, touch_selection, tree,
virtual_list, window_border, window_ext`.

Mapping onto the reference product's needs (`high` for existence, names as found in source/docs):

| Need | Exists | Module / type (as found) |
|---|---|---|
| Theme provider + tokens | yes | `gpui-base::theme_tokens::{SemanticThemeTokens, ColorTokens, RadiusTokens, SpacingTokens, TypographyTokens, ShadowTokens}`; `gpui-component::theme::{Theme, ThemeRegistry, ActiveTheme, ThemeMode}`; `Theme::change`, `Theme::update`, `ThemeRegistry::watch_dir` |
| Menu bar / native menu | yes | `menu::AppMenuBar`; `native_menu::NativeMenu` (macOS/Windows native; **Linux = GPUI `PopupMenu` fallback in `Root`'s overlay**, not an OS menu) |
| Context / dropdown / popup menus | yes | `menu::{ContextMenu, DropdownMenu, PopupMenu}`, `dropdown_button` |
| Command palette | yes | `command::{Command, CommandState, CommandEntry, CommandGroup}` |
| Split panes / docking | yes | `dock::{DockArea, Panel, PanelView, DockPlacement, DockState}`; tree/state in `gpui-base::dock`; also `ResizablePanelGroup`/`resizable` |
| Tabs | yes | `tab::` |
| Dialogs / popovers / sheets / notifications | yes | `dialog::`, `alert_dialog` (base), `popover::`, `sheet::`, `hover_card::`, `notification::`, `alert::`, `tooltip::` |
| List / table / tree / virtual list | yes | `table::` (row+column virtualization, sorting, cell selection, resizable columns), `list::`, `tree::`, `virtual_list::`, `searchable_list::`, `scroll::` |
| Icons | yes | `icon::{Icon, IconName}` + `gpui-kit-assets` (Lucide bundle, `default-icons.txt`, `LICENSE-LUCIDE`) |
| Text / code editing | yes | `input::{Input, Textarea, Editor, EditorState, InputState}`, `highlighter/` (Tree-sitter), `text::`, `text_selection` |
| Keybinding helpers | partial | `kbd::Kbd` renders a `Keystroke` tag; action/keymap machinery comes from GPUI itself (`gpui-base::actions`, GPUI `KeyBinding`); no kit-level shortcut *manager*/settings-file abstraction |
| Window chrome (Linux) | yes | `title_bar::`, `status_bar::`, `window_border::{WindowBorder, window_border}`, `root::Root` |
| **Terminal / VT / PTY widget** | **NO** | see §6.2 |

Also present and potentially useful: `breadcrumb`, `pagination`, `stepper`, `progress`, `spinner`,
`skeleton`, `shimmer`, `slider`, `switch`, `checkbox`, `radio`, `select`, `combobox`,
`color_picker`, `chart`, `plot`, `carousel`, `diff`, `clipboard`, `link`, `separator`, `form`,
`setting`, `speech` (audio/ALSA), `attachment`, `marker`, `bubble`, `message_scroller`.

### 6.2 What does NOT exist: no terminal (`high`)

- `[HTTP]` `find gpui-kit-0.7.3 gpui-base-0.7.3 gpui-component-0.7.3 -iname '*terminal*' -o -iname
  '*pty*' -o -iname '*vt*'` → no terminal/PTY module.
- `[HTTP]` the docs component catalogue (`site_component.html`) contains **zero** occurrences of the
  word "terminal"; the word appears in the docs corpus only as incidental prose ("run in a
  terminal", "the terminal prints…", or a `QuestionnaireChoiceDefinition::new("terminal", …)` UI
  string).
- Therefore GPUI Kit provides **no** VT/ANSI parser, no scrollback, no PTY, and no terminal renderer.
  ADR-003's `alacritty_terminal`/`vte`/`portable-pty` boundary (and the `apps/linux/src/term/mod.rs`
  `TerminalModel`) would remain Relayable's own code regardless of any Kit adoption.

### 6.3 Theme catalogue (relevant to fidelity planning) (`high`)

The docs list **38** theme choices (36 JSON variants + Default Light/Dark), including terminal-adjacent
names such as `Asciinema`, `Solarized`, `Tokyo Night`, `Gruvbox`. This is a *different* visual
catalogue from the reference product's measured tokens (`docs/research/design-tokens.md`), so it is
not a shortcut to reference fidelity — see §10.

---

## 7. API stability and maturity

- **Version-history shape (`high`):** the umbrella `gpui-kit` went 0.1.0 (empty) → 0.6.0 → …→ 0.7.3
  in 39 days; the underlying `gpui-component` line goes back to 2025-02. Minor numbers track the
  `gpui-pre` snapshot generation.
- **Changelog (`high`):** the **repository** has `release-notes.md` (244 lines, fetched `200`);
  the **published crates** ship **no** `CHANGELOG*` and no `release-notes.md`. So a crate-only
  consumer does not receive change notes.
- **Stability/marketing quotes (`high`, verbatim):**
  - Docs "Maturity" table: *"**Stable** — Used by production desktop applications on macOS,
    Windows, and Linux."* Labels: Stable / Preview / Experimental / Showcase only /
    Platform-dependent.
  - Docs: *"GPUI Kit's desktop components run in production applications, including Longbridge's."*
  - `gpui-component` README: *"ships 75+ documented components and primitives, WebAssembly support,
    AccessKit accessibility, UI integration testing, and an optional JavaScript extension runtime."*
  - `gpui-base` README: *"The current Rust interface is defined by the source code and generated API
    documentation."*
  - Docs "Coding Guides" (line 5136 in the corpus): *"**Stable** | Used by production desktop
    applications on macOS, Windows, and Linux."*
- **Semver signals (`medium-high`):** dependencies are **exact-pinned** across the family
  (`=0.3.9`, `=0.3.9` for macros/sum-tree), which is anti-compatible by design but internally
  consistent; yanked releases exist (0.6.5) and are superseded without renumbering — normal
  pre-1.0. `gpui-fast` allows compatible `0.1.x` updates.
- **Docs completeness (`medium`):** docs are extensive (an `/llms-full.txt` full-docs dump exists;
  ~4.5 MB). But the prior brief found README/source mismatches at 0.6.6 (`prelude` referenced in a
  README but absent from `src/lib.rs`). At 0.7.3, the docs' own text now explicitly answers the
  `gpui-pre` question (§5.4), so the previous "`prelude`" mismatch may be moot but was **not
  re-checked** for 0.7.3 (`UNVERIFIED` for this version).

---

## 8. Tests, CI, and maintenance signals

- **Tests (`high`):** `gpui-kit` 0.7.3 ships a `tests/` tree (e.g. `window.rs`, `lifecycle.rs`,
  `components.rs`, `input/`, `ui.rs`, `root.rs`, `controls.rs`, `interactions.rs`, `rendering.rs`,
  `collections.rs`, `search.rs`, `text_selection.rs`, `disclosure.rs`, `overlays.rs`, `menu.rs`,
  `dock.rs`), many gated on `required-features = ["test-support", "component"]`. `gpui-base` and
  `gpui-component` also ship tests/benches.
- **Published testing guide (`high`):** the earlier `[LOCAL]` read found `TESTING.md` in the 0.6.6
  `gpui-kit` crate; the 0.7.3 crate ships a `test` module/guide (docs page `/docs/test` exists).
- **CI (`high` for existence):** the repo's `.github/workflows/` contains `ci.yml`,
  `release.yml`, `release-gpui.yml`, `release-shell.yml`, `release-website.yml`, `test-docs.yml`.
  `[HTTP]` `ci.yml` fetched `200`; it runs typos, a docs recipe check, and Rust jobs. The
  `gpui-component` README badge targets this `ci.yml`.
- **Last release (`high`):** crates.io `gpui-kit` 0.7.3 @ 2026-10-12T07:27Z; GitHub `v0.7.3` @
  2026-10-12T07:27:58Z. Repo `pushed_at` 2026-10-16T03:54Z.
- **Issue activity (`high`):** `[HTTP]` GitHub search API: **86 open**, **608 closed** issues
  (type:issue). Most recent issue update observed: `#3533` open, 2026-10-16T00:46Z. The repo has
  115 open *items* (issues + PRs).
- **Assessment (`medium-high`, inferred):** **actively maintained and high-visibility** (17.6k
  stars, daily commits, 16 releases, an organized `-preview` tag history). The maintenance risk is
  not dormancy; it is **pre-1.0 velocity** — an exact-pinned `gpui-pre` snapshot that advances on
  its own 0.3.x cadence, plus a second `gpui-fast` lineage.

---

## 9. Integration impact on `apps/linux`

### 9.1 Startup and init chain (`high`)

`[HTTP]` docs + `gpui-kit-0.7.3/src/lib.rs` + `gpui-component-0.7.3/src/lib.rs`. A Kit app calls, in
order: `gpui_kit::application().run(|cx| { gpui_kit::init(cx); … })`. `gpui_kit::init` initializes
the enabled layers; `gpui_component::init(cx)` (the styled layer) runs the chain:
`theme::init, global_state::init, inspector::init, root::init, gpui_base::init, input::init,
diff::init, date_picker::init, dock::init, sheet::init, list::init, command::init, carousel::init,
notification::init, popover::init, questionnaire::init, menu::init, table::init, tooltip::init`.
`gpui-base` alone requires only `gpui_kit::base::init(cx)` (and the docs say not to call both).

### 9.2 Global theme registry and mandatory `Root` wrapper (`high`)

- A **global theme singleton** (`Theme`, `ThemeRegistry`, `ActiveTheme`) is installed at init;
  `ThemeRegistry` observes theme changes and refreshes windows. Docs caution that a raw
  `Theme::global_mut(cx)` edit does not run the synchronization/refresh path; use `Theme::change` /
  `Theme::update`.
- A `Root` wrapper is expected around window content: docs state `gpui_kit::open_window` *"wraps the
  returned view in its own Base `Root`"*, and *"do not wrap it in another `Root`. The `Root` supplies
  window-level overlays, menus, notifications, and focus coordination."* On Linux, `NativeMenu`
  fallback depends on that `Root` overlay existing.
- This is a real architectural coupling into startup — different from the current UI-001/UI-002
  `apps/linux` startup.

### 9.3 Assets (`high`)

`gpui_kit::application().with_assets(gpui_kit::assets::Assets)` (or the `gpui-kit` `assets` feature)
bundles the Lucide icons; omitting the feature lets an app ship its own SVGs named per `IconName`.
`gpui-kit-assets` uses `rust-embed` on native and declares `links = "gpui-kit-default-icons"`.

### 9.4 Transitive weight (`medium-high`)

`[HTTP]` the 0.7.3 manifests show a **non-optional** `http_client = "=0.3.9"` →
`gpui-pre-http-client` in `gpui-pre`, and `gpui-base`/`gpui-component` pull
`gpui-pre-reqwest-client`; the published `Cargo.lock` files include rustls/`rustls-webpki` and a
reqwest-based client. So a TLS/HTTP stack arrives by default even for a purely local app. Adopting
the Kit changes the dependency graph substantially versus the current minimal `gpui = "=0.2.2"`
pin.

### 9.5 Platform limits (`high`)

- `gpui-base` README "Platform Support": macOS (Apple Silicon + Intel), **Linux on x86_64**,
  Windows on x86_64, WebAssembly depending on APIs. **No iOS/Android.**
- The manifests **exclude** `gpui_platform` on `ios`/`android`
  (`cfg(not(any(target_os = "ios", target_os = "android")))`), and the mobile docs describe iOS as
  **experimental** with `gpui-pre-mobile` via a git dependency: *"Treat the iOS simulator path below
  as the documented target, not a general mobile support guarantee."* Mobile is explicitly
  *"not a general mobile support guarantee"*. This is relevant to `DEC-010` (Android app deferred)
  and to §5.4 of `full.md`.
- Linux backend features enabled: `["font-kit", "x11", "wayland", "runtime_shaders"]`. The docs say a
  window needs a working Vulkan driver in a graphical Wayland/X11 session.
- WebView (`gpui-wry`) is **experimental** and its **Linux path is unfinished** — not usable as a
  drop-in browser embed.

### 9.6 The `libc 0.2.189` hazard (`high` for declared deps; `unverified-by-build` for resolution)

Current `apps/linux` context (`[LOCAL]` `apps/linux/Cargo.toml`, `apps/linux/README.md` §12): the
`gpui = "=0.2.2"` pin compiles only while `apps/linux/Cargo.lock` holds `libc 0.2.189`; loosening it
reintroduces the `xattr 0.2.3` → `libc::ENOATTR` failure (`cargo check` exit 101). Relevant Kit facts:

- `[HTTP]` **No** of `gpui-kit`/`gpui-base`/`gpui-component`/`gpui-component-macros`/
  `gpui-kit-assets` declares `libc` or `xattr` in its manifest.
- `[HTTP]` `gpui-pre 0.3.9` **does** declare `libc` — but as `libc = "0.2"` for
  `cfg(any(target_os = "linux", target_os = "android"))` (a caret range, **not** an exact pin).
- `[HTTP]` the family's **own** lockfiles resolve `libc` **below** the breaking threshold:
  `gpui-kit-0.7.3/Cargo.lock` → `libc 0.2.189`; `gpui-pre-0.3.9/Cargo.lock` → `libc 0.2.186`. No
  `xattr` entry appears in `gpui-kit-0.7.3/Cargo.lock`.
- `[INF]` nothing in the family *adds or loosens* the `libc` requirement; the `libc = "0.2"` range is
  compatible with the `0.2.189` hold, and the Kit does not depend on the `xattr` crate the way
  upstream `gpui 0.2.2` does. **But this is a consequence of replacing the GPUI artifact** (the
  `gpui-pre` snapshot does not carry the same `xattr` dependency path), which is precisely the change
  not authorized here.
- **`unverified-by-build`:** the real consumer resolution (`cargo tree -i libc`, whether `xattr`
  appears at all, and whether the `libc 0.2.189` hold is still needed) was not executed.

---

## 10. Fidelity gate (`full.md` §0.2 / §2.4)

The requirement is fidelity to the reference videos, not a generic redesign. Two evidence points:

1. **The foundation is explicitly unstyled.** `[HTTP]` `gpui-base` 0.7.3 README, verbatim:
   > *"It provides interaction behavior, focus management, accessibility semantics, animation,
   > virtual lists, theme tokens, and other foundational capabilities **without imposing a visual
   > style**."*
   > *"**Presentation belongs to the application:** layout, size, color, spacing, radius, borders,
   > shadows, variants, and animation are defined by the application or a higher-level component."*
   > *"For example, `Button::new("save")` has no padding, background, radius, or size by default.
   > **Being unstyled is an explicit API contract, not a missing feature.**"*

2. **The override surface is real.** `[HTTP]` `gpui-base-0.7.3/src/theme_tokens.rs` defines
   `SemanticThemeTokens { colors, radius, spacing, typography, shadow }` with `ColorTokens`,
   `RadiusTokens`, `SpacingTokens`, `TypographyTokens`, `ShadowTokens`. Docs describe
   `Theme::change`, `Theme::update`, `ThemeRegistry::watch_dir`, and application-supplied `themes/`
   JSON. This means reference tokens (`docs/research/design-tokens.md`) *could* in principle be
   encoded as a `gpui-base` theme.

3. **Third-party proof of a custom design language.** `[HTTP]` crates.io: `gpui-omarchy` (MIT,
   `huacnlee/gpui-omarchy`) is an independent package that builds an Omarchy/Tokyo-Night visual
   identity using the Kit foundation. (This host is Omarchy/Hyprland per
   `docs/operations/toolchain-audit.md`.)

**Conclusion (`medium-high`):** the Kit *can* be made to reproduce a spec-defined design, but only
via the **unstyled `gpui-base` layer** (plus application-authored rendering). The **styled
`gpui-component` layer** ships a shadcn/macOS-Windows-flavoured default visual language — 38 built-in
themes, none of them the reference product's measured design — so using it as-is would push toward
exactly the "generic redesign" §0.2/§2.4 forbid. Either way, reproducing the reference design is
application work; the Kit does not hard-code it, and no Kit widget maps to a terminal.

---

## 11. ADR-003 "Option J" adjudication

ADR-003 (`docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md`, Option J) recorded the family
as **rejected as the pin**. Its recorded reasons, quoted verbatim:

> …`gpui-base`/`gpui-component` 0.7.3 (Apache-2.0, Longbridge `LICENSE-APACHE` …) depend on Zed's
> `gpui` only via the alias `package = "gpui-pre"`; `gpui-kit` 0.7.3 declares
> `license = "Apache-2.0"` but ships **no** license file at all.
> These are third-party republishes behind a rename chain with an unverified fork base. `full.md`
> §15.2 makes an external API/license question a human stop condition, so the pin stays on the
> upstream crates.io artifact (Option H) and these remain fallbacks.

Adjudication against live evidence (`2026-10-16`):

| # | Recorded reason | Verdict | Evidence |
|---|---|---|---|
| a | *"depend on Zed's `gpui` only via the alias `package = "gpui-pre"`"* | **CONFIRMED (strengthened)** | `[HTTP]` every 0.7.3 manifest: `gpui = { version = "=0.3.9", package = "gpui-pre" }` (and `=0.3.6` at 0.6.6 `[LOCAL]`). The vendor's own docs §"Why the dependency is named `gpui-pre`" acknowledge the alias; the mobile troubleshooting table documents *"incompatible GPUI types"*. |
| b | *"`gpui-kit` 0.7.3 declares `license = "Apache-2.0"` but ships no license file at all"* | **CONFIRMED** | `[HTTP]` `gpui-kit-0.7.3` tarball contains no `LICENSE*`/`NOTICE*`; siblings do ship `LICENSE-APACHE`; `gpui-kit-assets` ships only `LICENSE-LUCIDE`. |
| c | *"third-party republishes behind a rename chain with an unverified fork base"* | **PARTIALLY SUPERSEDED** — the rename chain is CONFIRMED; the fork base is now *self-declared and partially checkable*, but still not verified as byte-equivalent | `[HTTP]` `gpui-pre 0.3.9` declares `zed-version = "0.2.2"`, `zed-rev = "93d7456…"`, and ships Zed's Apache-2.0 text; both that rev and the 0.3.6 rev **exist upstream** and the upstream `gpui` at the rev is `version = "0.2.2"`. What remains unverified: that `gpui-pre` == upstream `gpui 0.2.2` at that rev modulo declared renames (no diff performed, no `.cargo_vcs_info.json` in `gpui-pre`). |

### 11.1 Reconsideration triggers #6 and #8 (ADR-003 "What Evidence Would Cause Reconsideration")

- **Trigger #6** — *"`Glass-HQ/gpui` (or any fork) gains a license file + (if applicable) `NOTICE`,
  making a revision pin legally viable and byte-reproducible; that would reopen Option I."*
  **NOT TRIGGERED / NOT ADDRESSED.** It concerns a *different* fork (`Glass-HQ/gpui`, Option I). It
  says nothing about `gpui-pre`/`gpui-kit`; nothing here changes its status. (`gpui-pre` already
  ships Zed's license text; the `gpui-kit` umbrella still ships none.)
- **Trigger #8** — *"`gpui-pre` / `gpui-ce` lineage stabilizes with a verified fork base (e.g.
  `gpui-pre`'s `zed-rev = 93d7456…` is confirmed against upstream and a stable pin policy exists);
  that would reopen Option J."* **PARTIALLY SATISFIED — closer, not met.** The ADR's *example rev*
  (`93d7456…`) is exactly the rev `gpui-pre` 0.3.9 names, it **is** confirmed against upstream, and
  a stable pin policy exists (every Kit release exact-pins the matching `gpui-pre-*` set). What is
  still missing is verification that the snapshot is the upstream source modulo declared
  rename/manifest adjustments, and a stated consumer policy. Trigger #8 is therefore **closer than
  at ADR-003 time but still not discharged.**

### 11.2 Does the rejection still stand?

**Yes — the recorded rejection stands and is materially reinforced.** The single strongest new fact
is §5: `gpui-kit` is **architecturally incompatible with the repository's existing `gpui = "=0.2.2"`
pin** — it never depends on package `gpui`; it exact-pins `gpui-pre`. This was implicit in the ADR
("depend on Zed's `gpui` only via the alias") and is now proven at the current version with the
literal manifest strings and the vendor's own "incompatible GPUI types" documentation. No
recommendation to reverse the ADR is made; this is research only.

---

## 12. Recommendation: **Reject** (current pin) / **Defer** (any adoption)

**Reject for the current `gpui = "=0.2.2"` artifact decision; Defer the adoption decision until/unless
the GPUI artifact itself is re-opened by ADR.**

**For adoption (what the Kit would give us):**
- `[HTTP]` A genuinely rich, test-covered component set that maps almost 1:1 onto the reference
  product's non-terminal needs: command palette, dock/split panes, tabs, menus, dialogs,
  popovers/sheets/notifications, virtualized table/tree/list, Lucide icons, a Rope/Tree-sitter editor.
- `[HTTP]` A real theming/token system with an explicitly **unstyled** foundation, plus a
  third-party existence proof (`gpui-omarchy`) of a fully custom visual identity.
- `[HTTP]` Same upstream source base (`zed gpui 0.2.2`) for the default `gpui-pre` snapshot, so the
  underlying framework behaviour is not a version leap.
- `[HTTP]` Very active maintenance (17.6k stars, daily commits, 16 releases) and a published test
  suite.

**Against adoption (decisive):**
1. `[HTTP]` **It cannot be used alongside `gpui = "=0.2.2"`.** No version depends on package `gpui`;
   the Kit exact-pins `gpui-pre` and re-exports that compilation as `gpui_kit::*`; both packages
   export extern crate `gpui`; the vendor documents that mixing them yields incompatible GPUI types.
2. `[HTTP]` The Kit is designed to be the **only** GPUI source (README: *"an application never lists
   GPUI itself"*), and ships its own `actions!` macro precisely because `gpui::Action` *"does not
   resolve when GPUI is consumed solely through this facade."*
3. `[HTTP]` **`gpui-kit` still ships no license file** at 0.7.3 (siblings do; `gpui-kit-assets`
   ships only `LICENSE-LUCIDE`). A per-crate license-file policy is a real gap for the umbrella crate.
4. `[HTTP]` Adopting it changes transitive weight (TLS/reqwest by default), inserts a global
   theme/registry singleton, and requires a mandatory `Root` wrapper at startup.
5. `[HTTP]` Its styled layer is shadcn/macOS-Windows flavoured; using it as-is risks the "generic
   redesign" that §0.2/§2.4 forbid. Only the unstyled `gpui-base` is fidelity-safe.
6. `[HTTP]` A **second** vendor GPUI lineage (`gpui-fast`) now exists, widening the trust surface for
   trigger #8.
7. **It provides no terminal/VT/PTY widget** (`[HTTP]` §6.2), so it does not reduce the
   `alacritty_terminal`/`vte`/`portable-pty` work at all.

**What only a real build could settle (`unverified-by-build`):** see §5.6.

---

## 13. Open questions / unverified

1. **Byte-equivalence of `gpui-pre` to upstream `gpui 0.2.2`** at `rev=93d7456…`: not diffed; no
   `.cargo_vcs_info.json` in `gpui-pre` to tie the tree to a commit.
2. **Exact `gpui-pre` requirement for `gpui-kit` 0.6.0–0.6.4** (0.6.5–0.7.3 are each confirmed by
   manifest; the 0.6.0–0.6.4 `^0.3.x` values come from the prior `[LOCAL]` cache/registry metadata).
3. **`gpui-shell`**: named in the docs but not queried; version/license/repo `UNVERIFIED`.
4. **True contributor total**: GitHub API page capped at 100; the actual count may exceed that.
5. **crates.io `rust_version`**: `null` for all versions; docs informally require **Rust 1.92+**
   (from `oo7 0.6.0`), while the host is rustc 1.98.1. No machine-checkable MSRV field exists.
6. **`gpui-omarchy` 0.2.1 license-file presence**: only the 0.1.3 `LICENSE` (`[LOCAL]`) was confirmed;
   not re-extracted at 0.2.1.
7. **README-vs-published-API mismatches at 0.7.3**: the prior mismatch (`prelude`) was not re-checked.
8. **All §5.6 build-level questions**, and any rendered-fidelity comparison.
9. **`gpui-component` 0.7.3 `Cargo.lock` / exact consumer resolution** ("two `gpui` packages coexist?"),
   not executed.
10. This file does **not** assert REX wire/binary compatibility, or any architectural compatibility
    beyond what the cited evidence supports.

---

## 14. Proposed dependency / ordering for adoption (if the lead ever re-opens the artifact)

Nothing here is authorized; it is a sequencing sketch for the lead, mirroring ADR-003 §"Reconsider".

1. **ADR amendment first.** Re-open the GPUI artifact decision; record that adoption would replace
   `gpui = "=0.2.2"` with the `gpui-pre` facade family, and record the license-file gap for
   `gpui-kit`.
2. **Verify the fork base.** Confirm `gpui-pre`'s declared `zed-rev` against upstream and diff the
   snapshot against upstream `gpui 0.2.2` modulo declared renames (closes trigger #8).
3. **Reconcile `libc`.** Decide whether the `libc 0.2.189` hold is still needed once `gpui-pre`
   replaces `gpui` (`gpui-pre` declares `libc = "0.2"`, not exact; the `xattr` path is not the same).
4. **License decision.** Decide whether the `gpui-kit` no-license-file fact is acceptable under this
   repo's per-crate licensing policy, or whether only `gpui-base` (which ships `LICENSE-APACHE`) is
   usable.
5. **Fidelity spike.** Prototype the reference layout on **`gpui-base`** (unstyled) with
   `design-tokens.md`-derived tokens; do not adopt `gpui-component` defaults for fidelity-critical
   surfaces. Compare against reference stills per `apps/linux/README.md` §6.
6. **Isolation spike.** Because `apps/linux` is its own workspace, prototype the pin swap there and
   prove `cargo check`/window creation before any shared-crate change.
7. **Only then** consider UI-004/UI-006/DOCS-001 downstream tasks (which `TASKS.md` gates on this
   research).

---

## 15. Search trail — URLs fetched, HTTP status, retrieval date

All retrieval `2026-10-16` (UTC). `curl` with a descriptive `User-Agent` unless noted. Status is the
final status after redirects (`-L`) where used.

### 15.1 crates.io API

| URL | Status |
|---|---|
| `https://crates.io/api/v1/crates/gpui-kit` | 200 |
| `https://crates.io/api/v1/crates/gpui-kit/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-base` | 200 |
| `https://crates.io/api/v1/crates/gpui-base/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-component` | 200 |
| `https://crates.io/api/v1/crates/gpui-component/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-component-macros` | 200 |
| `https://crates.io/api/v1/crates/gpui-component-macros/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-kit-assets` | 200 |
| `https://crates.io/api/v1/crates/gpui-kit-assets/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-pre` | 200 |
| `https://crates.io/api/v1/crates/gpui-pre/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-omarchy` | 200 |
| `https://crates.io/api/v1/crates/gpui-omarchy/versions` | 200 |
| `https://crates.io/api/v1/crates/gpui-fast` | 200 |
| `https://crates.io/api/v1/crates/gpui/0.2.2` | 200 |

### 15.2 `static.crates.io` `.crate` tarballs (extracted under `/tmp/gpui-kit-evidence/crates/`)

| URL | Status |
|---|---|
| `https://static.crates.io/crates/gpui-kit/gpui-kit-0.7.3.crate` | 200 |
| `https://static.crates.io/crates/gpui-base/gpui-base-0.7.3.crate` | 200 |
| `https://static.crates.io/crates/gpui-component/gpui-component-0.7.3.crate` | 200 |
| `https://static.crates.io/crates/gpui-component-macros/gpui-component-macros-0.7.3.crate` | 200 |
| `https://static.crates.io/crates/gpui-kit-assets/gpui-kit-assets-0.7.3.crate` | 200 |
| `https://static.crates.io/crates/gpui-pre/gpui-pre-0.3.9.crate` | 200 |
| `https://static.crates.io/crates/gpui/gpui-0.2.2.crate` | 200 |
| `https://static.crates.io/crates/gpui-fast/gpui-fast-0.1.8.crate` | 200 |
| `https://static.crates.io/crates/gpui-kit/gpui-kit-0.7.1.crate` | 200 |
| `https://static.crates.io/crates/gpui-kit/gpui-kit-0.7.2.crate` | 200 |
| `https://static.crates.io/crates/gpui-kit/gpui-kit-0.6.6.crate` (probe only) | 200 |

### 15.3 GitHub API and raw content

| URL | Status |
|---|---|
| `https://api.github.com/repos/longbridge/gpui-kit` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/releases?per_page=100` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/tags?per_page=100` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/contributors?per_page=100&anon=true` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/contents/` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/contents/.github/workflows` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/commits?per_page=10` | 200 |
| `https://api.github.com/repos/longbridge/gpui-kit/issues?state=all&per_page=8&sort=updated` | 200 |
| `https://api.github.com/search/issues?q=repo:longbridge/gpui-kit+type:issue+state:open` | 200 |
| `https://api.github.com/search/issues?q=repo:longbridge/gpui-kit+type:issue+state:closed` | 200 |
| `https://api.github.com/repos/zed-industries/zed/commits/93d745668805029476e6d7fbb19652116fed51de` | 200 |
| `https://api.github.com/repos/zed-industries/zed/commits/bcf6582ce3500df93a8a39366640173e6786cea6` | 200 |
| `https://api.github.com/repos/zed-industries/zed/contents/crates/gpui/Cargo.toml?ref=93d7456…` | 200 |
| `https://raw.githubusercontent.com/longbridge/gpui-kit/main/.github/workflows/ci.yml` | 200 |
| `https://raw.githubusercontent.com/longbridge/gpui-kit/main/LICENSE-APACHE` | 200 |
| `https://raw.githubusercontent.com/longbridge/gpui-kit/main/release-notes.md` | 200 |
| `https://raw.githubusercontent.com/longbridge/gpui-kit/main/LICENSE` | **404** (no root `LICENSE`) |

### 15.4 Docs site

| URL | Status |
|---|---|
| `https://gpui-kit.com/docs` | 301 → `/docs/` |
| `https://gpui-kit.com/docs/` | 200 (server-rendered; also `chromium --headless=new --dump-dom`) |
| `https://gpui-kit.com/llms-full.txt` | 200 (4,516,059 bytes; full-docs dump) |
| `https://gpui-kit.com/docs/installation` | 200 |
| `https://gpui-kit.com/docs/getting-started` | 200 |
| `https://gpui-kit.com/docs/comparison` | 200 |
| `https://gpui-kit.com/docs/design-guides` | 200 |
| `https://gpui-kit.com/component` | 200 |
| `https://gpui-kit.com/base` | 200 |

### 15.5 Local repository / cache evidence (`[LOCAL]`)

- `docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md`, `apps/linux/Cargo.toml`,
  `apps/linux/README.md`, `apps/linux/Cargo.lock`, `docs/research/design-tokens.md`,
  `docs/research/ui-catalog.md`, `docs/agent-loop/TASKS.md`, `full.md`, `AGENTS.md`.
- `/home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-kit-0.6.6/`,
  `…/gpui-base-0.6.6/`, `…/gpui-pre-0.3.6/` (prior agent's cache; existence re-checked).
- `/home/sulthonuladib/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/gp/ui/`.

### 15.6 Raw-evidence mirror

Raw bodies are saved under `/tmp/gpui-kit-evidence/` (47 files incl. extracted crates); see §16.2.

---

## 16. Validation record

### 16.1 File-write discipline

- Exactly one repository file was written: `docs/research/gpui-kit.md` (this file). No
  `docs/agent-loop/**`, `docs/adr/**`, `apps/**`, `Cargo.toml`, or `Cargo.lock` was touched.
- `git status --porcelain` was empty before writing (§16.3) and, after writing, shows only this one
  new path.
- All fetched raw evidence lives outside the repo under `/tmp/gpui-kit-evidence/`; no scratch or
  downloaded media was placed in the repository.

### 16.2 `/tmp/gpui-kit-evidence/` contents (auditable)

Top level: `gpui-{kit,base,component,component-macros,kit-assets,pre,omarchy,fast}.crate.json`,
`gpui-{kit,base,component,component-macros,kit-assets,pre,omarchy}.versions.json`,
`gpui-0.2.2.api.json`, `gh_repo.json`, `gh_releases.json`, `gh_tags.json`, `gh_contributors.json`,
`gh_root.json`, `gh_workflows.json`, `gh_commits.json`, `issues_{open,closed,recent}.json`,
`zed_93d.json`, `zed_bcf.json`, `zed_gpui_cargo.json`, `ci.yml`, `LICENSE-APACHE.root`,
`LICENSE.root` (404 body), `release-notes.md`, `docs_curl.html`, `docs_chromium.html`,
`docs_text.txt`, `llms-full.raw`, `site_{component,base}.html`,
`site_docs_{installation,comparison,design-guides,getting-started}.html`, `chromium.err`.
Subdirectory `crates/`: the eight `.crate` tarballs and their extracted trees, including
`gpui-kit-0.7.3/` (with `Cargo.toml`, `Cargo.toml.orig`, `src/lib.rs`, `README.md`, `tests/`),
`gpui-pre-0.3.9/`, `gpui-0.2.2/`, and `gpui-fast-0.1.8/`.

### 16.3 Verification commands actually run

```text
# before writing
git status --porcelain            # empty (clean tree)
date -u +%Y-%m-%dT%H:%M:%SZ       # 2026-10-16T04:55:57Z
# after writing (this file is the only new path)
git status --porcelain            # "?? docs/research/gpui-kit.md"  (recorded below)
```

### 16.4 Key confirmations vs the mandate

- **Network probe:** PASSED (see §1.4) — the blocker reported by the prior agent does not apply here.
- **CURRENT `gpui-kit` = 0.7.3**, requirement `gpui = "=0.3.9" package = "gpui-pre"`; **CONFIRMED**
  by the literal normalized manifest in the published tarball.
- **License-file gap:** **CONFIRMED** at the current version.
- **Upstream base:** `zed-rev` exists and matches `gpui 0.2.2`; **source byte-equivalence
  UNVERIFIED**.
- **Docs site:** reachable and server-rendered; content captured.
- **No terminal widget:** **CONFIRMED**.
- **Build-level questions:** **UNVERIFIED-BY-BUILD** (see §5.6).
