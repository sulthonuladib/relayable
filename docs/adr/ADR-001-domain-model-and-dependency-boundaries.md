# ADR-001 — Domain model and dependency boundaries

**Status:** Proposed — Lead decision; pending Manager acceptance and DECISIONS.md index update (DEC-002 → ADR-001)

## Context

Relayable is a persistent session platform whose server owns sessions and PTYs independently of any
viewer (`full.md` §1.3, `full.md:111`: "Session lifetime is independent of client lifetime."). Four
consumers must share one meaning of "session", "window", "block", and "client": the Rust server, the
CLI, the GPUI Linux client, and a future Kotlin/Android client (`full.md` §1.1, `full.md:94`). Lua
automation is a fifth consumer that must not bypass the same typed API (`full.md` §5.2,
`full.md:411`).

`full.md` §14.4 makes this ADR a prerequisite for parallel coding, requiring an interface artifact
with "entity types and IDs; session snapshot schema; request/event envelope; connection state model;
terminal stream interface; error representation; crate ownership and dependency direction"
(`full.md:962`). `full.md` §17 lists ADR-001 as "Domain and dependency boundaries" owned by the Lead,
done when the "Decision [is] recorded" (`full.md:1182`); `docs/agent-loop/DECISIONS.md` maps DEC-002
to this document.

Three constraints bound the decision space:

1. **No wire compatibility claim.** "Do **not** claim binary or wire-level compatibility with the
   original REX implementation unless the actual protocol is documented and interoperability tests
   prove it." (`full.md` §3, `full.md:323`; DOC-001 §2).
2. **Model purity and dependency direction.** "Model types must not depend on GPUI, Android, OS
   views, or transport." and "Protocol maps model operations to versioned requests/events."
   (`full.md` §5.2, `full.md:405–406`); "No cycles between crates." (`full.md:412`).
3. **Security is a release gate** (`full.md` §11, `full.md:758`): per-principal authorization
   (`full.md:447`) and restrictive local socket permissions (`full.md:762`).

This ADR decides the durable shape of the domain layer and the crate dependency graph. It does not
choose a serialization format or transport (ADR-002) or a terminal engine (ADR-003).

## Evidence base

- **`full.md`** — normative requirements; citations give section, verified line number, and a short
  verbatim quote.
- **DOC-001 (`docs/compatibility/reference-behavior-matrix.md`)** — documented reference behavior with
  per-row labels, cited by section heading only, never by its Source-column upstream line numbers;
  DOC-002 recorded that DOC-001's `events.md` line references do not match canonical numbers.
- **`docs/agent-loop/`** — `TASKS.md:98–100` states ADR-001 acceptance: context, options, decision,
  consequences, reconsideration evidence; no option may violate `full.md` §5.2 or §6 transport
  security. `DECISIONS.md` maps DEC-002 to ADR-001 and records the row as Pending.

**External citations.** This ADR makes one external technical claim, RFC 9562 (UUIDv7), cited with the
key `[RFC9562-local]` (local file path, source URL, UTC retrieval date, HTTP status, sha256 defined in
Sources). Any external claim not so supported is listed under "Unverified / outstanding verification"
with the outstanding step named.

Label vocabulary follows DOC-001 §3 Label legend. Anything not traceable to DOC-001 and originating
in this ADR is marked **Relayable-defined (no REX evidence)**.

## Options considered

Every option is admissible only if it satisfies `full.md` §5.2 dependency direction and §6 security
requirements (`docs/agent-loop/TASKS.md:98–100`). An option that would leak a UI/OS/transport
dependency into the model, or introduce a security side channel, is rejected on that ground alone.

| Option | Pros | Cons | Not chosen because |
|---|---|---|---|
| 1. Random opaque IDs, no entity tagging | Simplest; no parser | Cannot distinguish entity kinds without out-of-band schema; cross-entity misuse becomes a runtime lookup failure | `full.md:343` requires tagged IDs or explicit type fields; DOC-001 §5 documents the `type:` prefix shape |
| 2. Sequential integer IDs | Compact; obvious ordering | Mutable global counter; guessable IDs; reuse after compaction is easy to get wrong | `full.md:329` requires IDs "never reused"; guessable identifiers weaken the §11 posture (`full.md:762`) |
| 3. Tagged strings only, no Rust newtypes | Self-describing; minimal code | A bare `String` crosses every crate boundary; misuse is not a compile error | Satisfies `full.md:343` literally but not the intent; rejected for Option 4 |
| 4. Tagged wire form plus distinct Rust newtypes (**chosen**) | Self-describing wire form *and* compile-time misuse detection; matches DOC-001 §5 | Explicit parse/serialize layer and per-kind boilerplate | Chosen: the only option making cross-entity ambiguity both a compile-time and runtime error |
| 5. Model derives serialization and doubles as the wire type | Fewer types; no conversions; less day-one code | Serialization attributes and codegen leak into the domain, violating `full.md:405`; wire changes force domain edits | `full.md` §5.2:405–406 separates model from protocol; the conversion layer is accepted as cost |
| 6. Snapshot plus unbounded in-memory event log | Trivial replay while the process lives; simple reconnect | Unbounded memory | Violates `full.md` §12 ("bounded scrollback and queues", `full.md:799`) and §6.4:500 ("Bound per-terminal scrollback and outgoing queues.") |
| 7. Persisted event log as resume source of truth | Stronger resume across restart; complete audit history | Durable terminal-adjacent state conflicts with `full.md:490` ("Do not store unbounded PTY output as ordinary rows.") and `full.md:804`; crash-consistent replay is a large first-release commitment | `full.md:480` forbids promising crash/reboot survival unless implemented and tested; first release uses bounded retention plus `snapshot_required` |
| 8. Crate granularity: all nine §5.1 crates day one | Matches the §5.1 sketch literally | Creates boundaries whose content is undecided before ADR-003 | `full.md:401`: "Don't create a crate for every label on day one. Split only when it establishes a useful dependency boundary." |
| 9. Single crate with modules | Fastest start; no inter-crate wiring | Model/protocol/server boundary is a convention, not a compiler edge; §5.2 cycles cannot be mechanically checked | `full.md:405–412` is only checkable with real crate boundaries |
| 10. Six crates plus explicitly deferred splits (**chosen**) | Every rule-bearing boundary is a real crate; deferrals are named and justified; `cargo test` stays Android-free | Two later splits remain; first-release terminal/status code sits in `relayable-server` modules temporarily | Satisfies `full.md:401` while keeping §5.2 checkable; defers only boundaries depending on ADR-003 |
| 11. Layout owned by each client | Simple for one client; no layout revision traffic | Attached clients diverge; `full.md:338` requires separating shared layout from local UI preferences; DOC-001 §10.2 documents a revisioned session-level `session_view_changed` | Shared layout is a server-owned, revisioned entity; only window size/position and theme stay client-local |
| 12. ProgramStatus model is an OSC parser | One place to parse and store status | Puts byte-level terminal parsing in the domain layer, violating `full.md:405` | `full.md:631`: OSC 7501 is "a terminal-level integration, not the native Relayable wire protocol"; parsing lives in the terminal/status layer |
| 13. Lua embedded in the first release | Automation available sooner | `full.md:619`: "The initial goal is not to embed a full automation runtime before the typed API and event model stabilize."; risks an untyped second call path, forbidden by `full.md:411` | The typed API and event model must stabilize first; `relayable-automation` is deferred |

## Decision

The domain model is fixed as follows; the frozen sub-decisions A–N are detailed in "Interface
details" and "Proposed crate layout and dependency direction".

| # | Decision |
|---|---|
| A | Entities: exactly the eleven named by `full.md` §4:331–341. |
| B | ID encoding: opaque, never-reused `<kind>:<uuid>`, singular lowercase kind, UUIDv7-shaped portion inferred from DOC-001 §5. |
| C | Tagged IDs plus type fields: tagged text on the wire; distinct newtypes in `relayable-model`; no bare `String` ID across crates; `BlockId` still carries `session_id`. |
| D | Host profiles client-local: `host:<uuid>` stored with the client, never a server-side identity. |
| E | Labels are not identity: stable ID and human label separate; lowercase-comparable for hints only. |
| F | Snapshot plus revision: per-entity `revision: u64` from 1; server `sequence: u64`; explicit snapshots; resume cursor `(scope, last_sequence_applied)`; stale cursor yields `snapshot_required`. |
| G | Deletion and audit: IDs never reused; authorized destroy removes the live entity and records a tombstone; no terminal bytes persisted. |
| H | Event model: versioned envelope with type, payload, context, revision/sequence, timestamp; unknown fields ignorable. |
| I | Layout: server-owned tree of Blocks with split ratios inside a Window; local UI preferences stay client-local. |
| J | ProgramStatus: normalized record, not an OSC parser; terminal-level ingestion. |
| K | AutomationScript: Lua client of the same typed API under the same authorization model; embedding deferred. |
| L | Crate layout: six crates plus two named deferrals (below). |
| M | Dependency direction: the allowed and forbidden edge sets (below). |
| N | Lua path: same model/protocol path; no side channel. |

## Interface details

### Entities and IDs

The domain has exactly the eleven entities named by `full.md` §4:331–341, with the spec names kept
and a proposed Rust type name added. The entity set is frozen for the first release.

| # | Spec name (`full.md` §4) | Proposed Rust type | ID kind | Owning scope |
|---|---|---|---|---|
| 1 | Server | `Server` | `server` | process |
| 2 | Host profile | `HostProfile` | `host` | client-local |
| 3 | Session | `Session` | `session` | server |
| 4 | Window | `Window` | `window` | session |
| 5 | Block | `Block` | `block` | window (carries `session_id`) |
| 6 | Client | `Client` | `client` | server / session |
| 7 | PTY/process | `PtyProcess` | `pty` | server |
| 8 | Layout | `Layout` | — (tree inside `Window`) | window |
| 9 | Event | `Event` | — (envelope) | server |
| 10 | ProgramStatus | `ProgramStatus` | — (keyed by record `id`) | block/terminal |
| 11 | Automation script | `AutomationScript` | `script` | client/server |

`Layout`, `Event`, and `ProgramStatus` are value types owned by their container entity; they have no
ID namespace because they are not independently addressable by target resolution. `ProgramStatus`
records are addressed by the terminal-level record `id` documented in DOC-001 §11.2 Records, IDs, and
lifetimes, not by a Relayable entity ID.

**ID encoding (B).** IDs are opaque, never reused (`full.md` §4:329: "IDs that are never reused"), and
encoded as `<kind>:<uuid>` with a singular lowercase kind.

| Kind tag | Status |
|---|---|
| `session` | Documented prefix form `session:<uuid>`; DOC-001 §5 Domain hierarchy and stable IDs |
| `block` | Documented prefix form `block:<uuid>`; DOC-001 §5 Domain hierarchy and stable IDs |
| `server` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |
| `host` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |
| `window` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |
| `client` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |
| `pty` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |
| `script` | **Relayable-defined (no REX evidence)** extension of the documented `type:` prefix convention |

**Documented prefix form (`verified-official-docs`).** DOC-001 §5 Domain hierarchy and stable IDs
documents `session:<uuid>` and `block:<uuid>` and gives the examples
`session:01a10cf3-2655-7e48-9b9c-43dd6612bef5` and
`block:01a10d99-0c06-7795-bf67-798ce4c36a33`, and records that IDs "never change and are never
reused". DOC-001 §5 marks the ID scheme beyond the prefix as `unknown`.

**Documented examples are UUIDv7-shaped (`inferred`).** In both examples the version nibble is `7`.
This matches RFC 9562 §5.7 UUID Version 7, which defines a version-7 UUID as "a time-ordered value
field derived from the widely implemented and well-known Unix Epoch timestamp source, the number of
milliseconds since midnight 1 Jan 1970 UTC, leap seconds excluded" [RFC9562-local]. The RFC's field
layout is [RFC9562-local] §5.7:

| Field | Bits | RFC 9562 §5.7 definition |
|---|---|---|
| `unix_ts_ms` | 0–47 | "48-bit big-endian unsigned number of the Unix Epoch timestamp in milliseconds" |
| `ver` | 48–51 | "4-bit version field ... set to 0b0111 (7)" |
| `rand_a` | 52–63 | "12 bits of pseudorandom data to provide uniqueness ... and/or optional constructs to guarantee additional monotonicity" |
| `var` | 64–65 | "2-bit variant field ... set to 0b10" |
| `rand_b` | 66–127 | "final 62 bits of pseudorandom data ... and/or an optional counter" |

**Adopting UUIDv7 for all entity kinds (Relayable-defined).** Relayable uses the version-7 layout
for the `<uuid>` portion of every kind, not only `session`/`block`. The reference documents no UUID
version for the prefixes DOC-001 §5 lists, so this adoption is **Relayable-defined (no REX evidence)**;
the RFC defines the layout, not REX's choice of it. The document shape is fixed, but the concrete UUID
generation library and its version remain an implementation-time verification step; this ADR names no
Rust crate or API.

**RFC 9562 caveats that constrain how IDs may be used.**

- **Time-ordered is not strict monotonicity (required caveat).** "Normally, time-based UUIDs from this
  document will be monotonic due to an embedded timestamp; however, implementations can guarantee
  additional monotonicity" only via counters, and "Counter rollovers MUST be handled by the
  application to avoid sorting issues" ([RFC9562-local] §6.2). Relayable does not use UUIDv7 as its
  ordering mechanism; total order comes from the server `sequence` (Decision F).
- **IDs are not security capabilities.** "they MUST NOT be used as security capabilities (identifiers
  whose mere possession grants access)"; an embedded timestamp "does signal the order of creation"
  ([RFC9562-local] §8). Authorization must never rely on possession of an entity ID
  (`full.md` §11, `full.md:762`).
- **Timestamps are advisory; UUIDs are opaque.** Timestamps MAY be altered with "no requirement or
  guarantee about how close the clock value needs to be to the actual time" ([RFC9562-local] §6.1),
  and UUIDs should be treated "as opaquely as possible" (§6.12). The UUID timestamp is not an audit
  timestamp, and the model must not parse the UUID for behavior.
- **Storage and entropy.** Text storage is "unnecessarily verbose"; UUIDs "SHOULD be stored ... as the
  underlying 128-bit binary value" where feasible, and generation "SHOULD utilize a cryptographically
  secure pseudorandom number generator (CSPRNG)" ([RFC9562-local] §6.13, §6.9). Text is the wire/CLI
  form; the metadata store may hold the binary value.

**Tagged IDs plus type fields (C).** The wire/CLI form is the tagged string; it is self-describing
and makes cross-entity misuse detectable. Inside Rust each kind is a distinct newtype (`SessionId`,
`BlockId`, `WindowId`, `ClientId`, `ServerId`, `HostProfileId`, `PtyProcessId`, `ScriptId`) in
`relayable-model`. No bare `String` ID crosses a crate boundary. This does both halves of
`full.md` §4:343: "Use tagged IDs or explicit entity type fields to prevent cross-entity ambiguity."
A `BlockId` inside a window context still carries `session_id` for scoping, so no command relies on an
implicit global lookup.

**Host profiles are client-local (D).** `HostProfileId` values (`host:<uuid>`) are stored with the
client and are never accepted as server-side identities. A host label is a mutable, user-visible
label. DOC-001 §9.2 Key bindings, actions, hosts, logging documents
`rex.host{ label, endpoints, endpoint, id, icon }`, showing `label` and `id` are separate fields;
DOC-001 §13 Connection lifecycle records the host profile as client-side connection configuration
(labeled `inferred` there). The server address list is therefore client-local configuration.

**Labels are not identity (E).** The model stores stable ID and human label separately
(`full.md` §4:329, `full.md:343`). Labels are lowercase-comparable and case-insensitive for resolution
hints only. Target-resolution order is documented behavior owned by the CLI and its documentation per
DOC-001 §6 Target resolution: full ID, then case-insensitive label, then 1-based position, then ID
suffix of at least four characters; ambiguity is an error and exits with status 3 (DOC-001 §7.5 Exit
codes). This ADR defines or invents no CLI flags or syntax. The model must make each step expressible
and must never guess between ambiguous targets (`full.md` §8.1, `full.md:605`: "Ambiguous targets fail
before mutation; never guess silently."; DOC-001 §14 D-06, `verified-official-docs`).

### Snapshot and revision model

This ADR owns the snapshot model required by `full.md` §6.1.

- Every entity carries `revision: u64`, starting at 1 and incremented on every committed mutation of
  that entity. It is the optimistic-concurrency token for mutations on that entity, satisfying
  "Stable entity IDs and monotonically increasing revisions where state changes." (`full.md` §6.1,
  `full.md:449`).
- The server maintains a monotonic `sequence: u64` assigned to every committed event. It defines total
  order and is the resynchronization cursor.
- A **snapshot** is a consistent projection of entities at a given server `sequence`, requested
  explicitly, satisfying "Snapshot request plus incremental event stream." (`full.md` §6.1,
  `full.md:450`; `full.md` §8.4, `full.md:625`).
- A **resume cursor** is `(scope, last_sequence_applied)`. If the cursor predates the server's retained
  floor, the server answers with a structured `snapshot_required` outcome plus a full snapshot rather
  than pretending to replay, satisfying "Resume cursor/token semantics where safe; full snapshot
  fallback when events/output cannot be replayed." (`full.md` §6.1, `full.md:455`).
- Event retention is bounded in memory with a documented, configurable retention floor. There is no
  persisted event log in the first release; `full.md` §6.4 permits a small transactional store for
  durable metadata only (`full.md:490`). Retention and queue sizes are configuration parameters, not
  hard-coded constants: "Add regression thresholds only after a stable baseline exists."
  (`full.md` §12, `full.md:796`).
- A terminal **screen snapshot** and **scrollback recovery** are distinct capabilities. Relayable
  never claims byte-exact recovery of the terminal byte stream (`full.md` §6.1, `full.md:460`).
- This model satisfies `full.md` §6.1:449, §6.1:450, §6.1:455, §6.4:490, and §6.4:495
  ("Crash-consistent writes.").

### Events

Every state change is an `Event` with:

| Field | Meaning | Status |
|---|---|---|
| `type` | Stable name string | Aligned with DOC-001 §10.1 Sources, naming, envelope (`ev.type`) |
| scope | server / session / window / block / client / status | Relayable-defined envelope dimension |
| entity_id | Optional tagged entity ID | Relayable-defined envelope field |
| `revision` | Entity revision at commit | DOC-001 §10.1 and §10.2 document per-event `revision` |
| `sequence` | Server-wide monotonic order/cursor | Relayable-defined; DOC-001 §10.1 marks a universal envelope sequence `unknown` |
| timestamp | Optional commit time | `full.md` §8.4:625 ("timestamp where useful") |

A versioned envelope carries event type, payload, entity/session context, sequence or revision, and
timestamp (`full.md` §8.4, `full.md:625`). Unknown optional fields and unknown event types must be
ignorable safely (`full.md` §1.3, `full.md:121`: "Older clients ignore unknown event fields/types
where safe"; `full.md` §8.4:625; DOC-001 §10.6 Events on the CLI, `verified-official-docs`).

Event type names are reused exactly for the semantics DOC-001 §10 documents, naming the section. The
names below are REX-derived and appear verbatim in DOC-001:

| Scope | Event names | DOC-001 section |
|---|---|---|
| session/window/block | `window_created`, `window_closed`, `window_label_changed`, `active_window_changed`, `block_created`, `block_closed`, `session_label_changed`, `session_view_changed`, `client_connected`, `client_disconnected`, `client_connection_changed`, `creator_changed` | §10.2 Session events (`verified-official-docs`) |
| terminal | `terminal.child_exited`, `terminal.pwd_changed`, `terminal.title_changed`, `terminal.bell`, `terminal.process_changed`, `terminal.desktop_notification`, `terminal.progress_report`, `terminal.clipboard_written`, `terminal.program_status_changed`, `terminal.program_status_removed` | §10.3 Terminal events (`verified-official-docs`) |
| server | `session_created`, `session_destroyed`, `client_joined`, `client_left`, `client_changed`, `keymap_changed`, `hosts_changed`, `tailscale_status_changed` | §10.4 Server events (`verified-official-docs`) |

Payload fields documented by DOC-001 §10 for the same datum are reused verbatim: `session_id`,
`block_id`, `window_id`, `revision`, `client_id`, `principal`, `connection_state`, `exit_code`,
`pwd`, `title`, `state`, `progress`, `text`, `id`, `ids`, `record`, `reason`, `generation`,
`foreground`, `body`, `error` (DOC-001 §10.2, §10.3, §10.4). Any event type or field Relayable adds is
labeled **Relayable-defined (no REX evidence)** in the schema and this ADR, and is never presented as
a REX name. DOC-001 §10.1 labels a universal envelope version, global sequence number, and envelope
timestamp as `unknown`; the `sequence` and envelope `timestamp` fields above are Relayable-defined
additions required by `full.md` §8.4:625, not attributed to the reference.

### Deletion and audit

IDs are never reused (`full.md` §4:329). An authorized destroy removes the entity from the live
snapshot and records a tombstone (ID, destroy sequence, timestamp) in the metadata store when
persistence is enabled (`full.md` §6.3:478). The metadata store never holds terminal bytes
(`full.md` §12:804: "keep terminal byte streams out of metadata DB"; `full.md` §6.4:490). Closing a
client window does not destroy the server-owned session or PTY (`full.md` §6.3:475), and a network
disconnect does not destroy it (`full.md` §6.3:476); crash and host-reboot survival is explicitly
**not claimed** (`full.md` §6.3:480). DOC-001 §5 records deletion permanence and session-close audit
history as `unknown`; the tombstone is Relayable-defined, not documented reference behavior.

### Layout

`Layout` is a tree of `Block`s with split ratios inside a `Window` (`full.md` §4:338). The
authoritative shared layout is server-side and mutated through the same API as other window/block
operations; local client UI preferences (window size and position, theme) are client-local and must
never leak into the model or wire schema (`full.md` §5.3:418: "do not leak GPUI types into shared
crates"). Shared layout changes are revisioned on the `Window` and reported through the session-level
`session_view_changed` event carrying `revision` (DOC-001 §10.2).

### ProgramStatus

`ProgramStatus` is a normalized model of terminal-reported status. Ingestion is a terminal-level
integration, not the native wire protocol (`full.md` §8.5, `full.md:631`). DOC-001 §11 Program Status
Protocol (OSC 7501) is the documented terminal-level source of record. Its documented state set is
`idle`, `working`, `done`, `blocked`, and `error`, with `clear` explicitly "Not a state." (DOC-001
§11.3 States). Its documented keys are `state`, `id`, `kind`, `progress`, `app`, `title`, and `msg`
(DOC-001 §11.4 Keys); DOC-001 §11.2 documents the `id` grammar, root record, and parent/child
nesting.

The model's status shape must not be an OSC parser. Parsing lives in the terminal/status layer; the
model receives a normalized record built from the documented keys. Status changes are relayed with
`terminal.program_status_changed` / `terminal.program_status_removed` and the documented payload
fields `id`, `record`, `reason`, and `ids` (DOC-001 §10.3 Terminal events). The relation between
`terminal.progress_report` payload fields and OSC 7501 keys is `unknown` in DOC-001 §10.3 and must not
be assumed.

### Automation script

An `AutomationScript` (a Lua script) is a client of the same typed API under the same authorization
model, with no hidden bypass (`full.md` §5.2, `full.md:411`). Lua integration is deferred until the
typed API stabilizes (`full.md` §8.3, `full.md:619`) and must not be embedded prematurely. Lua calls
the typed API through the same model/protocol path; there is no side channel.

## Proposed crate layout and dependency direction

Day-one crates: `relayable-model`, `relayable-protocol`, `relayable-client`, `relayable-server`,
`relayable-pty`, `relayable-cli`; plus the non-crate directories in the table below.

| Path | Responsibility | ID/entity exposure |
|---|---|---|
| `crates/relayable-model` | Domain types, IDs, revisions, snapshot/value types | Owns all entity types and newtype IDs |
| `crates/relayable-protocol` | Versioned requests/events, envelope, schema mapping | Maps model operations to messages |
| `crates/relayable-client` | Connection lifecycle, RPC, resync, event sequencing | Consumes model + protocol |
| `crates/relayable-server` | API, authorization, lifecycle, persistence, dispatch; first-release terminal/status modules | Composes model, protocol, pty |
| `crates/relayable-pty` | PTY/process ownership and supervisor boundary; opaque handles only | No domain types |
| `crates/relayable-cli` | `relayable` binary, target resolution, output formats | Consumes model + protocol + client |
| `protocol/{schema,examples,compatibility}` | Schemas, examples, compatibility fixtures | Non-crate |
| `tests/{protocol,lifecycle,reconnect,multi-client,terminal-compat,fixtures}` | Integration and fixture tests | Non-crate |
| `apps/linux` | GPUI desktop client | Must not leak GPUI types into shared crates |
| `apps/android` | Kotlin/Jetpack Compose app and Gradle wrapper | Independent Gradle project |

Deferred splits: `relayable-terminal` and `relayable-status` (their boundary depends on the
terminal-engine decision in ADR-003; first-release code lives in modules inside `relayable-server`),
and `relayable-automation` (after the typed API stabilizes).

This is justified against `full.md` §5.1:401: "Don't create a crate for every label on day one. Split
only when it establishes a useful dependency boundary." The six chosen crates each carry a real
dependency rule (see below); the three deferred crates do not yet have a decidable boundary. Android
is an independent Gradle project and must not make default `cargo test` require Android tooling
(`full.md` §5.1, `full.md:401`; `full.md` §5.3, `full.md:418`).

**Allowed edges (M).**

| From | To | Why |
|---|---|---|
| `relayable-protocol` | `relayable-model` | Protocol maps model operations to messages (`full.md` §5.2:406) |
| `relayable-client` | `relayable-model`, `relayable-protocol` | Client consumes model + wire types |
| `relayable-server` | `relayable-model`, `relayable-protocol`, `relayable-pty` | "Server composes persistence, authz, PTY, model, protocol, and event dispatch." (`full.md` §5.2:408) |
| `relayable-cli` | `relayable-model`, `relayable-protocol`, `relayable-client` | "GPUI/Android/CLI are independent clients of the API." (`full.md` §5.2:410) |
| `relayable-pty` | (nothing in the domain) | Opaque handles only; it is an OS boundary (`full.md` §5.2:407) |
| later `relayable-terminal` / `relayable-status` | `relayable-model` at most | Boundary depends on ADR-003 |
| later `relayable-automation` | `relayable-model`, `relayable-client` only | "Lua must call the typed API and obey the same authorization/invariants; no hidden bypass." (`full.md` §5.2:411) |

**Forbidden edges (M).**

| Forbidden | Reason |
|---|---|
| `relayable-model` → `relayable-protocol`, `relayable-client`, `relayable-server`, `relayable-pty`, `relayable-cli` | Model must not depend on transport (`full.md` §5.2:405) |
| `relayable-model` → GPUI, Android, OS views, transport, async runtime, serialization framework | Purity requirement (below) |
| `relayable-pty` → `relayable-server` | PTY is an OS boundary owned by the server; reverse edge is a cycle |
| anything → `apps/*` | Clients are leaves (`full.md` §5.2:410) |
| `relayable-server` → `gpui` / `android` | GPUI/Android are independent clients (`full.md` §5.2:410) |
| any cycle | "No cycles between crates." (`full.md` §5.2:412) |

`relayable-model` must be pure: no async runtime, no transport, and no serialization-framework
derives. `relayable-protocol` owns wire types and provides explicit conversions. This satisfies
`full.md` §5.2:405 and §5.2:406. The consequence is an explicit conversion layer, accepted in
exchange for a stable domain. The direction is enforced and tested by an assertion over `cargo
metadata` output (or an equivalent CI check) that compares actual edges against the allowed set and
fails on any forbidden edge or cycle. `Cargo.toml` is Lead-owned, so a worker cannot change the graph
(`AGENTS.md` path ownership).

**Lua path (N).** Lua calls the typed API through the same model/protocol path. When
`relayable-automation` is created, its only allowed domain dependencies are `relayable-model` and
`relayable-client`; it must not link `relayable-server` internals or a private dispatch table.

## Consequences

- A hand-written conversion layer sits between `relayable-model` and `relayable-protocol`: wire changes
  do not force domain changes, and domain changes require an explicit protocol mapping update.
- Wire compatibility is testable without touching the domain, which carries no serialization
  attributes.
- The per-entity `revision` plus server `sequence` pair is the concurrency and resync substrate; any
  mutation API must supply the expected entity revision.
- Reconnect after the retention floor yields `snapshot_required`, not a silent partial replay; clients
  implement snapshot fallback.
- No persisted event log means no resume across server restart, consistent with `full.md` §6.3:480;
  tombstone metadata is audit data, not a replay log.
- `relayable-terminal`/`relayable-status`/`relayable-automation` do not exist on day one; their code
  lives in `relayable-server` modules until ADR-003 and API stability justify a split.
- The domain is usable unchanged by the GPUI client and a future Kotlin client; Android codegen
  consumes protocol types, never domain types.
- Security posture is preserved: no model-side privilege shortcut, no host profile accepted as a
  server identity, no capability granted by an event payload.

## Reconsider if

- Wire types must double as domain types to satisfy a generated Kotlin/Android binding and the
  conversion layer proves to be the dominant cost.
- The deferred `relayable-terminal` boundary proves necessary earlier than ADR-003 (for example if
  terminal-engine spiking needs an isolated backend before the engine is chosen).
- Measurement shows the per-entity `revision` plus server `sequence` pair is redundant after a stable
  baseline exists.
- A persisted event log becomes required for resume across server restart, changing the
  `full.md` §6.3:480 guarantee set.
- A REX wire protocol becomes documented **and** an interoperability test is in scope, reopening the
  no-wire-compatibility boundary (`full.md:323`).
- A documented need appears for server-side host profiles, reopening the client-local `HostProfileId`
  decision; or newly documented REX ID forms conflict with the tagged kind prefixes, reopening
  Decision B.

## Open questions

- **UUID implementation and RFC caveats.** The concrete UUID library/version and whether a §6.2 counter
  method is needed remain implementation-time checks (see Unverified / outstanding verification).
- **`sequence` vs `revision` redundancy.** Whether the two can be one field is unresolved until a
  measured baseline exists.
- **Retention floor defaults.** The initial bounded in-memory retention size and configuration surface
  must be measured (`full.md` §12:796).
- **Tombstone schema.** Exact metadata-store schema and pruning for tombstones belong with persistence.
- **Whether `Layout` should be independently addressable.** Decision A treats it as a value type inside
  `Window`; a future API that addresses layout nodes directly would reopen this ADR.
- **Reference gaps.** DOC-001 §5 records deletion permanence and session-close audit history as
  `unknown`; the tombstone is a Relayable choice pending documentation. The ID-suffix resolution step
  (DOC-001 §6) needs a stable short-ID derivation not defined here.

## References

- `full.md` §0.1:22; §1.1:94; §1.3:111,121; §3:323; §4:329,331–341,338,343; §5.1:401;
  §5.2:405–412; §5.3:418.
- `full.md` §6.1:447,449,450,455,460; §6.3:475,476,478,480; §6.4:490,492,495,500.
- `full.md` §7.1:513–515 (XDG state/runtime paths); §8.1:605; §8.3:619; §8.4:625; §8.5:631.
- `full.md` §11:758,762,765; §12:796,799,804; §13.1:813; §14.4:962; §17:1182.
- DOC-001 §2 No binary/wire-compatibility claim; §3 Label legend; §5 Domain hierarchy and stable IDs;
  §6 Target resolution; §7.5 Exit codes; §9.2 Key bindings, actions, hosts, logging.
- DOC-001 §10.1 Sources, naming, envelope; §10.2 Session events; §10.3 Terminal events; §10.4 Server
  events; §10.6 Events on the CLI.
- DOC-001 §11 Program Status Protocol (OSC 7501), incl. §11.2 Records, IDs, and lifetimes, §11.3
  States, §11.4 Keys; §13 Connection lifecycle; §14 Relayable deltas (D-06, D-18, D-32).
- `docs/agent-loop/TASKS.md:98–100`; `docs/agent-loop/DECISIONS.md` (DEC-002).

## Unverified / outstanding verification

- **Concrete UUID implementation.** No UUID crate, library, or version is named or verified.
  Outstanding step: select and pin a CSPRNG-backed UUIDv7 implementation and check it against RFC 9562
  §5.7 and §6.9 with the local test vector ([RFC9562-local] Appendix A.6).
- **Serialization and transport.** No external serialization/transport claim is made here and no such
  documentation was fetched for this ADR; those specifics belong to ADR-002.
- **DOC-001 §15 unknowns.** Reference-matrix unknowns (event retention/replay, allowed
  `connection_state` values, deletion permanence) remain unverified; this ADR does not fill them in and
  marks overlapping answers Relayable-defined.
- **RFC 9562 optional monotonicity.** Whether the generator needs a §6.2 counter method is undecided;
  total order comes from the server `sequence`. Outstanding step: confirm during implementation whether
  batch generation needs a counter.

## Sources

- `[RFC9562-local]` — RFC 9562, "Universally Unique IDentifiers (UUIDs)", May 2024. Local file
  `/tmp/relayable-adr002-evidence/rfc9562-uuid.txt` (114638 bytes); source URL
  `https://www.rfc-editor.org/rfc/rfc9562.txt`; HTTP 200; retrieved **2026-10-15T16:58:20Z** (UTC);
  sha256 `041870106b8701d9fa9261dcbc1679a6b7ab18cb179b37b69d7232431eac8042`. Retrieval metadata is
  recorded in `/tmp/relayable-adr002-evidence/INDEX.md` (row `rfc9562-uuid.txt`). Every `[RFC9562-local]`
  citation above resolves to this single retrieval.
- `full.md`, `docs/compatibility/reference-behavior-matrix.md` (DOC-001), and `docs/agent-loop/**` are
  repository sources, not external; their citations are given inline by section and line.
