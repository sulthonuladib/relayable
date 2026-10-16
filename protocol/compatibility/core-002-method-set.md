# CORE-002 method and message set (slice interface note)

Status: **frozen input to the CORE-002 server, client, and test assignments.** This file is the
slice's interface note because `docs/architecture/**` is frozen and not owned by this slice. It does
**not** change `docs/architecture/interface-contract.md` (the parent contract) or either ADR; where a
name is not traceable to repository evidence it is marked `Relayable-defined (no REX evidence)`.

Shipped product name is **Relayable** (`full.md` §0.1); the binary and CLI name is `relayable`. The
`rex ...` spellings in `docs/compatibility/reference-behavior-matrix.md` (the matrix) are **reference
evidence only**; no `REX_*` environment name is ever read (DEC-006).

CORE-001 behavior remains authoritative in `protocol/compatibility/core-001-method-set.md`. CORE-002
supersedes the CORE-001 note on exactly one clause — CORE-001 §9's "**Events**: no event is emitted"
— and is otherwise purely additive per ADR-002 D3: no existing field is renumbered or retyped, no
existing member moves into a different `oneof`, and no existing field changes meaning. Every new
field/variant is optional or default-preserving; readers continue to ignore unknown optional fields,
unknown enum values, and unknown `oneof` members safely.

Crate ownership of these shapes: `crates/relayable-protocol/**` (schema and generated `v1` types) is
owned by the protocol task; `crates/relayable-server/**`, `crates/relayable-client/**`, and
`tests/**` consume this frozen interface.

## A. Method: `session.resume`

| Item | Value |
| --- | --- |
| Method string | `session.resume` |
| `Request.operation` member | `session_resume` (tag `18`, next free after `terminal_resize = 17`) |
| Request message | `SessionResumeRequest { string session_id; uint64 last_sequence_applied; }` |
| `Response.result` member | `session_resume` (tag `18`, next free after `terminal_resize = 17`) |
| Response message | `SessionResumeResponse` (field list in B) |
| Rust constant | `relayable_protocol::methods::SESSION_RESUME` |
| Owner | server |

`last_sequence_applied = 0` means **"no cursor"** (the client has applied nothing yet). Any other
value is a resume cursor: the sequence of the last control event the client has already applied.

## B. Retained event log and resume cursor

### B.1 The log

- There is **one server-wide retained log of control events**. Event sequences are assigned by the
  log and are **strictly increasing from `1`**.
- `sequence` is the **total order** and the **resync cursor**. The server's existing global
  `sequence` is henceforth the log's head sequence (`current_sequence`).
- `Envelope.sequence == Event.sequence` for every event frame.
- Every state change that a resume cursor must cover is exactly the set listed in the §D event
  table, and each emits **exactly one** event. No event is emitted that does not correspond to a
  real server state change. In particular, `terminal.resize` mutates the block revision and size but
  deliberately emits **no** event (CORE-001 parity; geometry is projected by
  `session.view`/`BlockRecord` and the attach-time `TerminalSnapshot`), so a replay-only resumer is
  **not** promised resize notifications.

### B.2 Retention

- Retention is bounded by **COUNT** = configuration parameter `event_retention_events`
  (`RELAYABLE_EVENT_RETENTION_EVENTS`, default `4096`; see F).
- Eviction drops **oldest first**.
- `retained_floor_sequence` = the **oldest retained sequence**, or `current_sequence + 1` when the
  log is empty.
- `retained_event_count` = the number of retained events.
- One server-wide log is the single source of truth for `current_sequence`,
  `retained_floor_sequence`, and `retained_event_count`.

### B.3 Replayable cursor (response shape)

A cursor `last_sequence_applied` is **REPLAYABLE** iff

```
retained_floor_sequence <= last_sequence_applied + 1
AND
last_sequence_applied <= current_sequence
```

When replayable, `SessionResumeResponse` carries:

- `snapshot_required = false`
- `snapshot` unset
- `snapshot_sequence = last_sequence_applied`
- `events` = the retained events with sequences exactly
  `last_sequence_applied + 1 ..= current_sequence` — **contiguous, no gap, no duplicate, in order**
- `current_sequence`, `retained_floor_sequence`, `retained_event_count` = current log values
- `snapshot_required_reason` = empty

### B.4 Snapshot fallback (response shape)

Otherwise — **no cursor** (`last_sequence_applied = 0`), a cursor **below the floor**, or a cursor
**ahead of the server** — `SessionResumeResponse` carries:

- `snapshot_required = true`
- `events` **EMPTY** (never a false replay)
- `snapshot` = a **FRESH consistent projection** at `current_sequence` (a `SessionSnapshotResponse`)
- `snapshot_sequence = snapshot.sequence = current_sequence`
- live events start at `snapshot_sequence + 1`
- `snapshot_required_reason` ∈ { `no_cursor`, `cursor_below_floor`, `cursor_ahead` }

### B.5 Terminal output is outside this sequence

Terminal output is **OUTSIDE** the event sequence. It is ordered per stream by `stream_seq`
(`TerminalOutput`) and recovered via the attach-time `TerminalSnapshot`. Byte-exact terminal replay
across a detach is **NOT** guaranteed and is not claimed (`full.md` §6.1:460).

## C. Ordering and atomicity contract

`session.resume` is serialized against event appends by **one lock**. While that lock is held the
server must, in order:

1. capture `S = current_sequence`;
2. build the replay-or-snapshot payload;
3. enqueue the `session.resume` response frame into **that connection's writer queue**;
4. register that connection for live delivery **from `S + 1`**.

Consequences the tests assert:

- The response frame always **precedes any event with sequence > `S`** on that connection.
- A subscriber can **never** see a gap or a duplicate across the snapshot/replay boundary.
- A connection receives live events **ONLY after its first successful `session.resume`**. A bare
  `session.view` does **not** subscribe the connection to live events.

A projection returned by `session.view` or `session.list` reads the head sequence lock-free, so its
`sequence` may already include a change whose event has a higher sequence; this is safe because every
event carries **absolute post-change values** and re-application is idempotent.

## D. Emitted events

All events are control-class frames on `stream_id = 0` with `request_id` unset.
`Event.revision` = the affected entity revision when known, else `0`.

| Event (`Event.event` member) | Tag | Payload fields | Emitted when |
| --- | --- | --- | --- |
| `client_connection_state_changed` | 10 | `client_id`, `connection_state` | after a successful handshake (`negotiated`) and when a resume/view completes synchronization (`ready`) |
| `session_state_changed` | 11 | `session_id`, `state` | session create (`ACTIVE`) and confirmed close (`CLOSED`) |
| `client_disconnected` | 12 | `client_id`, `reason` | on disconnect, immediately after the `disconnected` state change (see below) |
| `writer_lease_changed` | 13 | `session_id`, `block_id`, `stream_id`, `previous_client_id`, `new_client_id`, `lease_held` | writer-lease acquire, release, and takeover |
| `terminal_child_exited` | 14 | `session_id`, `block_id`, `exit_code`, `signal`, `exited` | terminal program exit/reap |

Details:

- `client_connection_state_changed.connection_state` is the contract §4 spelling from
  `relayable_model::ConnectionState::as_str()`.
- **On disconnect**, the server emits `client_connection_state_changed{connection_state:
  "disconnected"}` **immediately followed by** `client_disconnected{client_id, reason}` — **two
  sequences, same cause, ordered** — and then removes the client record, so a snapshot lists live
  clients only (CORE-001 snapshot semantics preserved). `reason` ∈ { `idle_timeout`, `peer_closed`,
  `transport_error`, `flow_control_overflow`, `server_shutdown` }. For an abrupt transport drop the
  reason is classified **first-writer-wins** between `peer_closed` (clean reader EOF) and
  `transport_error` (a failed send or other I/O error), so either value is valid on that path;
  `idle_timeout` is exact for a missed idle deadline.
- `session_state_changed.state` uses `SessionState` (`ACTIVE` / `CLOSED`).
- `writer_lease_changed`: a release with no new holder sets `new_client_id` empty and
  `lease_held = false`.
- `terminal_child_exited`: `signal` is empty when the program exited normally; `exit_code` carries
  the code when known; `exited` distinguishes observed exit from a not-yet-reaped state.
- **NOT emitted, and why:** terminal title/pwd. There is no non-fabricated cheap source short of OSC
  parsing / screen emulation, which is out of scope and forbidden. **No event may carry terminal
  bytes.**

## E. Writer-lease takeover (contract §5)

`block.attach` gained an additive field:

```
BlockAttachRequest { string block_id = 1; AttachMode mode = 2; bool takeover = 3; }
```

(`BlockAttachRequest` is defined in `protocol/schema/control.proto`; the task brief said
`terminal.proto`, which is where `AttachMode` lives — the field is on the actual message.
`BlockAttachResponse` is unchanged.)

- `takeover = true` from a **second client holding a valid session** takes the single lease, emits
  `writer_lease_changed`, and **demotes** the previous holder's stream so its further `terminal.input`
  is rejected with `input_not_owned` and is **never silently dropped**.
- `takeover = false` keeps **CORE-001 behavior** (a second writer is rejected with
  `input_not_owned`).
- Authorization basis is the acting **client identity on the local IPC socket**, and it is **not
  session-scoped**: any client that completed the handshake on that socket may act, because no
  authn/authz mechanism exists in this slice (deferred). Writer exclusivity is enforced solely by the
  single lease, and a non-holder's `terminal.input`/`terminal.resize` is always rejected with
  `input_not_owned`.

## F. Configuration parameters

Values are **parameters, not invented constants**.

| Environment variable | Meaning | Default | Invalid |
| --- | --- | --- | --- |
| `RELAYABLE_EVENT_RETENTION_EVENTS` | retained-log count bound (`event_retention_events`) | `4096` | startup error, non-zero exit |
| `RELAYABLE_IDLE_TIMEOUT_MILLIS` | connection idle timeout | `30000` | startup error, non-zero exit |
| `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS` | server heartbeat interval | `5000` | startup error, non-zero exit |

- Heartbeat interval **must be < idle timeout**, or startup fails.
- Only **`RELAYABLE_*`** names are read. `REX_*` names are reference evidence and are **never read**
  (DEC-006).

## G. Explicit non-claims / deferred

- No server-crash or host-reboot session survival.
- Terminal output is **not** byte-exact replayable.
- No persistence / SQLite at CORE-002. A durable SQLite store landed later in STORE-001 (INT-018 `f4d59f8`).
- No authn / authz. Consequently `block.attach` and `session.resume` are **not** session-scoped:
  any handshaken client on the local IPC socket may act, and the writer lease is the only exclusivity
  mechanism.
- No TLS.
- No automatic client reconnect/backoff loop in this slice. Backoff stays a **client policy
  parameter** and is unimplemented.
- CLI `events` / `wait` / `attach` surfaces remain deferred.
- The transition to `disconnected` is observable through the event pair above; the client record is
  then removed, so a snapshot lists live clients only (CORE-001 snapshot semantics preserved).

## H. Compatibility

- Additive per ADR-002 D3. CORE-001 notes remain authoritative for CORE-001 behavior and are
  superseded **ONLY** on the clause "no events are emitted by this slice".
- Every new field/variant is optional or default-preserving.
- Unknown/ignored handling is unchanged: unknown optional fields, unknown enum values, and unknown
  `oneof` members stay safely ignorable.
- Schema placement note: `SessionState` moved from `control.proto` to `events.proto` (same package,
  same wire meaning, same generated type `relayable.protocol.v1.SessionState`) so `events.proto`
  needs no import and `control.proto` can import `Event` for `SessionResumeResponse` without a
  protobuf import cycle.
