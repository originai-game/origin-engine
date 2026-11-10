import json, collections, sys
sys.path.insert(0, ".")
from stage import stage

j = json.load(open("joined.json"))
ORDER = ["tree", "style", "layout", "paint", "docs", "front", "env", "observe", "ctor", "test", "?"]
AB = {"tree": "T", "style": "S", "layout": "L", "paint": "P", "docs": "D", "front": "F",
      "env": "E", "observe": "O", "ctor": "C", "test": "X", "?": "?"}


def st(r):
    if "test" in r["file"]:
        return "test"
    return stage(r["file"], r["fn"], r["line"])


def profile(name):
    rows = j[name]["rows"]
    out = {}
    for f in j[name]["fields"]:
        rs = [r for r in rows if r["field"] == f]
        w = collections.Counter(st(r) for r in rs if r["kind"] == "write")
        rd = collections.Counter(st(r) for r in rs if r["kind"] == "read")
        out[f] = (len(rs), w, rd)
    return out


def fmt(c):
    return " ".join(f"{AB[k]}{c[k]}" for k in ORDER if c.get(k))


if __name__ == "__main__":
    for name in sys.argv[1:]:
        for f, (n, w, r) in profile(name).items():
            print(f"{f:22} {n:5}  W[{fmt(w)}]  R[{fmt(r)}]")
