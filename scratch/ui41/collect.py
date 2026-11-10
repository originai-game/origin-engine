#!/usr/bin/env python3
"""Rename every field of one struct, cargo check og-ui, collect each access
site from the compiler's field errors, restore the file.

usage: collect.py LANE STRUCT FILE START_LINE OUT.json
"""
import json, re, subprocess, sys, pathlib

lane, struct, rel, start, out = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
path = pathlib.Path(lane) / rel
src = path.read_text()
lines = src.split("\n")
# struct body: from START_LINE (1-based, the `struct` line) to the matching close brace
i = start - 1
assert re.search(r"struct %s\b" % struct, lines[i]), lines[i]
depth = 0
fields = []
for j in range(i, len(lines)):
    line = lines[j]
    if depth == 1:
        m = re.match(r"^(\s{4})((?:pub(?:\(crate\))? )?)(r#)?(\w+):", line)
        if m:
            fields.append((j, m.group(4)))
    depth += line.count("{") - line.count("}")
    if depth == 0 and j > i:
        end = j
        break
new = list(lines)
for j, name in fields:
    new[j] = re.sub(r"\b%s:" % name, name + "_zq:", new[j], count=1)
path.write_text("\n".join(new))
try:
    p = subprocess.run(
        ["cargo", "check", "-p", "og-ui", "--lib", "--tests", "--message-format=json"],
        cwd=lane, capture_output=True, text=True,
    )
finally:
    path.write_text(src)
names = {n for _, n in fields}
sites = []
for raw in p.stdout.splitlines():
    try:
        msg = json.loads(raw)
    except Exception:
        continue
    if msg.get("reason") != "compiler-message":
        continue
    d = msg["message"]
    code = (d.get("code") or {}).get("code")
    text = d["message"]
    fm = re.search(r"field `(\w+)`|field named `(\w+)`|fields? (`[\w`, ]+`)", text)
    if code not in ("E0609", "E0615", "E0560", "E0026", "E0063", "E0027", "E0559"):
        continue
    found = []
    for g in re.findall(r"`(\w+)`", text):
        if g in names:
            found.append(g)
    span = next((s for s in d["spans"] if s["is_primary"]), d["spans"][0])
    for f in found or ["?"]:
        sites.append({
            "field": f, "code": code, "file": span["file_name"],
            "line": span["line_start"], "col": span["column_start"],
            "text": span["text"][0]["text"] if span["text"] else "",
            "msg": text,
        })
json.dump({"struct": struct, "fields": [n for _, n in fields], "sites": sites}, open(out, "w"), indent=1)
print(struct, len(fields), "fields", len(sites), "sites")
