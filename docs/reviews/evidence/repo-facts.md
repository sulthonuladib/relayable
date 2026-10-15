# REVIEW-001 evidence: repository facts (read-only transcript)

Collected by `generalist:repo-facts` in worktree
`/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`.
All commands are read-only Git operations; no state-mutating Git command was run.
Each entry records the command, the process exit code, and raw stdout/stderr.
The only file written by this task is this file (`docs/reviews/evidence/repo-facts.md`).

---

## 1. `date -u`

Command:
```
date -u
```
Exit code: `0`
Raw output:
```
Thu Oct 15 06:23:09 PM UTC 2026
```

---

## 2. `git --version`

Command:
```
git --version
```
Exit code: `0`
Raw output:
```
git version 2.55.0
```

---

## 3. `git rev-parse HEAD`

Command:
```
git rev-parse HEAD
```
Exit code: `0`
Raw output:
```
a1091e58f32d363ceb2614ab4915329b5de6a869
```

---

## 4. `git status --porcelain -b`

Command:
```
git status --porcelain -b
```
Exit code: `0`
Raw output:
```
## review/review-001-adr-pass
```
(Working tree clean; branch `review/review-001-adr-pass`, no upstream tracking line, no changes.)

---

## 5. `git log --oneline -15`

Command:
```
git log --oneline -15
```
Exit code: `0`
Raw output:
```
a1091e5 DEC-010: defer Android client from current scope (keep model/protocol Android-ready)
d42ae72 Docs: add restart trigger to NEXT.md; record Manager auto-start convenience
19eb33a DEC-009: remove dead auto-rename hook block from config.fish
7a7294f DEC-009: uninstall herdr-automatic-rename plugin; clear residual dirs
9e3a92c DEC-009 follow-up: fully disable herdr-automatic-rename plugin (user request)
16205a1 INT-003 clean-start handoff: doc-002 evidence on master; Wave 2 worktrees retired
a1f1719 INT-003: integrate research/doc-002 (DOC-002 citation-audit + URL drift evidence; repair still pending)
54bec1e Track REVIEW-001: no independent review of Wave 2 ADRs/spike (wind-down); record ADR-002 unverified items
827708e Handoff correction: worktree removal pending interactive confirmation (not performed)
3c3d5ff INT-002 handoff: integrate ADR-001/002 + TERM-001/ADR-003; preserve DOC-002 evidence; wind-down checkpoint
669cb84 Merge master into research/doc-002 for current agent-loop state
2ff3447 DOC-002 evidence: preserve citation audit + URL drift re-check (repair pending writer pass)
31bed30 INT-002: integrate research/adr-001-002 (ADR-001 domain model, ADR-002 protocol/transport, interface contract)
78b83f2 INT-002: integrate research/term-001-adr-003 (TERM-001 PTY spike + ADR-003 Proposed)
69e5600 ADR-001/ADR-002: domain model + dependency boundaries; protocol schema + transport; §14.4 interface contract
```

---

## 6. `git show --stat --oneline 69e5600` — ADR-001/002 + interface contract

Command:
```
git show --stat --oneline 69e5600
```
Exit code: `0`
Raw output:
```
69e5600 ADR-001/ADR-002: domain model + dependency boundaries; protocol schema + transport; §14.4 interface contract
 ...R-001-domain-model-and-dependency-boundaries.md | 467 +++++++++++++++++++++
 docs/adr/ADR-002-protocol-schema-and-transport.md  | 461 ++++++++++++++++++++
 docs/architecture/interface-contract.md            | 316 ++++++++++++++
 3 files changed, 1244 insertions(+)
```

---

## 7. `git show --stat --oneline 2e41da4` — ADR-003

Command:
```
git show --stat --oneline 2e41da4
```
Exit code: `0`
Raw output:
```
2e41da4 docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)
 ...DR-003-terminal-engine-and-gpui-artifact-pin.md | 626 +++++++++++++++++++++
 1 file changed, 626 insertions(+)
```

---

## 8. `git show --stat --oneline 085375c` — spike commit

Command:
```
git show --stat --oneline 085375c
```
Exit code: `0`
Raw output:
```
085375c spike(terminal-engine): add engine-independent PTY/resize/byte-stream/RSS harness
 spikes/terminal-engine/.gitignore                  |   3 +
 spikes/terminal-engine/Cargo.lock                  |  47 +
 spikes/terminal-engine/Cargo.toml                  |  49 +
 spikes/terminal-engine/README.md                   |  93 ++
 spikes/terminal-engine/STATUS.md                   |  46 +
 spikes/terminal-engine/results/README.md           |  94 ++
 .../results/inputs/hyperlink_dir/alpha.txt         |   1 +
 .../results/inputs/hyperlink_dir/beta.txt          |   1 +
 .../terminal-engine/results/inputs/less_input.txt  | 400 +++++++++
 .../results/raw/checks/cargo_build.txt             |  11 +
 .../results/raw/checks/cargo_clippy.txt            |  12 +
 .../results/raw/checks/cargo_fmt_check.txt         |  10 +
 .../terminal-engine/results/raw/esc_selftest.txt   |  45 +
 spikes/terminal-engine/results/raw/less.bin        |  24 +
 spikes/terminal-engine/results/raw/less.bin.meta   |  10 +
 spikes/terminal-engine/results/raw/less.esc        |  12 +
 spikes/terminal-engine/results/raw/less.seq        |  54 ++
 .../terminal-engine/results/raw/less.summary.txt   |  56 ++
 spikes/terminal-engine/results/raw/nvim.bin        |  69 ++
 spikes/terminal-engine/results/raw/nvim.bin.meta   |  10 +
 spikes/terminal-engine/results/raw/nvim.esc        |  96 ++
 spikes/terminal-engine/results/raw/nvim.seq        | 994 ++++++++++++++++++++
 .../terminal-engine/results/raw/nvim.summary.txt   |  56 ++
 spikes/terminal-engine/results/raw/nvim_mouse.bin  |  69 ++
 .../results/raw/nvim_mouse.bin.meta                |  10 +
 .../results/raw/nvim_mouse.esc                     |  96 ++
 .../results/raw/nvim_mouse.seq                     | 990 ++++++++++++++++++++
 .../results/raw/nvim_mouse.summary.txt             |  56 ++
 .../results/raw/nvim_title.bin                     |  69 ++
 .../results/raw/nvim_title.bin.meta                |  10 +
 .../results/raw/nvim_title.esc                     |  97 ++
 .../results/raw/nvim_title.seq                     | 997 +++++++++++++++++++++
 .../results/raw/nvim_title.summary.txt             |  56 ++
 .../results/raw/osc8_ls.bin                        |   2 +
 .../terminal-engine/results/raw/osc8_ls.bin.meta   |  10 +
 .../terminal-engine/results/raw/osc8_ls.esc        |  12 +
 .../terminal-engine/results/raw/osc8_ls.seq        |  45 +
 .../results/raw/osc8_ls.summary.txt                |  55 ++
 .../terminal-engine/results/raw/pty_size.txt       |  31 +
 .../terminal-engine/results/raw/run_meta.txt       |  53 ++
 .../results/raw/scrollback_bounded.txt             |  37 +
 .../results/raw/scrollback_discard.txt             |  37 +
 .../results/raw/scrollback_unbounded.txt           |  37 +
 .../results/raw/tools_available.txt                |  46 +
 spikes/terminal-engine/results/raw/top.bin         |  48 +
 spikes/terminal-engine/results/raw/top.bin.meta    |  10 +
 spikes/terminal-engine/results/raw/top.esc         |  43 +
 spikes/terminal-engine/results/raw/top.seq         | 619 +++++++++++++
 spikes/terminal-engine/results/raw/top.summary.txt |  56 ++
 spikes/terminal-engine/scripts/run.sh              | 167 ++++
 spikes/terminal-engine/src/bin/capture_stream.rs   | 353 ++++++++
 spikes/terminal-engine/src/bin/esc_selftest.rs     | 116 +++
 spikes/terminal-engine/src/bin/pty_size.rs         | 216 +++++
 spikes/terminal-engine/src/bin/scrollback_mem.rs   | 377 ++++++++
 spikes/terminal-engine/src/esc.rs                  | 439 +++++++++
 spikes/terminal-engine/src/lib.rs                  |   8 +
 spikes/terminal-engine/src/pty.rs                  | 214 +++++
 57 files changed, 7674 insertions(+)
```

---

## 9. `git show --stat --oneline 78b83f2` — merge of term-001/ADR-003

Command:
```
git show --stat --oneline 78b83f2
```
Exit code: `0`
Raw output:
```
78b83f2 INT-002: integrate research/term-001-adr-003 (TERM-001 PTY spike + ADR-003 Proposed)

 ...DR-003-terminal-engine-and-gpui-artifact-pin.md | 626 +++++++++++++
 spikes/terminal-engine/.gitignore                  |   3 +
 spikes/terminal-engine/Cargo.lock                  |  47 +
 spikes/terminal-engine/Cargo.toml                  |  49 +
 spikes/terminal-engine/README.md                   |  93 ++
 spikes/terminal-engine/STATUS.md                   |  46 +
 spikes/terminal-engine/results/README.md           |  94 ++
 .../results/inputs/hyperlink_dir/alpha.txt         |   1 +
 .../results/inputs/hyperlink_dir/beta.txt          |   1 +
 .../terminal-engine/results/inputs/less_input.txt  | 400 +++++++++
 .../results/raw/checks/cargo_build.txt             |  11 +
 .../results/raw/checks/cargo_clippy.txt            |  12 +
 .../results/raw/checks/cargo_fmt_check.txt         |  10 +
 .../terminal-engine/results/raw/esc_selftest.txt   |  45 +
 spikes/terminal-engine/results/raw/less.bin        |  24 +
 spikes/terminal-engine/results/raw/less.bin.meta   |  10 +
 spikes/terminal-engine/results/raw/less.esc        |  12 +
 spikes/terminal-engine/results/raw/less.seq        |  54 ++
 .../terminal-engine/results/raw/less.summary.txt   |  56 ++
 spikes/terminal-engine/results/raw/nvim.bin        |  69 ++
 spikes/terminal-engine/results/raw/nvim.bin.meta   |  10 +
 spikes/terminal-engine/results/raw/nvim.esc        |  96 ++
 spikes/terminal-engine/results/raw/nvim.seq        | 994 ++++++++++++++++++++
 .../terminal-engine/results/raw/nvim.summary.txt   |  56 ++
 spikes/terminal-engine/results/raw/nvim_mouse.bin  |  69 ++
 .../results/raw/nvim_mouse.bin.meta                |  10 +
 .../results/raw/nvim_mouse.esc                     |  96 ++
 .../results/raw/nvim_mouse.seq                     | 990 ++++++++++++++++++++
 .../results/raw/nvim_mouse.summary.txt             |  56 ++
 .../results/raw/nvim_title.bin                     |  69 ++
 .../results/raw/nvim_title.bin.meta                |  10 +
 .../results/raw/nvim_title.esc                     |  97 ++
 .../results/raw/nvim_title.seq                     | 997 +++++++++++++++++++++
 .../results/raw/nvim_title.summary.txt             |  56 ++
 .../results/raw/osc8_ls.bin                        |   2 +
 .../terminal-engine/results/raw/osc8_ls.bin.meta   |  10 +
 .../terminal-engine/results/raw/osc8_ls.esc        |  12 +
 .../terminal-engine/results/raw/osc8_ls.seq        |  45 +
 .../results/raw/osc8_ls.summary.txt                |  55 ++
 .../terminal-engine/results/raw/pty_size.txt       |  31 +
 .../terminal-engine/results/raw/run_meta.txt       |  53 ++
 .../results/raw/scrollback_bounded.txt             |  37 +
 .../results/raw/scrollback_discard.txt             |  37 +
 .../results/raw/scrollback_unbounded.txt           |  37 +
 .../results/raw/tools_available.txt                |  46 +
 spikes/terminal-engine/results/raw/top.bin         |  48 +
 spikes/terminal-engine/results/raw/top.bin.meta    |  10 +
 spikes/terminal-engine/results/raw/top.esc         |  43 +
 spikes/terminal-engine/results/raw/top.seq         | 619 +++++++++++++
 spikes/terminal-engine/results/raw/top.summary.txt |  56 ++
 spikes/terminal-engine/scripts/run.sh              | 167 ++++
 spikes/terminal-engine/src/bin/capture_stream.rs   | 353 ++++++++
 spikes/terminal-engine/src/bin/esc_selftest.rs     | 116 +++
 spikes/terminal-engine/src/bin/pty_size.rs         | 216 +++++
 spikes/terminal-engine/src/bin/scrollback_mem.rs   | 377 ++++++++
 spikes/terminal-engine/src/esc.rs                  | 439 +++++++++
 spikes/terminal-engine/src/lib.rs                  |   8 +
 spikes/terminal-engine/src/pty.rs                  | 214 +++++
 58 files changed, 8300 insertions(+)
```

---

## 10. `git show --stat --oneline 31bed301` — merge of ADR-001/002

Command:
```
git show --stat --oneline 31bed301
```
Exit code: `0`
Raw output:
```
31bed30 INT-002: integrate research/adr-001-002 (ADR-001 domain model, ADR-002 protocol/transport, interface contract)

 ...R-001-domain-model-and-dependency-boundaries.md | 467 +++++++++++++++++++++
 docs/adr/ADR-002-protocol-schema-and-transport.md  | 461 ++++++++++++++++++++
 docs/architecture/interface-contract.md            | 316 ++++++++++++++
 3 files changed, 1244 insertions(+)
```

---

## 11. First appearance of `docs/adr/` and ADR-003's existence claim

Command:
```
git log --diff-filter=A --format='%h %ad %s' --date=short -- docs/adr
```
Exit code: `0`
Raw output:
```
69e5600 2026-10-16 ADR-001/ADR-002: domain model + dependency boundaries; protocol schema + transport; §14.4 interface contract
2e41da4 2026-10-16 docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)
```

Command:
```
git log --oneline --follow -- docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md | tail -5
```
Exit code: `0`
Raw output:
```
2e41da4 docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)
```

Supporting commands:

```
$ git show -s --format='%h author=%ai commit=%ci %s' 69e5600 2e41da4 085375c 78b83f2 31bed30
69e5600 author=2026-10-16 00:09:31 +0700 commit=2026-10-16 00:09:31 +0700 ADR-001/ADR-002: domain model + dependency boundaries; protocol schema + transport; §14.4 interface contract
2e41da4 author=2026-10-16 00:08:21 +0700 commit=2026-10-16 00:08:21 +0700 docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)
085375c author=2026-10-16 00:06:16 +0700 commit=2026-10-16 00:06:16 +0700 spike(terminal-engine): add engine-independent PTY/resize/byte-stream/RSS harness
78b83f2 author=2026-10-16 00:13:04 +0700 commit=2026-10-16 00:13:04 +0700 INT-002: integrate research/term-001-adr-003 (TERM-001 PTY spike + ADR-003 Proposed)
31bed30 author=2026-10-16 00:13:04 +0700 commit=2026-10-16 00:13:04 +0700 INT-002: integrate research/adr-001-002 (ADR-001 domain model, ADR-002 protocol/transport, interface contract)
exit=0

$ git merge-base --is-ancestor 69e5600 2e41da4   # 69e5600 ancestor of 2e41da4?
exit=1
$ git merge-base --is-ancestor 2e41da4 69e5600   # 2e41da4 ancestor of 69e5600?
exit=1

$ git show -s --format='%h parents=%p' 2e41da4 69e5600 085375c
2e41da4 parents=085375c
69e5600 parents=b96da27
085375c parents=b96da27
exit=0

$ git ls-tree --name-only 2e41da4^ docs/
docs/agent-loop
docs/compatibility
docs/operations
docs/research
exit=0
$ git ls-tree --name-only 69e5600^ docs/
docs/agent-loop
docs/compatibility
docs/operations
docs/research
exit=0

$ git log --all --diff-filter=A --format='%h %ci %s' --date=iso -- docs/adr
69e5600 2026-10-16 00:09:31 +0700 ADR-001/ADR-002: domain model + dependency boundaries; protocol schema + transport; §14.4 interface contract
2e41da4 2026-10-16 00:08:21 +0700 docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)
```

`git log` prints newest-first. The **earliest** commit that added any file under `docs/adr/`
is therefore the last line, `2e41da4` (2026-10-16 00:08:21 +0700, the ADR-003 commit); the next is
`69e5600` (2026-10-16 00:09:31 +0700, ADR-001 and ADR-002). Both are sibling branches sharing the
same parent `b96da27`, and neither is an ancestor of the other. `2e41da4^` (`085375c`) and
`69e5600^` (`b96da27`) both have no `docs/adr/` directory in their trees, so the directory did not
exist in either branch's base.

ADR-003's claim (lines ~620–627): *"`docs/adr/` did not exist before this file was written
(`ls docs/adr` → `No such file or directory`)"*. Verified as **consistent with the commit
evidence**: ADR-003's commit `2e41da4` (00:08:21) is the earliest commit to add anything under
`docs/adr/`, and its parent tree has no `docs/adr/`. ADR-001/002 were authored/committed on a
parallel branch only ~70 seconds later (00:09:31), so the claim is true for the worktree/branch in
which ADR-003 was authored and by earliest-commit ordering, but a reader who sees `git log`'s
newest-first order (`69e5600` printed first) could mistakenly conclude ADR-001/002 predate it.

---

## 12. Reconciliation of TESTS.md:112 (`58 files +8300`) vs ADR-003 (`57 files changed, 7674 insertions(+)`)

Command:
```
git show --stat --oneline 2e41da4 | tail -2
```
Exit code: `0`
Raw output:
```
 ...DR-003-terminal-engine-and-gpui-artifact-pin.md | 626 +++++++++++++++++++++
 1 file changed, 626 insertions(+)
```

Command:
```
git show --shortstat 085375c 2e41da4
```
Exit code: `0`
Raw output:
```
commit 085375c5fe02e79fbecb6d057a5aea306c7a3565
Author: Sulthonul Adib <adibcyber4rt@gmail.com>
Date:   Fri Oct 16 00:06:16 2026 +0700

    spike(terminal-engine): add engine-independent PTY/resize/byte-stream/RSS harness
    
    Phase 1 of TERM-001/ADR-003 (DEC-004): standalone crate under spikes/terminal-engine/ proving real PTY ownership, real TIOCSWINSZ resize + SIGWINCH, raw full-screen byte streams with byte-level escape classification (nvim/less/top; OSC 8 via ls --hyperlink), a 64 MiB scrollback/RSS observation, and a detector self-test. No terminal-emulation engine dependency; phase 2 cancelled by owner direction. All results are provenance-stamped and generated by scripts/run.sh on this host; cargo build / fmt --check / clippy --all-targets all exit 0.

 57 files changed, 7674 insertions(+)

commit 2e41da4326f51752c80f318b7868918bc207315d
Author: Sulthonul Adib <adibcyber4rt@gmail.com>
Date:   Fri Oct 16 00:08:21 2026 +0700

    docs(adr): ADR-003 terminal engine and GPUI artifact pin (Proposed; PTY-level spike evidence; engine-level spike pending)

 1 file changed, 626 insertions(+)
```

Command:
```
git show --shortstat 78b83f2
```
Exit code: `0`
Raw output:
```
 58 files changed, 8300 insertions(+)
```

Context (already quoted above): `docs/agent-loop/TESTS.md:112` reads
`| 48 | ... | 2e41da4+085375c (spike+ADR-003, 58 files +8300); ...`.
`docs/adr/ADR-003-...md` (~line 570) cites
`git show --stat 085375c5fe02e79fbecb6d057a5aea306c7a3565 | tail -1` → `57 files changed, 7674 insertions(+)`.

**Exact reconciliation:** There is no contradiction; the two figures measure different scopes.
`085375c` alone (the spike) is **57 files / 7674 insertions**. `2e41da4` alone (ADR-003) is
**1 file / 626 insertions**. TESTS.md row 48 reports the two commits **together**:
1 + 57 = **58 files**, and 626 + 7674 = **8300 insertions**, which is exactly the shortstat of the
integration merge `78b83f2` (`58 files changed, 8300 insertions(+)`). So `58 / 8300` **is the sum of
`2e41da4` + `085375c`**, and ADR-003's `57 / 7674` is correct for the spike commit in isolation.
Neither figure is wrong; they must not be compared as if they described the same diff range.

---

## 13. Tracked `docs/reviews/` and spike evidence files

Command:
```
git ls-files docs/reviews | head
```
Exit code: `0`
Raw output:
```
(empty — no tracked files under docs/reviews/)
```
`docs/reviews/` was untracked before this write; it now contains only this evidence file,
which is intentionally untracked.

Command:
```
git ls-files spikes/terminal-engine/results/raw | head -40
```
Exit code: `0`
Raw output:
```
spikes/terminal-engine/results/raw/checks/cargo_build.txt
spikes/terminal-engine/results/raw/checks/cargo_clippy.txt
spikes/terminal-engine/results/raw/checks/cargo_fmt_check.txt
spikes/terminal-engine/results/raw/esc_selftest.txt
spikes/terminal-engine/results/raw/less.bin
spikes/terminal-engine/results/raw/less.bin.meta
spikes/terminal-engine/results/raw/less.esc
spikes/terminal-engine/results/raw/less.seq
spikes/terminal-engine/results/raw/less.summary.txt
spikes/terminal-engine/results/raw/nvim.bin
spikes/terminal-engine/results/raw/nvim.bin.meta
spikes/terminal-engine/results/raw/nvim.esc
spikes/terminal-engine/results/raw/nvim.seq
spikes/terminal-engine/results/raw/nvim.summary.txt
spikes/terminal-engine/results/raw/nvim_mouse.bin
spikes/terminal-engine/results/raw/nvim_mouse.bin.meta
spikes/terminal-engine/results/raw/nvim_mouse.esc
spikes/terminal-engine/results/raw/nvim_mouse.seq
spikes/terminal-engine/results/raw/nvim_mouse.summary.txt
spikes/terminal-engine/results/raw/nvim_title.bin
spikes/terminal-engine/results/raw/nvim_title.bin.meta
spikes/terminal-engine/results/raw/nvim_title.esc
spikes/terminal-engine/results/raw/nvim_title.seq
spikes/terminal-engine/results/raw/nvim_title.summary.txt
spikes/terminal-engine/results/raw/osc8_ls.bin
spikes/terminal-engine/results/raw/osc8_ls.bin.meta
spikes/terminal-engine/results/raw/osc8_ls.esc
spikes/terminal-engine/results/raw/osc8_ls.seq
spikes/terminal-engine/results/raw/osc8_ls.summary.txt
spikes/terminal-engine/results/raw/pty_size.txt
spikes/terminal-engine/results/raw/run_meta.txt
spikes/terminal-engine/results/raw/scrollback_bounded.txt
spikes/terminal-engine/results/raw/scrollback_discard.txt
spikes/terminal-engine/results/raw/scrollback_unbounded.txt
spikes/terminal-engine/results/raw/tools_available.txt
spikes/terminal-engine/results/raw/top.bin
spikes/terminal-engine/results/raw/top.bin.meta
spikes/terminal-engine/results/raw/top.esc
spikes/terminal-engine/results/raw/top.seq
spikes/terminal-engine/results/raw/top.summary.txt
```
The spike's committed evidence files are present in the tracked tree (40 entries shown; the full
`results/raw/` set plus `results/inputs/` and `results/README.md` are tracked as part of the spike).

---

## Reconciliation notes (summary paragraph)

HEAD is `a1091e58f32d363ceb2614ab4915329b5de6a869` on branch `review/review-001-adr-pass` with a
clean working tree (`git status --porcelain -b` shows only `## review/review-001-adr-pass`); the
`date -u` at collection time was `Thu Oct 15 06:23:09 PM UTC 2026` (which is 2026-10-16 ~01:23
+0700, matching the repository commit dates). The apparent `TESTS.md` vs ADR-003 discrepancy is not
a real one: `085375c` (spike) is `57 files changed, 7674 insertions(+)`, `2e41da4` (ADR-003) is
`1 file changed, 626 insertions(+)`, and TESTS.md's `58 files +8300` is exactly their sum, which
also equals the integration merge `78b83f2`'s shortstat `58 files changed, 8300 insertions(+)`;
ADR-003 cites only the spike commit, so both numbers are correct at their respective scopes. For
ADR-003's "`docs/adr/` did not exist" claim, the earliest commit adding anything under `docs/adr/`
is `2e41da4` itself (2026-10-16 00:08:21 +0700, ADR-003); ADR-001/002 (`69e5600`, 00:09:31 +0700)
came ~70 seconds later on a sibling branch sharing parent `b96da27`, and neither parent tree
contained `docs/adr/`, so the claim is consistent with the commit evidence while being narrowly
scoped to ADR-003's own branch/worktree. `docs/reviews/` had no tracked files before this write,
and the spike's committed `results/raw/` evidence files are present in the tree.
