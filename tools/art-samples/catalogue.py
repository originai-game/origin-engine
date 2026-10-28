#!/usr/bin/env python3
"""Makes the style catalogue honest per style (docs/art.md §2.6, stream A7)
from the sample frames: `look.support` is `content` where the style needs
authored art, `full` where its frame reads as the style (verdicts.json) and
`og style check` passed on its family's scene, else `partial` with
`missing` naming what is still not there. og-taste §12's table follows.
Also writes the evidence: the check's summary per style and the notes'
37-row table.

    catalogue.py <capture-dir> <evidence-dir>

<capture-dir> is capture.ts's --out (<slug>.png, <slug>.style.json).
"""
import json, os, re, sys

repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
cap, evidence = sys.argv[1], sys.argv[2]
cat_path = os.path.join(repo, "packages/knowledge/looks/catalogue.json")
taste_path = os.path.join(repo, "packages/knowledge/skills/og-taste/SKILL.md")
cat = json.load(open(cat_path))
verdicts = json.load(open(os.path.join(os.path.dirname(__file__), "verdicts.json")))["styles"]
STALE = re.compile(r"^the (cook's asset conformance|runtime pieces of docs/art.md)")


def check(slug):
    """og style check's result: (ok, the off assets with their failed checks)."""
    j = json.load(open(os.path.join(cap, f"{slug}.style.json")))
    off = []
    for a in j.get("assets", []):
        if a.get("ok") is False:
            bad = [f"{c['name']} {c['value']} against {c['limit']}" for c in a.get("checks", []) if not c.get("ok")]
            off.append(f"{a['asset']}: {'; '.join(bad)}")
    return not off, off


summary, rows = {}, []
for s in cat["styles"]:
    slug, look = s["slug"], s["look"]
    v = verdicts[slug]
    ok, off = check(slug)
    authored = "authored" in look["layers"]
    kept = [m for m in look.get("missing", []) if not STALE.match(m)]
    if authored:
        support = "content"
        missing = v.get("missing") or kept or [f"the art is content: {s['assets']['spriteScale']}, drawn for the style"]
    elif v["reads"] and ok:
        support, missing = "full", []
    else:
        support = "partial"
        missing = list(v.get("missing", []))
        if not v["reads"] and not missing:
            missing = [v["why"]]
        for o in off:
            if not any(o.split(":")[0] in m for m in missing):
                missing.append(f"og style check on the sample: {o}")
    look["support"], look["missing"] = support, missing
    summary[slug] = {"family": look["family"], "ok": ok, "off": off}
    frame = f"frames/{slug}.jpg"
    rows.append(f"| `{slug}` | {look['family']} | [{slug}.jpg]({frame}) | {'yes' if v['reads'] else 'no'}: {v['why']} | {'pass' if ok else 'off: ' + '; '.join(off)} | {support} |")

json.dump(cat, open(cat_path, "w"), indent=2, ensure_ascii=False)
open(cat_path, "a").write("\n")

taste = open(taste_path).read()
for s in cat["styles"]:
    pat = re.compile(r"^(\| `" + re.escape(s["slug"]) + r"` \| [^|]+ \| [^|]+ \| )(\w+)( \|)", re.M)
    taste, n = pat.subn(lambda m: m.group(1) + s["look"]["support"] + m.group(3), taste)
    assert n == 1, s["slug"]
open(taste_path, "w").write(taste)

os.makedirs(evidence, exist_ok=True)
json.dump(summary, open(os.path.join(evidence, "style-check.json"), "w"), indent=1)
open(os.path.join(evidence, "table.md"), "w").write(
    "| Style | Family | Frame | Reads as the style | og style check | Support |\n|---|---|---|---|---|---|\n" + "\n".join(rows) + "\n")
print(f"{sum(1 for s in cat['styles'] if s['look']['support'] == 'full')} full, "
      f"{sum(1 for s in cat['styles'] if s['look']['support'] == 'partial')} partial, "
      f"{sum(1 for s in cat['styles'] if s['look']['support'] == 'content')} content")
