#!/usr/bin/env python3
"""The fx-showcase looks diorama's tree, crate and spheres as model assets,
so the cook conforms them (docs/art.md §2.3): `make.py <out> <look-slug|none>`
writes a project at <out> whose project.ogt names looks/main.ogt (extending
the preset) and whose main.ts places them as templates/fx-showcase/src/looks.ts
does, with its camera. The ground, wall, post and lantern stay runtime
geometry, as in the diorama (the cook never sees them)."""
import json, os, shutil, struct, sys

out, slug = sys.argv[1], sys.argv[2]
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../.."))
show = os.path.join(repo, "templates/fx-showcase/assets")
shutil.rmtree(out, ignore_errors=True)
for d in ["assets/models", "assets/textures", "assets/looks", "src"]:
    os.makedirs(os.path.join(out, d))
for t in ["tile.png", "crate.png"]:
    shutil.copy(os.path.join(show, "textures", t), os.path.join(out, "assets/textures", t))

def w(rel, text):
    with open(os.path.join(out, rel), "w") as f:
        f.write(text)

w("project.ogt", f"""type project
version 1
---
name "Art diorama"
id "art-diorama"
engine "0.6.6"
entry "src/main.ts"
profile "3d"
tick 60
seed 1
{'' if slug == 'none' else 'look "looks/main.ogt"'}
""")
w("assets/looks/main.ogt", "type look\nversion 1\n---\n" + ("" if slug == "none" else f'extends "{slug}"\n'))

# The tree: a trunk and three stepped leaf boxes, each turned 45 degrees more.
import math
parts = ['material "bark" {\n  color 0.35 0.22 0.12\n  roughness 0.9\n}\nmaterial "leaves" {\n  color 0.18 0.5 0.22\n  roughness 0.9\n}\n',
         'part "trunk" {\n  material "bark"\n  position 0 0.9 0\n  boxGeometry {\n    width 0.5\n    height 1.8\n    depth 0.5\n  }\n}\n']
for i in range(3):
    s = 2.6 - i * 0.7
    a = 0.785 * i / 2
    parts.append(f'part "leaves-{i}" {{\n  material "leaves"\n  position 0 {2.0 + i * 0.75} 0\n  quaternion 0 {math.sin(a):.6f} 0 {math.cos(a):.6f}\n  boxGeometry {{\n    width {s:.2f}\n    height 0.7\n    depth {s:.2f}\n  }}\n}}\n')
w("assets/models/tree.ogt", "type model\nversion 1\n---\n" + "".join(parts))

balls = [("red", (0.85, 0.12, 0.1), 0, 0.85), ("gold", (1, 0.78, 0.34), 1, 0.3), ("blue", (0.15, 0.35, 0.9), 0, 0.5),
         ("chrome", (0.95, 0.93, 0.9), 1, 0.05), ("green", (0.25, 0.7, 0.3), 0, 0.7)]
for name, c, metal, rough in balls:
    w(f"assets/models/ball-{name}.ogt", f"""type model
version 1
---
material "{name}" {{
  color {c[0]} {c[1]} {c[2]}
  metalness {metal}
  roughness {rough}
}}
part "ball" {{
  material "{name}"
  sphereGeometry {{
    radius 0.7
  }}
}}
""")

# The crate: a 1.2 m box with crate.png on every face, as a glb (type model has no texture maps).
h = 0.6
faces = [((1,0,0),(0,0,-1),(0,1,0)), ((-1,0,0),(0,0,1),(0,1,0)), ((0,1,0),(1,0,0),(0,0,-1)),
         ((0,-1,0),(1,0,0),(0,0,1)), ((0,0,1),(1,0,0),(0,1,0)), ((0,0,-1),(-1,0,0),(0,1,0))]
pos, nrm, uv, idx = [], [], [], []
for n, u, v in faces:
    base = len(pos)
    for (a, b), (s, t) in zip([(-1,-1),(1,-1),(1,1),(-1,1)], [(0,1),(1,1),(1,0),(0,0)]):
        pos.append([h * (n[k] + a * u[k] + b * v[k]) for k in range(3)])
        nrm.append(list(n)); uv.append([s, t])
    idx += [base, base + 1, base + 2, base, base + 2, base + 3]
png = open(os.path.join(show, "textures/crate.png"), "rb").read()
bin_ = b""; views = []
def add(b):
    global bin_
    while len(bin_) % 4: bin_ += b"\0"
    views.append({"buffer": 0, "byteOffset": len(bin_), "byteLength": len(b)}); bin_ += b
    return len(views) - 1
f = lambda rows: b"".join(struct.pack("<%df" % len(r), *r) for r in rows)
pv, nv, tv = add(f(pos)), add(f(nrm)), add(f(uv))
iv = add(struct.pack("<%dI" % len(idx), *idx)); im = add(png)
doc = {"asset": {"version": "2.0"}, "buffers": [{"byteLength": 0}], "bufferViews": views,
       "accessors": [{"bufferView": pv, "componentType": 5126, "count": 24, "type": "VEC3", "min": [-h]*3, "max": [h]*3},
                     {"bufferView": nv, "componentType": 5126, "count": 24, "type": "VEC3"},
                     {"bufferView": tv, "componentType": 5126, "count": 24, "type": "VEC2"},
                     {"bufferView": iv, "componentType": 5125, "count": 36, "type": "SCALAR"}],
       "images": [{"bufferView": im, "mimeType": "image/png"}], "textures": [{"source": 0}],
       "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0, "roughnessFactor": 0.8}}],
       "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
       "nodes": [{"mesh": 0}], "scenes": [{"nodes": [0]}], "scene": 0}
while len(bin_) % 4: bin_ += b"\0"
doc["buffers"][0]["byteLength"] = len(bin_)
j = json.dumps(doc).encode()
while len(j) % 4: j += b" "
glb = b"glTF" + struct.pack("<II", 2, 28 + len(j) + len(bin_)) + struct.pack("<I", len(j)) + b"JSON" + j + struct.pack("<I", len(bin_)) + b"BIN\0" + bin_
open(os.path.join(out, "assets/models/crate.glb"), "wb").write(glb)
shutil.copy(os.path.join(os.path.dirname(__file__), "main.ts"), os.path.join(out, "src/main.ts"))
