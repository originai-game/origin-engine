#!/usr/bin/env python3
"""Join access sites to their enclosing fn and impl; classify read/write."""
import os
import json, re, collections, pathlib

D = pathlib.Path(__file__).resolve().parent
LANE = pathlib.Path(os.environ.get("OG_UI", "crates/og-ui"))
items = json.load(open(D / "index.json"))
by_file = collections.defaultdict(list)
for x in items:
    by_file[x["file"]].append(x)
srcs = {}


def src_line(f, n):
    if f not in srcs:
        srcs[f] = (LANE / f).read_text().split("\n")
    return srcs[f][n - 1]


def enclosing(f, line):
    fn = impl = None
    for x in by_file.get(f, []):
        if x["start"] <= line <= x["end"]:
            if x["kind"] == "fn" and (fn is None or x["start"] >= fn["start"]):
                fn = x
            if x["kind"] == "impl" and (impl is None or x["start"] >= impl["start"]):
                impl = x
    return fn, impl


WRITE_AFTER = re.compile(
    r"^(\s*(\[[^\]]*\]|\.\w+\b(?!\())*\s*)(=[^=>]|\+=|-=|\*=|/=|\|=|&=|\^=|<<=|>>=|%=)"
    r"|^\s*(\[[^\]]*\]|\.\w+\b(?!\())*\s*\.(push|push_str|insert|clear|retain|remove|extend|truncate|get_mut|entry|iter_mut|values_mut|take|sort\w*|dedup\w*|drain|pop|swap|resize|append|replace|borrow_mut|split_off|as_mut|last_mut|first_mut|splice|fill|copy_from_slice|clone_from|extend_from_slice|reserve|shrink_to_fit|or_insert\w*|set|apply\w*|retain_mut)\b"
)


def access_kind(site):
    f = site["file"].removeprefix("crates/og-ui/")
    line = src_line(f, site["line"])
    if site["code"] == "E0560":
        return "write"  # struct literal
    col = site["col"] - 1
    field = site["field"]
    # find `.field` at or near the column
    k = line.find(field, max(0, col - 1))
    if k < 0:
        k = line.find(field)
    after = line[k + len(field):] if k >= 0 else ""
    before = line[:k] if k >= 0 else ""
    if WRITE_AFTER.search(after):
        return "write"
    if re.search(r"&mut\s+[\w.\[\]()]*\.?$", before) or re.search(r"(mem::(take|replace|swap)\(&mut\s+[\w.\[\]()]*\.?)$", before):
        return "write"
    return "read"


def load(name):
    d = json.load(open(D / f"{name}.json"))
    rows = []
    seen = set()
    for s in d["sites"]:
        k = (s["field"], s["file"], s["line"], s["col"])
        if k in seen:
            continue
        seen.add(k)
        f = s["file"].removeprefix("crates/og-ui/")
        fn, impl = enclosing(f, s["line"])
        rows.append({
            "field": s["field"], "file": f, "line": s["line"],
            "fn": fn["name"] if fn else None,
            "fn_start": fn["start"] if fn else None,
            "impl": impl["head"] if impl else None,
            "impl_start": impl["start"] if impl else None,
            "kind": access_kind(s),
        })
    return d["fields"], rows


if __name__ == "__main__":
    out = {}
    for n in ("ui", "state", "node"):
        fields, rows = load(n)
        out[n] = {"fields": fields, "rows": rows}
        c = collections.Counter(r["kind"] for r in rows)
        print(n, c)
    json.dump(out, open(D / "joined.json", "w"))
