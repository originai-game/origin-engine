"""Every `impl Ui` block: its stage, the states it reads and writes, its
calls into other stages' methods, and the target state."""
import os
import json, re, collections, sys
sys.path.insert(0, ".")
from stage import stage
import owners

LANE = os.environ.get("OG_UI", "crates/og-ui").rstrip("/") + "/"
items = json.load(open("index.json"))
j = json.load(open("joined.json"))
methods = json.load(open("methods.json"))["methods"]
impls = [x for x in items if x["kind"] == "impl" and re.match(r"impl(<[^>]*>)? Ui$", x["head"])]
fns = [x for x in items if x["kind"] == "fn"]

TARGET = {"tree": None, "style": "Style", "layout": "Layout", "paint": "Paint", "docs": "Docs",
          "front": "Ui (front end)", "env": "Ui (configuration API over Env)", "observe": "Ui (observers, &self)", "ctor": "Ui"}


def in_block(r, b):
    return r["file"] == b["file"] and b["start"] <= r["line"] <= b["end"]


src = {}
def text(f):
    if f not in src:
        src[f] = open(LANE + f).read().split("\n")
    return src[f]

rows_out = []
for b in impls:
    member = [f for f in fns if f["file"] == b["file"] and b["start"] < f["start"] <= b["end"]
              and not any(g is not f and g["file"] == f["file"] and b["start"] < g["start"] < f["start"] and f["end"] <= g["end"] for g in fns)]
    # stage by line weight
    weight = collections.Counter()
    for f in member:
        if f["name"] == "end_frame":
            for n in range(f["start"], f["end"] + 1):
                weight[stage(f["file"], f["name"], n)] += 1
        else:
            weight[stage(f["file"], f["name"], f["start"])] += f["end"] - f["start"] + 1
    st = weight.most_common(1)[0][0] if weight else "?"
    R, W = collections.Counter(), collections.Counter()
    for r in j["ui"]["rows"]:
        if in_block(r, b) and r["field"] not in ("state", "nodes"):
            (W if r["kind"] == "write" else R)[owners.UI[r["field"]]] += 1
    for r in j["state"]["rows"]:
        if in_block(r, b):
            (W if r["kind"] == "write" else R)[owners.STATE[r["field"]]] += 1
    for r in j["node"]["rows"]:
        if in_block(r, b):
            (W if r["kind"] == "write" else R)["node." + owners.NODE[r["field"]]] += 1
    if st == "tree":
        it = R["Interaction"] + W["Interaction"] + R["node.Interaction"] + W["node.Interaction"]
        tr = R["Tree"] + W["Tree"] + R["node.Tree"] + W["node.Tree"] + R["node.Authored"] + W["node.Authored"]
        target = "Interaction" if it > tr else "Tree"
    else:
        target = TARGET.get(st, "?")
    # calls to other stages
    calls = collections.Counter()
    lines = text(b["file"])
    for n in range(b["start"], b["end"] + 1):
        for m in re.finditer(r"\b(?:self|ui)\s*\.\s*(\w+)\s*(?:::<[^>]*>)?\(", lines[n - 1]):
            c = methods.get(m.group(1))
            if c and c["block"] != f"{b['file']}:{b['start']}":
                cs = stage(c["file"], m.group(1), c["start"])
                if cs != st:
                    calls[cs] += 1
    rows_out.append({
        "block": f"{b['file']}:{b['start']}", "lines": b["end"] - b["start"] + 1, "stage": st,
        "target": target, "fns": len(member),
        "names": [f["name"] for f in member],
        "R": dict(R), "W": dict(W), "calls": dict(calls),
    })
json.dump(rows_out, open("blocks.json", "w"), indent=1)

def short(c):
    return ", ".join(f"{k} {v}" for k, v in sorted(c.items(), key=lambda kv: -kv[1]))

if __name__ == "__main__":
    for r in rows_out:
        print(f"{r['block']:42} {r['lines']:5} {r['stage']:8} -> {r['target']:14} W[{short(r['W'])}] calls[{short(r['calls'])}]")
