# Toolchain audit — Rust / GPUI / Linux / systemd (ENV-001, worker A)

- **Task:** ENV-001 (`Task ledger`: Rust/GPUI/systemd audit; no installs, no system changes).
- **Branch / worktree:** `research/env-audit` @ `/home/sulthonuladib/.herdr/worktrees/relayable/research-env-audit`.
- **Base commit at audit time:** `3db73c2d28c6b7a98e424456768299e8d51b694d`.
- **Audit date:** `2026-10-15T23:05:18+07:00` (start; all probes in the same session, same date).
- **Host / user:** `falcon` / `sulthonuladib`.
- **Scope:** Rust toolchain + cargo/mise, GPUI crates.io/GitHub evidence, Linux + systemd user mode, display/host facts, GPUI system libraries.
- **Excluded (other worker / out of scope):** Android CLI, `adb`, JDK, SDK, Gradle, emulator (worker B owns `docs/operations/android-toolchain.md`).
- **Method:** strictly read-only. No installs, no `sudo`, no service enable/start/stop, no port/firewall or config changes, no builds, no `cargo fetch`/`build`/`search`/`add`, no writes outside this file. Commands were time-boxed; none hung. Exit codes were captured after each command; the seven HTTP probe rows whose exit codes were added in the 2026-10-15 remediation pass are re-captures (see remediation provenance below). Raw output is trimmed only of obvious noise (long HTTP header dumps, `head`-truncated listings noted inline).
- **Remediation provenance (2026-10-15):** the seven A2 HTTP probe rows flagged as missing exit codes in the independent ENV-001 verification were re-captured in a follow-up read-only pass on 2026-10-15 by re-running the same GETs with the recorded descriptive `User-Agent`; HTTP statuses and bodies matched (byte counts 21647 / 2702 / 109692 / 5975; license endpoint `HTTP 404`). The two `| head -40` rows report the shell pipeline exit; `curl`'s own pipe status is annotated inline (it varies when `head` closes the pipe).

## Header block (raw)

```text
$ pwd
/home/sulthonuladib/.herdr/worktrees/relayable/research-env-audit
EXIT: 0

$ git status --short --branch
## research/env-audit
EXIT: 0

$ git rev-parse HEAD
3db73c2d28c6b7a98e424456768299e8d51b694d
EXIT: 0

$ git branch --show-current
research/env-audit
EXIT: 0

$ date -Is
2026-10-15T23:05:18+07:00
EXIT: 0

$ hostname
falcon
EXIT: 0

$ uname -a
Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux
EXIT: 0

$ id -un
sulthonuladib
EXIT: 0

$ ls -la docs/operations
ls: cannot access 'docs/operations': No such file or directory
EXIT: 2
```

## Verified matrix (summary)

| Area | Finding | Status |
|---|---|---|
| rustc | 1.98.1 (48a229cea 2026-09-01) | OK |
| cargo | 1.98.1 (797e8a9bc 2026-08-05) | OK |
| rustup | stable default + 1.97; clippy, rustfmt, rust-src installed; single target `x86_64-unknown-linux-gnu` | OK |
| mise | 2026.10.5; manages codex/go/java/node, **not Rust** | OK |
| Cargo cache | `~/.cargo/registry` 914 MB; **no `gpui` crate cached** (only third-party `gpui-*` crates) | Partial |
| crates.io reachability | HTTP works **with a `User-Agent`**; anonymous requests get 403/404 | OK (UA required) |
| `gpui` on crates.io | 0.2.2, license Apache-2.0, repo `github.com/zed-industries/zed`, updated 2025-10-22 | OK |
| `github.com/Glass-HQ/gpui` | reachable, default branch `main`, pushed 2026-05-16, not a GitHub fork; **no root LICENSE file found (404), GitHub license API 404**; crate manifest declares Apache-2.0 | ⚠ Verify before pinning |
| systemd | 261 (261.2-1-arch); user manager `running`; `Linger=yes`; XDG runtime dir mode 700 | OK |
| journalctl --user | usable (`-n 3` returned entries) | OK |
| No relayable unit yet | no `relayable*` unit files in any user unit path; expected in Phase 0 | Informational |
| Display | Wayland session (`wayland-1`), XWayland `DISPLAY=:0`; omarchy desktop | OK |
| Fonts | 798 font entries; monospace → JetBrainsMono Nerd Font | OK |
| GPUI system deps | wayland-client 1.26.0, xkbcommon 1.13.2, fontconfig 2.18.3, vulkan 1.4.357; libEGL/libGL/vulkan libs present; `/dev/dri` render nodes present | OK |
| Host resources | 20 CPUs (i7-12700H), 31 GiB RAM, 51 GB free on `/` | OK |
| herdr | 0.9.5 at `~/.local/bin/herdr` | OK |
| Repo scaffold | base commit is spec-only: no `Cargo.toml`, no `rust-toolchain.toml` | Informational (expected Phase 0) |

---

## A1 — Rust / cargo / rustup / mise

**Date/host:** 2026-10-15, `falcon`, worktree `research/env-audit`. Exit codes recorded per command.

### rustc / cargo / rustup

```text
$ rustc --version
rustc 1.98.1 (48a229cea 2026-09-01)
EXIT: 0

$ cargo --version
cargo 1.98.1 (797e8a9bc 2026-08-05)
EXIT: 0

$ rustup show
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/sulthonuladib/.rustup

installed toolchains
--------------------
stable-x86_64-unknown-linux-gnu (active, default)
1.97-x86_64-unknown-linux-gnu

active toolchain
----------------
name: stable-x86_64-unknown-linux-gnu
active because: it's the default toolchain
installed targets:
  x86_64-unknown-linux-gnu
EXIT: 0

$ rustup show active-toolchain
stable-x86_64-unknown-linux-gnu (default)
EXIT: 0

$ rustup component list --installed
cargo-x86_64-unknown-linux-gnu
clippy-x86_64-unknown-linux-gnu
rust-docs-x86_64-unknown-linux-gnu
rust-src
rust-std-x86_64-unknown-linux-gnu
rustc-x86_64-unknown-linux-gnu
rustfmt-x86_64-unknown-linux-gnu
EXIT: 0

$ rustup target list --installed
x86_64-unknown-linux-gnu
EXIT: 0

$ command -v rustfmt
/home/sulthonuladib/.cargo/bin/rustfmt
EXIT: 0

$ rustfmt --version
rustfmt 1.9.0-stable (48a229ceae 2026-09-01)
EXIT: 0

$ command -v cargo-clippy
/home/sulthonuladib/.cargo/bin/cargo-clippy
EXIT: 0

$ cargo clippy --version
clippy 0.1.98 (48a229ceae 2026-09-01)
EXIT: 0
```

### mise (read-only)

Commands that could mutate (`mise install`, `mise use`, `mise upgrade`, `mise self-update`) were deliberately **not** run. Only listing/current commands were used.

```text
$ command -v mise
/usr/bin/mise
EXIT: 0

$ mise --version
2026.10.5 linux-x64 (2026-10-08)
mise WARN  mise version 2026.10.10 available
mise WARN  self-update is disabled for this install, update mise the same way you installed it
EXIT: 0

$ mise ls
codex  0.157.0
codex  0.157.1
codex  0.160.0
codex  0.162.1                   ~/.config/mise/config.toml  latest
go     1.27.2                    ~/.config/mise/config.toml  latest
java   temurin-25.0.4+101.0.LTS  ~/.config/mise/config.toml  temurin-25
node   22.22.3
node   26.8.2                    ~/.config/mise/config.toml  26.8.2
EXIT: 0

$ mise current
codex 0.162.1
go 1.27.2
java temurin-25.0.4+101.0.LTS
node 26.8.2
EXIT: 0
```

Note: mise does **not** manage the Rust toolchain on this host; rustup does (default `stable`). `mise ls` shows Java (Temurin 25) already managed by mise — relevant to the Android worker only.

### Cargo home and cache (presence only; no fetch/build)

```text
$ (echo "CARGO_HOME=[$CARGO_HOME]")
CARGO_HOME=[]
EXIT: 0            # unset → default ~/.cargo
```

```text
$ ls -la /home/sulthonuladib/.cargo
total 220
drwxr-xr-x 1 sulthonuladib sulthonuladib    190 Oct 15 22:22 .
drwx------ 1 sulthonuladib sulthonuladib   1326 Oct 15 08:05 ..
drwxr-xr-x 1 sulthonuladib sulthonuladib    258 Sep 25 16:14 bin
-rw-r--r-- 1 sulthonuladib sulthonuladib    472 Sep 25 16:14 .crates2.json
-rw-r--r-- 1 sulthonuladib sulthonuladib    104 Sep 25 16:14 .crates.toml
-rw-r--r-- 1 sulthonuladib sulthonuladib    300 Sep 25 14:21 env
-rw-r--r-- 1 sulthonuladib sulthonuladib    185 Sep 25 14:21 env.fish
-rw-r--r-- 1 sulthonuladib sulthonuladib 208896 Oct 15 22:22 .global-cache
-rw-r--r-- 1 sulthonuladib sulthonuladib      0 Sep 25 14:25 .package-cache
-rw-r--r-- 1 sulthonuladib sulthonuladib      0 Sep 25 14:25 .package-cache-mutate
drwxr-xr-x 1 sulthonuladib sulthonuladib     50 Sep 25 14:25 registry
EXIT: 0

$ du -sh /home/sulthonuladib/.cargo/registry
914M	/home/sulthonuladib/.cargo/registry
EXIT: 0

$ ls -la /home/sulthonuladib/.cargo/registry
total 4
drwxr-xr-x 1 sulthonuladib sulthonuladib  50 Sep 25 14:25 .
drwxr-xr-x 1 sulthonuladib sulthonuladib 190 Oct 15 22:22 ..
drwxr-xr-x 1 sulthonuladib sulthonuladib  64 Sep 25 14:25 cache
-rw-r--r-- 1 sulthonuladib sulthonuladib  17 Sep 25 14:25 CACHEDIR.TAG
drwxr-xr-x 1 sulthonuladib sulthonuladib  64 Sep 25 14:25 index
drwxr-xr-x 1 sulthonuladib sulthonuladib  64 Sep 25 14:25 src
EXIT: 0

$ ls /home/sulthonuladib/.cargo/registry/src
index.crates.io-1949cf8c6b5b557f
EXIT: 0

$ ls /home/sulthonuladib/.cargo/registry/cache
index.crates.io-1949cf8c6b5b557f
EXIT: 0
```

Repo-level Cargo config (checked for offline/mirror/vendor settings):

```text
$ ls -la /home/sulthonuladib/.cargo/config.toml
ls: cannot access '/home/sulthonuladib/.cargo/config.toml': No such file or directory
EXIT: 2

$ ls -la .cargo          # worktree
ls: cannot access '.cargo': No such file or directory
EXIT: 2

$ ls -d vendor           # worktree
ls: cannot access 'vendor': No such file or directory
EXIT: 2
```

Conclusion: no custom cargo config, no mirror, no vendor dir. Registry cache is populated (914 MB) from prior unrelated work.

---

## A2 — GPUI availability and license evidence (registry / URL lookups only)

No builds, no installs, no `cargo search`/`fetch`/`add`, nothing written into the project.

### Connectivity probe

```text
$ curl -sI -m 10 https://crates.io          # no User-Agent
HTTP/2 403
content-length: 0
...
EXIT: 0            # HTTP status 403; connection itself works
```

```text
$ curl -s -o /dev/null -w '%{http_code}' -H 'User-Agent: relayable-env-audit/0.1 (local read-only research)' https://crates.io
404
EXIT: 0
```

Interpretation (evidence-based): the host has working HTTPS egress to crates.io; crates.io rejects anonymous (no-UA) requests with 403 and the API expects a descriptive `User-Agent`. All following API calls include one.

### crates.io `gpui` crate

```text
$ curl -s -m 20 -H 'User-Agent: relayable-env-audit/0.1 (local read-only research)' \
    https://crates.io/api/v1/crates/gpui
# bytes: 21647 (raw JSON; first 300 bytes:)
{"crate":{"id":"gpui","name":"gpui","updated_at":"2025-10-22T03:56:29.835700Z","versions":[...],
 "keywords":["desktop","gui","immediate"],"categories":["gui"],"badges":[], ...

# Parsed key fields (exact values from that raw JSON):
crate.name: gpui
crate.max_version: 0.2.2
crate.max_stable_version: 0.2.2
crate.license: None            # crate-level field empty; per-version license is set
crate.repository: https://github.com/zed-industries/zed
crate.homepage: https://gpui.rs
crate.created_at: 2022-06-23T23:38:59.482625Z
crate.updated_at: 2025-10-22T03:56:29.835700Z
crate.downloads: 360402
versions.count: 7
version: 0.2.2       | created: 2025-10-22T03:56:29.835700Z | rust_version: None | license: Apache-2.0 | yanked: False
version: 0.2.1       | created: 2025-10-14T04:50:51.658529Z | rust_version: None | license: Apache-2.0 | yanked: False
version: 0.2.1-test3 | created: 2025-10-14T04:07:15.753179Z | rust_version: None | license: Apache-2.0 | yanked: True
version: 0.2.0       | created: 2025-10-09T04:28:33.248962Z | rust_version: None | license: Apache-2.0 | yanked: False
version: 0.2.0-test.4| created: 2025-10-06T01:11:03.634185Z | rust_version: None | license: Apache-2.0 | yanked: True
EXIT: 0            # HTTP 200; bytes and fields matched in the 2026-10-15 re-run
```

```text
$ curl -s -m 20 -H 'User-Agent: relayable-env-audit/0.1 (local read-only research)' \
    https://crates.io/api/v1/crates/gpui/0.2.2
# raw JSON: 2702 bytes. Parsed key fields:
crate: gpui
num: 0.2.2
license: Apache-2.0
crate_size: 5291873            # ~5.29 MB .crate file
published_by: mikayla-maki
created_at: 2025-10-22T03:56:29.835700Z
yanked: False
rust_version: None
features: ['default', 'inspector', 'leak-detection', 'macos-blade', 'runtime_shaders', 'screen-capture', 'test-support', 'wayland', 'windows-manifest', 'x11']
EXIT: 0            # HTTP 200; bytes and fields matched in the 2026-10-15 re-run
```

Sparse index cross-check (same metadata, raw JSON lines):

```text
$ curl -s -m 20 -H 'User-Agent: relayable-env-audit/0.1 (local read-only research)' \
    https://index.crates.io/gp/ui/gpui
# bytes: 109692; index line count: 7
version: 0.2.1-test3 | yanked: True
version: 0.2.1       | yanked: False
version: 0.2.2       | yanked: False
# from the tail of the 0.2.2 line:
"features":{"default":["font-kit","wayland","x11","windows-manifest"],
            "test-support":[...], "x11":[...]}
EXIT: 0            # HTTP 200; byte count (109692) and yank flags matched in the 2026-10-15 re-run
```

### `github.com/Glass-HQ/gpui` (the repo linked in full.md §20)

```text
$ curl -sI -m 15 -L -H 'User-Agent: ...' https://github.com/Glass-HQ/gpui
HTTP/2 200
date: Thu, 15 Oct 2026 16:06:01 GMT
content-type: text/html; charset=utf-8
... (remaining GitHub security/cookie headers omitted as noise)
EXIT: 0
```

```text
$ curl -s -m 20 -H 'User-Agent: ...' -H 'Accept: application/vnd.github+json' \
    https://api.github.com/repos/Glass-HQ/gpui
# bytes: 5975. Parsed key fields:
full_name: Glass-HQ/gpui
description: A native Rust UI framework for building fast, cross-platform apps across macOS, iOS, Linux, Windows, and the web.
default_branch: main
pushed_at: 2026-05-16T02:19:19Z
created_at: 2026-02-04T15:19:02Z
fork: False
archived: False
html_url: https://github.com/Glass-HQ/gpui
stargazers_count: 142
size: 7331
license.spdx_id: None          # GitHub reports no detected license
parent.full_name: None         # not a GitHub fork
source.full_name: None
EXIT: 0            # HTTP 200; bytes and fields matched in the 2026-10-15 re-run
```

README (first lines) — shows the repository carries Zed's GPUI readme:

```text
$ curl -s -m 20 -H 'User-Agent: ...' https://raw.githubusercontent.com/Glass-HQ/gpui/main/README.md | head -40
# Welcome to GPUI!

GPUI is a hybrid immediate and retained mode, GPU accelerated, UI framework
for Rust, designed to support a wide variety of applications.

...
GPUI is still in active development as we work on the Zed code editor, and is still pre-1.0.
There will often be breaking changes between versions. ...
EXIT: 0            # pipeline (head); curl's own pipe status varies (0, or 23 when `head` closes the pipe)
```

Crate manifest in that repo — declares Apache-2.0 and points back to Zed:

```text
$ curl -s -m 20 -H 'User-Agent: ...' https://raw.githubusercontent.com/Glass-HQ/gpui/main/crates/gpui/Cargo.toml | head -40
[package]
name = "gpui"
version = "0.2.2"
edition.workspace = true
authors = ["Nathan Sobo <nathan@zed.dev>"]
description = "Zed's GPU-accelerated UI framework"
repository = "https://github.com/zed-industries/zed"
publish = true
license = "Apache-2.0"
homepage = "https://gpui.rs"
...
[features]
default = ["font-kit", "wayland", "x11", "windows-manifest"]
wayland = [ "bitflags" ]
x11 = ["scap?/x11"]
EXIT: 0            # pipeline (head); curl's own pipe status varies (0, or 23 when `head` closes the pipe)
```

Repository-root license file check (HTTP statuses):

```text
LICENSE       -> HTTP 404
LICENSE-APACHE-> HTTP 404
LICENSE-GPL   -> HTTP 404
LICENSE.md    -> HTTP 404
COPYING       -> HTTP 404
$ curl -s -o /dev/null -w '%{http_code}' ... https://api.github.com/repos/Glass-HQ/gpui/license
HTTP 404
EXIT: 0
```

### Local cache presence for `gpui`

```text
$ find /home/sulthonuladib/.cargo/registry -maxdepth 4 -iname '*gpui*'
.../registry/cache/index.crates.io-1949cf8c6b5b557f/gpui-component-macros-0.6.6.crate
.../registry/cache/index.crates.io-1949cf8c6b5b557f/gpui-kit-0.6.6.crate
.../registry/cache/index.crates.io-1949cf8c6b5b557f/gpui-omarchy-0.1.3.crate
.../registry/cache/index.crates.io-1949cf8c6b5b557f/gpui-pre-*.3.6.crate   (many)
... (third-party gpui-ecosystem crates only; full list not reproduced)
EXIT: 0

$ ls /home/sulthonuladib/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ | grep -x 'gpui'
# (no output)
grep EXIT: 1                      # the bare `gpui` crate is NOT unpacked in src

$ ls /home/sulthonuladib/.cargo/registry/index/.../.cache/gp/ui/
gpui-base  gpui-component  gpui-component-macros  gpui-kit  gpui-kit-assets
gpui-omarchy  gpui-pre  gpui-pre-apple  gpui-pre-collections  gpui-pre-derive-refineable
gpui-pre-http-client  gpui-pre-linux  gpui-pre-macos  gpui-pre-macros  gpui-pre-perf
gpui-pre-platform  gpui-pre-refineable  gpui-pre-reqwest  gpui-pre-scheduler
gpui-pre-shared-string  gpui-pre-sum-tree  gpui-pre-util  gpui-pre-util-macros
gpui-pre-web  gpui-pre-wgpu  gpui-pre-windows  gpui-pre-zlog  gpui-pre-ztracing
gpui-pre-ztracing-macro
EXIT: 0                           # no bare `gpui` index cache entry either
```

**Conclusion (A2):** `gpui` 0.2.2 exists on crates.io, Apache-2.0 licensed, and is maintained against upstream `zed-industries/zed`; the `Glass-HQ/gpui` repository is reachable and carries the same `gpui` 0.2.2 sources/manifest, but **no repository-root license file was found** at audit time. The host does not currently have the `gpui` crate cached, so the first GPUI build will require a network download (~5.29 MB crate plus its dependency tree). No license reuse decision is made here — that belongs to an ADR (full.md §5.3) before any vendoring/pinning.

---

## A3 — Linux / systemd user mode

**Date/host:** 2026-10-15, `falcon`, worktree `research-env-audit`. Read-only; no units were enabled, started, or reloaded.

```text
$ systemctl --version
systemd 261 (261.2-1-arch)
+PAM +AUDIT -SELINUX +APPARMOR -IMA +IPE +SMACK +SECCOMP +GCRYPT +GNUTLS +OPENSSL +ACL +BLKID +CURL +ELFUTILS +FIDO2 +IDN2 +KMOD +LIBCRYPTSETUP +LIBCRYPTSETUP_PLUGINS +LIBFDISK +PCRE2 +PWQUALITY +P11KIT +QRENCODE +TPM2 +BZIP2 +LZ4 +XZ +ZLIB +ZSTD +BPF_FRAMEWORK +BTF +XKBCOMMON +UTMP +LIBARCHIVE
EXIT: 0

$ systemctl --user is-system-running
running
EXIT: 0

$ systemctl --user list-unit-files --no-pager | head -60
UNIT FILE                                                           STATE     PRESET
app-ai.opencode.desktop-124132.scope                                transient -
app-com.google.Chrome-2545318.scope                                 transient -
app-ghostty-surface-transient-2564968.scope                         transient -
app-ghostty-surface-transient-2672846.scope                         transient -
app-ghostty-surface-transient-3971.scope                            transient -
app-Hyprland-gtk\x2dlaunch-25ed5f57.scope                           transient -
app-Hyprland-gtk\x2dlaunch-34419dce.scope                           transient -
app-Hyprland-gtk\x2dlaunch-bee26422.scope                           transient -
app-Hyprland-omarchy\x2dhyprland\x2dmonitor\x2dwatch-b27a9184.scope transient -
app-Hyprland-udiskie-41c4c400.scope                                 transient -
app-org.chromium.Chromium-2553917.scope                             transient -
tmux-spawn-0771acc7-12f3-4e29-ac38-a444a43fa35a.scope               transient -
tmux-spawn-d18fa928-f6e2-4549-9eac-586ffd9ed5db.scope               transient -
app-com.mitchellh.ghostty.service                                   enabled   enabled
app-gnome\x2dkeyring\x2dpkcs11@autostart.service                    generated -
app-gnome\x2dkeyring\x2dsecrets@autostart.service                   generated -
app-limine\x2drestore\x2dnotify@autostart.service                   generated -
app-org.gnome.SettingsDaemon.DiskUtilityNotify@autostart.service    generated -
arbitrator-aliases.service                                          disabled  enabled
at-spi-dbus-bus.service                                             static    -
bt-agent.service                                                    enabled   enabled
dbus-:1.23-org.a11y.atspi.Registry@0.service                        transient -
dbus-broker.service                                                 disabled  enabled
...
pipewire-pulse.service                                              enabled   enabled
(truncated at 60 lines; full listing is longer)
PIPESTATUS: 0 1                     # `head` closed the pipe → systemctl got SIGPIPE (141); list itself succeeded
```

User unit search paths, and confirmation that no Relayable units exist yet:

```text
$ ls -la /home/sulthonuladib/.config/systemd/user
total 4
drwxr-xr-x 1 sulthonuladib sulthonuladib 220 Oct  6 10:18 .
drwxr-xr-x 1 sulthonuladib sulthonuladib   8 Sep 25 17:25 ..
-rw-r--r-- 1 sulthonuladib sulthonuladib 262 Oct  6 10:18 arbitrator-aliases.service
drwxr-xr-x 1 sulthonuladib sulthonuladib  86 Oct 12 17:02 default.target.wants
drwxr-xr-x 1 sulthonuladib sulthonuladib  80 Sep 25 17:29 graphical-session-pre.target.wants
drwxr-xr-x 1 sulthonuladib sulthonuladib 308 Oct 15 03:51 graphical-session.target.wants
EXIT: 0

$ ls -la /etc/systemd/user (first 20 lines)
total 4
drwxr-xr-x 1 root root 216 Sep 25 17:25 .
drwxr-xr-x 1 root root 510 Oct  7 01:30 ..
drwxr-xr-x 1 root root  42 Sep 25 17:25 graphical-session-pre.target.wants
drwxr-xr-x 1 root root  38 Sep 25 17:25 pipewire.service.wants
lrwxrwxrwx 1 root root  41 Sep 25 17:25 pipewire-session-manager.service -> /usr/lib/systemd/user/wireplumber.service
drwxr-xr-x 1 root root 168 Sep 25 17:25 sockets.target.wants
EXIT: 0

$ ls -la /usr/lib/systemd/user (first 20 lines)
total 468
... (system-provided units and slices only)
EXIT: 0

$ systemctl --user list-unit-files --no-pager | grep -i relayable
# (no output)
PIPESTATUS: 0 1                     # no matching unit files

$ ls /usr/lib/systemd/user/relayable* /etc/systemd/user/relayable* \
     /home/sulthonuladib/.config/systemd/user/relayable*
ls: cannot access '/usr/lib/systemd/user/relayable*': No such file or directory
ls: cannot access '/etc/systemd/user/relayable*': No such file or directory
ls: cannot access '/home/sulthonuladib/.config/systemd/user/relayable*': No such file or directory
EXIT: 2
```

journald usability:

```text
$ journalctl --user -n 3 --no-pager
Oct 15 23:06:35 falcon uwsm_hyprland.desktop[2797764]: > Warning:          Virtual modifier ScrollLock multiply defined
Oct 15 23:06:35 falcon uwsm_hyprland.desktop[2797764]: >                   Using 0, ignoring 0
Oct 15 23:06:35 falcon uwsm_hyprland.desktop[2797764]: Errors from xkbcomp are not fatal to the X server
EXIT: 0
```

Session/linger and runtime directory facts relevant to full.md §7.1/§7.2:

```text
$ loginctl show-user sulthonuladib
UID=1000
GID=1000
Name=sulthonuladib
Timestamp=Wed 2026-10-14 17:07:42 WIB
TimestampMonotonic=16226588
RuntimePath=/run/user/1000
Service=user@1000.service
Slice=user-1000.slice
Display=1
State=active
Sessions=2 1
IdleHint=no
IdleSinceHint=0
IdleSinceHintMonotonic=0
Linger=yes
EXIT: 0

$ echo "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR"
XDG_RUNTIME_DIR=/run/user/1000
EXIT: 0

$ ls -ld "$XDG_RUNTIME_DIR"
drwx------ 20 sulthonuladib sulthonuladib 700 Oct 15 23:06 /run/user/1000
EXIT: 0

$ stat -c '%a %U %G %n' /run/user/1000
700 sulthonuladib sulthonuladib /run/user/1000
EXIT: 0

$ echo "XDG_STATE_HOME=$XDG_STATE_HOME"
XDG_STATE_HOME=/home/sulthonuladib/.local/state
EXIT: 0

$ echo "XDG_CONFIG_HOME=$XDG_CONFIG_HOME"
XDG_CONFIG_HOME=/home/sulthonuladib/.config
EXIT: 0
```

Implication for the server: a `systemd --user` service can place a Unix socket at `$XDG_RUNTIME_DIR/relayable/` (parent dir mode 700, user-owned), matching full.md §7.1. `Linger=yes` already keeps the user manager alive after logout on this host, which supports the optional post-logout persistence mode (full.md §7.2) — but that is a policy decision for an ADR, not made here.

---

## A4 — Display / host

**Date/host:** 2026-10-15, `falcon`. Read-only.

```text
$ echo "WAYLAND_DISPLAY=$WAYLAND_DISPLAY"; echo "DISPLAY=$DISPLAY"; \
  echo "XDG_SESSION_TYPE=$XDG_SESSION_TYPE"; echo "DESKTOP_SESSION=$DESKTOP_SESSION"
WAYLAND_DISPLAY=wayland-1
DISPLAY=:0
XDG_SESSION_TYPE=wayland
DESKTOP_SESSION=omarchy
EXIT: 0

$ fc-list | wc -l
798
EXIT: 0

$ fc-match monospace
JetBrainsMonoNerdFont-Regular.ttf: "JetBrainsMono Nerd Font" "Regular"
EXIT: 0

$ df -h
Filesystem                Size  Used Avail Use% Mounted on
dev                        16G     0   16G   0% /dev
run                        16G  2.0M   16G   1% /run
efivarfs                  256K  173K   79K  69% /sys/firmware/efi/efivars
/dev/mapper/omarchy_root  159G  107G   51G  68% /
tmpfs                      16G  4.2M   16G   1% /dev/shm
none                      1.0M     0  1.0M   0% /run/credentials/systemd-journald.service
none                      1.0M     0  1.0M   0% /run/credentials/systemd-resolved.service
tmpfs                      16G  1.7G   14G  11% /tmp
/dev/mapper/omarchy_root  159G  107G   51G  68% /var/log
/dev/mapper/omarchy_root  159G  107G   51G  68% /var/cache/pacman/pkg
/dev/mapper/omarchy_root  159G  107G   51G  68% /home
/dev/nvme0n1p5            2.0G  782M  1.3G  39% /boot
tmpfs                     3.2G   27M  3.1G   1% /run/user/1000
/dev/nvme0n1p1            311G  276G   32G  90% /mnt/games
EXIT: 0

$ free -h
               total        used        free      shared  buff/cache   available
Mem:            31Gi       9.0Gi       9.1Gi       660Mi        15Gi        21Gi
Swap:           62Gi       2.5Gi        59Gi
EXIT: 0

$ nproc
20
EXIT: 0

$ lscpu | grep -E 'Architecture|Model name|CPU\(s\)|Thread|Core\(s\)|Socket|Virtualization|Hypervisor|L3 cache'
Architecture:                            x86_64
CPU(s):                                  20
On-line CPU(s) list:                     0-19
Model name:                              12th Gen Intel(R) Core(TM) i7-12700H
Thread(s) per core:                      2
Core(s) per socket:                      14
Socket(s):                               1
CPU(s) scaling MHz:                      60%
Virtualization:                          VT-x
L3 cache:                                24 MiB (1 instance)
NUMA node0 CPU(s):                       0-19
EXIT: 0

$ command -v herdr
/home/sulthonuladib/.local/bin/herdr
EXIT: 0

$ herdr --help   # verified `--version` exists; listing truncated at 25 lines
herdr — terminal workspace manager for AI coding agents

Usage: herdr [options]
       herdr --session <name> [options]
       herdr --machine <label-or-id> <command>
       herdr --remote <ssh-target> [--session <name>]
       ...
PIPESTATUS: 0 1                     # head/SIGPIPE, help printed fine
EXIT: 141

$ herdr --version
herdr 0.9.5
EXIT: 0
```

### GPUI-relevant system libraries (same date/host)

```text
$ pkg-config --modversion wayland-client
1.26.0
EXIT: 0

$ pkg-config --modversion xkbcommon
1.13.2
EXIT: 0

$ pkg-config --modversion fontconfig
2.18.3
EXIT: 0

$ pkg-config --modversion vulkan
1.4.357
EXIT: 0

$ ldconfig -p | grep -E 'libwayland-client|libxkbcommon|libvulkan|libEGL|libGL\.'
	libxkbcommon.so.0 (libc6,x86-64) => /usr/lib/libxkbcommon.so.0
	libxkbcommon.so (libc6,x86-64) => /usr/lib/libxkbcommon.so
	libxkbcommon-x11.so.0 (libc6,x86-64) => /usr/lib/libxkbcommon-x11.so.0
	libxkbcommon-x11.so (libc6,x86-64) => /usr/lib/libxkbcommon-x11.so
	libwayland-client.so.0 (libc6,x86-64) => /usr/lib/libwayland-client.so.0
	libwayland-client.so.0 (libc6) => /usr/lib32/libwayland-client.so.0
	libwayland-client.so (libc6,x86-64) => /usr/lib/libwayland-client.so
	libwayland-client.so (libc6) => /usr/lib32/libwayland-client.so
	libvulkan_intel_hasvk.so (libc6,x86-64) => /usr/lib/libvulkan_intel_hasvk.so
	libvulkan_intel.so (libc6,x86-64) => /usr/lib/libvulkan_intel.so
	libvulkan.so.1 (libc6,x86-64) => /usr/lib/libvulkan.so.1
	libvulkan.so (libc6,x86-64) => /usr/lib/libvulkan.so
	libGL.so.1 (libc6,x86-64) => /usr/lib/libGL.so.1
	libGL.so (libc6,x86-64) => /usr/lib/libGL.so
	libEGL_nvidia.so.0 (libc6,x86-64) => /usr/lib/libEGL_nvidia.so.0
	libEGL_nvidia.so (libc6,x86-64) => /usr/lib/libEGL_nvidia.so
	libEGL_mesa.so.0 (libc6,x86-64) => /usr/lib/libEGL_mesa.so.0
	libEGL_mesa.so (libc6,x86-64) => /usr/lib/libEGL_mesa.so
	libEGL.so.1 (libc6,x86-64) => /usr/lib/libEGL.so.1
	libEGL.so (libc6,x86-64) => /usr/lib/libEGL.so
EXIT: 0

$ ls -l /dev/dri
total 0
drwxr-xr-x  2 root root        120 Oct 14 17:07 by-path
crw-rw----+ 1 root video  226,   1 Oct 14 17:07 card1
crw-rw----+ 1 root video  226,   2 Oct 14 17:07 card2
crw-rw-rw- 1 root render 226, 128 Oct 14 17:07 renderD128
crw-rw-rw- 1 root render 226, 129 Oct 14 17:07 renderD129
EXIT: 0
```

### Repo scaffold state at base commit

```text
$ ls -la
-rw-r--r-- 1 sulthonuladib sulthonuladib  3616 Oct 15 22:43 AGENTS.md
drwxr-xr-x 1 sulthonuladib sulthonuladib    20 Oct 15 22:36 docs
-rw-r--r-- 1 sulthonuladib sulthonuladib 80874 Oct 15 22:36 full.md
-rw-r--r-- 1 sulthonuladib sulthonuladib    77 Oct 15 22:36 .git
-rw-r--r-- 1 sulthonuladib sulthonuladib   378 Oct 15 22:36 .gitignore
drwxr-xr-x 1 sulthonuladib sulthonuladib    26 Oct 15 23:05 .pi
EXIT: 0

$ ls rust-toolchain.toml Cargo.toml
ls: cannot access 'rust-toolchain.toml': No such file or directory
ls: cannot access 'Cargo.toml': No such file or directory
EXIT: 2
```

---

## A5 — Gaps blocking implementation

| # | Gap | Impact | Safest next step (proposal only — nothing executed) |
|---|---|---|---|
| G1 | No `Cargo.toml` / `rust-toolchain.toml` in the repo at base commit | No workspace to build or pin a toolchain; blocks BOOT-scaffold, CORE-001, UI-001 | Lead creates the workspace scaffold + `rust-toolchain.toml` pin (e.g. `1.98.1`) as part of the Phase 0/1 scaffold; not part of this read-only audit |
| G2 | `gpui` crate is **not** in the local cargo cache; first build must download it | First GPUI build needs network; offline builds impossible until cached; no vendor/mirror config exists | After ADR for UI framework, pin `gpui = "=0.2.2"` (or a verified git revision) and run the approved UI-001 spike build; audit itself deliberately did not populate caches |
| G3 | GPUI source/license: crates.io `gpui` 0.2.2 is Apache-2.0 and points to `zed-industries/zed`; `Glass-HQ/gpui` root has **no LICENSE file** (404) and GitHub reports no license | Legal/verification risk before vendoring or pinning the GitHub URL; full.md §5.3 requires isolating GPUI and pinning version/revision | ADR records chosen artifact + license evidence: prefer the crates.io release (per-version `license: Apache-2.0`) or verify Zed's GPUI crate license at the exact pinned revision before any reuse; do not vendor `Glass-HQ/gpui` until license is verified |
| G4 | GPUI's real Wayland/X11 behavior unverified on this host (no build/run in this audit; no renderer/driver version probe) | UI-001 depends on a working window under the actual session (`XDG_SESSION_TYPE=wayland`, `DISPLAY=:0` XWayland fallback) | UI-001 spike builds the minimal GPUI window and verifies input/cursor under Wayland with X11 fallback; add renderer/EGL details only if the spike fails |
| G5 | No relayable systemd unit exists; service model (user unit, socket activation, `TimeoutStopSec`) undecided | Expected in Phase 0; blocks Phase 4 service tests only | ADR for service/persistence model (full.md §7.2) before writing any unit; install/enable remains gated on user approval |
| G6 | Rust is managed by rustup, not mise; no project pin yet | Toolchain drift between agents/hosts | Add `rust-toolchain.toml` (full.md §5.1) rather than a mise rust pin; decide in the scaffold commit |
| G7 | crates.io API returns 403/404 for requests without a descriptive `User-Agent` | Scripts/CI querying crates.io anonymously may fail confusingly | Any future tooling that queries crates.io must send a descriptive `User-Agent`; document in the tooling scripts |
| G8 | Android/JDK/SDK/Gradle deliberately not audited here | CORE/AND tasks need that evidence from worker B's `docs/operations/android-toolchain.md` | Consume worker B's file; do not duplicate |
| G9 | Root disk 68% used, 51 GB free; builds + Android SDK/emulators may consume significant space | Large build targets could fill `/` | Per-run disk check before heavy builds; no cleanup performed (not authorized) |
| G10 | Docker/container, firewall/port behavior, DNS/NAT environment untested | Relevant later to remote transport (§6.2) but not to Phase 0/1 | Defer to the remote-protocol phase; probe read-only at that time |

Not a gap: rustc/cargo/rustfmt/clippy, systemd user mode, journald, Wayland/X11 libs, Vulkan/EGL/Mesa/NVIDIA libs, fonts, `/dev/dri`, XDG paths, `herdr` are all present and functional on this host.

---

## A6 — What changed on this host

**Nothing.** This audit was strictly read-only:

- no packages installed or upgraded; no `sudo`; no global/system/toolchain config changes;
- no services enabled, started, stopped, or reloaded; no unit files created;
- no ports opened or firewall changes;
- no `cargo fetch`/`build`/`search`/`add`; no project or target directory created; cargo caches untouched;
- no Android/JDK/SDK/Gradle commands run (worker B scope);
- no files written outside `docs/operations/toolchain-audit.md`.

`git status --short` was empty both before writing this file (after all probes) and after the commit; the only tracked change from ENV-001 (worker A) is this document (validation below).

---

## Validation record (worker A)

```text
$ git status --short            # before writing this file (after all read-only probes)
# (empty)
EXIT: 0

$ git status --short            # after writing this file
?? docs/operations/
EXIT: 0

$ git add docs/operations/toolchain-audit.md
A  docs/operations/toolchain-audit.md
# only this file staged (never `git add -A`)
EXIT: 0

$ git commit -m "docs(env-audit): add Rust/GPUI/systemd toolchain audit (ENV-001)"
[research/env-audit 5a6bc9a] docs(env-audit): add Rust/GPUI/systemd toolchain audit (ENV-001)
 1 file changed, 785 insertions(+)
 create mode 100644 docs/operations/toolchain-audit.md
EXIT: 0

$ git status --short            # after commit
# (empty)
EXIT: 0

$ git show --stat HEAD
commit 5a6bc9ad4dca4956044436950cf16f039dbb5689
Author: Sulthonul Adib <adibcyber4rt@gmail.com>
Date:   Thu Oct 15 23:08:57 2026 +0700

    docs(env-audit): add Rust/GPUI/systemd toolchain audit (ENV-001)

 docs/operations/toolchain-audit.md | 785 +++++++++++++++++++++++++++++++++++++
 1 file changed, 785 insertions(+)
EXIT: 0

$ git diff-tree --no-commit-id --name-only -r HEAD
docs/operations/toolchain-audit.md
EXIT: 0
```

This validation block was then updated by a follow-up, append-only commit (an amend was avoided
because a parallel worker owns another file on this branch); each commit on this branch touches
only `docs/operations/**`.

No push performed.
