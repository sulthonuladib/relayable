# Relayable interface contract

Status: frozen interface note required by `full.md` §14.4:962 "Before parallel coding, the lead writes an interface note or
ADR containing:". It must exist before parallel coding because `full.md` §14.3:944 warns "Don't parallelize tightly coupled
implementation until API contracts and file ownership are explicit." This is a contract, not an implementation: field lists
are illustrative shapes, not a serialization schema. Sibling decision records (written in parallel):
`docs/adr/ADR-001-domain-model-and-dependency-boundaries.md`, `docs/adr/ADR-002-protocol-schema-and-transport.md`.

Evidence labels follow DOC-001 ("Relayable — REX reference behavior matrix"). Names from the reference docs cite the DOC-001
section that documents them; everything else is **Relayable-defined (no REX evidence)**. No REX binary or wire compatibility
is claimed (`full.md` §3:323 "Do not claim binary or wire-level compatibility with the original REX implementation unless the
actual protocol is documented and interoperability tests prove it."; DOC-001 §2 "No binary/wire-compatibility claim").

## 1. Entity types and IDs

`full.md` §4:331–341 defines eleven domain entities. Each is owned by `relayable-model`, has one identity rule, and is
server-authoritative unless noted.

| Entity | Identifies it | Scope | Authority |
| --- | --- | --- | --- |
| Server | `server:` ID | service instance | server |
| HostProfile | `host:` ID (label + endpoints) | client-local | client |
| Session | `session:` ID | server session group | server |
| Window | `window:` ID | within a session | server |
| Block | `block:` ID | within a window | server |
| Client | `client:` ID | attached client record | server |
| PtyProcess | `pty:` ID | server-owned PTY/process tree | server |
| Layout | owning `window:` ID (no separate ID) | within a window | server (shared layout) |
| Event | `(scope_id, server_sequence)` | a scope | server |
| ProgramStatus | owning `block:` ID + status `id` | within a block | server, parsed from terminal |
| AutomationScript | `script:` ID | server script registry | server |

The only first-release block kind is `terminal` (`full.md` §4:335 "First implementation is a terminal; do not build future
block plugins prematurely."). Layout has no independent ID; `full.md` §4:338 "Separate shared layout state from local client
UI preferences where the reference behavior implies that distinction." Shared layout is server-authoritative, client UI
preferences are client-local and never part of a snapshot. An event is not ID-addressable: it is identified by `(scope_id,
server_sequence)`.

Text form is `<kind>:<uuid>` with lowercase singular kinds. DOC-001 §5 "Domain hierarchy and stable IDs" documents
`session:<uuid>` and `block:<uuid>` (verified-official-docs) with examples `session:01a10cf3-2655-7e48-9b9c-43dd6612bef5` and
`block:01a10d99-0c06-7795-bf67-798ce4c36a33`, both carrying UUID version nibble `7`. The prefixes `server`, `host`, `window`,
`client`, `pty`, `script`, and adopting UUIDv7 per RFC 9562 as the wire layout, are **Relayable-defined (no REX evidence)**;
DOC-001 §5 records "ID scheme beyond the `type:` prefix and the never-reused rule" as unknown.

Rules: IDs are opaque, stable, and never reused (`full.md` §4:329 "Use stable, collision-resistant IDs that are never
reused."); labels are mutable and never identity (`full.md` §4:329 "Labels may change and must not serve as permanent
identities."; DOC-001 §5 "Domain hierarchy and stable IDs" records labels as changeable and non-unique); deletion permanence
and whether a session close retains audit history are undecided (DOC-001 §5 "Deletion permanence and whether a session close
leaves audit history" is unknown) and the contract only requires the decision stay expressible; `host:` IDs are client-local
and never accepted as a server identity, with server targeting resolving to a `server:` ID (DOC-001 §6 "Target resolution").
Rust uses one distinct newtype per kind in `relayable-model` (`SessionId`, `WindowId`, `BlockId`, `ClientId`, `PtyId`,
`ServerId`, `HostProfileId`, `ScriptId`), and no bare `String` ID crosses a crate boundary. This does both halves of `full.md`
§4:343 "Use tagged IDs or explicit entity type fields to prevent cross-entity ambiguity."

**Target resolution** belongs to `relayable-cli` and the docs layer, never to `relayable-model`. It follows the DOC-001 §6
"Target resolution" first-match order: (1) full ID; (2) case-insensitive label; (3) 1-based position; (4) ID suffix of at
least four characters. Ambiguity is an error (DOC-001 §6 "Ambiguous target: stop, list matches, exit 3"; DOC-001 §7.5 "Exit
codes"): stop without mutation and exit with the ambiguous-target status. The contract must make every step expressible and
must never guess among ambiguous targets; the `ambiguous_target` error in §6 carries this over the API.

## 2. Session snapshot schema

A snapshot is a consistent projection of the domain entities at one server `sequence`. Clients synchronize by taking a
snapshot, then applying events. Per-entity `revision: u64`, starting at 1, is the optimistic-concurrency token for that
entity: a mutation names the revision it expects and a mismatch is a conflict. Server-wide monotonic `sequence: u64` is total
event order and the resync cursor (`full.md` §6.1:449 "Stable entity IDs and monotonically increasing revisions where state
changes."; `full.md` §6.1:450 "Snapshot request plus incremental event stream."). `revision`, `session_id`, `block_id`, and
`window_id` are documented field names (DOC-001 §8.2 "Schemas and examples"; DOC-001 §10 "Events").

| Snapshot field | Shape |
| --- | --- |
| `sequence` | u64, server-wide |
| `server` | descriptor: version, capabilities, limits |
| `session` | `session_id`, `label`, `revision` |
| `windows` | `window_id`, `label`, `revision`, active flag |
| `blocks` | `block_id`, `window_id`, kind (`terminal`), `label`, `revision`, cols/rows, program status |
| `layout` | per window: tree of `block_id` + split ratios |
| `clients` | `client_id`, kind, platform, version, `connection_state`, `principal`, `connected_at` |
| `status` | program-status records, keyed by `block_id` + status `id` |
| `automation_scripts` | present only when scripts exist |

Payload field names DOC-001 documents for the same datum are reused verbatim: `session_id`, `block_id`, `window_id`,
`revision`, `client_id`, `principal`, `connection_state`, `connected_at`, `exit_code`, `pwd`, `title`, `state`, `progress`,
`text`, `id`, `ids`, `record`, `generation`, `foreground` — documented across DOC-001 §10 "Events", §11 "Program Status
Protocol (OSC 7501)", §12 "Multi-client behavior", and §13 "Connection lifecycle". This reuse keeps snapshot fields and event
payloads aligned.

Resume is `(scope, last_sequence_applied)`. If the cursor predates the retained event floor, the answer is a full snapshot
with a `snapshot_required` reason, never a false promise of replay (`full.md` §6.1:455 "Resume cursor/token semantics where
safe; full snapshot fallback when events/output cannot be replayed."). A terminal screen snapshot and scrollback recovery are
separate capabilities, and byte-exact terminal stream recovery is not guaranteed (`full.md` §6.1:460 "Don't claim exact
recovery of terminal byte streams unless the server guarantees it. A terminal screen snapshot and scrollback recovery are
different capabilities."). No crash or host-reboot session survival is claimed (`full.md` §6.3:480 "Do not promise survival
after server crash or host reboot unless one of these is implemented and tested"; `full.md` §6.3:484 "or another proven
OS-level arrangement."): a restored session definition is a restart, not a resurrected process. Scrollback and outgoing queues
are bounded (`full.md` §6.4:500 "Bound per-terminal scrollback and outgoing queues. Handle high-volume terminal output without
unbounded memory.").

## 3. Request/event envelope

**Framing.** Every message is one `Envelope`: a 4-byte little-endian unsigned length prefix followed by one serialized
`Envelope` body. Logical streams are multiplexed on one connection via `stream_id`, where `stream_id = 0` is the control
stream. The same framing is used over the local Unix socket and over remote TLS (`full.md` §6.2:464 "**Local:** Unix domain
socket with restrictive ownership/mode; no network port required."; `full.md` §6.2:465 "**Remote:** authenticated encrypted
native protocol."). Framing is **Relayable-defined (no REX evidence)**; DOC-001 §8.3 "Calling methods, session defaulting,
errors" records the transport/wire format as unknown.

| Envelope field | Type | Rule |
| --- | --- | --- |
| `protocol_version` | string | `major.minor` |
| `request_id` | optional uint64 | correlates `Request` and `Response` |
| `stream_id` | uint64 | `0` = control stream |
| `sequence` | optional uint64 | server-assigned |
| `trace_id` | optional string | diagnostics only, never identity |
| `timestamp` | optional | informational only |
| `body` | `oneof` | `Handshake`, `Request`, `Response`, `Event`, `TerminalData`, `FlowControl`, `Heartbeat` |

**Evolution.** Proto3-style field numbers are never reused or retyped; removed fields and enum values are reserved; every enum
defines `0 = UNSPECIFIED`; clients ignore unknown optional fields and unknown enum values safely (`full.md` §8.4:625 "Ignore
unknown optional fields safely."; `full.md` §1.3:121 "Older clients ignore unknown event fields/types where safe; incompatible
protocol versions fail clearly."). The operation set is a typed `oneof` inside `Request`; each operation carries a stable
`method` name string for diagnostics and discovery (`full.md` §8.2:613 "Build a typed API with request/response schemas and
stable error codes."). This contract fixes the shape and does not invent method names; the first-release method list is
decided by the server-API task.

**Versioning.** `protocol_version` is `major.minor`; minor changes are additive only. A client whose major is unsupported
receives the structured `incompatible_version` error with the supported range (`full.md` §6.1:445 "Handshake and
protocol-version negotiation."; `full.md` §6.1:456 "Clear response for incompatible client/server versions.").

**Handshake.** Client sends `Handshake{client_kind, platform, client_version, protocol_version, capabilities[]}` (`full.md`
§6.1:446 "Client kind/platform/version/capability declaration."). Server replies with its version, capabilities, and `limits`
(max frame bytes, max concurrent streams, credit window bytes, heartbeat interval, idle timeout), or the
`incompatible_version` error. No session state mutates before the handshake completes. All limit values are provisional
configuration parameters pending measurement (`full.md` §12:781 "Performance is based on measurements, not guesses or
marketing targets.").

**Heartbeats.** Heartbeats are bidirectional. A missed idle deadline marks the client disconnected and emits the DOC-001 §10
"Events" documented client-disconnect event family (`client_connection_changed`, `client_disconnected`) without destroying the
session (`full.md` §6.1:454 "Heartbeats/timeouts that mark a client disconnected without killing the session."; `full.md`
§6.3:475–476 closing a client window or losing the network does not destroy the server-owned session).

**Replay, duplicates, diagnostics.** Deduplicate on `(connection_id, request_id)` for the connection lifetime plus a bounded
recent window; a duplicate returns the original outcome, or `duplicate_request` when that outcome is no longer retained; a
stale `sequence` yields `snapshot_required`, not a best-effort patch (`full.md` §6.1:455; `full.md` §6.1:458 "Tests for
malformed, oversized, truncated, unknown, stale, and replayed messages."). `trace_id` is optional on every envelope; terminal
bytes, tokens, and keys must never be logged (`full.md` §11:770 "Logging: never emit private terminal output, tokens, keys, or
full command arguments by default."); a malformed request must not crash the server (`full.md` §7.4:563 "A malformed client
request must not crash the server.").

## 4. Connection state model

State names below are **Relayable-defined (no REX evidence)**. DOC-001 §13 "Connection lifecycle" confirms that a
`connection_state` field exists and that `client_connection_changed` carries it, but records the allowed values as unknown
(also DOC-001 §10 "Events", §15 "Unknowns summary"). The field name `connection_state` is DOC-001-documented and is reused
verbatim.

| State | Entered by | Guaranteed; legal requests | Transitions out |
| --- | --- | --- | --- |
| `Disconnected` | initial; transport drop | no session contact; connect only | → `TransportConnected` |
| `TransportConnected` | transport opens | bytes flow, no semantics; `Handshake` only | → `Negotiated`; → `Failed` |
| `Negotiated` | handshake reply | version + capabilities agree; auth, handshake completion | → `Authenticated`; → `Failed` |
| `Authenticated` | auth accepted | identity known, no state; snapshot request | → `Synchronizing`; → `Failed` |
| `Synchronizing` | snapshot requested | snapshot in flight; heartbeat, cancel | → `Ready`; → `Failed` |
| `Ready` | snapshot applied | live state usable; all legal operations | → `Resynchronizing`; → `Closing`; → `Disconnected` |
| `Resynchronizing` | stale sequence / reconnect | fallback to full snapshot; heartbeat, resync | → `Ready`; → `Disconnected` |
| `Closing` | client shutdown | no new requests; drain only | → `Closed`; → `Disconnected` |
| `Closed` | transport closed | terminal; none | none |
| `Failed(reason)` | terminal error | reason recorded; none | none |

A transport drop anywhere moves to `Disconnected`; only handshake requests are legal before `Negotiated`; while
`Resynchronizing`, only heartbeat and resync requests are legal; a client in `Failed` never silently returns to `Ready` —
recovery is a fresh connect.

Reconnect: a transport drop moves to `Disconnected` and never destroys the server-owned session (`full.md` §6.3:473
"First-release guarantees:"; `full.md` §6.3:475–476); reattach requests a fresh snapshot (`full.md` §6.3:477 "Reopening the
client requests a current snapshot and can reattach."). Backoff is bounded; specific values are parameters, not invented
constants. The server holds a client record using the DOC-001-documented field name `connection_state` (DOC-001 §12
"Multi-client behavior", §13 "Connection lifecycle") with the Relayable-defined state values above published on
`client_connection_changed`; the snapshot exposes `client_id`, `principal`, and `connected_at` (DOC-001 §12 "Multi-client
behavior").

## 5. Terminal stream interface

Per terminal block there is one logical stream identified by `stream_id` (distinct from the control stream `stream_id = 0`).
Message names below are **Relayable-defined (no REX evidence)**; the required capabilities are from `full.md` §6.1.

| Direction | Message | Fields | Rules |
| --- | --- | --- | --- |
| server → client | `TerminalOutput` | `stream_id`, `stream_seq`, `bytes` | ordered per stream; `stream_seq` monotonic per stream |
| client → server | `TerminalInput` | `stream_id`, `bytes` | legal only for the input-lease holder |
| client → server | `TerminalResize` | `stream_id`, `cols`, `rows`, `pixel_width?`, `pixel_height?` | `pixel_*` optional |
| server → client | `TerminalSnapshot` | `stream_id`, `snapshot_seq`, `cols`, `rows`, `screen_bytes`, `scrollback_bytes` | recovery, not output replay |

`TerminalOutput` is not byte-exact-replayable across a detach (`full.md` §6.1:460). Resize follows `full.md` §6.1:452
"Terminal resize, dimensions, relevant capabilities, and client reconnect behavior." and is verified by `stty size` (`full.md`
§13:824 "resize and verify `stty size` or equivalent;"). Reattach returns `TerminalSnapshot`: a screen plus bounded scrollback
recovery, distinct from output replay; retained is bounded scrollback, lost is output evicted from the bounded queue
(`full.md` §6.4:500). Flow control is credit-based per stream in bytes: the client grants credits, and when credits are
exhausted the server pauses reads and buffers within a bounded budget, then escalates to detach-with-reason instead of
unbounded growth (`full.md` §6.1:451 "Terminal input/output with bounded flow control and backpressure."; `full.md` §11:769
"IPC: enforce message and queue size limits; protect against slow-client memory growth."; `full.md` §12:799 "bounded
scrollback and queues;").

**Multi-client input arbitration.** Attaching declares a mode of `writer` or `observer`; exactly one writer lease exists per
block; a second `writer` must explicitly and authorizedly take the lease and the previous holder is notified through an event;
input from a non-holder is rejected with `input_not_owned` and is never silently dropped. This satisfies `full.md` §6.1:453
"Multiple clients attached to the same session with documented input arbitration." and `full.md` §11:767 "Input: all attached
clients share a terminal input stream only according to documented semantics; display connected clients and avoid surprising
simultaneous writes." DOC-001 §12 "Multi-client behavior" records the reference arbitration rules as unknown, so the
mode/lease semantics and the `writer`/`observer` names are **Relayable-defined (no REX evidence)**.

SSH is at most a tunnel/bootstrap path, not the application protocol (`full.md` §6.2:467 "SSH may optionally be a
tunnel/bootstrap path, but don't treat SSH as the fundamental application protocol if remote session functionality is intended
to replace an SSH-centric workflow."). OSC 7501 is a terminal-level integration, not the native wire protocol (`full.md`
§8.5:631 "Treat OSC 7501 as a terminal-level integration, not the native Relayable wire protocol.").

## 6. Error representation

An error is `Error{code: enum, message: string, details, retryable: bool}`, carried in a `Response` for the matching
`request_id`. Errors are structured only: never a silent failure and never a panic on malformed input (`full.md` §7.4:563).

| Code | Source | Condition |
| --- | --- | --- |
| `bad_request` | DOC-001 §8.3 "Calling methods, session defaulting, errors" | the arguments were wrong |
| `method_not_found` | DOC-001 §8.3 "Calling methods, session defaulting, errors" | server older than the docs you are reading |
| `incompatible_version` | **Relayable-defined (no REX evidence)** | unsupported protocol major |
| `unauthenticated` | **Relayable-defined (no REX evidence)** | no valid principal |
| `unauthorized` | **Relayable-defined (no REX evidence)** | principal lacks permission |
| `unknown_entity` | **Relayable-defined (no REX evidence)** | ID resolves to nothing |
| `ambiguous_target` | **Relayable-defined (no REX evidence)** | target matches more than one entity |
| `stale_sequence` | **Relayable-defined (no REX evidence)** | cursor predates retained events |
| `snapshot_required` | **Relayable-defined (no REX evidence)** | replay impossible; full snapshot required |
| `duplicate_request` | **Relayable-defined (no REX evidence)** | `request_id` already resolved |
| `input_not_owned` | **Relayable-defined (no REX evidence)** | input from a non-writer client |
| `timeout` | **Relayable-defined (no REX evidence)** | declared deadline elapsed |
| `cancelled` | **Relayable-defined (no REX evidence)** | client cancelled the request |
| `rate_limited` | **Relayable-defined (no REX evidence)** | caller exceeded a policy limit |
| `internal` | **Relayable-defined (no REX evidence)** | unexpected server fault |

`bad_request` and `method_not_found` are reused deliberately for exactly the two conditions above. DOC-001 §8.3 "Calling
methods, session defaulting, errors" marks the complete error-code set unknown ("Complete error-code set" is unknown; only
`bad_request` and `method_not_found` are named), so no other name above may be read as a REX name. A request may declare a
deadline and be cancelled by the client; cancellation is idempotent and must not leave partial mutation (`full.md` §6.1:448
"Request IDs, structured errors, clear timeout behavior, and cancellation where applicable.").

## 7. Crate ownership and dependency direction

| Crate | Owns | Constraint |
| --- | --- | --- |
| `relayable-model` | entities, IDs, revisions, snapshots | pure domain: no transport, no async runtime, no serialization-framework derives |
| `relayable-protocol` | schema, wire messages, codec, version negotiation | owns conversions to/from model |
| `relayable-client` | connection lifecycle, RPC, resync, event sequencing | no server internals |
| `relayable-server` | API, authorization, lifecycle, persistence, event dispatch | composes model, protocol, pty |
| `relayable-pty` | OS boundary | opaque handles only |
| `relayable-cli` | the `relayable` binary | independent API client |

`full.md` §5.2:405 "Model types must not depend on GPUI, Android, OS views, or transport." Deferred: `relayable-terminal` /
`relayable-status` (boundary pending the terminal-engine ADR-003 decision; first-release code lives in `relayable-server`
modules) and `relayable-automation` (after the typed API stabilizes). Directory ownership:
`protocol/{schema,examples,compatibility}` (schema + compatibility fixtures),
`tests/{protocol,lifecycle,reconnect,multi-client,terminal-compat,fixtures}` (contract tests per area), `apps/linux/`, and
`apps/android/` (independent Gradle project, never required for default `cargo test`).

| Edge | Allowed |
| --- | --- |
| `protocol → model` | yes |
| `client → {model, protocol}` | yes |
| `server → {model, protocol, pty}` | yes |
| `cli → {model, protocol, client}` | yes |
| `pty →` nothing in the domain | yes (empty) |
| later `terminal` / `status` → `model` | yes, at most |
| `automation → {model, client}` | yes |
| `model → {protocol, client, server, pty, cli, GPUI, Android, OS views, transport}` | forbidden |
| `pty → server` | forbidden |
| anything → `apps/*` | forbidden |
| any cycle | forbidden |

`full.md` §5.2:412 "No cycles between crates." The `relayable-protocol` crate owner owns the schema; the QA/test owner for
each `tests/` area owns contract tests. Cross-cutting changes go through the Lead and require schema and contract-test updates
before changing consumers (`full.md` §14.4:971 "Workers can propose changes, but cross-cutting changes go through the lead.
Update schemas and contract tests before changing all consumers.").

## What this contract deliberately does not decide

- The first-release method list: §3 fixes the `Request` shape and the diagnostic `method` string,
but not method names (server-API task; DOC-001 §8.1 "Discovery and version coupling" bounds the inventory).
- Concrete protocol encoding language and the remote transport selection (ADR-002).
- Terminal engine and the `relayable-terminal` boundary (ADR-003).
- Numeric limits: frame size, stream count, credit window, heartbeat interval, idle timeout,
scrollback bytes, and backoff bounds (`full.md` §12:781).
- Deletion permanence and audit history (DOC-001 §5 "Domain hierarchy and stable IDs").
- Persistence schema, retention, and pruning (`full.md` §6.4:492 "Define:").
- Authentication/authorization mechanisms and `principal` semantics (DOC-001 §13 "Connection
lifecycle").
- Environment-variable namespace (`RELAYABLE_*` vs REX aliases vs both) — ADR-005 per DOC-001 §14
row D-08.

## Reconsider if

- DOC-001 or the official reference documents `connection_state` values, a transport/wire format, a
global event sequence, or a retention policy — re-check §3 and §4 for compatibility deltas.
- Measurement shows the framing or credit model cannot meet the performance plan (`full.md` §12:781)
— revise §3 or §5 with recorded evidence.
- ADR-003 selects a terminal engine whose state model cannot feed `TerminalSnapshot` as specified in
§5 — revise the recovery contract.
- The server-API task finds the typed `oneof` cannot express a required operation — route the shape
change through the Lead before changing consumers (`full.md` §14.4:971).

## References

- `full.md` §4:327–345, §5.1:347–401, §5.2:403–414, §6.1:439–460, §6.2:462–469, §6.3:471–486,
§6.4:488–504, §8.2:611–615, §8.4:623–627, §8.5:629–638, §11:758–778, §12:781–805, §13:809–882, §14.3:944, §14.4:960–971.
- `docs/compatibility/reference-behavior-matrix.md` (DOC-001) §2, §5, §6, §7.5, §8.1, §8.2, §8.3,
§10, §11, §12, §13, §14, §15.
- Sibling decisions: `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md`,
`docs/adr/ADR-002-protocol-schema-and-transport.md`.
