# Relayable — REX reference behavior matrix (DOC-001)

## 1. Purpose and context

- **Deliverable:** `docs/compatibility/reference-behavior-matrix.md` (task DOC-001, branch `research/protocol-compat`).
- **Purpose:** record the user-visible behavior documented by the official live REX documentation — the reference product — so Relayable can match it or deliberately differ from it, with every claim traceable to a retrieval. This matrix is evidence input for the ADR work and for DOC-001 in `docs/agent-loop/TASKS.md`.
- **Reference product:** REX by Superlogical. "REX" is used here only as research/compatibility naming for the reference product; the product under construction is **Relayable** (`full.md` §0.1).
- **Evidence status:** documentary only. No `rex` binary was executed and no REX server endpoint was called while producing this matrix. All quotes come from the canonical Markdown alternates of the official docs; the retrieval log is §4.
- **Cross-branch material excluded:** video/GoRex evidence produced by the parallel UI research branch was not used; `observed-video` therefore cannot appear here.
- **Compiled:** 2026-10-15, from the retrievals listed in §4.

## 2. No binary/wire-compatibility claim

**No binary or wire-level compatibility with REX is claimed or supported by this matrix; it records documented user-visible behavior only.**

`full.md` §3 sets the same boundary: "Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it." The fetched official docs do not document a REX wire protocol (§8.3, §15), and no interoperability test was run on this branch.

## 3. Label legend

Allowed label values (exactly these seven):

- `observed-video` — behavior seen in reference video footage (owned by the UI research branch).
- `verified-official-docs` — stated by the official live REX documentation retrieved in §4.
- `verified-by-test` — proven by a test executed in this repository.
- `implemented` — present in Relayable code.
- `intentionally-different` — Relayable deliberately behaves differently from REX, justified by an explicit source decision.
- `inferred` — not stated directly; deduced, with the justification written in the row.
- `unknown` — the official docs are silent; such rows say `not documented in current docs`.

**On this branch only `verified-official-docs`, `intentionally-different`, `inferred`, and `unknown` appear.** `observed-video` requires video evidence and belongs to `research/ui-reference`. `verified-by-test` and `implemented` require tests or implementation, and this branch contains neither.

## 4. Retrieval log

All fetches with `curl -sS -L`; HTTP status and curl exit code are from the fetch logs in the two research notes. sha256 is of the artifact on disk under `/tmp/relayable-protocol-compat/raw/`. All timestamps are UTC on 2026-10-15. In the claim tables below, a source cell links the canonical `.md` alternate actually used for quotes and shows that artifact's retrieval timestamp.

| # | URL | HTTP | curl exit | Retrieved (UTC) | sha256 | Used for |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | https://www.superlogical.com/rex/docs/automate/cli | 200 | 0 | 2026-10-15T16:07:38Z | `feb7a664dcce882e2196d83a1eb9c266add551ef48ce4bbefc8690ef4d753dab` | HTML shell; quotes come from row 2 (JS-rendered HTML, plain grep unreliable) |
| 2 | https://www.superlogical.com/rex/docs/automate/cli.md | 200 | 0 | 2026-10-15T16:07:39Z | `e61162828b0876173085622831cbdae56ab8072bbc8f83c7dee51ea933912b93` | CLI quotes (§5–§7) |
| 3 | https://www.superlogical.com/rex/docs/automate/server-api | 200 | 0 | 2026-10-15T16:07:38Z | `4e8d5040b54d88f6dcdf26b2f50a582f0d46fd022297e4ac3d2065741f51cdec` | HTML shell; quotes come from row 4 |
| 4 | https://www.superlogical.com/rex/docs/automate/server-api.md | 200 | 0 | 2026-10-15T16:07:39Z | `6e31118649040ff616fd83d4fc743aadb657ef0ad93e7d0a4100eb0785882038` | Server API quotes (§8) |
| 5 | https://www.superlogical.com/rex/docs/reference/lua | 200 | 0 | 2026-10-15T16:03:58Z | `80ec081d2cb4ac9cd0a94a2b64b2462d2a0e270e839a456575a09f4b2afc1509` | HTML shell; quotes come from row 6 |
| 6 | https://www.superlogical.com/rex/docs/reference/lua.md | 200 | 0 | 2026-10-15T16:04:47Z | `934773e17693cc6222eff8e40bf79911247500fdc130e92e887bb27678bd1a0f` | Lua API quotes (§9) |
| 7 | https://www.superlogical.com/rex/docs/reference/events | 200 | 0 | 2026-10-15T16:03:59Z | `2f49cd8dc63077723262955f45c8fc1b2aa2d40890c795202106a39c0f6629ac` | HTML shell; quotes come from row 8 |
| 8 | https://www.superlogical.com/rex/docs/reference/events.md | 200 | 0 | 2026-10-15T16:04:47Z | `e0c8b1cdfb0967f674f4dece59f7bc3d17782ab0e928920d8070902c92c7d5a0` | Event names/payloads (§10) |
| 9 | https://www.superlogical.com/rex/docs/automate/lua-events | 200 | 0 | 2026-10-15T16:04:00Z | `ae6e2b5864cccf526ba61bda462461543e7463fda9407ab1f034f147de8bcec5` | HTML shell; quotes come from row 10 |
| 10 | https://www.superlogical.com/rex/docs/automate/lua-events.md | 200 | 0 | 2026-10-15T16:04:47Z | `312426e5edef5c80abb34785cda150de4c30acb8045912c4b6139b388d6af4a9` | Event handler/scope semantics (§10.5) |
| 11 | https://www.superlogical.com/rex/docs/automate/lua-scripts | 200 | 0 | 2026-10-15T16:04:00Z | `4abac36d940f3a2f3468006b04e5932632328868000fdcf125fd280fa727fbd7` | HTML shell; quotes come from row 12 |
| 12 | https://www.superlogical.com/rex/docs/automate/lua-scripts.md | 200 | 0 | 2026-10-15T16:04:48Z | `0d8b1547357bda30e9b85e86f054f240fcf604711cf805d66d98f81da556a372` | `rex do` script semantics (§9.4) |
| 13 | https://www.superlogical.com/rex/docs/build/program-status | 200 | 0 | 2026-10-15T16:04:00Z | `0e1f209b29201f167c944ee1f20465ebfb1c7c9390cbf19ea364eeeb9f5b05f7` | HTML shell; quotes come from row 14 |
| 14 | https://www.superlogical.com/rex/docs/build/program-status.md | 200 | 0 | 2026-10-15T16:04:48Z | `9b3f32f6e53989c3d0909ce13cda4fb7dc15e29718f5a096281eb3ac4a951829` | OSC 7501 protocol quotes (§11) |
| 15 | https://www.superlogical.com/rex/docs | 200 | 0 | 2026-10-15T16:04:49Z | `f668b09c300daf7b2ddef8f6a62cc0355f50c08b41aa853d49d8aadc34b461c1` | Docs navigation only; sha matches the pre-existing recorded fetch log |
| 16 | https://www.superlogical.com/rex/docs/installation.md | 200 | 0 | 2026-10-15T16:05:01Z | `514e3a81ded0e1d7413e06d6e51c3c071ab5cad0955752b8d204f908816d09c7` | Multi-client availability context (§12) |
| 17 | https://www.superlogical.com/sitemap.xml | 404 | 0 | 2026-10-15T16:04:01Z | `cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d` | 404 page; sitemap unavailable |
| 18 | https://www.superlogical.com/sitemap.xml | 404 | 0 | 2026-10-15T16:07:39Z | `cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d` | Repeated fetch by the CLI/API worker; same 404 body |
| 19 | https://www.superlogical.com/sitemap-index.xml | 404 | 0 | 2026-10-15T16:04:06Z | `cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d` | 404 page |
| 20 | https://www.superlogical.com/sitemap_index.xml | 404 | 0 | 2026-10-15T16:04:06Z | `cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d` | 404 page |
| 21 | https://www.superlogical.com/sitemap-0.xml | 404 | 0 | 2026-10-15T16:04:06Z | `cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d` | 404 page |
| 22 | https://www.superlogical.com/robots.txt | 200 | 0 | 2026-10-15T16:04:06Z | `b033243267bcbdab84cb01e940811d6d77a894d78f0470faaf0eb43f8e4efcc9` | No `Sitemap:` directive |

Notes:

- The HTML pages are SvelteKit documents; the site advertises a Markdown alternate via `<link rel="alternate" type="text/markdown">`. Plain `grep` over the HTML did not reliably surface code examples, so the `.md` alternates (rows 2, 4, 6, 8, 10, 12, 14) are the extraction source for every quote below.
- `sitemap.xml` and its common alternates returned 404 (bodies are the site's HTML 404 page), and `robots.txt` has no `Sitemap:` directive, so sitemap-driven page discovery was impossible. Additional pages were reached from navigation/links inside fetched pages.
- Two separate workers fetched `sitemap.xml` on 2026-10-15; both fetches are listed (rows 17–18) because both appear in the source notes. The 404 body sha256 is identical.
- The `cli.html`/`server-api.html` artifacts from a previous crashed attempt were re-fetched; rows 1 and 3 record the current artifacts.
- No API server was contacted; all server API statements are documentary claims.

## 5. Domain hierarchy and stable IDs

| Documented entity or rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Session | verified-official-docs | "A **session** is a group of related work, such as one project. The server keeps a session running whether or not anything is looking at it." | [cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Window | verified-official-docs | "A **window** is a tab within a session. It holds a layout of blocks." | [cli.md:37](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Block | verified-official-docs | "A **block** is one application in a window. Today that means a terminal." | [cli.md:38](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Client | verified-official-docs | "A **client** is a program connected to the server, such as the app. Clients matter when you ask the app itself to do something, like open its command palette." | [cli.md:39–41](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Container order session → window → block | verified-official-docs | Definitions above: a window is "a tab within a session"; a block is "one application in a window". | [cli.md:34–38](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Labels exist for all four entity kinds | verified-official-docs | "Sessions, windows, blocks, and clients can also have a label, which is a name you choose and can change." | [cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Labels are not necessarily unique | verified-official-docs | "Labels are convenient at a prompt, but they can change and two things can share one." | [cli.md:100–102](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Stable IDs never change and are never reused | verified-official-docs | "Each one has an ID that never changes and is never reused, such as `block:01a10d99-0c06-7795-bf67-798ce4c36a33`." | [cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Tagged ID examples | verified-official-docs | `session:01a10cf3-2655-7e48-9b9c-43dd6612bef5`; `session:<uuid>`; `block:<uuid>`; `block:01a1...` | [cli.md:81](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z; [server-api.md:85–86](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| ID scheme beyond the `type:` prefix and the never-reused rule | unknown | `not documented in current docs` | [cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Server addressed by host label or address | verified-official-docs | "A server, by the label of a host from `rex hosts` or by address." | [cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Server-level events describe the server as a whole | verified-official-docs | "These describe the server rather than one session." | [events.md:107–108](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Deletion permanence and whether a session close leaves audit history | unknown | `not documented in current docs` (only a `session_destroyed` event is documented) | [events.md:116](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

## 6. Target resolution

| Documented rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Default target is the CLI context | verified-official-docs | "With no options, a CLI command acts based on its context. Inside a Rex terminal that is the terminal's own block and session, so `rex split` splits the terminal you typed it in." | [cli.md:47–51](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Context comes from environment variables set in each terminal | verified-official-docs | "Rex knows where you are from three environment variables it sets in every terminal" | [cli.md:52–53](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| First match wins | verified-official-docs | "Rex tries to match a session, window, block, or client target (such as `-C`) in this order and stops at the first kind of match it finds" | [cli.md:78–80](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 1. Full ID | verified-official-docs | "A full ID, such as `session:01a10cf3-2655-7e48-9b9c-43dd6612bef5`." | [cli.md:81](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 2. Case-insensitive label | verified-official-docs | "A case-insensitive label, such as `build`." | [cli.md:82](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 3. Position counting from one (windows in tab order) | verified-official-docs | "A position, counting from one. Windows are numbered in tab order, so `-w 2` is the second tab." | [cli.md:83–84](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 4. ID suffix of at least four characters | verified-official-docs | "The end of an ID, at least four characters long, such as `12bef5`. Listing commands print these short IDs." | [cli.md:85–86](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Ambiguous target: stop, list matches, exit 3 | verified-official-docs | "If a target matches more than one thing, the command stops without changing anything, lists what matched, and exits with status 3" | [cli.md:88–89](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Ambiguity error sample | verified-official-docs | `Error: session target "work" is ambiguous` followed by one line per match with short IDs | [cli.md:92–93](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Server target is the exception | verified-official-docs | "A server is the exception. `-S` takes only a host label or an address, and a value with a port is always an address." | [cli.md:97–98](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Scripts should persist IDs rather than labels | verified-official-docs | "In a script that has to keep working, save the ID a command returns and use that." | [cli.md:100–102](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Whether label matching is exact or prefix/substring based | unknown | `not documented in current docs` | [cli.md:82](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |

## 7. CLI

### 7.1 Documented commands by family

| Command or capability | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| CLI identity | verified-official-docs | "`rex` is the command-line interface to Rex." | [cli.md:8](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Administrative role | verified-official-docs | "As an administrative tool, it can create sessions, split panes, list active clients, force disconnect a client, etc." | [cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Automation role | verified-official-docs | "As an automation tool, it has a complete, powerful Lua scripting interface." | [cli.md:12](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Available on `PATH` with pre-set context in every terminal | verified-official-docs | "Every terminal you open in Rex has `rex` on its `PATH` automatically, and the context is setup so that the CLI already knows what session you're in, block you're looking at, etc." | [cli.md:14–16](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Help entry points | verified-official-docs | "Run `rex help` for every command, or `rex help <command>` for the details of one." | [cli.md:27–28](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Session list: `rex ls` | verified-official-docs | `rex ls` — "list your sessions" | [cli.md:22–24](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Session inspect: `rex session inspect` | verified-official-docs | `rex session inspect` — "show details of the session you are in" | [cli.md:23](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Block inspect: `rex block inspect` | verified-official-docs | `rex block inspect` — "show details of the terminal you are typing in" | [cli.md:24](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Split: `rex split` | verified-official-docs | "so `rex split` splits the terminal you typed it in" | [cli.md:49–51](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Send input: `rex send` | verified-official-docs | `rex send -s build -w 2 'make test'` — "type into window 2 of the "build" session" | [cli.md:70–71](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Capture output: `rex capture` | verified-official-docs | `rex capture -b logs` — "print the screen of the block labeled "logs"" | [cli.md:71–72](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Hosts: `rex hosts` | verified-official-docs | "by the label of a host from `rex hosts`" | [cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Server-scoped listing: `rex -S <server> ls` | verified-official-docs | `rex -S studio ls` — "list the sessions on the host labeled "studio"" | [cli.md:73](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Waiting: `rex wait`, `rex run --wait` | verified-official-docs | "Commands that wait for a program, such as `rex wait` and `rex run --wait`, exit with that program's status instead." | [cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Event stream: `rex events` | verified-official-docs | "streamed from the CLI with `rex events`"; scopes `rex events`, `rex events -s SESSION`, `rex events --all` | [events.md:10–11, 153–159](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Server API: `rex api list`, `rex api describe <method>`, `rex api call <method> [args]` | verified-official-docs | "rex api list" / "rex api describe session.new_split" / "rex api call server.version" | [server-api.md:12–14](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Script runner: `rex do` | verified-official-docs | "`rex do` runs a Lua script against a Rex server." | [lua-scripts.md:8](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Create sessions, list active clients, force disconnect capability | verified-official-docs | "it can create sessions, split panes, list active clients, force disconnect a client, etc." (capability only; no command names printed on this page) | [cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Complete `rex` command tree | unknown | `not documented in current docs` — the page says "Run `rex help` for every command" but does not reproduce the list | [cli.md:27](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| The product exposes more commands than the docs page prints | inferred | Justification: the page directs the reader to `rex help` "for every command" and presents the listings only as "try it" examples, but does not print the full set; therefore a larger command set exists in the product though it is not documented in current docs. | [cli.md:18–28](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Command aliases (alternate command names) | unknown | `not documented in current docs` | [cli.md:27](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Commands from the `full.md` §8.1 candidate family list (`version`, `doctor`, `server …`, `new`, `window …`, `session rename/close`, `attach/detach`, `clients list/disconnect`, `config check/reload`) | unknown | `not documented in current docs` — see §14 D-01 | [full.md §8.1](full.md) · repository text |

### 7.2 Flags and aliases

| Flag | Label | Documented meaning (verbatim) | Source |
| --- | --- | --- | --- |
| `-s`, `--session` | verified-official-docs | "A session" | [cli.md:64](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `-w`, `--window` | verified-official-docs | "A window within the session" | [cli.md:65](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `-b`, `--block` | verified-official-docs | "A block" | [cli.md:66](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `-C`, `--client` | verified-official-docs | "A client. `REX_CLIENT` sets a default." | [cli.md:67](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `-S`, `--server` | verified-official-docs | "A server, by the label of a host from `rex hosts` or by address. `REX_SERVER` sets a default." | [cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `--json` | verified-official-docs | "Pass `--json` to get JSON instead, which is stable to parse and includes full IDs" | [cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `-q` | verified-official-docs | "Use `-q` to print only the names" (`rex api list -q`) | [server-api.md:43](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `--all` (event stream) | verified-official-docs | "Everything, including clients joining and leaving." (`rex events --all`) | [events.md:159](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `--all` (`rex do` event scope) | verified-official-docs | "Pass `--all` to hear every session, including ones created after the script starts" | [lua-events.md:99–100](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `--for` (`rex do`) | verified-official-docs | "The time given with `--for` runs out, such as `--for 10m`." | [lua-events.md:112](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `--fail-fast` (`rex do`) | verified-official-docs | "Pass `--fail-fast` to exit with status 1 at the first failure instead." | [lua-events.md:133](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `--args` (`rex do`) | verified-official-docs | "pass all the arguments as JSON with `--args`" | [lua-scripts.md:77](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `-e` (`rex do`) | verified-official-docs | "inline with `-e`" (`rex do -e '<code>'`) | [lua-scripts.md:40](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `--wait` (in `rex run --wait`) | verified-official-docs | "such as `rex wait` and `rex run --wait`" | [cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Additional flags, aliases, or short-flag clustering | unknown | `not documented in current docs` | [cli.md:63–68](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |

### 7.3 Environment variables

| Variable | Label | Documented meaning (verbatim) | Source |
| --- | --- | --- | --- |
| `REX_SESSION` | verified-official-docs | "The ID of the session the terminal belongs to" | [cli.md:56](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `REX_BLOCK` | verified-official-docs | "The ID of the terminal's own block" | [cli.md:57](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `REX_SERVER` | verified-official-docs | "The address of the server that runs the terminal" | [cli.md:58](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `REX_CLIENT` | verified-official-docs | "A client. `REX_CLIENT` sets a default." | [cli.md:67](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| The three variables Rex sets in every terminal | verified-official-docs | "Rex knows where you are from three environment variables it sets in every terminal" followed by `REX_SESSION`, `REX_BLOCK`, `REX_SERVER` | [cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Whether `REX_CLIENT` is set automatically in terminals | unknown | `not documented in current docs` — it appears only as a default source for `-C` | [cli.md:52–58, 67](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Any environment variables other than the four `REX_*` names | unknown | `not documented in current docs` | [cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |

### 7.4 JSON output

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Tables by default; `--json` is stable to parse and includes full IDs | verified-official-docs | "Most commands print a table meant for reading. Pass `--json` to get JSON instead, which is stable to parse and includes full IDs" | [cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `rex ls --json` shape | verified-official-docs | `{"sessions": [{"session_id": "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5", "label": "Research"}, {…"session_id": "session:01a10d97-…", "label": "SL Website"}]}` | [cli.md:110–121](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Per-command JSON schemas for other commands | unknown | `not documented in current docs` — only the `rex ls --json` sample is printed | [cli.md:106–121](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |

### 7.5 Exit codes

| Status | Label | Meaning (verbatim) | Source |
| --- | --- | --- | --- |
| 0 | verified-official-docs | "The command succeeded" | [cli.md:129](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 1 | verified-official-docs | "The command failed" | [cli.md:130](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 2 | verified-official-docs | "The command line was invalid" | [cli.md:131](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 3 | verified-official-docs | "A target was not found or was ambiguous" | [cli.md:132](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 4 | verified-official-docs | "The server could not be reached" | [cli.md:133](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| 130 | verified-official-docs | "The command was interrupted" | [cli.md:134](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Wait commands propagate the child program's status | verified-official-docs | "Commands that wait for a program, such as `rex wait` and `rex run --wait`, exit with that program's status instead." | [cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| `rex api call` failure mapping to the exit table | unknown | `not documented in current docs` — the page says "exits with a nonzero status" and shows `status=400` in the server message, but does not map it to the CLI table | [server-api.md:146–154](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Exit status when `rex do --for` time runs out | unknown | `not documented in current docs` | [lua-events.md:112](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |

## 8. Server API

### 8.1 Discovery and version coupling

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| The server documents its own API | verified-official-docs | "The server documents its own API. You can list every method, read a method's arguments and result, and call it, all from the CLI:" | [server-api.md:8–9](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Documentation always matches the connected server's version | verified-official-docs | "Because the documentation comes from the server you are connected to, it always matches that server's version." | [server-api.md:17–18](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| CLI and Lua APIs derive dynamically from the server API | verified-official-docs | "The CLI and Lua APIs all derive dynamically from the server API it is connected to, so these commands are also important for scripting and automation at a higher level." | [server-api.md:20–21](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `rex api list` prints every method with a short description | verified-official-docs | "`rex api list` prints every method with a short description" (sample columns `METHOD`, `DESCRIPTION`) | [server-api.md:26–34](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Sample listed methods | verified-official-docs | `block.close` — "Close a block within a session."; `block.event` — "Deliver an external event to a block within a session."; `block.method` — "Invoke a typed method on a block within a session" | [server-api.md:31–34](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| A second table lists block-provided methods | verified-official-docs | "A second table lists the methods that blocks provide, such as reading a terminal's screen." | [server-api.md:36–37](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `rex api list -q` prints names only | verified-official-docs | "Use `-q` to print only the names, which is handy for searching" | [server-api.md:43–46](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `rex api describe` content | verified-official-docs | "`rex api describe` prints everything the server knows about one method: what it does, the arguments it takes, what it returns, and an example of each." | [server-api.md:52–53](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Describe sample method name | verified-official-docs | `Method: session.set_block_label` | [server-api.md:59](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Machine-readable discovery fields | verified-official-docs | "Each method has a `name`, a `description`, an `input_schema`, an `output_schema`, and, where the server has them, a `request_example`, a `response_example`, and a `validation_notes`." | [server-api.md:246–248](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Method names explicitly present in the fetched docs | verified-official-docs | `block.close`, `block.event`, `block.method`, `client.set_label`, `session.set_block_label`, `session.set_label`, `session.set_window_label`, `session.view`, `session.new_split`, `session.create`, `session.list`, `server.version`, `com.superlogical.terminal.format`, `com.superlogical.terminal.pwd`, `com.superlogical.terminal.title` (sample tables are truncated with `...`, so this is not an exhaustive inventory) | [server-api.md:26–46, 59, 110–135, 201–233](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z; [lua.md:173–250](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Complete method inventory and every method's schemas | unknown | `not documented in current docs` — the live inventory comes from the connected server; the docs print truncated samples | [server-api.md:26–34](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Method naming/stability policy | unknown | `not documented in current docs` — docs show dotted names and creator-qualified names but state no policy | [server-api.md:26–34, 201–216](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |

### 8.2 Schemas and examples

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Schema dialect is JSON Schema | verified-official-docs | "The schemas are [JSON Schema](https://json-schema.org)." | [server-api.md:94](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `properties` lists each argument and its type | verified-official-docs | "`properties` lists each argument and its type." | [server-api.md:96](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `required` lists arguments that must be present | verified-official-docs | "`required` lists the arguments that must be present." | [server-api.md:97](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `additionalProperties: false` makes misspelled arguments an error | verified-official-docs | "`additionalProperties: false` means a misspelled argument is an error rather than being ignored." | [server-api.md:98–99](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Input schema sample (`session.set_block_label`) | verified-official-docs | `"block_id": { "type": ["string", "null"] }`; `"label": { "type": "string" }`; `"session_id": { "type": "string" }`; `"required": ["session_id"]`; `"additionalProperties": false` | [server-api.md:63–71](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Output schema sample | verified-official-docs | `"revision": { "type": "integer", "minimum": 0 }`; `"required": ["revision"]`; `"additionalProperties": false` | [server-api.md:75–81](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Request example | verified-official-docs | `{"session_id": "session:<uuid>", "block_id": "block:<uuid>", "label": "shell"}` | [server-api.md:84–88](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Response example | verified-official-docs | `{"revision": 42}` | [server-api.md:91](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Some methods print validation notes | verified-official-docs | "Some methods also print validation notes, which describe rules the schema can't express." | [server-api.md:102–103](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Which JSON Schema draft/version is used | unknown | `not documented in current docs` | [server-api.md:94](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |

### 8.3 Calling methods, session defaulting, errors

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `rex api call` prints the result as JSON | verified-official-docs | "`rex api call` calls a method and prints its result as JSON:" | [server-api.md:107](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Sample result (`server.version`) | verified-official-docs | `{"rex_version": "0.1.0", "platform": "darwin_arm64", ...}` | [server-api.md:110–115](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Argument forms: JSON object, `@file`, or stdin `-` | verified-official-docs | "Arguments are a single JSON object. Pass it directly, read it from a file with `@`, or read it from stdin with `-`" | [server-api.md:118–125](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `-s` auto-fills `session_id` for most `session.` methods | verified-official-docs | "Most `session.` methods need a `session_id`. Name the session with `-s` and `rex api call` adds it to the arguments for you" | [server-api.md:127–129](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `rex api call` does not default to the current session | verified-official-docs | "Unlike other `rex` commands, `rex api call` doesn't default to the session you are in. Inside a Rex terminal, pass `-s "$REX_SESSION"` to use it." | [server-api.md:134–135](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Other IDs are never auto-filled | verified-official-docs | "Other IDs are never filled in. A method that acts on a block needs its `block_id` in the arguments." | [server-api.md:137–138](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Indented on a terminal, one line when piped | verified-official-docs | "The result is indented when printed to a terminal and kept on one line when piped, so it is ready for `jq`" | [server-api.md:140–142](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Error shape | verified-official-docs | `Error: ... input: validating: block_id is required (status=400 code=bad_request)` | [server-api.md:149–154](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `bad_request` meaning | verified-official-docs | "`bad_request` means the arguments were wrong" | [server-api.md:157–158](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `method_not_found` meaning | verified-official-docs | "`method_not_found` means the server has no method by that name, which can happen when the server is older than the docs you are reading." | [server-api.md:158–159](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Complete error-code set | unknown | `not documented in current docs` — only `bad_request` and `method_not_found` are named | [server-api.md:157–159](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Transport/wire format of `rex api call` | unknown | `not documented in current docs` — no JSON-RPC/HTTP/WebSocket statement | [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Authentication/authorization for the server API | unknown | `not documented in current docs` | [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Version negotiation or capability handshake | unknown | `not documented in current docs` — only the "always matches that server's version" statement and the `method_not_found` note | [server-api.md:17–18, 157–159](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Pagination, streaming, and rate limits | unknown | `not documented in current docs` | [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Whether the server API is reachable remotely | unknown | `not documented in current docs` — `-S` is the only remote-facing statement | [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |

### 8.4 Lua bindings of the server API

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `rex.call` takes a method's full name and arguments | verified-official-docs | "`rex.call` takes a method's full name and its arguments as a table" | [server-api.md:164–166](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| `rex.session` helper mapping | verified-official-docs | `session.view` → `rex.session.view{}`; `session.set_label` → `rex.session.set_label{ label = "work" }`; `session.new_split` → `rex.session.new_split{ ... }` | [server-api.md:172–178](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Helpers fill `session_id`; `rex.call` does not | verified-official-docs | "The helpers fill in `session_id` when it is left out. `rex.call` doesn't, so pass `rex.session_id` yourself when calling a `session.` method through it." | [server-api.md:181–183](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Methods without helpers | verified-official-docs | "A few `session.` methods have no helper because they don't act on an existing session, such as `session.list` and `session.create`." | [server-api.md:185–187](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Failed Lua call convention | verified-official-docs | "A failed call returns `nil` and the server's error message" | [server-api.md:189–193](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Block method invocation wrapper | verified-official-docs | "To call one, pass the `block_id` to call it on, and put the method's own arguments under `args`" — `{"block_id": "block:01a1...", "args": {"trim": true}}` | [server-api.md:219–225](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| Lua block helper fills the current block | verified-official-docs | "In Lua, `rex.block.call` builds that wrapper for you and fills in the current block" — `rex.block.call(terminal, "format", { trim = true })` | [server-api.md:227–233](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |

## 9. Lua API and script semantics

### 9.1 Runtime

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Lua version and standard library | verified-official-docs | "Rex runs Lua 5.1 with the full standard library, including `io`, `os`, and `require`." | [lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Only the global `rex` table is added | verified-official-docs | "Rex only adds API to the global `rex` table." | [lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Functions are tagged by where they work | verified-official-docs | "Rex runs Lua in two places, the configuration file and scripts, and each function below is tagged with where it works" (tags `Config`, `Script`, `Config at load`, `Config later`) | [lua.md:11–26](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| "No effect" tag meaning | verified-official-docs | "**No effect** means the call is accepted and does nothing useful. A script's key bindings and hosts never reach a keymap or the server." | [lua.md:44–45](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| "Missing" tag meaning | verified-official-docs | "**Missing** means the name is `nil`. Calling it is an error, so a file shared between both should check for it first." | [lua.md:46–47](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| "Fails" tag meaning | verified-official-docs | "**Fails** means the call returns `nil` and an error message, and is reported as a warning." | [lua.md:48](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Lua patch level, build flags, LuaJIT/5.4 compatibility | unknown | `not documented in current docs` — only "Lua 5.1" is stated | [lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Licensing/embedding requirements for Lua | unknown | `not documented in current docs` | [lua.md](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |

Function availability by context, exactly as documented:

| Functions | Config, at load | Config, later | Script, at load | Script, later | Label | Source |
| --- | --- | --- | --- | --- | --- | --- |
| `rex.bind`, `rex.unbind` | Yes | Functions only | No effect | No effect | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.mode`, `rex.bindings`, `rex.modes` | Yes | Error | No effect | Error | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.action` | Yes | Error | Yes | Error | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.host` | Yes | Error | No effect | Error | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.on` | Warning | Error | Yes | Error | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.call`, `rex.session.*`, `rex.block.call` | Fails | Yes | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.invoke` | Warning | Yes | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.client.queue` | Error | Actions only | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.layout.*`, `rex.log` | Yes | Yes | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.sleep`, `rex.wait`, `rex.stop` | Missing | Missing | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.args`, `rex.session_id`, `rex.block_id` | Missing | Missing | Yes | Yes | verified-official-docs | [lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |

### 9.2 Key bindings, actions, hosts, logging

| Function/behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `rex.bind(key, action[, args])` / `rex.bind(key, fn)` | verified-official-docs | "Binds a key to a Lua function called as `fn(ctx, ev)`. Return `true` to mark the key as handled. Not allowed in a mode." | [lua.md:60–76](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Key event fields | verified-official-docs | "`ev.keybinding` is the key that was pressed. For a sequence, `ev.prefix` is a list of the keys before it." | [lua.md:78–80](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.unbind` forms | verified-official-docs | "Removes the bindings of a key in this file and in the client's defaults." (plus `rex.unbind{ mode = name }` and `rex.unbind(binding)`) | [lua.md:82–97](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.mode(name[, options])` | verified-official-docs | "Declares a mode and sets its options. The only option is `exclusive`, a boolean that defaults to `false`." | [lua.md:99–101](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.bindings([filter])` / `rex.modes()` | verified-official-docs | "Returns the bindings this file and the files it requires have made so far, in order."; "Returns the modes declared or bound so far, each a table with `name`, `exclusive`, and `source`." | [lua.md:103–127](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.action{ ... }` fields | verified-official-docs | Required `name`, `title`, `run`; optional `description`, `category` (defaults to `Actions`), `keywords`, `args`, `repeats` | [lua.md:131–146](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Action `ctx` fields | verified-official-docs | "`ctx` has `session_id`, `block_id`, `client_id`, `origin`, and `server`. A field Rex doesn't know is absent." | [lua.md:150–152](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.invoke(name[, args])` | verified-official-docs | "Runs an action and returns its result, or `nil` and an error message." | [lua.md:154–161](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Action nesting limit | verified-official-docs | "Calls can nest up to 16 deep." | [lua.md:163](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.client.queue(name[, args])` | verified-official-docs | "Asks the client to perform one of its own actions, such as `pane.zoom`."; "Raises an error when there is no client to ask." | [lua.md:165–172](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.host{ ... }` fields | verified-official-docs | Required `label`, `endpoints`; optional `endpoint`, `id`, `icon` | [lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.servers()` / `rex.server(label)` | verified-official-docs | "Returns the labels of the other servers available to the running action or script, as a list."; "Returns a table with the same `session`, `block`, and `call` functions as `rex`, whose calls go to that server."; "Calls made through it don't fill in a `session_id` or `block_id`." | [lua.md:329–337](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.log([level, ]message)` | verified-official-docs | "`level` is `debug`, `info`, `warn`, or `error`, and defaults to `info`. In a script the message is printed to the terminal." | [lua.md:339–341](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |

### 9.3 Call surface

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Call results and failure convention | verified-official-docs | "Every call in this section returns a result table, or `nil` and an error message. A failure never raises an error." | [lua.md:164–165](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session.<method>(args)` fills `session_id` but not other IDs | verified-official-docs | "`session_id` is filled in when it is left out. Other IDs, such as `block_id`, are not." | [lua.md:167–171](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.call(method[, args])` reaches any server method | verified-official-docs | "Calls any server method by its full name, including the ones `rex.session` doesn't cover." | [lua.md:215–222](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.block.call(creator, method[, args])` fills the current block | verified-official-docs | "`block_id` is filled in from the current block when it is left out." | [lua.md:226–234](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: inspection | verified-official-docs | `view` — "Return the session's windows, layouts, and blocks."; `list_blocks` — "List every block and whether it is placed or detached."; `list_clients` — "List the clients attached to the session."; `describe_block` — "Return a block's placement and the methods it offers."; `help` — "Describe the methods available through the session." | [lua.md:178–206](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: creation | verified-official-docs | `new_window` — "Create a window from a layout."; `new_split` — "Create a split beside an existing block."; `new_block` — "Create a block without placing it in a layout."; `new_layer` — "Create a floating layer in a window." | [lua.md:180–186](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: closing | verified-official-docs | `close_window` — "Close a window and every block in it."; `close_layer` — "Close a floating layer and every block in it." | [lua.md:183–184](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: block layout | verified-official-docs | `move_block` — "Move a block into a new split beside another."; `swap_blocks` — "Exchange the positions of two blocks."; `detach_block` — "Remove a block from its layout without closing it."; `float_block` — "Move a tiled block into a new floating layer."; `zoom_block` — "Toggle or set whether a block fills its window." | [lua.md:185–190](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: resizing and stacking | verified-official-docs | `resize_pane` — "Move the border between two blocks."; `resize_layer` — "Set a floating layer's position and size."; `raise_layer`, `lower_layer` — "Change a floating layer's stacking order." | [lua.md:187–190](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: windows and focus | verified-official-docs | `move_window` — "Move a window in the tab order."; `focus_block` — "Focus a block."; `focus_window` — "Focus a window."; `focus_direction` — "Focus the nearest block in a direction."; `focus_next_pane`, `focus_previous_pane` — "Focus the next or previous pane."; `focus_next_window`, `focus_previous_window` — "Focus the next or previous window." | [lua.md:191–198](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: labels | verified-official-docs | `set_label` — "Set the session's label."; `set_window_label` — "Set a window's label."; `set_block_label` — "Set a block's label." | [lua.md:199–201](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| `rex.session` method inventory: event scope | verified-official-docs | `attach`, `detach` — "Change which session a script hears events from." | [lua.md:205](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Terminal block method inventory: inspection | verified-official-docs | `format` — "Return the screen as text, HTML, or VT sequences."; `process` — "Describe the terminal's child and foreground processes."; `pwd` — "Return the working directory the terminal last reported."; `title` — "Return the terminal's title."; `size` — "Return the terminal's size in cells and pixels." | [lua.md:237–243](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Terminal block method inventory: control | verified-official-docs | `write` — "Write bytes to the terminal as if they were typed."; `reset` — "Reset the terminal."; `resize` — "Request a size for the terminal." | [lua.md:242–244](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Terminal block method inventory: presentation and status | verified-official-docs | `set_theme` — "Replace the terminal's default colors."; `program_status` — "Return the status records programs have reported."; `list_dir` — "List the directories under a path." | [lua.md:244–250](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Layout builders | verified-official-docs | `rex.layout.block{ flavor, options }`; `rex.layout.horizontal(ratio, a, b)`; `rex.layout.vertical(ratio, a, b)` — "They only build tables and call nothing." | [lua.md:252–262](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Terminal layout options | verified-official-docs | `command`, `shell` ("`none` starts `command` directly"), `cwd`, `initial_input`, `exit` (`{ on_completion = false }` keeps the terminal open) | [lua.md:264–274](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |

### 9.4 `rex do` script semantics

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Script lifecycle | verified-official-docs | "A script runs once from top to bottom and exits (unless there are active event handlers running)." | [lua-scripts.md:18–20](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Invocation forms | verified-official-docs | `rex do ./layout.lua`; `rex do -e '<code>'`; `rex do - < layout.lua` | [lua-scripts.md:38–41](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| File vs action detection | verified-official-docs | "The argument is treated as a file when it ends in `.lua` or contains a slash. Anything else is the name of an action to run." | [lua-scripts.md:43–45](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Default target and `-s`/`-b` override | verified-official-docs | "A script run inside a Rex terminal acts on that terminal's session and block. Use `-s` and `-b` to target a different one" | [lua-scripts.md:47–52](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Standard library and `require` | verified-official-docs | "Scripts have Lua's full standard library, including `io` and `os`, and `require` loads other Lua files." | [lua-scripts.md:54–55](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Scripts execute on the CLI host | verified-official-docs | "Scripts are executed locally where the `rex` CLI is, so security and program access is up to you." | [lua-scripts.md:57–58](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `key=value` arguments and type inference | verified-official-docs | "`n=2` is a number, `on=true` is a boolean, and `name=x` is a string." | [lua-scripts.md:65–75](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| JSON arguments via `--args` (or `@file`) | verified-official-docs | "pass all the arguments as JSON with `--args`" | [lua-scripts.md:77–81](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Script result printing | verified-official-docs | "The value a script returns is printed as JSON. Tables become objects or arrays. Add `--json` for indented output. A script that returns nothing prints nothing." | [lua-scripts.md:85–88](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Script error exit status | verified-official-docs | "A script that raises an error prints the message and exits with status 1" | [lua-scripts.md:95–98](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `rex.stop(code)` status propagation | verified-official-docs | "To exit with a status of your choosing, call `rex.stop` with it. The script ends immediately" | [lua-scripts.md:100–106](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Output and logging policy | verified-official-docs | "`print` writes to the terminal as usual, and `rex.log` writes a log line." | [lua-scripts.md:108–109](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Failed API call does not stop the script | verified-official-docs | "A failed call doesn't stop the script. It returns `nil` and an error message, so check the second value when the result matters" | [lua-scripts.md:133–140](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Session defaults in calls | verified-official-docs | "Calls fill in the script's session, so most calls don't need a `session_id`. A call that acts on one block, such as `set_block_label`, still needs its `block_id`." | [lua-scripts.md:112–116](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `rex.wait` returns the event or `nil, "timeout"` | verified-official-docs | "`rex.wait` returns the event as a table. If the timeout passes first, it returns `nil` and `\"timeout\"`." | [lua-scripts.md:195–200](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| `rex.wait` filter forms | verified-official-docs | "A table matches an event whose fields equal the table's. A function is called with the event and matches when it returns `true`." | [lua-scripts.md:183–196](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| No timeout means wait forever | verified-official-docs | "Without it, `rex.wait` waits forever." | [lua-scripts.md:184–196](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Worked example output | verified-official-docs | `{"block_id":"block:01a10deb-c752-789f-9add-5dfd3a5b3214","exit_code":1}` | [lua-scripts.md:241–242](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Script/API call timeouts and cancellation | unknown | `not documented in current docs` — only `rex.wait` timeout, `rex do --for`, `rex.stop`, and Ctrl+C are described | [lua-scripts.md](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |
| Script outbound network/file capabilities beyond the standard library | unknown | `not documented in current docs` — the docs say scripts run locally with the full standard library | [lua-scripts.md:57–58](https://www.superlogical.com/rex/docs/automate/lua-scripts.md) · 2026-10-15T16:04:48Z |

## 10. Events

### 10.1 Sources, naming, envelope

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Three event sources | verified-official-docs | Session events — "Changes to one session's windows, blocks, and clients."; Terminal events — "Things a terminal reports."; Server events — "Changes to the server as a whole." | [events.md:13–20](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Session/server events use the type name | verified-official-docs | "The event's type, such as `block_closed`." | [events.md:16](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Terminal events use `terminal.` prefix | verified-official-docs | "`terminal.` and the event, such as `terminal.bell`" | [events.md:18](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Full creator-qualified spelling works | verified-official-docs | "`terminal.bell` is short for `com.superlogical.terminal.bell`. Either spelling works." | [lua-events.md:69–71](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `ev.type` | verified-official-docs | "The event's type. For every terminal event this is `block_event`." | [events.md:36](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `ev.session_id` | verified-official-docs | "The session the event happened in. Absent for most server events." | [events.md:37](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `ev.payload` | verified-official-docs | "The event's own fields again, as a table of their own." | [events.md:38](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Block event fields | verified-official-docs | `ev.block_id` — "The block the event came from."; `ev.creator_name` — "The kind of block, such as `com.superlogical.terminal`."; `ev.name` — "The event's name without the creator, such as `bell`." | [events.md:43–46](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Wildcard registration | verified-official-docs | "Register for `block_event` to receive every event from every block, and use `ev.creator_name` and `ev.name` to tell them apart." | [events.md:51–52](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `ctx` mirrors the IDs and is what calls default to | verified-official-docs | "`ctx.session_id` and `ctx.block_id` hold the same IDs, and are what API calls in the handler default to." | [events.md:48–49](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Event ordering hint | verified-official-docs | "`session_view_changed` follows almost every other event in this table, because most of them change the layout." | [events.md:74–76](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Envelope version, global sequence number, envelope timestamp | unknown | `not documented in current docs` — per-event fields such as `revision`, `generation`, `hash`, and `connected_at` exist, but no universal envelope version/sequence/timestamp is specified | [events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

### 10.2 Session events

| Event | Label | When (verbatim) | Payload fields | Source |
| --- | --- | --- | --- | --- |
| `window_created` | verified-official-docs | "A window is created." | `window_id`, `label` | [events.md:59](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `window_closed` | verified-official-docs | "A window is closed." | `window_id` | [events.md:60](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `window_label_changed` | verified-official-docs | "A window is renamed." | `window_id`, `label` | [events.md:61](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `active_window_changed` | verified-official-docs | "A different window becomes the active one." | `window_id` | [events.md:62](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `block_created` | verified-official-docs | "A block is created." | `block_id` | [events.md:63](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `block_closed` | verified-official-docs | "A block is closed." | `block_id` | [events.md:64](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `session_label_changed` | verified-official-docs | "The session is renamed." | `label` | [events.md:65](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `session_view_changed` | verified-official-docs | "The session's layout changes in any way." | `revision` | [events.md:66](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_connected` | verified-official-docs | "A client attaches to the session." | `client_id`, `principal`, `connected_at` | [events.md:69](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_disconnected` | verified-official-docs | "A client detaches from the session." | `client_id`, `principal` | [events.md:70](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_connection_changed` | verified-official-docs | "An attached client's connection is lost or restored." | `client_id`, `principal`, `connection_state` | [events.md:71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `creator_changed` | verified-official-docs | "The kinds of block the session can create have changed." | (no fields listed) | [events.md:72](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

### 10.3 Terminal events

| Event | Label | When (verbatim) | Payload fields | Source |
| --- | --- | --- | --- | --- |
| `terminal.child_exited` | verified-official-docs | "The terminal's program exits." | `exit_code`, and `error` when it is not zero | [events.md:82](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.pwd_changed` | verified-official-docs | "The program reports a new working directory." | `pwd`, `owner` | [events.md:83](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.title_changed` | verified-official-docs | "The terminal's title changes." | `title`, `owner` | [events.md:84](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.bell` | verified-official-docs | "The terminal receives a bell character." | (no fields listed) | [events.md:85](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.process_changed` | verified-official-docs | "The foreground process changes." | `generation`, `foreground`, `ancestors` | [events.md:86](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.desktop_notification` | verified-official-docs | "The program asks for a desktop notification." | `title`, `body` | [events.md:87](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.progress_report` | verified-official-docs | "The program reports progress." | `state`, `progress` | [events.md:88](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.clipboard_written` | verified-official-docs | "The program writes to the clipboard." | `text` | [events.md:89](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.program_status_changed` | verified-official-docs | "A program reports or updates its status." | `id`, `record`, `reason` | [events.md:90](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.program_status_removed` | verified-official-docs | "Program status records are removed." | `ids`, `reason` | [events.md:91](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

Notes on the events above:

| Note | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `terminal.child_exited` is about the started program | verified-official-docs | "It doesn't fire for each command you run inside a shell. A terminal started with a `command` fires it when that command exits." | [events.md:95–99](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `pwd` is a `file://` URL | verified-official-docs | "`pwd` is a URL such as `file://hostname/Users/you/src`, not a plain path. It is only reported by shells set up to report it." | [events.md:100–102](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `owner` shape | verified-official-docs | "`owner` is a table with the `pid` and `name` of the process that made the change." | [events.md:103–104](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `foreground` shape | verified-official-docs | "`foreground` is a table describing the process in the foreground, with `name`, `cwd`, `pid`, and more. It can be `nil`." | [events.md:105–106](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `terminal.progress_report` vs OSC 7501 record fields | unknown | `not documented in current docs` — the relationship between `state`/`progress` payload fields and OSC 7501 keys is not stated on the events page | [events.md:88](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

### 10.4 Server events

| Event | Label | When (verbatim) | Payload fields | Source |
| --- | --- | --- | --- | --- |
| `session_created` | verified-official-docs | "A session is created." | `session`, a table with `session_id` and `label` | [events.md:112](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `session_destroyed` | verified-official-docs | "A session is destroyed." | `session_id` | [events.md:116](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_joined` | verified-official-docs | "A client connects to the server." | `client` | [events.md:119](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_left` | verified-official-docs | "A client disconnects from the server." | `client_id` | [events.md:122](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client_changed` | verified-official-docs | "A connected client's details change." | `client` | [events.md:123](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `keymap_changed` | verified-official-docs | "The configuration file is reloaded." | `generation`, `hash` | [events.md:124](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `hosts_changed` | verified-official-docs | "The list of hosts changes." | `hash` | [events.md:125](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `tailscale_status_changed` | verified-official-docs | "The server's Tailscale status changes." | (no fields listed) | [events.md:126](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `client` payload table | verified-official-docs | "`client` is a table with the client's `client_id`, `info` (its `kind`, `name`, `platform`, and `version`), `principal`, and `connected_at`." | — | [events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

### 10.5 Handler registration, scope, and failure behavior

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `rex.on(event, fn)` signature | verified-official-docs | "`rex.on` takes the name of an event and a function to call each time that event happens" — `rex.on(event, function(ctx, ev) ... end)` | [lua-events.md:32–38](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Handler keeps the script alive | verified-official-docs | "A script that calls `rex.on` doesn't exit when it reaches its last line. `rex do` keeps it running and calls its handlers as events arrive, one at a time." | [lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Registration time is the initial run | verified-official-docs | "Handlers are registered while the script first runs. Calling `rex.on` later, from inside a handler, is an error." | [lua-events.md:45–46](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Handler context fields | verified-official-docs | `ctx.session_id` — "The session the event happened in."; `ctx.block_id` — "The block the event came from, for a block event." | [lua-events.md:48–53](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| API calls default to the event's session | verified-official-docs | "API calls inside a handler default to the event's session rather than the one the script was started in" | [lua-events.md:55–56](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Misspelled event names are registered but never called | verified-official-docs | "Rex doesn't check an event name against a list, so a misspelled name is not an error. The handler is registered and never called." | [lua-events.md:86–87](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Default event scope is one session | verified-official-docs | "By default, a script reacts to events of one session: the one it was run in, or the one named with `-s`." | [lua-events.md:91–93](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `--all` scope | verified-official-docs | "Pass `--all` to hear every session, including ones created after the script starts" | [lua-events.md:99–100](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Stopping conditions | verified-official-docs | "A handler calls `rex.stop(code)`."; "The time given with `--for` runs out, such as `--for 10m`."; "The session it hears is destroyed. A script run with `--all` never stops this way."; "You press Ctrl+C." | [lua-events.md:105–116](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Handler error does not stop the script by default | verified-official-docs | "An error in a handler is printed and the script keeps running, so one bad event doesn't end a long-lived script" | [lua-events.md:123–126](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `--fail-fast` exits 1 at first failure | verified-official-docs | "Pass `--fail-fast` to exit with status 1 at the first failure instead." | [lua-events.md:133](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Blocking inside a handler | verified-official-docs | "`rex.wait` and `rex.sleep` also work inside a handler. No other event is handled until they return." | [lua-events.md:143–144](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| `rex.on` is invalid in configuration files | verified-official-docs | "`rex.on` only works in a script run with `rex do`. The configuration file is never sent events, so a handler registered there is reported as a warning and never called." | [lua-events.md:153–155](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Event retention, replay, and buffering policy | unknown | `not documented in current docs` — only "one at a time" delivery and the `session_view_changed` ordering note are documented | [lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Delivery while a handler is busy (queue, drop, or coalesce) | unknown | `not documented in current docs` beyond "No other event is handled until they return." | [lua-events.md:143–144](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| Ordering across sessions or global sequence numbers | unknown | `not documented in current docs`; the per-session `revision` and per-keymap `generation`/`hash` are the only ordering-adjacent fields | [events.md:66, 124](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Allowed `connection_state` values | unknown | `not documented in current docs` | [events.md:71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

### 10.6 Events on the CLI

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| `rex events --json` is JSON Lines | verified-official-docs | "`rex events --json` prints one JSON object per line. Each has a `type` and a `payload` holding the fields listed on this page" | [events.md:129–130](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Session events are wrapped in `session_event` | verified-official-docs | "A session event arrives wrapped in a `session_event`, which adds the session it belongs to." | [events.md:136–137](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Terminal events are wrapped again in `block_event` | verified-official-docs | "A terminal event is wrapped once more, in a `block_event`"; "Lua removes both layers of wrapping" | [events.md:137–148](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `rex events` scope: sessions only | verified-official-docs | "Sessions being created, destroyed, and changed." | [events.md:157](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `rex events -s SESSION` scope | verified-official-docs | "Everything in one session, including terminal events." | [events.md:158](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `rex events --all` scope | verified-official-docs | "Everything, including clients joining and leaving." | [events.md:159](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Unknown events and fields are tolerated | verified-official-docs | "Rex never checks an event name against a list, so a handler can be registered for an event that only a newer server sends. Ignore fields you don't recognize rather than failing on them." | [events.md:163–166](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

## 11. Program Status Protocol (OSC 7501)

### 11.1 Scope, syntax, and encoding

| Documented rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Purpose | verified-official-docs | "A terminal escape sequence that lets a program tell the terminal what it is doing: idle, working, waiting on the user, finished, or failed, and why." | [program-status.md:6–9](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Generic protocol, not Superlogical-specific | verified-official-docs | "This is a generic terminal protocol. While it is used for rich functionality within Rex, the protocol itself has no Superlogical-specific functionality or language." | [program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| RFC 2119/8174 keywords | verified-official-docs | "The key words MUST, MUST NOT, SHOULD, and MAY are used as described in RFC 2119 and RFC 8174, and only when they appear in capitals." | [program-status.md:47–49](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Byte values | verified-official-docs | `ESC` = `0x1B`; `OSC` = `ESC ]` (`0x1B 0x5D`); `ST` = `ESC \` (`0x1B 0x5C`) "or `BEL`, that is `0x07`" | [program-status.md:51–57](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Report/record definitions | verified-official-docs | "A **report** is one sequence sent by a program. A **record** is what the terminal stores as a result." | [program-status.md:63–66](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Report and query syntax | verified-official-docs | `OSC 7501 ; pairs ST` (report); `OSC 7501 ; ? ST` (feature detection) | [program-status.md:79–81](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Grammar | verified-official-docs | `pairs := pair (":" pair)*`; `pair := key "=" value`; `key := [a-z]+`; `value := [A-Za-z0-9_.,+/=-]*` | [program-status.md:83–86](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Pair separator and whitespace | verified-official-docs | "The body is a list of `key=value` pairs separated by `:` and nothing else. The value character set excludes `:` and `;`, so nothing needs escaping. Whitespace around keys and values is removed." | [program-status.md:88–90](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Malformed pair handling | verified-official-docs | "A malformed pair (no `=`, empty key, or a byte outside the value set) is skipped and the rest of the report is still processed." | [program-status.md:91–92](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Unknown key handling | verified-official-docs | "Unknown keys MUST be ignored. This is how the protocol is extended." | [program-status.md:93](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Repeated key handling | verified-official-docs | "If a key repeats, the last value wins." | [program-status.md:94](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Free-text encoding | verified-official-docs | "Free-text values (`msg`, `title`) are standard base64 of UTF-8 text. Padding is optional." | [program-status.md:96–97](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Decoded text must not contain control characters | verified-official-docs | "The decoded text MUST NOT contain control characters (`U+0000`–`U+001F`, `U+007F`, `U+0080`–`U+009F`), including newline or tab." | [program-status.md:98–100](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Whole-report discard | verified-official-docs | "A report is discarded whole if it exceeds one of the limits, if its base64 does not decode, or if decoded text contains a control character. Nothing from a discarded report is applied." | [program-status.md:101–103](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

### 11.2 Records, IDs, and lifetimes

| Documented rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Record model | verified-official-docs | "A terminal holds a **set of records**, one per id. Each record holds exactly the keys of the last report that used that id." | [program-status.md:106–108](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| ID grammar | verified-official-docs | `id := segment ("/" segment)*`; `segment := [A-Za-z0-9_.+-]{1,32}` | [program-status.md:113–116](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| No `id` addresses the root record | verified-official-docs | "A report with no `id` addresses the **root record**." | [program-status.md:118–120](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Each report replaces its record completely | verified-official-docs | "**Each report replaces its record completely.** A key missing from the report is missing from the record afterwards." | [program-status.md:121–123](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Invalid `id` is ignored | verified-official-docs | "A report whose `id` does not match this grammar is ignored. Falling back to the root record would let a malformed id overwrite it." | [program-status.md:124–126](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `/` nesting and its uses | verified-official-docs | "`/` gives records a parent and child relationship: `build/test` is a child of `build`, and `build` is a child of the root record. A parent does not have to exist." | [program-status.md:127–132](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Nesting is used for clearing and `app` inheritance, MAY be used for grouping | verified-official-docs | "Terminals MUST use this relationship for clearing ... and `app` inheritance ... and MAY use it for grouping. It means nothing else." | [program-status.md:132–134](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Root and child records exist concurrently | verified-official-docs | "Root and child records exist at the same time. A program can be `idle` at its prompt while one of its workers is `blocked`." | [program-status.md:135–136](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Screen switching does not affect records | verified-official-docs | "Records belong to the terminal, not to a screen. Switching between the primary and alternate screen has no effect on them." | [program-status.md:137–138](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Reset behavior | verified-official-docs | "A full reset (`RIS`) removes every record. A soft reset (`DECSTR`) does not." | [program-status.md:138–139](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| No heartbeat | verified-official-docs | "There is no heartbeat, and terminals MUST NOT require re-sending to keep a record alive." | [program-status.md:154–155](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Drop rules on process exit or shell prompt | verified-official-docs | "When the process attached to the terminal exits, or a new shell prompt begins (OSC 133 `A`), the terminal MUST drop `working` and `blocked` records and MAY drop `idle` ones. `done` and `error` survive both events." | [program-status.md:160–165](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

### 11.3 States

`state` is required; a report without `state`, or with an unrecognized value, is ignored ([program-status.md:141–143, 163](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z).

| `state` | Label | Meaning (verbatim) | Lifetime (verbatim) | Source |
| --- | --- | --- | --- | --- |
| `idle` | verified-official-docs | "At rest, waiting for the user's next instruction." | "Until replaced or cleared." | [program-status.md:140](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `working` | verified-official-docs | "Running. May carry `progress`." | "Until replaced, cleared, process exit, or next shell prompt." | [program-status.md:141](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `done` | verified-official-docs | "Finished a piece of work. The result is ready to look at and the user has not seen it yet." | "Until replaced or cleared. Survives process exit and shell prompt." | [program-status.md:142](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `blocked` | verified-official-docs | "Cannot continue until the user does something. `kind` says what, `msg` says why. May carry `progress`." | "Same as `working`." | [program-status.md:143](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `error` | verified-official-docs | "Failed and stopped." | "Same as `done`." | [program-status.md:144](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `clear` | verified-official-docs | "Not a state. Removes the addressed record and every record beneath it. With no `id`, removes every record on the terminal." | (not applicable) | [program-status.md:145](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Interrupt/cancel guidance | verified-official-docs | "When the user interrupts or cancels a program, report `idle`." | — | [program-status.md:147–148](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Program exiting should leave `done`/`error` | verified-official-docs | "A program that exits as soon as it finishes SHOULD report `done` or `error` right before exiting, so a record is left behind for the user to find." | — | [program-status.md:148–151](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Terminal decides when to stop showing `done`/`error` | verified-official-docs | "The terminal decides when to stop showing them, for example when the user returns to the terminal and presses a key." | — | [program-status.md:165–166](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

### 11.4 Keys

| Key | Label | Rules (verbatim excerpts) | Source |
| --- | --- | --- | --- |
| `state` | verified-official-docs | "Required. An unrecognized value causes the whole report to be ignored, so a state added later never turns into `idle` on an older terminal." | [program-status.md:163](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `id` | verified-official-docs | "path, see Records and ids"; "Absent means the root record." | [program-status.md:164](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `kind` | verified-official-docs | "`permission` \| `question` \| `auth`"; "With `blocked` only, ignored otherwise." | [program-status.md:165](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `kind` values | verified-official-docs | "`permission`: approval to do something."; "`question`: the user must type an answer."; "`auth`: a login, token, or credential."; "Unrecognized values are treated as absent." | [program-status.md:165](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `progress` | verified-official-docs | "integer `0` to `100`"; "With `working` or `blocked`, ignored otherwise. Absent means indeterminate: the program is busy but has no percentage to report. Anything else is treated as absent." | [program-status.md:166](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `app` | verified-official-docs | "Stable identifier for the program, such as `cargo`, `terraform`, or `claude-code`."; character set `[A-Za-z0-9_.+-]{1,32}`; "This is not a label; `title` is the label."; "A record without `app` takes it from its nearest ancestor that has one."; "A value outside the character set is treated as absent." | [program-status.md:167](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `title` | verified-official-docs | "base64 UTF-8"; "Short human-readable label for the record. For programs that report several records. The root record's label is normally the window title." | [program-status.md:168](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `msg` | verified-official-docs | "base64 UTF-8"; "One human-readable line: what the record is doing, waiting for, or finished. Terminals MAY shorten it and MUST NOT read meaning into it." | [program-status.md:169](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Key case sensitivity | unknown | `not documented in current docs` — the grammar is `[a-z]+` but no explicit case rule is stated | [program-status.md:85](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

### 11.5 Feature detection

| Documented rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Query and reply forms | verified-official-docs | `program → OSC 7501 ; ? ST`; `terminal → OSC 7501 ; ? : states=idle,working,done,blocked,error : kinds=permission,question,auth ST` | [program-status.md:172–176](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Detection is optional | verified-official-docs | "Detection is optional. Terminals MUST ignore unknown OSCs, so a program MAY send reports without querying first, for example when it cannot read replies." | [program-status.md:178–180](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Reply must contain `?`; MAY list `states`/`kinds` | verified-official-docs | "A supporting terminal MUST reply with `?`. It MAY add `states`, the comma-separated `state` values it accepts (not counting `clear`), and `kinds`, the `kind` values it accepts." | [program-status.md:182–184](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Default `states`/`kinds` when omitted | verified-official-docs | "A reply without `states` means exactly `idle`, `working`, `done`, `blocked`, and `error`. A reply without `kinds` means exactly `permission`, `question`, and `auth`. Those lists are fixed and do not change in later revisions." | [program-status.md:186–188](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Unknown reply pairs must be ignored | verified-official-docs | "Programs MUST ignore pairs in the reply that they do not understand." | [program-status.md:191–192](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| No-reply timeout is program-chosen | verified-official-docs | "A program that gets no reply within a time of its choosing treats the protocol as unsupported." | [program-status.md:194–195](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `CSI c` disambiguation | verified-official-docs | "To avoid waiting, send primary device attributes (`CSI c`) right after the query. Every terminal answers that, so if its reply arrives first, the protocol is unsupported." | [program-status.md:195–198](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Whether a reply's `states`/`kinds` may be empty; a `?` query carrying other keys; reply interleaving | unknown | `not documented in current docs` | [program-status.md:172–198](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

### 11.6 Security, limits, and revision

| Documented rule | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Control characters and markup | verified-official-docs | "Terminals MUST refuse a report whose decoded `msg` or `title` contains a control character, MUST NOT treat either as markup, and SHOULD disarm text direction overrides and other invisible formatting characters when showing them outside the terminal grid." | [program-status.md:208–212](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| The only reply bytes are feature detection | verified-official-docs | "The only bytes a terminal writes back are the feature detection reply, which contains nothing from any report. Ids, titles, and messages are never sent back, and there is no way to read records." | [program-status.md:213–216](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Origin labeling | verified-official-docs | "Anything a terminal shows from a record SHOULD say which terminal it came from, so a program cannot pretend to be a program in another terminal." | [program-status.md:217–219](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Rate limiting | verified-official-docs | "A program can change state as fast as it can write. Terminals SHOULD rate-limit anything a record causes outside the terminal, such as desktop notifications or sounds." | [program-status.md:220–222](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Whole-sequence limit | verified-official-docs | "Whole sequence, `OSC` through `ST` — 4096 bytes. The largest legal report is under 3300." | [program-status.md:227](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Key length limit | verified-official-docs | "Key length — 16 bytes" | [program-status.md:228](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `msg` limit | verified-official-docs | "`msg` — 2732 bytes encoded, 2048 decoded. Check the encoded size before decoding." | [program-status.md:229](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `title` limit | verified-official-docs | "`title` — 256 bytes encoded, 192 decoded" | [program-status.md:230](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `app` limit | verified-official-docs | "`app` — 32 bytes" | [program-status.md:231](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| `id` limit | verified-official-docs | "`id` — 128 bytes total, 32 per segment, 8 levels deep" | [program-status.md:232](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Records-per-terminal limit and eviction | verified-official-docs | "Records per terminal — 256. Terminals MUST support at least 64. When a report would create a record past the cap, the record that was updated least recently is removed to make room." | [program-status.md:233](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation to OSC 9;4 | verified-official-docs | "A terminal SHOULD stop mapping OSC 9;4 once it has received an OSC 7501 report, until the next full reset." | [program-status.md:243–248](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation to desktop-notification sequences | verified-official-docs | "A notification is a one-time event. ... A terminal MAY produce a notification when a record changes." | [program-status.md:250–257](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation to OSC 133 | verified-official-docs | "It is emitted by the shell, not the program, so it can only say that a command is running or has finished with an exit code, never what the program is doing or whether it is stuck." | [program-status.md:259–264](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation to OSC 0/2 | verified-official-docs | "A single free-form string with no structure." | [program-status.md:266](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation to OSC 21337 and OSC 9999 | verified-official-docs | "Earlier work on the same idea. They describe presentation, with colors and indicators, or require JSON parsing" | [program-status.md:268–271](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Spec revision | verified-official-docs | "2026-10-10 \| 0.4" — changes: "Feature detection reply may list supported states and kinds. Say why control characters are refused and what `app` is for. Remove terminfo." | [program-status.md:313–317](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Use within Rex | verified-official-docs | "Sessions with completed or blocked tasks show up with an indicator in the session picker."; "Tab headers for unfocused tabs show an in-progress spinner and a variety symbols for blocked states."; "Lua scripts can react to `terminal.program_status_changed` events." | [program-status.md:30–38](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Planned future use | verified-official-docs | "In the future, we plan to expand this functionality in many more ways, such as push notifying our mobile applications, a unified priority inbox view, and many more." | [program-status.md:40–43](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Unterminated/malformed `ST` handling | unknown | `not documented in current docs` | [program-status.md:79–103](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| Relation between OSC 7501 and any native Rex client/server protocol | unknown | `not documented in current docs` — the page only contrasts OSC 7501 with other terminal sequences and calls it a generic terminal protocol | [program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |

## 12. Multi-client behavior

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Server keeps a session alive regardless of viewers | verified-official-docs | "The server keeps a session running whether or not anything is looking at it." | [cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Admin capability to list and force-disconnect clients | verified-official-docs | "it can create sessions, split panes, list active clients, force disconnect a client, etc." (no command names printed) | [cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Lua client listing | verified-official-docs | `rex.session.list_clients` — "List the clients attached to the session." | [lua.md:180](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Client identity fields | verified-official-docs | "`client` is a table with the client's `client_id`, `info` (its `kind`, `name`, `platform`, and `version`), `principal`, and `connected_at`." | [events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Session-level client connection events | verified-official-docs | `client_connected` — "A client attaches to the session."; `client_disconnected` — "A client detaches from the session."; `client_connection_changed` — "An attached client's connection is lost or restored." | [events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Server-level client events | verified-official-docs | `client_joined` — "A client connects to the server."; `client_left` — "A client disconnects from the server."; `client_changed` — "A connected client's details change." | [events.md:119–123](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Multiple client platforms exist | verified-official-docs | "We already have Rex servers running on Linux and Windows, and we have Rex clients for iOS, Linux, and Windows at varying stages of readiness." | [installation.md](https://www.superlogical.com/rex/docs/installation.md) · 2026-10-15T16:05:01Z |
| Current public availability | verified-official-docs | "At the current stage of beta testing, only the macOS application is available." | [installation.md](https://www.superlogical.com/rex/docs/installation.md) · 2026-10-15T16:05:01Z |
| Local server in the macOS client | verified-official-docs | "The macOS client is fully self-contained with a local server that lets you get up and running quickly." | [installation.md](https://www.superlogical.com/rex/docs/installation.md) · 2026-10-15T16:05:01Z |
| CLI attach/detach commands | unknown | `not documented in current docs` — `attach`/`detach` are documented as Lua `rex.session` methods (event scope), not as CLI commands | [lua.md:205](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Input/focus/layout arbitration when several clients are attached | unknown | `not documented in current docs` | [events.md:119–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Client `info.kind` allowed values | unknown | `not documented in current docs` | [events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |

## 13. Connection lifecycle

| Documented behavior | Label | Evidence (verbatim excerpt) | Source |
| --- | --- | --- | --- |
| Session survives client disconnection | verified-official-docs | "The server keeps a session running whether or not anything is looking at it." | [cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Attach/detach/lost/restored are observable | verified-official-docs | `client_connected` — "A client attaches to the session."; `client_disconnected` — "A client detaches from the session."; `client_connection_changed` — "An attached client's connection is lost or restored." | [events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| `connection_state` field exists | verified-official-docs | `client_connection_changed` payload includes `client_id`, `principal`, `connection_state` | [events.md:71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Allowed `connection_state` values | unknown | `not documented in current docs` | [events.md:71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Handshake, reconnect, or backoff behavior | unknown | `not documented in current docs` | [events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Authentication/authorization and identity (`principal`) semantics | unknown | `not documented in current docs` — `principal` appears in payloads only | [events.md:69–71, 128–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| Session survival across server restart or crash | unknown | `not documented in current docs` — no server-crash or reboot survival claim is made anywhere in the fetched docs | [cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| Host profile fields used for connecting | verified-official-docs | `rex.host{ label, endpoints, endpoint, id, icon }`; "`endpoints` — Required. A list of addresses, tried in order." | [lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Host profile is a client-side connection configuration | inferred | Justification: `rex.host` carries `label`, address lists, a stable `id` for renaming, and an `icon` for clients, and `-S` accepts "the label of a host from `rex hosts`"; together these imply saved client-side connection parameters, though the docs do not use that phrase. | [lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| Other servers are addressable from Lua | verified-official-docs | `rex.servers()` "Returns the labels of the other servers available to the running action or script, as a list."; `rex.server(label)` "Raises an error for an unknown label." | [lua.md:329–337](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |

## 14. Relayable deltas against `full.md` assumptions

Result values: **confirmed** (docs support the assumption), **partly contradicted** (docs narrow or correct it), **still unverified** (docs are silent or incomplete). Every row carries the label that applies to the evidence used. Delta rows cite `full.md` sections and the official docs; `full.md` is repository text, not a URL.

| `full.md` assumption (section) | Label | Result and evidence | Source |
| --- | --- | --- | --- |
| D-01 Candidate command family list: `relayable version`, `doctor`, `server status/run/inspect`, `ls`, `new`, `session inspect/rename/close`, `window list/new/close`, `split`, `run`, `block inspect/close`, `attach/detach`, `send/send-key/capture/wait`, `events`, `hosts list/add/remove/inspect`, `clients list/disconnect`, `api list/describe/call`, `config check/reload`, `do` (§8.1) | unknown | **still unverified** — the docs document the names in §7.1 only; the complete command tree is `not documented in current docs`; the candidate list must not be treated as verified. | [full.md §8.1](full.md); [cli.md:18–28](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-02 Human-readable output by default; machine-readable `--json` with stable schemas (§8.1) | verified-official-docs | **confirmed** at documentation level: "Pass `--json` to get JSON instead, which is stable to parse and includes full IDs". Per-command schemas remain undocumented (§7.4). | [full.md §8.1](full.md); [cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-03 Stable exit code semantics documented and tested (§8.1) | verified-official-docs | **confirmed** for documentation: the 0/1/2/3/4/130 table plus wait-command pass-through. No test evidence exists on this branch (no implementation), so the "tested" half is out of scope here. | [full.md §8.1](full.md); [cli.md:125–136](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-04 Context-aware defaults inside a managed terminal (§8.1) | verified-official-docs | **confirmed**: "the context is setup so that the CLI already knows what session you're in, block you're looking at". | [full.md §8.1](full.md); [cli.md:14–16](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-05 Explicit host/session/window/block/client targeting (§8.1) | verified-official-docs | **confirmed**: `-s`, `-w`, `-b`, `-C`, `-S` documented. | [full.md §8.1](full.md); [cli.md:63–68](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-06 Ambiguous targets fail before mutation; never guess silently (§8.1) | verified-official-docs | **confirmed**: "the command stops without changing anything, lists what matched, and exits with status 3". | [full.md §8.1](full.md); [cli.md:88–89](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-07 `REX_SESSION`, `REX_BLOCK`, `REX_SERVER` context variables (§8.1) | verified-official-docs | **confirmed**, and `REX_CLIENT` is additionally documented as the default for `-C`. | [full.md §8.1](full.md); [cli.md:56–67](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-08 Env namespace: `RELAYABLE_*`, REX aliases, or both; do not mix namespaces accidentally (§8.1; DEC-006, ADR-005) | unknown | **still unverified/open** — DEC-006 is Pending and points at `docs/adr/ADR-005`; the docs document only `REX_*` names, and the Relayable namespace is `not documented in current docs`. | [full.md §8.1](full.md); [DECISIONS.md](../agent-loop/DECISIONS.md); [cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-09 Typed API with request/response schemas and stable error codes (§8.2) | verified-official-docs | **confirmed** for schemas (JSON Schema) and named codes `bad_request`/`method_not_found`; the complete error-code set is still unknown (§8.3). | [full.md §8.2](full.md); [server-api.md:63–103, 157–159](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| D-10 Discovery equivalent to listing methods, describing schemas, and calling (§8.2) | verified-official-docs | **confirmed**: `rex api list`, `rex api describe`, `rex api call`, plus `--json`. | [full.md §8.2](full.md); [server-api.md:8–14, 236–248](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| D-11 Explicit method registration/authorization policy; no arbitrary internal methods (§8.2) | unknown | **still unverified** — docs describe discovery/calling, but method registration/authorization policy is `not documented in current docs`. | [full.md §8.2](full.md); [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| D-12 Lua 5.1 plus standard library as a candidate (§8.3) | verified-official-docs | **confirmed**: "Rex runs Lua 5.1 with the full standard library, including `io`, `os`, and `require`." | [full.md §8.3](full.md); [lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| D-13 Confirm Lua scope and licensing/embedding requirements (§8.3) | unknown | **still unverified** — the docs state the runtime; embedding and licensing terms are `not documented in current docs`. | [full.md §8.3](full.md); [lua.md](https://www.superlogical.com/rex/docs/reference/lua.md) · 2026-10-15T16:04:47Z |
| D-14 Event envelope with type, payload, and entity/session context (§8.4) | verified-official-docs | **confirmed**: `ev.type`, `ev.payload`, `ev.session_id`, plus block context fields. | [full.md §8.4](full.md); [events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| D-15 Envelope version, sequence number, and timestamp "where useful" (§8.4) | unknown | **still unverified** — a universal envelope version/sequence/timestamp is `not documented in current docs`; only per-event fields (`revision`, `generation`, `hash`, `connected_at`) are specified. | [full.md §8.4](full.md); [events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| D-16 Subscription scope (§8.4) | verified-official-docs | **confirmed**: default is one session; `--all` covers every session; the CLI has three documented scopes. | [full.md §8.4](full.md); [lua-events.md:91–100](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z; [events.md:153–159](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| D-17 Replay behavior and event retention (§8.4) | unknown | **still unverified** — `not documented in current docs`. | [full.md §8.4](full.md); [lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md) · 2026-10-15T16:04:47Z |
| D-18 Ignore unknown optional fields safely (§8.4) | verified-official-docs | **confirmed**: "Ignore fields you don't recognize rather than failing on them." | [full.md §8.4](full.md); [events.md:163–166](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| D-19 Potential reference events: session create/destroy, client join/leave/change, window/block lifecycle, terminal title/PWD/process changes, bell, child exit, program-status changes (§8.4) | verified-official-docs | **confirmed**: every listed family appears in §10.2–10.4. | [full.md §8.4](full.md); [events.md:59–129](https://www.superlogical.com/rex/docs/reference/events.md) · 2026-10-15T16:04:47Z |
| D-20 OSC 7501 states shorthand: `idle`, `working`, `done`, `blocked`, `error`, and `clear` (§8.5) | verified-official-docs | **partly contradicted** — `idle`, `working`, `done`, `blocked`, `error` are confirmed states; `clear` is documented as "Not a state." that removes records. | [full.md §8.5](full.md); [program-status.md:140–145](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-21 OSC 7501 possible progress, app identity, title/message, blocked kind, nested IDs (§8.5) | verified-official-docs | **confirmed**: `progress`, `app`, `title`, `msg`, `kind`, and `/`-nested `id`s are all specified. | [full.md §8.5](full.md); [program-status.md:113–139, 163–169](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-22 OSC 7501 exact encoding/base64 requirements (§8.5) | verified-official-docs | **confirmed**: `msg`/`title` are "standard base64 of UTF-8 text. Padding is optional." | [full.md §8.5](full.md); [program-status.md:96–100](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-23 OSC 7501 lifetime semantics (§8.5) | verified-official-docs | **confirmed**: per-state lifetimes plus MUST/MAY drop rules on process exit and shell prompt. | [full.md §8.5](full.md); [program-status.md:140–145, 154–165](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-24 OSC 7501 feature detection (§8.5) | verified-official-docs | **confirmed**: query/reply forms, `states`/`kinds` defaults, `CSI c` disambiguation. | [full.md §8.5](full.md); [program-status.md:172–198](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-25 OSC 7501 malformed sequence handling (§8.5) | verified-official-docs | **confirmed** for the documented cases (skipped malformed pairs, ignored bad `id`, whole-report discard). Remaining cases (e.g. unterminated `ST`) are unknown (§11.6). | [full.md §8.5](full.md); [program-status.md:88–103, 124–126](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-26 OSC 7501 unknown-state behavior (§8.5) | verified-official-docs | **confirmed**: "An unrecognized value causes the whole report to be ignored, so a state added later never turns into `idle` on an older terminal." | [full.md §8.5](full.md); [program-status.md:163](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-27 Treat OSC 7501 as a terminal-level integration, not the native Relayable wire protocol (§6, §8.5) | verified-official-docs | **direction supported** — the page calls OSC 7501 "a generic terminal protocol" with "no Superlogical-specific functionality or language"; the relation to any native protocol is not documented (D-28). | [full.md §8.5](full.md); [program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-28 Relation between the native client/server protocol and OSC 7501 (§6) | unknown | **still unverified** — `not documented in current docs`. | [full.md §6](full.md); [program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md) · 2026-10-15T16:04:48Z |
| D-29 Product naming: Relayable and the public CLI `relayable`; REX only as reference/compatibility naming (§0.1) | intentionally-different | **deliberate difference**: the reference docs name the CLI `rex`; `full.md` §0.1 says "The product name is **Relayable**." and "Preferred public CLI: `relayable`." | [full.md §0.1](full.md); [cli.md:8](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-30 Any REX binary/wire protocol documentation and interoperability (§3) | unknown | **still unverified** — any wire protocol is `not documented in current docs` and no interop test was run; this matrix claims none. | [full.md §3](full.md); [server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md) · 2026-10-15T16:07:39Z |
| D-31 Block first implementation is a terminal; no premature plugins (§4) | verified-official-docs | **confirmed**: "A **block** is one application in a window. Today that means a terminal." | [full.md §4](full.md); [cli.md:38](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |
| D-32 Stable IDs never reused; labels may change and are not permanent identities (§4) | verified-official-docs | **confirmed**: "Each one has an ID that never changes and is never reused"; labels "can change and two things can share one". | [full.md §4](full.md); [cli.md:43–45, 100–102](https://www.superlogical.com/rex/docs/automate/cli.md) · 2026-10-15T16:07:39Z |

## 15. Unknowns summary — `not documented in current docs`

Each item below is silent in the fetched official docs; the matrix rows above label these `unknown`. Items are grouped for the Wave 2 ADR/implementation plan; none of them may be filled in by invention.

- CLI: complete command tree; command aliases; pod commands from the `full.md` §8.1 candidate list; `--json` schemas for commands other than `rex ls`; exit-code mapping for `rex api call` failures; exit status when `rex do --for` expires; whether `REX_CLIENT` is set automatically; any env vars beyond the four `REX_*` names; whether label matching is exact or prefix-based.
- Server API: transport/wire format; authentication/authorization; version negotiation/capability handshake; complete error-code set; complete method inventory and every schema; naming/stability policy; pagination/streaming/rate limits; remote reachability.
- Lua: patch level/build flags/LuaJIT compatibility; licensing/embedding requirements; per-call timeouts and cancellation; outbound network capabilities beyond the standard library.
- Events: envelope version/global sequence/global timestamp; retention/replay; buffering or drop policy while handlers are busy; ordering across sessions; allowed `connection_state` values; client `info.kind` values; the relation between `terminal.progress_report` fields and OSC 7501 keys.
- OSC 7501: unterminated or malformed `ST`; a `?` query carrying other keys; reply interleaving with other output; key case sensitivity; whether reply `states`/`kinds` may be empty; relation to any native Rex client/server protocol.
- Multi-client/lifecycle: input, focus, and layout arbitration among several attached clients; CLI attach/detach commands; handshake/reconnect/backoff; authentication and `principal` semantics; session survival across server restart or crash.
- Community/process: sitemap-driven page discovery was impossible (`sitemap.xml` 404); `automate/shell-scripting` appears in the docs navigation but was not fetched by these research workers, so its content is not represented here.
