# DOC-002 evidence — citation audit + URL drift re-check (durable copy)

**Status:** evidence complete; **the matrix repair itself is NOT applied.** The DOC-002 Lead ran to
the wind-down, and its session had agent-management tools only (no `bash`/`edit`/`write`), so the
single-file edit could not be landed without launching a new writer — which wind-down forbade.
This directory is the durable copy of the evidence produced under `/tmp/doc002/` on 2026-10-15
(`/tmp` is volatile; this copy is the authoritative one).

## Findings

1. **URL drift: zero.** All 21 recorded URLs were re-fetched `2026-10-15T16:57:25Z–16:57:34Z` with
   `curl -sS -L --max-time 30 --connect-timeout 15`. Every HTTP-200 body's fresh `sha256` equals the
   DOC-001 recorded value and is `cmp`-identical to the original artifacts in
   `/tmp/relayable-protocol-compat/raw/`. The 4 `sitemap*.xml` variants remain HTTP 404 (already 404
   in DOC-001). The `.md` alternates are still genuine Markdown. **Conclusion: the line-number defect
   is a DOC-001 transcription error, not content drift.**
2. **The defect is broader than `events.md`.** 433 links = 386 line-number citations + 47 non-line
   refs. Verdicts: **193 MATCH / 115 SHIFT / 78 NOT_FOUND / 0 AMBIGUOUS** across `cli.md`,
   `server-api.md`, `lua.md`, `events.md`, `lua-events.md`, `lua-scripts.md`, `program-status.md`.
   An events-only fix would leave ~142 wrong citations standing.

## Files

| File | Contents |
|---|---|
| `citation-audit.md` | Full 386-row citation table + 75-row `events.md` correction list + per-file heading outlines with line numbers + summary counts |
| `citation-raw.tsv` | Machine-readable per-citation rows |
| `fetch-report.md` | Per-URL HTTP status, fresh vs DOC-001 sha256, drift verdict, batch window, curl shape |
| `audit.py`, `adjudicate.py` | Helper scripts that derived the audit (regenerable) |
| `urls.txt`, `fetch.sh`, `fetch.log` | Exact fetch set, fetch script, and command log |

## Next step (one bounded writer pass)

Apply `citation-audit.md` to `docs/compatibility/reference-behavior-matrix.md`:

1. correct **all 7 source files'** line-number citations (not only `events.md`);
2. convert `NOT_FOUND`/paraphrase rows to `[file.md §<heading>](url)` + the existing verbatim quote —
   never invent a number;
3. add the re-verification record to the §4 retrieval log and update the "as-of" timestamps to the
   re-check window above;
4. keep all `observed-video`/`verified-official-docs`/… labels unchanged and add no REX wire/binary
   compatibility claim (`full.md` §3);
5. commit on `research/doc-002`.
