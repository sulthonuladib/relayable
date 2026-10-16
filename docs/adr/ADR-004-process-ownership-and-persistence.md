# ADR-004 — Local/remote process ownership and persistence guarantees

| Field | Value |
|---|---|
| Status | **Proposed — Lead decision; pending Manager acceptance and DECISIONS.md index update (DEC-005 → ADR-004)** |
| Date | 2026-10-15 |
| Deciders | Lead (Relayable / Superlogical) |
| Supersedes | None |
| Related | ADR-001, ADR-002, ADR-003, ADR-005; DEC-005; CORE-001; CORE-002 |

> **Status note.** This ADR records the Lead's decision for DEC-005 (local/remote process ownership and
> persistence guarantees). It is `Proposed` pending Manager acceptance and the `docs/agent-loop/DECISIONS.md`
> index update (`DEC-005 → ADR-004`). It changes **no code**: it fixes the ownership boundary that already
> exists in the implementation, chooses the persistence design to be built, and states plainly what is and
> is not guaranteed today. Choices not traceable to REX/reference evidence are marked
> **Relayable-defined (no REX evidence)** inline. No binary or wire compatibility with the reference
> product is claimed (`full.md` §0.1, §3).

---

## Abstract

Relayable's first-release ownership model is **one foreground server process that owns each session's
PTY child in-process, per session, independently of any client connection**. Closing a client window,
losing the network, an idle timeout, a mid-request drop, or a flow-control overflow must not signal,
reap, or stop the child; only `session.close` and graceful server shutdown (SIGINT/SIGTERM) terminate
children. This ADR confirms that record and extends it with the persistence design `full.md` §6.4
requires: a small transactional **SQLite** metadata/audit store that holds **no terminal bytes**, a
defined schema, bounded retention, crash-consistent write rules, and an explicit statement of what is
restored after a restart. It also fixes the identity scope: **one user per server** for the first
release. It explicitly does **not** claim session survival across server crash or host reboot, and it
defers all supervisor mechanisms `full.md` §6.3:480–484 names until a spike proves them.

This ADR records design; it changes no code in the first release.

---

## Context

### What the spec requires

`full.md` §1.3:111 makes the core principle explicit: "**Session lifetime is independent of client
lifetime.** Closing the app or losing network must not kill a server-owned PTY." `full.md` §6.3:475–478
enumerates the first-release guarantees: closing a client window does not destroy the server-owned
session or PTY (`:475`); a remote network disconnect does not destroy it (`:476`); reopening the client
requests a current snapshot and can reattach (`:477`); a session can be explicitly stopped/destroyed
through authorized API calls (`:478`).

`full.md` §6.3:480 is a hard boundary on promises: "Do not promise survival after server crash or host
reboot unless one of these is implemented and tested:" followed by three named mechanisms — "a separate
supervisor owns PTYs and allows server restarts to reattach" (`:482`), "per-session supervisor
services/processes preserve PTY descriptors" (`:483`), "or another proven OS-level arrangement" (`:484`).
`full.md` §6.3:486 adds: "Host reboot necessarily ends ordinary processes unless a job is deliberately
restored/restarted. Persist session definitions separately from live process state and offer clear
restart policy instead of false guarantees." `full.md` §1.3:120 states the principle directly: "**No fake
persistence.** SQLite metadata alone cannot restore a live PTY after a server crash. State guarantees
must reflect implemented supervision and tested recovery."

`full.md` §6.4:490 requires a durable metadata store: "Use SQLite or another small transactional store
for durable metadata after evaluating migration, locking, and test needs. Persist identities, labels,
layouts, host-independent session metadata, preferences, and selected audit/event data. Do not store
unbounded PTY output as ordinary rows." `full.md` §6.4:492–498 lists what must be defined: "DB schema and
migrations", "Retention and pruning", "Crash-consistent writes", "Event revisions and snapshot
consistency", "What is restored after a process/server restart", "Backup/export behavior if needed".
`full.md` §6.4:500 requires bounded scrollback and queues. `full.md` §12:799 restates "bounded scrollback
and queues" and `full.md` §12:804 requires "keep terminal byte streams out of metadata DB".

`full.md` §7.1 fixes filesystem conventions (`:508`–`:519`): per-user persistent state at
`$XDG_STATE_HOME/relayable`, default `~/.local/state/relayable/` (`:513`); runtime/socket at
`$XDG_RUNTIME_DIR/relayable/` (`:514`); `/etc/relayable/` (`:512`) and `/run/relayable/` (`:515`) only for
a system deployment; and "Never assume `$HOME` or `$XDG_RUNTIME_DIR` exists before checking service
context" (`:519`). `full.md` §7.2:523 chooses the workstation default ("`systemd --user` server, running
under the logged-in user's identity … owns that user's sessions and executes processes with that user's
normal permissions"). `full.md` §7.2:525 makes the remote/host-service mode optional and warns that a
single `relayable` daemon account "must not silently impersonate all remote users or run everybody's
commands under that account. Per-user workers/services may be required." `full.md` §11:766 imposes the
same constraint as a release gate ("Implement a deliberate per-user execution model or scope initial
release to one user per server"). `full.md` §11:771 adds: "package upgrades must not silently kill
sessions; define how the service drains/restarts or preserves session supervision." `full.md` §11:762
requires a restrictive local socket; `full.md` §11:770 forbids logging private terminal output, tokens,
keys, or full command arguments by default. `full.md` §5.2:408 places persistence composition in the
server crate; `full.md` §5.2:412 forbids crate cycles.

### What the implementation already does

The current server is a single foreground binary — `crates/relayable-server/src/main.rs:3`: "Usage:
`relayable-server run` (foreground, no systemd unit, no forking)." It owns every child **in-process**:

- `crates/relayable-server/src/session.rs:1-7` (module doc, verbatim): "A [`BlockRuntime`] owns the
  server-side PTY child for the lifetime of the session, independently of any client connection: it holds
  the [`PtyHandle`], a dedicated PTY-writer thread, the bounded output tail, the per-stream output
  sequence, and the fan-out to attached streams. Dropping a client never signals, reaps, or stops the
  child; only [`BlockRuntime::close`] does."
- `session.rs:292` defines `pub(crate) struct BlockRuntime`; it holds `output: Mutex<OutputState>`
  (`:301`), `writer_lease: Mutex<Option<WriterLease>>` (`:304`), and `pty: Mutex<PtyHandle>` (`:305`),
  plus reader/reaper/input thread join handles.
- `crates/relayable-server/src/registry.rs:56` holds `sessions: Mutex<BTreeMap<SessionId,
  Arc<SessionRuntime>>>` in `ServerShared`; `create_session` (`:86`) spawns the child at `:98` via
  `relayable_pty::PtyHandle::spawn(...)`, with the doc at `:84-85`: "The child is owned by the registry,
  never by a client."
- `session.rs:903` defines `SessionRuntime` (`label: Option<String>` at `:905`, `created_at_millis` at
  `:906`), and `:902` notes: "Closed sessions are retained as tombstones (ADR-001)."
- The OS handle type is opaque: `crates/relayable-pty/src/lib.rs:121` `pub struct PtyHandle { master:
  Box<dyn MasterPty + Send>, child: Box<dyn Child + Send + Sync>, … }`; spawn is `lib.rs:148`; the child
  becomes a session leader because `portable-pty` calls `setsid()`/`TIOCSCTTY` (`lib.rs:169-176`);
  termination is `try_wait` (`lib.rs:236`), `kill` (`lib.rs:270`), `shutdown(grace)` (`lib.rs:292`).

Client teardown never touches the child. `crates/relayable-server/src/lib.rs:1-6`: "A dropped client
never signals, reaps, or stops a child; only `session.close` (or graceful server shutdown) does."
`crates/relayable-server/src/connection.rs:312`: "Tear down the transport only: the session, block, PTY,
and child stay up." `connection.rs:79-80` (`close_now`): "…The session and child are untouched."

Graceful shutdown does terminate children. `lib.rs:123-135` requests shutdown, closes connections with
`DisconnectReason::ServerShutdown`, closes every live block (`session.block.close(500 ms)`), joins
connection threads, and unlinks the socket. Signal handling is limited to SIGINT/SIGTERM:
`crates/relayable-server/src/signals.rs:19-32` installs handlers that are "async-signal-safe" and only
store `true` in an `AtomicBool` (`:40-42`, store at `:41`); there is no `SIGHUP` handler, no `sd_notify`, and no SIGKILL
path.

Sessions and all metadata are **in memory only**. There is no database, no state directory, and no
serde-persisted state: grep over all `Cargo.toml` manifests finds no `rusqlite`, `sqlx`, `sled`, or
`redb` (verified — none present); `crates/relayable-server/Cargo.toml:13-23` lists only `relayable-model`,
`relayable-protocol`, `relayable-pty`, `anyhow`, and `libc = "=0.2.190"` ("# Signal handling only.").
`std::fs` appears in production server code only for `/proc` reads and socket unlink. No systemd unit
exists (`scripts/ci.sh:197` skips "package build tests (deb/rpm/systemd packaging)" because no recipe
exists; `docs/operations/toolchain-audit.md:747` gap G5: "No relayable systemd unit exists; service
model … undecided … ADR for service/persistence model (full.md §7.2) before writing any unit").

The protocol layer already states the same non-claims. `protocol/compatibility/core-001-method-set.md`
§9 (`:127-140`) lists "**Persistence**: none; no tombstones, no retention/pruning, no metadata store",
"**Authn/authz**: no mechanism", and "**Peer-credential checks** (`SO_PEERCRED`) …: deferred";
`:219` says "Crash/reboot survival is **not** provided or claimed."
`protocol/compatibility/core-002-method-set.md` §G (`:195-205`) lists "No server-crash or host-reboot
session survival", "Terminal output is **not** byte-exact replayable", and "No persistence / SQLite".
The frozen `docs/architecture/interface-contract.md:93-95` says "No crash or host-reboot session survival
is claimed … a restored session definition is a restart, not a resurrected process."

### Constraints already fixed by prior ADRs

ADR-001 fixed the domain model and dependency direction. Notably: host profiles are client-local and
"never a server-side identity" (`ADR-001:86`); Tombstone/audit writes "records a tombstone (ID, destroy
sequence, timestamp) in the metadata store when persistence is enabled", and "[t]he metadata store never
holds terminal bytes", with crash/host-reboot survival explicitly not claimed (`ADR-001:286-291`); event
retention is bounded in memory with a configurable floor, and "[t]here is no persisted event log in the
first release" (`ADR-001:230-236`); "Relayable **never claims** byte-exact recovery of the terminal byte
stream" (`ADR-001:239`); the server crate "Composes model, protocol, pty" and composes persistence
(`ADR-001:337,361`); `Cargo.toml` is Lead-owned so a worker cannot change the graph (`ADR-001:383-384`); and
"[n]o persisted event log means no resume across server restart, consistent with `full.md` §6.3:480"
(`ADR-001:400`). ADR-001 explicitly deferred the exact tombstone schema to persistence (`ADR-001:433`:
"**Tombstone schema.** Exact metadata-store schema and pruning for tombstones belong with persistence.")
and flagged a reconsider-if trigger: "[a] persisted event log becomes required for resume across server
restart" (`ADR-001:417`).

ADR-002 fixed the local transport as a pathname Unix-domain socket under
`$XDG_RUNTIME_DIR/relayable/` with restrictive mode, `/run/relayable/` reserved for a system service,
and noted peer credentials (`SO_PEERCRED`) are connection-scoped and not yet used (`ADR-002:251-259`).
ADR-003 owns engine pinning; ADR-005 governs the `RELAYABLE_*` environment-variable namespace (DEC-006)
and must be honored by any new configuration names.

---

## Options Considered

### Process ownership / survival across server restart

`full.md` §6.3:480–484 names three admissible mechanisms. All three are evaluated against the
first-release constraint that the implementation is a single foreground server
(`crates/relayable-server/src/main.rs:3`) on the verified target (Linux, `systemd --user` available,
`docs/operations/toolchain-audit.md`; systemd user manager present, no relayable unit).

| Option | What it would require | Cost / risk | Android-readiness (DEC-010) | First-release verdict |
|---|---|---|---|---|
| **Status quo: in-process ownership** (adopted) | Nothing beyond what exists: `Arc<BlockRuntime>` holds the `PtyHandle` and child, per session, independent of connections (`session.rs:1-7,292-305`; `registry.rs:56,84-85`). | None today; a server restart/Host reboot ends children (documented). | Host-independent; no POSIX supervisor assumed. | **Chosen.** Satisfies §6.3:475–478; makes no §6.3:480 promise. |
| **(A) Separate supervisor process owns PTYs and reattaches after a server restart** (`full.md:482`) | A second privileged-adjacent process; PTY master fds must outlive the server and be handed to a restarted server — fd passing over a `SCM_RIGHTS` Unix socket; a versioned lifecycle/handshake protocol between server and supervisor; separate packaging, systemd unit, and supervision; failure-mode design (supervisor crash kills sessions); security review of the supervisor's trust boundary (`full.md` §11:760). | High: an entirely new OS-level component with its own protocol, tests, and security surface — comparable in effort to the server itself. `full.md` §6.3:480 makes it optional; no spike exists. | Adds a process/socket model Android may not be able to host; must not leak into the model/protocol (model must stay host-independent, `full.md` §5.2:405). | **Deferred.** No implemented, tested spike; meets none of the first-release acceptance criteria without new evidence. |
| **(B) Per-session supervisor services/processes preserve PTY descriptors** (`full.md:483`) | One process/service per live session; transient systemd units or an equivalent launcher; descriptor custody and reattachment; lifecycle protocol; per-unit state/permissions; packaging (`TimeoutStopSec`, `Restart`, ownership) and security review; unbounded per-session process growth risk. | Highest operational complexity; a unit per session is a large surface with many failure modes; `full.md` §12:800 ("avoid one unbounded task per client/event") counsels against unbounded per-session machinery without measurement. | Worst Android fit: assumes a systemd-like per-user/per-session supervisor that does not exist on Android. | **Deferred.** |
| **(C) Another proven OS-level arrangement** (`full.md:484`) | Any other tested OS mechanism (e.g., a broker); must be "proven" per §6.3:480 and tested. | Undefined until a specific mechanism is proposed with evidence. | Unknown. | **Deferred** — no candidate with tested evidence exists. |

### Persistence store

`full.md` §6.4:490 names SQLite first and permits "another small transactional store … after evaluating
migration, locking, and test needs." The store is metadata/audit only; it never holds terminal bytes
(`full.md` §6.4:490, §12:804; `ADR-001:239,286-291`).

| Option | Migration | Locking / concurrency | Test needs | Verdict |
|---|---|---|---|---|
| **SQLite (in-process, single writer)** | Numbered, versioned SQL migrations in a `schema_migrations` table or `PRAGMA user_version`; SQL is inspectable and testable. | One writer serialized inside the server; `PRAGMA journal_mode=WAL`, `busy_timeout`, and `synchronous`; a `flock` single-instance guard proves no second writer. | `tempfile` (already a workspace dependency, `Cargo.toml:29-44`) provides isolated DBs per test; the server is already sync/threads-only (root `Cargo.toml` design note), so a synchronous driver fits. | **Chosen.** Best migration + locking + test story of the candidates; engine is small and embedded. |
| `sled` / `redb` (embedded KV) | No SQL migration story; callers evolve key/value encodings by hand and must version them. | Embedded key-value; `sled` is beta-quality, so durability/locking guarantees are harder to justify as a release gate. | Workable with `tempfile`, but no query layer for schema evolution or ad-hoc inspection. | Rejected: weaker migration/query story and (for `sled`) beta maturity. |
| `sqlx` (async driver) | SQL migrations supported. | Async pool is unnecessary for one synchronous writer and pulls async/tokio into an explicitly sync-only workspace (root `Cargo.toml` design note). | Adds build weight and a second execution model for no first-release benefit. | Rejected: async + build weight for a single-writer store. |
| Plain files / append-only JSON | No schema versioning or migration mechanism beyond ad-hoc rewrites. | No transactions or locking; a torn write cannot be made crash-consistent. | Hard to test crash consistency. | Rejected: cannot give "crash-consistent writes" (`full.md` §6.4:495). |

**Binding choice.** The concrete Rust binding (for example `rusqlite` with the `bundled` feature versus a
system libsqlite3) is **Relayable-defined (no REX evidence)** and is a build decision to be made when the
store crate is created. It is not made here because Android must be able to host the same storage
interface without breaking DEC-010 (`docs/agent-loop/DECISIONS.md:17`), and no storage crate is currently
a dependency (verified: no `rusqlite`/`sqlx`/`sled`/`redb` in any manifest).

---

## Decision

### Decision 1 — Process ownership: the foreground server owns the child tree, in-process

**D1.1 (ownership boundary).** The foreground server process owns every PTY/child tree. Ownership is
**per session, not per connection**, and is held in-process by `Arc<BlockRuntime>` inside
`ServerShared.sessions` (`crates/relayable-server/src/session.rs:1-7,292-305`;
`crates/relayable-server/src/registry.rs:56,84-85,98`). The PTY handle is opaque and non-`Clone`
(`crates/relayable-pty/src/lib.rs:121,148,169-176,236,270,292`). No out-of-process supervisor exists.

**D1.2 (guaranteed, implemented and tested).** Closing a client window, losing the network, an idle
timeout, a mid-request drop, or a flow-control overflow removes only the client record, its streams, and
its writer lease. It never signals, reaps, or stops the child
(`crates/relayable-server/src/lib.rs:1-6`; `connection.rs:79-80,312-315`). Reattaching preserves the same
`pty_process_id`/`program_pid` and the same child. This is exercised by the test suites (function names
and verified locations):

- `server_end_to_end` (`crates/relayable-server/tests/server_e2e.rs:508`),
- `disconnected_client_record_is_removed` (`server_e2e.rs:1319`),
- `idle_timeout_reason_is_reported_to_a_subscriber` (`server_e2e.rs:1392`),
- `reconnect_preserves_identity_and_advances_the_stream` (`tests/reconnect/main.rs:42`),
- `snapshot_at_s_plus_events_has_no_gap_no_duplicate_and_the_child_pid_survives`
  (`tests/reconnect/main.rs:188`),
- `resume_without_a_snapshot_continues_the_event_stream` (`tests/reconnect/main.rs:308`),
- `core_001_vertical_slice` (`tests/lifecycle/main.rs:102`),
- `lease_release_on_disconnect_is_announced` (`tests/multi-client/main.rs:227`),
- `slow_client_output_overflow_detaches_with_flow_control_overflow` (`tests/flow-control/main.rs:131`),
- `slow_client_receives_at_most_credit_window_bytes_before_overflow`
  (`tests/flow-control/main.rs:467`).

**D1.3 (guaranteed, implemented and tested).** Only `session.close` and graceful server shutdown
terminate children. Shutdown installs SIGINT/SIGTERM handlers
(`crates/relayable-server/src/signals.rs:20-44`) and the accept loop exits at
`crates/relayable-server/src/lib.rs:123-135`, which closes every live block and reaps the children. Test:
`graceful_shutdown_kills_live_children` (`crates/relayable-server/tests/server_e2e.rs:869`).

**D1.4 (supervision options deferred).** All three mechanisms named at `full.md` §6.3:480–484 — "(A) a
separate supervisor owns PTYs and allows server restarts to reattach" (`:482`), "(B) per-session
supervisor services/processes preserve PTY descriptors" (`:483`), "(C) another proven OS-level
arrangement" (`:484`) — are **deferred** from the first release. The first release runs one foreground
server owning PTYs in-process (`crates/relayable-server/src/main.rs:3`). Each option's requirements and
risks are in the Options table above; none is implemented, tested, packaged, or security-reviewed. This
decision may be revisited via the reconsider-if triggers (1), (2), or (7) below.

**D1.5 (service-level restarts are not session survival).** A future systemd user unit may use
`Restart=on-failure` with bounded backoff (`full.md` §7.2:531: "`Restart=on-failure` and backoff without
restart loops") to restart the **server process**, never sessions. User lingering is an opt-in
post-logout availability choice, not session survival (`full.md` §7.2:535). Socket activation is used
"only if justified and correctly implemented" (`full.md` §7.2:534). No unit exists today; installing or
enabling a service is gated on explicit user approval (`full.md` §7.2:537, §11:775) and any future unit
must inspect real CLI options first (`full.md` §7.2:539).

**D1.6 (confirms ADR-001).** ADR-001 already fixed "[n]o persisted event log means no resume across
server restart" (`ADR-001:400`); this ADR confirms that and does not reopen it. The corresponding
reconsider-if trigger remains ADR-001:417.

**D1.7 (plain non-claim).** **Server-crash and host-reboot session survival is NOT implemented and NOT
claimed** (`full.md` §1.3:120, §6.3:480, §6.3:486; `protocol/compatibility/core-001-method-set.md:219`;
`protocol/compatibility/core-002-method-set.md:197`; `docs/architecture/interface-contract.md:93-95`). SIGKILL
and host power-loss orphan behaviour is **untested and unclaimed**: no test kills the server with SIGKILL
and no orphan/adoption policy is documented. No systemd unit exists (`crates/relayable-server/src/main.rs:3`;
`scripts/ci.sh:197`; `docs/operations/toolchain-audit.md:747`).

**D1.8 (upgrade continuity, stated honestly).** `full.md` §11:771 requires that "package upgrades must
not silently kill sessions." **Today they do end live children**, because a restart or upgrade terminates
the foreground server and graceful shutdown reaps every live child (`crates/relayable-server/src/lib.rs:123-135`;
test `graceful_shutdown_kills_live_children`, `server_e2e.rs:869`). This is a documented known gap, not a
hidden one. The drain story for the first release is: a graceful shutdown ends children deliberately;
clients observe the state change through the event stream (`terminal_child_exited` and session/connection
state events, `protocol/compatibility/core-002-method-set.md` §D) and through `session.view`/`session.list`,
and persisted session definitions (Decision 2) can be presented as an explicit restart action. Silent
death is prevented by documentation and explicit, user-visible state — **not** by supervision. Upgrade
continuity is recorded as future work and as reconsider-if trigger (2) below.

### Decision 2 — Persistence: SQLite metadata and audit, no terminal bytes

**D2.1 (store).** Relayable adopts **SQLite, accessed in-process by a single writer**, as the durable
metadata/audit store required by `full.md` §6.4:490. The store is metadata/audit only; it never holds
terminal bytes (`full.md` §6.4:490, §12:804; `ADR-001:239,286-291`). The engine choice is justified
against `sled`/`redb`, `sqlx`, and plain files in the Options table. The concrete binding is
**Relayable-defined (no REX evidence)** and is fixed when the crate is created, subject to DEC-010.

**D2.2 (crate boundary).** Persistence lives behind a new `crates/relayable-store` crate so the engine is
replaceable. `crates/relayable-server` composes persistence (`full.md` §5.2:408; `ADR-001:337,361`). The
workspace graph is Lead-owned (`ADR-001:383-384`), so creating the crate and adding it to the workspace is
Lead-integrated future work; no crate cycles are permitted (`full.md` §5.2:412). This is a deliberate
extension of ADR-001's fixed day-one crate list: `crates/relayable-store` is a new crate boundary
beyond that enumeration, while `ADR-001:337` continues to place persistence *composition* inside
`relayable-server` (the workspace graph remains Lead-owned).

**D2.3 (schema — Relayable-defined (no REX evidence)).** The first-release schema is host-independent by
construction: it holds **no pids, no file descriptors, no hostnames, no peer credentials, and no host
identity**, consistent with `ADR-001:86` (host profiles are client-local). The schema is:

| Table | Columns (purpose) | Notes |
|---|---|---|
| `schema_migrations` | `version`, `applied_at` | Alternative: `PRAGMA user_version`. Both are Relayable-defined (no REX evidence). |
| `sessions` | `session_id` (UUID PK), `label`, `created_at_millis`, `closed_at_millis`, `state` (`active` \| `closed` \| `interrupted`), `last_known_revision` | Session definition, not live process state (`full.md` §6.3:486). |
| `windows` / `blocks` **or** `session_layout` | Opaque, schema-versioned layout document (per window: `block_id` + split ratios) | Supplies the "layouts" metadata `full.md` §6.4:490 requires; shape aligns with `interface-contract.md:72-82` (`layout`). |
| `tombstones` | `session_id`, `destroy_sequence`, `timestamp` | **Closes the deferral at `ADR-001:433`**; matches the tombstone `ADR-001:286-291` describes and the "no terminal bytes" rule (`ADR-001:89`). |
| `identities` | `client_id`, display name, timestamps — **display records only** | No credentials: there is no authn in the first release (`protocol/compatibility/core-001-method-set.md:127-140`; `core-002-method-set.md:195-205`), so **no tokens, keys, or credential material are persisted** (`full.md` §11:764, §11:770). |
| `preferences` | Versioned key/value, scoped per user, host-independent | Per `full.md` §6.4:490 ("preferences"). |
| `audit_events` | Session lifecycle, session state changes, writer-lease changes, destroys | Only `full.md` §6.4:490's "selected audit/event data". |

**Absolutely NOT stored as rows:** PTY output, scrollback, or any terminal byte stream (`full.md` §6.4:490
"Do not store unbounded PTY output as ordinary rows"; `full.md` §12:804 "keep terminal byte streams out of
metadata DB"; `ADR-001:68,239`). Also never persisted by default: screen contents, input bytes,
environment values, and full command arguments (`full.md` §11:770). The tombstone/audit trail is audit
data, not a replay log (`ADR-001:400`).

**D2.4 (bounds and retention).** Memory bounds already implemented and **preserved** (they are load-bearing
for `full.md` §6.4:500, §12:799–805):

| Bound | Value | Anchor |
|---|---|---|
| Input queue messages per block | `INPUT_QUEUE_MESSAGES = 1024` | `crates/relayable-server/src/session.rs:29` |
| Input queue bytes per block | `INPUT_QUEUE_BYTES = 4 * 1024 * 1024` | `session.rs:31` |
| Retained output tail per block (`output_tail_bytes`) | default `32_768` | `crates/relayable-server/src/config.rs:41,126` |
| Outbound queue messages per connection | `OUTBOUND_QUEUE_MESSAGES = 8192` | `crates/relayable-server/src/connection.rs:38` |
| Duplicate-request dedup window | 64 | `connection.rs:281` (`RequestWindow::new(64)`) |
| Retained control events (`RELAYABLE_EVENT_RETENTION_EVENTS`) | default `4096` | `crates/relayable-server/src/config.rs:23`; `crates/relayable-server/src/events.rs:141-153,201` |
| Per-stream credit window | `262_144` bytes | `crates/relayable-protocol/src/limits.rs:14-20` |

Only `output_tail_bytes` and `event_retention_events` are `ServerConfig` parameters (the latter
overridable via `RELAYABLE_EVENT_RETENTION_EVENTS`); the remaining bounds are hard-coded `const`s —
`INPUT_QUEUE_MESSAGES`/`INPUT_QUEUE_BYTES` (`session.rs:29,31`), `OUTBOUND_QUEUE_MESSAGES`
(`connection.rs:38`), the 64-entry dedup window (`connection.rs:281`), `THREAD_JOIN_TIMEOUT`
(`session.rs:33`), and `credit_window_bytes` from the protocol `DEFAULT_LIMITS` (`limits.rs:14-20`).
These may be promoted to configuration later (`ADR-001:230-236`); any new name
lives in the ADR-005 `RELAYABLE_*` namespace, and the exact new name is fixed at implementation under
ADR-005 (this ADR does not invent one). Server-side scrollback is **NOT PRESENT** today:
`crates/relayable-server/src/session.rs:470` sets `scrollback_bytes: Vec::new()` in `attach_stream`
(consistent with `core-001-method-set.md:136-137`). Today's scrollback is client-engine-owned
(`apps/linux/src/term/mod.rs:31,164`). The persisted metadata has its own bounds: `audit_events`,
`tombstones`, and closed sessions are pruned by **age and row count** on startup and on close, so DB
growth is bounded by construction (`full.md` §6.4:494 "Retention and pruning"). The concrete age/row
defaults are **Relayable-defined (no REX evidence)** configuration, to be measured (`full.md` §12:796)
and named under ADR-005.

**D2.5 (crash-consistent writes).** Crash consistency (`full.md` §6.4:495) is defined by:
`PRAGMA journal_mode=WAL`; an explicit `synchronous` policy (`NORMAL` in WAL mode as the default
performance/durability trade-off, with `FULL` available for stricter durability); one writer serialized
through the server's store task; one transaction per logical operation; `busy_timeout` set so contention
cannot fail spuriously; and a `flock`-based single-instance guard so a second server cannot become a
second writer. The WAL/`synchronous` policy, the single store task, one-transaction-per-operation, the
`busy_timeout` value, the `flock` guard, and startup reconciliation are all
**Relayable-defined (no REX evidence)** design choices. On startup, the server performs explicit
reconciliation:
any row left in state `active` is marked `interrupted`, because the process that owned it is gone — it is
**never** silently reported as live (`full.md` §6.3:486 "offer clear restart policy instead of false
guarantees"; `ADR-001:286-291`). Memory remains authoritative for live state; the DB never substitutes
for a live child.

**D2.6 (event revisions and snapshot consistency).** Per-entity `revision: u64` and the server-wide
monotonic `sequence: u64` stay exactly as frozen in `docs/architecture/interface-contract.md:63-92`. The
server `sequence` is **per server run and NOT durable**: a restarted server starts a new epoch, so
clients must discard cursors and take a fresh snapshot with a `snapshot_required` reason
(`interface-contract.md:88`; `ADR-001:398`). `session.resume` against a session that is not live on the
current epoch must fail with a documented reason rather than fake a reattach.

**D2.7 (what is restored after a process/server restart).** Required by `full.md` §6.4:497.

| Restored | Not restored |
|---|---|
| Session definitions: `session_id`, `label`, timestamps, `state` | The live child/PTY and its pids |
| Layout metadata (windows/blocks/split ratios) | The writer lease |
| Identities (display records only) | Connected clients |
| Preferences | Output tail / scrollback |
| Tombstones | Byte-exact terminal screen |
| Audit trail | Event-sequence continuity (new epoch) |

A restored session definition **is a restart, not a resurrected process** (`full.md` §6.3:486;
`docs/architecture/interface-contract.md:93-95`). The protocol and UI must present restoring a definition as
an explicit **restart action**, never as a silent reattach.

**D2.8 (storage location and backup/export).** Per-user persistent state lives at
`$XDG_STATE_HOME/relayable`, default `~/.local/state/relayable/` (`full.md` §7.1:513). The runtime/socket
stays at `$XDG_RUNTIME_DIR/relayable/` (`full.md` §7.1:514; `ADR-002:251-259`). `/etc/relayable/`
(`full.md` §7.1:512) and `/run/relayable/` (`full.md` §7.1:515) apply only to a system deployment. The
system-mode state directory (FHS `/var/lib/relayable`) is **Relayable-defined (no REX evidence)** and
applies only to the gated system mode. Directories are resolved and validated at startup without assuming
`$HOME` or `$XDG_RUNTIME_DIR` exist (`full.md` §7.1:519). Backup/export: an online-consistent copy via
the SQLite backup API or `VACUUM INTO`, plus a schema-versioned, host-independent JSON/NDJSON export
(**Relayable-defined (no REX evidence)**). Import/restore is out of scope for the first release. Exports
contain no terminal bytes because none are stored.

### Decision 3 — Multi-user and identity scope: one user per server

**D3.1.** The first release serves **one user per server**. The default deployment is `systemd --user`
under the logged-in user's identity, owning that user's sessions and running children with that user's
normal permissions (`full.md` §7.2:523), reachable over a user-owned Unix socket with restrictive mode
(`full.md` §11:762; `ADR-002:251-259`).

**D3.2.** System-service mode is explicitly **not implemented and gated**. An optional system unit is
permitted "only after security review" (`full.md` §7.2:529). When it is implemented it must use a
deliberate per-user execution model (per-user workers/services running as the target user), because "a
daemon running as one `relayable` account must not silently impersonate all remote users or run
everybody's commands under that account" (`full.md` §7.2:525; `full.md` §11:766). That mode requires
peer-credential checks (`SO_PEERCRED`) and an authz mechanism, neither of which exists: `SO_PEERCRED` is
deferred (`protocol/compatibility/core-001-method-set.md:138`) and there is "no authn/authz"
(`core-002-method-set.md:195-205`). `full.md` §11:766 permits the first-release scope to be "one user per
server" instead.

**D3.3.** Consequence: no cross-user session visibility and no impersonation is claimed. Persisted
identities are display records only and carry no credentials (D2.3).

**D3.4 (Android readiness, DEC-010).** The ownership/identity model must not assume POSIX uid
multiplexing or a system daemon. The session and identity types stay host-independent and usable in a
single-app-uid deployment, and no protocol field may depend on a host uid (`ADR-001:86`;
`docs/agent-loop/DECISIONS.md:17`). This is why no pid/fd/host identity is persisted (D2.3) and why the
supervisor options are evaluated against Android in the Options table.

---

## Explicit non-guarantees

| Property | Status | Authority |
|---|---|---|
| Session/PTY survival across client close or network loss | **Guaranteed — implemented and tested** | `full.md` §6.3:475–477; `crates/relayable-server/src/lib.rs:1-6`; tests `server_e2e.rs:508,1319,1392`, `tests/reconnect/main.rs:42,188,308`, `tests/lifecycle/main.rs:102`, `tests/multi-client/main.rs:227`, `tests/flow-control/main.rs:131,467` |
| Survival across server crash or host reboot | **NOT implemented, NOT claimed** | `full.md` §1.3:120, §6.3:480, §6.3:486, §6.4:497; `core-001-method-set.md:219`; `core-002-method-set.md:197`; `interface-contract.md:93-95` |
| Byte-exact terminal stream recovery | **Not claimed** | `full.md` §6.1:460; `ADR-001:239`; `interface-contract.md:88-92` |
| Event-sequence continuity across a restart | **Not claimed** (new per-run epoch) | D2.6; `interface-contract.md:63-92` |
| Upgrade/restart continuity (children preserved) | **NOT implemented** — children end on restart | `full.md` §11:771; `crates/relayable-server/src/lib.rs:123-135`; test `server_e2e.rs:869` |
| SIGKILL / host power-loss orphan behaviour | **Untested / unclaimed** | D1.7; no SIGKILL or orphan test exists |
| Multi-user isolation, authn, and authz | **NOT implemented** | `full.md` §11:766; `core-001-method-set.md:127-140`; `core-002-method-set.md:195-205` |
| systemd unit | **Not present** | `crates/relayable-server/src/main.rs:3`; `scripts/ci.sh:197`; `docs/operations/toolchain-audit.md:747` |
| SQLite persistence store | **Designed here; not implemented** | This ADR, Decision 2; verified: no storage crate in any manifest |
| Unbounded terminal output retained in the metadata DB | **Never** — no terminal bytes are persisted | `full.md` §6.4:490, §12:804; `ADR-001:239,286-291` |

---

## Consequences

- **The first-release guarantee set is exactly `full.md` §6.3:475–478.** Everything beyond it is
  explicitly not claimed, satisfying `full.md` §1.3:120 ("No fake persistence … State guarantees must
  reflect implemented supervision and tested recovery") and `full.md` §6.3:486 ("clear restart policy
  instead of false guarantees").
- **Persistence is decoupled from liveness.** A restarted server can list past session definitions and
  offer an explicit restart, but it cannot resurrect a process. This keeps the DB small, host-independent,
  and cheap to prune, and keeps live state authoritative in memory.
- **The store is an audit/metadata layer, never a replay log.** The bound is structural: no terminal
  bytes means no path by which the DB can be mistaken for a terminal buffer. Reconnect/replay semantics
  remain the in-memory event bus plus fresh snapshots (`interface-contract.md:88`).
- **Upgrades remain a known limitation.** Until a supervisor mechanism is proven, an upgrade restarts the
  server and ends its children; the honest mitigation is user-visible state and a documented restart,
  not a false continuity promise (`full.md` §11:771).
- **One user per server is a deliberate scope reduction, not an oversight.** It is the option
  `full.md` §11:766 permits when no per-user execution model exists, and it avoids a credential surface
  (no `SO_PEERCRED`, no authz) that is not implemented.
- **`crates/relayable-store` is a new build dependency surface.** Adding it changes the workspace graph,
  which is Lead-owned (`ADR-001:383-384`); the binding choice must preserve Android hostability (DEC-010).
- **Cross-cutting, not local.** Creating the store touches server startup/shutdown ordering
  (`crates/relayable-server/src/signals.rs` shutdown path), configuration naming (ADR-005), and the
  protocol/UI restart action; these are listed as implied future work.

## What Evidence Would Cause Reconsideration

1. **A supervision spike wins.** A supervisor or per-session-supervisor spike demonstrates a live PTY
   reattaching across a server restart on the target kernel/systemd, with passing tests — this reopens
   D1.4 and the `full.md` §6.3:480 guarantee set.
2. **Reboot/upgrade continuity becomes a release gate,** or `full.md` §11:771 is exercised by a real
   release and the documented child-ending behavior is judged insufficient.
3. **A second user must be served by one daemon.** That forces the per-user worker/service model plus
   `SO_PEERCRED` (deferred, `core-001-method-set.md:138`) and an authz mechanism, reopening D3.1/D3.2.
4. **SQLite proves wrong.** Measured migration, locking, or test problems with the chosen binding, or
   Android cannot host it without breaking DEC-010 (`docs/agent-loop/DECISIONS.md:17`). The store engine
   or binding is revisited.
5. **Measured bounds contradict this ADR.** Retention/scrollback defaults measured per `full.md` §12:796
   contradict the bounds in D2.4 or the pruning policy.
6. **Orphans appear.** SIGKILL/orphan tests produce children that survive unaccounted for, contradicting
   D1.7 and requiring an explicit orphan policy.
7. **A persisted event log becomes required** for replay/resume across server restart — this is ADR-001's
   own trigger (`ADR-001:417`) and would change the guarantee set.

## Implied future work (not done)

Each item below is **NOT DONE** and is recorded only so it is not presented as implemented.

- [ ] **NOT DONE** — Create `crates/relayable-store` behind which the engine is replaceable.
- [ ] **NOT DONE** — Add the store dependency to the workspace graph (`Cargo.toml`, Lead-owned,
      `ADR-001:383-384`).
- [ ] **NOT DONE** — Server startup reconciliation (mark stale `active` rows `interrupted`) and the
      single-instance `flock` guard.
- [ ] **NOT DONE** — Write path for sessions, layout, identities, preferences, tombstones, and audit in
      `crates/relayable-server`.
- [ ] **NOT DONE** — Prune task for `audit_events`/`tombstones`/closed sessions by age and row count.
- [ ] **NOT DONE** — Startup/shutdown ordering against the existing `crates/relayable-server/src/signals.rs`
      shutdown path.
- [ ] **NOT DONE** — `README`/docs for the state directory (`full.md` §7.1) and backup/export procedure.
- [ ] **NOT DONE** — Product-level "restart session" surface (protocol + UI), which must present a restart,
      not a resurrected process (D2.7).
- [ ] **NOT DONE** — Optional, gated systemd user unit; must inspect real CLI options first
      (`full.md` §7.2:539) and is gated on user approval (`full.md` §7.2:537, §11:775).
- [ ] **NOT DONE** — Any supervisor option from D1.4, if reconsider-if (1) fires.
- [ ] **NOT DONE** — `SO_PEERCRED` and an authz mechanism for any multi-user mode.
- [ ] **NOT DONE** — Exact environment-variable names for store/retention settings, fixed at
      implementation under ADR-005 (this ADR fixes none).

## Evidence

Evidence classes used: **spec anchors**; **code read** (`crates/relayable-server/**`,
`crates/relayable-pty/src/lib.rs`, `crates/relayable-protocol/src/limits.rs`, `apps/linux/src/term/mod.rs`,
`Cargo.toml`); **tests read** (function names and line anchors in
`crates/relayable-server/tests/server_e2e.rs`, `tests/reconnect/main.rs`, `tests/lifecycle/main.rs`,
`tests/multi-client/main.rs`, `tests/flow-control/main.rs`, `tests/events/main.rs`); and **commands
actually run** (git state, `grep` over manifests for storage crates, line-anchor reads). No cargo test run
was performed because no code changed.

| Anchor | What it establishes | Class |
|---|---|---|
| `full.md` §1.3:111, §1.3:120 | Client-independent session lifetime; "No fake persistence" | Spec |
| `full.md` §6.3:475–478 | First-release guarantees (close/disconnect/reattach/explicit stop) | Spec |
| `full.md` §6.3:480, §6.3:482–484 | No crash/reboot promise; the three supervision mechanisms | Spec |
| `full.md` §6.3:486 | Persist definitions separately; clear restart policy | Spec |
| `full.md` §6.4:490, §6.4:492–498, §6.4:500 | SQLite-or-similar metadata; define list; bounded scrollback/queues | Spec |
| `full.md` §7.1:512–519 | XDG paths; `/etc` & `/run` system-only; never assume `$HOME`/`$XDG_RUNTIME_DIR` | Spec |
| `full.md` §7.2:523, §7.2:525, §7.2:527–535 | `systemd --user` default; per-user workers for system mode; test list | Spec |
| `full.md` §11:760, §11:762, §11:764, §11:766, §11:770, §11:771 | Security release gate; socket perms; one-user scope; logging; upgrade continuity | Spec |
| `full.md` §12:799, §12:804, §12:805 | Bounded queues/scrollback; no terminal bytes in metadata DB; idle vs high-output | Spec |
| `full.md` §5.2:405, §5.2:408, §5.2:412, §5.2:414 | Model purity; server composes persistence; no cycles; ADR content | Spec |
| `ADR-001:86,89,239,286-291,337,361,383-384,398,400,417,433` | Host profiles client-local; tombstone/no terminal bytes; no byte-exact recovery; crate roles; Lead-owned graph; snapshot fallback; no persisted event log; reconsider-if; tombstone-schema deferral to persistence | Spec/ADR |
| `ADR-002:251-259` | Pathname Unix socket under `$XDG_RUNTIME_DIR/relayable/`; `/run` system-only; `SO_PEERCRED` connection-scoped/deferred | ADR |
| `docs/architecture/interface-contract.md:63-92` | Snapshot schema; `revision`/`sequence`; `snapshot_required`; "a restored session definition is a restart" | Spec |
| `docs/architecture/interface-contract.md:150-180` | Connection state model; transport drop never destroys the session | Spec |
| `protocol/compatibility/core-001-method-set.md:127-140,210-219` | Persistence none, authn none, `SO_PEERCRED` deferred; disconnect removes only client records; crash/reboot not claimed | Spec |
| `protocol/compatibility/core-002-method-set.md:195-205` | No crash/reboot survival; no byte-exact replay; no persistence/SQLite; no authn/authz | Spec |
| `crates/relayable-server/src/session.rs:1-7,29,31,33,292,301,304,305,470,903-906` | In-process `BlockRuntime`/PTY ownership; bounds; empty server scrollback; `SessionRuntime`; tombstones | Code |
| `crates/relayable-server/src/registry.rs:56,84-85,98,182` | `ServerShared.sessions`; child owned by registry; tombstone on close | Code |
| `crates/relayable-server/src/lib.rs:1-6,123-135` | Dropped client never stops child; graceful shutdown reaps children; socket unlink | Code |
| `crates/relayable-server/src/connection.rs:38,79-80,281,312` | Outbound bound; `close_now` leaves session/child untouched; dedup window; transport-only teardown | Code |
| `crates/relayable-server/src/signals.rs:1-6,19-32,40-42` | SIGINT/SIGTERM only; async-signal-safe atomic flag | Code |
| `crates/relayable-server/src/config.rs:23,41,43,126` | `output_tail_bytes` default; `event_retention_events` default | Code |
| `crates/relayable-server/src/events.rs:141-153,201` | Bounded retained event log (drop-oldest) | Code |
| `crates/relayable-protocol/src/limits.rs:14-20` | Provisional credit window and limits | Code |
| `crates/relayable-pty/src/lib.rs:121,148,169-176,236,270,292` | Opaque `PtyHandle`; spawn/setsid; try_wait/kill/shutdown | Code |
| `crates/relayable-server/src/main.rs:3` | Foreground binary; no systemd unit, no forking | Code |
| `apps/linux/src/term/mod.rs:31,164` | Scrollback is client-engine-owned today | Code |
| `Cargo.toml:29-44`; `crates/relayable-server/Cargo.toml:13-23` | `tempfile` workspace dep; server deps; no storage crate | Code/command |
| `server_e2e.rs:508,869,1319,1392`; `tests/reconnect/main.rs:42,188,308`; `tests/lifecycle/main.rs:102,451`; `tests/multi-client/main.rs:227`; `tests/flow-control/main.rs:131,467`; `tests/events/main.rs:260` | Tests proving client-close/network-loss survival, shutdown reaping, flow-control survival | Tests |
| `scripts/ci.sh:197`; `docs/operations/toolchain-audit.md:747` | No systemd/packaging recipe; service/persistence model undecided (gap G5) | Spec/audit |
| `docs/agent-loop/DECISIONS.md:12,17,125` | DEC-005 index row; DEC-010 Android deferral | Spec |

## References

- `full.md` (Relayable product specification) — §0.1, §1.3, §5.2, §6.1, §6.3, §6.4, §7.1, §7.2, §11, §12, §14.4.
- `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md` — domain model, tombstone, event-log, and
  crate-boundary decisions this ADR extends.
- `docs/adr/ADR-002-protocol-schema-and-transport.md` — local Unix-socket transport and peer-credential
  boundary.
- `docs/adr/ADR-003-terminal-engine-and-gpui-artifact-pin.md` — terminal engine; related field references
  DEC-005 / ADR-004.
- `docs/adr/ADR-005-environment-variable-namespace.md` — `RELAYABLE_*`-only namespace governing any new
  configuration name.
- `docs/architecture/interface-contract.md` — snapshot schema, connection state model, and the
  "restart, not resurrected process" clause.
- `protocol/compatibility/core-001-method-set.md` and `core-002-method-set.md` — implemented method sets
  and explicit non-claims.
- `docs/agent-loop/DECISIONS.md` — DEC-005 index row and DEC-010 Android guidance.
- `docs/operations/toolchain-audit.md` — verified host toolchain, systemd audit (gap G5).
