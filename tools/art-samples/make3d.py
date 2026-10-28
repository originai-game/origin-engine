#!/usr/bin/env python3
"""Writes a 3D sample scene's glTF assets (README.md): the crate (a 1.2 m box
with crate.png on every face) and the ground (a 40 m plane with ground.png
repeated every 4 m), so the cook conforms their textures as a model's
(og-cook README §Art conformance) and their meshes to ArtShape.

    make3d.py <assets-dir> <crate.png> <ground.png>

The textures are embedded in the glb (type model has no texture maps).
The other beats are `type model` text in assets/models/.
"""
import json, os, struct, sys


def glb(path, pos, nrm, uv, idx, png):
    blob, views = b"", []

    def add(b):
        nonlocal blob
        while len(blob) % 4:
            blob += b"\0"
        views.append({"buffer": 0, "byteOffset": len(blob), "byteLength": len(b)})
        blob += b
        return len(views) - 1

    f = lambda rows: b"".join(struct.pack("<%df" % len(r), *r) for r in rows)
    pv, nv, tv = add(f(pos)), add(f(nrm)), add(f(uv))
    iv = add(struct.pack("<%dI" % len(idx), *idx))
    im = add(png)
    lo = [min(p[k] for p in pos) for k in range(3)]
    hi = [max(p[k] for p in pos) for k in range(3)]
    doc = {
        "asset": {"version": "2.0", "generator": "tools/art-samples/make3d.py"},
        "buffers": [{"byteLength": 0}],
        "bufferViews": views,
        "accessors": [
            {"bufferView": pv, "componentType": 5126, "count": len(pos), "type": "VEC3", "min": lo, "max": hi},
            {"bufferView": nv, "componentType": 5126, "count": len(pos), "type": "VEC3"},
            {"bufferView": tv, "componentType": 5126, "count": len(pos), "type": "VEC2"},
            {"bufferView": iv, "componentType": 5125, "count": len(idx), "type": "SCALAR"},
        ],
        "images": [{"bufferView": im, "mimeType": "image/png" if png[:4] == b"\x89PNG" else "image/jpeg"}],
        "samplers": [{"wrapS": 10497, "wrapT": 10497}],
        "textures": [{"source": 0, "sampler": 0}],
        "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0, "roughnessFactor": 0.9}}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "nodes": [{"mesh": 0}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    }
    while len(blob) % 4:
        blob += b"\0"
    doc["buffers"][0]["byteLength"] = len(blob)
    j = json.dumps(doc).encode()
    while len(j) % 4:
        j += b" "
    out = b"glTF" + struct.pack("<II", 2, 28 + len(j) + len(blob)) + struct.pack("<I", len(j)) + b"JSON" + j + struct.pack("<I", len(blob)) + b"BIN\0" + blob
    open(path, "wb").write(out)


def crate(path, png, h=0.6):
    faces = [((1, 0, 0), (0, 0, -1), (0, 1, 0)), ((-1, 0, 0), (0, 0, 1), (0, 1, 0)), ((0, 1, 0), (1, 0, 0), (0, 0, -1)),
             ((0, -1, 0), (1, 0, 0), (0, 0, 1)), ((0, 0, 1), (1, 0, 0), (0, 1, 0)), ((0, 0, -1), (-1, 0, 0), (0, 1, 0))]
    pos, nrm, uv, idx = [], [], [], []
    for n, u, v in faces:
        base = len(pos)
        for (a, b), (s, t) in zip([(-1, -1), (1, -1), (1, 1), (-1, 1)], [(0, 1), (1, 1), (1, 0), (0, 0)]):
            pos.append([h * (n[k] + a * u[k] + b * v[k]) + (h if k == 1 else 0) for k in range(3)])
            nrm.append(list(n))
            uv.append([s, t])
        idx += [base, base + 1, base + 2, base, base + 2, base + 3]
    glb(path, pos, nrm, uv, idx, png)


def ground(path, png, size=40.0, repeat=10.0, cells=8):
    # A grid, not one quad, so a decimator or voxeliser has vertices to keep.
    pos, nrm, uv, idx = [], [], [], []
    for j in range(cells + 1):
        for i in range(cells + 1):
            x, z = -size / 2 + size * i / cells, -size / 2 + size * j / cells
            pos.append([x, 0.0, z])
            nrm.append([0.0, 1.0, 0.0])
            uv.append([repeat * i / cells, repeat * j / cells])
    for j in range(cells):
        for i in range(cells):
            a = j * (cells + 1) + i
            idx += [a, a + cells + 1, a + 1, a + 1, a + cells + 1, a + cells + 2]
    glb(path, pos, nrm, uv, idx, png)


if __name__ == "__main__":
    out, crate_png, ground_png = sys.argv[1:4]
    os.makedirs(os.path.join(out, "models"), exist_ok=True)
    crate(os.path.join(out, "models/crate.glb"), open(crate_png, "rb").read())
    ground(os.path.join(out, "models/ground.glb"), open(ground_png, "rb").read())
