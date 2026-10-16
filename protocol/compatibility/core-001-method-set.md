# CORE-001 method and message set (slice interface note)

Status: **frozen input to the CORE-001 server, client, and CLI assignments.** This file is the
slice's interface note because `docs/architecture/**` is frozen and not owned by this slice. It does
**not** change `docs/architecture/interface-contract.md` (the parent contract) or either ADR; where a
name is not traceable to repository evidence it is marked `Relayable-defined (no REX evidence)`.

Shipped product name is **Relayable** (`full.md` §0.1); the binary and CLI name is `relayable`. The
`rex ...` spellings in `docs/compatibility/reference-behavior-matrix.md` (the matrix) are
**reference evidence only**. `relayable ls`, `relayable session inspect`, and so on are our spelling
of the matrix's documented `rex ls` behavior. There is no `rex` binary, alias, or package name.

Acceptance properties covered: a buildable workspace; `relayable-model` + `relayable-protocol` with
tagged IDs, revision/sequence, a framed `Envelope` codec, the error-code vocabulary, the provisional
limits, and the local socket path/ownership rules. Section 9 lists what is explicitly **not**
implemented.

## 1. Crate ownership (this slice)

| Path | Owns |
| --- | --- |
| `crates/relayable-model` | Entities, tagged IDs, revisions, snapshot/value types. Pure: no transport, async runtime, or serialization derives. |
| `crates/relayable-protocol` | Schema (`protocol/schema/*.proto`), generated `v1` types, `model <-> wire` ID conversions, frame codec, method/error/limit vocabularies, optional `transport-unix`. |
| Stubs (`relayable-pty`, `relayable-server`, `relayable-client`, `relayable-cli`, `tests`) | No logic yet; later CORE-001 assignments replace them. |

## 2. Method table

`Request.method` carries one of these exact strings. The frozen contract fixes the `Request` shape
but leaves the first-release method list to the server-API task, which is this slice.

| Method string | Request message | Response message | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `session.create` | `SessionCreateRequest` | `SessionCreateResponse` | server | matrix §8.2/§8.3 (documented server method) |
| `session.list` | `SessionListRequest` | `SessionListResponse` | server | matrix §8.2 |
| `session.view` | `SessionSnapshotRequest` | `SessionSnapshotResponse` | server | matrix §8.2/§8.3 |
| `session.close` | `SessionCloseRequest` | `SessionCloseResponse` | server | mirrors documented CLI family `session close` (`full.md` §8.1) and documented `block.close`; method string **Relayable-defined (no REX evidence)** |
| `block.attach` | `BlockAttachRequest` | `BlockAttachResponse` | server | **Relayable-defined (no REX evidence)** |
| `block.detach` | `BlockDetachRequest` | `BlockDetachResponse` | server | **Relayable-defined (no REX evidence)** |
| `terminal.input` | `TerminalInput` | `TerminalInputAccepted` | server | **Relayable-defined (no REX evidence)** |
| `terminal.resize` | `TerminalResize` | `TerminalResizeAccepted` | server | **Relayable-defined (no REX evidence)** |

## 3. Message set

The generated Rust types live in `relayable_protocol::v1`.

| Message | Contract source | Notes |
| --- | --- | --- |
| `Envelope`, `Handshake`, `ServerHello`, `Limits`, `Heartbeat`, `Request`, `Response`, `Error`, `ErrorCode` | contract §3, §6 | Envelope body is a `oneof`. |
| `SessionCreateRequest/Response`, `SessionListRequest`, `SessionSummary`, `SessionListResponse`, `SessionSnapshotRequest/Response`, `ServerRecord`, `SessionRecord`, `WindowRecord`, `BlockRecord`, `ProgramStatus`, `ClientRecord`, `SessionCloseRequest/Response`, `BlockAttachRequest/Response`, `BlockDetachRequest/Response`, `TerminalInputAccepted`, `TerminalResizeAccepted`, `SessionState`, `BlockKind`, `ProgramState` | contract §2, §5, §7 | Snapshot is a projection at one server `sequence`; per-entity `revision` is the optimistic-concurrency token. |
| `AttachMode`, `TerminalInput`, `TerminalResize`, `TerminalOutput`, `TerminalSnapshot`, `FlowControl` | contract §5 | Contract §3 lists the envelope body as `terminal_data` while §5 names the message `TerminalOutput`; both hold here: field `Envelope.terminal_data` has type `TerminalOutput`. |
| `Event`, `ClientConnectionStateChanged`, `SessionStateChanged` | contract §3 | The `Event` variant exists so the envelope shape is complete; **no events are emitted by this slice.** |

Field names documented by the matrix (`session_id`, `block_id`, `window_id`, `revision`,
`client_id`, `principal`, `connection_state`, `connected_at`, `exit_code`, `pwd`, `title`, `state`,
`progress`) are reused verbatim. All other names are **Relayable-defined (no REX evidence)**.

## 4. Framing

- One message = **4-byte little-endian `u32` length prefix** followed by one protobuf-serialized
  `Envelope`.
- `stream_id = 0` is always the **control stream**; terminal streams are multiplexed with nonzero
  `stream_id`.
- The same framing is used over the local Unix socket and (later) the remote TLS transport.
- The codec rejects (distinguishably) zero-length frames, truncated prefixes/bodies, over-limit
  frames, and malformed bodies; a clean EOF at a frame boundary yields `Ok(None)`.
- Default `max_frame_bytes` = `1048576` (provisional).

## 5. Protocol version

- Advertised version: **`1.0`** (`major.minor`).
- Minor changes are additive only; clients ignore unknown optional fields and unknown enum values.
- An unsupported major yields the structured `incompatible_version` error.

## 6. Provisional limits

`relayable_protocol::limits::DEFAULT_LIMITS`, advertised via `ServerHello.limits`. **Provisional
configuration parameters pending measurement** (`full.md` §12; the contract leaves them open).

| Limit | Value |
| --- | --- |
| `max_frame_bytes` | 1048576 |
| `max_concurrent_streams` | 8 |
| `credit_window_bytes` | 262144 |
| `heartbeat_interval_millis` | 5000 |
| `idle_timeout_millis` | 30000 |

## 7. Socket path and permissions

`relayable_protocol::transport::default_socket_path()` (feature `transport-unix`, **not** a default
feature; DEC-010 keeps `std::os::unix`/socket types out of a default build):

1. `$XDG_RUNTIME_DIR/relayable/relayable.sock` when `XDG_RUNTIME_DIR` is a non-empty existing
   directory;
2. else `/run/user/<uid>/relayable/relayable.sock` when that runtime root exists;
3. else `$TMPDIR/relayable-<uid>/relayable.sock`, falling back to `/tmp/relayable-<uid>/relayable.sock`.

`bind_listener` creates the parent directory `0700`, removes a stale socket file only when
connecting to it is refused (never replaces a live listener), and sets the socket file `0600`
(contract §3; `full.md` §6.2, §7.1, §11). Peer-credential checks (`SO_PEERCRED`) are deferred
(section 9).

## 8. Error codes → contract strings

`relayable_protocol::error_code::contract_str(ErrorCode)` returns exactly:

| `ErrorCode` | contract string | Source |
| --- | --- | --- |
| `BadRequest` | `bad_request` | matrix §8.3 |
| `MethodNotFound` | `method_not_found` | matrix §8.3 |
| `IncompatibleVersion` | `incompatible_version` | Relayable-defined (no REX evidence) |
| `Unauthenticated` | `unauthenticated` | Relayable-defined (no REX evidence) |
| `Unauthorized` | `unauthorized` | Relayable-defined (no REX evidence) |
| `UnknownEntity` | `unknown_entity` | Relayable-defined (no REX evidence) |
| `AmbiguousTarget` | `ambiguous_target` | Relayable-defined (no REX evidence) |
| `StaleSequence` | `stale_sequence` | Relayable-defined (no REX evidence) |
| `SnapshotRequired` | `snapshot_required` | Relayable-defined (no REX evidence) |
| `DuplicateRequest` | `duplicate_request` | Relayable-defined (no REX evidence) |
| `InputNotOwned` | `input_not_owned` | Relayable-defined (no REX evidence) |
| `Timeout` | `timeout` | Relayable-defined (no REX evidence) |
| `Cancelled` | `cancelled` | Relayable-defined (no REX evidence) |
| `RateLimited` | `rate_limited` | Relayable-defined (no REX evidence) |
| `Internal` | `internal` | Relayable-defined (no REX evidence) |

`bad_request` and `method_not_found` are the only names reused from the reference; the matrix marks
the complete error-code set unknown.

## 9. Deferred / explicitly not implemented in this slice

- **Events**: no event is emitted. `Event`/`ClientConnectionStateChanged`/`SessionStateChanged`
  exist only to complete the `Envelope` shape.
- **Authn/authz**: no mechanism; `principal` is left unset.
- **Persistence**: none at CORE-001 (no tombstones, no retention/pruning, no metadata store). A durable SQLite store landed later in STORE-001 (INT-018 `f4d59f8`); see ADR-004 Decision 2.
- **Remote TLS transport**: not implemented.
- **Snapshot sections**: `layout`, `status`, and `automation_scripts` are not in
  `SessionSnapshotResponse`.
- **Terminal engine / screen emulation**: none. `TerminalSnapshot.screen_bytes` is the contract's
  bounded byte tail (not an emulated screen); `scrollback_bytes` is left empty.
- **Peer-credential checks** (`SO_PEERCRED`) and any `libc`/`unsafe`: deferred; not part of the
  CORE-001 acceptance list.
- **Environment `RELAYABLE_BLOCK`/`RELAYABLE_CLIENT`**: not defined in this slice (see section 11).

## 10. CLI surface and exit codes

Exit codes adopt the matrix §7.5 semantics verbatim as the frozen table for the CLI assignment; this
is the basis, not an invention.

| Code | Meaning |
| --- | --- |
| `0` | success |
| `1` | command failed |
| `2` | command line invalid |
| `3` | target not found or ambiguous |
| `4` | server could not be reached |
| `130` | interrupted |

Server-error → exit-code mapping the CLI follows: contract `unknown_entity` / `ambiguous_target` →
**3**; connect/handshake failure or any transport error before a server response → **4**; local
argument/usage error → **2**; any other server-reported error code → **1**. `130` requires SIGINT
handling in the CLI assignment (not in model/protocol).

## 11. Environment variables

- `RELAYABLE_SERVER` — CLI input: filesystem path of the server's Unix socket, overriding
  `$XDG_RUNTIME_DIR/relayable/relayable.sock`. If unset, socket discovery is as in section 7.
- `RELAYABLE_SESSION` — CLI input: the default session target for commands that take `-s/--session`
  when the flag is absent.
- `XDG_RUNTIME_DIR` — the only other environment input (socket discovery).

`RELAYABLE_BLOCK` and `RELAYABLE_CLIENT` are **not** defined in this slice: it has no
block-targeting or client-targeting command. The `REX_*` names in the matrix are reference evidence
only and are **not** read. `RELAYABLE_*`-only is adopted here per ADR-002 D14 and DEC-006 remains
the pending final call (ADR-005). Neither `relayable-model` nor `relayable-protocol` reads any of
these variables for control flow.

## Server implementation notes

Scope: the CORE-001 server lives in `crates/relayable-server/**` (`config`, `registry`, `session`,
`connection`, `dispatch`, `snapshot`, `signals`, `lib`, `main`). It is synchronous threads + blocking
`std::io` over `std::os::unix::net`; no async runtime, no `clap`. The server owns each PTY child in
the session registry, never in a connection: a client disconnect, timeout, reset, or mid-request
drop removes only the connection's client record, streams, and writer lease. Only `session.close`
and graceful shutdown terminate a child.

Dispatch interpretations (the spec fixed the outcome but not always the mapping):

- Handshake: the first frame must be a `Handshake` envelope with `request_id = 1`; any other body,
  or a missing/different `request_id`, gets `bad_request` and closes. An unsupported major gets
  `incompatible_version` with `details{"accepted": "1.x"}`. Both sides advertise empty
  `capabilities`.
- Method/operation: `Request.method` must equal the string for its `operation` variant; an empty,
  absent, unknown, or mismatched pair gets `method_not_found` (contract §6).
- `block.attach`: `mode` must be `WRITER` or `OBSERVER` (never `UNSPECIFIED`). A second `WRITER`
  while the lease is held, or `terminal.input`/`terminal.resize` from a non-lease stream, gets
  `input_not_owned`. Exceeding `max_concurrent_streams` on one connection gets `bad_request` with
  `details{"limit": ...}`. Stream ids start at `1`; `0` is control only, so `terminal.input` or
  `terminal.resize` naming `stream_id = 0` is an unregistered stream and gets `unknown_entity`.
  The tail snapshot and subscriber registration happen under the block's single output lock, so
  `screen_bytes` plus the frames delivered after attach are a contiguous, non-duplicated suffix of
  the block output (no gap at the attach boundary).
- Snapshot `clients` lists every client currently connected to the server, across all sessions;
  `principal` is unset and `layout`/`status`/`automation_scripts` are absent (authentication and
  those sections are deferred). `BlockRecord.cols/rows` is the current PTY size; `program_status` is
  `RUNNING` while alive and `EXITED` + `exit_code`/`signal` after the reaper observes exit. A
  `Failed(reason)` connection state is transmitted as the contract §4 spelling `Failed`; the reason
  is logged server-side, never put on the wire.
- `session.close` is idempotent. It calls `PtyHandle::shutdown(~500 ms)` (the writer is already
  taken, so its `exit\n` path does not run), then an explicit `PtyHandle::kill()` (SIGHUP, poll,
  SIGKILL fallback) and a confirming `try_wait()`. Because an interactive shell enables job control,
  a descendant such as `sleep 30 &` gets its own process group but shares the child's session, so
  the server also signals the child's process group (when it is its own group leader, as
  `portable-pty`'s `setsid()` guarantees) and every `/proc` member of the child's session; the block
  reader/reaper/input threads are joined with a bounded (~1 s) wait and detached if needed, never
  joined unbounded. Only when the child is confirmed reaped does the server clear the writer lease
  and subscribers, mark the block dead, mark the session `CLOSED`, bump the entity revision and the
  server `sequence`, and retain the record as an in-memory tombstone. If the exit cannot be
  confirmed, the session stays `ACTIVE` (lease/subscribers intact, still listed) and the reply is
  `Error{ code: internal, message: "child did not exit within 500 ms", retryable: true,
  details{pid} }`. `session.list` returns active sessions only. Attaching to, or sending input on, a
  closed block or a stale stream gets `unknown_entity`. Crash/reboot survival is **not** provided or
  claimed.
- `terminal.input` enqueues into a bounded per-block input queue serviced by a dedicated PTY-writer
  thread; the connection thread never performs a blocking PTY write. `TerminalInputAccepted.accepted_bytes`
  therefore means "accepted into the bounded queue", not "written to the PTY". A full queue/byte
  budget (4 MiB) replies `Error{ code: rate_limited, retryable: true }` instead of blocking or
  growing memory.
- Flow control (output): each stream starts with `credit_window_bytes` of credit; `FlowControl.ack_bytes`
  tops it up, capped at the window. A stream that exhausts its window or whose bounded (8192-frame)
  outbound queue is full has its connection closed (a transport drop); the session and child stay
  alive. Heartbeats are emitted by the writer when idle for `heartbeat_interval_millis`; the reader's
  socket read timeout is `idle_timeout_millis` and a timeout is a disconnect, not a child kill.
- Naming/descriptor deviations from the frozen contract: the contract §3 envelope field `timestamp`
  is implemented as `timestamp_unix_millis`; contract §2's `server` descriptor lists version,
  capabilities, and limits, but `ServerRecord` carries only id/version/sequence/started-at and the
  capabilities/limits are carried in the handshake `ServerHello` instead. The contract's `block.close`
  is **reference evidence only** for the `session.close` string; it is not a wire method (the CORE-001
  method set is the eight names in §2), and the implementation's internal method is
  `BlockRuntime::close`.
- Signals/shutdown: with one documented `unsafe` block in `signals.rs`, `SIGINT`/`SIGTERM` handlers
  only set an `AtomicBool`. The accept loop is non-blocking with a 20 ms retry sleep. On shutdown the
  server closes connections, kills every live child via its PTY handle, joins connection threads
  best-effort, unlinks the socket, and exits `0`. Orphaned children on `SIGINT` are therefore **not**
  a documented non-guarantee here: `graceful_shutdown_kills_live_children` proves the foreground
  server dies on `SIGTERM` and the recorded child pid disappears.

Evidence: `crates/relayable-server/tests/server_e2e/**` (`cargo test -p relayable-server --offline`)
spawns a real `relayable-server` subprocess and drives raw frames to prove handshake (`1.0` +
limits), `session:<uuidv7>` identity, PTY I/O (`/dev/pts/`), `stty size` before and after a resize,
disconnect survival of the same shell process, reattach, `session.close` child termination, the
conformance errors (oversized/zero-length/truncated frames, `method_not_found`, `unknown_entity`,
writer conflict, observer input), descendant termination on close, a non-reading program not wedging
the connection plus `rate_limited` on input-queue overflow, attach-snapshot/frame contiguity, and
closed-block/stale-stream `unknown_entity`. In-crate unit tests cover the bounded tail buffer, the
duplicate request window, resize record/revision, `/proc` session parsing, the session registry, and
the "cannot confirm child exit" close branch (test-only hook).

Known tested gap: the output flow-control **slow-client** path (a client that stops reading is closed
once the credit window and the outbound queue are exhausted) is implemented but is **not** directly
asserted by an automated test in this slice. The input-queue overflow path (`rate_limited`) **is**
asserted.
