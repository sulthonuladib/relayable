#!/usr/bin/env python3
"""Produce /tmp/doc002/citation-audit.md from citation-raw.tsv + manual
adjudication of the handful of code-fragment/paraphrase rows where the
automatic fragment-union could not be trusted.

Reads only /tmp/doc002 + the raw sources; writes only /tmp/doc002.
"""
import re
import os
from collections import Counter, defaultdict

RAW = "/tmp/doc002/raw"
TSV = "/tmp/doc002/citation-raw.tsv"
OUT = "/tmp/doc002/citation-audit.md"

SOURCE_ORDER = ["cli.md", "server-api.md", "lua.md", "events.md",
                "lua-events.md", "lua-scripts.md", "program-status.md",
                "installation.md"]


def parse_cited(s):
    out = []
    for part in re.split(r"[,;]", s or ""):
        part = part.strip()
        m = re.match(r"^(\d+)\s*-\s*(\d+)$", part)
        if m:
            out.append((int(m.group(1)), int(m.group(2))))
            continue
        m = re.match(r"^(\d+)$", part)
        if m:
            out.append((int(m.group(1)), int(m.group(1))))
    return out


def parse_ranges(s):
    return parse_cited(s)


def fmt(ranges):
    if not ranges:
        return ""
    parts = []
    for a, b in ranges:
        parts.append(str(a) if a == b else f"{a}\u2013{b}")
    return ", ".join(parts)


# ---- manual overrides: (matrix_line, source) -> dict ----
# Only rows where the automatic fragment-union produced a code-fragment
# artifact or where a paraphrase/unknown row's cited line is itself the
# intended anchor.  VERDICT stays within MATCH/SHIFT/NOT_FOUND/AMBIGUOUS.
OVERRIDES = {
    (81, "cli.md"): dict(actual="81", verdict="MATCH", corrected="unchanged (cli.md:81)"),
    (81, "server-api.md"): dict(actual="85-86", verdict="MATCH", corrected="unchanged (server-api.md:85\u201386)"),
    (215, "server-api.md"): dict(actual="65-67,69-70", verdict="MATCH", corrected="unchanged (server-api.md:63\u201371 covers the schema sample)"),
    (295, "lua.md"): dict(actual="125-132", verdict="SHIFT", corrected="lua.md:125\u2013132"),
    (300, "lua.md"): dict(actual="317-323", verdict="MATCH", corrected="unchanged (lua.md:313\u2013327 covers the host table)"),
    (471, "program-status.md"): dict(actual="80-81", verdict="MATCH", corrected="unchanged (program-status.md:79\u201381 covers the fence + syntax lines)"),
    (486, "program-status.md"): dict(actual="110-111", verdict="SHIFT", corrected="program-status.md:110\u2013111"),
    (491, "program-status.md"): dict(actual="124-126", verdict="SHIFT", corrected="program-status.md:124\u2013126"),
    (496, "program-status.md"): dict(actual="153-156", verdict="SHIFT", corrected="program-status.md:153\u2013156"),
    (524, "program-status.md"): dict(actual="168", verdict="MATCH", corrected="unchanged (program-status.md:168)"),
    (525, "program-status.md"): dict(actual="169", verdict="MATCH", corrected="unchanged (program-status.md:169)"),
    (590, "events.md"): dict(actual="71", verdict="MATCH", corrected="unchanged (events.md:71)"),
    # events.md unknown/paraphrase rows: the cited line is the documented anchor
    (85, "events.md"): dict(actual="", verdict="NOT_FOUND",
                            corrected="keep 116 (session_destroyed event); no verbatim quote \u2014 nearest heading `## Server Events` (raw line 108)"),
    (368, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="drop number; heading `## Events in Lua` (raw line 21) + quote (field definitions at 34\u201346)"),
    (410, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:91 (`terminal.progress_report` row); no verbatim quote"),
    (445, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:68, 120 (`session_view_changed` revision; `keymap_changed` generation/hash); no verbatim quote"),
    (446, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="keep 71 (`connection_state` field list); values not documented"),
    (581, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:115\u2013125 (server-events table); no verbatim quote"),
    (582, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:124\u2013125 (`client` payload table); no verbatim quote"),
    (591, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="keep 71 (`connection_state` field list); values not documented"),
    (592, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:69\u201371 (client connection events); no verbatim quote"),
    (618, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:36\u201338 (`ev.type`, `ev.session_id`, `ev.payload`); row is a paraphrase"),
    (619, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="keep 34\u201346 (field table); no verbatim quote"),
    (620, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:157\u2013159 (`rex events` scope table); row is a paraphrase"),
    (623, "events.md"): dict(actual="", verdict="NOT_FOUND",
                             corrected="events.md:59\u2013129 (event tables); row is a paraphrase"),
    (593, "events.md"): dict(actual="69-71,124-125", verdict="SHIFT", corrected="events.md:69\u201371, 124\u2013125"),
}


def load_rows():
    # build url map from the matrix (read-only)
    matrix_path = "/home/sulthonuladib/.herdr/worktrees/relayable/research-doc-002/docs/compatibility/reference-behavior-matrix.md"
    url_map = {}
    link_re = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
    with open(matrix_path, encoding="utf-8") as fh:
        for ln, line in enumerate(fh.read().splitlines(), 1):
            for m in link_re.finditer(line):
                url_map[(ln, m.group(1))] = m.group(2)
    with open(TSV, encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        rows = []
        for line in fh:
            vals = line.rstrip("\n").split("\t")
            d = dict(zip(header, vals))
            d["url"] = url_map.get((int(d["matrix_line"]), d["link_text"]), "")
            rows.append(d)
    return rows


def nearest_heading(lines, line_no):
    for i in range(min(line_no, len(lines)), 0, -1):
        if re.match(r"^#{1,4} ", lines[i - 1]):
            return i, lines[i - 1].strip()
    return None, None


def heading_outline():
    out = []
    for f in SOURCE_ORDER:
        with open(os.path.join(RAW, f), encoding="utf-8") as fh:
            lines = fh.read().splitlines()
        n = len(lines)
        out.append(f"### {f} ({n} lines)")
        fence = False
        for i, line in enumerate(lines, 1):
            if line.startswith("```"):
                fence = not fence
                continue
            if re.match(r"^#{1,4} ", line):
                tag = " \u2014 **code block, not a heading**" if fence else ""
                out.append(f"- {i}: `{line.strip()}`{tag}")
        out.append("")
    return "\n".join(out)


def main():
    rows = load_rows()
    raw_cache = {}
    for f in SOURCE_ORDER:
        with open(os.path.join(RAW, f), encoding="utf-8") as fh:
            raw_cache[f] = fh.read().splitlines()

    final = []
    for d in rows:
        ml = int(d["matrix_line"])
        src = d["source"]
        if src in SOURCE_ORDER:
            d["kind"] = "source" if d["cited"] else "noline"
        elif src in ("full.md", "DECISIONS.md"):
            d["kind"] = "repo"
        else:
            d["kind"] = "other"
        key = (ml, src)
        verdict = d["verdict"]
        actual = d["actual"]
        corrected = ""
        note = d["note"]
        if key in OVERRIDES:
            ov = OVERRIDES[key]
            actual = ov["actual"]
            verdict = ov["verdict"]
            corrected = ov["corrected"]
        # derive corrected ref if not overridden
        if not corrected:
            if verdict == "MATCH":
                corrected = "unchanged; cited correct"
            elif verdict == "SHIFT":
                corrected = f"{src}:{fmt(parse_ranges(actual))}"
            elif verdict in ("NOT_FOUND", "AMBIGUOUS"):
                cited = parse_cited(d["cited"])
                start = cited[0][0] if cited else 1
                hi, ht = nearest_heading(raw_cache[src], start)
                if hi:
                    corrected = f"drop number; heading `{ht}` (raw line {hi}) + quote"
                else:
                    corrected = "no verbatim quote"
            else:
                corrected = "n/a"
        rec = dict(d)
        rec["verdict_final"] = verdict
        rec["actual_final"] = actual
        rec["corrected"] = corrected
        final.append(rec)

    # counts
    vc = Counter(r["verdict_final"] for r in final)
    per = defaultdict(Counter)
    for r in final:
        per[r["source"]][r["verdict_final"]] += 1

    # source-file citations with line numbers only
    line_cites = [r for r in final if r["kind"] == "source"]
    vc2 = Counter(r["verdict_final"] for r in line_cites)

    lines_out = []
    A = lines_out.append
    A("# Citation line-number audit \u2014 Relayable reference behavior matrix (DOC-002)")
    A("")
    A("**Scope:** every markdown-link citation in `docs/compatibility/reference-behavior-matrix.md` "
      "(read-only) checked against the fresh canonical raw Markdown sources in `/tmp/doc002/raw/`.")
    A("**Method:** fix-string search of a distinctive core substring of the cited quote, "
      "case-sensitive, after stripping `*`/`` ` ``/`…`/`...` and collapsing whitespace (with a "
      "whitespace-insensitive fallback); the full source is normalized into a char→line index so a "
      "multi-line quote yields its real line range. Helper script: `/tmp/doc002/audit.py`.")
    A("")
    A("**Matrix length:** the matrix is 648 lines by `wc -l` (649 by the reader, i.e. a final line "
      "without a trailing newline). All matrix line numbers below are 1-based `wc -l`-style lines.")
    A("")
    A("## Summary counts")
    A("")
    A(f"- Total markdown-link citations found: **{len(final)}**")
    A(f"  - documentation citations with explicit line numbers: **{len(line_cites)}**")
    A(f"  - documentation links with no line numbers (source `.md` only): **{sum(1 for r in final if r['kind']=='noline')}**")
    A(f"  - repository-text links (`full.md`, `DECISIONS.md`): **{sum(1 for r in final if r['kind']=='repo')}**")
    A("")
    A("### Verdicts, line-number citations only (n=%d)" % len(line_cites))
    A("")
    A("| Verdict | Count |")
    A("| --- | ---: |")
    for k in ["MATCH", "SHIFT", "NOT_FOUND", "AMBIGUOUS"]:
        A(f"| {k} | {vc2.get(k,0)} |")
    A(f"| **total** | **{len(line_cites)}** |")
    A("")
    A("### Per-source-file breakdown (line-number citations)")
    A("")
    A("| Source | MATCH | SHIFT | NOT_FOUND | AMBIGUOUS | total |")
    A("| --- | ---: | ---: | ---: | ---: | ---: |")
    for f in SOURCE_ORDER:
        c = per[f]
        tot = sum(c.get(k, 0) for k in ["MATCH", "SHIFT", "NOT_FOUND", "AMBIGUOUS"])
        A(f"| {f} | {c.get('MATCH',0)} | {c.get('SHIFT',0)} | {c.get('NOT_FOUND',0)} | {c.get('AMBIGUOUS',0)} | {tot} |")
    A("")
    A("### Per-source-file breakdown (all links, incl. no-line-ref)")
    A("")
    A("| Source | MATCH | SHIFT | NOT_FOUND | AMBIGUOUS | NO_LINE_REF | total |")
    A("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for f in SOURCE_ORDER:
        c = per[f]
        tot = sum(c.values())
        A(f"| {f} | {c.get('MATCH',0)} | {c.get('SHIFT',0)} | {c.get('NOT_FOUND',0)} | {c.get('AMBIGUOUS',0)} | {c.get('NO_LINE_REF',0)} | {tot} |")
    A("")

    # events.md highlights
    ev = [r for r in final if r["source"] == "events.md"]
    ev_shift = [r for r in ev if r["verdict_final"] == "SHIFT"]
    ev_nf = [r for r in ev if r["verdict_final"] == "NOT_FOUND"]
    A("## events.md headline (core deliverable)")
    A("")
    A(f"- events.md citations: **{len(ev)}** \u2014 MATCH **{sum(1 for r in ev if r['verdict_final']=='MATCH')}**, "
      f"SHIFT **{len(ev_shift)}**, NOT_FOUND **{len(ev_nf)}**, AMBIGUOUS **{sum(1 for r in ev if r['verdict_final']=='AMBIGUOUS')}**.")
    A(f"- SHIFT or NOT_FOUND (i.e. the cited line number is wrong or the row has no verbatim quote): "
      f"**{len(ev_shift)+len(ev_nf)}** of {len(ev)}.")
    A("- The full correction list is in \u00a7\"events.md corrections\" below.")
    A("")

    # heading outlines
    A("## Heading outlines (with line numbers)")
    A("")
    A("`grep -n '^#\\{1,4\\} '` per file, with `#` lines that fall inside fenced code blocks marked. "
      "`installation.md` has no headings other than `# Installation` and no fenced blocks.")
    A("")
    A(heading_outline())

    # no line ref table
    A("## Citations without line numbers (\u201cno line ref\u201d)")
    A("")
    A("| matrix line | section | raw citation | source | kind |")
    A("| ---: | --- | --- | --- | --- |")
    for r in final:
        if r["verdict_final"] == "NO_LINE_REF":
            A(f"| {r['matrix_line']} | {r['heading']} | `[{r['link_text']}]({r['url']})` | {r['source']} | {r['kind']} |")
    A("")

    # full table
    A("## Full citation table")
    A("")
    A("`actual` is the machine-located line/range of the cited quote; `verdict` is adjudicated. "
      "Rows marked with \u2020 were manually adjudicated (code-fragment union artifact or paraphrase anchor).")
    A("")
    A("| matrix line | table/section | raw citation | source | cited lines | actual lines found | verdict | corrected ref |")
    A("| ---: | --- | --- | --- | --- | --- | --- | --- |")
    for r in final:
        if r["verdict_final"] == "NO_LINE_REF":
            continue
        mark = "\u2020" if (int(r["matrix_line"]), r["source"]) in OVERRIDES else ""
        A(f"| {r['matrix_line']}{mark} | {r['heading']} | `[{r['link_text']}]({r['url']})` | "
          f"{r['source']} | {r['cited'] or '\u2014'} | {r['actual_final'] or '\u2014'} | "
          f"{r['verdict_final']} | {r['corrected']} |")
    A("")

    # events corrections
    A("## events.md corrections")
    A("")
    A("One row per events.md citation. \u201cunchanged\u201d means the cited line is already correct.")
    A("")
    A("| # | matrix line | section | cited ref | actual lines | verdict | corrected ref |")
    A("| ---: | ---: | --- | --- | --- | --- | --- |")
    for i, r in enumerate(ev, 1):
        A(f"| {i} | {r['matrix_line']} | {r['heading']} | `{r['link_text']}` | "
          f"{r['actual_final'] or '\u2014'} | {r['verdict_final']} | {r['corrected']} |")
    A("")

    # sanity control
    A("## Method sanity control (3 known-good cli.md citations)")
    A("")
    A("The method was calibrated on three citations whose cited line is unmistakable:")
    A("")
    A("| citation | quoted text | cited | located | verdict |")
    A("| --- | --- | --- | --- | --- |")
    A("| `cli.md:8` | `rex` is the command-line interface to Rex. | 8 | 8 | MATCH |")
    A("| `cli.md:37` | A **window** is a tab within a session. It holds a layout of blocks. | 37 | 37 | MATCH |")
    A("| `cli.md:38` | A **block** is one application in a window. Today that means a terminal. | 38 | 38 | MATCH |")
    A("")
    A("These three come out MATCH, so the method is not returning mass failure. The SHIFT/NOT_FOUND "
      "results below are therefore genuine line-number defects (or genuine \u201cnot documented\u201d rows), "
      "not a broken matcher.")
    A("")

    # caveats
    A("## Caveats")
    A("")
    A("- Rows whose row evidence is a paraphrase or an `unknown` marker (no double-quoted verbatim "
      "excerpt) cannot be located verbatim; they are reported NOT_FOUND and the corrected ref points "
      "at the nearest heading or the documented anchor line.")
    A("- Rows citing code samples (JSON/grammar tables) were manually adjudicated because a "
      "fragment-union search can match a fragment that also appears elsewhere; the "
      "\u2020-marked rows carry the adjudicated result.")
    A("- Nothing in the repository was modified. No network access was used.")
    A("")

    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines_out) + "\n")

    print("wrote", OUT)
    print("total", len(final), "line_cites", len(line_cites), "verdicts", dict(vc2))
    for f in SOURCE_ORDER:
        c = per[f]
        print(f, "total", sum(c.values()), dict(c))


if __name__ == "__main__":
    main()
