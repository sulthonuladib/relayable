# REVIEW-001 — independent review of ADR-001, ADR-002, ADR-003 and the terminal-engine spike

## 1. Header block

**Reviewer role.** Independent, read-only review; owned writable path `docs/reviews/**` only.
No artifact under review was edited.

**Branch / worktree.** `review/review-001-adr-pass` @ `a1091e58` in
`/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`.

**Source commits reviewed.** `69e5600` (ADR-001 + ADR-002 + interface contract);
`2e41da4` (ADR-003) + `085375c` (spike); integration `78b83f2` (term-001/ADR-003) and
`31bed301` (adr-001/002).

**Date.** `Thu Oct 15 06:23:09 PM UTC 2026` (`repo-facts.md` §1).

---

## 2. Scope and method

Reviewed artifacts: `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md`,
`docs/adr/ADR-002-protocol-schema-and-transport.md`,
`docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md`,
`docs/architecture/interface-contract.md`, and the committed Terminal Engine spike
(`spikes/terminal-engine/**`, commit `085375c`).

Method: independent read-only evidence agents + separate read-only reviewers + primary-source
checks + adversarial counterarguments. Raw evidence lives in four files:

- `docs/reviews/evidence/repo-facts.md`
- `docs/reviews/evidence/adr-002-protocol-local-checks.md`
- `docs/reviews/evidence/adr-002-url-retrieval.md`
- `docs/reviews/evidence/adr-003-spike-rerun.md`

Questions the assignment asked: (1) does each ADR decide what `full.md` requires, honestly, with
verifiable evidence; (2) is the entity/ID coverage complete and consistent across ADR-001 and the
interface contract; (3) is the ADR-002 option analysis and transport posture sound; (4) does
committed evidence support ADR-003's pins and the terminal-engine claim; (5) what is blocking
before the next build steps.

No file under review was modified. `git status --porcelain` at assembly time shows only
`?? docs/reviews/` (untracked review directory).

---

## 3. Verdict summary

| ADR | Verdict | Blocking? | Condition |
|---|---|---|---|
| ADR-001 | accept-with-nits | 1 blocking item for CORE-001 model code | reconcile PTY ID type name |
| ADR-002 | accept-with-nits | 3 blocking items for DEC-003 acceptance | refresh Unverified table; fix overstated coverage; add no-reserialize test |
| ADR-003 + spike | accept-with-nits (pins + provisional direction only; keep status `Proposed`) | 1 blocking item for engine adoption | TERM-002 gates the engine choice, UI-001 gates the gpui pin |

No ADR must be recorded as Accepted until its blocking items land.

---

## 4. ADR-001 findings

ADR-001 covers all eleven `full.md` §4 entities with no invented or renamed entities; its
allowed/forbidden edge set is acyclic and faithful to `full.md` §5.2; non-spec ID choices are
labelled **Relayable-defined (no REX evidence)**. The defects are cross-artifact divergences,
citation-accuracy errors, and one under-labelled choice. None invalidates the architecture. The
PTY naming conflict must be settled before CORE-001 implements `relayable-model`.

| ID | Severity | Artifact | file:line | Finding | Evidence |
|---|---|---|---|---|---|
| A-1 | **BLOCKING** (gates CORE-001 model code) | ADR-001 + contract | `ADR-001…:187` vs `interface-contract.md:51` | Same entity has two Rust ID newtype names. ADR-001's list ends `…HostProfileId, PtyProcessId, ScriptId`; the contract's list is `…ClientId, PtyId, ServerId…`. One source of truth is required. Recommended resolution: `PtyProcessId` (matches entity `PtyProcess`, `ADR-001…:107`), with `interface-contract.md:51` updated — or the reverse. Lead decision reserved, but model code must not start first. | Raw grep (`result:line-spotcheck#1`): `ADR-001-domain-model-and-dependency-boundaries.md:187: \`BlockId\`, \`WindowId\`, \`ClientId\`, \`ServerId\`, \`HostProfileId\`, \`PtyProcessId\`, \`ScriptId\`) in`; `interface-contract.md:51: Rust uses one distinct newtype per kind in \`relayable-model\` (\`SessionId\`, \`WindowId\`, \`BlockId\`, \`ClientId\`, \`PtyId\`,` |
| A-2 | non-blocking | ADR-001 + contract | `ADR-001…:117` vs `interface-contract.md:31` | AutomationScript owning scope drift: `client/server` (ADR-001) vs `server` (contract). | `ADR-001…:117` `\| 11 \| Automation script \| AutomationScript \| script \| client/server \|`; `interface-contract.md:31` `AutomationScript \| script: ID \| server script registry \| server` (`result:line-spotcheck#1`) |
| A-3 | nit | ADR-001 + contract | `interface-contract.md:29` vs `ADR-001… (Events table)` | Event identity `(scope_id, server_sequence)` appears in the contract but is never stated as a tuple in ADR-001, whose Events table carries `scope` + `sequence` only. | `interface-contract.md:29` `\| Event \| \`(scope_id, server_sequence)\` \| a scope \| server \|` (`result:line-spotcheck#1`) |
| A-4 | nit | ADR-001 + contract | `ADR-001…:107,112` vs `interface-contract.md:21,26` | Server/Client scope wording drift ("process" vs "service instance"; "server / session" vs "attached client record / server"). | `result:adr001-review#1` F5 |
| A-5 | nit | ADR-001 + contract | `ADR-001…:88,213`; `interface-contract.md:64` | `revision` "from 1" narrows DOC-001 §8.2's documented `integer, minimum: 0` without saying so. | `result:adr001-review#1` F12 |
| A-6 | nit | ADR-001 | `ADR-001…:429–442` | ADR-001 never references the sibling `docs/architecture/interface-contract.md`, though its §1/§7 mirror ADR-001's entity and edge tables. References list contains only `full.md`, DOC-001, TASKS.md, DECISIONS.md. | `result:line-spotcheck#1` B8: `interface-contract` in ADR-001 → `No matches found` |
| A-7 | nit | ADR-001 | `ADR-001…:119–123` | "Layout/Event/ProgramStatus … have no ID namespace" is a Relayable decision (`full.md` §4:331–343 is silent on addressability) but is not labelled **Relayable-defined**, unlike the prefix table at `:135–145`. | `result:adr001-review#1` F8; `ADR-001…:423` open question |
| A-8 | nit | ADR-001 | `ADR-001…:42,57,442`; `:44`; `:29` | Three citation-accuracy errors: (1) `TASKS.md:98–100` cited but the acceptance text is at `TASKS.md:106–108`; (2) "DECISIONS.md records the row as Pending" (`:44`) but `DECISIONS.md:9` says **Proposed**; (3) `:29` cites `full.md:758` (section header) for a sentence at `full.md:760`. | `DECISIONS.md:9` `\| DEC-002 \| … \| **Proposed** — ADR-001 integrated \`31bed30\` \|`; `TASKS.md:106–108`; `result:adr001-review#1` F2/F3/F13 |
| A-9 | non-blocking (citation overreach) | ADR-001 | `ADR-001…:68,228` | Uses `full.md:804` (terminal byte streams) and `full.md:490` to justify rejecting a persisted event log, although `full.md:490` also permits "selected audit/event data". The no-persisted-event-log decision is still defensible under `full.md:480`. | `full.md:490` "…and selected audit/event data."; `full.md:804` "keep terminal byte streams out of metadata DB" (`result:adr001-review#1` F10) |
| A-10 | nit | ADR-001 | `ADR-001…:71,94` vs `:334` | "two named deferrals" / "Two later splits" while the deferral paragraph names three deferred crates (`relayable-terminal`, `relayable-status`, `relayable-automation`); the "two" appears to count split events, not crates. | `ADR-001…:94`, `:71`, `:334` (`result:adr001-review#1` F11) |

**ADR-001 counterargument (strongest against).** The ID scheme is locked one layer deeper than the
evidence warrants: `full.md:343` requires only "tagged IDs or explicit entity type fields", DOC-001
§5 marks the ID scheme beyond the `type:` prefix as **unknown**, and ADR-001 infers UUIDv7 from
exactly two examples (`ADR-001…:150–170`) then fixes version-7 for every kind (`:151`). There is no
ID-format-version or storage-re-encoding story although the store "may hold the binary value"
(`:176`), and short-ID suffix resolution is left undefined (`:425`). Secondary: `relayable-terminal`/
`relayable-status` are deferred (`:334`) while `full.md` §5.4 requires a documented spike and a
short benchmark/test app **before choosing** a terminal engine; settled by ADR-003's phase-2 spike.
Third: neither artifact states which §14.4 item each owns, and no persisted-metadata set is named.

**Strongest case for accepting.** The entity coverage and dependency graph are faithful and
verifiable; every normative statement carries a citation (four are wrong, none invents); the
Relayable-defined choices are explicitly labelled; the decision set is internally consistent
(`result:adr001-review#1` states no self-contradiction besides the L wording defect); and the
counterargument concerns a labelling/versioning detail, not the architecture, which is sound and
faithful to `full.md` §5.2.

---

## 5. ADR-002 findings

The fourteen decisions are internally coherent, match the frozen interface contract where both
speak, and no ADR-002 decision is contradicted by `full.md`. The architecture (proto3/prost, D5
framing, UDS-only local + off-by-default TLS remote, closed error enum, `major.minor` negotiation)
is acceptable. The **Unverified table is stale on 9 of 10 rows**, one capability-coverage row is
overstated, and D7 omits two now-known operational facts. These are documentation/evidence
updates, not decision changes.

| ID | Severity | file:line | Finding | Evidence |
|---|---|---|---|---|
| B-1 | **BLOCKING** (gates recording DEC-003 as Accepted) | `ADR-002:429–442` (and inline 80,84,85,94,102,105,171,210,260,274,408) | Unverified table is stale on 9 of 10 rows — items #1–#9 settled (#1,#2,#9 by live local experiment, #3–#8 by live retrieval); only #10 (Android QUIC) remains STILL-OPEN. Inline `unverified` cells must be updated. | `adr-002-protocol-local-checks.md` (Items #1,#2,#9); `adr-002-url-retrieval.md` §12 |
| B-2 | **BLOCKING** (gates DEC-003 acceptance) | `ADR-002:391,404` | Capability-coverage row 447 (authn/authz) is overstated: D6 is only the transport peer-credential boundary and the interface contract explicitly defers authentication/authorization mechanisms and `principal` semantics, so `ADR-002:404` "No capability is left uncovered" is false as written. Cite D2 `auth.proto` for the message **surface**, mark the **mechanism** deferred. | `interface-contract.md` §7; `ADR-002:391` (`result:adr002-verify#1` F3) |
| B-3 | **BLOCKING** (gates DEC-003 acceptance) | `ADR-002:206–212`, `:324` | D4's no-decode→re-encode rule is load-bearing (prost DROPS unknown fields) yet D11 row 4 (`:324`) tests only tolerance of unknown optional fields. Add a test asserting no client-side decode→re-encode path. | `adr-002-protocol-local-checks.md` §Item#1: v2 wire bytes 24 → v1 re-encode 9, `extra=""` |
| B-4 | non-blocking | `ADR-002:252,254` | D7 says "TLS 1.3" but `rustls 0.23.46` default features include `tls12`, so the implementation may negotiate TLS 1.2; pin a minimum version or state acceptance. | `adr-002-protocol-local-checks.md` §Item#2 (`cargo tree -e features` → `rustls feature "tls12"`) |
| B-5 | non-blocking | `ADR-002:102,260` | D7 provider disclosure missing: rustls default provider is `aws-lc-rs` (native C/assembly; `aws-lc-sys` 423 `.c` + 951 asm), no OpenSSL; operational consequence (C toolchain/CMake, cross-compile/Android impact under DEC-010) is undisclosed. | `adr-002-protocol-local-checks.md` §Item#2; `DECISIONS.md` DEC-010 |
| B-6 | nit | `ADR-002:408,441` | `protoc` is now verified present on the host (`libprotoc 36.1`, found by prost-build on `PATH` with no extra config) but `AGENTS.md`'s verified-environment table has no `protoc` row; ADR-002 text is stale. | `adr-002-protocol-local-checks.md` §Item#9 (`protoc --version` → `libprotoc 36.1`) |
| B-7 | nit | `ADR-002:24–71` | ADR-002's evidence base (`proto3.md`, `crates-*.json`, READMEs) is not repo-tracked (lives in `/tmp/relayable-adr002-evidence/`), so external claims are not reproducible from the repo. | `adr-002-url-retrieval.md` §10 (`grep -rn 'evolving-your-protocol' --include='*.md' .` → no output) |
| B-8 | nit | `ADR-002:80` | Internal inconsistency: pros cite capnp-rpc "Object-capability remote procedure call system" while the con says "no staged Rust RPC path shown". | `adr-002-url-retrieval.md` §10; `ADR-002:80` |
| B-9 | nit | `ADR-002:94,105` | Treats "flow-control semantics" as an RFC 6455 question, but RFC 6455 defines no application flow control. | `adr-002-url-retrieval.md` §7 (`grep -n -i "flow control" rfc6455.txt` → no output, exit 1) |
| B-10 | nit | `ADR-002:84` | Calls JSON schema/evolution "unverified" when JSON supplies none by design (understated, not wrong). | `adr-002-url-retrieval.md` §4 (`grep -c -i "schema" rfc8259.txt` → `0`) |
| B-11 | nit | `ADR-002:24–71` | Lists `https://docs.rs/prost/latest/...` style settling URLs; use version-pinned URLs so evidence cannot drift. | `ADR-002:24–71` (`result:adr002-verify#1` F7) |

**ADR-002 counterargument (strongest against).** D7's security posture is stated more strongly than
it is pinned — TLS 1.3 is named but `tls12` is on by default in the chosen rustls (B-4), and the
settled `aws-lc-rs` native/C/asm default undercuts the unstated "pure Rust" impression and
complicates the Android/cross-compile goal DEC-010 preserves (B-5). Combined with the stale
Unverified table (B-1), a reader could mistake deferred verification for done work.

**Strongest case for accepting.** The option analysis is genuinely comparative and retrieval-dated;
every shared shape is deferred to the frozen contract rather than re-invented; D4's conservative
no-relay rule is *proven necessary* by the prost experiment; D10's 15 error codes exactly match the
contract §6; D11 enumerates the `full.md` §6.1:458 test matrix row-by-row. Nothing rises to a design
defect.

---

## 6. ADR-003 + spike findings

ADR-003 is unusually honest: status `Proposed — PTY-level spike evidence available; engine-level
spike pending` (`ADR-003:5,213`), the engine-behaviour §5.4 rows (`:243,244,246,248,249,250`) are `spike-pending` while `:245` (Hyperlinks) and `:247` (Scrollback) are marked evidenced/mechanism-only, and an
explicit disclaimer that phase 1 is engine-independent (`:253–262`). The four pins are published and
resolvable (Alacritty 0.26.0 and gpui 0.2.2 checksums verified against the crates.io index). However
the Decision pins three engines that **no committed evidence exercises**, so the pins are a declared
direction, not an evidenced choice. Keep `Proposed`; TERM-002 (engine adapter spike) is a hard gate
for engine adoption, UI-001 for the gpui pin.

| ID | Severity | file:line | Finding | Evidence |
|---|---|---|---|---|
| C-1 | **BLOCKING** (for engine adoption) | `spikes/terminal-engine/Cargo.toml:31–34`; `ADR-003:217–223,243–250` | No committed evidence exercises any engine. The spike crate lists only `nix`+`libc` and comments "deliberately depends on no terminal-emulation engine"; the engine-behaviour §5.4 rows (`:243,244,246,248,249,250`) are `spike-pending` while `:245` (Hyperlinks) and `:247` (Scrollback) are marked evidenced/mechanism-only; `full.md` §5.4:424/433 require a documented spike and a benchmark/test app **before choosing**; `full.md` §5.3:417 requires proving a minimal GPUI window builds (UI-001 outstanding). Therefore the three engine pins are a declared direction, not an evidenced choice: TERM-002 is a hard gate for engine adoption and UI-001 for the `gpui = "=0.2.2"` pin. | `adr-003-spike-rerun.md` §8, §9; `ADR-003:255–256` "validates neither `alacritty_terminal` nor any engine" |
| C-2 | non-blocking (evidence inaccuracy) | `ADR-003:238` | Claims SGR counts "…/2/374" but its cited `top.summary.txt:51` records `count=335`; `results/README.md:73` repeats 374 (nvim 337 and less 2 match). | `ADR-003:238`; `spikes/terminal-engine/results/raw/top.summary.txt:51` (`result:adr003-verify#1` F1) |
| C-3 | non-blocking (reproducibility) | `ADR-003:573`; `results/raw/*` | Committed `results/raw/*` carry `git_head b96da276…` and embed the retired worktree path (also leaking into the OSC-8 `file://` payload); `scripts/run.sh` deletes/regenerates `results/raw`, so re-running is replacement not reproduction; `:573` discloses `git_head` but not the retired path or the reproducibility limit. | `adr-003-spike-rerun.md` §6 (`pty_size.txt` provenance header `# git_head: b96da276…`; `cargo_clippy.txt` embeds `research-term-001-adr-003`) |
| C-4 | non-blocking (ephemeral evidence) | `ADR-003:350–356,368–378,427–465,471–560` | Published-source inspection and license-file sha256s come from `/tmp/term001-evidence/`, not repo-tracked; not reproducible from the repo. | `ADR-003:350–356`; `result:adr003-verify#1` F2 |
| C-5 | nit | `ADR-003:163–164,303,392` | `.cargo_vcs_info.json` `"dirty": true` + sha1 `69e2130295c2649963eb639fc70b4f2ee8ea1624` is ADR-self-quoted from the extracted tarball; independent evidence verified resolution/checksums only. | `ADR-003:392`; `adr-003-spike-rerun.md` §7 |
| C-6 | non-blocking (scope, disclosed) | `ADR-003:217–223` vs `:255–256` | The Decision pins three engines while ADR-003 itself says phase 1 "validates neither `alacritty_terminal` nor any engine". Disclosed, so a scope limit rather than a defect. | `ADR-003:217–223`, `:255–256` |

**Positives recorded (verified).**
- All four pins resolve on crates.io and are not yanked: `alacritty_terminal` 0.26.0, `vte` 0.15.0,
  `portable-pty` 0.9.0, `gpui` 0.2.2 (checksum `979b45cf…` matches `ADR-003:158`).
  Evidence: `adr-003-spike-rerun.md` §7–8.
- `cargo build`, `cargo clippy --all-targets`, `cargo fmt --check` all exit 0 with zero warnings.
  Evidence: `adr-003-spike-rerun.md` §1–3.
- Live `pty_size` reproduces `MASTER_WINSIZE_INITIAL=24 80` / `SET_WINSIZE_RC=0` /
  `CHILD_STTY_SIZE_AFTER=40 120` / `SIGWINCH_OBSERVED=true` / `VERDICT=PASS`; `esc_selftest`
  reproduces `SELFTEST_FAILURES=0` / `VERDICT=PASS`. Evidence: `adr-003-spike-rerun.md` §4–5.
- "57 files changed, 7674 insertions(+)" confirmed for `085375c`; the TESTS.md "58/8300" is the sum
  of `2e41da4` (1/626) + `085375c` (57/7674) = merge `78b83f2` (`58 files changed, 8300
  insertions(+)`), so there is no contradiction.
- `docs/adr/` was first created by `2e41da4`, consistent with `ADR-003:622`.
  Evidence: `repo-facts.md` §8, §11–12.

**ADR-003 counterargument (strongest against accepting).** `full.md` §5.4:424 says do not write a
terminal emulator from scratch unless candidates fail a documented spike, and §5.4:433 says create a
benchmark/test app **before choosing**; the engine-level spike was cancelled, so the choice is
recorded before the required test. Pinning `gpui = "=0.2.2"` also precedes the §5.3:417 requirement
to prove a minimal window builds. Settled by keeping `Proposed` and gating on TERM-002/UI-001.

**Strongest case for accepting.** The ADR does not claim an accepted engine choice — status
`Proposed`, the engine-behaviour §5.4 rows (`:243,244,246,248,249,250`) are `spike-pending` while `:245` (Hyperlinks) and `:247` (Scrollback) are marked evidenced/mechanism-only, the honest statement disclaims engine
validation, and the reconsideration list is concrete and falsifiable. The pins themselves are
verified published artifacts, and the two engine-independent probes are genuinely reproducible.

---

## 7. ADR-002 unverified items — settled vs still-open

| # | Item | ADR-002 status | Determination now | Evidence (command/URL + date) | ADR line to change |
|---|---|---|---|---|---|
| 1 | prost unknown fields | unverified | **SETTLED: DROPS** | local round-trip experiment 24→9 bytes; `prost-derive-0.14.4/src/lib.rs:191` `skip_field` | 433, 210 |
| 2 | rustls | unverified | **SETTLED: default provider aws-lc-rs, no OpenSSL, NOT pure Rust** | `cargo tree \| grep -i -E 'openssl\|ring\|aws-lc'` | 434, 260, 102 |
| 3 | Cap'n Proto | unverified | **SETTLED** (add fields/methods + rename safe; ordinal/type/default changes unsafe; adding a field to an existing union is safe, moving one is not) | `https://capnproto.org/language.html` HTTP 200, sha256 `b45ed50c…`, 2026-10-15T18:27:08Z | 435, 80 |
| 4 | FlatBuffers | unverified | **SETTLED** (date-based `YY.MM.DD`, not SemVer; add-at-end, `deprecated`, `--conform`) | `https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md` sha256 `14da2c0c…` (wiki page `https://github.com/google/flatbuffers/wiki/Versioning` sha256 `ae028b2e…`) + `https://flatbuffers.dev/evolution/` sha256 `2e5c610c…`, 18:27:08–34Z | 436, 81 |
| 5 | JSON/serde_json | unverified | **SETTLED** (RFC 8259 syntax only; RFC 7493 I-JSON profile; no schema/evolution exists by design) | rfc8259 sha256 `61a5378f…`, rfc7493 sha256 `a44ccf14…`, 18:27:09Z | 437, 84 |
| 6 | Avro | unverified | **SETTLED** (writer schema accompanies data; name match; reader defaults; listed promotions) | `https://avro.apache.org/docs/1.11.1/specification/` sha256 `f8cb0cb4…`, 18:27:15Z | 438, 85 |
| 7 | RFC 6455 | unverified | **SETTLED** (framing + Close/Ping/Pong ≤125 B unfragmented + masking + status-coded close; NO application flow control) | rfc6455.txt sha256 `76577532…`, 18:27:16Z | 439, 94, 105 |
| 8 | gRPC version skew | unverified | **SETTLED** (`grpc/grpc` `doc/versioning.md`: SemVer + mandatory wire interop; no negotiation handshake) | `https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md` sha256 `678c7ffb…`, 18:27:16Z | 440, 274 |
| 9 | protoc | unverified | **SETTLED: present, `/usr/bin/protoc`, `libprotoc 36.1`, prost-build found it on PATH with no extra config** | `protoc --version` | 441, 408 |
| 10 | Android QUIC viability | unverified | **STILL-OPEN** (behavioral/platform; needs a real Android/NDK build+run and device measurements) | no URL | 442 unchanged |

9 of 10 items are settled; #10 remains STILL-OPEN. Note: the researcher session that produced
`result:adr002-docs#1` had no web tools; its un-retrieved analysis was superseded by the live curl
pass (`adr-002-url-retrieval.md`), which also corrected its Cap'n-Proto union claim (adding a field
to an existing union is safe; moving one is unsafe).

---

## 8. Exact commands and observed results

Every row is copied from the named evidence file (trimmed where marked); untrimmed output is in
`docs/reviews/evidence/*.md`.

| Command (verbatim) | Exit code | Observed (trimmed, verbatim from evidence) | Evidence file |
|---|---|---|---|
| `date -u` | 0 | `Thu Oct 15 06:23:09 PM UTC 2026` | `repo-facts.md` §1 |
| `git rev-parse HEAD` | 0 | `a1091e58f32d363ceb2614ab4915329b5de6a869` | `repo-facts.md` §3 |
| `git status --porcelain -b` | 0 | `## review/review-001-adr-pass` | `repo-facts.md` §4 |
| `git show --stat --oneline 69e5600` | 0 | `3 files changed, 1244 insertions(+)` | `repo-facts.md` §6 |
| `git show --stat --oneline 2e41da4` | 0 | `1 file changed, 626 insertions(+)` | `repo-facts.md` §7 |
| `git show --stat --oneline 085375c` | 0 | `57 files changed, 7674 insertions(+)` | `repo-facts.md` §8 |
| `git show --stat --oneline 78b83f2` | 0 | `58 files changed, 8300 insertions(+)` | `repo-facts.md` §9 |
| `git show --stat --oneline 31bed301` | 0 | `3 files changed, 1244 insertions(+)` | `repo-facts.md` §10 |
| `git log --diff-filter=A --format='%h %ad %s' --date=short -- docs/adr` | 0 | `69e5600 2026-10-16 ADR-001/ADR-002: …` / `2e41da4 2026-10-16 docs(adr): ADR-003 …` | `repo-facts.md` §11 |
| `cargo build` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.92s` | `adr-003-spike-rerun.md` §1 |
| `cargo clippy --all-targets` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.40s` (no `warning:`/`error:`) | `adr-003-spike-rerun.md` §2 |
| `cargo fmt --check` | 0 | (empty stdout and stderr) | `adr-003-spike-rerun.md` §3 |
| `./target/debug/pty_size` | 0 | `MASTER_WINSIZE_INITIAL=24 80` … `SET_WINSIZE_RC=0` … `CHILD_STTY_SIZE_AFTER=40 120` … `SIGWINCH_OBSERVED=true` … `VERDICT=PASS` | `adr-003-spike-rerun.md` §4 |
| `./target/debug/esc_selftest` | 0 | `SELFTEST_FAILURES=0` / `VERDICT=PASS` | `adr-003-spike-rerun.md` §5 |
| `cargo metadata --format-version 1 --manifest-path /tmp/adr003-resolve/Cargo.toml` | 0 | resolved `alacritty_terminal = 0.26.0`, `gpui = 0.2.2`, `portable-pty = 0.9.0`, `vte = 0.15.0` | `adr-003-spike-rerun.md` §7 |
| `cargo fetch --manifest-path /tmp/adr003-resolve/Cargo.toml` | 0 | empty stderr | `adr-003-spike-rerun.md` §7 |
| `protoc --version` | 0 | `libprotoc 36.1` | `adr-002-protocol-local-checks.md` §Item#9 |
| `cd /tmp/adr002-unknown && cargo run --quiet` | 0 | `v2 wire bytes (24): …` … `v1 re-encoded bytes (9): 08 07 12 05 68 65 6c 6c 6f` … `RESULT: DROPPED - unknown field 3 did NOT survive v1 decode + v1 re-encode` | `adr-002-protocol-local-checks.md` §Item#1 |
| `cd /tmp/adr002-rustls && cargo tree \| grep -i -E 'openssl\|ring\|aws-lc'` | 0 | `├── aws-lc-rs v1.18.2` / `│   ├── aws-lc-sys v0.46.0` (no `openssl`, no `ring`) | `adr-002-protocol-local-checks.md` §Item#2 |
| `curl … https://capnproto.org/language.html` | 0 | `HTTP_STATUS=200`, 61485 bytes, sha256 `b45ed50c1e26e92435fe74f179504f6ea5ac97e3bc705c32bd8162748508f127` | `adr-002-url-retrieval.md` §1 |
| `curl … https://github.com/google/flatbuffers/wiki/Versioning` | 0 | `HTTP_STATUS=200`, 201400 bytes, sha256 `ae028b2e…` (raw md sha256 `14da2c0c…`) | `adr-002-url-retrieval.md` §2 |
| `curl … https://flatbuffers.dev/evolution/` | 0 | `HTTP_STATUS=200`, 52419 bytes, sha256 `2e5c610cc1b5bc5a0968f150ae0cbab51c6321948f886d37ca6d74b161d7df8c` | `adr-002-url-retrieval.md` §3 |
| `curl … https://raw.githubusercontent.com/google/flatbuffers/master/docs/source/FlatBuffers.md` | 0 | `HTTP_STATUS=404`, 14 bytes (**BLOCKED**) | `adr-002-url-retrieval.md` §3 |
| `curl … https://www.rfc-editor.org/rfc/rfc8259.txt` | 0 | `HTTP_STATUS=200`, sha256 `61a5378f4255c720…` | `adr-002-url-retrieval.md` §4 |
| `curl … https://www.rfc-editor.org/rfc/rfc7493.txt` | 0 | `HTTP_STATUS=200`, sha256 `a44ccf1451a7bf4f…` | `adr-002-url-retrieval.md` §5 |
| `curl … https://avro.apache.org/docs/1.11.1/specification/` | 0 | `HTTP_STATUS=200`, sha256 `f8cb0cb4475949a9…` | `adr-002-url-retrieval.md` §6 |
| `curl … https://www.rfc-editor.org/rfc/rfc6455.txt` | 0 | `HTTP_STATUS=200`, sha256 `765775326aee0ecc…` | `adr-002-url-retrieval.md` §7 |
| `curl … https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md` | 0 | `HTTP_STATUS=200`, sha256 `678c7ffb0d6f9b8b…` | `adr-002-url-retrieval.md` §8a |
| `curl … https://grpc.io/docs/guides/versioning/` | 0 | `HTTP_STATUS=404`, 28782 bytes (**BLOCKED**) | `adr-002-url-retrieval.md` §8b |
| `grep -c -i "schema" /tmp/adr002fetch/rfc8259.txt` | 1 (inferred: `grep -c` exits 1 on zero matches; the evidence file records the output only) | `0` | `adr-002-url-retrieval.md` §4 |
| `grep -n -i "flow control" /tmp/adr002fetch/rfc6455.txt` | 1 | (no output) | `adr-002-url-retrieval.md` §7 |
| `grep -rn 'evolving-your-protocol' --include='*.md' .` | 1 (inferred; the evidence file records `(no output)`, not an exit code) | (no output) | `adr-002-url-retrieval.md` §10 |
| `grep -i 'openssl' Cargo.lock` (in `/tmp/adr002-rustls`) | 1 | (no match) | `adr-002-protocol-local-checks.md` §Item#2 |
| ADR-001 grep line checks (`PtyProcessId` / `PtyId` / `client/server` / `TASKS.md:` etc.) | n/a | see §3 A-1/A-2/A-6/A-8 | `result:line-spotcheck#1` |
| `git log --oneline -3 HEAD` | 0 | `a1091e5 DEC-010: defer Android client …` / `d42ae72 Docs: add restart trigger …` / `19eb33a DEC-009: remove dead auto-rename hook block …` | captured live at assembly (read-only) |
| `git log --oneline -3 master` | 0 | `0d002ac agent-loop: launch REVIEW-001, DOC-002 repair, TERM-002 (checkpoint 12)` / `a1091e5 DEC-010: defer Android client …` / `d42ae72 Docs: add restart trigger …` | captured live at assembly (read-only) |

### Web retrievals

All retrievals 2026-10-15, 18:27:0x–34Z (`adr-002-url-retrieval.md`).

| URL | HTTP | sha256 (short) |
|---|---|---|
| `https://capnproto.org/language.html` | 200 | `b45ed50c1e26e924` |
| `https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md` | 200 | `14da2c0c86d86867` |
| `https://flatbuffers.dev/evolution/` | 200 | `2e5c610cc1b5bc5a` |
| `https://www.rfc-editor.org/rfc/rfc8259.txt` | 200 | `61a5378f4255c720` |
| `https://www.rfc-editor.org/rfc/rfc7493.txt` | 200 | `a44ccf1451a7bf4f` |
| `https://avro.apache.org/docs/1.11.1/specification/` | 200 | `f8cb0cb4475949a9` |
| `https://www.rfc-editor.org/rfc/rfc6455.txt` | 200 | `765775326aee0ecc` |
| `https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md` | 200 | `678c7ffb0d6f9b8b` |
| `https://grpc.io/docs/guides/versioning/` | **404** | `43fe38df1bcdbd3b` |
| `https://flatbuffers.dev/flatbuffers_guide_writing_schema.html` (redirect stub) | 200 | `653ba106b3e75bb8` |
| `https://raw.githubusercontent.com/google/flatbuffers/master/docs/source/FlatBuffers.md` | **404** | `d5558cd419c8d46b` |

`docs/reviews/evidence/*.md` contains the untrimmed output and full retrieval records.

---

## 9. Blocking vs non-blocking summary + recommended Manager actions

| Item | Gates | Action owner | Action |
|---|---|---|---|
| A-1 | CORE-001 model code | Lead | Pick one PTY ID newtype name before writing `relayable-model` (recommend `PtyProcessId`, update `interface-contract.md:51`). |
| B-1 / B-2 / B-3 | recording DEC-003 as Accepted | Lead / Manager | Refresh ADR-002 Unverified table + inline cells (B-1); fix coverage row 447 / claim 404 (B-2); add a no-reserialize test (B-3). |
| C-1 | adoption of any terminal engine / the gpui pin | Lead | Keep ADR-003 `Proposed`; accept it only as a pins + provisional-direction record. TERM-002 (engine adapter spike: reflow, selection, wide/Unicode glyphs, vim/nvim/tmux/htop round-trips, parser/view separation) is a HARD GATE for the engine choice; UI-001 gates the gpui pin. |
| A-2–A-10, B-4–B-11, C-2–C-6 | documentation quality | Lead | Fix on the next ADR pass; none gates a build step. |

**Explicit answer.** TERM-002 is a hard gate for accepting the engine choice; it is **NOT** a
precondition for starting CORE-001 provided CORE-001 does not adopt `alacritty_terminal`/`vte`/
`portable-pty`.

---

## 10. Independence, limitations, and repo state

- **Independence.** Evidence was produced by separate agents (repo-facts, adr002-local, adr002-fetch,
  spike-rerun); findings came from separate read-only reviewers (`result:adr001-review#1`,
  `result:adr002-verify#1`, `result:adr003-verify#1`); primary sources were fetched and hashed; each
  review states adversarial counterarguments. The scribe that assembled this file is not an analyst
  and reproduced the Lead's verdicts/severities/findings faithfully.
- **Researcher web-tool limitation.** `result:adr002-docs#1` had no shell/web and could not retrieve
  any URL; it marked all retrievals "NOT FETCHED". Its analysis is superseded by the live curl pass
  (`adr-002-url-retrieval.md`), which also corrected its Cap'n-Proto union claim.
- **Ephemeral evidence.** ADR-002's staged artifacts (`proto3.md`, `crates-*.json`, READMEs) and
  ADR-003's published-source/license greps live under `/tmp/relayable-adr002-evidence/` and
  `/tmp/term001-evidence/`; they are not repo-tracked, so those claims are not reproducible from the
  repo alone.
- **Historical byte captures.** Committed spike `results/raw/*` are an artifact of tree `b96da276` in
  the since-retired `research-term-001-adr-003` worktree; only `pty_size` and `esc_selftest` are
  reproducible from current HEAD. Re-running `scripts/run.sh` replaces (not reproduces) them.
- **58/8300 vs 57/7674.** Not a contradiction: `085375c` (spike) = 57/7674, `2e41da4` (ADR-003) =
  1/626, and `1+57=58` / `626+7674=8300` equals merge `78b83f2` shortstat.
- **Repo state at review time.** Worktree `review/review-001-adr-pass` @ `a1091e58`; `repo-facts.md`
  records a clean tree (`## review/review-001-adr-pass`). At assembly time the working tree shows
  `## review/review-001-adr-pass` and `?? docs/reviews/`. `master` had advanced past `a1091e58`;
  raw `git log --oneline -3 master` captured live:
  `0d002ac agent-loop: launch REVIEW-001, DOC-002 repair, TERM-002 (checkpoint 12)` /
  `a1091e5 DEC-010: defer Android client from current scope (keep model/protocol Android-ready)` /
  `d42ae72 Docs: add restart trigger to NEXT.md; record Manager auto-start convenience`.

Post-assembly corrections applied after an independent audit (report-audit): exit-code provenance fixed for one grep row; two citation precision fixes (ADR-003 quote line, §5.4 bullet claim); one false citation-error finding (A-8 sub-claim 3) removed; FlatBuffers hash/URL pairing corrected.

---

## 11. Evidence index

- `docs/reviews/evidence/repo-facts.md` — read-only Git transcript: commits, shortstats, `docs/adr/`
  first-appearance, TESTS.md reconciliation, spike evidence file list.
- `docs/reviews/evidence/adr-002-protocol-local-checks.md` — live local experiments settling ADR-002
  items #1 (prost drops unknown fields), #2 (rustls default `aws-lc-rs`, no OpenSSL), #9 (`protoc`).
- `docs/reviews/evidence/adr-002-url-retrieval.md` — live curl pass settling items #3–#8 with HTTP
  status, sha256, retrieval timestamps; item #10 still open.
- `docs/reviews/evidence/adr-003-spike-rerun.md` — spike re-run: build/clippy/fmt, `pty_size` and
  `esc_selftest` reproduction, provenance analysis, pin resolution.

Reviewer findings sources: `result:adr001-review#1`, `result:adr002-verify#1`,
`result:adr003-verify#1` (plus the `result:adr002-*`, `result:spike-rerun#1`, `result:repo-facts#1`,
`result:line-spotcheck#1` evidence refs).
