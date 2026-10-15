# CORE-001 CLI surface and JSON schemas

Status: **frozen interface note for the `relayable` CLI** (`crates/relayable-cli`). It is the
sibling of `protocol/compatibility/core-001-method-set.md` (the wire/method note) and does not
change `docs/architecture/interface-contract.md`. Where a name is not traceable to repository
evidence it is marked `Relayable-defined (no REX evidence)`.

Shipped product name is **Relayable** (`full.md` §0.1); the binary, package name, help text, and
every output string say `relayable`. The `rex ...` spellings in
`docs/compatibility/reference-behavior-matrix.md` (the matrix) are **reference evidence only**.
There is no `rex` binary, alias, or package name.

## 1. Binary and commands

`crates/relayable-cli/Cargo.toml` declares `[[bin]] name = "relayable"`. The CLI depends only on
`{relayable-model, relayable-protocol, relayable-client}` (ADR-001 §7); it must not depend on
`relayable-server` or `relayable-pty`. **Therefore there is no `relayable server` subcommand:** the
foreground server is the separate `relayable-server` binary. This is the ADR-001 split.

```
relayable --help | -h
relayable --version | -V
relayable version [--json]
relayable ls [--json]
relayable new [--json]
relayable session inspect [-s|--session <target>] [--json]
relayable session close   [-s|--session <target>] [--json]
```

- `--help`/`-h` prints the usage above to **stdout** and exits `0`.
- `--version`/`-V` and `version` print `relayable <crate version> (protocol 1.0)`.
- No other flags exist. `--label`, `send`, `capture`, `attach`, `detach`, `clients`, `doctor`,
  `server …`, and the rest of the `full.md` §8.1 candidate family are **deferred** (matrix §7.1
  records the complete command tree as unknown).

Human output (Relayable-defined, no REX evidence except where noted):

- `ls` — one line per session: `session:<uuid>  <label or ->`.
- `new` — the session id, window id, block id, PTY id, then `pid:<n>`, one per line.
- `session inspect` — the session line, then one line per window
  (`window:<uuid>  active=<bool>  <label or ->`) and one line per block
  (`block:<uuid>  <cols>x<rows>  <state>  pid=<n>  pty:<uuid>`).
- `session close` — `session:<uuid>  <state>`.

## 2. Target resolution

Resolution is owned by `relayable-cli`, never by `relayable-model` (contract §1), and follows the
matrix §6 first-match order verbatim:

1. full ID, such as `session:01a10cf3-2655-7e48-9b9c-43dd6612bef5` (matrix §6, verified-official-docs);
2. case-insensitive label (matrix §6);
3. 1-based position within `relayable ls` order (matrix §6);
4. ID suffix of at least four characters (matrix §6).

The command fetches `session.list` first and resolves against that order. If a target matches more
than one session, the command **stops without mutating anything**, prints the matches with their
short (six-character) ids, and exits `3` (matrix §6, verified-official-docs). If `-s/--session` is
absent, `$RELAYABLE_SESSION` is used when set; if neither is present, the command defaults to the
single session when exactly one exists and otherwise errors (ambiguous → `3`, missing → `2`).

Whether label matching is exact or substring-based is recorded as unknown by the matrix; this slice
uses **exact, case-insensitive** label equality (matrix §6 unknown resolved as a Relayable-defined
choice).

## 3. Exit codes

Adopted verbatim from matrix §7.5 (all rows verified-official-docs; the `rex` command name is
reference evidence and ships as `relayable`):

| Code | Meaning |
| --- | --- |
| `0` | the command succeeded |
| `1` | the command failed |
| `2` | the command line was invalid |
| `3` | a target was not found or was ambiguous |
| `4` | the server could not be reached |
| `130` | the command was interrupted |

Mapping to those codes (the concrete rule frozen by `core-001-method-set.md` §10):

| Source | Exit |
| --- | --- |
| `ClientError::Connect`, `Transport`, `Disconnected`, `NotHandshaken` | `4` |
| server code `incompatible_version` | `4` |
| server code `unknown_entity`, `ambiguous_target` | `3` |
| local target not found / ambiguous | `3` |
| local usage error, no target and no sessions | `2` |
| any other server error code | `1` |
| `Protocol`, `Timeout`, any other failure | `1` |

**`130` is implemented by the shell default, not emitted by the CLI.** Without `libc` the CLI cannot
install a signal handler; `SIGINT` terminates the process with the shell-reported interrupt status
and the CLI never returns `130` itself. This is documented rather than faked; adding real `SIGINT`
handling is deferred to a slice that may take a `libc` dependency.

## 4. JSON schemas

All JSON is printed compactly to **stdout**. No floats and no timestamps appear anywhere. Key sets
are frozen; key order is not significant (`serde_json` uses sorted maps without the
`preserve_order` feature).

`ls --json` — **matrix-backed**: shape taken verbatim from the matrix §7.4 sample
(verified-official-docs). Each session object has exactly two keys.

```json
{"sessions":[{"session_id":"session:<uuid>","label":"Research"},{"session_id":"session:<uuid>","label":null}]}
```

`new --json` — **Relayable-defined** (matrix §7.4 records per-command schemas other than `ls` as
unknown).

```json
{"session_id":"session:<uuid>","window_id":"window:<uuid>","block_id":"block:<uuid>","pty_process_id":"pty:<uuid>","program_pid":1234,"label":null}
```

`session inspect --json` — **Relayable-defined** projection of the contract §2 snapshot. Enum values
are lowercase snake_case strings; `state` of a program, block, and session use `unspecified`,
`running`/`exited`, `terminal`, and `active`/`closed` respectively. `pwd` and `title` are present in
the wire `ProgramStatus` but are deliberately not projected by this CLI version; timestamps are
never projected.

```json
{
  "sequence": 7,
  "server": {"server_id":"server:<uuid>","protocol_version":"1.0","sequence":7},
  "session": {"session_id":"session:<uuid>","label":"Research","revision":2,"state":"active"},
  "windows": [{"window_id":"window:<uuid>","session_id":"session:<uuid>","label":null,"revision":1,"active":true}],
  "blocks": [{
    "block_id":"block:<uuid>","window_id":"window:<uuid>","session_id":"session:<uuid>",
    "kind":"terminal","label":null,"revision":1,"cols":80,"rows":24,
    "program_status":{"state":"running","exit_code":null,"signal":null},
    "pty_process_id":"pty:<uuid>","program_pid":99,"writer_attached":false
  }],
  "clients": [{
    "client_id":"client:<uuid>","kind":"cli","platform":"linux","version":"0.0.0",
    "connection_state":"Ready","principal":null
  }]
}
```

`session close --json` — **Relayable-defined**.

```json
{"session_id":"session:<uuid>","state":"closed"}
```

`version --json` — **Relayable-defined**.

```json
{"name":"relayable","version":"0.0.0","protocol_version":"1.0"}
```

## 5. Error JSON

With `--json`, a failed command prints to **stdout** (the exit code still follows §3):

```json
{"error":{"code":"<contract lowercase code>","message":"...","details":{}}}
```

`code` is the contract §6 lowercase string when the error came from the server
(`relayable_protocol::error_code::contract_str`). CLI-local failures that have no server code map
to the closest contract code:

| Source | `code` |
| --- | --- |
| server error | its own contract code |
| target not found | `unknown_entity` |
| target ambiguous | `ambiguous_target` |
| missing target / usage | `bad_request` |
| read timeout | `timeout` |
| connect / transport / disconnect / protocol / not handshaken | `internal` |

`details` is the server's `Error.details` map for server errors (string values); for an ambiguous
target it is `{"matches":[{"session_id":"session:<uuid>","label":null}, …]}`; otherwise `{}`.

Without `--json`, errors print `Error: <message>` to **stderr**; an ambiguous target adds one line
per match with its short id, matching the matrix §6 sample `Error: session target "work" is
ambiguous`.

## 6. Environment namespace

`RELAYABLE_*` only; no `REX_*` aliases are read (`REX_*` in the matrix is reference evidence only).

- `RELAYABLE_SERVER` — filesystem path of the server's Unix socket, overriding
  `relayable_protocol::transport::default_socket_path()`; set by `core-001-method-set.md` §11.
- `RELAYABLE_SESSION` — default session target when `-s/--session` is absent.

This is the ADR-002 D14 choice; the final `RELAYABLE_*` vs alias call remains `DEC-006` / ADR-005
pending, as noted in `core-001-method-set.md` §11.

## 7. Client library surface used

The CLI uses `relayable-client` (`ClientConfig`, `Client::from_env`, `list_sessions`,
`create_session`, `session_snapshot`, `close_session`) and `relayable-model` ID types. The client is
synchronous and blocking: `std::io` over `std::os::unix::net`, no async runtime, no `clap`
(ADR-002; `full.md` §5.2).

Every control RPC (and the initial handshake) is bounded by
`relayable_client::CONTROL_RPC_TIMEOUT` (10 s); on expiry the client returns `ClientError::Timeout`,
which the CLI maps to exit `1` (§3), so an unresponsive server can never hang the CLI indefinitely.
`Client::wait_for_output` instead honours its caller-supplied timeout. Protocol types that appear in
the client's public API are re-exported from `relayable_client` (`AttachMode`, `Error`, `ErrorCode`,
`Limits`, `ServerHello`, `SessionRecord`, `SessionSnapshotResponse`, `SessionSummary`,
`TerminalSnapshot`, `TransportError`) so consumers following ADR-001's `automation → {model, client}`
edge need no direct `relayable-protocol` dependency.

## 8. Deferred / not implemented in this slice

- Additional commands and flags (`--label`, `send`, `capture`, `attach`, `detach`, `wait`, `events`,
  `hosts`, `clients`, `doctor`, `config`, `server …`).
- Real `SIGINT` handling and the `130` exit path.
- `RELAYABLE_BLOCK` / `RELAYABLE_CLIENT` (no block- or client-targeting command exists here; see
  `core-001-method-set.md` §11).
- Timestamps and the `pwd`/`title` program-status fields in JSON.
- Remote/TLS server targeting (`-S/--server`).
