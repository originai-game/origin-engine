#!/usr/bin/env python3
"""Writes a 3D sample scene's glTF assets (README.md): the crate (a 1.2 m box
with crate.png on every face) and the ground (a 40 m plane with ground.png
repeated every 4 m), so the cook conforms their textures as a model's
(og-cook README §Art conformance) and their meshes to ArtShape.

    make3d.py <assets-dir> <crate.png> <ground.png>
    make3d.py ground <out.glb> <color.jpg> [normal.jpg] [repeat-metres] [size-metres]
        a ground alone: the plane (40 m, or size-metres) with the colour map (and a normal
        map) repeated every <repeat-metres> (default 4): the
        cinematic-3d family's realistic set takes its Poly Haven ground
        at 1024 texels every 2 m, ArtTexels.density 512
    make3d.py ridge <out.glb>
        a far ridge of mountains (fractal noise, 480 x 80 m from 70 m
        behind where it is placed, peaks to about 55 m) in one untextured rock colour: the realistic set's far
        layer, hazed by the look's fog and atmosphere

The textures are embedded in the glb (type model has no texture maps).
The other beats are `type model` text in assets/models/.
"""
import json, os, struct, sys


def glb(path, pos, nrm, uv, idx, png, normal=None, color=None):
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
    im = add(png) if png else None
    nm = add(normal) if normal else None
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
        "images": [{"bufferView": v, "mimeType": "image/png" if b[:4] == b"\x89PNG" else "image/jpeg"} for v, b in ((im, png), (nm, normal)) if b],
        "samplers": [{"wrapS": 10497, "wrapT": 10497}],
        "textures": [{"source": i, "sampler": 0} for i in range(2 if normal else 1)],
        "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0, "roughnessFactor": 0.9}, **({"normalTexture": {"index": 1}} if normal else {})}
                      if png else {"pbrMetallicRoughness": {"baseColorFactor": color, "metallicFactor": 0, "roughnessFactor": 0.95}}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "nodes": [{"mesh": 0}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    }
    if not png:
        for k in ("images", "samplers", "textures"):
            doc.pop(k)
        doc["meshes"][0]["primitives"][0]["attributes"].pop("TEXCOORD_0")
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


def ground(path, png, size=40.0, repeat=10.0, cells=8, normal=None):
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
    glb(path, pos, nrm, uv, idx, png, normal)


def ridge(path, w=480.0, d=80.0, nx=128, nz=24, seed=7, near=70.0, peak=55.0):
    import math, random
    rnd = random.Random(seed)
    octaves = [(rnd.uniform(0, 6.28), rnd.uniform(0, 6.28), f, a) for f, a in ((1.5, 1.0), (3.1, 0.5), (6.7, 0.25), (13.0, 0.12), (27.0, 0.06))]

    def height(u, v):
        h = sum(a * (0.5 + 0.5 * math.sin(f * u * 6.28 + p) * math.cos(f * 0.7 * v * 6.28 + q)) for p, q, f, a in octaves)
        return peak * (h / 1.93) ** 1.6 * min(1.0, 3 * v) * min(1.0, 3 * (1 - v) + 0.4)

    pos = [[-w / 2 + w * i / nx, height(i / nx, j / nz), -near - d * j / nz] for j in range(nz + 1) for i in range(nx + 1)]
    idx = []
    for j in range(nz):
        for i in range(nx):
            a = j * (nx + 1) + i
            idx += [a, a + 1, a + nx + 1, a + 1, a + nx + 2, a + nx + 1]
    nrm = [[0.0, 0.0, 0.0] for _ in pos]
    for t in range(0, len(idx), 3):
        a, b, c = (pos[k] for k in idx[t:t + 3])
        u, v = [b[k] - a[k] for k in range(3)], [c[k] - a[k] for k in range(3)]
        n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
        for k in idx[t:t + 3]:
            nrm[k] = [nrm[k][i] + n[i] for i in range(3)]
    nrm = [[x / (sum(y * y for y in n) ** 0.5 or 1) for x in n] for n in nrm]
    nrm = [n if n[1] >= 0 else [-x for x in n] for n in nrm]
    uv = [[0.0, 0.0] for _ in pos]
    # One colour, no texture: nothing for ArtTexels to judge on a far form the haze carries.
    glb(path, pos, nrm, uv, idx, None, color=[0.105, 0.118, 0.098, 1.0])


if __name__ == "__main__" and sys.argv[1] == "ridge":
    ridge(sys.argv[2])
elif __name__ == "__main__" and sys.argv[1] == "ground":
    a = sys.argv[2:]
    normal = open(a[2], "rb").read() if len(a) > 2 and not a[2].replace(".", "").isdigit() else None
    nums = [float(x) for x in a[2:] if x.replace(".", "").isdigit()]
    every, size = (nums + [4.0, 40.0][len(nums):])[:2]
    ground(a[0], open(a[1], "rb").read(), size=size, repeat=size / every, cells=16, normal=normal)
elif __name__ == "__main__":
    out, crate_png, ground_png = sys.argv[1:4]
    os.makedirs(os.path.join(out, "models"), exist_ok=True)
    crate(os.path.join(out, "models/crate.glb"), open(crate_png, "rb").read())
    ground(os.path.join(out, "models/ground.glb"), open(ground_png, "rb").read())
