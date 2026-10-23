#!/usr/bin/env python3
"""Writes the showcase's tree (CC0), byte for byte, for the `lod` station:

    python3 templates/fx-showcase/tools/make-tree.py templates/fx-showcase/assets

models/tree.glb: a tapering trunk (bark) under a canopy of three lumpy,
smooth-shaded blobs (leaves), about 2,300 triangles, two materials. The cook
simplifies it into its levels of detail (`model { lods 1 0.5 0.25 }`, the
default). Standard library only; the lumps come from a fixed hash.
"""
import json, math, os, struct, sys

out = sys.argv[1] if len(sys.argv) > 1 else "templates/fx-showcase/assets"


def lump(x, y, z):
    # A smooth, fixed displacement field over the unit sphere.
    return 1 + 0.12 * math.sin(3.1 * x + 1.7 * y) * math.cos(2.3 * z - 0.9 * y) + 0.06 * math.sin(7 * x + 5 * z)


def sphere(cx, cy, cz, r, rings, segs):
    pos, nrm, idx = [], [], []
    for i in range(rings + 1):
        v = math.pi * i / rings
        for j in range(segs + 1):
            u = 2 * math.pi * j / segs
            x, y, z = math.sin(v) * math.cos(u), math.cos(v), math.sin(v) * math.sin(u)
            k = lump(x, y, z)
            pos.append((cx + r * k * x, cy + r * k * y * 0.85, cz + r * k * z))
    for i in range(rings):
        for j in range(segs):
            a = i * (segs + 1) + j
            b = a + segs + 1
            idx += [a, a + 1, b, a + 1, b + 1, b]
    return pos, idx


def trunk(rings, segs):
    pos, idx = [], []
    for i in range(rings + 1):
        t = i / rings
        y = 2.2 * t
        r = 0.22 * (1 - 0.45 * t)
        for j in range(segs + 1):
            u = 2 * math.pi * j / segs
            pos.append((r * math.cos(u), y, r * math.sin(u)))
    for i in range(rings):
        for j in range(segs):
            a = i * (segs + 1) + j
            b = a + segs + 1
            idx += [a, b, a + 1, a + 1, b, b + 1]
    return pos, idx


def normals(pos, idx):
    n = [[0.0, 0.0, 0.0] for _ in pos]
    for t in range(0, len(idx), 3):
        a, b, c = (pos[idx[t + k]] for k in range(3))
        e1 = [b[i] - a[i] for i in range(3)]
        e2 = [c[i] - a[i] for i in range(3)]
        f = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]]
        for k in range(3):
            for i in range(3):
                n[idx[t + k]][i] += f[i]
    out = []
    for v in n:
        l = math.sqrt(sum(x * x for x in v)) or 1
        out.append(tuple(x / l for x in v))
    return out


def merge(parts):
    pos, idx = [], []
    for p, i in parts:
        base = len(pos)
        pos += p
        idx += [x + base for x in i]
    return pos, idx


bark = trunk(12, 18)
leaves = merge([sphere(0, 2.9, 0, 1.25, 22, 30), sphere(0.7, 2.4, 0.3, 0.8, 16, 22), sphere(-0.6, 2.5, -0.4, 0.85, 16, 22)])

bin_ = bytearray()
views, accessors, prims = [], [], []
for m, (p, i) in enumerate([bark, leaves]):
    n = normals(p, i)
    for data, kind, comp, fmt in ((p, "VEC3", 5126, "<3f"), (n, "VEC3", 5126, "<3f")):
        at = len(bin_)
        for v in data:
            bin_ += struct.pack(fmt, *v)
        views.append({"buffer": 0, "byteOffset": at, "byteLength": len(bin_) - at})
        acc = {"bufferView": len(views) - 1, "componentType": comp, "count": len(data), "type": kind}
        if data is p:
            acc["min"] = [min(v[k] for v in p) for k in range(3)]
            acc["max"] = [max(v[k] for v in p) for k in range(3)]
        accessors.append(acc)
    at = len(bin_)
    for x in i:
        bin_ += struct.pack("<I", x)
    views.append({"buffer": 0, "byteOffset": at, "byteLength": len(bin_) - at})
    accessors.append({"bufferView": len(views) - 1, "componentType": 5125, "count": len(i), "type": "SCALAR"})
    a = len(accessors)
    prims.append({"attributes": {"POSITION": a - 3, "NORMAL": a - 2}, "indices": a - 1, "material": m})

doc = {
    "asset": {"version": "2.0", "generator": "make-tree.py"},
    "buffers": [{"byteLength": len(bin_)}],
    "bufferViews": views,
    "accessors": accessors,
    "materials": [
        {"name": "bark", "pbrMetallicRoughness": {"baseColorFactor": [0.32, 0.2, 0.12, 1], "metallicFactor": 0, "roughnessFactor": 0.9}},
        {"name": "leaves", "pbrMetallicRoughness": {"baseColorFactor": [0.16, 0.42, 0.14, 1], "metallicFactor": 0, "roughnessFactor": 0.75}},
    ],
    "meshes": [{"name": "tree", "primitives": prims}],
    "nodes": [{"name": "tree", "mesh": 0}],
    "scenes": [{"nodes": [0]}],
    "scene": 0,
}
text = json.dumps(doc, separators=(",", ":")).encode()
text += b" " * (-len(text) % 4)
bin_ += b"\0" * (-len(bin_) % 4)
glb = b"glTF" + struct.pack("<II", 2, 12 + 8 + len(text) + 8 + len(bin_))
glb += struct.pack("<I", len(text)) + b"JSON" + text + struct.pack("<I", len(bin_)) + b"BIN\0" + bytes(bin_)
os.makedirs(os.path.join(out, "models"), exist_ok=True)
with open(os.path.join(out, "models", "tree.glb"), "wb") as f:
    f.write(glb)
tris = (len(bark[1]) + len(leaves[1])) // 3
print(f"models/tree.glb: {tris} triangles, {len(glb)} bytes")
