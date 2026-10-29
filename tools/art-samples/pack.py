#!/usr/bin/env python3
"""Packs a downloaded model into one glb for a sample scene (README.md):
the cinematic-3d family's realistic set (sets/realistic/) from the origin
asset catalogue (Poly Haven, CC0) and Meshy.

    pack.py <in.gltf|in.glb> <out.glb> <height-m> [--tex N] [--color-tex N] [--quality Q]
            [--drop arm|normal ...] [--image <uri>=<file> ...]

Every image is re-encoded as JPEG (PNG when it has alpha) at most N texels
on its long side (default 1024) at quality Q (default 82); `--drop` leaves
a map kind out (the occlusion-roughness-metal `arm` or the `normal` map,
named by the material slot that reads it). `--color-tex` sizes the base
colour maps on their own (the map ArtTexels.density measures);
`--image` reads a glTF's image <uri> from another file (the same asset's
larger map, from its publisher). A new root node scales the
model uniformly to `height-m` metres tall, stands its base on y = 0 and
centres it on x = z = 0; the cook conforms what it packs (og-cook README
§Art conformance). Needs ImageMagick (`magick`); no Python packages.
"""
import json, math, os, struct, subprocess, sys


def read(path):
    data = open(path, "rb").read()
    if data[:4] == b"glTF":
        jl = struct.unpack_from("<I", data, 12)[0]
        doc = json.loads(data[20:20 + jl])
        rest = data[20 + jl:]
        bins = [rest[8:8 + struct.unpack_from("<I", rest, 0)[0]]] if rest else [b""]
        return doc, bins, None
    doc = json.loads(data)
    base = os.path.dirname(path)
    bins = [open(os.path.join(base, b["uri"]), "rb").read() for b in doc["buffers"]]
    return doc, bins, base


def mat_mul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]


def node_matrix(n):
    if "matrix" in n:
        m = n["matrix"]
        return [[m[c * 4 + r] for c in range(4)] for r in range(4)]
    t = n.get("translation", [0, 0, 0])
    x, y, z, w = n.get("rotation", [0, 0, 0, 1])
    s = n.get("scale", [1, 1, 1])
    r = [[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
         [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
         [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]]
    return [[r[i][0] * s[0], r[i][1] * s[1], r[i][2] * s[2], t[i]] for i in range(3)] + [[0, 0, 0, 1]]


def bounds(doc):
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    scene = doc["scenes"][doc.get("scene", 0)]

    def walk(i, m):
        n = doc["nodes"][i]
        m = mat_mul(m, node_matrix(n))
        if "mesh" in n:
            for p in doc["meshes"][n["mesh"]]["primitives"]:
                a = doc["accessors"][p["attributes"]["POSITION"]]
                for cx in (a["min"][0], a["max"][0]):
                    for cy in (a["min"][1], a["max"][1]):
                        for cz in (a["min"][2], a["max"][2]):
                            v = [sum(m[r][k] * c for k, c in enumerate((cx, cy, cz, 1))) for r in range(3)]
                            for k in range(3):
                                lo[k], hi[k] = min(lo[k], v[k]), max(hi[k], v[k])
        for c in n.get("children", []):
            walk(c, m)

    for i in scene["nodes"]:
        walk(i, [[1 if r == c else 0 for c in range(4)] for r in range(4)])
    return lo, hi


def reencode(raw, size, quality):
    alpha = subprocess.run(["magick", "-", "-format", "%[opaque]", "info:"], input=raw, capture_output=True, check=True).stdout.strip() == b"False"
    fmt = "png:-" if alpha else "jpg:-"
    args = ["magick", "-", "-resize", f"{size}x{size}>", "-strip"] + (["-define", "png:compression-level=9"] if alpha else ["-quality", str(quality), "-sampling-factor", "4:2:0"]) + [fmt]
    return subprocess.run(args, input=raw, capture_output=True, check=True).stdout, "image/png" if alpha else "image/jpeg"


def pack(src, dst, height, size=1024, quality=82, drop=(), color=None, images=None):
    doc, bins, base = read(src)
    # Material slots whose maps are dropped.
    slots = {"arm": ("occlusionTexture", "metallicRoughnessTexture"), "normal": ("normalTexture",)}
    dropped = {k for d in drop for k in slots[d]}
    for m in doc.get("materials", []):
        for k in dropped:
            m.pop(k, None)
            m.get("pbrMetallicRoughness", {}).pop(k, None)
    used, colour = set(), set()
    for m in doc.get("materials", []):
        b = m.get("pbrMetallicRoughness", {}).get("baseColorTexture")
        if b:
            colour.add(doc["textures"][b["index"]]["source"])
        for k, v in list(m.items()) + list(m.get("pbrMetallicRoughness", {}).items()):
            if isinstance(v, dict) and "index" in v:
                used.add(doc["textures"][v["index"]]["source"])
    blob, views = b"", []

    def add(b):
        nonlocal blob
        while len(blob) % 4:
            blob += b"\0"
        views.append({"buffer": 0, "byteOffset": len(blob), "byteLength": len(b)})
        blob += b
        return len(views) - 1

    # Geometry views, copied as they are.
    remap = {}
    for i, v in enumerate(doc.get("bufferViews", [])):
        if any(img.get("bufferView") == i for img in doc.get("images", [])):
            continue
        b = bins[v.get("buffer", 0)][v.get("byteOffset", 0):v.get("byteOffset", 0) + v["byteLength"]]
        remap[i] = add(b)
        for k in ("byteStride", "target"):
            if k in v:
                views[-1][k] = v[k]
    for a in doc.get("accessors", []):
        if "bufferView" in a:
            a["bufferView"] = remap[a["bufferView"]]
    for i, img in enumerate(doc.get("images", [])):
        if i not in used:
            raw = b""
        elif "uri" in img:
            uri = img.pop("uri")
            raw = open((images or {}).get(uri) or os.path.join(base, uri), "rb").read()
        else:
            v = doc["bufferViews"][img["bufferView"]]
            raw = bins[v.get("buffer", 0)][v.get("byteOffset", 0):v.get("byteOffset", 0) + v["byteLength"]]
        if raw:
            raw, mime = reencode(raw, color if color and i in colour else size, quality)
        else:
            raw, mime = reencode(b"P3 1 1 255 128 128 128", 1, quality)
        img.pop("uri", None)
        img["bufferView"], img["mimeType"] = add(raw), mime
    doc["bufferViews"] = views
    lo, hi = bounds(doc)
    k = height / (hi[1] - lo[1])
    scene = doc["scenes"][doc.get("scene", 0)]
    doc["nodes"].append({"name": "pack", "children": scene["nodes"], "scale": [k] * 3,
                         "translation": [-k * (lo[0] + hi[0]) / 2, -k * lo[1], -k * (lo[2] + hi[2]) / 2]})
    scene["nodes"] = [len(doc["nodes"]) - 1]
    for key in ("extensionsUsed", "extensionsRequired"):
        doc.pop(key, None)
    doc["asset"]["generator"] = "tools/art-samples/pack.py"
    while len(blob) % 4:
        blob += b"\0"
    doc["buffers"] = [{"byteLength": len(blob)}]
    j = json.dumps(doc, separators=(",", ":")).encode()
    while len(j) % 4:
        j += b" "
    out = b"glTF" + struct.pack("<II", 2, 28 + len(j) + len(blob)) + struct.pack("<I", len(j)) + b"JSON" + j + struct.pack("<I", len(blob)) + b"BIN\0" + blob
    open(dst, "wb").write(out)
    print(f"{dst}: {len(out)} bytes, {height} m tall (x{k:.3f}), {(hi[0] - lo[0]) * k:.2f} x {(hi[2] - lo[2]) * k:.2f} m across")


if __name__ == "__main__":
    a = sys.argv[1:]
    opt = lambda f, d: a[a.index(f) + 1] if f in a else d
    drop = [a[i + 1] for i, x in enumerate(a) if x == "--drop"]
    images = dict(a[i + 1].split("=", 1) for i, x in enumerate(a) if x == "--image")
    color = int(opt("--color-tex", 0)) or None
    pack(a[0], a[1], float(a[2]), int(opt("--tex", 1024)), int(opt("--quality", 82)), drop, color, images)
