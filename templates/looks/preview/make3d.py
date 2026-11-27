#!/usr/bin/env python3
"""Writes the preview scene's two glb models (README.md) from sources/:

    python3 make3d.py

  3d/assets/models/crate.glb   a 1.2 m crate, sources/crate.jpg on each face
  3d/assets/models/ground.glb  a 60 m meadow, sources/grass.jpg every 4 m: flat
                               where the scene stands, rolling beyond it, so a
                               look's facets, bands or paint show on the land;
                               10 x 10 cells (200 triangles), within every
                               look's prop budget, so no look's cook
                               simplifies it

The textures are embedded; the heights come from fixed sines, so a run writes
the same bytes. The other models are checked file.model TypeScript under
3d/content/models/.
"""
import json, math, os, struct

HERE = os.path.dirname(os.path.abspath(__file__))


def glb(path, pos, nrm, uv, idx, image):
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
    im = add(image)
    lo = [min(p[k] for p in pos) for k in range(3)]
    hi = [max(p[k] for p in pos) for k in range(3)]
    doc = {
        "asset": {"version": "2.0", "generator": "templates/looks/preview/make3d.py"},
        "buffers": [{"byteLength": 0}],
        "bufferViews": views,
        "accessors": [
            {"bufferView": pv, "componentType": 5126, "count": len(pos), "type": "VEC3", "min": lo, "max": hi},
            {"bufferView": nv, "componentType": 5126, "count": len(pos), "type": "VEC3"},
            {"bufferView": tv, "componentType": 5126, "count": len(pos), "type": "VEC2"},
            {"bufferView": iv, "componentType": 5125, "count": len(idx), "type": "SCALAR"},
        ],
        "images": [{"bufferView": im, "mimeType": "image/jpeg"}],
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
    with open(path, "wb") as o:
        o.write(out)


def crate(path, image, h=0.6):
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
    glb(path, pos, nrm, uv, idx, image)


def height(x, z):
    """Metres above y = 0: none within 5 m of the scene's middle (0, -1), rising
    over the next 7 m into swells of up to about 1.6 m."""
    r = math.hypot(x, z + 1)
    rise = min(1.0, max(0.0, (r - 5.0) / 7.0))
    rise = rise * rise * (3 - 2 * rise)
    swell = (0.55 * math.sin(0.31 * x + 0.7) * math.cos(0.27 * z - 0.4)
             + 0.3 * math.sin(0.63 * x - 0.52 * z + 1.9)
             + 0.15 * math.cos(1.1 * x + 0.9 * z))
    # Back towards the hills (z below -14) the land settles again, so the
    # far layer stands clear of it.
    settle = min(1.0, max(0.0, (z + 22.0) / 8.0))
    return rise * settle * (0.8 + 0.8 * swell)


def ground(path, image, size=60.0, every=4.0, cells=10):
    n = cells + 1
    pos, uv = [], []
    for j in range(n):
        for i in range(n):
            x, z = -size / 2 + size * i / cells, -size / 2 + size * j / cells
            pos.append([x, height(x, z), z])
            uv.append([x / every, z / every])
    idx = []
    for j in range(cells):
        for i in range(cells):
            a = j * n + i
            idx += [a, a + n, a + 1, a + 1, a + n, a + n + 1]
    # Smooth normals from the height's slope: the look decides on facets.
    e = 0.05
    nrm = []
    for x, _, z in pos:
        dx = (height(x + e, z) - height(x - e, z)) / (2 * e)
        dz = (height(x, z + e) - height(x, z - e)) / (2 * e)
        l = math.sqrt(dx * dx + 1 + dz * dz)
        nrm.append([-dx / l, 1 / l, -dz / l])
    glb(path, pos, nrm, uv, idx, image)


if __name__ == "__main__":
    models = os.path.join(HERE, "3d", "assets", "models")
    read = lambda name: open(os.path.join(HERE, "sources", name), "rb").read()
    crate(os.path.join(models, "crate.glb"), read("crate.jpg"))
    ground(os.path.join(models, "ground.glb"), read("grass.jpg"))
