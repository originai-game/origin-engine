"""Calls between Ui methods by stage: which stage's methods call which."""
import os
import json, re, collections, sys
sys.path.insert(0, ".")
from stage import stage

LANE = os.environ.get("OG_UI", "crates/og-ui").rstrip("/") + "/"
items = json.load(open("index.json"))
impls = [x for x in items if x["kind"] == "impl" and re.match(r"impl(<[^>]*>)? Ui$", x["head"])]
fns = [x for x in items if x["kind"] == "fn"]
methods = {}
for b in impls:
    for f in fns:
        if f["file"] == b["file"] and b["start"] < f["start"] <= b["end"]:
            # direct member: not nested in another fn of the block
            if not any(g is not f and g["file"] == f["file"] and g["start"] < f["start"] and f["end"] <= g["end"] and b["start"] < g["start"] for g in fns):
                f["block"] = f"{b['file']}:{b['start']}"
                methods[f["name"]] = f
src = {}
def text(f):
    if f not in src:
        src[f] = open(LANE + f).read().split("\n")
    return src[f]

edges = collections.Counter()
edge_ex = collections.defaultdict(set)
for name, f in methods.items():
    lines = text(f["file"])
    for n in range(f["start"], f["end"] + 1):
        line = lines[n - 1]
        caller = stage(f["file"], f["name"], n)
        for m in re.finditer(r"\b(?:self|ui|this|u)\s*\.\s*(\w+)\s*(?:::<[^>]*>)?\(", line):
            callee = methods.get(m.group(1))
            if not callee:
                continue
            cs = stage(callee["file"], callee["name"], callee["start"])
            edges[(caller, cs)] += 1
            edge_ex[(caller, cs)].add(callee["name"])
json.dump({"methods": {k: {"file": v["file"], "start": v["start"], "end": v["end"], "block": v["block"]} for k, v in methods.items()}}, open("methods.json", "w"))
ORDER = ["tree", "style", "layout", "paint", "docs", "front", "env", "observe", "ctor"]
print("caller\\callee " + " ".join(f"{s[:6]:>7}" for s in ORDER))
for a in ORDER:
    print(f"{a:13} " + " ".join(f"{edges.get((a, b), 0):7}" for b in ORDER))
if len(sys.argv) > 1:
    a, b = sys.argv[1], sys.argv[2]
    print(sorted(edge_ex[(a, b)]))
print(len(methods), "methods")
