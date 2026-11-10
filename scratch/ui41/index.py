#!/usr/bin/env python3
"""Index og-ui's items: impl blocks and fns with line spans (brace matched
on source with comments, strings and chars blanked). Writes index.json."""
import json, re, sys, pathlib

root = pathlib.Path(sys.argv[1])  # crates/og-ui
out = sys.argv[2]


def blank(s):
    o = list(s)
    i, n = 0, len(s)
    def fill(a, b):
        for k in range(a, b):
            if o[k] != "\n":
                o[k] = " "
    while i < n:
        c = s[i]
        if s.startswith("//", i):
            j = s.find("\n", i)
            j = n if j < 0 else j
            fill(i, j); i = j
        elif s.startswith("/*", i):
            d, j = 1, i + 2
            while j < n and d:
                if s.startswith("/*", j): d += 1; j += 2
                elif s.startswith("*/", j): d -= 1; j += 2
                else: j += 1
            fill(i, j); i = j
        elif re.match(r'b?r(#*)"', s[i:i+40]) and (i == 0 or not (s[i-1].isalnum() or s[i-1] == '_')):
            m = re.match(r'b?r(#*)"', s[i:])
            hashes = m.group(1)
            j = s.find('"' + hashes, i + m.end())
            j = n if j < 0 else j + 1 + len(hashes)
            fill(i, j); i = j
        elif c == '"':
            j = i + 1
            while j < n and s[j] != '"':
                j += 2 if s[j] == "\\" else 1
            fill(i, j + 1); i = j + 1
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", s[i:i+12])
            if m:
                fill(i, i + m.end()); i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(o)


items = []
for p in sorted(root.joinpath("src").rglob("*.rs")):
    rel = str(p.relative_to(root))
    s = p.read_text()
    b = blank(s)
    line_of = [0] * (len(b) + 1)
    ln = 1
    for k, ch in enumerate(b):
        line_of[k] = ln
        if ch == "\n":
            ln += 1
    line_of[len(b)] = ln
    # matching braces
    match = {}
    st = []
    for k, ch in enumerate(b):
        if ch == "{": st.append(k)
        elif ch == "}" and st: match[st.pop()] = k
    for m in re.finditer(r"(?m)^[ \t]*((?:pub(?:\([^)]*\))? )?(?:const )?(?:async )?(?:unsafe )?(?:extern \"C\" )?fn (\w+)|impl\b[^{;]*)", b):
        # header runs to the first `{` or `;` at depth 0 of parens/brackets
        k, d = m.end(), 0
        while k < len(b):
            ch = b[k]
            if ch in "([<" and not (ch == "<" and b[k-1] == "-"): d += 1
            elif ch in ")]>" and not (ch == ">" and b[k-1] in "-="): d -= 1
            elif ch == ";" and d <= 0: k = -1; break
            elif ch == "{" and d <= 0: break
            k += 1
        if k < 0 or k >= len(b) or k not in match:
            continue
        head = re.sub(r"\s+", " ", s[m.start():k]).strip()
        kind = "fn" if m.group(2) else "impl"
        items.append({
            "file": rel, "kind": kind, "name": m.group(2) or head,
            "head": head[:200], "start": line_of[m.start()], "end": line_of[match[k]],
        })
json.dump(items, open(out, "w"), indent=0)
print(len(items), "items")
