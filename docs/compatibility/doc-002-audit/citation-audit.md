# Citation line-number audit — Relayable reference behavior matrix (DOC-002)

**Scope:** every markdown-link citation in `docs/compatibility/reference-behavior-matrix.md` (read-only) checked against the fresh canonical raw Markdown sources in `/tmp/doc002/raw/`.
**Method:** fix-string search of a distinctive core substring of the cited quote, case-sensitive, after stripping `*`/`` ` ``/`…`/`...` and collapsing whitespace (with a whitespace-insensitive fallback); the full source is normalized into a char→line index so a multi-line quote yields its real line range. Helper script: `/tmp/doc002/audit.py`.

**Matrix length:** the matrix is 648 lines by `wc -l` (649 by the reader, i.e. a final line without a trailing newline). All matrix line numbers below are 1-based `wc -l`-style lines.

## Summary counts

- Total markdown-link citations found: **433**
  - documentation citations with explicit line numbers: **386**
  - documentation links with no line numbers (source `.md` only): **12**
  - repository-text links (`full.md`, `DECISIONS.md`): **34**

### Verdicts, line-number citations only (n=386)

| Verdict | Count |
| --- | ---: |
| MATCH | 193 |
| SHIFT | 115 |
| NOT_FOUND | 78 |
| AMBIGUOUS | 0 |
| **total** | **386** |

### Per-source-file breakdown (line-number citations)

| Source | MATCH | SHIFT | NOT_FOUND | AMBIGUOUS | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| cli.md | 56 | 8 | 17 | 0 | 81 |
| server-api.md | 29 | 6 | 13 | 0 | 48 |
| lua.md | 16 | 24 | 15 | 0 | 55 |
| events.md | 29 | 33 | 13 | 0 | 75 |
| lua-events.md | 16 | 3 | 3 | 0 | 22 |
| lua-scripts.md | 16 | 3 | 3 | 0 | 22 |
| program-status.md | 31 | 38 | 14 | 0 | 83 |
| installation.md | 0 | 0 | 0 | 0 | 0 |

### Per-source-file breakdown (all links, incl. no-line-ref)

| Source | MATCH | SHIFT | NOT_FOUND | AMBIGUOUS | NO_LINE_REF | total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cli.md | 56 | 8 | 17 | 0 | 0 | 81 |
| server-api.md | 29 | 6 | 13 | 0 | 6 | 54 |
| lua.md | 16 | 24 | 15 | 0 | 2 | 57 |
| events.md | 29 | 33 | 13 | 0 | 0 | 75 |
| lua-events.md | 16 | 3 | 3 | 0 | 0 | 22 |
| lua-scripts.md | 16 | 3 | 3 | 0 | 1 | 23 |
| program-status.md | 31 | 38 | 14 | 0 | 0 | 83 |
| installation.md | 0 | 0 | 0 | 0 | 3 | 3 |

## events.md headline (core deliverable)

- events.md citations: **75** — MATCH **29**, SHIFT **33**, NOT_FOUND **13**, AMBIGUOUS **0**.
- SHIFT or NOT_FOUND (i.e. the cited line number is wrong or the row has no verbatim quote): **46** of 75.
- The full correction list is in §"events.md corrections" below.

## Heading outlines (with line numbers)

`grep -n '^#\{1,4\} '` per file, with `#` lines that fall inside fenced code blocks marked. `installation.md` has no headings other than `# Installation` and no fenced blocks.

### cli.md (137 lines)
- 6: `# The CLI`
- 30: `## Taxonomy`
- 47: `## Command Targets`
- 76: `### Targets`
- 104: `## Machine-Readable Output`

### server-api.md (252 lines)
- 6: `# Server API`
- 24: `## Discovering Methods`
- 50: `## Reading a Method's Documentation`
- 105: `## Calling a Method`
- 147: `### Errors`
- 162: `## Calling From Lua`
- 199: `## Block Methods`
- 236: `## Machine-Readable Documentation`

### lua.md (342 lines)
- 6: `# Lua API`
- 12: `## Where Each Function Works`
- 48: `## Key Bindings`
- 53: `### `rex.bind(key, action[, args])``
- 65: `### `rex.bind(key, fn)``
- 73: `### `rex.unbind(key[, target])``
- 81: `### `rex.unbind{ mode = name }``
- 85: `### `rex.unbind(binding)``
- 89: `### `rex.mode(name[, options])``
- 94: `### `rex.bindings([filter])``
- 110: `### `rex.modes()``
- 115: `## Actions`
- 119: `### `rex.action{ ... }``
- 141: `### `rex.invoke(name[, args])``
- 151: `### `rex.client.queue(name[, args])``
- 160: `## Calling Rex`
- 167: `### `rex.session.<method>(args)``
- 215: `### `rex.call(method[, args])``
- 226: `### `rex.block.call(creator, method[, args])``
- 252: `### `rex.layout``
- 273: `## Events`
- 278: `### `rex.on(event, fn)``
- 285: `### `rex.wait(event[, filter][, timeout])``
- 293: `### `rex.sleep(seconds)``
- 297: `### `rex.stop([code])``
- 301: `## Script Values`
- 311: `## Hosts and Other Servers`
- 313: `### `rex.host{ ... }``
- 325: `### `rex.servers()``
- 330: `### `rex.server(label)``
- 337: `## Logging`
- 339: `### `rex.log([level, ]message)``

### events.md (165 lines)
- 6: `# Events`
- 21: `## Events in Lua`
- 54: `## Session Events`
- 78: `## Terminal Events`
- 108: `## Server Events`
- 127: `## Events on the CLI`
- 161: `## Unknown Events`

### lua-events.md (186 lines)
- 6: `# Lua Events`
- 30: `## Handlers`
- 58: `## Event Names`
- 89: `## Event Scope`
- 106: `## Stopping`
- 124: `## Handler Failures`
- 138: `## Handlers or `rex.wait``
- 151: `## Invalid in Configuration Files`
- 160: `## Example: Notify on Bell`

### lua-scripts.md (243 lines)
- 6: `# Lua Scripts`
- 34: `## Running a Script`
- 60: `## Arguments`
- 84: `## Results and Errors`
- 111: `## Calling Rex`
- 155: `### Layouts`
- 176: `## Sleeping and Waiting`
- 204: `## Example: Run Tests in a Split`

### program-status.md (320 lines)
- 6: `# Program Status Protocol (OSC 7501)`
- 17: `# Terraform has finished planning and is waiting for the user to approve` — **code block, not a heading**
- 18: `# the apply. The message decodes to "Apply 3 to add, 1 to change, 0 to destroy?".` — **code block, not a heading**
- 26: `## Motivation`
- 45: `## In Rex`
- 60: `## Conventions`
- 77: `## Syntax`
- 104: `## Records and ids`
- 133: `## States`
- 159: `## Keys`
- 171: `## Feature detection`
- 198: `## Security`
- 217: `## Limits`
- 235: `## Relationship to other sequences`
- 271: `## Examples`
- 275: `### One program`
- 287: `### Several records`
- 303: `### A shell function`
- 313: `## Revision history`

### installation.md (19 lines)
- 6: `# Installation`

## Citations without line numbers (“no line ref”)

| matrix line | section | raw citation | source | kind |
| ---: | --- | --- | --- | --- |
| 131 | 7.1 Documented commands by family | `[full.md §8.1](full.md)` | full.md | repo |
| 211 | 8.2 Schemas and examples | `[JSON Schema](https://json-schema.org)` | None | other |
| 237 | 8.3 Calling methods, session defaulting, errors | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 238 | 8.3 Calling methods, session defaulting, errors | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 240 | 8.3 Calling methods, session defaulting, errors | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 241 | 8.3 Calling methods, session defaulting, errors | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 268 | 9.1 Runtime | `[lua.md](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | noline |
| 348 | 9.4 `rex do` script semantics | `[lua-scripts.md](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | noline |
| 577 | 12. Multi-client behavior | `[installation.md](https://www.superlogical.com/rex/docs/installation.md)` | installation.md | noline |
| 578 | 12. Multi-client behavior | `[installation.md](https://www.superlogical.com/rex/docs/installation.md)` | installation.md | noline |
| 579 | 12. Multi-client behavior | `[installation.md](https://www.superlogical.com/rex/docs/installation.md)` | installation.md | noline |
| 605 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 606 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 607 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 608 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 609 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 610 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 611 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 612 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.1](full.md)` | full.md | repo |
| 612 | 14. Relayable deltas against `full.md` assumptions | `[DECISIONS.md](../agent-loop/DECISIONS.md)` | DECISIONS.md | repo |
| 613 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.2](full.md)` | full.md | repo |
| 614 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.2](full.md)` | full.md | repo |
| 615 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.2](full.md)` | full.md | repo |
| 615 | 14. Relayable deltas against `full.md` assumptions | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 616 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.3](full.md)` | full.md | repo |
| 617 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.3](full.md)` | full.md | repo |
| 617 | 14. Relayable deltas against `full.md` assumptions | `[lua.md](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | noline |
| 618 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 619 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 620 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 621 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 622 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 623 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.4](full.md)` | full.md | repo |
| 624 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 625 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 626 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 627 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 628 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 629 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 630 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 631 | 14. Relayable deltas against `full.md` assumptions | `[full.md §8.5](full.md)` | full.md | repo |
| 632 | 14. Relayable deltas against `full.md` assumptions | `[full.md §6](full.md)` | full.md | repo |
| 633 | 14. Relayable deltas against `full.md` assumptions | `[full.md §0.1](full.md)` | full.md | repo |
| 634 | 14. Relayable deltas against `full.md` assumptions | `[full.md §3](full.md)` | full.md | repo |
| 634 | 14. Relayable deltas against `full.md` assumptions | `[server-api.md](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | noline |
| 635 | 14. Relayable deltas against `full.md` assumptions | `[full.md §4](full.md)` | full.md | repo |
| 636 | 14. Relayable deltas against `full.md` assumptions | `[full.md §4](full.md)` | full.md | repo |

## Full citation table

`actual` is the machine-located line/range of the cited quote; `verdict` is adjudicated. Rows marked with † were manually adjudicated (code-fragment union artifact or paraphrase anchor).

| matrix line | table/section | raw citation | source | cited lines | actual lines found | verdict | corrected ref |
| ---: | --- | --- | --- | --- | --- | --- | --- |
| 73 | 5. Domain hierarchy and stable IDs | `[cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 34-36 | 35-36 | MATCH | unchanged; cited correct |
| 74 | 5. Domain hierarchy and stable IDs | `[cli.md:37](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 37 | 37 | MATCH | unchanged; cited correct |
| 75 | 5. Domain hierarchy and stable IDs | `[cli.md:38](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 38 | 38 | MATCH | unchanged; cited correct |
| 76 | 5. Domain hierarchy and stable IDs | `[cli.md:39–41](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 39-41 | 39-41 | MATCH | unchanged; cited correct |
| 77 | 5. Domain hierarchy and stable IDs | `[cli.md:34–38](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 34-38 | 37-38 | MATCH | unchanged; cited correct |
| 78 | 5. Domain hierarchy and stable IDs | `[cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 43-45 | 44-45 | MATCH | unchanged; cited correct |
| 79 | 5. Domain hierarchy and stable IDs | `[cli.md:100–102](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 100-102 | 100-101 | MATCH | unchanged; cited correct |
| 80 | 5. Domain hierarchy and stable IDs | `[cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 43-45 | 43-44 | MATCH | unchanged; cited correct |
| 81† | 5. Domain hierarchy and stable IDs | `[cli.md:81](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 81 | 81 | MATCH | unchanged (cli.md:81) |
| 81† | 5. Domain hierarchy and stable IDs | `[server-api.md:85–86](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 85-86 | 85-86 | MATCH | unchanged (server-api.md:85–86) |
| 82 | 5. Domain hierarchy and stable IDs | `[cli.md:43–45](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 43-45 | — | NOT_FOUND | drop number; heading `## Taxonomy` (raw line 30) + quote |
| 83 | 5. Domain hierarchy and stable IDs | `[cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 68 | 68 | MATCH | unchanged; cited correct |
| 84 | 5. Domain hierarchy and stable IDs | `[events.md:107–108](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 107-108 | 110 | SHIFT | events.md:110 |
| 85† | 5. Domain hierarchy and stable IDs | `[events.md:116](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 116 | — | NOT_FOUND | keep 116 (session_destroyed event); no verbatim quote — nearest heading `## Server Events` (raw line 108) |
| 91 | 6. Target resolution | `[cli.md:47–51](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 47-51 | 49-51 | MATCH | unchanged; cited correct |
| 92 | 6. Target resolution | `[cli.md:52–53](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 52-53 | 51-52 | SHIFT | cli.md:51–52 |
| 93 | 6. Target resolution | `[cli.md:78–80](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 78-80 | 78-79 | MATCH | unchanged; cited correct |
| 94 | 6. Target resolution | `[cli.md:81](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 81 | 81 | MATCH | unchanged; cited correct |
| 95 | 6. Target resolution | `[cli.md:82](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 82 | 82 | MATCH | unchanged; cited correct |
| 96 | 6. Target resolution | `[cli.md:83–84](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 83-84 | 83-84 | MATCH | unchanged; cited correct |
| 97 | 6. Target resolution | `[cli.md:85–86](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 85-86 | 85-86 | MATCH | unchanged; cited correct |
| 98 | 6. Target resolution | `[cli.md:88–89](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 88-89 | 88-89 | MATCH | unchanged; cited correct |
| 99 | 6. Target resolution | `[cli.md:92–93](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 92-93 | — | NOT_FOUND | drop number; heading `### Targets` (raw line 76) + quote |
| 100 | 6. Target resolution | `[cli.md:97–98](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 97-98 | 97-98 | MATCH | unchanged; cited correct |
| 101 | 6. Target resolution | `[cli.md:100–102](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 100-102 | 101-102 | MATCH | unchanged; cited correct |
| 102 | 6. Target resolution | `[cli.md:82](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 82 | — | NOT_FOUND | drop number; heading `### Targets` (raw line 76) + quote |
| 110 | 7.1 Documented commands by family | `[cli.md:8](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 8 | 8 | MATCH | unchanged; cited correct |
| 111 | 7.1 Documented commands by family | `[cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 10-12 | 11-12 | MATCH | unchanged; cited correct |
| 112 | 7.1 Documented commands by family | `[cli.md:12](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 12 | 12-13 | SHIFT | cli.md:12–13 |
| 113 | 7.1 Documented commands by family | `[cli.md:14–16](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 14-16 | 15-17 | SHIFT | cli.md:15–17 |
| 114 | 7.1 Documented commands by family | `[cli.md:27–28](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 27-28 | 27-28 | MATCH | unchanged; cited correct |
| 115 | 7.1 Documented commands by family | `[cli.md:22–24](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 22-24 | 22 | MATCH | unchanged; cited correct |
| 116 | 7.1 Documented commands by family | `[cli.md:23](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 23 | 23 | MATCH | unchanged; cited correct |
| 117 | 7.1 Documented commands by family | `[cli.md:24](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 24 | 24 | MATCH | unchanged; cited correct |
| 118 | 7.1 Documented commands by family | `[cli.md:49–51](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 49-51 | 50-51 | MATCH | unchanged; cited correct |
| 119 | 7.1 Documented commands by family | `[cli.md:70–71](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 70-71 | 71 | MATCH | unchanged; cited correct |
| 120 | 7.1 Documented commands by family | `[cli.md:71–72](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 71-72 | 72 | MATCH | unchanged; cited correct |
| 121 | 7.1 Documented commands by family | `[cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 68 | 68 | MATCH | unchanged; cited correct |
| 122 | 7.1 Documented commands by family | `[cli.md:73](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 73 | 73 | MATCH | unchanged; cited correct |
| 123 | 7.1 Documented commands by family | `[cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 136 | 136-137 | SHIFT | cli.md:136–137 |
| 124 | 7.1 Documented commands by family | `[events.md:10–11, 153–159](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 10-11;153-159 | 11 | MATCH | unchanged; cited correct |
| 125 | 7.1 Documented commands by family | `[server-api.md:12–14](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 12-14 | 12-14;26;29;43;110;201;242 | SHIFT | server-api.md:12–14, 26, 29, 43, 110, 201, 242 |
| 126 | 7.1 Documented commands by family | `[lua-scripts.md:8](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 8 | — | NOT_FOUND | drop number; heading `# Lua Scripts` (raw line 6) + quote |
| 127 | 7.1 Documented commands by family | `[cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 10-12 | 11-12 | MATCH | unchanged; cited correct |
| 128 | 7.1 Documented commands by family | `[cli.md:27](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 27 | 27 | MATCH | unchanged; cited correct |
| 129 | 7.1 Documented commands by family | `[cli.md:18–28](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 18-28 | 27 | MATCH | unchanged; cited correct |
| 130 | 7.1 Documented commands by family | `[cli.md:27](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 27 | — | NOT_FOUND | drop number; heading `# The CLI` (raw line 6) + quote |
| 137 | 7.2 Flags and aliases | `[cli.md:64](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 64 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 138 | 7.2 Flags and aliases | `[cli.md:65](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 65 | 65 | MATCH | unchanged; cited correct |
| 139 | 7.2 Flags and aliases | `[cli.md:66](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 66 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 140 | 7.2 Flags and aliases | `[cli.md:67](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 67 | 67 | MATCH | unchanged; cited correct |
| 141 | 7.2 Flags and aliases | `[cli.md:68](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 68 | 68 | MATCH | unchanged; cited correct |
| 142 | 7.2 Flags and aliases | `[cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 106-107 | 106-107 | MATCH | unchanged; cited correct |
| 143 | 7.2 Flags and aliases | `[server-api.md:43](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 43 | 40 | SHIFT | server-api.md:40 |
| 144 | 7.2 Flags and aliases | `[events.md:159](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 159 | 159 | MATCH | unchanged; cited correct |
| 145 | 7.2 Flags and aliases | `[lua-events.md:99–100](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 99-100 | 99-100 | MATCH | unchanged; cited correct |
| 146 | 7.2 Flags and aliases | `[lua-events.md:112](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 112 | 112 | MATCH | unchanged; cited correct |
| 147 | 7.2 Flags and aliases | `[lua-events.md:133](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 133 | 133 | MATCH | unchanged; cited correct |
| 148 | 7.2 Flags and aliases | `[lua-scripts.md:77](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 77 | 77 | MATCH | unchanged; cited correct |
| 149 | 7.2 Flags and aliases | `[lua-scripts.md:40](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 40 | 40 | MATCH | unchanged; cited correct |
| 150 | 7.2 Flags and aliases | `[cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 136 | 136 | MATCH | unchanged; cited correct |
| 151 | 7.2 Flags and aliases | `[cli.md:63–68](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 63-68 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 157 | 7.3 Environment variables | `[cli.md:56](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 56 | 56 | MATCH | unchanged; cited correct |
| 158 | 7.3 Environment variables | `[cli.md:57](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 57 | 57 | MATCH | unchanged; cited correct |
| 159 | 7.3 Environment variables | `[cli.md:58](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 58 | 58 | MATCH | unchanged; cited correct |
| 160 | 7.3 Environment variables | `[cli.md:67](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 67 | 67 | MATCH | unchanged; cited correct |
| 161 | 7.3 Environment variables | `[cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 52-58 | 51-52 | SHIFT | cli.md:51–52 |
| 162 | 7.3 Environment variables | `[cli.md:52–58, 67](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 52-58;67 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 163 | 7.3 Environment variables | `[cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 52-58 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 169 | 7.4 JSON output | `[cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 106-107 | 106-107 | MATCH | unchanged; cited correct |
| 170 | 7.4 JSON output | `[cli.md:110–121](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 110-121 | 81;114;118 | SHIFT | cli.md:81, 114, 118 |
| 171 | 7.4 JSON output | `[cli.md:106–121](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 106-121 | — | NOT_FOUND | drop number; heading `## Machine-Readable Output` (raw line 104) + quote |
| 177 | 7.5 Exit codes | `[cli.md:129](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 129 | 129 | MATCH | unchanged; cited correct |
| 178 | 7.5 Exit codes | `[cli.md:130](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 130 | 130 | MATCH | unchanged; cited correct |
| 179 | 7.5 Exit codes | `[cli.md:131](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 131 | 131 | MATCH | unchanged; cited correct |
| 180 | 7.5 Exit codes | `[cli.md:132](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 132 | 132 | MATCH | unchanged; cited correct |
| 181 | 7.5 Exit codes | `[cli.md:133](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 133 | 133 | MATCH | unchanged; cited correct |
| 182 | 7.5 Exit codes | `[cli.md:134](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 134 | 134 | MATCH | unchanged; cited correct |
| 183 | 7.5 Exit codes | `[cli.md:136](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 136 | 136-137 | SHIFT | cli.md:136–137 |
| 184 | 7.5 Exit codes | `[server-api.md:146–154](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 146-154 | 149-150 | MATCH | unchanged; cited correct |
| 185 | 7.5 Exit codes | `[lua-events.md:112](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 112 | — | NOT_FOUND | drop number; heading `## Stopping` (raw line 106) + quote |
| 193 | 8.1 Discovery and version coupling | `[server-api.md:8–9](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 8-9 | 8-9 | MATCH | unchanged; cited correct |
| 194 | 8.1 Discovery and version coupling | `[server-api.md:17–18](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 17-18 | 17-18 | MATCH | unchanged; cited correct |
| 195 | 8.1 Discovery and version coupling | `[server-api.md:20–21](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 20-21 | 20-22 | SHIFT | server-api.md:20–22 |
| 196 | 8.1 Discovery and version coupling | `[server-api.md:26–34](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 26-34 | 26 | MATCH | unchanged; cited correct |
| 197 | 8.1 Discovery and version coupling | `[server-api.md:31–34](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 31-34 | 31-33 | MATCH | unchanged; cited correct |
| 198 | 8.1 Discovery and version coupling | `[server-api.md:36–37](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 36-37 | — | NOT_FOUND | drop number; heading `## Discovering Methods` (raw line 24) + quote |
| 199 | 8.1 Discovery and version coupling | `[server-api.md:43–46](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 43-46 | 40 | SHIFT | server-api.md:40 |
| 200 | 8.1 Discovery and version coupling | `[server-api.md:52–53](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 52-53 | 52-53 | MATCH | unchanged; cited correct |
| 201 | 8.1 Discovery and version coupling | `[server-api.md:59](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 59 | — | NOT_FOUND | drop number; heading `## Reading a Method's Documentation` (raw line 50) + quote |
| 202 | 8.1 Discovery and version coupling | `[server-api.md:246–248](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 246-248 | — | NOT_FOUND | drop number; heading `## Machine-Readable Documentation` (raw line 236) + quote |
| 203 | 8.1 Discovery and version coupling | `[server-api.md:26–46, 59, 110–135, 201–233](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 26-46;59;110-135;201-233 | — | NOT_FOUND | drop number; heading `## Discovering Methods` (raw line 24) + quote |
| 203 | 8.1 Discovery and version coupling | `[lua.md:173–250](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 173-250 | — | NOT_FOUND | drop number; heading `### `rex.session.<method>(args)`` (raw line 167) + quote |
| 204 | 8.1 Discovery and version coupling | `[server-api.md:26–34](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 26-34 | — | NOT_FOUND | drop number; heading `## Discovering Methods` (raw line 24) + quote |
| 205 | 8.1 Discovery and version coupling | `[server-api.md:26–34, 201–216](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 26-34;201-216 | — | NOT_FOUND | drop number; heading `## Discovering Methods` (raw line 24) + quote |
| 211 | 8.2 Schemas and examples | `[server-api.md:94](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 94 | — | NOT_FOUND | drop number; heading `## Reading a Method's Documentation` (raw line 50) + quote |
| 212 | 8.2 Schemas and examples | `[server-api.md:96](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 96 | 96 | MATCH | unchanged; cited correct |
| 213 | 8.2 Schemas and examples | `[server-api.md:97](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 97 | 97 | MATCH | unchanged; cited correct |
| 214 | 8.2 Schemas and examples | `[server-api.md:98–99](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 98-99 | 98-99 | MATCH | unchanged; cited correct |
| 215† | 8.2 Schemas and examples | `[server-api.md:63–71](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 63-71 | 65-67,69-70 | MATCH | unchanged (server-api.md:63–71 covers the schema sample) |
| 216 | 8.2 Schemas and examples | `[server-api.md:75–81](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 75-81 | 70;80;98 | SHIFT | server-api.md:70, 80, 98 |
| 217 | 8.2 Schemas and examples | `[server-api.md:84–88](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 84-88 | 85-86 | MATCH | unchanged; cited correct |
| 218 | 8.2 Schemas and examples | `[server-api.md:91](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 91 | — | NOT_FOUND | drop number; heading `## Reading a Method's Documentation` (raw line 50) + quote |
| 219 | 8.2 Schemas and examples | `[server-api.md:102–103](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 102-103 | 102-103 | MATCH | unchanged; cited correct |
| 220 | 8.2 Schemas and examples | `[server-api.md:94](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 94 | — | NOT_FOUND | drop number; heading `## Reading a Method's Documentation` (raw line 50) + quote |
| 226 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:107](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 107 | 107 | MATCH | unchanged; cited correct |
| 227 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:110–115](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 110-115 | 113 | MATCH | unchanged; cited correct |
| 228 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:118–125](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 118-125 | 118-119 | MATCH | unchanged; cited correct |
| 229 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:127–129](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 127-129 | 127-128 | MATCH | unchanged; cited correct |
| 230 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:134–135](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 134-135 | 134-135 | MATCH | unchanged; cited correct |
| 231 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:137–138](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 137-138 | 137-138 | MATCH | unchanged; cited correct |
| 232 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:140–142](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 140-142 | — | NOT_FOUND | drop number; heading `## Calling a Method` (raw line 105) + quote |
| 233 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:149–154](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 149-154 | 154 | MATCH | unchanged; cited correct |
| 234 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:157–158](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 157-158 | 157-158 | MATCH | unchanged; cited correct |
| 235 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:158–159](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 158-159 | 158-160 | SHIFT | server-api.md:158–160 |
| 236 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:157–159](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 157-159 | — | NOT_FOUND | drop number; heading `### Errors` (raw line 147) + quote |
| 239 | 8.3 Calling methods, session defaulting, errors | `[server-api.md:17–18, 157–159](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 17-18;157-159 | 18 | MATCH | unchanged; cited correct |
| 247 | 8.4 Lua bindings of the server API | `[server-api.md:164–166](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 164-166 | 164 | MATCH | unchanged; cited correct |
| 248 | 8.4 Lua bindings of the server API | `[server-api.md:172–178](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 172-178 | — | NOT_FOUND | drop number; heading `## Calling From Lua` (raw line 162) + quote |
| 249 | 8.4 Lua bindings of the server API | `[server-api.md:181–183](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 181-183 | 181-183 | MATCH | unchanged; cited correct |
| 250 | 8.4 Lua bindings of the server API | `[server-api.md:185–187](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 185-187 | 185-186 | MATCH | unchanged; cited correct |
| 251 | 8.4 Lua bindings of the server API | `[server-api.md:189–193](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 189-193 | 189 | MATCH | unchanged; cited correct |
| 252 | 8.4 Lua bindings of the server API | `[server-api.md:219–225](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 219-225 | 219-220 | MATCH | unchanged; cited correct |
| 253 | 8.4 Lua bindings of the server API | `[server-api.md:227–233](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 227-233 | 227-228 | MATCH | unchanged; cited correct |
| 261 | 9.1 Runtime | `[lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 8-9 | 9-10 | SHIFT | lua.md:9–10 |
| 262 | 9.1 Runtime | `[lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 8-9 | 8-9 | MATCH | unchanged; cited correct |
| 263 | 9.1 Runtime | `[lua.md:11–26](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 11-26 | 14-15 | MATCH | unchanged; cited correct |
| 264 | 9.1 Runtime | `[lua.md:44–45](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 44-45 | 41-42 | SHIFT | lua.md:41–42 |
| 265 | 9.1 Runtime | `[lua.md:46–47](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 46-47 | 43-44 | SHIFT | lua.md:43–44 |
| 266 | 9.1 Runtime | `[lua.md:48](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 48 | 45-46 | SHIFT | lua.md:45–46 |
| 267 | 9.1 Runtime | `[lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 8-9 | — | NOT_FOUND | drop number; heading `# Lua API` (raw line 6) + quote |
| 274 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 275 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 276 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 277 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 278 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 279 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 280 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 281 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 282 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 283 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 284 | 9.1 Runtime | `[lua.md:28–39](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 28-39 | — | NOT_FOUND | drop number; heading `## Where Each Function Works` (raw line 12) + quote |
| 290 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:60–76](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 60-76 | 67-68 | MATCH | unchanged; cited correct |
| 291 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:78–80](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 78-80 | 70-71 | SHIFT | lua.md:70–71 |
| 292 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:82–97](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 82-97 | 75 | SHIFT | lua.md:75 |
| 293 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:99–101](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 99-101 | 91-92 | SHIFT | lua.md:91–92 |
| 294 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:103–127](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 103-127 | 96-97;112-113 | SHIFT | lua.md:96–97, 112–113 |
| 295† | 9.2 Key bindings, actions, hosts, logging | `[lua.md:131–146](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 131-146 | 125-132 | SHIFT | lua.md:125–132 |
| 296 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:150–152](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 150-152 | 137-138 | SHIFT | lua.md:137–138 |
| 297 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:154–161](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 154-161 | 143 | SHIFT | lua.md:143 |
| 298 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:163](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 163 | 149 | SHIFT | lua.md:149 |
| 299 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:165–172](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 165-172 | 153;158 | SHIFT | lua.md:153, 158 |
| 300† | 9.2 Key bindings, actions, hosts, logging | `[lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 313-327 | 317-323 | MATCH | unchanged (lua.md:313–327 covers the host table) |
| 301 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:329–337](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 329-337 | 327-328;332-333;335 | SHIFT | lua.md:327–328, 332–333, 335 |
| 302 | 9.2 Key bindings, actions, hosts, logging | `[lua.md:339–341](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 339-341 | 341-342 | SHIFT | lua.md:341–342 |
| 308 | 9.3 Call surface | `[lua.md:164–165](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 164-165 | 164-165 | MATCH | unchanged; cited correct |
| 309 | 9.3 Call surface | `[lua.md:167–171](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 167-171 | 169-170 | MATCH | unchanged; cited correct |
| 310 | 9.3 Call surface | `[lua.md:215–222](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 215-222 | 217-218 | MATCH | unchanged; cited correct |
| 311 | 9.3 Call surface | `[lua.md:226–234](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 226-234 | 229 | MATCH | unchanged; cited correct |
| 312 | 9.3 Call surface | `[lua.md:178–206](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 178-206 | 178-181;206 | MATCH | unchanged; cited correct |
| 313 | 9.3 Call surface | `[lua.md:180–186](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 180-186 | 182-185 | MATCH | unchanged; cited correct |
| 314 | 9.3 Call surface | `[lua.md:183–184](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 183-184 | 186-187 | SHIFT | lua.md:186–187 |
| 315 | 9.3 Call surface | `[lua.md:185–190](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 185-190 | 188-192 | SHIFT | lua.md:188–192 |
| 316 | 9.3 Call surface | `[lua.md:187–190](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 187-190 | 193-195 | SHIFT | lua.md:193–195 |
| 317 | 9.3 Call surface | `[lua.md:191–198](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 191-198 | 196-201 | SHIFT | lua.md:196–201 |
| 318 | 9.3 Call surface | `[lua.md:199–201](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 199-201 | 202-204 | SHIFT | lua.md:202–204 |
| 319 | 9.3 Call surface | `[lua.md:205](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 205 | 205 | MATCH | unchanged; cited correct |
| 320 | 9.3 Call surface | `[lua.md:237–243](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 237-243 | 240-244 | SHIFT | lua.md:240–244 |
| 321 | 9.3 Call surface | `[lua.md:242–244](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 242-244 | 245-247 | SHIFT | lua.md:245–247 |
| 322 | 9.3 Call surface | `[lua.md:244–250](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 244-250 | 248-250 | MATCH | unchanged; cited correct |
| 323 | 9.3 Call surface | `[lua.md:252–262](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 252-262 | 255 | MATCH | unchanged; cited correct |
| 324 | 9.3 Call surface | `[lua.md:264–274](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 264-274 | 268 | MATCH | unchanged; cited correct |
| 330 | 9.4 `rex do` script semantics | `[lua-scripts.md:18–20](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 18-20 | — | NOT_FOUND | drop number; heading `# Lua Scripts` (raw line 6) + quote |
| 331 | 9.4 `rex do` script semantics | `[lua-scripts.md:38–41](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 38-41 | 41 | MATCH | unchanged; cited correct |
| 332 | 9.4 `rex do` script semantics | `[lua-scripts.md:43–45](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 43-45 | 44 | MATCH | unchanged; cited correct |
| 333 | 9.4 `rex do` script semantics | `[lua-scripts.md:47–52](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 47-52 | 47 | MATCH | unchanged; cited correct |
| 334 | 9.4 `rex do` script semantics | `[lua-scripts.md:54–55](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 54-55 | 54-55 | MATCH | unchanged; cited correct |
| 335 | 9.4 `rex do` script semantics | `[lua-scripts.md:57–58](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 57-58 | 57-58 | MATCH | unchanged; cited correct |
| 336 | 9.4 `rex do` script semantics | `[lua-scripts.md:65–75](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 65-75 | 75-76 | SHIFT | lua-scripts.md:75–76 |
| 337 | 9.4 `rex do` script semantics | `[lua-scripts.md:77–81](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 77-81 | 77 | MATCH | unchanged; cited correct |
| 338 | 9.4 `rex do` script semantics | `[lua-scripts.md:85–88](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 85-88 | 86-88 | MATCH | unchanged; cited correct |
| 339 | 9.4 `rex do` script semantics | `[lua-scripts.md:95–98](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 95-98 | 95 | MATCH | unchanged; cited correct |
| 340 | 9.4 `rex do` script semantics | `[lua-scripts.md:100–106](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 100-106 | 102-103 | MATCH | unchanged; cited correct |
| 341 | 9.4 `rex do` script semantics | `[lua-scripts.md:108–109](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 108-109 | 109 | MATCH | unchanged; cited correct |
| 342 | 9.4 `rex do` script semantics | `[lua-scripts.md:133–140](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 133-140 | 135-136 | MATCH | unchanged; cited correct |
| 343 | 9.4 `rex do` script semantics | `[lua-scripts.md:112–116](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 112-116 | 131-133 | SHIFT | lua-scripts.md:131–133 |
| 344 | 9.4 `rex do` script semantics | `[lua-scripts.md:195–200](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 195-200 | 199-200 | MATCH | unchanged; cited correct |
| 345 | 9.4 `rex do` script semantics | `[lua-scripts.md:183–196](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 183-196 | 194-196 | MATCH | unchanged; cited correct |
| 346 | 9.4 `rex do` script semantics | `[lua-scripts.md:184–196](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 184-196 | 197 | SHIFT | lua-scripts.md:197 |
| 347 | 9.4 `rex do` script semantics | `[lua-scripts.md:241–242](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 241-242 | 242 | MATCH | unchanged; cited correct |
| 349 | 9.4 `rex do` script semantics | `[lua-scripts.md:57–58](https://www.superlogical.com/rex/docs/automate/lua-scripts.md)` | lua-scripts.md | 57-58 | — | NOT_FOUND | drop number; heading `## Running a Script` (raw line 34) + quote |
| 357 | 10.1 Sources, naming, envelope | `[events.md:13–20](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 13-20 | 17-19 | MATCH | unchanged; cited correct |
| 358 | 10.1 Sources, naming, envelope | `[events.md:16](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 16 | 17 | SHIFT | events.md:17 |
| 359 | 10.1 Sources, naming, envelope | `[events.md:18](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 18 | 18 | MATCH | unchanged; cited correct |
| 360 | 10.1 Sources, naming, envelope | `[lua-events.md:69–71](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 69-71 | 69-70 | MATCH | unchanged; cited correct |
| 361 | 10.1 Sources, naming, envelope | `[events.md:36](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 36 | 36 | MATCH | unchanged; cited correct |
| 362 | 10.1 Sources, naming, envelope | `[events.md:37](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 37 | 37 | MATCH | unchanged; cited correct |
| 363 | 10.1 Sources, naming, envelope | `[events.md:38](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 38 | 38 | MATCH | unchanged; cited correct |
| 364 | 10.1 Sources, naming, envelope | `[events.md:43–46](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 43-46 | 44-46 | MATCH | unchanged; cited correct |
| 365 | 10.1 Sources, naming, envelope | `[events.md:51–52](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 51-52 | 51-52 | MATCH | unchanged; cited correct |
| 366 | 10.1 Sources, naming, envelope | `[events.md:48–49](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 48-49 | 48-49 | MATCH | unchanged; cited correct |
| 367 | 10.1 Sources, naming, envelope | `[events.md:74–76](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 74-76 | 74-75 | MATCH | unchanged; cited correct |
| 368† | 10.1 Sources, naming, envelope | `[events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 34-46 | — | NOT_FOUND | drop number; heading `## Events in Lua` (raw line 21) + quote (field definitions at 34–46) |
| 374 | 10.2 Session events | `[events.md:59](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 59 | 61 | SHIFT | events.md:61 |
| 375 | 10.2 Session events | `[events.md:60](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 60 | 62 | SHIFT | events.md:62 |
| 376 | 10.2 Session events | `[events.md:61](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 61 | 63 | SHIFT | events.md:63 |
| 377 | 10.2 Session events | `[events.md:62](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 62 | 64 | SHIFT | events.md:64 |
| 378 | 10.2 Session events | `[events.md:63](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 63 | 65 | SHIFT | events.md:65 |
| 379 | 10.2 Session events | `[events.md:64](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 64 | 66 | SHIFT | events.md:66 |
| 380 | 10.2 Session events | `[events.md:65](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 65 | 67 | SHIFT | events.md:67 |
| 381 | 10.2 Session events | `[events.md:66](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 66 | 68 | SHIFT | events.md:68 |
| 382 | 10.2 Session events | `[events.md:69](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 69 | 69 | MATCH | unchanged; cited correct |
| 383 | 10.2 Session events | `[events.md:70](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 70 | 70 | MATCH | unchanged; cited correct |
| 384 | 10.2 Session events | `[events.md:71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 71 | 71 | MATCH | unchanged; cited correct |
| 385 | 10.2 Session events | `[events.md:72](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 72 | 72 | MATCH | unchanged; cited correct |
| 391 | 10.3 Terminal events | `[events.md:82](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 82 | 85 | SHIFT | events.md:85 |
| 392 | 10.3 Terminal events | `[events.md:83](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 83 | 86 | SHIFT | events.md:86 |
| 393 | 10.3 Terminal events | `[events.md:84](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 84 | 87 | SHIFT | events.md:87 |
| 394 | 10.3 Terminal events | `[events.md:85](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 85 | 88 | SHIFT | events.md:88 |
| 395 | 10.3 Terminal events | `[events.md:86](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 86 | 89 | SHIFT | events.md:89 |
| 396 | 10.3 Terminal events | `[events.md:87](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 87 | 90 | SHIFT | events.md:90 |
| 397 | 10.3 Terminal events | `[events.md:88](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 88 | 91 | SHIFT | events.md:91 |
| 398 | 10.3 Terminal events | `[events.md:89](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 89 | 92 | SHIFT | events.md:92 |
| 399 | 10.3 Terminal events | `[events.md:90](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 90 | 93 | SHIFT | events.md:93 |
| 400 | 10.3 Terminal events | `[events.md:91](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 91 | 94 | SHIFT | events.md:94 |
| 406 | 10.3 Terminal events | `[events.md:95–99](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 95-99 | 99-100 | SHIFT | events.md:99–100 |
| 407 | 10.3 Terminal events | `[events.md:100–102](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 100-102 | 101-102 | MATCH | unchanged; cited correct |
| 408 | 10.3 Terminal events | `[events.md:103–104](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 103-104 | 103-104 | MATCH | unchanged; cited correct |
| 409 | 10.3 Terminal events | `[events.md:105–106](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 105-106 | 105-106 | MATCH | unchanged; cited correct |
| 410† | 10.3 Terminal events | `[events.md:88](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 88 | — | NOT_FOUND | events.md:91 (`terminal.progress_report` row); no verbatim quote |
| 416 | 10.4 Server events | `[events.md:112](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 112 | 115 | SHIFT | events.md:115 |
| 417 | 10.4 Server events | `[events.md:116](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 116 | 116 | MATCH | unchanged; cited correct |
| 418 | 10.4 Server events | `[events.md:119](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 119 | 117 | SHIFT | events.md:117 |
| 419 | 10.4 Server events | `[events.md:122](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 122 | 118 | SHIFT | events.md:118 |
| 420 | 10.4 Server events | `[events.md:123](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 123 | 119 | SHIFT | events.md:119 |
| 421 | 10.4 Server events | `[events.md:124](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 124 | 120 | SHIFT | events.md:120 |
| 422 | 10.4 Server events | `[events.md:125](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 125 | 121 | SHIFT | events.md:121 |
| 423 | 10.4 Server events | `[events.md:126](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 126 | 122 | SHIFT | events.md:122 |
| 424 | 10.4 Server events | `[events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 128-129 | 124-125 | SHIFT | events.md:124–125 |
| 430 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:32–38](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 32-38 | 32-33 | MATCH | unchanged; cited correct |
| 431 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 40-43 | 41-43 | MATCH | unchanged; cited correct |
| 432 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:45–46](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 45-46 | 45-46 | MATCH | unchanged; cited correct |
| 433 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:48–53](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 48-53 | 52-53 | MATCH | unchanged; cited correct |
| 434 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:55–56](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 55-56 | 55-56 | MATCH | unchanged; cited correct |
| 435 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:86–87](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 86-87 | 86-87 | MATCH | unchanged; cited correct |
| 436 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:91–93](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 91-93 | 91-92 | MATCH | unchanged; cited correct |
| 437 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:99–100](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 99-100 | 99-100 | MATCH | unchanged; cited correct |
| 438 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:105–116](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 105-116 | 110;112-114 | MATCH | unchanged; cited correct |
| 439 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:123–126](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 123-126 | 126-127 | SHIFT | lua-events.md:126–127 |
| 440 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:133](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 133 | 133 | MATCH | unchanged; cited correct |
| 441 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:143–144](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 143-144 | 148-149 | SHIFT | lua-events.md:148–149 |
| 442 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:153–155](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 153-155 | 153 | MATCH | unchanged; cited correct |
| 443 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 40-43 | 42-43 | MATCH | unchanged; cited correct |
| 444 | 10.5 Handler registration, scope, and failure behavior | `[lua-events.md:143–144](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 143-144 | 148-149 | SHIFT | lua-events.md:148–149 |
| 445† | 10.5 Handler registration, scope, and failure behavior | `[events.md:66, 124](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 66;124 | — | NOT_FOUND | events.md:68, 120 (`session_view_changed` revision; `keymap_changed` generation/hash); no verbatim quote |
| 446† | 10.5 Handler registration, scope, and failure behavior | `[events.md:71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 71 | — | NOT_FOUND | keep 71 (`connection_state` field list); values not documented |
| 452 | 10.6 Events on the CLI | `[events.md:129–130](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 129-130 | 129-130 | MATCH | unchanged; cited correct |
| 453 | 10.6 Events on the CLI | `[events.md:136–137](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 136-137 | 136-137 | MATCH | unchanged; cited correct |
| 454 | 10.6 Events on the CLI | `[events.md:137–148](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 137-148 | 137;150 | SHIFT | events.md:137, 150 |
| 455 | 10.6 Events on the CLI | `[events.md:157](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 157 | 157 | MATCH | unchanged; cited correct |
| 456 | 10.6 Events on the CLI | `[events.md:158](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 158 | 158 | MATCH | unchanged; cited correct |
| 457 | 10.6 Events on the CLI | `[events.md:159](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 159 | 159 | MATCH | unchanged; cited correct |
| 458 | 10.6 Events on the CLI | `[events.md:163–166](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 163-166 | 163-165 | MATCH | unchanged; cited correct |
| 466 | 11.1 Scope, syntax, and encoding | `[program-status.md:6–9](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 6-9 | 8-10 | SHIFT | program-status.md:8–10 |
| 467 | 11.1 Scope, syntax, and encoding | `[program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 20-23 | 22-24 | SHIFT | program-status.md:22–24 |
| 468 | 11.1 Scope, syntax, and encoding | `[program-status.md:47–49](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 47-49 | 62-63 | SHIFT | program-status.md:62–63 |
| 469 | 11.1 Scope, syntax, and encoding | `[program-status.md:51–57](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 51-57 | 69 | SHIFT | program-status.md:69 |
| 470 | 11.1 Scope, syntax, and encoding | `[program-status.md:63–66](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 63-66 | 74-75 | SHIFT | program-status.md:74–75 |
| 471† | 11.1 Scope, syntax, and encoding | `[program-status.md:79–81](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 79-81 | 80-81 | MATCH | unchanged (program-status.md:79–81 covers the fence + syntax lines) |
| 472 | 11.1 Scope, syntax, and encoding | `[program-status.md:83–86](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 83-86 | 86 | MATCH | unchanged; cited correct |
| 473 | 11.1 Scope, syntax, and encoding | `[program-status.md:88–90](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 88-90 | 89-91 | SHIFT | program-status.md:89–91 |
| 474 | 11.1 Scope, syntax, and encoding | `[program-status.md:91–92](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 91-92 | 92-93 | SHIFT | program-status.md:92–93 |
| 475 | 11.1 Scope, syntax, and encoding | `[program-status.md:93](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 93 | 94 | SHIFT | program-status.md:94 |
| 476 | 11.1 Scope, syntax, and encoding | `[program-status.md:94](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 94 | 95 | SHIFT | program-status.md:95 |
| 477 | 11.1 Scope, syntax, and encoding | `[program-status.md:96–97](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 96-97 | 96-97 | MATCH | unchanged; cited correct |
| 478 | 11.1 Scope, syntax, and encoding | `[program-status.md:98–100](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 98-100 | 97-99 | SHIFT | program-status.md:97–99 |
| 479 | 11.1 Scope, syntax, and encoding | `[program-status.md:101–103](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 101-103 | 102 | MATCH | unchanged; cited correct |
| 485 | 11.2 Records, IDs, and lifetimes | `[program-status.md:106–108](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 106-108 | 106-107 | MATCH | unchanged; cited correct |
| 486† | 11.2 Records, IDs, and lifetimes | `[program-status.md:113–116](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 113-116 | 110-111 | SHIFT | program-status.md:110–111 |
| 487 | 11.2 Records, IDs, and lifetimes | `[program-status.md:118–120](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 118-120 | 114 | SHIFT | program-status.md:114 |
| 488 | 11.2 Records, IDs, and lifetimes | `[program-status.md:121–123](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 121-123 | 117-118 | SHIFT | program-status.md:117–118 |
| 489 | 11.2 Records, IDs, and lifetimes | `[program-status.md:124–126](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 124-126 | 120-121 | SHIFT | program-status.md:120–121 |
| 490 | 11.2 Records, IDs, and lifetimes | `[program-status.md:127–132](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 127-132 | 122-124 | SHIFT | program-status.md:122–124 |
| 491† | 11.2 Records, IDs, and lifetimes | `[program-status.md:132–134](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 132-134 | 124-126 | SHIFT | program-status.md:124–126 |
| 492 | 11.2 Records, IDs, and lifetimes | `[program-status.md:135–136](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 135-136 | 127-128 | SHIFT | program-status.md:127–128 |
| 493 | 11.2 Records, IDs, and lifetimes | `[program-status.md:137–138](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 137-138 | 129-130 | SHIFT | program-status.md:129–130 |
| 494 | 11.2 Records, IDs, and lifetimes | `[program-status.md:138–139](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 138-139 | 130-131 | SHIFT | program-status.md:130–131 |
| 495 | 11.2 Records, IDs, and lifetimes | `[program-status.md:154–155](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 154-155 | 152-153 | SHIFT | program-status.md:152–153 |
| 496† | 11.2 Records, IDs, and lifetimes | `[program-status.md:160–165](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 160-165 | 153-156 | SHIFT | program-status.md:153–156 |
| 500 | 11.3 States | `[program-status.md:141–143, 163](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 141-143;163 | — | NOT_FOUND | drop number; heading `## States` (raw line 133) + quote |
| 504 | 11.3 States | `[program-status.md:140](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 140 | 140;142 | SHIFT | program-status.md:140, 142 |
| 505 | 11.3 States | `[program-status.md:141](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 141 | 141 | MATCH | unchanged; cited correct |
| 506 | 11.3 States | `[program-status.md:142](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 142 | 142 | MATCH | unchanged; cited correct |
| 507 | 11.3 States | `[program-status.md:143](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 143 | 143 | MATCH | unchanged; cited correct |
| 508 | 11.3 States | `[program-status.md:144](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 144 | 144 | MATCH | unchanged; cited correct |
| 509 | 11.3 States | `[program-status.md:145](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 145 | 145 | MATCH | unchanged; cited correct |
| 510 | 11.3 States | `[program-status.md:147–148](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 147-148 | 147 | MATCH | unchanged; cited correct |
| 511 | 11.3 States | `[program-status.md:148–151](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 148-151 | 147-149 | SHIFT | program-status.md:147–149 |
| 512 | 11.3 States | `[program-status.md:165–166](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 165-166 | 156-157 | SHIFT | program-status.md:156–157 |
| 518 | 11.4 Keys | `[program-status.md:163](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 163 | 163 | MATCH | unchanged; cited correct |
| 519 | 11.4 Keys | `[program-status.md:164](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 164 | 164 | MATCH | unchanged; cited correct |
| 520 | 11.4 Keys | `[program-status.md:165](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 165 | 165 | MATCH | unchanged; cited correct |
| 521 | 11.4 Keys | `[program-status.md:165](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 165 | 165 | MATCH | unchanged; cited correct |
| 522 | 11.4 Keys | `[program-status.md:166](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 166 | 166 | MATCH | unchanged; cited correct |
| 523 | 11.4 Keys | `[program-status.md:167](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 167 | 167 | MATCH | unchanged; cited correct |
| 524† | 11.4 Keys | `[program-status.md:168](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 168 | 168 | MATCH | unchanged (program-status.md:168) |
| 525† | 11.4 Keys | `[program-status.md:169](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 169 | 169 | MATCH | unchanged (program-status.md:169) |
| 526 | 11.4 Keys | `[program-status.md:85](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 85 | — | NOT_FOUND | drop number; heading `## Syntax` (raw line 77) + quote |
| 532 | 11.5 Feature detection | `[program-status.md:172–176](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 172-176 | 175 | MATCH | unchanged; cited correct |
| 533 | 11.5 Feature detection | `[program-status.md:178–180](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 178-180 | 178-180 | MATCH | unchanged; cited correct |
| 534 | 11.5 Feature detection | `[program-status.md:182–184](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 182-184 | 182-184 | MATCH | unchanged; cited correct |
| 535 | 11.5 Feature detection | `[program-status.md:186–188](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 186-188 | 186-188 | MATCH | unchanged; cited correct |
| 536 | 11.5 Feature detection | `[program-status.md:191–192](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 191-192 | 190-191 | SHIFT | program-status.md:190–191 |
| 537 | 11.5 Feature detection | `[program-status.md:194–195](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 194-195 | 193-194 | SHIFT | program-status.md:193–194 |
| 538 | 11.5 Feature detection | `[program-status.md:195–198](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 195-198 | 194-196 | SHIFT | program-status.md:194–196 |
| 539 | 11.5 Feature detection | `[program-status.md:172–198](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 172-198 | — | NOT_FOUND | drop number; heading `## Feature detection` (raw line 171) + quote |
| 545 | 11.6 Security, limits, and revision | `[program-status.md:208–212](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 208-212 | 203-206 | SHIFT | program-status.md:203–206 |
| 546 | 11.6 Security, limits, and revision | `[program-status.md:213–216](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 213-216 | 207-209 | SHIFT | program-status.md:207–209 |
| 547 | 11.6 Security, limits, and revision | `[program-status.md:217–219](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 217-219 | 210-212 | SHIFT | program-status.md:210–212 |
| 548 | 11.6 Security, limits, and revision | `[program-status.md:220–222](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 220-222 | 213-215 | SHIFT | program-status.md:213–215 |
| 549 | 11.6 Security, limits, and revision | `[program-status.md:227](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 227 | 227 | MATCH | unchanged; cited correct |
| 550 | 11.6 Security, limits, and revision | `[program-status.md:228](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 228 | — | NOT_FOUND | drop number; heading `## Limits` (raw line 217) + quote |
| 551 | 11.6 Security, limits, and revision | `[program-status.md:229](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 229 | 229 | MATCH | unchanged; cited correct |
| 552 | 11.6 Security, limits, and revision | `[program-status.md:230](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 230 | — | NOT_FOUND | drop number; heading `## Limits` (raw line 217) + quote |
| 553 | 11.6 Security, limits, and revision | `[program-status.md:231](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 231 | — | NOT_FOUND | drop number; heading `## Limits` (raw line 217) + quote |
| 554 | 11.6 Security, limits, and revision | `[program-status.md:232](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 232 | — | NOT_FOUND | drop number; heading `## Limits` (raw line 217) + quote |
| 555 | 11.6 Security, limits, and revision | `[program-status.md:233](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 233 | 233 | MATCH | unchanged; cited correct |
| 556 | 11.6 Security, limits, and revision | `[program-status.md:243–248](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 243-248 | 244-246 | MATCH | unchanged; cited correct |
| 557 | 11.6 Security, limits, and revision | `[program-status.md:250–257](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 250-257 | 249 | SHIFT | program-status.md:249 |
| 558 | 11.6 Security, limits, and revision | `[program-status.md:259–264](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 259-264 | 256-259 | SHIFT | program-status.md:256–259 |
| 559 | 11.6 Security, limits, and revision | `[program-status.md:266](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 266 | 262 | SHIFT | program-status.md:262 |
| 560 | 11.6 Security, limits, and revision | `[program-status.md:268–271](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 268-271 | 267-268 | SHIFT | program-status.md:267–268 |
| 561 | 11.6 Security, limits, and revision | `[program-status.md:313–317](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 313-317 | 317 | MATCH | unchanged; cited correct |
| 562 | 11.6 Security, limits, and revision | `[program-status.md:30–38](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 30-38 | 50-53 | SHIFT | program-status.md:50–53 |
| 563 | 11.6 Security, limits, and revision | `[program-status.md:40–43](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 40-43 | 56-58 | SHIFT | program-status.md:56–58 |
| 564 | 11.6 Security, limits, and revision | `[program-status.md:79–103](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 79-103 | — | NOT_FOUND | drop number; heading `## Syntax` (raw line 77) + quote |
| 565 | 11.6 Security, limits, and revision | `[program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 20-23 | — | NOT_FOUND | drop number; heading `# the apply. The message decodes to "Apply 3 to add, 1 to change, 0 to destroy?".` (raw line 18) + quote |
| 571 | 12. Multi-client behavior | `[cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 34-36 | 35-36 | MATCH | unchanged; cited correct |
| 572 | 12. Multi-client behavior | `[cli.md:10–12](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 10-12 | 11-12 | MATCH | unchanged; cited correct |
| 573 | 12. Multi-client behavior | `[lua.md:180](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 180 | 180 | MATCH | unchanged; cited correct |
| 574 | 12. Multi-client behavior | `[events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 128-129 | 124-125 | SHIFT | events.md:124–125 |
| 575 | 12. Multi-client behavior | `[events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 69-71 | 69-71 | MATCH | unchanged; cited correct |
| 576 | 12. Multi-client behavior | `[events.md:119–123](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 119-123 | 117-119 | SHIFT | events.md:117–119 |
| 580 | 12. Multi-client behavior | `[lua.md:205](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 205 | — | NOT_FOUND | drop number; heading `### `rex.session.<method>(args)`` (raw line 167) + quote |
| 581† | 12. Multi-client behavior | `[events.md:119–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 119-129 | — | NOT_FOUND | events.md:115–125 (server-events table); no verbatim quote |
| 582† | 12. Multi-client behavior | `[events.md:128–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 128-129 | — | NOT_FOUND | events.md:124–125 (`client` payload table); no verbatim quote |
| 588 | 13. Connection lifecycle | `[cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 34-36 | 35-36 | MATCH | unchanged; cited correct |
| 589 | 13. Connection lifecycle | `[events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 69-71 | 69-71 | MATCH | unchanged; cited correct |
| 590† | 13. Connection lifecycle | `[events.md:71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 71 | 71 | MATCH | unchanged (events.md:71) |
| 591† | 13. Connection lifecycle | `[events.md:71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 71 | — | NOT_FOUND | keep 71 (`connection_state` field list); values not documented |
| 592† | 13. Connection lifecycle | `[events.md:69–71](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 69-71 | — | NOT_FOUND | events.md:69–71 (client connection events); no verbatim quote |
| 593† | 13. Connection lifecycle | `[events.md:69–71, 128–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 69-71;128-129 | 69-71,124-125 | SHIFT | events.md:69–71, 124–125 |
| 594 | 13. Connection lifecycle | `[cli.md:34–36](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 34-36 | — | NOT_FOUND | drop number; heading `## Taxonomy` (raw line 30) + quote |
| 595 | 13. Connection lifecycle | `[lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 313-327 | 320 | MATCH | unchanged; cited correct |
| 596 | 13. Connection lifecycle | `[lua.md:313–327](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 313-327 | — | NOT_FOUND | drop number; heading `### `rex.host{ ... }`` (raw line 313) + quote |
| 597 | 13. Connection lifecycle | `[lua.md:329–337](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 329-337 | 327-328;333 | SHIFT | lua.md:327–328, 333 |
| 605 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:18–28](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 18-28 | — | NOT_FOUND | drop number; heading `# The CLI` (raw line 6) + quote |
| 606 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:106–107](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 106-107 | 106-107 | MATCH | unchanged; cited correct |
| 607 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:125–136](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 125-136 | — | NOT_FOUND | drop number; heading `## Machine-Readable Output` (raw line 104) + quote |
| 608 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:14–16](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 14-16 | 16-17 | SHIFT | cli.md:16–17 |
| 609 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:63–68](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 63-68 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 610 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:88–89](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 88-89 | 88-89 | MATCH | unchanged; cited correct |
| 611 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:56–67](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 56-67 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 612 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:52–58](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 52-58 | — | NOT_FOUND | drop number; heading `## Command Targets` (raw line 47) + quote |
| 613 | 14. Relayable deltas against `full.md` assumptions | `[server-api.md:63–103, 157–159](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 63-103;157-159 | 94 | MATCH | unchanged; cited correct |
| 614 | 14. Relayable deltas against `full.md` assumptions | `[server-api.md:8–14, 236–248](https://www.superlogical.com/rex/docs/automate/server-api.md)` | server-api.md | 8-14;236-248 | — | NOT_FOUND | drop number; heading `# Server API` (raw line 6) + quote |
| 616 | 14. Relayable deltas against `full.md` assumptions | `[lua.md:8–9](https://www.superlogical.com/rex/docs/reference/lua.md)` | lua.md | 8-9 | 9-10 | SHIFT | lua.md:9–10 |
| 618† | 14. Relayable deltas against `full.md` assumptions | `[events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 34-46 | — | NOT_FOUND | events.md:36–38 (`ev.type`, `ev.session_id`, `ev.payload`); row is a paraphrase |
| 619† | 14. Relayable deltas against `full.md` assumptions | `[events.md:34–46](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 34-46 | — | NOT_FOUND | keep 34–46 (field table); no verbatim quote |
| 620 | 14. Relayable deltas against `full.md` assumptions | `[lua-events.md:91–100](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 91-100 | — | NOT_FOUND | drop number; heading `## Event Scope` (raw line 89) + quote |
| 620† | 14. Relayable deltas against `full.md` assumptions | `[events.md:153–159](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 153-159 | — | NOT_FOUND | events.md:157–159 (`rex events` scope table); row is a paraphrase |
| 621 | 14. Relayable deltas against `full.md` assumptions | `[lua-events.md:40–43](https://www.superlogical.com/rex/docs/automate/lua-events.md)` | lua-events.md | 40-43 | — | NOT_FOUND | drop number; heading `## Handlers` (raw line 30) + quote |
| 622 | 14. Relayable deltas against `full.md` assumptions | `[events.md:163–166](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 163-166 | 165 | MATCH | unchanged; cited correct |
| 623† | 14. Relayable deltas against `full.md` assumptions | `[events.md:59–129](https://www.superlogical.com/rex/docs/reference/events.md)` | events.md | 59-129 | — | NOT_FOUND | events.md:59–129 (event tables); row is a paraphrase |
| 624 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:140–145](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 140-145 | 145 | MATCH | unchanged; cited correct |
| 625 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:113–139, 163–169](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 113-139;163-169 | — | NOT_FOUND | drop number; heading `## Records and ids` (raw line 104) + quote |
| 626 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:96–100](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 96-100 | 96-97 | MATCH | unchanged; cited correct |
| 627 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:140–145, 154–165](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 140-145;154-165 | — | NOT_FOUND | drop number; heading `## States` (raw line 133) + quote |
| 628 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:172–198](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 172-198 | — | NOT_FOUND | drop number; heading `## Feature detection` (raw line 171) + quote |
| 629 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:88–103, 124–126](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 88-103;124-126 | — | NOT_FOUND | drop number; heading `## Syntax` (raw line 77) + quote |
| 630 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:163](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 163 | 163 | MATCH | unchanged; cited correct |
| 631 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 20-23 | 22-24 | SHIFT | program-status.md:22–24 |
| 632 | 14. Relayable deltas against `full.md` assumptions | `[program-status.md:20–23](https://www.superlogical.com/rex/docs/build/program-status.md)` | program-status.md | 20-23 | — | NOT_FOUND | drop number; heading `# the apply. The message decodes to "Apply 3 to add, 1 to change, 0 to destroy?".` (raw line 18) + quote |
| 633 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:8](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 8 | — | NOT_FOUND | drop number; heading `# The CLI` (raw line 6) + quote |
| 635 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:38](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 38 | 38 | MATCH | unchanged; cited correct |
| 636 | 14. Relayable deltas against `full.md` assumptions | `[cli.md:43–45, 100–102](https://www.superlogical.com/rex/docs/automate/cli.md)` | cli.md | 43-45;100-102 | 43;100-101 | MATCH | unchanged; cited correct |

## events.md corrections

One row per events.md citation. “unchanged” means the cited line is already correct.

| # | matrix line | section | cited ref | actual lines | verdict | corrected ref |
| ---: | ---: | --- | --- | --- | --- | --- |
| 1 | 84 | 5. Domain hierarchy and stable IDs | `events.md:107–108` | 110 | SHIFT | events.md:110 |
| 2 | 85 | 5. Domain hierarchy and stable IDs | `events.md:116` | — | NOT_FOUND | keep 116 (session_destroyed event); no verbatim quote — nearest heading `## Server Events` (raw line 108) |
| 3 | 124 | 7.1 Documented commands by family | `events.md:10–11, 153–159` | 11 | MATCH | unchanged; cited correct |
| 4 | 144 | 7.2 Flags and aliases | `events.md:159` | 159 | MATCH | unchanged; cited correct |
| 5 | 357 | 10.1 Sources, naming, envelope | `events.md:13–20` | 17-19 | MATCH | unchanged; cited correct |
| 6 | 358 | 10.1 Sources, naming, envelope | `events.md:16` | 17 | SHIFT | events.md:17 |
| 7 | 359 | 10.1 Sources, naming, envelope | `events.md:18` | 18 | MATCH | unchanged; cited correct |
| 8 | 361 | 10.1 Sources, naming, envelope | `events.md:36` | 36 | MATCH | unchanged; cited correct |
| 9 | 362 | 10.1 Sources, naming, envelope | `events.md:37` | 37 | MATCH | unchanged; cited correct |
| 10 | 363 | 10.1 Sources, naming, envelope | `events.md:38` | 38 | MATCH | unchanged; cited correct |
| 11 | 364 | 10.1 Sources, naming, envelope | `events.md:43–46` | 44-46 | MATCH | unchanged; cited correct |
| 12 | 365 | 10.1 Sources, naming, envelope | `events.md:51–52` | 51-52 | MATCH | unchanged; cited correct |
| 13 | 366 | 10.1 Sources, naming, envelope | `events.md:48–49` | 48-49 | MATCH | unchanged; cited correct |
| 14 | 367 | 10.1 Sources, naming, envelope | `events.md:74–76` | 74-75 | MATCH | unchanged; cited correct |
| 15 | 368 | 10.1 Sources, naming, envelope | `events.md:34–46` | — | NOT_FOUND | drop number; heading `## Events in Lua` (raw line 21) + quote (field definitions at 34–46) |
| 16 | 374 | 10.2 Session events | `events.md:59` | 61 | SHIFT | events.md:61 |
| 17 | 375 | 10.2 Session events | `events.md:60` | 62 | SHIFT | events.md:62 |
| 18 | 376 | 10.2 Session events | `events.md:61` | 63 | SHIFT | events.md:63 |
| 19 | 377 | 10.2 Session events | `events.md:62` | 64 | SHIFT | events.md:64 |
| 20 | 378 | 10.2 Session events | `events.md:63` | 65 | SHIFT | events.md:65 |
| 21 | 379 | 10.2 Session events | `events.md:64` | 66 | SHIFT | events.md:66 |
| 22 | 380 | 10.2 Session events | `events.md:65` | 67 | SHIFT | events.md:67 |
| 23 | 381 | 10.2 Session events | `events.md:66` | 68 | SHIFT | events.md:68 |
| 24 | 382 | 10.2 Session events | `events.md:69` | 69 | MATCH | unchanged; cited correct |
| 25 | 383 | 10.2 Session events | `events.md:70` | 70 | MATCH | unchanged; cited correct |
| 26 | 384 | 10.2 Session events | `events.md:71` | 71 | MATCH | unchanged; cited correct |
| 27 | 385 | 10.2 Session events | `events.md:72` | 72 | MATCH | unchanged; cited correct |
| 28 | 391 | 10.3 Terminal events | `events.md:82` | 85 | SHIFT | events.md:85 |
| 29 | 392 | 10.3 Terminal events | `events.md:83` | 86 | SHIFT | events.md:86 |
| 30 | 393 | 10.3 Terminal events | `events.md:84` | 87 | SHIFT | events.md:87 |
| 31 | 394 | 10.3 Terminal events | `events.md:85` | 88 | SHIFT | events.md:88 |
| 32 | 395 | 10.3 Terminal events | `events.md:86` | 89 | SHIFT | events.md:89 |
| 33 | 396 | 10.3 Terminal events | `events.md:87` | 90 | SHIFT | events.md:90 |
| 34 | 397 | 10.3 Terminal events | `events.md:88` | 91 | SHIFT | events.md:91 |
| 35 | 398 | 10.3 Terminal events | `events.md:89` | 92 | SHIFT | events.md:92 |
| 36 | 399 | 10.3 Terminal events | `events.md:90` | 93 | SHIFT | events.md:93 |
| 37 | 400 | 10.3 Terminal events | `events.md:91` | 94 | SHIFT | events.md:94 |
| 38 | 406 | 10.3 Terminal events | `events.md:95–99` | 99-100 | SHIFT | events.md:99–100 |
| 39 | 407 | 10.3 Terminal events | `events.md:100–102` | 101-102 | MATCH | unchanged; cited correct |
| 40 | 408 | 10.3 Terminal events | `events.md:103–104` | 103-104 | MATCH | unchanged; cited correct |
| 41 | 409 | 10.3 Terminal events | `events.md:105–106` | 105-106 | MATCH | unchanged; cited correct |
| 42 | 410 | 10.3 Terminal events | `events.md:88` | — | NOT_FOUND | events.md:91 (`terminal.progress_report` row); no verbatim quote |
| 43 | 416 | 10.4 Server events | `events.md:112` | 115 | SHIFT | events.md:115 |
| 44 | 417 | 10.4 Server events | `events.md:116` | 116 | MATCH | unchanged; cited correct |
| 45 | 418 | 10.4 Server events | `events.md:119` | 117 | SHIFT | events.md:117 |
| 46 | 419 | 10.4 Server events | `events.md:122` | 118 | SHIFT | events.md:118 |
| 47 | 420 | 10.4 Server events | `events.md:123` | 119 | SHIFT | events.md:119 |
| 48 | 421 | 10.4 Server events | `events.md:124` | 120 | SHIFT | events.md:120 |
| 49 | 422 | 10.4 Server events | `events.md:125` | 121 | SHIFT | events.md:121 |
| 50 | 423 | 10.4 Server events | `events.md:126` | 122 | SHIFT | events.md:122 |
| 51 | 424 | 10.4 Server events | `events.md:128–129` | 124-125 | SHIFT | events.md:124–125 |
| 52 | 445 | 10.5 Handler registration, scope, and failure behavior | `events.md:66, 124` | — | NOT_FOUND | events.md:68, 120 (`session_view_changed` revision; `keymap_changed` generation/hash); no verbatim quote |
| 53 | 446 | 10.5 Handler registration, scope, and failure behavior | `events.md:71` | — | NOT_FOUND | keep 71 (`connection_state` field list); values not documented |
| 54 | 452 | 10.6 Events on the CLI | `events.md:129–130` | 129-130 | MATCH | unchanged; cited correct |
| 55 | 453 | 10.6 Events on the CLI | `events.md:136–137` | 136-137 | MATCH | unchanged; cited correct |
| 56 | 454 | 10.6 Events on the CLI | `events.md:137–148` | 137;150 | SHIFT | events.md:137, 150 |
| 57 | 455 | 10.6 Events on the CLI | `events.md:157` | 157 | MATCH | unchanged; cited correct |
| 58 | 456 | 10.6 Events on the CLI | `events.md:158` | 158 | MATCH | unchanged; cited correct |
| 59 | 457 | 10.6 Events on the CLI | `events.md:159` | 159 | MATCH | unchanged; cited correct |
| 60 | 458 | 10.6 Events on the CLI | `events.md:163–166` | 163-165 | MATCH | unchanged; cited correct |
| 61 | 574 | 12. Multi-client behavior | `events.md:128–129` | 124-125 | SHIFT | events.md:124–125 |
| 62 | 575 | 12. Multi-client behavior | `events.md:69–71` | 69-71 | MATCH | unchanged; cited correct |
| 63 | 576 | 12. Multi-client behavior | `events.md:119–123` | 117-119 | SHIFT | events.md:117–119 |
| 64 | 581 | 12. Multi-client behavior | `events.md:119–129` | — | NOT_FOUND | events.md:115–125 (server-events table); no verbatim quote |
| 65 | 582 | 12. Multi-client behavior | `events.md:128–129` | — | NOT_FOUND | events.md:124–125 (`client` payload table); no verbatim quote |
| 66 | 589 | 13. Connection lifecycle | `events.md:69–71` | 69-71 | MATCH | unchanged; cited correct |
| 67 | 590 | 13. Connection lifecycle | `events.md:71` | 71 | MATCH | unchanged (events.md:71) |
| 68 | 591 | 13. Connection lifecycle | `events.md:71` | — | NOT_FOUND | keep 71 (`connection_state` field list); values not documented |
| 69 | 592 | 13. Connection lifecycle | `events.md:69–71` | — | NOT_FOUND | events.md:69–71 (client connection events); no verbatim quote |
| 70 | 593 | 13. Connection lifecycle | `events.md:69–71, 128–129` | 69-71,124-125 | SHIFT | events.md:69–71, 124–125 |
| 71 | 618 | 14. Relayable deltas against `full.md` assumptions | `events.md:34–46` | — | NOT_FOUND | events.md:36–38 (`ev.type`, `ev.session_id`, `ev.payload`); row is a paraphrase |
| 72 | 619 | 14. Relayable deltas against `full.md` assumptions | `events.md:34–46` | — | NOT_FOUND | keep 34–46 (field table); no verbatim quote |
| 73 | 620 | 14. Relayable deltas against `full.md` assumptions | `events.md:153–159` | — | NOT_FOUND | events.md:157–159 (`rex events` scope table); row is a paraphrase |
| 74 | 622 | 14. Relayable deltas against `full.md` assumptions | `events.md:163–166` | 165 | MATCH | unchanged; cited correct |
| 75 | 623 | 14. Relayable deltas against `full.md` assumptions | `events.md:59–129` | — | NOT_FOUND | events.md:59–129 (event tables); row is a paraphrase |

## Method sanity control (3 known-good cli.md citations)

The method was calibrated on three citations whose cited line is unmistakable:

| citation | quoted text | cited | located | verdict |
| --- | --- | --- | --- | --- |
| `cli.md:8` | `rex` is the command-line interface to Rex. | 8 | 8 | MATCH |
| `cli.md:37` | A **window** is a tab within a session. It holds a layout of blocks. | 37 | 37 | MATCH |
| `cli.md:38` | A **block** is one application in a window. Today that means a terminal. | 38 | 38 | MATCH |

These three come out MATCH, so the method is not returning mass failure. The SHIFT/NOT_FOUND results below are therefore genuine line-number defects (or genuine “not documented” rows), not a broken matcher.

## Caveats

- Rows whose row evidence is a paraphrase or an `unknown` marker (no double-quoted verbatim excerpt) cannot be located verbatim; they are reported NOT_FOUND and the corrected ref points at the nearest heading or the documented anchor line.
- Rows citing code samples (JSON/grammar tables) were manually adjudicated because a fragment-union search can match a fragment that also appears elsewhere; the †-marked rows carry the adjudicated result.
- Nothing in the repository was modified. No network access was used.

