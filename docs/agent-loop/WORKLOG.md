# WORKLOG — Relayable

Append-only checkpoint log. Each entry: objective, files changed, exact commands + exit codes, tests run, evidence paths, risks, next tasks.

---

## Checkpoint 1 — Phase 0 kickoff: repository and toolchain discovery (`full.md` §0.4, §19)

**Objective:** Record actual repository and toolchain state before any code changes. Preserve existing work.

**Files changed in this checkpoint:** `AGENTS.md`, `docs/agent-loop/{STATUS,TASKS,DECISIONS,WORKLOG,NEXT}.md` (new). No source files created yet. Nothing deleted.

### Commands run and observed results

`pwd` → `/home/sulthonuladib/personal/relayable`

`git rev-parse --abbrev-ref HEAD` → `master`

`git status --short` → *(no output — clean working tree)*

`git log --oneline -20` →
```
7181f92 plan update
1b0d8c8 init and commit full.md
```

`ls -la` → only `full.md` (80874 bytes) and `.git/`

`wc -l -c full.md` → `1279 80874`

`find . -maxdepth 2 -type f -not -path './.git/*' | sort` → `./full.md` *(only file in repo)*

`find .. \( -name AGENTS.md -o -name AGENTS.override.md \)` → no `AGENTS.md` inside the Relayable repo. Matches exist only in unrelated sibling projects (`../discord-bot`, `../fold_deez`, `../dillon`, `../scratchpad`, etc.) and in `node_modules` — none apply to this repository.

### Toolchain (`full.md` §0.4)

| Command | Observed |
|---|---|
| `rustc --version` | `rustc 1.98.1 (48a229cea 2026-09-01)` |
| `cargo --version` | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| `rustup show active-toolchain` | `stable-x86_64-unknown-linux-gnu (default)` |
| `command -v mise` | `/usr/bin/mise` — **present** |
| `command -v android` | `/home/sulthonuladib/.local/bin/android` — **present** |
| `command -v adb` | `/usr/bin/adb` — **present** |
| `command -v java` | `/home/sulthonuladib/.local/share/mise/installs/java/temurin-25/bin/java` |
| `java -version` | `openjdk version "25.0.4.1" 2026-08-18 LTS` — Temurin 25.0.4.1+1 |
| `command -v gradle` | **ABSENT** — Gradle wrapper required (§10.1.7) |
| `command -v ffmpeg` | `/usr/bin/ffmpeg` — **present** (frame extraction feasible) |
| `command -v yt-dlp` | `/usr/bin/yt-dlp` — **present** (video retrieval feasible) |
| `command -v chromium` | `/usr/bin/chromium` — **present** (browsing feasible) |
| `uname -a` | `Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux` |
| `herdr -V` | `herdr 0.9.5` |
| `echo $SHELL` / `$HERDR_ENV` | `/usr/bin/fish` / `1` (tool shell executes as `bash`) |

### Interpretation

- The repository is **empty except for the specification** → `full.md` §0.4 "if the repository is empty, establish a minimal workspace" applies. No existing architecture to preserve or migrate.
- **No blocker for video research:** `yt-dlp`, `ffmpeg`, and `chromium` are all installed, so the §2.1 workflow can proceed to actual frame inspection. This must still be proven per-video, and failures recorded honestly rather than assumed away.
- **No blocker for Android tooling discovery:** `android` CLI, `adb`, and JDK 25 are present. Only `gradle` is missing, which is expected — §10.1.7 requires committing a Gradle wrapper and never depending on a system Gradle.
- `mise` is present, so if a toolchain pin is later needed, `mise.toml` is the compatible mechanism (§0.3.11) — not silent global installation.

### Tests run

None. No build system exists yet. No test claimed.

### Risks / caveats

- `java -version` is JDK **25**; Android Gradle Plugin compatibility with JDK 25 is unverified and must be checked before any Android build attempt (§10.1.4).
- Toolchain versions above are recorded, not validated against project requirements — no `Cargo.toml`/`rust-toolchain.toml` exists yet to pin against.

### Next tasks (ordered)

1. BOOT-002 — commit scaffold (this checkpoint).
2. REF-001/002/003, DOC-001, ENV-001 — delegate bounded research with disjoint path ownership.
3. ADR-001 — domain model and dependency boundaries once DOC-001 evidence lands.
4. CORE-001 — server-owned PTY + CLI session + terminal I/O + reconnect vertical slice.
