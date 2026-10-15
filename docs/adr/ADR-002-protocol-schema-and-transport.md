# ADR-002 — Protocol schema and transport

**Status:** Proposed — Lead decision; pending Manager acceptance and DECISIONS.md index update (DEC-003 → ADR-002)

## Abstract

Relayable needs one versioned, schema-driven native protocol carrying control requests, event sync, terminal streaming, authentication, and admin/diagnostic operations between the Rust server, the CLI, the GPUI desktop client, and a future Kotlin/Android client (`full.md` §6.1:441: "The protocol must have explicit boundaries for control requests, event sync, terminal streaming, auth, and admin/diagnostic operations."). This ADR selects **proto3 with the `prost` toolchain**, a **length-prefixed envelope with numbered logical streams**, a **Unix domain socket** as the only first-release transport with an **off-by-default TLS 1.3 over TCP** remote mode, an **AIP-155-derived request-identity rule**, a **closed error enum with optional typed protobuf details**, and **`major.minor` version negotiation**. It also fixes the boundary that Relayable claims **no binary or wire-level compatibility with REX**.

This ADR is a sibling of `docs/architecture/interface-contract.md` (the `full.md` §14.4 contract) and `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md`. Envelope field names, error code names, and framing match the interface contract exactly; shapes owned by the interface contract or ADR-001 are deferred to, not restated.

## Context

Relayable is a persistent session platform in which the server owns sessions and PTYs independently of any viewer (`full.md` §1.3:111: "Session lifetime is independent of client lifetime."). The protocol is therefore the contract between five consumers that must agree on one meaning of every message: the Rust server, the `relayable` CLI, the GPUI Linux client, a future Kotlin/Android client, and Lua automation. `full.md` §5.2:406 defines the protocol crate's job as mapping "model operations to versioned requests/events"; it does not choose the encoding or the transport, which is this ADR's scope.

- **Schema-driven and bounded.** `full.md` §6.1:441 requires a "documented schema-driven protocol" with explicit operation boundaries; `full.md` §6.1:445–458 lists fourteen required capabilities (handshake/version negotiation, capability declaration, authn/authz, request IDs and structured errors, stable IDs and revisions, snapshot plus events, terminal I/O flow control, resize/reconnect, multi-client arbitration, heartbeats, resume fallback, incompatible-version response, redaction-safe diagnostics, malformed/oversized/truncated/unknown/stale/replayed tests).
- **No invented behavior.** `full.md` §0.3 rule 5 and `AGENTS.md` forbid inventing protocol behavior, CLI flags, or library APIs. Every external claim here cites a local evidence file retrieved 2026-10-15 (Evidence base); anything unsupported is listed under Unverified / outstanding verification.
- **No REX wire claim.** `full.md` §3:323: "Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it. Replicating documented user-visible behavior and implementing an equivalent native protocol are distinct goals." DOC-001 §2 No binary/wire-compatibility claim repeats this, and DOC-001 §8.3 Calling methods, session defaulting, errors records the transport/wire format of `rex api call` as `unknown`.
- **Transport bounds.** `full.md` §6.2:464: "**Local:** Unix domain socket with restrictive ownership/mode; no network port required." `full.md` §6.2:465: "**Remote:** authenticated encrypted native protocol. Choose a maintained TLS/QUIC/WebSocket-over-TLS/etc. approach based on requirements and practical cross-platform support. Do not roll custom cryptography."
- **Android reuse.** `full.md` §10:720 says to "share schema and compatibility tests with Rust; evaluate generated Kotlin bindings or a compatible Kotlin protocol implementation," so a schema language with mature multi-language codegen is required.
- **Interface contract already fixes shared shapes.** `docs/architecture/interface-contract.md` §3 fixes the `Envelope` field names and framing prefix, §5 fixes the terminal stream messages, and §6 fixes the error code names. This ADR chooses the encoding language, the multiplexing model, and the transports that carry those shapes; it does not redefine them.

## Evidence base

Repository sources are cited inline by section plus verified line number with a short verbatim quote. External sources use the short keys below; `full.md` and `docs/compatibility/reference-behavior-matrix.md` (DOC-001) are repository text, not external retrievals. DOC-001 is cited by section heading only, because DOC-001's Source-column line numbers are known to be unreliable (DOC-002) and must not be propagated. The original external files were retrieved with mechanical `curl` on 2026-10-15 between 16:58:04Z and 16:58:33Z; `/tmp/relayable-adr002-evidence/INDEX.md` (written 2026-10-15T16:58:52Z) records HTTP status, byte count, sha256, and UTC timestamp, but that bundle is **not** repo-tracked. REVIEW-001's independent re-verification supersedes it for Unverified items #1–#9: the local experiments and version-pinned URL retrievals, with raw commands, HTTP statuses, sha256, and UTC timestamps, now live in the repo-tracked `docs/reviews/evidence/adr-002-protocol-local-checks.md` and `docs/reviews/evidence/adr-002-url-retrieval.md`, and are cited where those items are settled. Two `protobuf.dev` URLs that would have strengthened the comparison, `https://protobuf.dev/programming-guides/best-practices/` and `https://protobuf.dev/programming-guides/api/`, returned HTTP 404 `[index]`; no substitute was fetched and no claim depends on them.

How claims are handled in this ADR:

- A **REX-derived name** may appear only if it appears verbatim in DOC-001, and the DOC-001 section is named at the point of use. Only `bad_request`, `method_not_found`, and the `REX_*` environment-variable names qualify; every other name in this ADR is marked **Relayable-defined (no REX evidence)**.
- An **external technical claim** is supported by a short evidence key whose file, URL, HTTP status, and UTC retrieval date are in the Sources table. Quotes are verbatim from that file.
- Anything not supportable from a local file is marked **unverified** and collected under Unverified / outstanding verification with the URL or command that would settle it; nothing unverified is presented as settled and no live web access was required to finish this ADR.

### Sources

| Key | Local file | Source URL | HTTP | Retrieved (UTC) | sha256 |
|---|---|---|---|---|---|
| `[index]` | `INDEX.md` | (local evidence index) | — | 2026-10-15T16:58:52Z | — |
| `[proto3]` | `proto3.md` | https://protobuf.dev/programming-guides/proto3/ | 200 | 2026-10-15T16:58:04Z | — |
| `[prost]` | `prost-README.md` | https://raw.githubusercontent.com/tokio-rs/prost/master/README.md | 200 | 2026-10-15T16:58:06Z | — |
| `[prost-repo]` | `prost-repo.json` | https://api.github.com/repos/tokio-rs/prost | 200 | 2026-10-15T16:58:07Z | — |
| `[prost-issue-439]` | `prost-issue-439.json` | https://api.github.com/repos/tokio-rs/prost/issues/439 | 200 | 2026-10-15T16:58:07Z | — |
| `[crates-prost]` | `crates-prost.json` | https://crates.io/api/v1/crates/prost | 200 | 2026-10-15T16:58:09Z | — |
| `[crates-prost-build]` | `crates-prost-build.json` | https://crates.io/api/v1/crates/prost-build | 200 | 2026-10-15T16:58:10Z | — |
| `[crates-prost-types]` | `crates-prost-types.json` | https://crates.io/api/v1/crates/prost-types | 200 | 2026-10-15T16:58:11Z | — |
| `[crates-tonic]` | `crates-tonic.json` | https://crates.io/api/v1/crates/tonic | 200 | 2026-10-15T16:58:12Z | — |
| `[crates-capnp]` | `crates-capnp.json` | https://crates.io/api/v1/crates/capnp | 200 | 2026-10-15T16:58:13Z | — |
| `[capnp]` | `capnp-rust-README.md` | https://raw.githubusercontent.com/capnproto/capnproto-rust/master/README.md | 200 | 2026-10-15T16:58:14Z | — |
| `[capnp-repo]` | `capnp-rust-repo.json` | https://api.github.com/repos/capnproto/capnproto-rust | 200 | 2026-10-15T16:58:14Z | — |
| `[crates-flatbuffers]` | `crates-flatbuffers.json` | https://crates.io/api/v1/crates/flatbuffers | 200 | 2026-10-15T16:58:15Z | — |
| `[flatbuffers]` | `flatbuffers-README.md` | https://raw.githubusercontent.com/google/flatbuffers/master/README.md | 200 | 2026-10-15T16:58:15Z | — |
| `[crates-rmp-serde]` | `crates-rmp-serde.json` | https://crates.io/api/v1/crates/rmp-serde | 200 | 2026-10-15T16:58:16Z | — |
| `[rfc8949]` | `rfc8949-cbor.txt` | https://www.rfc-editor.org/rfc/rfc8949.txt | 200 | 2026-10-15T16:58:22Z | — |
| `[crates-ciborium]` | `crates-ciborium.json` | https://crates.io/api/v1/crates/ciborium | 200 | 2026-10-15T16:58:16Z | — |
| `[rfc8610]` | `rfc8610-cddl.txt` | https://www.rfc-editor.org/rfc/rfc8610.txt | 200 | 2026-10-15T16:58:22Z | — |
| `[crates-serde_json]` | `crates-serde_json.json` | https://crates.io/api/v1/crates/serde_json | 200 | 2026-10-15T16:58:18Z | — |
| `[crates-apache-avro]` | `crates-apache-avro.json` | https://crates.io/api/v1/crates/apache-avro | 200 | 2026-10-15T16:58:18Z | — |
| `[rfc9113]` | `rfc9113-http2.txt` | https://www.rfc-editor.org/rfc/rfc9113.txt | 200 | 2026-10-15T16:58:20Z | — |
| `[rfc9000]` | `rfc9000-quic.txt` | https://www.rfc-editor.org/rfc/rfc9000.txt | 200 | 2026-10-15T16:58:21Z | — |
| `[rfc8446]` | `rfc8446-tls13.txt` | https://www.rfc-editor.org/rfc/rfc8446.txt | 200 | 2026-10-15T16:58:21Z | — |
| `[crates-quinn]` | `crates-quinn.json` | https://crates.io/api/v1/crates/quinn | 200 | 2026-10-15T16:58:24Z | — |
| `[crates-rustls]` | `crates-rustls.json` | https://crates.io/api/v1/crates/rustls | 200 | 2026-10-15T16:58:25Z | — |
| `[crates-tokio-rustls]` | `crates-tokio-rustls.json` | https://crates.io/api/v1/crates/tokio-rustls | 200 | 2026-10-15T16:58:26Z | — |
| `[crates-tokio-tungstenite]` | `crates-tokio-tungstenite.json` | https://crates.io/api/v1/crates/tokio-tungstenite | 200 | 2026-10-15T16:58:27Z | — |
| `[man-unix7]` | `man-unix7.html` | https://man7.org/linux/man-pages/man7/unix.7.html | 200 | 2026-10-15T16:58:32Z | — |
| `[man-socket7]` | `man-socket7.html` | https://man7.org/linux/man-pages/man7/socket.7.html | 200 | 2026-10-15T16:58:33Z | — |
| `[aip155]` | `aip-155.html` | https://google.aip.dev/155 | 200 | 2026-10-15T16:58:31Z | — |
| `[grpc-error]` | `grpc-error.html` | https://grpc.io/docs/guides/error/ | 200 | 2026-10-15T16:58:30Z | — |
| `[grpc-status]` | `grpc-status-codes.html` | https://grpc.io/docs/guides/status-codes/ | 200 | 2026-10-15T16:58:29Z | — |
| `[connect]` | `connect-protocol.html` | https://connectrpc.com/docs/protocol/ | 200 | 2026-10-15T16:58:30Z | — |
| `[local-checks]` | `docs/reviews/evidence/adr-002-protocol-local-checks.md` | (live local experiments on this host) | — | 2026-10-15T18:23:08Z | — |
| `[url-retrieval]` | `docs/reviews/evidence/adr-002-url-retrieval.md` | (live `curl` re-verification pass) | — | 2026-10-15T18:27:03Z | — |
| `[capnp-evolution]` | `adr-002-url-retrieval.md` §1 | https://capnproto.org/language.html | 200 | 2026-10-15T18:27:08Z | `b45ed50c1e26e92435fe74f179504f6ea5ac97e3bc705c32bd8162748508f127` |
| `[flatbuffers-versioning]` | `adr-002-url-retrieval.md` §2 | https://raw.githubusercontent.com/wiki/google/flatbuffers/Versioning.md | 200 | 2026-10-15T18:27:08Z | `14da2c0c86d868671e1b46cdf15a725e1f446d4ebf09e944549b6e86246ccbea` |
| `[flatbuffers-evolution]` | `adr-002-url-retrieval.md` §3 | https://flatbuffers.dev/evolution/ | 200 | 2026-10-15T18:27:34Z | `2e5c610cc1b5bc5a0968f150ae0cbab51c6321948f886d37ca6d74b161d7df8c` |
| `[rfc8259]` | `adr-002-url-retrieval.md` §4 | https://www.rfc-editor.org/rfc/rfc8259.txt | 200 | 2026-10-15T18:27:09Z | `61a5378f4255c720beb2a4b4a63b29540147c140f36988bf086291989b4cd2d7` |
| `[rfc7493]` | `adr-002-url-retrieval.md` §5 | https://www.rfc-editor.org/rfc/rfc7493.txt | 200 | 2026-10-15T18:27:09Z | `a44ccf1451a7bf4f4bbb78b39e10ed0e9a27ff2ee5b639c6458042b5a7197062` |
| `[avro-spec]` | `adr-002-url-retrieval.md` §6 | https://avro.apache.org/docs/1.11.1/specification/ | 200 | 2026-10-15T18:27:15Z | `f8cb0cb4475949a9545a78764ddd2049500574c57e07103c8ffdabd35dfc15b3` |
| `[rfc6455]` | `adr-002-url-retrieval.md` §7 | https://www.rfc-editor.org/rfc/rfc6455.txt | 200 | 2026-10-15T18:27:16Z | `765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b` |
| `[grpc-versioning]` | `adr-002-url-retrieval.md` §8a | https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md | 200 | 2026-10-15T18:27:16Z | `678c7ffb0d6f9b8b67695954d64f55ed84b50d1f25b29968b012d94de406d684` |

Rows added for the 2026-10-15T18:27Z re-verification pass record `sha256` exactly as fetched from `[url-retrieval]`; the pre-existing rows above predate that pass and their `sha256` values were recorded only in the non-repo-tracked `/tmp/relayable-adr002-evidence/INDEX.md`, so they are shown as `—`. `[local-checks]` and `[url-retrieval]` are the repo-tracked evidence files that settle Unverified items #1–#9.

## Options considered

"Not chosen because" is the decisive reason, not a summary of the cons. Each option was admissible only if it could satisfy `full.md` §6.1:441 and the §6.2 security rules.

### Serialization and interface definition language

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **proto3 + `prost`/`prost-build` (chosen)** | Vendor-documented field evolution `[proto3]`; idiomatic generated Rust with derives and retained comments `[prost]`; multi-language codegen for Android (`full.md` §10:720); prost not archived, `pushed_at` 2026-10-05T05:23:21Z `[prost-repo]`; versions 0.14.4 for `prost`/`prost-build`/`prost-types` (newest 2026-06-07T10:13–10:14Z) `[crates-prost]` `[crates-prost-build]` `[crates-prost-types]` | `prost-build` requires external `protoc` `[prost]`; no runtime reflection or message descriptors `[prost]`; `prost` drops unknown fields on decode, so no reserialize hop is allowed (D4; `[local-checks]` §Item#1) | Only option with vendor-documented evolution, a maintained Rust implementation, and a viable multi-language path |
| gRPC / `tonic` 0.14.6 (2026-05-07) `[crates-tonic]` | Standard service definitions and streaming; maintained | Forces HTTP/2 framing and header/trailer machinery on a protocol whose boundaries Relayable must own; local transport is a Unix socket; Connect states "Bidirectional streaming requires HTTP/2" `[connect]`; gRPC trailer details carry documented caveats `[grpc-error]` | `full.md` §6.2:464 requires a Unix-socket local transport; mandatory HTTP/2 couples our boundaries to a third party's framing |
| Cap'n Proto `capnp` 0.27.2 (repo not archived, pushed 2026-09-08) `[crates-capnp]` `[capnp-repo]` | Zero-copy reads; README advertises "protocol evolvability" and `capnp-rpc` "Object-capability remote procedure call system" `[capnp]` | Evolution rules now documented: adding fields/enumerants/methods with increasing ordinals, and renames, are safe; changing ordinals/types/defaults, changing type IDs, or moving fields into/out of an existing union is not; adding fields to an existing union is safe `[capnp-evolution]`. `capnp-rpc` is advertised, but no staged evidence shows the bounded input-arbitration semantics `full.md` §6.1:453 requires, so the advertised RPC path and the con no longer conflict. | Not chosen: proto3's vendor-documented evolution plus a generated Rust path is the better evidenced fit, and the required bounded-arbitration/RPC-security behavior is not evidenced in the staged set |
| FlatBuffers 25.12.19 `[crates-flatbuffers]` | README: "It doesn't need to be the same language/schema version, FlatBuffers ensures the data is readable across languages and schema versions." `[flatbuffers]`; zero-copy reads | Versioning is date-based `YY.MM.DD`, explicitly not SemVer `[flatbuffers-versioning]`; evolution adds fields at the end (or with explicit `id`s), never removes a field (mark it `deprecated`), names may change, and `flatc --conform` can enforce it `[flatbuffers-evolution]`; no RPC/session layer staged | Evolution is now documented, but there is no staged RPC/session layer |
| MessagePack / `rmp-serde` 1.3.1 (2025-12-23) `[crates-rmp-serde]` | Compact; trivial serde wiring | RFC 8949 Appendix E.2: MessagePack "has not yet had a transition. The evolution of MessagePack is impeded by an imperative to maintain complete backwards compatibility with existing stored data, while only few bytecodes are still available for extension… The extension mechanism for MessagePack remains unclear." `[rfc8949]` | The documented evolution posture is the opposite of what a long-lived protocol needs |
| CBOR `ciborium` 0.2.2 (last release 2024-01-24) `[crates-ciborium]` + CDDL | Self-describing IETF standard | RFC 8949 design goal: "Data must be able to be decoded without a schema description." `[rfc8949]` conflicts with schema-driven validation; RFC 8610 §3.5.4 "Cuts in Maps" shows the extensible-map idiom admits malformed data ("does match this specification") and "at the time of writing, CDDL tools cannot detect such cases automatically" `[rfc8610]` | Documented extensibility caveat plus self-describing design conflict with a strict envelope |
| JSON / `serde_json` 1.0.154 (2026-10-11) `[crates-serde_json]` | Universal tooling; readable | RFC 8259 defines syntax/formatting only and contains zero occurrences of "schema": JSON has no schema and no version-negotiation/evolution mechanism **by design** `[rfc8259]` (RFC 7493 I-JSON is only a restricted interoperability profile `[rfc7493]`); binary payloads need extra encoding; evolution is a rename problem | No schema/evolution story exists by design; JSON stays a CLI output concern (DOC-001 §7.4 JSON output) |
| Apache Avro `apache-avro` 0.22.0 `[crates-apache-avro]` | Reader/writer schema resolution is first-class | Writer schema accompanies the data; records match by (unqualified) name, writer-only fields are ignored, reader defaults are used for reader-only fields, and only the listed primitive promotions are allowed `[avro-spec]`; registry-oriented model fits ingestion more than interactive sessions | Schema resolution is now documented, but the registry-oriented fit is poor for interactive sessions |

### Framing and multiplexing

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **Length-prefixed envelope + numbered logical streams with credit flow control (chosen)** | One connection carries control, events, terminal bytes, heartbeats; matches interface contract §3; credit discipline modeled on RFC 9113 §5.2.1 `[rfc9113]`; per-stream backpressure for slow clients (`full.md` §11:769) | Relayable must define its own window policy, because RFC 9113 §5.2.1(7) says the standard "does not stipulate how a receiver decides when to send this frame or the value that it sends" `[rfc9113]` | Only option keeping all required boundaries on one bounded connection without adopting an external HTTP/2 stack |
| One connection/process per logical stream | Simple isolation; independent flow control | Many connections per session; repeated handshakes; heartbeat and session model become ambiguous | Connection fan-out undermines the single-connection session/event model |
| HTTP/2 framing via `tonic` | Standard multiplexing, flow control, RST_STREAM | Imports a large stack for a Unix-socket-first product; `[connect]` notes bidirectional streaming needs HTTP/2; we own terminal semantics anyway | Same reason as gRPC; the standard is used only as documented precedent |
| WebSocket framing `tokio-tungstenite` 0.30.0 `[crates-tokio-tungstenite]` | Widely deployed; proxy-friendlier upgrades | RFC 6455 specifies framing and Close(0x8)/Ping(0x9)/Pong(0xA) control frames (≤125 bytes, unfragmented) plus a status-coded close, but defines **no application flow control** `[rfc6455]`; double-frames over D5 | RFC 6455 supplies no application flow control, plus a redundant framing layer |
| QUIC streams `quinn` 0.11.12 `[crates-quinn]` | Native stream multiplexing and two-level flow control `[rfc9000]` | A second transport to operate; no staged client-platform support evidence | Deferred, not rejected; see D7 revisit triggers |
| Newline-delimited JSON lines | Trivial to debug; no framing library | No length bound; terminal bytes must be base64-expanded; no multiplexing | No bounded framing, no binary payloads, no multiplexing |

### Remote transport

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **TLS 1.3 over TCP carrying the identical D5 framing, off by default (chosen)** | Maintained `rustls` 0.23.46 (2026-10-14) `[crates-rustls]` and `tokio-rustls` 0.26.6 (2026-09-28) `[crates-tokio-rustls]`; TLS 1.3 needs only "a reliable, in-order data stream" `[rfc8446]` | Default crypto provider is `aws-lc-rs` (native C/assembly, no OpenSSL), not pure Rust `[local-checks]` §Item#2; no listener until secure configuration is complete (`full.md` §6.2:466) | Converges local and remote on one framing and reuses D5 unchanged |
| Plaintext TCP | Simplest; no crypto dependency | Violates `full.md` §6.2:465 and the §11 gate | Forbidden by the security requirements |
| QUIC + `quinn` | Native per-stream flow control/reset; migration; 0-RTT resume `[rfc9000]` | Second transport; no staged client-platform support evidence | Deferred, not rejected; revisit on measured lossy-link head-of-line blocking, migration/0-RTT need, or Android NAT findings |
| WebSocket over TLS | Proxy/traversal friendliness | RFC 6455 specifies framing/close but **no application flow control** `[rfc6455]`; second framing layer; HTTP/1.1 upgrade lacks per-stream flow control `[connect]` | No application flow control in RFC 6455 plus redundant framing |
| SSH tunnel only | Reuses an audited remote path | `full.md` §6.2:467: "SSH may optionally be a tunnel/bootstrap path, but don't treat SSH as the fundamental application protocol…" | Not the fundamental protocol; retained only as an optional bootstrap |

### Version negotiation

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **`protocol_version` `major.minor` in the handshake plus a structured incompatible-version error carrying the accepted range (chosen)** | Simple and testable; maps to `full.md` §6.1:445 and §6.1:456; modeled on the one fully documented mechanism, `[rfc9000]` §6/§6.1 | Minor-version discipline depends on D3 enforcement in CI | Only option directly satisfying both §6.1:445 and §6.1:456 |
| No version field; rely on `method_not_found` | Zero protocol work | No pre-operation failure; DOC-001 §8.3 defines `method_not_found` as "the server has no method by that name" (`verified-official-docs`), not a version signal | Violates `full.md` §6.1:456 and §1.3:121 ("incompatible protocol versions fail clearly") |
| Capability bitmask only | Flexible feature detection | Cannot express a breaking change | Capabilities complement but do not replace version |
| QUIC-style Version Negotiation packet over our framing | Standard precedent `[rfc9000]` | Designed for QUIC pre-state; an ordered stream handshake already carries the version | Rejected as mechanism; the RFC is cited as precedent for the accepted-range shape |

### Error model

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **Closed Relayable-defined enum + optional strongly-typed protobuf details (chosen)** | Matches interface contract §6; typed details validated by the schema pipeline (`full.md` §8.2:613); documented prior art `[grpc-error]` `[connect]` | Discipline needed to keep the enum closed and treat unknown codes safely | Only option giving a stable closed code set plus machine-readable detail without HTTP coupling |
| gRPC numeric codes with trailer details | Battle-tested semantics; `[grpc-status]` documents `UNIMPLEMENTED` and `RESOURCE_EXHAUSTED` (`verified-official-docs`) | We are not on HTTP/2; `[grpc-error]` records that "Existing proxies, loggers, and other standard HTTP request processors don't have visibility into the error details" and "Larger error detail payloads may run into protocol limits (like max headers size), effectively losing the original error" | Numeric codes and trailer details are artifacts of the HTTP/2 binding we do not adopt |
| Free-form string message only | Simplest | No stable code; untestable; violates `full.md` §8.2:613 ("stable error codes") | Fails the typed-API requirement |
| HTTP status codes only | Familiar | Borrows HTTP semantics onto a non-HTTP protocol; interface contract §6 already fixes a domain code set | Wrong layer; HTTP status is meaningless on a Unix socket |

### Request identity and idempotency

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| **Optional client-chosen `uint64` `request_id` with per-connection plus bounded-window dedup (chosen)** | Cheap on the wire; implements the AIP-155 rules `[aip155]`; matches interface contract §3 | Deviates from AIP-155's UUID recommendation, so the restriction must be documented `[aip155]`; dedup is a bounded window | Satisfies idempotency within a documented bounded scope and matches the frozen interface contract |
| UUID-string `request_id` (AIP-155 default) | Globally collision-resistant `[aip155]` | Conflicts with interface contract §3 (`uint64`); larger per-request wire form | Interface contract is frozen; D8 records the deviation |
| Server-assigned ID only | No client bookkeeping | Client cannot deduplicate a retry | Defeats the idempotency purpose |
| No request ID | Simpler | No idempotency, no retry safety, no correlation | Violates `full.md` §6.1:448 |

## Decision

D1–D14 are frozen. D14 is advisory and owned by a different record. Every numeric value is a provisional configuration parameter pending measurement; none is validated by benchmark. `full.md` §12:796: "Add regression thresholds only after a stable baseline exists."

### Decision summary

| # | Decision | Primary evidence |
|---|---|---|
| D1 | proto3 IDL; `prost`/`prost-build` codegen | `[proto3]` `[prost]` `[crates-prost]` |
| D2 | Schema files `common/control/events/terminal/auth/admin.proto`; `protocol/{examples,compatibility}` | `full.md` §6.1:441; Relayable-defined paths |
| D3 | protobuf field-number evolution discipline; `optional` explicit presence; enums start `0 = UNSPECIFIED`; CI descriptor diff | `[proto3]` |
| D4 | Do not depend on unknown-field retention; no relay/proxy that reserializes | `[proto3]` `[prost]` |
| D5 | 4-byte little-endian length prefix; `stream_id`; credit flow control modeled on HTTP/2 | interface contract §3; `[rfc9113]` |
| D6 | Local transport is a pathname Unix domain socket with mode plus peer credentials | `full.md` §6.2:464; `[man-unix7]` |
| D7 | Remote is TLS 1.3 over TCP with identical framing, off by default; 0-RTT off | `full.md` §6.2:465–469; `[rfc8446]` `[crates-rustls]` |
| D8 | Optional client-chosen `uint64` `request_id`; per-connection plus bounded dedup | `[aip155]`; interface contract §3 |
| D9 | `major.minor` version negotiation with an accepted-range error | `full.md` §6.1:445,456; `[rfc9000]` |
| D10 | Closed Relayable enum + optional typed protobuf details | interface contract §6; `[grpc-error]` `[connect]` |
| D11 | `tests/protocol/` conformance suite plus one fuzz target | `full.md` §6.1:458; §11:773 |
| D12 | No REX wire/binary compatibility claim | `full.md` §3:323; DOC-001 §2 |
| D13 | Optional `trace_id`; redaction-safe logging; no crash on malformed input | `full.md` §6.1:457; §7.4:563; §11:770 |
| D14 | Advisory: recommend `RELAYABLE_*`-only namespace | `full.md` §8.1:609; §0.1:24; DOC-001 §7.3 |

Provisional configuration parameters introduced by this ADR (none validated; all pending the `full.md` §12 benchmark plan):

| Parameter | Provisional anchor | Validation status |
|---|---|---|
| Maximum frame size | 16,384 octets (documented HTTP/2 floor `[rfc9113]`) | Provisional |
| Initial per-stream credit window | 65,535 octets (documented HTTP/2 initial window `[rfc9113]`) | Provisional |
| Concurrent stream limit | Not below the documented "no smaller than 100" guidance `[rfc9113]` | Provisional |
| Heartbeat interval and idle timeout | Unset | Provisional |
| Dedup window | Bounded; size unset | Provisional |
| Reconnect backoff bounds | Unset | Provisional |

### D1 — IDL and codegen: proto3 with `prost` + `prost-build`

Relayable uses proto3 as the IDL and the `prost` toolchain for generated Rust. Retrieval-date versions: `prost` 0.14.4, `prost-build` 0.14.4, `prost-types` 0.14.4, newest 2026-06-07T10:13–10:14Z `[crates-prost]` `[crates-prost-build]` `[crates-prost-types]` `[index]`. `tokio-rs/prost` is not archived, `pushed_at` 2026-10-05T05:23:21Z, `open_issues_count` 248 `[prost-repo]`. `prost` has a rolling MSRV; "The current MSRV is 1.85." `[prost]`, satisfied by the verified local rustc 1.98.1 (`AGENTS.md`). `prost` generates "simple, idiomatic, and readable Rust types by taking advantage of Rust `derive` attributes" and "Retains comments from `.proto` files" `[prost]`, and "Preserves unknown enum values during deserialization" `[prost]`. It "Does not include support for runtime reflection or message descriptors" `[prost]`, so `full.md` §8.2:613 discovery is served by a generated registry (server-API task), not runtime reflection. `prost-build` requires `protoc`: "With `prost-build` v0.11 release, `protoc` will be required to invoke `compile_protos` (unless `skip_protoc` is enabled). Prost will no longer provide bundled `protoc` or attempt to compile `protoc` for users." `[prost]` `protoc` is therefore a recorded build-time dependency; it is verified present on this host: plain `protoc` resolves to `/usr/bin/protoc` → `protoc-36.1.0`, version `libprotoc 36.1`, and `prost-build` 0.14.4 finds it on `PATH` with no `PROTOC`/`PROTOC_INCLUDE` or `protoc_executable(...)` configuration (`[local-checks]` §Item#9).

Rationale for choosing proto3 over the alternatives is the vendor's own documented evolution discipline (D3): field numbers are the stable identity of a field, and the "Adding new fields is safe" / "Removing fields is safe" rules give a schema that can grow without a version bump per change. The rejected alternatives are rejected for the reasons in the serialization table; the load-bearing external facts are MessagePack's documented evolution impediment `[rfc8949]`, CBOR's self-describing design goal plus CDDL's extensibility caveat `[rfc8949]` `[rfc8610]`, now-documented evolution semantics for Cap'n Proto/FlatBuffers/Avro, HTTP/2 coupling for gRPC, and JSON's by-design absence of a schema.

### D2 — Schema layout

Schema files live under `protocol/schema/`, split along the `full.md` §6.1:441 boundaries. File and directory names are **Relayable-defined (no REX evidence)**; DOC-001 §8.3 records the reference wire format as `unknown`.

| File | Contents |
|---|---|
| `protocol/schema/common.proto` | ID wrappers, `revision`/`sequence` scalars, `Envelope`, `Error`, shared types |
| `protocol/schema/control.proto` | Control requests/responses (session, window, block, client, layout) |
| `protocol/schema/events.proto` | Event envelope and payloads (subscription scope, replay/retention semantics) |
| `protocol/schema/terminal.proto` | Terminal stream messages, resize, snapshot, flow control |
| `protocol/schema/auth.proto` | Handshake, capability declaration, authentication/authorization |
| `protocol/schema/admin.proto` | Admin and diagnostic operations |

`protocol/examples/` holds example payloads; `protocol/compatibility/` holds version-skew vectors shared with Android (`full.md` §10:720); `tests/protocol/` holds the D11 suite. All are **Relayable-defined (no REX evidence)**.

Rationale: the split mirrors the operation boundaries the specification mandates rather than one monolithic file, so that an auth or admin change does not touch the control or terminal schema; and the `protocol/compatibility/` path exists so the Android client can run the same golden vectors before its UI exists (`full.md` §10:720).

### D3 — Evolution rules

Relayable adopts the protobuf field-evolution discipline as policy. From `[proto3]`, verbatim: "You must give each field in your message definition a number between `1` and `536,870,911`."; "Field numbers `19,000` to `19,999` are reserved for the Protobuf implementation."; "This number **cannot be changed once your message type is in use**."; "Field numbers **should never be reused**."; "Adding new fields is safe."; "Removing fields is safe." plus "The same field number must not used again in your updated message type."; "Changing field numbers for any existing field is not safe."; "Moving fields into an existing `oneof` is not safe."; "Changing a single explicit presence field or extension into a member of a **new** `oneof` is safe."; and, on reserved numbers, "If you update a message type by entirely deleting a field, or commenting it out, future developers can reuse the field number when making their own updates to the type… To make sure this doesn't happen, add your deleted field number to the `reserved` list."

Policy layered on top:

1. **Explicit presence for decision-bearing scalars.** Use proto3 `optional`, because `[proto3]` states: "`optional`: (recommended) An `optional` field is in one of two possible states: the field is set, and contains a value that was explicitly set or parsed from the wire. It will be serialized to the wire. / the field is unset, and will return the default value. It will not be serialized to the wire."
2. **Never decide on an implicit-presence scalar.** A default (`false`/`0`) implicit field cannot be distinguished from unset `[proto3]`.
3. **No ProtoJSON or text format on the wire.** `[proto3]`: "Some actions can cause unknown fields to be lost. For example, if you do one of the following, unknown fields are lost: Serialize a proto to JSON. Iterate over all of the fields in a message to populate a new message." JSON is a CLI presentation concern (DOC-001 §7.4 JSON output).
4. **Enums start at `0 = UNSPECIFIED`; removed enum values are `reserved`.** `[proto3]`: "unrecognized proto3 `enum` values will be preserved in the message, but how this is represented when the message is deserialized is language-dependent."
5. **Wire-compatibility check in CI.** A descriptor-diff step runs in CI over generated file descriptors emitted at build time, failing unsafe changes. The check is Relayable-defined; no specific third-party tool is named because none was verified.

Rationale: the specification's rules distinguish "Binary Wire-safe Changes" from unsafe ones, so a schema change that violates a rule must be caught before merge. That is why the CI descriptor-diff check is part of the decision and not a follow-up: with `prost` generating the descriptors, a mechanical comparison is possible without any runtime reflection, which `prost` does not provide `[prost]`.

### D4 — Unknown fields: do not depend on retention

Protobuf documents retention: "Proto3 messages preserve unknown fields and include them during parsing and in the serialized output, which matches proto2 behavior." `[proto3]`. That is the protobuf model, not a guarantee about every implementation.

**Verified behavior (settles the prior gap):** `prost` 0.14.4 **drops** unknown fields on decode. A v2 message carrying an added field (field 3) decoded by a v1-only `prost` type and re-encoded loses those bytes (24 bytes → 9 bytes; `extra=""` after round-trip); the generated `merge_field` catch-all calls `encoding::skip_field`, which advances past the bytes and retains nothing, and there is no unknown-field storage or `Message` API. The same result holds for `prost` 0.13.5, and 0.14.4 is the newest release (`[local-checks]` §Item#1). `[prost]` claims only "Preserves unknown enum values during deserialization"; it makes no unknown-field claim. The one staged issue is not about unknown fields: `prost-issue-439` is "Fix #438: Check for overflow in `Duration` and `Timestamp` processing", state `closed`, created 2021-02-14T01:03:49Z `[prost-issue-439]`. Therefore the first release does not rely on retention: no store-and-forward, proxy, or relay hop reserializes decoded messages; each side serializes only messages it constructed; readers tolerate unknown optional fields and unknown enum values by construction (no required fields, no exhaustive enum assumptions), per `full.md` §1.3:121 ("Older clients ignore unknown event fields/types where safe"), `full.md` §8.4:625 ("Ignore unknown optional fields safely."), and DOC-001 §10.6 Events on the CLI ("Ignore fields you don't recognize rather than failing on them.").

Rationale: a relay would have to decode a message it does not own and re-encode it. Because `prost` drops unknown fields, that hop would silently corrupt forward-compatible data. The behavior is now verified (`[local-checks]` §Item#1), so the design forbids the hop for the first release and D11 row 17 asserts that no client-side decode→re-encode path exists.

### D5 — Framing and multiplexing

One connection carries numbered logical streams. Every message is one `Envelope`: a **4-byte little-endian unsigned length prefix** followed by one serialized `Envelope` body. `stream_id = 0` is the control stream. The envelope fields and `oneof body { Handshake, Request, Response, Event, TerminalData, FlowControl, Heartbeat }` match `docs/architecture/interface-contract.md` §3 exactly: `protocol_version` (`major.minor` string), optional `uint64 request_id`, `uint64 stream_id`, optional `uint64 sequence`, optional `string trace_id`, optional `timestamp`, and the body `oneof`. Framing itself is **Relayable-defined (no REX evidence)**; DOC-001 §8.3 records the reference transport/wire format as `unknown`.

The design is **modeled on documented HTTP/2 flow control** as precedent, not dependency:

- **Frame size.** `[rfc9113]` §4.2: "This setting can have any value between 2^14 (16,384) and 2^24-1 (16,777,215) octets, inclusive." and "All implementations MUST be capable of receiving and minimally processing frames up to 2^14 octets in length, plus the 9-octet frame header". Relayable adopts a bounded, configurable maximum frame size provisionally anchored at 16,384 octets; oversized or truncated frames are rejected with a structured error, never a panic.
- **Credit-based flow control.** `[rfc9113]` §5.2.1: "Flow control is based on WINDOW_UPDATE frames… This is a credit-based scheme."; "Flow control is directional with overall control provided by the receiver."; "The initial value for the flow-control window is 65,535 octets for both new streams and the overall connection."; "The frame type determines whether flow control applies to a frame. Of the frames specified in this document, only DATA frames are subject to flow control; all other frame types do not consume space in the advertised flow-control window. This ensures that important control frames are not blocked by flow control." Relayable adopts the same principle: only terminal data consumes credit.
- **Our own window policy, proven deadlock-free.** `[rfc9113]` §5.2.2: "Endpoints MUST read and process HTTP/2 frames from the TCP receive buffer as soon as data is available. Failure to read promptly could lead to a deadlock when critical frames, such as WINDOW_UPDATE, are not read and acted upon." Because §5.2.1(7) states "This document does not stipulate how a receiver decides when to send this frame or the value that it sends, nor does it specify how a sender chooses to send packets," Relayable specifies its own policy and proves no deadlock under a stalled reader (D11). The initial credit window is provisional.
- **Cancellation with late-frame tolerance.** Modeled on `[rfc9113]` §6.4: "after sending the RST_STREAM, the sending endpoint MUST be prepared to receive and process additional frames sent on the stream that might have been sent by the peer prior to the arrival of the RST_STREAM". A canceled stream id tolerates late frames.
- **Settings precedent.** `[rfc9113]` §6.5.2 supplies the shape: documented initial maximum frame size 16,384; "It is recommended that this value be no smaller than 100" for concurrent streams; "An endpoint that receives a SETTINGS frame with any unknown or unsupported identifier MUST ignore that setting." Relayable exchanges an equivalent bounded settings block with provisional values.

Rationale: the local transport is a Unix socket where a single connection must still carry three very different message classes (control RPC, event stream, terminal bytes). Numbered streams plus credit-based flow control are the documented way to prevent a slow terminal reader from blocking control and heartbeat messages; framing is defined here rather than borrowed from HTTP/2 so it can run over both a Unix socket and TLS over TCP without an HTTP stack. The message classes and their credit behavior (Relayable-defined mapping to interface contract §3) are:

| Message class | Envelope body | Consumes stream credit | Notes |
|---|---|---|---|
| Control RPC | `Request` / `Response` | No | Never blocked by terminal backpressure |
| Event sync | `Event` | No | Ordered by server `sequence`; interface contract §3 |
| Terminal bytes | `TerminalData` | Yes | The only credit-consuming class |
| Flow control | `FlowControl` | No | Credit grants and window updates |
| Heartbeat | `Heartbeat` | No | Never blocked by a stalled reader (`full.md` §6.1:454) |
| Handshake | `Handshake` | No | Legal only before session state mutates |

### D6 — Local transport: Unix domain socket only

`full.md` §6.2:464: "**Local:** Unix domain socket with restrictive ownership/mode; no network port required." The socket lives under `full.md` §7.1:514 (`$XDG_RUNTIME_DIR/relayable/` or another compliant runtime path), with `/run/relayable/` reserved for a system service (`full.md` §7.1:515). The checklist requires "Local Unix socket has safe permissions" (`full.md` §13:873).

- **Directory and file mode.** `[man-unix7]`: "In the Linux implementation, pathname sockets honor the permissions of the directory they are in."; "On Linux, connecting to a stream socket object requires write permission on that socket…" plus its portability warning "Portable programs should not rely on this feature for security." Mode bits are therefore defense-in-depth, not the sole access control, consistent with `full.md` §11:762 ("Local socket: restrictive filesystem permissions, peer identity where available…").
- **No abstract-namespace socket as access control.** `[man-unix7]`: "Socket permissions have no meaning for abstract sockets: the process umask(2) has no effect when binding an abstract socket, and changing the ownership and permissions of the object (via fchown(2) and fchmod(2)) has no effect on the accessibility of the socket." Because mode bits are part of the design, the first release binds a pathname socket.
- **Peer credentials.** `[man-unix7]`: "`SO_PEERCRED` This read-only socket option returns the credentials of the peer process connected to this socket. The returned credentials are those that were in effect at the time of the call to connect(2), listen(2), or socketpair(2)." and "The use of this option is possible only for connected AF_UNIX stream sockets…". `SO_PEERCRED` is Linux-specific and **connection-scoped** (it identifies the connection's peer, not each message). The documented per-message alternative is `SO_PASSCRED`/`SCM_CREDENTIALS`: `[man-unix7]` says enabling `SO_PASSCRED` "causes receipt of the credentials of the sending process in an `SCM_CREDENTIALS` ancillary message in each subsequently received message."

Rationale: `full.md` §6.2:464 makes the local transport a Unix socket, and the portability warning means mode bits alone would be a fragile access control; peer credentials add a kernel-checked identity. The authn/authz design belongs to the `full.md` §11/§8.2 work; this ADR fixes only the transport-level boundary that peer credentials are available and connection-scoped. Threat notes that follow from the documented behavior:

- **Socket-file substitution.** A pathname socket in a writable directory could be replaced; the runtime directory is therefore user-owned and restrictive (`full.md` §11:762).
- **`SO_PEERCRED` staleness.** Credentials are those in effect at connect/listen/socketpair time `[man-unix7]`, so a long-lived connection is not re-attested per message; a design needing per-message attestation must use `SO_PASSCRED`/`SCM_CREDENTIALS`.
- **Abstract sockets excluded.** Their permissions are inert `[man-unix7]`, so they cannot carry the mode-bit part of the access-control design.
- **No network port.** The local transport opens no listener at all (`full.md` §6.2:464).

### D7 — Remote transport: TLS 1.3 over TCP, off by default

Remote mode carries the **identical D5 framing** over **TLS 1.3 over TCP**, implemented with `rustls` (`max_stable_version` 0.23.46, newest 2026-10-14T09:54:15Z) `[crates-rustls]` and `tokio-rustls` (0.26.6, 2026-09-28T09:18:10Z) `[crates-tokio-rustls]`. What TLS buys and does not buy, per `[rfc8446]`: "the only requirement from the underlying transport is a reliable, in-order data stream."; and "TLS records are typed, which allows multiple higher-level protocols to be multiplexed over the same record layer." The second clause is about TLS content types, **not** application streams; TLS therefore supplies no application flow control, no per-stream cancellation, and no application multiplexing, so D5 must carry all of that unchanged over both transports.

- **0-RTT disabled by default** unless D8 request-id dedup is enforced. `[rfc8446]` §4.2.10: "IMPORTANT NOTE: The security properties for 0-RTT data are weaker than those for other kinds of TLS data. Specifically: … 2. There are no guarantees of non-replay between connections."
- **Off by default, conservative binding.** `full.md` §6.2:466: "Bind remote listeners conservatively, off by default until secure configuration is complete." The §13 checklist requires "Remote mode cannot be accidentally exposed unauthenticated."
- **No custom cryptography** (`full.md` §6.2:465); **SSH is not the protocol** (`full.md` §6.2:467: "SSH may optionally be a tunnel/bootstrap path, but don't treat SSH as the fundamental application protocol…"); **NAT limits documented, not assumed** (`full.md` §6.2:469).
- **TLS minimum version (disclosure, not a decision change).** D7 names TLS 1.3, but `rustls` 0.23.46's default features include `tls12` (`cargo tree -e features` shows `rustls feature "tls12"`), so a default build could negotiate TLS 1.2. The position D7 implies is that **TLS 1.3 is the minimum**: the implementation disables the `tls12` feature (or otherwise configures a TLS 1.3 floor) so the wire never negotiates below TLS 1.3. This is a disclosure of what D7 already implies, not a change to D7. Evidence: `[local-checks]` §Item#2.

**Crypto provider (verified).** `rustls` 0.23.46's default feature is `aws_lc_rs`, so the default build uses `aws-lc-rs` 1.18.2 → `aws-lc-sys` 0.46.0 — native C/assembly, so the default is **not** pure Rust — and pulls **no OpenSSL** (`ring` is the non-default alternative). `aws-lc-sys` 0.46.0 contains 423 `.c` and 951 assembly files and needs a C toolchain/CMake; the build succeeded here with `/usr/bin/cc` and `/usr/bin/cmake`, no `nasm` (the `prebuilt-nasm` feature was active). Under DEC-010 (Android deferred; keep the model/protocol Android-ready), that native `aws-lc-sys` dependency is the cross-compile/Android implication to track: a future Android target needs an NDK/toolchain able to build `aws-lc-sys`, or `default-features = false` with the `ring` provider. Evidence: `[local-checks]` §Item#2.

Rationale for the rejected alternatives: plaintext TCP is forbidden by `full.md` §6.2:465; WebSocket would add a second framing layer inside D5 and RFC 6455 defines no application flow control `[rfc6455]`; and SSH-only would repeat the SSH-centric workflow the specification says not to treat as fundamental. QUIC + `quinn` (0.11.12, 2026-09-14) `[crates-quinn]` is **deferred, not rejected**: it is the strongest documented native fit (`[rfc9000]` §4.1 two-level flow control, §4.2 `MAX_STREAM_DATA`/`MAX_DATA`, §4.4 `RESET_STREAM`, §6 version negotiation), but adds a second transport with no staged client-platform evidence. Revisit on measured head-of-line blocking on a lossy mobile link, a need for connection migration or 0-RTT resume, or NAT/traversal benefits confirmed after Android testing.

### D8 — Request identity and idempotency

Relayable adopts the documented AIP-155 pattern `[aip155]`, verbatim: "The most important purpose for request IDs is to provide idempotency guarantees: allowing the same request to be issued more than once without subsequent calls having any effect."; "Providing a request ID **must** guarantee idempotency."; "If a duplicate request is detected, the server **should** return the response for the previously successful request, because the client most likely did not receive the previous response."; "APIs **may** choose any reasonable timeframe for honoring request IDs."; "The `request_id` field **must** be provided on the request message to which it applies (and it **must not** be a field on resources themselves)."; "Request IDs **should** be optional."; "The format restrictions for request IDs **must** be documented."

**Format restriction and deviation.** AIP-155 recommends UUIDs and says "Request IDs **should** be able to be UUIDs." Relayable's `request_id` is instead an **optional client-chosen `uint64`**, matching the frozen interface contract §3. This is the documented format restriction: 64-bit unsigned integers, optional, not UUID strings. AIP-155 itself permits a service to document its own restriction, so this deviation is recorded rather than left implicit. Dedup scope is **per connection for the connection lifetime plus a bounded recent window** (interface contract §3); a duplicate returns the original outcome, or the Relayable-defined `duplicate_request` error when the outcome is no longer retained. A request with no id is not idempotent and carries no retry guarantee.

Rationale: the retry-safety property is what makes reconnect and network-failure recovery safe (`full.md` §6.1:448 requires "Request IDs, structured errors, clear timeout behavior, and cancellation where applicable."). A bounded dedup window rather than permanent retention follows AIP-155's allowance that "APIs **may** choose any reasonable timeframe for honoring request IDs" and keeps memory bounded per `full.md` §12:799.

### D9 — Version negotiation

`protocol_version` is a `major.minor` string; minor changes are additive only under D3. An unsupported major yields the structured `incompatible_version` error carrying the accepted range, satisfying `full.md` §6.1:456 and `full.md` §1.3:121 ("incompatible protocol versions fail clearly"). The response shape is modeled on `[rfc9000]` §6 ("Version negotiation allows a server to indicate that it does not support the version the client used.") and §6.1 ("If the version selected by the client is not acceptable to the server, the server responds with a Version Negotiation packet… This includes a list of versions that the server will accept."). The weakest documented comparator is recorded for completeness: Connect's `Connect-Protocol-Version` header says "(In the future, it may also be used to support revisions to this protocol.)" `[connect]`; Relayable does not adopt a header because it is not HTTP-based. gRPC version-skew guidance is now located: `grpc/grpc:doc/versioning.md` states that gRPC libraries follow SemVer, that major bumps are rare, and that wire interoperability across versions is mandatory ("no backward incompatible protocol changes are allowed: old clients must continue interoperating correctly with new servers and new servers with old clients"); it defines **no** version-negotiation handshake, which is the gap D9 fills `[grpc-versioning]`.

Client kind/platform/version/capability declaration is carried in the interface contract §3 `Handshake` body (`full.md` §6.1:446). Change classification under D3, which is what makes "minor changes are additive only" mechanical:

| Change | Class | Rule |
|---|---|---|
| Add an optional field | Minor, additive | `[proto3]`: "Adding new fields is safe." |
| Remove a field and reserve its number | Minor | `[proto3]`: "Removing fields is safe."; the number must not be reused |
| Retype or renumber an existing field | Forbidden | `[proto3]`: "Changing field numbers for any existing field is not safe." |
| Move a field into an existing `oneof` | Forbidden | `[proto3]`: "Moving fields into an existing `oneof` is not safe." |
| Move an explicit-presence field into a **new** `oneof` | Safe | `[proto3]`: "Changing a single explicit presence field or extension into a member of a **new** `oneof` is safe." |
| Add an enum value | Safe | `[proto3]`: "Adding additional values to an enum is safe." |
| Add a new `Envelope` body variant | Minor, additive | New `oneof` member; existing members unchanged |
| Remove or rename a method | Major | Interface contract §6 `method_not_found`; discovery-registry change |

Rationale: a client that speaks a newer major must fail before it mutates any server state, and the error must tell it what the server accepts so a downgrade is possible without guessing.

### D10 — Error representation

An error is `Error { code, message, details, retryable }` carried in a `Response` for the matching `request_id`, exactly matching `docs/architecture/interface-contract.md` §6. `code` is a **closed Relayable-defined enum**; `details` is optional strongly-typed protobuf. The first-release set is the interface contract §6 list. Provenance of each name, required because a REX-derived name may only be used when DOC-001 documents it verbatim:

| Code | Provenance |
|---|---|
| `bad_request` | DOC-001 §8.3 Calling methods, session defaulting, errors: "`bad_request` means the arguments were wrong" (`verified-official-docs`) |
| `method_not_found` | DOC-001 §8.3: "`method_not_found` means the server has no method by that name, which can happen when the server is older than the docs you are reading" (`verified-official-docs`) |
| `incompatible_version` | **Relayable-defined (no REX evidence)** |
| `unauthenticated` | **Relayable-defined (no REX evidence)** |
| `unauthorized` | **Relayable-defined (no REX evidence)** |
| `unknown_entity` | **Relayable-defined (no REX evidence)** |
| `ambiguous_target` | **Relayable-defined (no REX evidence)** |
| `stale_sequence` | **Relayable-defined (no REX evidence)** |
| `snapshot_required` | **Relayable-defined (no REX evidence)** |
| `duplicate_request` | **Relayable-defined (no REX evidence)** |
| `input_not_owned` | **Relayable-defined (no REX evidence)** |
| `timeout` | **Relayable-defined (no REX evidence)** |
| `cancelled` | **Relayable-defined (no REX evidence)** |
| `rate_limited` | **Relayable-defined (no REX evidence)** |
| `internal` | **Relayable-defined (no REX evidence)** |

DOC-001 §8.3 marks the complete error-code set `unknown` ("only `bad_request` and `method_not_found` are named"), so every code except those two is Relayable-defined and must never be presented as a REX name. The code-plus-typed-details shape is justified by documented prior art: `[grpc-error]`: "The error model described above is the official gRPC error model… You may have noticed that it's quite limited and doesn't include the ability to communicate error details."; and `[connect]`: "Connect represents categories of errors as codes, and each code maps to a specific HTTP status code.", with details carrying "the fully-qualified Protobuf message name" plus base64 protobuf. Relayable deliberately does **not** adopt gRPC's numeric status ids or trailer details; `[grpc-error]` itself records the trailer caveats "Existing proxies, loggers, and other standard HTTP request processors don't have visibility into the error details" and "Larger error detail payloads may run into protocol limits (like max headers size), effectively losing the original error". Two semantic precedents are borrowed (verbatim) but not imported as names or ids: `[grpc-status]` `UNIMPLEMENTED` = "The operation is not implemented or is not supported/enabled in this service." and `RESOURCE_EXHAUSTED` = "Some resource has been exhausted". Timeouts/deadlines and cancellation are required by `full.md` §6.1:448 and must leave no partial mutation; the interface contract §6 defines `timeout` and `cancelled`, and timeout values are provisional.

### D11 — Conformance and test plan

The acceptance gate for the CORE-001 protocol work is the suite in `tests/protocol/`. `full.md` §6.1:458: "Tests for malformed, oversized, truncated, unknown, stale, and replayed messages."; `full.md` §11:773: "fuzz parsers/protocol boundaries where practical; test unauthorized calls and malformed payloads."; `full.md` §11:768: "validate request sizes/IDs/schema fields before acting."; `full.md` §8.2:615 requires integration tests for "malformed inputs, missing fields, unknown methods, target ambiguity, authorization failure, and backwards/forwards compatibility."

| # | Conformance test | Requirement |
|---|---|---|
| 1 | Malformed frame | `full.md` §6.1:458; no panic |
| 2 | Oversized frame | `full.md` §6.1:458; structured error at the configured bound |
| 3 | Truncated frame | `full.md` §6.1:458 |
| 4 | Unknown optional field | `full.md` §6.1:458; `full.md` §8.4:625 |
| 5 | Unknown enum value | `full.md` §6.1:458; `full.md` §1.3:121 |
| 6 | Unknown message type on a stream | `full.md` §6.1:458 |
| 7 | Stale `sequence` | `full.md` §6.1:458; yields `snapshot_required` |
| 8 | Replayed `request_id` | `full.md` §6.1:458; D8 outcome |
| 9 | Handshake-order violation | `full.md` §6.1:445–446; interface contract §4 |
| 10 | Wrong-major version | `full.md` §6.1:456; D9 |
| 11 | Capability absent (server must not send a capability the client did not declare) | `full.md` §6.1:446 |
| 12 | Flow-control overrun (credit violation) | D5; `full.md` §6.1:451; `full.md` §11:769 |
| 13 | Stalled-reader deadlock test | D5; `[rfc9113]` §5.2.2 |
| 14 | Post-cancel late-frame tolerance | D5; `[rfc9113]` §6.4 |
| 15 | Slow-client memory bound | `full.md` §11:769; `full.md` §12:799 |
| 16 | Authorization-failure paths | `full.md` §11:773; `full.md` §8.2:615 |
| 17 | No client-side decode→re-encode path | D4; `prost` drops unknown fields on decode (v2→v1 round-trip 24→9 bytes, `extra=""`), so any such path would silently corrupt forward-compatible data (`[local-checks]` §Item#1) |

`protocol/compatibility/` holds golden version-skew vectors shared with Android (`full.md` §10:720), and there is one fuzz target for the frame decoder plus the envelope parser (`full.md` §11:773). Tests assert the configured bound, not a hard-coded constant. **No protocol test suite exists yet in this repository.** The repository is Phase 0 (spec-only; `AGENTS.md`: "Repository state: **Phase 0** — spec-only repo at start"), and no `crates/` or `tests/` directory exists in this worktree, so these tests are the CORE-001 acceptance gate, not completed work. Rationale: `full.md` §0.3 rule 6 requires tests for every non-trivial feature, and this suite's existence is the condition the protocol work will be judged against. The planned fixtures and harness:

| Fixture | Location | Purpose |
|---|---|---|
| Golden version-skew vectors | `protocol/compatibility/` | Shared with Android (`full.md` §10:720) |
| Example payloads | `protocol/examples/` | Documentation and test seeds (D2) |
| Fuzz corpus | `tests/protocol/` | Frame decoder and envelope parser (`full.md` §11:773) |
| Malformed/oversized/truncated frames | `tests/protocol/` | D11 rows 1–3 (`full.md` §6.1:458) |
| Authorization-failure requests | `tests/protocol/` | D11 row 16 (`full.md` §11:773) |

### D12 — No REX wire/binary compatibility

**Relayable claims no binary or wire-level compatibility with REX.** `full.md` §3:323: "Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it. Replicating documented user-visible behavior and implementing an equivalent native protocol are distinct goals." DOC-001 §2 No binary/wire-compatibility claim: "No binary or wire-level compatibility with REX is claimed or supported by this matrix; it records documented user-visible behavior only." Relayable's schema is its own. DOC-001 §8.3 records the "Transport/wire format of `rex api call`" as `unknown`, and DOC-001 §15 Unknowns summary lists "transport/wire format" and "version negotiation/capability handshake" among server-API unknowns. No interoperability test has been run, and `full.md` §3:323 forbids the claim now. Revisiting requires (a) an actual documented REX protocol and (b) a passing interoperability test; until then the boundary is closed.

### D13 — Diagnostics and redaction

Optional `trace_id` on every envelope (`full.md` §6.1:457: "Redaction-safe diagnostics and optional tracing IDs."); `trace_id` is a diagnostics correlation value, never an identity and never used for authorization or lookup. Redaction-safe logging follows a documented rule set: `full.md` §11:770: "Logging: never emit private terminal output, tokens, keys, or full command arguments by default." A malformed request must not crash the server (`full.md` §7.4:563: "A malformed client request must not crash the server."; `full.md` §13:831: "stress malformed/oversized messages"). Bounded IPC per `full.md` §11:769: "IPC: enforce message and queue size limits; protect against slow-client memory growth." Terminal bytes, tokens, and keys are excluded from every log sink. Rationale: `full.md` §11 is a release gate, so the redaction rules are part of the protocol decision rather than deferred to a logging implementation task.

### D14 — Recommendation for DEC-006 (environment-variable namespace; advisory)

**Advisory only. Not decided here; ADR-005/DECISIONS.md owns the record.** The requirement: `full.md` §8.1:609: "Reference documentation currently describes `REX_SESSION`, `REX_BLOCK`, and `REX_SERVER` as context variables. Relayable should choose whether to use `RELAYABLE_*`, support aliases for compatibility, or set both; document the decision in an ADR and test it. Do not mix namespaces accidentally." `full.md` §0.1:24: "Do not label the product 'REX'… in the new UI, package metadata, application title, README, docs, CLI help, or marketing copy except when describing the reference product during research." DOC-001 §7.3 Environment variables documents `REX_SESSION`, `REX_BLOCK`, `REX_SERVER`, with `REX_CLIENT` only as the default for `-C` (DOC-001 §7.2 Flags and aliases).

| Option | Description | Consequences | Chosen / not chosen |
|---|---|---|---|
| **A (recommended): `RELAYABLE_*` only** | Reads and writes only `RELAYABLE_SESSION`, `RELAYABLE_BLOCK`, `RELAYABLE_SERVER`, plus `RELAYABLE_CLIENT` for the documented `-C`/`--client` default | REX-named automation scripts that read those variables will not work unmodified; a documented read-only `REX_*` fallback can be added later behind a compatibility task with tests; the decision must be tested (`full.md` §8.1:609) | Recommended: consistent with `full.md` §0.1:24 and avoids implying a REX claim `full.md` §3:323 forbids |
| B: write both namespaces | Sets `RELAYABLE_*` and `REX_*` | Two namespaces to keep coherent; silently encourages REX-named automation; conflicts with "Do not mix namespaces accidentally" (`full.md` §8.1:609) and §0.1:24 | Not recommended: mixing is what §8.1:609 warns against and it implies an unsupported compatibility posture |
| C: `REX_*` only | No Relayable namespace work | Contradicts `full.md` §0.1:24 and implies the compatibility claim `full.md` §3:323 forbids; DOC-001 §6 Target resolution records the resolution rules as `unknown` | Rejected: violates the product-name rule and the compatibility boundary |

For Option A, the concrete consequences are: REX-named automation scripts that read those variables will not work unmodified; a documented read-only `REX_*` fallback can be added later behind a compatibility task with tests, because the current `unknown` resolution rules in DOC-001 §6 make silent guessing unacceptable; and the decision must be tested, as `full.md` §8.1:609 requires. The Manager and the ADR-005 owner decide.

## Schema layout and evolution rules

The schema is split per D2 and evolves per D3. Concretely:

- Every message declares field numbers explicitly, and no field number is ever retyped; removed fields and removed enum values are added to `reserved`, and a removed field name may also be reserved so JSON/TextFormat encodings still parse (`[proto3]`).
- Every enum has `0 = UNSPECIFIED`, and unknown enum values are never interpreted as semantic state (`full.md` §1.3:121; `full.md` §8.4:625; `[proto3]`).
- Decision-bearing scalars use `optional` explicit presence; no decision is made on an implicit-presence scalar (D3 rules 1–2).
- `Envelope` is the only top-level wire unit. `protocol_version`, `request_id`, `stream_id`, `sequence`, `trace_id`, `timestamp`, and the body `oneof` are frozen per interface contract §3; adding a body variant is additive, while retyping or renumbering any envelope field is forbidden.
- The wire is binary proto3 only; ProtoJSON and text format are forbidden on the wire (D3 rule 3).
- The CI descriptor-diff step blocks unsafe changes before merge (D3 rule 5).
- No part of the schema relies on an intermediary reserializing a decoded message (D4).
- New schema files follow the D2 split along the `full.md` §6.1:441 boundaries rather than being appended to an unrelated file.

## Recommendation for DEC-006

See D14. Advisory only; ADR-005/DECISIONS.md owns the record. The single recommendation is **Option A (`RELAYABLE_*` only)**, with a tested decision and a possible later read-only `REX_*` fallback behind a compatibility task.

## Required-capability coverage (`full.md` §6.1:445–458)

| `full.md` §6.1 required capability | Line | Covered by | Note |
|---|---|---|---|
| Handshake and protocol-version negotiation | 445 | D9 | Interface contract §3 `Handshake` body |
| Client kind/platform/version/capability declaration | 446 | D9 | Interface contract §3 `Handshake` |
| Authentication and per-principal authorization | 447 | D2 (message surface) + D6 (transport boundary); mechanism deferred | `docs/architecture/interface-contract.md` §7 defers "Authentication/authorization mechanisms and `principal` semantics" to the §11/§8.2 work; D2 fixes only the `auth.proto` message surface and D6 only the Unix-socket peer-credential transport boundary |
| Request IDs, structured errors, clear timeouts, cancellation | 448 | D8, D10 | Interface contract §3/§6 |
| Stable entity IDs and monotonically increasing revisions | 449 | Interface contract §1/§2 (ADR-001) | Not a protocol-shape decision |
| Snapshot request plus incremental event stream | 450 | Interface contract §2/§3 (ADR-001) | `events.proto` per D2 |
| Terminal I/O with bounded flow control and backpressure | 451 | D5, interface contract §5 | |
| Terminal resize, dimensions, capabilities, reconnect | 452 | Interface contract §5 | `terminal.proto` per D2 |
| Multiple clients with documented input arbitration | 453 | Interface contract §5 | Input-lease semantics are interface-contract-defined |
| Heartbeats/timeouts marking a client disconnected without killing the session | 454 | Interface contract §3/§4 | Envelope `Heartbeat` body per D5 |
| Resume cursor/full-snapshot fallback | 455 | Interface contract §2 (ADR-001) | `snapshot_required` in interface contract §6 |
| Clear response for incompatible client/server versions | 456 | D9 | `incompatible_version` in interface contract §6 |
| Redaction-safe diagnostics and optional tracing IDs | 457 | D13 | Envelope `trace_id` per D5 |
| Tests for malformed/oversized/truncated/unknown/stale/replayed messages | 458 | D11 | `tests/protocol/` suite |

Every `full.md` §6.1 capability has a message surface or transport boundary assigned by D1–D14, the interface contract, or ADR-001, but **not every mechanism is decided here**: the authentication/authorization mechanism and `principal` semantics are explicitly deferred by `docs/architecture/interface-contract.md` §7 ("Authentication/authorization mechanisms and `principal` semantics"), so row 447 assigns only the `auth.proto` surface (D2) and the peer-credential boundary (D6), not the mechanism.

## Consequences

- **Build-time dependency: `protoc` (verified present).** `prost-build` requires it `[prost]`, and `protoc` is not yet listed in the verified local environment table in `AGENTS.md`. The build must not silently install it (`full.md` §0.3:10–11); a missing `protoc` is reported as a missing prerequisite. It **is** present on this host: `/usr/bin/protoc` → `protoc-36.1.0`, version `libprotoc 36.1`, found by `prost-build` 0.14.4 on `PATH` with no extra configuration (`[local-checks]` §Item#9).
- **No runtime reflection.** `prost` has no runtime reflection or descriptors `[prost]`, so discovery is a generated registry (server-API task), not reflection.
- **A conversion layer sits between `relayable-protocol` wire types and `relayable-model` domain types** (ADR-001); the direction `protocol → model` is unchanged.
- **All limits are provisional** (max frame size, initial credit window, concurrent streams, heartbeat interval, idle timeout, scrollback bytes, dedup window, backoff bounds); `full.md` §12:796 forbids hard thresholds before a measured baseline, supplied by the §12 benchmark plan.
- **Two transports share one framing**; a transport switch does not change message semantics, error codes, or flow-control behavior.
- **One protocol surface for all clients**; `protocol/compatibility/` vectors let Android be validated before its UI is complete.
- **A CI descriptor-diff check becomes a release gate.**
- **0-RTT is off by default**; enabling it requires the D8 idempotency guarantee and is a tested configuration change.
- **No relay/proxy in the first release** (D4).
- **DEC-006 remains open**; D14 only recommends.

## Reconsider if

- Measurement shows the D5 credit model or frame-size bound cannot meet the `full.md` §12 benchmark plan (§12:781), or the stalled-reader test reveals an unfixable deadlock.
- The descriptor-diff check cannot be implemented with available tooling, or prost's descriptors prove insufficient.
- A relay, proxy, or store-and-forward feature becomes required; that reopens D4 and requires verified prost unknown-field behavior.
- ADR-003 selects a terminal engine whose state model cannot feed the interface contract §5 terminal snapshot, forcing a `terminal.proto` change (route through the Lead per `full.md` §14.4:971).
- A documented REX protocol appears **and** an interoperability test is in scope, reopening D12.
- A lossy-mobile measurement, migration need, or Android NAT/traversal finding justifies QUIC, reopening D7.
- `protoc`-based generation proves unworkable in target build environments, or Android codegen cannot consume proto3 without a large FFI surface, reopening D1 (`full.md` §10:720).

## Unverified / outstanding verification

REVIEW-001 (B-1) re-verified every row. Items #1–#9 are now settled (local experiment or live retrieval, all repo-tracked in `docs/reviews/evidence/**`); only #10 remains open. The table keeps its original three-column structure.

| # | Verification item | Status and evidence |
|---|---|---|
| 1 | Whether `prost` retains **unknown fields** (not just unknown enum values) | **SETTLED — `prost` DROPS unknown fields on decode.** v2→v1 round-trip 24→9 bytes, `extra=""`; the generated catch-all calls `encoding::skip_field` and there is no unknown-field storage; holds for 0.14.4 and 0.13.5, and 0.14.4 is newest. Evidence: `[local-checks]` §Item#1. D4; D11 row 17. |
| 2 | Whether `rustls` is "pure Rust"/avoids OpenSSL, and its default crypto provider (`ring` vs `aws-lc-rs`) | **SETTLED — default provider is `aws-lc-rs` 1.18.2 → `aws-lc-sys` 0.46.0 (native C/assembly, not pure Rust); no OpenSSL in the default tree** (`ring` is the non-default alternative). Evidence: `[local-checks]` §Item#2. See D7. |
| 3 | Cap'n Proto evolution rules | **SETTLED** — adding fields/enumerants/methods with increasing ordinals and renames are safe; changing ordinals/types/defaults/type IDs and moving fields into/out of an existing union are not; adding fields to an existing union is safe. `[capnp-evolution]` (https://capnproto.org/language.html, 2026-10-15T18:27:08Z). |
| 4 | FlatBuffers schema-evolution/versioning rules | **SETTLED** — release versioning is date-based `YY.MM.DD`, not SemVer `[flatbuffers-versioning]`; fields are added at the end (or with explicit `id`s), never removed (mark `deprecated`), names may change, and `flatc --conform` enforces it `[flatbuffers-evolution]`. |
| 5 | JSON (`serde_json`) schema and evolution guidance | **SETTLED — none by design.** RFC 8259 defines syntax only and has zero occurrences of "schema"; RFC 7493 I-JSON adds only an interoperability profile. `[rfc8259]` `[rfc7493]`. |
| 6 | Apache Avro reader/writer schema-resolution rules | **SETTLED** — the writer schema accompanies the data; records match by (unqualified) name, writer-only fields are ignored, reader defaults are used, and only the listed primitive promotions are allowed. `[avro-spec]` (https://avro.apache.org/docs/1.11.1/specification/, 2026-10-15T18:27:15Z). |
| 7 | WebSocket (RFC 6455) framing, close, and control semantics | **SETTLED** — framing plus Close(0x8)/Ping(0x9)/Pong(0xA) control frames (≤125 bytes, unfragmented), client→server masking, and a status-coded close; RFC 6455 defines **no application flow control**. `[rfc6455]` (https://www.rfc-editor.org/rfc/rfc6455.txt, 2026-10-15T18:27:16Z). |
| 8 | gRPC version-skew guidance | **SETTLED (document located)** — `grpc/grpc:doc/versioning.md` (https://raw.githubusercontent.com/grpc/grpc/master/doc/versioning.md, 2026-10-15T18:27:16Z) mandates wire interoperability across versions and defines **no** version-negotiation handshake. `[grpc-versioning]`. |
| 9 | Presence and version of `protoc` on the build host | **SETTLED — present.** `/usr/bin/protoc` → `protoc-36.1.0`, `libprotoc 36.1`; `prost-build` 0.14.4 finds it on `PATH` with no extra config. Evidence: `[local-checks]` §Item#9. |
| 10 | Whether client-platform QUIC support is viable for Android (D7 deferral) | **STILL-OPEN.** A document lookup cannot settle this; a real Android/NDK build and run of the chosen QUIC stack plus on-device NAT/migration/0-RTT measurements are required. Unchanged. |

## Open questions

- **`request_id` format deviation from AIP-155.** D8 documents a `uint64` format where AIP-155 recommends UUIDs; re-examine if the interface contract is reopened.
- **Dedup window size.** The bounded dedup window is a configuration parameter pending a measurement or compatibility need.
- **Flow-control window defaults and update policy.** The initial credit window and the window-update algorithm are unspecified until the stalled-reader test and benchmark plan exist.
- **Max frame size default.** Provisional anchor is the documented 16,384-octet floor `[rfc9113]`; the adopted default is a configuration parameter.
- **Concrete CI descriptor-diff tooling.** D3 names the check but no specific tool; selection is an implementation-time verification step.
- **Method-name inventory.** This ADR intentionally names no protocol method; the `full.md` §8.2:613 discovery surface and the first-release method list are owned by the server-API task, bounded by DOC-001 §8.1 Discovery and version coupling.
- **Terminal flow-control interaction with resume.** How credit state behaves across reattach and `TerminalSnapshot` is owned by the interface contract §5 but not yet specified in detail.
- **Reference gaps carried forward.** DOC-001 §8.3 marks the transport/wire format, version negotiation, authentication, and complete error-code set `unknown`; every non-DOC-001 name here is Relayable-defined.

## References

- `full.md` §0.1:24; §0.3:5,10–11; §1.3:111,121; §3:323; §4:329,343; §5.2:405–412; §6.1:441,445–458,460; §6.2:464–469; §6.3:473–480,486; §6.4:490,500; §7.1:510–515; §7.4:563; §8.1:605–609; §8.2:613,615; §8.4:625; §8.5:631; §10:720; §11:762,767–773; §12:781,796,799,804; §13:824,831,873; §14.4:962,971.
- `docs/compatibility/reference-behavior-matrix.md` (DOC-001) §2 No binary/wire-compatibility claim; §3 Label legend; §6 Target resolution; §7.2 Flags and aliases; §7.3 Environment variables; §7.4 JSON output; §8.1 Discovery and version coupling; §8.2 Schemas and examples; §8.3 Calling methods, session defaulting, errors; §10.6 Events on the CLI; §15 Unknowns summary.
- `docs/architecture/interface-contract.md` §3 Request/event envelope; §4 Connection state model; §5 Terminal stream interface; §6 Error representation; §7 Crate ownership and dependency direction.
- `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md` (snapshot/revision model, crate layout, allowed/forbidden edges).
- External evidence: the Sources table in Evidence base.
