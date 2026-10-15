#!/usr/bin/env python3
"""Citation line-number audit for the REX reference behavior matrix (DOC-002).

Reads the matrix (repo, read-only) and the fresh canonical raw markdown
sources in /tmp/doc002/raw/, extracts every markdown-link citation, locates
its quoted evidence verbatim in the source file, and emits machine-derived
line numbers + verdicts.

Writes ONLY under /tmp/doc002/. Never touches the repository, never runs git
write commands, never uses the network.
"""
import re
import os
from collections import Counter, defaultdict

MATRIX = "/home/sulthonuladib/.herdr/worktrees/relayable/research-doc-002/docs/compatibility/reference-behavior-matrix.md"
RAW = "/tmp/doc002/raw"

SOURCES = [
    "cli.md", "server-api.md", "lua.md", "events.md", "lua-events.md",
    "lua-scripts.md", "program-status.md", "installation.md",
]


def norm(s):
    s = s.replace("\u2026", " ")
    s = re.sub(r"\.\.\.", " ", s)
    s = s.replace("*", "").replace("`", "")
    s = re.sub(r"\s+", " ", s)
    return s.strip()


def despaced(s):
    return re.sub(r"\s+", "", norm(s))


class Index:
    def __init__(self, lines, despaced_mode=False):
        self.text = []
        self.line_of = []
        for i, line in enumerate(lines, 1):
            t = despaced(line) if despaced_mode else norm(line)
            for ch in t:
                self.text.append(ch)
                self.line_of.append(i)
            self.text.append(" ")
            self.line_of.append(i)
        self.text = "".join(self.text)

    def find_all(self, cand):
        res = []
        if not cand:
            return res
        start = 0
        while True:
            i = self.text.find(cand, start)
            if i < 0:
                break
            res.append((self.line_of[i], self.line_of[i + len(cand) - 1]))
            start = i + 1
        return res


def load_indexes():
    idx = {}
    for f in SOURCES:
        with open(os.path.join(RAW, f), encoding="utf-8") as fh:
            lines = fh.read().splitlines()
        idx[f] = {"norm": Index(lines, False), "desp": Index(lines, True), "lines": lines}
    return idx


INDEXES = load_indexes()


def parse_matrix():
    with open(MATRIX, encoding="utf-8") as fh:
        lines = fh.read().splitlines()
    link_re = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
    rows = []
    heading = ""
    for ln, line in enumerate(lines, 1):
        h = re.match(r"^(#{1,4})\s+(.*)$", line)
        if h:
            heading = h.group(2).strip()
        for m in link_re.finditer(line):
            rows.append({
                "matrix_line": ln,
                "heading": heading,
                "link_text": m.group(1),
                "url": m.group(2),
                "matrix_text": line,
                "span": m.span(),
            })
    return rows, lines


LINE_TEXT_RE = re.compile(r"^([A-Za-z0-9._-]+\.md)(?::\s*(.+))?$")


def classify(link_text):
    lt = link_text.strip()
    m = LINE_TEXT_RE.match(lt)
    if m:
        f, nums = m.group(1), m.group(2)
        if f == "full.md":
            return "full.md", None, "repo"
        if f not in SOURCES:
            return f, nums, "other"
        if not nums:
            return f, None, "noline"
        return f, nums.strip(), "source"
    if re.match(r"^full\.md\b", lt):
        return "full.md", None, "repo"
    if lt.endswith(".md"):
        return lt, None, "other"
    return None, None, "other"


def parse_cited(lines_str):
    if not lines_str:
        return []
    out = []
    for part in re.split(r"[,;]", lines_str):
        part = part.strip()
        if not part:
            continue
        part = part.replace("\u2013", "-").replace("\u2014", "-").replace("\u2012", "-")
        m = re.match(r"^(\d+)\s*-\s*(\d+)$", part)
        if m:
            out.append((int(m.group(1)), int(m.group(2))))
            continue
        m = re.match(r"^(\d+)$", part)
        if m:
            v = int(m.group(1))
            out.append((v, v))
    return out


def ranges_from_set(s):
    s = sorted(s)
    if not s:
        return []
    out = []
    start = prev = s[0]
    for v in s[1:]:
        if v == prev + 1:
            prev = v
            continue
        out.append((start, prev))
        start = prev = v
    out.append((start, prev))
    return out


def split_unescaped_pipe(line):
    return re.split(r"(?<!\\)\|", line)


def row_evidence(row):
    line = row["matrix_text"]
    if line.lstrip().startswith("|"):
        cells = [c.strip() for c in split_unescaped_pipe(line)]
        while cells and cells[0] == "":
            cells.pop(0)
        while cells and cells[-1] == "":
            cells.pop()
        src_i = None
        for i, c in enumerate(cells):
            if re.search(r"\]\([^)]*\)", c):
                src_i = i
                break
        ev_cells = cells[:src_i] if src_i is not None else cells
        evidence = " ".join(ev_cells)
    else:
        evidence = line[:row["span"][0]]
    return evidence


def candidate_quotes(evidence):
    qs = re.findall(r'"((?:[^"\\]|\\.)*)"', evidence)
    out = []
    for q in qs:
        q = q.replace('\\"', '"').replace("\\\\", "\\")
        if len(norm(q)) >= 12:
            out.append(q)
    return out


def code_segments(evidence):
    ev = re.sub(r'"((?:[^"\\]|\\.)*)"', " ", evidence)
    ev = re.sub(r"\.\.\.", "\x00", ev).replace("\u2026", "\x00")
    ev = re.sub(r"[{}\[\]();]", "\x00", ev)
    segs = []
    for seg in ev.split("\x00"):
        seg = seg.strip(" \t,;:.-")
        if len(norm(seg)) >= 8 and re.search(r"[A-Za-z]", seg):
            segs.append(seg)
    return segs


def search_any(cand):
    c = norm(cand)
    if c:
        ms = INDEXES[SRC]["norm"].find_all(c)
        if ms:
            return "norm", ms
    d = despaced(cand)
    if d:
        ms = INDEXES[SRC]["desp"].find_all(d)
        if ms:
            return "desp", ms
    return None, []


def locate_quote(q):
    """Try the full quote, then elided/structural segments, then sentences.
    Return (candidate, matches) for the first strategy that locates text."""
    attempts = [q]
    s = re.sub(r"\.\.\.", "\x00", q).replace("\u2026", "\x00")
    s = re.sub(r"[{}\[\]]", "\x00", s)
    attempts += [x.strip(" \t,;:.-") for x in s.split("\x00")]
    attempts += [x.strip() for x in re.split(r"(?<=[.;])\s+", q)]
    for a in attempts:
        if len(norm(a)) < 12:
            continue
        mode, ms = search_any(a)
        if ms:
            return a, ms
    return None, []


def nearest_heading(source, line_no):
    lines = INDEXES[source]["lines"]
    best = None
    for i in range(min(line_no, len(lines)), 0, -1):
        if re.match(r"^#{1,4} ", lines[i - 1]):
            best = (i, lines[i - 1].strip())
            break
    return best


def main():
    global SRC
    rows, _ = parse_matrix()
    out = []
    for r in rows:
        src, nums, kind = classify(r["link_text"])
        rec = dict(r)
        rec.update({"kind": kind, "source": src, "nums": nums,
                    "cited": parse_cited(nums) if kind == "source" else [],
                    "actual": [], "verdict": "", "mode": "", "cand": "",
                    "evidence": row_evidence(r), "note": "", "nhead": None})
        if kind == "source" and src in INDEXES:
            SRC = src
            evidence = rec["evidence"]
            quotes = candidate_quotes(evidence)
            matched = set()
            cand_desc = []
            for q in quotes:
                cand, ms = locate_quote(q)
                if ms:
                    cand_desc.append(f"{cand[:50]} -> {[(a, b) for a, b in ms]}")
                    for a, b in ms:
                        matched.update(range(a, b + 1))
            if not quotes:
                for seg in code_segments(evidence):
                    mode, ms = search_any(seg)
                    if ms:
                        cand_desc.append(f"[code] {seg[:50]} -> {[(a, b) for a, b in ms]}")
                        for a, b in ms:
                            matched.update(range(a, b + 1))
            rec["actual"] = ranges_from_set(matched)
            rec["cand"] = " | ".join(cand_desc)[:400]
            if not matched:
                rec["verdict"] = "NOT_FOUND"
                if not cand_desc and not quotes:
                    rec["note"] = "no verbatim quote in row evidence (paraphrase/unknown)"
                elif "not documented" in evidence:
                    rec["note"] = "row is an 'unknown' row; evidence is the not-documented marker"
                else:
                    rec["note"] = "quoted text not found verbatim"
                start = rec["cited"][0][0] if rec["cited"] else 1
                rec["nhead"] = nearest_heading(src, start)
            else:
                cited_set = set()
                for a, b in rec["cited"]:
                    cited_set.update(range(a, b + 1))
                rec["verdict"] = "MATCH" if matched <= cited_set else "SHIFT"
                if len(matched) > 1:
                    rr = ranges_from_set(matched)
                    span = rr[-1][1] - rr[0][0]
                    if span > 40 and not (matched & cited_set):
                        rec["verdict"] = "AMBIGUOUS"
            rec["mode"] = "norm/desp"
        elif kind == "noline":
            rec["verdict"] = "NO_LINE_REF"
            rec["note"] = "source link has no line numbers"
        else:
            rec["verdict"] = "NO_LINE_REF"
            rec["note"] = "repo text" if src in ("full.md",) else "no line numbers in link"
        out.append(rec)

    with open("/tmp/doc002/citation-raw.tsv", "w", encoding="utf-8") as fh:
        fh.write("matrix_line\theading\tlink_text\tsource\tcited\tactual\tverdict\tnote\tcandidate\tevidence\n")
        for r in out:
            cited = ";".join(f"{a}-{b}" if a != b else str(a) for a, b in r["cited"])
            actual = ";".join(f"{a}-{b}" if a != b else str(a) for a, b in r["actual"])
            fh.write("\t".join([
                str(r["matrix_line"]), r["heading"], r["link_text"], str(r["source"]),
                cited, actual, r["verdict"], r["note"],
                r["cand"].replace("\t", " "), r["evidence"][:400].replace("\t", " "),
            ]) + "\n")

    vc = Counter(r["verdict"] for r in out)
    print("TOTAL citation links:", len(out))
    for k, v in vc.most_common():
        print(f"  {k}: {v}")
    per = defaultdict(Counter)
    for r in out:
        per[r["source"]][r["verdict"]] += 1
    print()
    print("per-source (all links, incl. no-line-ref):")
    for f in SOURCES:
        c = per[f]
        print(f"  {f}: total={sum(c.values())} {dict(c)}")
    print()
    print("no-line-ref links (source links w/o line numbers):")
    for r in out:
        if r["kind"] == "noline":
            print(f"  matrix {r['matrix_line']}: {r['link_text']} ({r['source']}) actual={r['actual']}")
    return out


if __name__ == "__main__":
    SRC = None
    main()
