# ADR-002 live URL retrieval evidence

Task: close the "no retrieval date / not fetched" gap in the unverified-items analysis
(`result:adr002-docs#1`) for items **#3 Cap'n Proto evolution, #4 FlatBuffers versioning/evolution,
#5 JSON/serde_json, #6 Avro schema resolution, #7 RFC 6455, #8 gRPC version skew, #10 Android QUIC**.

Items #1 (`prost` unknown fields), #2 (`rustls`), #9 (`protoc`) are **out of scope** (owned elsewhere).

- **Author/scope:** `generalist:adr002-fetch`. Read-only on the repo except this file.
- **Repo:** `/home/sulthonuladib/.herdr/worktrees/relayable/review-review-001-adr-pass`
- **Fetch dir (outside repo):** `/tmp/adr002fetch/`
- **Run date (UTC):** `2026-10-15` (see per-item `date -u`).
- **Only repo file written:** `docs/reviews/evidence/adr-002-url-retrieval.md`.

## 0. Baseline / network proof

```
$ date -u
Thu Oct 15 06:27:03 PM UTC 2026

$ curl -sS -o /dev/null -w '%{http_code}\n' https://www.rfc-editor.org/rfc/rfc6455.txt
200
$ echo $?
0
```

Network is live and unauthenticated fetches succeed. All retrievals below were run in the same
session at `2026-10-15T18:27:0xZ`–`2026-10-15T18:27:34Z`. (Initial `date -u` printed
`18:27:03Z`; subsequent per-URL `date -u` values are recorded per item.)

Method used for every URL:

```
$ mkdir -p /tmp/adr002fetch
$ date -u
$ curl -sSL -w '\nHTTP_STATUS=%{http_code}\nURL_EFFECTIVE=%{url_effective}\n' \
       -o /tmp/adr002fetch/<name> <url> ; echo "CURL_EXIT=$?"
$ sha256sum /tmp/adr002fetch/<name>
$ wc -c < /tmp/adr002fetch/<name>
```

---

## 1. Cap'n Proto evolution — https://capnproto.org/language.html

- **date -u:** `2026-10-15T18:27:08Z`
- **Command:** `curl -sSL -w '\nHTTP_STATUS=%{http_code}\nURL_EFFECTIVE=%{url_effective}\n' -o /tmp/adr002fetch/capnp-language.html https://capnproto.org/language.html`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://capnproto.org/language.html`
- **Bytes / sha256:** `61485` / `b45ed50c1e26e92435fe74f179504f6ea5ac97e3bc705c32bd8162748508f127`
- **Anchor present:** `<h2 id="evolving-your-protocol">Evolving Your Protocol</h2>` at line 715.

**Quoted excerpt (verbatim, section "Evolving Your Protocol"):**

> New types, constants, and aliases can be added anywhere, since they obviously don't affect the
> encoding of any existing type.
>
> New fields, enumerants, and methods may be added to structs, enums, and interfaces, respectively,
> as long as each new member's number is larger than all previous members. Similarly, new fields
> may be added to existing groups and unions.
>
> …
>
> Any symbolic name can be changed, as long as the type ID / ordinal numbers stay the same.
>
> …
>
> Any change not listed above should be assumed NOT to be safe. In particular:
> - You cannot change a field, method, or enumerant's number.
> - You cannot change a field or method parameter's type or default value.
> - You cannot change a type's ID.
> - You cannot change the name of a type that doesn't have an explicit ID, as the implicit ID is
>   generated based in part on the type name.
> - You cannot move a type to a different scope or file unless it has an explicit ID, as the
>   implicit ID is based in part on the scope's ID.
> - You cannot move an existing field into or out of an existing union, nor can you form a new
>   union containing more than one existing field.

And on unions specifically:

> This caveat only applies when moving an existing field into a new union; **adding new fields to an
> existing union does not create a problem**, because existing programs should already know to
> check the union's tag.

**Verdict: SETTLED.** Safe: adding new fields/enumerants/methods (with increasing ordinals),
re-arranging/retyping names (names are not on the wire, but a type's *implicit* ID depends on its
name). Unsafe: changing ordinals, changing field/parameter types or defaults, changing type IDs,
moving fields into/out of an existing union, forming a new union from >1 existing field.

> ⚠️ **Refutes the attached analysis (`result:adr002-docs#1`) and the task's own #1 claim.**
> Both state that "adding members to an existing union is unsafe". The fetched source says the
> opposite: *"adding new fields to an existing union does not create a problem"*. Only *moving an
> existing field into/out of* a union is unsafe. The source does **not** use the words
> "remove/delete/drop" for fields at all — there is no field-removal rule; fields are retained by
> ordinal. Treat "removing fields is unsafe" as **unsupported by this source** (the actual rule is
> "you cannot renumber", so a deleted ordinal leaves an unused gap).

---

## 2. FlatBuffers versioning rationale — https://github.com/google/flatbuffers/wiki/Versioning

- **date -u:** `2026-10-15T18:27:08Z` (raw fallback) / `2026-10-15T18:27:14Z` (wiki HTML)
- **Commands:**
  - `curl -sSL ... -o /tmp/adr002fetch/flatbuffers-wiki-Versioning.md https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md` → `0` / `200` / `https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md` / `3492` bytes / `14da2c0c86d868671e1b46cdf15a725e1f446d4ebf09e944549b6e86246ccbea`
  - `curl -sSL ... -o /tmp/adr002fetch/flatbuffers-wiki-Versioning.html https://github.com/google/flatbuffers/wiki/Versioning` → `0` / `200` / `https://github.com/google/flatbuffers/wiki/Versioning` / `201400` bytes / `ae028b2e103e8750070d0ff7ad53068513513e6fe2a63ccc9311d0d9e26455b2`

**Quoted excerpt (raw markdown, verbatim):**

> ## Versioning Scheme
>
> FlatBuffers has a version in the format: `YY.MM.DD`
> …
> An example version would be: `22.9.14`
> …
> ### Not Semantic Versioning
>
> This is very similar format to [Semantic Versioning](https://semver.org/), but we don't prescribe
> to hold to semantic versioning.
> …
> Since FlatBuffers involves multiple pieces working in tandem: flatc … the individual language
> "run time" libraries … the schema version … Its hard to assign one version to unite all those
> things together and follow Semantic Versioning exactly.

**Verdict: SETTLED.** FlatBuffers release versioning is date-based (`YY.MM.DD`), explicitly
**not** SemVer. (The wiki documents *release/version-numbering* policy; schema-evolution rules
live on the docs site — see item 3.)

---

## 3. FlatBuffers schema evolution — writing-schema guide (redirected)

- **Requested URL:** `https://flatbuffers.dev/flatbuffers_guide_writing_schema.html`
  - **date -u:** `2026-10-15T18:27:08Z`
  - **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://flatbuffers.dev/flatbuffers_guide_writing_schema.html/`
  - **Bytes / sha256:** `422` / `653ba106b3e75bb8375e663caf0e99e07d4d7e1601038f90e2566fab6c4a3827`
  - **Content:** a **redirect stub only** — `<link rel="canonical" href="../schema/">` and
    `location.href="../schema/"`. **Not usable as evidence.**
- **Followed redirect chain:**
  - `https://flatbuffers.dev/schema/` → `0` / `200` / `82699` bytes / `eaa2c4a4c819cd1fac0edddf5a585d7d4bad406a0dfbfd1dd74d207fcb825328` (`2026-10-15T18:27:21Z`) — nav links to `../evolution/`.
  - `https://flatbuffers.dev/evolution/` → `0` / `200` / `52419` bytes / `2e5c610cc1b5bc5a0968f150ae0cbab51c6321948f886d37ca6d74b161d7df8c` (`2026-10-15T18:27:34Z`) — **canonical evolution rules.**
- **Fallback attempted:** `https://raw.githubusercontent.com/google/flatbuffers/master/docs/source/FlatBuffers.md` → `0` / **`404`** / `14` bytes / `d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed`. **BLOCKED (404, path no longer exists).** (`main` branch and `schemas.md` also 404.)

**Quoted excerpt (from `https://flatbuffers.dev/evolution/`, verbatim):**

> FlatBuffers enables the schema to evolve over time while still maintaining forwards and backwards
> compatibility with old flatbuffers.
>
> ## Addition
> New fields MUST be added to the end of the table definition. …
> You can ignore this rule if you use the `id` attribute on all the fields of a table.
>
> ## Removal
> You MUST not remove a field from the schema, even if you don't use it anymore. You simply stop
> writing them to the buffer. Its encouraged to mark the field `deprecated` by adding the
> `deprecated` attribute. …
>
> ## Name Changes
> Its generally OK to change the name of tables and fields, as these are not serialized to the buffer.
> …
> ## Version Control
> … If this is not practical for you, use explicit field `id`s, which should always generate a merge
> conflict if two people try to allocate the same `id`.
>
> ## Checking Conformity
> … the `flatc` compiler has a option to do just that: `--conform FILE` …

**Verdict: SETTLED** (via the canonical `/evolution/` page, not the requested guide URL).
Rules confirmed: add new fields **at the end** (unless explicit `id`s are used on all fields);
**never remove** a field — mark it `deprecated`; names may change; use explicit `id`s with
`--conform` to enforce evolution. The page does **not** literally say "never reuse ids"; that is a
reasonable paraphrase of "you MUST not remove a field" + explicit-`id` merge-conflict guidance.

---

## 4. JSON syntax — RFC 8259

- **date -u:** `2026-10-15T18:27:09Z`
- **Command:** `curl -sSL ... -o /tmp/adr002fetch/rfc8259.txt https://www.rfc-editor.org/rfc/rfc8259.txt`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://www.rfc-editor.org/rfc/rfc8259.txt`
- **Bytes / sha256:** `28360` / `61a5378f4255c720beb2a4b4a63b29540147c140f36988bf086291989b4cd2d7`

**Quoted excerpt (Abstract, verbatim):**

> JavaScript Object Notation (JSON) is a lightweight, text-based, language-independent data
> interchange format. It was derived from the ECMAScript Programming Language Standard. JSON defines
> a small set of formatting rules for the portable representation of structured data.
> This document removes inconsistencies with other specifications of JSON, repairs specification
> errors, and offers experience-based interoperability guidance.

**Schema/version probe (verbatim command + result):**

```
$ grep -c -i "schema" /tmp/adr002fetch/rfc8259.txt
0
```

(The only case-insensitive matches for "version" in the document concern Unicode/ECMA-262
versions, not data-format versioning.)

**Verdict: SETTLED.** RFC 8259 defines JSON **syntax/formatting only**. It contains **no** schema
and **no** version-negotiation or evolution mechanism. The attached analysis is correct, and the
stronger conclusion holds: JSON supplies *no* schema/evolution guarantees by design.

---

## 5. I-JSON interoperability profile — RFC 7493

- **date -u:** `2026-10-15T18:27:09Z`
- **Command:** `curl -sSL ... -o /tmp/adr002fetch/rfc7493.txt https://www.rfc-editor.org/rfc/rfc7493.txt`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://www.rfc-editor.org/rfc/rfc7493.txt`
- **Bytes / sha256:** `12849` / `a44ccf1451a7bf4f4bbb78b39e10ed0e9a27ff2ee5b639c6458042b5a7197062`

**Quoted excerpt (Abstract, verbatim):**

> I-JSON (short for "Internet JSON") is a restricted profile of JSON designed to maximize
> interoperability and increase confidence that software can process it successfully with
> predictable results.

and §2:

> An I-JSON message is a JSON text, as defined by RFC 7159. … I-JSON messages MUST be encoded using
> UTF-8 [RFC3629].

**Verdict: SETTLED.** I-JSON is a restricted **interoperability profile** of JSON. It still
provides no schema or version negotiation.

---

## 6. Apache Avro schema resolution — https://avro.apache.org/docs/1.11.1/specification/

- **date -u:** `2026-10-15T18:27:15Z`
- **Command:** `curl -sSL ... -o /tmp/adr002fetch/avro-1.11.1-specification.html https://avro.apache.org/docs/1.11.1/specification/`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://avro.apache.org/docs/1.11.1/specification/`
- **Bytes / sha256:** `157942` / `f8cb0cb4475949a9545a78764ddd2049500574c57e07103c8ffdabd35dfc15b3`
- **Fallback `1.12.0` not needed:** primary returned 200; not fetched.

**Quoted excerpt (section "Schema Resolution", verbatim):**

> A reader of Avro data, whether from an RPC or a file, can always parse that data because the
> original schema must be provided along with the data. However, the reader may be programmed to
> read data into a different schema. … We refer to the schema used to write the data as the writer's
> schema, and the schema that the application expects the reader's schema. …
>
> To match, one of the following must hold: both schemas are arrays whose item types match … both
> schemas are records with the same (unqualified) name … either schema is a union … the writer's
> schema may be promoted to the reader's as follows: `int` is promotable to `long`, `float`, or
> `double`; `long` is promotable to `float` or `double`; `float` is promotable to `double`; `string`
> is promotable to `bytes`; `bytes` is promotable to `string`.
>
> **if both are records:** the ordering of fields may be different: fields are matched by name. …
> if the writer's record contains a field with a name not present in the reader's record, the
> writer's value for that field is ignored. if the reader's record schema has a field that contains
> a default value, and writer's schema does not have a field with the same name, then the reader
> should use the default value from its field. if the reader's record schema has a field with no
> default value, and writer's schema does not have a field with the same name, an error is signalled.
>
> **if both are enums:** if the writer's symbol is not present in the reader's enum and the reader
> has a default value, then that value is used, otherwise an error is signalled.
>
> **if both are unions:** The first schema in the reader's union that matches the selected writer's
> union schema is recursively resolved against it. if none match, an error is signalled.

**Verdict: SETTLED.** The writer's schema accompanies the data; resolution matches records by
field name, ignores writer-only fields, uses reader defaults for reader-only fields (else error),
resolves enums/unions per reader's schema, and permits only the enumerated primitive promotions.

---

## 7. WebSocket — RFC 6455

- **date -u:** `2026-10-15T18:27:16Z`
- **Command:** `curl -sSL ... -o /tmp/adr002fetch/rfc6455.txt https://www.rfc-editor.org/rfc/rfc6455.txt`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://www.rfc-editor.org/rfc/rfc6455.txt`
- **Bytes / sha256:** `162067` / `765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b`

**Quoted excerpt (§5.5 Control Frames, lines 1967–1979, verbatim):**

> Control frames are identified by opcodes where the most significant bit of the opcode is 1.
> Currently defined opcodes for control frames include 0x8 (Close), 0x9 (Ping), and 0xA (Pong).
> Opcodes 0xB-0xF are reserved for further control frames yet to be defined.
> …
> All control frames MUST have a payload length of 125 bytes or less and MUST NOT be fragmented.

§5.5.1 Close:

> The Close frame MAY contain a body … If there is a body, the first two bytes of the body MUST be a
> 2-byte unsigned integer (in network byte order) representing a status code with value /code/
> defined in Section 7.4. … Close frames sent from client to server must be masked as per Section 5.3.

**Flow-control probe (verbatim):**

```
$ grep -n -i "flow control" /tmp/adr002fetch/rfc6455.txt
(no output; exit 1)
```

**Verdict: SETTLED.** RFC 6455 defines the base framing, control frames Close(0x8)/Ping(0x9)/
Pong(0xA) limited to **≤125 bytes and non-fragmented**, client→server masking, continuation-based
fragmentation, and a status-coded close handshake. It defines **no application-level flow control**
(0 occurrences of "flow control"; transport flow control is left to TCP).

> ⚠️ **Refutes the ADR-002 wording.** ADR-002 L94 (`docs/adr/ADR-002-protocol-schema-and-transport.md`)
> says `No RFC 6455 staged → frame/close/flow-control semantics **unverified**`. RFC 6455 settles
> frame/close semantics but **has no flow-control semantics at all**, so "flow-control semantics"
> is a nonexistent target — the ADR is checking for a guarantee the RFC does not make. (The
> separate design point "a second framing layer over D5" is a judgment, not a source claim.)

---

## 8. gRPC version skew

### 8a. https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md

- **date -u:** `2026-10-15T18:27:16Z`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / `200` / `https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md`
- **Bytes / sha256:** `6201` / `678c7ffb0d6f9b8b67695954d64f55ed84b50d1f25b29968b012d94de406d684`

**Quoted excerpt (verbatim):**

> All gRPC implementations use a three-part version number (`vX.Y.Z`) and follow semantic versioning
> …
> - **Major version bumps** only happen on rare occasions. … Most importantly, a major version
>   increase must not break wire compatibility with other gRPC implementations so that existing gRPC
>   libraries remain fully interoperable.
> …
> Note that while major version bump allows changing APIs used by the users, it must not impact the
> interoperability of the implementation with other gRPC implementations and the previous major
> version released. That means that **no backward incompatible protocol changes are allowed**: old
> clients must continue interoperating correctly with new servers and new servers with old clients.

### 8b. https://grpc.io/docs/guides/versioning/

- **date -u:** `2026-10-15T18:27:17Z`
- **CURL_EXIT / HTTP_STATUS / URL_EFFECTIVE:** `0` / **`404`** / `https://grpc.io/docs/guides/versioning/`
- **Bytes / sha256:** `28782` / `43fe38df1bcdbd3b31d5d6b84730a6d2066ec850d6d2ac87ed200050f67f0a2e`
- **Note:** returns a 404 page body (28 KB), i.e. the guide URL does not exist.

**Verdict: SETTLED (document located) — but it is *release/library* versioning, not a wire
version-negotiation protocol.** The authoritative gRPC versioning guide exists at
`grpc/grpc:doc/versioning.md`. Its stance on skew is: gRPC libraries follow SemVer, major bumps
are rare, and **wire interoperability across versions is mandatory** ("no backward incompatible
protocol changes are allowed; old clients must continue interoperating correctly with new servers
and new servers with old clients"). It does **not** define a version-negotiation handshake.
The `grpc.io` guide URL is 404.

> ADR-002 L274 / L440 says gRPC version-skew guidance is `**unverified**` / `No such document
> staged; settling URL unknown and must be located`. The settling URL is now located
> (`https://github.com/grpc/grpc/blob/master/doc/versioning.md`), and its content shows gRPC handles
> skew by mandating wire interoperability rather than by negotiating versions.

---

## 9. Item #10 — Android client-platform QUIC viability

- **No fetch can settle this.** No URL is authoritative.
- **Verdict: STILL-OPEN.**
- **Settling evidence required (not speculation):**
  1. A real Android/NDK build of the chosen QUIC stack (e.g. Rust `quinn`, or a platform/Cronet
     stack) for the target ABI(s), recording exact toolchain/versions.
  2. Confirmation the stack builds, links, and **runs** at the target minimum API level, with logs.
  3. Real-device measurements for NAT/traversal, connection migration, and 0-RTT, because D7's
     revisit triggers are behavioral.
- ADR-002 L95/L104 (`A second transport to operate; no staged client-platform support evidence`) is
  accurate and not contradicted by any fetched source.

---

## 10. ADR-002's own sources: does it already link these canonical URLs?

Verbatim command:

```
$ grep -n -i -E 'capnp|flatbuffer|6455|avro|8259|serde_json|grpc' \
    docs/adr/ADR-002-protocol-schema-and-transport.md
```

Results (line numbers in ADR-002, 461 lines total):

| ADR line | Relevant content |
|---|---|
| 46 | `[capnp]` = `capnp-rust-README.md` ← `https://raw.githubusercontent.com/capnproto/capnproto-rust/master/README.md` (200, 2026-10-15T16:58:14Z) |
| 49 | `[flatbuffers]` = `flatbuffers-README.md` ← `https://raw.githubusercontent.com/google/flatbuffers/master/README.md` (200, 2026-10-15T16:58:15Z) |
| 55 | `[crates-apache-avro]` = `crates-apache-avro.json` ← `https://crates.io/api/v1/crates/apache-avro` |
| 54 | `[crates-serde_json]` = `crates-serde_json.json` ← `https://crates.io/api/v1/crates/serde_json` |
| 66/67 | `[grpc-error]`, `[grpc-status]` = grpc.io error/status docs |
| 80/81/84/85/94 | prose "**unverified**" claims for Cap'n Proto / FlatBuffers / JSON / Avro / RFC 6455 |
| 105 | RFC 6455 prose "unstaged → semantics **unverified**" |
| 274 | gRPC version-skew "**unverified**" |
| 435–442 | Unverified-items table rows #3–#7, #10 |

Canonical-evolution-anchor probe:

```
$ grep -rn 'evolving-your-protocol' --include='*.md' .
(no output)
```

**Finding — the attached analysis's provenance claim is only half right:**
- ADR-002's Sources table **does list the upstream README URLs** (`capnp-rust-README.md` L46,
  `flatbuffers-README.md` L49), and the *matching evidence files in `/tmp/relayable-adr002-evidence/`
  do contain the canonical anchors*:
  - `capnp-rust-README.md` L61: `- [protocol evolvability](https://capnproto.org/language.html#evolving-your-protocol)`
  - `flatbuffers-README.md` L88: `(see [rationale](https://github.com/google/flatbuffers/wiki/Versioning))`
- **However, those README files are NOT repo-tracked.** They exist only in
  `/tmp/relayable-adr002-evidence/`; `grep -rn 'evolving-your-protocol' --include='*.md' .` finds
  nothing in the worktree, and no capnp/flatbuffers README is committed. ADR-002 itself links the
  README *URLs*, not the canonical evolution URLs, and never links `capnproto.org/language.html`,
  `flatbuffers.dev/evolution/`, the RFCs, or the Avro spec directly. So the attached doc's claim
  that "the ADR's own staged `[capnp]` README links the exact anchor" is true of the `/tmp` evidence
  bundle but **not reproducible from repo-tracked files alone**.

Also note: the ADR evidence table references many downloaded artifacts (`crates-*.json`, `proto3.md`,
`rfc8949-cbor.txt`, `rfc8446-tls13.txt`, `connect-protocol.html`, `aip-155.html`, `man-*.html`, …)
that likewise live in `/tmp/relayable-adr002-evidence/` and are not committed.

---

## 11. ADR-002 lines contradicted by fetched sources

| ADR-002 line (quoted) | Contradicted / refined by |
|---|---|
| L80: `Evolution-rules page not fetched, so evolution semantics **unverified**` | Source fetched (item 1): `https://capnproto.org/language.html#evolving-your-protocol`. Safe/unsafe rules are explicit. |
| L81: `Evolution rules **unverified** (detailed rationale is an off-repo wiki not fetched)` | Both fetched (items 2–3): wiki `Versioning` (date-based, not SemVer) and `https://flatbuffers.dev/evolution/` (add-at-end, no removal, `deprecated`, explicit `id`s, `--conform`). |
| L84: `No schema/evolution document staged → schema and evolution **unverified**` | Fetched (items 4–5): RFC 8259 has **zero** occurrences of "schema"; the gap is not "unverified" but "does not exist by design". |
| L85: `Schema-resolution rules **unverified** (no Avro spec staged)` | Fetched (item 6): Avro 1.11.1 specification §"Schema Resolution" is explicit. |
| L94: `No RFC 6455 staged → frame/close/flow-control semantics **unverified**` | Fetched (item 7): RFC 6455 settles frame/close; it has **no** "flow control" text, so "flow-control semantics" is a nonexistent target. |
| L274 / L440: `gRPC version-skew guidance is **unverified**` / `settling URL unknown` | Fetched (item 8): `grpc/grpc:doc/versioning.md` is the settling document; it mandates wire interoperability and defines no version-negotiation protocol. |

**Additional correction to the attached analysis (`result:adr002-docs#1`) item #3:** its statement
that "a new member cannot be added to an existing union" is **wrong**; RFC/spec source says adding
new fields to an existing union is safe (item 1 quote). The task-brief's own #1 claim carries the
same error.

---

## 12. Summary table

| # | Item | Status | One-line answer | URL retrieved | Retrieval date (UTC) | sha256 (short) |
|---|---|---|---|---|---|---|
| 3 | Cap'n Proto evolution | **SETTLED** | Add fields/methods (increasing ordinals) & rename safely; cannot change ordinals/types/defaults or move fields into/out of a union; **adding new fields to an existing union is safe** | https://capnproto.org/language.html | 2026-10-15T18:27:08Z | `b45ed50c1e26e924` |
| 4 | FlatBuffers versioning | **SETTLED** | Date-based `YY.MM.DD`, explicitly **not** SemVer | https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md (also wiki HTML 200) | 2026-10-15T18:27:08Z | `14da2c0c86d86867` |
| 4 | FlatBuffers evolution | **SETTLED** | Add fields at end (or explicit `id`s); never remove, mark `deprecated`; names may change; `--conform` enforces | https://flatbuffers.dev/evolution/ (guide URL 200→redirect stub; `/schema/` 200) | 2026-10-15T18:27:34Z | `2e5c610cc1b5bc5a` |
| 5 | RFC 8259 JSON | **SETTLED** | Syntax only; **zero** "schema"; no version negotiation | https://www.rfc-editor.org/rfc/rfc8259.txt | 2026-10-15T18:27:09Z | `61a5378f4255c720` |
| 5 | RFC 7493 I-JSON | **SETTLED** | Restricted interoperability profile of JSON | https://www.rfc-editor.org/rfc/rfc7493.txt | 2026-10-15T18:27:09Z | `a44ccf1451a7bf4f` |
| 6 | Avro schema resolution | **SETTLED** | Writer's schema accompanies data; name-match, ignore writer-only, reader defaults, union/enum rules, listed promotions | https://avro.apache.org/docs/1.11.1/specification/ | 2026-10-15T18:27:15Z | `f8cb0cb4475949a9` |
| 7 | RFC 6455 WebSocket | **SETTLED** | Framing + Close/Ping/Pong (≤125 B, unfragmented) + masking + status-coded close; **no application flow control** | https://www.rfc-editor.org/rfc/rfc6455.txt | 2026-10-15T18:27:16Z | `765775326aee0ecc` |
| 8 | gRPC version skew | **SETTLED** (doc located) | gRPC follows SemVer and mandates wire interoperability; **no** version-negotiation handshake | https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md | 2026-10-15T18:27:16Z | `678c7ffb0d6f9b8b` |
| 8 | grpc.io versioning guide | **BLOCKED (404)** | Guide URL does not exist | https://grpc.io/docs/guides/versioning/ | 2026-10-15T18:27:17Z | `43fe38df1bcdbd3b` |
| 10 | Android QUIC viability | **STILL-OPEN** | No fetch can settle it; needs a real Android/NDK build+run and device measurements | (none) | — | — |

Local raw files retained at `/tmp/adr002fetch/` (all `<name>` files above), plus
`/tmp/adr002fetch/rawlog.txt` and `/tmp/adr002fetch/rawlog2.txt`.

## 13. Remaining gaps / notes

- Item #8: the only authoritative gRPC versioning document found is release/API versioning. If the
  review requires a *wire* version-negotiation contract specifically, the finding is that **gRPC
  does not define one**; compatibility is delegated to mandatory wire interoperability plus protobuf
  field evolution. This is an evidence-backed negative, not an unresolved fetch.
- Item #10 remains genuinely open: it is a platform/behavioral question, not a document lookup.
- The `flatbuffers_guide_writing_schema.html` URL is now a redirect stub; cite
  `https://flatbuffers.dev/evolution/` (evolution rules) and `https://flatbuffers.dev/schema/`
  (schema grammar) instead.
- Avro spec edition: fetched 1.11.1. The pinned crate is `apache-avro` 0.22.0; the implemented spec
  edition should be confirmed from the crate's own docs before asserting a specific edition.
