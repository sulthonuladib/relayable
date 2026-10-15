# DOC-002 — Live re-verification of reference URLs in `docs/compatibility/reference-behavior-matrix.md`

**Task mode:** network evidence gathering only. No repository files were edited, created, or deleted. No git write commands were run. No packages installed. All artifacts written under `/tmp/doc002/` only.

**Basis of comparison:** DOC-001 recorded sha256 values (fetched 2026-10-15) and the original DOC-001 raw artifacts, which still existed on disk at `/tmp/relayable-protocol-compat/raw/` (see §5).

**Fetch window (UTC brackets for the whole batch):**
- `BATCH_START=2026-10-15T16:57:25Z`
- `BATCH_END=2026-10-15T16:57:34Z`

**Fetch command shape used for every URL** (`--max-time 30 --connect-timeout 15` added as instructed):

```
date -u +%Y-%m-%dT%H:%M:%SZ
curl -sS -L --max-time 30 --connect-timeout 15 -o /tmp/doc002/raw/<slug> \
  -w 'HTTP=%{http_code} SIZE=%{size_download} URL=%{url_effective}\n' '<URL>' ; echo "curl_exit=$?"
date -u +%Y-%m-%dT%H:%M:%SZ
sha256sum /tmp/doc002/raw/<slug>
```

The batch was executed by `/tmp/doc002/fetch.sh` (input list `/tmp/doc002/urls.txt`); stdout/stderr transcript at `/tmp/doc002/fetch.log`.

**Overall result:** all 21 requests completed with `curl_exit=0`. All 21 fresh sha256 values are **identical** to the DOC-001 recorded values. 17 URLs returned HTTP 200; 4 URLs (`sitemap.xml`, `sitemap-index.xml`, `sitemap_index.xml`, `sitemap-0.xml`) returned **HTTP 404**. No network failure occurred, so the "stop after 2 failed URLs" rule was never triggered.

---

## 1. Master table

| # | slug | URL | HTTP | curl exit | UTC retrieved | fresh sha256 | DOC-001 sha256 | CHANGED? |
|---|------|-----|------|-----------|---------------|--------------|----------------|----------|
| 1 | cli.html | https://www.superlogical.com/rex/docs/automate/cli | 200 | 0 | 2026-10-15T16:57:25Z | feb7a664dcce882e2196d83a1eb9c266add551ef48ce4bbefc8690ef4d753dab | feb7a664dcce882e2196d83a1eb9c266add551ef48ce4bbefc8690ef4d753dab | no |
| 2 | cli.md | https://www.superlogical.com/rex/docs/automate/cli.md | 200 | 0 | 2026-10-15T16:57:26Z | e61162828b0876173085622831cbdae56ab8072bbc8f83c7dee51ea933912b93 | e61162828b0876173085622831cbdae56ab8072bbc8f83c7dee51ea933912b93 | no |
| 3 | server-api.html | https://www.superlogical.com/rex/docs/automate/server-api | 200 | 0 | 2026-10-15T16:57:26Z | 4e8d5040b54d88f6dcdf26b2f50a582f0d46fd022297e4ac3d2065741f51cdec | 4e8d5040b54d88f6dcdf26b2f50a582f0d46fd022297e4ac3d2065741f51cdec | no |
| 4 | server-api.md | https://www.superlogical.com/rex/docs/automate/server-api.md | 200 | 0 | 2026-10-15T16:57:26Z | 6e31118649040ff616fd83d4fc743aadb657ef0ad93e7d0a4100eb0785882038 | 6e31118649040ff616fd83d4fc743aadb657ef0ad93e7d0a4100eb0785882038 | no |
| 5 | lua.html | https://www.superlogical.com/rex/docs/reference/lua | 200 | 0 | 2026-10-15T16:57:27Z | 80ec081d2cb4ac9cd0a94a2b64b2462d2a0e270e839a456575a09f4b2afc1509 | 80ec081d2cb4ac9cd0a94a2b64b2462d2a0e270e839a456575a09f4b2afc1509 | no |
| 6 | lua.md | https://www.superlogical.com/rex/docs/reference/lua.md | 200 | 0 | 2026-10-15T16:57:27Z | 934773e17693cc6222eff8e40bf79911247500fdc130e92e887bb27678bd1a0f | 934773e17693cc6222eff8e40bf79911247500fdc130e92e887bb27678bd1a0f | no |
| 7 | events.html | https://www.superlogical.com/rex/docs/reference/events | 200 | 0 | 2026-10-15T16:57:27Z | 2f49cd8dc63077723262955f45c8fc1b2aa2d40890c795202106a39c0f6629ac | 2f49cd8dc63077723262955f45c8fc1b2aa2d40890c795202106a39c0f6629ac | no |
| 8 | events.md | https://www.superlogical.com/rex/docs/reference/events.md | 200 | 0 | 2026-10-15T16:57:28Z | e0c8b1cdfb0967f674f4dece59f7bc3d17782ab0e928920d8070902c92c7d5a0 | e0c8b1cdfb0967f674f4dece59f7bc3d17782ab0e928920d8070902c92c7d5a0 | no |
| 9 | lua-events.html | https://www.superlogical.com/rex/docs/automate/lua-events | 200 | 0 | 2026-10-15T16:57:28Z | ae6e2b5864cccf526ba61bda462461543e7463fda9407ab1f034f147de8bcec5 | ae6e2b5864cccf526ba61bda462461543e7463fda9407ab1f034f147de8bcec5 | no |
| 10 | lua-events.md | https://www.superlogical.com/rex/docs/automate/lua-events.md | 200 | 0 | 2026-10-15T16:57:29Z | 312426e5edef5c80abb34785cda150de4c30acb8045912c4b6139b388d6af4a9 | 312426e5edef5c80abb34785cda150de4c30acb8045912c4b6139b388d6af4a9 | no |
| 11 | lua-scripts.html | https://www.superlogical.com/rex/docs/automate/lua-scripts | 200 | 0 | 2026-10-15T16:57:29Z | 4abac36d940f3a2f3468006b04e5932632328868000fdcf125fd280fa727fbd7 | 4abac36d940f3a2f3468006b04e5932632328868000fdcf125fd280fa727fbd7 | no |
| 12 | lua-scripts.md | https://www.superlogical.com/rex/docs/automate/lua-scripts.md | 200 | 0 | 2026-10-15T16:57:29Z | 0d8b1547357bda30e9b85e86f054f240fcf604711cf805d66d98f81da556a372 | 0d8b1547357bda30e9b85e86f054f240fcf604711cf805d66d98f81da556a372 | no |
| 13 | program-status.html | https://www.superlogical.com/rex/docs/build/program-status | 200 | 0 | 2026-10-15T16:57:30Z | 0e1f209b29201f167c944ee1f20465ebfb1c7c9390cbf19ea364eeeb9f5b05f7 | 0e1f209b29201f167c944ee1f20465ebfb1c7c9390cbf19ea364eeeb9f5b05f7 | no |
| 14 | program-status.md | https://www.superlogical.com/rex/docs/build/program-status.md | 200 | 0 | 2026-10-15T16:57:30Z | 9b3f32f6e53989c3d0909ce13cda4fb7dc15e29718f5a096281eb3ac4a951829 | 9b3f32f6e53989c3d0909ce13cda4fb7dc15e29718f5a096281eb3ac4a951829 | no |
| 15 | docs-index.html | https://www.superlogical.com/rex/docs | 200 | 0 | 2026-10-15T16:57:31Z | f668b09c300daf7b2ddef8f6a62cc0355f50c08b41aa853d49d8aadc34b461c1 | f668b09c300daf7b2ddef8f6a62cc0355f50c08b41aa853d49d8aadc34b461c1 | no |
| 16 | installation.md | https://www.superlogical.com/rex/docs/installation.md | 200 | 0 | 2026-10-15T16:57:32Z | 514e3a81ded0e1d7413e06d6e51c3c071ab5cad0955752b8d204f908816d09c7 | 514e3a81ded0e1d7413e06d6e51c3c071ab5cad0955752b8d204f908816d09c7 | no |
| 17 | sitemap-xml-1 | https://www.superlogical.com/sitemap.xml | **404** | 0 | 2026-10-15T16:57:32Z | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | no |
| 18 | sitemap-index-xml | https://www.superlogical.com/sitemap-index.xml | **404** | 0 | 2026-10-15T16:57:33Z | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | no |
| 19 | sitemap_index-xml | https://www.superlogical.com/sitemap_index.xml | **404** | 0 | 2026-10-15T16:57:33Z | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | no |
| 20 | sitemap-0-xml | https://www.superlogical.com/sitemap-0.xml | **404** | 0 | 2026-10-15T16:57:33Z | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d | no |
| 21 | robots.txt | https://www.superlogical.com/robots.txt | 200 | 0 | 2026-10-15T16:57:34Z | b033243267bcbdab84cb01e940811d6d77a894d78f0470faaf0eb43f8e4efcc9 | b033243267bcbdab84cb01e940811d6d77a894d78f0470faaf0eb43f8e4efcc9 | no |

**No URL has a CHANGED=yes. No URL has CHANGED=unknown.**

---

## 2. Per-URL drift verdict

1. **cli.html** — UNCHANGED. sha256 matches DOC-001 exactly; HTTP 200, 25586 bytes. Alternate link still advertises `/rex/docs/automate/cli.md`.
2. **cli.md** — UNCHANGED. sha256 matches; HTTP 200, 4664 bytes; genuine Markdown (YAML front matter).
3. **server-api.html** — UNCHANGED. sha256 matches; HTTP 200, 38860 bytes.
4. **server-api.md** — UNCHANGED. sha256 matches; HTTP 200, 7088 bytes; genuine Markdown.
5. **lua.html** — UNCHANGED. sha256 matches; HTTP 200, 50429 bytes.
6. **lua.md** — UNCHANGED. sha256 matches; HTTP 200, 12854 bytes; genuine Markdown.
7. **events.html** — UNCHANGED. sha256 matches; HTTP 200, 31465 bytes.
8. **events.md** — UNCHANGED. sha256 matches; HTTP 200, 7308 bytes; genuine Markdown.
9. **lua-events.html** — UNCHANGED. sha256 matches; HTTP 200, 33998 bytes.
10. **lua-events.md** — UNCHANGED. sha256 matches; HTTP 200, 5863 bytes; genuine Markdown.
11. **lua-scripts.html** — UNCHANGED. sha256 matches; HTTP 200, 43105 bytes.
12. **lua-scripts.md** — UNCHANGED. sha256 matches; HTTP 200, 6860 bytes; genuine Markdown.
13. **program-status.html** — UNCHANGED. sha256 matches; HTTP 200, 45357 bytes.
14. **program-status.md** — UNCHANGED. sha256 matches; HTTP 200, 16582 bytes; genuine Markdown.
15. **docs-index.html** — UNCHANGED (content hash matches DOC-001), but note the redirect: requested `https://www.superlogical.com/rex/docs`, and `url_effective` was `https://www.superlogical.com/rex/docs/installation`. The captured body is the **Installation** page (`<title>Installation · Rex Docs</title>`), i.e. the docs root redirects/lands on `/rex/docs/installation`; there is no distinct docs landing page at `/rex/docs`. Same behavior DOC-001 saw (same hash f668…).
16. **installation.md** — UNCHANGED. sha256 matches; HTTP 200, 611 bytes; genuine Markdown.
17. **sitemap-xml-1** — UNCHANGED hash-wise, but **DEAD ENDPOINT**: HTTP **404**, 5642-byte SvelteKit 404 page (`<title>404 — Superlogical</title>`, body "Sorry, this page doesn't exist.", `noindex`). DOC-001 recorded the same hash cc0f…, so this was already a 404 at DOC-001 time — not new drift.
18. **sitemap-index-xml** — UNCHANGED hash-wise; HTTP **404**, identical 5642-byte 404 body (same sha256 cc0f…).
19. **sitemap_index-xml** — UNCHANGED hash-wise; HTTP **404**, identical 5642-byte 404 body.
20. **sitemap-0-xml** — UNCHANGED hash-wise; HTTP **404**, identical 5642-byte 404 body.
21. **robots.txt** — UNCHANGED. sha256 matches; HTTP 200, 63 bytes. Content: `# allow crawling everything by default` / `User-agent: *` / `Disallow:` (empty Disallow). Note: robots.txt does not advertise any Sitemap directive.

**Drift conclusion:** across all 21 URLs there is **zero content drift** versus DOC-001 — every fresh sha256 equals the recorded sha256, and every fresh file is byte-identical to the DOC-001 original (`cmp`, §5). The only correctness concern is not drift but that the four `sitemap*.xml` URLs are (and already were) HTTP 404 dead ends, so they cannot support any sitemap-derived citation.

---

## 3. Heading outlines and line counts (fresh copies)

### `wc -l`
```
cli.md: 137
server-api.md: 252
lua.md: 342
events.md: 165
lua-events.md: 186
lua-scripts.md: 243
program-status.md: 320
```

### Heading outline (`grep -n '^#\{1,4\} ' <file>`)

**`cli.md`**
```
6:# The CLI
30:## Taxonomy
47:## Command Targets
76:### Targets
104:## Machine-Readable Output
```

**`server-api.md`**
```
6:# Server API
24:## Discovering Methods
50:## Reading a Method's Documentation
105:## Calling a Method
147:### Errors
162:## Calling From Lua
199:## Block Methods
236:## Machine-Readable Documentation
```

**`lua.md`**
```
6:# Lua API
12:## Where Each Function Works
48:## Key Bindings
53:### `rex.bind(key, action[, args])`
65:### `rex.bind(key, fn)`
73:### `rex.unbind(key[, target])`
81:### `rex.unbind{ mode = name }`
85:### `rex.unbind(binding)`
89:### `rex.mode(name[, options])`
94:### `rex.bindings([filter])`
110:### `rex.modes()`
115:## Actions
119:### `rex.action{ ... }`
141:### `rex.invoke(name[, args])`
151:### `rex.client.queue(name[, args])`
160:## Calling Rex
167:### `rex.session.<method>(args)`
215:### `rex.call(method[, args])`
226:### `rex.block.call(creator, method[, args])`
252:### `rex.layout`
273:## Events
278:### `rex.on(event, fn)`
285:### `rex.wait(event[, filter][, timeout])`
293:### `rex.sleep(seconds)`
297:### `rex.stop([code])`
301:## Script Values
311:## Hosts and Other Servers
313:### `rex.host{ ... }`
325:### `rex.servers()`
330:### `rex.server(label)`
337:## Logging
339:### `rex.log([level, ]message)`
```

**`events.md`**
```
6:# Events
21:## Events in Lua
54:## Session Events
78:## Terminal Events
108:## Server Events
127:## Events on the CLI
161:## Unknown Events
```

**`lua-events.md`**
```
6:# Lua Events
30:## Handlers
58:## Event Names
89:## Event Scope
106:## Stopping
124:## Handler Failures
138:## Handlers or `rex.wait`
151:## Invalid in Configuration Files
160:## Example: Notify on Bell
```

**`lua-scripts.md`**
```
6:# Lua Scripts
34:## Running a Script
60:## Arguments
84:## Results and Errors
111:## Calling Rex
155:### Layouts
176:## Sleeping and Waiting
204:## Example: Run Tests in a Split
```

**`program-status.md`**
```
6:# Program Status Protocol (OSC 7501)
17:# Terraform has finished planning and is waiting for the user to approve
18:# the apply. The message decodes to "Apply 3 to add, 1 to change, 0 to destroy?".
26:## Motivation
45:## In Rex
60:## Conventions
77:## Syntax
104:## Records and ids
133:## States
159:## Keys
171:## Feature detection
198:## Security
217:## Limits
235:## Relationship to other sequences
271:## Examples
275:### One program
287:### Several records
303:### A shell function
313:## Revision history
```

> **Caveat on `program-status.md` lines 17–18:** those two lines are **not headings**. They are shell-comment lines (`# Terraform has finished planning…`) inside a fenced code-block example near the top of the file; the `grep '^#\{1,4\} '` pattern matched them because they begin with `#`. Treat only line 6 as the document title and lines 26+ as the real outline. This is a pattern artifact, not a document-structure change.
>
> Similarly, in `lua.md` / `lua-events.md` / `lua-scripts.md` / `server-api.md` / `cli.md`, the `#`-prefixed lines that are *not* present are the YAML front-matter delimiters `---` (not `#`), so they were not matched. Line numbers above are raw line numbers in the fresh file.

---

## 4. Step 3b / 3c results

### (3b) `head -5` — Markdown authenticity check

All seven requested `.md` files are **genuine Markdown**, each opening with YAML front matter. No file's body is HTML; there is no `<html`, `<!DOCTYPE`, `<div`, or `<script` anywhere in any `.md`.

```
===== cli.md =====
---
title: "The CLI"
canonical_url: https://www.superlogical.com/rex/docs/automate/cli
---

===== server-api.md =====
---
title: "Server API"
canonical_url: https://www.superlogical.com/rex/docs/automate/server-api
---

===== lua.md =====
---
title: "Lua API"
canonical_url: https://www.superlogical.com/rex/docs/reference/lua
---

===== events.md =====
---
title: "Events"
canonical_url: https://www.superlogical.com/rex/docs/reference/events
---

===== lua-events.md =====
---
title: "Lua Events"
canonical_url: https://www.superlogical.com/rex/docs/automate/lua-events
---

===== lua-scripts.md =====
---
title: "Lua Scripts"
canonical_url: https://www.superlogical.com/rex/docs/automate/lua-scripts
---

===== program-status.md =====
---
title: "Program Status Protocol"
canonical_url: https://www.superlogical.com/rex/docs/build/program-status
---
```

Bonus (`installation.md`, also fetched):
```
---
title: "Installation"
canonical_url: https://www.superlogical.com/rex/docs/installation
---
```

`file(1)` confirms plain text, not HTML:
```
cli.md:            Unicode text, UTF-8 text
server-api.md:     ASCII text
lua.md:            ASCII text
events.md:         ASCII text
lua-events.md:     ASCII text
lua-scripts.md:    ASCII text
program-status.md: Unicode text, UTF-8 text, with very long lines (339)
installation.md:   ASCII text
```

HTML-wrapper grep counts (all zero for every `.md`):
```
cli.md: html_open=0 doctype=0 div=0 script=0
server-api.md: html_open=0 doctype=0 div=0 script=0
lua.md: html_open=0 doctype=0 div=0 script=0
events.md: html_open=0 doctype=0 div=0 script=0
lua-events.md: html_open=0 doctype=0 div=0 script=0
lua-scripts.md: html_open=0 doctype=0 div=0 script=0
program-status.md: html_open=0 doctype=0 div=0 script=0
installation.md: html_open=0 doctype=0 div=0 script=0
```

### (3c) Advertised Markdown alternate link in HTML pages

`grep -o 'rel="alternate"[^>]*'`:

| HTML page | alternate tag found |
|---|---|
| cli.html | `rel="alternate" type="text/markdown" href="/rex/docs/automate/cli.md"/` |
| server-api.html | `rel="alternate" type="text/markdown" href="/rex/docs/automate/server-api.md"/` |
| lua.html | `rel="alternate" type="text/markdown" href="/rex/docs/reference/lua.md"/` |
| events.html | `rel="alternate" type="text/markdown" href="/rex/docs/reference/events.md"/` |
| lua-events.html | `rel="alternate" type="text/markdown" href="/rex/docs/automate/lua-events.md"/` |
| lua-scripts.html | `rel="alternate" type="text/markdown" href="/rex/docs/automate/lua-scripts.md"/` |
| program-status.html | `rel="alternate" type="text/markdown" href="/rex/docs/build/program-status.md"/` |
| docs-index.html (bonus) | `rel="alternate" type="text/markdown" href="/rex/docs/installation.md"/` |

Full tags (one per page, exactly one match each):
```
cli.html:            <link rel="alternate" type="text/markdown" href="/rex/docs/automate/cli.md"/>
server-api.html:     <link rel="alternate" type="text/markdown" href="/rex/docs/automate/server-api.md"/>
lua.html:            <link rel="alternate" type="text/markdown" href="/rex/docs/reference/lua.md"/>
events.html:         <link rel="alternate" type="text/markdown" href="/rex/docs/reference/events.md"/>
lua-events.html:     <link rel="alternate" type="text/markdown" href="/rex/docs/automate/lua-events.md"/>
lua-scripts.html:    <link rel="alternate" type="text/markdown" href="/rex/docs/automate/lua-scripts.md"/>
program-status.html: <link rel="alternate" type="text/markdown" href="/rex/docs/build/program-status.md"/>
docs-index.html:     <link rel="alternate" type="text/markdown" href="/rex/docs/installation.md"/>
```
Every HTML page's advertised `.md` alternate corresponds to an existing `.md` URL that returned HTTP 200 in this batch.

---

## 5. DOC-001 original artifacts — existence and direct comparison

**Does `/tmp/relayable-protocol-compat/raw/` exist?** **YES.** It was read-only inspected; nothing in it was deleted or modified.

```
$ ls -la /tmp/relayable-protocol-compat/raw/
total 428
-rw-r--r-- 1 sulthonuladib sulthonuladib 25586 Oct 15 23:07 cli.html
-rw-r--r-- 1 sulthonuladib sulthonuladib  4664 Oct 15 23:07 cli.md
-rw-r--r-- 1 sulthonuladib sulthonuladib  4210 Oct 15 23:04 cli.txt
-rw-r--r-- 1 sulthonuladib sulthonuladib 15943 Oct 15 23:04 docs-index.html
-rw-r--r-- 1 sulthonuladib sulthonuladib 31465 Oct 15 23:04 events.html
-rw-r--r-- 1 sulthonuladib sulthonuladib  7308 Oct 15 23:04 events.md
-rw-r--r-- 1 sulthonuladib sulthonuladib   611 Oct 15 23:05 installation.md
-rw-r--r-- 1 sulthonuladib sulthonuladib 33998 Oct 15 23:04 lua-events.html
-rw-r--r-- 1 sulthonuladib sulthonuladib  5863 Oct 15 23:04 lua-events.md
-rw-r--r-- 1 sulthonuladib sulthonuladib 50429 Oct 15 23:03 lua.html
-rw-r--r-- 1 sulthonuladib sulthonuladib 12854 Oct 15 23:04 lua.md
-rw-r--r-- 1 sulthonuladib sulthonuladib 43105 Oct 15 23:04 lua-scripts.html
-rw-r--r-- 1 sulthonuladib sulthonuladib  6860 Oct 15 23:04 lua-scripts.md
-rw-r--r-- 1 sulthonuladib sulthonuladib 45357 Oct 15 23:04 program-status.html
-rw-r--r-- 1 sulthonuladib sulthonuladib 16582 Oct 15 23:04 program-status.md
-rw-r--r-- 1 sulthonuladib sulthonuladib    63 Oct 15 23:04 robots.txt
-rw-r--r-- 1 sulthonuladib sulthonuladib 38860 Oct 15 23:07 server-api.html
-rw-r--r-- 1 sulthonuladib sulthonuladib  7088 Oct 15 23:07 server-api.md
-rw-r--r-- 1 sulthonuladib sulthonuladib  6469 Oct 15 23:04 server-api.txt
-rw-r--r-- 1 sulthonuladib sulthonuladib  5642 Oct 15 23:04 sitemap-0.xml
-rw-r--r-- 1 sulthonuladib sulthonuladib  5642 Oct 15 23:04 sitemap-index.xml
-rw-r--r-- 1 sulthonuladib sulthonuladib  5642 Oct 15 23:04 sitemap_index.xml
-rw-r--r-- 1 sulthonuladib sulthonuladib  5642 Oct 15 23:07 sitemap.xml
```

**Yes — the originals were sha256'd directly.** Their hashes match the DOC-001 recorded values in the task brief, confirming the recorded values are genuine and the on-disk DOC-001 artifacts were not altered between DOC-001 and now:

```
feb7a664dcce882e2196d83a1eb9c266add551ef48ce4bbefc8690ef4d753dab  cli.html
e61162828b0876173085622831cbdae56ab8072bbc8f83c7dee51ea933912b93  cli.md
d3ef34cb44e0f3ed89de583ad417d44891debda5d0bffdda19079297d8604a81  cli.txt            (not in DOC-001 list)
f668b09c300daf7b2ddef8f6a62cc0355f50c08b41aa853d49d8aadc34b461c1  docs-index.html
2f49cd8dc63077723262955f45c8fc1b2aa2d40890c795202106a39c0f6629ac  events.html
e0c8b1cdfb0967f674f4dece59f7bc3d17782ab0e928920d8070902c92c7d5a0  events.md
514e3a81ded0e1d7413e06d6e51c3c071ab5cad0955752b8d204f908816d09c7  installation.md
ae6e2b5864cccf526ba61bda462461543e7463fda9407ab1f034f147de8bcec5  lua-events.html
312426e5edef5c80abb34785cda150de4c30acb8045912c4b6139b388d6af4a9  lua-events.md
80ec081d2cb4ac9cd0a94a2b64b2462d2a0e270e839a456575a09f4b2afc1509  lua.html
934773e17693cc6222eff8e40bf79911247500fdc130e92e887bb27678bd1a0f  lua.md
4abac36d940f3a2f3468006b04e5932632328868000fdcf125fd280fa727fbd7  lua-scripts.html
0d8b1547357bda30e9b85e86f054f240fcf604711cf805d66d98f81da556a372  lua-scripts.md
0e1f209b29201f167c944ee1f20465ebfb1c7c9390cbf19ea364eeeb9f5b05f7  program-status.html
9b3f32f6e53989c3d0909ce13cda4fb7dc15e29718f5a096281eb3ac4a951829  program-status.md
b033243267bcbdab84cb01e940811d6d77a894d78f0470faaf0eb43f8e4efcc9  robots.txt
4e8d5040b54d88f6dcdf26b2f50a582f0d46fd022297e4ac3d2065741f51cdec  server-api.html
6e31118649040ff616fd83d4fc743aadb657ef0ad93e7d0a4100eb0785882038  server-api.md
05e30934ab5a14c49f316cf422d21fff46710c00583323ffe7b269d51f9812e5  server-api.txt     (not in DOC-001 list)
cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d  sitemap-0.xml
cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d  sitemap-index.xml
cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d  sitemap_index.xml
cc0f75799dae4f9e64665f415c936fb1d12d7c26b88e3841368e3a7fe48e2a3d  sitemap.xml
```

**Direct byte comparison (`cmp` fresh vs original)** — every pair is `IDENTICAL`:

```
cli.html            vs cli.html:            IDENTICAL
cli.md              vs cli.md:              IDENTICAL
server-api.html     vs server-api.html:     IDENTICAL
server-api.md       vs server-api.md:       IDENTICAL
lua.html            vs lua.html:            IDENTICAL
lua.md              vs lua.md:              IDENTICAL
events.html         vs events.html:         IDENTICAL
events.md           vs events.md:           IDENTICAL
lua-events.html     vs lua-events.html:     IDENTICAL
lua-events.md       vs lua-events.md:       IDENTICAL
lua-scripts.html    vs lua-scripts.html:    IDENTICAL
lua-scripts.md      vs lua-scripts.md:      IDENTICAL
program-status.html vs program-status.html: IDENTICAL
program-status.md   vs program-status.md:   IDENTICAL
docs-index.html     vs docs-index.html:     IDENTICAL
installation.md     vs installation.md:     IDENTICAL
sitemap-xml-1       vs sitemap.xml:         IDENTICAL
sitemap-index-xml   vs sitemap-index.xml:   IDENTICAL
sitemap_index-xml   vs sitemap_index.xml:   IDENTICAL
sitemap-0-xml       vs sitemap-0.xml:       IDENTICAL
robots.txt          vs robots.txt:          IDENTICAL
```

The DOC-001 originals also include two extra artifacts not in the comparison list — `cli.txt` and `server-api.txt` — which are the only files in that directory with no DOC-002 counterpart (DOC-002 fetched the `.md` alternates only). They were left untouched.

**Implication for line-number citations:** because all fresh `.md`/`.html` files are byte-identical to the DOC-001 originals, the artifacts have not drifted, so any DOC-001 line-number citation that was correct against these artifacts is still correct. If a line-number citation is wrong, the cause is a DOC-001 transcription/lookup error, **not** content drift between DOC-001 and this re-verification.

---

## 6. Failures and errors

- **curl-level failures: none.** All 21 `curl_exit=0`; no DNS, TLS, timeout, or connection error text was printed by any request. The "network unavailable → stop after 2 failures" path did not trigger.
- **HTTP-level failures: 4** — all four sitemap URLs returned `HTTP=404` with curl exit 0 (i.e. transport succeeded, resource is gone). Exact status lines:
  ```
  HTTP=404 SIZE=5642 URL=https://www.superlogical.com/sitemap.xml
  HTTP=404 SIZE=5642 URL=https://www.superlogical.com/sitemap-index.xml
  HTTP=404 SIZE=5642 URL=https://www.superlogical.com/sitemap_index.xml
  HTTP=404 SIZE=5642 URL=https://www.superlogical.com/sitemap-0.xml
  ```
  All four bodies are the same 5642-byte SvelteKit 404 page (`<title>404 — Superlogical</title>`, `Sorry, this page doesn't exist.`, `meta name="robots" content="noindex"`). Their identical sha256 `cc0f7579…` also equals DOC-001's recorded value, so DOC-001 saw the same 404s; this is a pre-existing dead endpoint, not new breakage.
- **Redirect of note (not a failure):** URL 15 requested `https://www.superlogical.com/rex/docs` and `url_effective` was `https://www.superlogical.com/rex/docs/installation` — the docs root resolves to the Installation page. DOC-001 recorded the same body hash, so this too pre-dates DOC-002.

---

## 7. Artifacts produced (all under `/tmp/doc002/`, none inside the repo)

| Path | Contents |
|---|---|
| `/tmp/doc002/fetch-report.md` | this report |
| `/tmp/doc002/fetch.sh` | the fetch batch script |
| `/tmp/doc002/urls.txt` | slug→URL list |
| `/tmp/doc002/fetch.log` | full stdout/stderr transcript with per-URL HTTP line, curl_exit, bracketing dates, and sha256 |
| `/tmp/doc002/raw/<slug>` ×21 | fresh fetched bodies |

## 8. Bottom line

Every cited documentation URL in the DOC-001 material is **live and byte-for-byte unchanged** as of `2026-10-15T16:57:25Z–16:57:34Z`: 17× HTTP 200 with matching sha256, and 4× HTTP 404 (`sitemap*.xml`) whose 404-page sha256 also matches DOC-001. The `.md` alternates are genuine Markdown (no HTML shell), and every HTML page still advertises its corresponding `.md` alternate. There is **no evidence of drift**, so DOC-001's recorded hashes and any line numbers derived against these artifacts remain valid; the only actionable finding for citation repair is that the four sitemap URLs are dead (404) and should not be cited as content sources.
