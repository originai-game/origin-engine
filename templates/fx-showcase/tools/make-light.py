#!/usr/bin/env python3
"""Regenerates the light station's self-made assets (CC0), byte for byte:

    python3 templates/fx-showcase/tools/make-light.py templates/fx-showcase/assets

- models/cornell.glb: a Cornell box, 2 m a side, from x -1..1, y 0..2,
  z -1..1, open towards +z: a white floor, ceiling and back wall, a red left
  wall, a green right wall (each a primitive of its own, facing inwards,
  matte), and a tall and a short white block on the floor (flat faces).
  The baked station's scene (scenes/cornell.ogt) marks it `static true`.
- models/window-room.glb: the volumetric station's room, 8 x 4 x 8 m with a
  window of four panes in its sun-facing (+x) wall and an open front (+z),
  all faces inwards and outwards so it casts the sun's shadow everywhere
  but through the window.
- textures/lattice.png: a 64 x 64 spot light cookie: a lattice of four by
  four panes (white) between dark bars.
- textures/lantern.png: a 64 x 384 point light cookie, six square faces
  stacked top to bottom (+X, -X, +Y, -Y, +Z, -Z), each a grid of round
  holes in a dark shade, warm on the sides and dimmer below.

Standard library only (struct, zlib, json); deterministic.
"""
import json, math, os, struct, sys, zlib


def png(path, w, h, pixel):
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w):
            raw.extend(pixel(x, y))
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(data)


def quad(prims, mat, corners, normal):
    p = prims.setdefault(mat, ([], [], []))
    base = len(p[0])
    for c in corners:
        p[0].append(c)
        p[1].append(normal)
    p[2].extend([base, base + 1, base + 2, base, base + 2, base + 3])


def norm(v):
    l = math.sqrt(sum(x * x for x in v))
    return tuple(x / l for x in v)


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def face(prims, mat, corners):
    """A quad whose normal follows its winding (counter-clockwise front)."""
    n = norm(cross(sub(corners[1], corners[0]), sub(corners[2], corners[0])))
    quad(prims, mat, corners, n)


def block(prims, mat, cx, cz, sx, sy, sz, angle):
    c, s = math.cos(angle), math.sin(angle)
    def at(x, y, z):
        return (cx + x * c + z * s, y, cz - x * s + z * c)
    hx, hz = sx / 2, sz / 2
    v = {(i, j, k): at(i * hx, j * sy, k * hz) for i in (-1, 1) for j in (0, 1) for k in (-1, 1)}
    face(prims, mat, [v[-1, 1, 1], v[1, 1, 1], v[1, 1, -1], v[-1, 1, -1]])  # top
    face(prims, mat, [v[-1, 0, 1], v[1, 0, 1], v[1, 1, 1], v[-1, 1, 1]])  # +z
    face(prims, mat, [v[1, 0, -1], v[-1, 0, -1], v[-1, 1, -1], v[1, 1, -1]])  # -z
    face(prims, mat, [v[1, 0, 1], v[1, 0, -1], v[1, 1, -1], v[1, 1, 1]])  # +x
    face(prims, mat, [v[-1, 0, -1], v[-1, 0, 1], v[-1, 1, 1], v[-1, 1, -1]])  # -x


def glb(path, prims, materials):
    buf = bytearray()
    views, accessors, primitives = [], [], []
    def add(data, target, comp, kind, count, mn=None, mx=None):
        while len(buf) % 4:
            buf.append(0)
        views.append({"buffer": 0, "byteOffset": len(buf), "byteLength": len(data), "target": target})
        buf.extend(data)
        a = {"bufferView": len(views) - 1, "componentType": comp, "count": count, "type": kind}
        if mn is not None:
            a["min"], a["max"] = mn, mx
        accessors.append(a)
        return len(accessors) - 1
    for mat in sorted(prims):
        pos, nrm, idx = prims[mat]
        mn = [min(p[k] for p in pos) for k in range(3)]
        mx = [max(p[k] for p in pos) for k in range(3)]
        a_pos = add(b"".join(struct.pack("<3f", *p) for p in pos), 34962, 5126, "VEC3", len(pos), mn, mx)
        a_nrm = add(b"".join(struct.pack("<3f", *n) for n in nrm), 34962, 5126, "VEC3", len(nrm))
        a_idx = add(b"".join(struct.pack("<I", i) for i in idx), 34963, 5125, "SCALAR", len(idx))
        primitives.append({"attributes": {"POSITION": a_pos, "NORMAL": a_nrm}, "indices": a_idx, "material": mat})
    doc = {
        "asset": {"version": "2.0", "generator": "fx-showcase tools/make-light.py"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": os.path.basename(path)[:-4]}],
        "meshes": [{"primitives": primitives}],
        "materials": materials,
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": len(buf)}],
    }
    js = json.dumps(doc, separators=(",", ":"), sort_keys=True).encode()
    while len(js) % 4:
        js += b" "
    while len(buf) % 4:
        buf.append(0)
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(buf))
    out += struct.pack("<II", len(js), 0x4E4F534A) + js
    out += struct.pack("<II", len(buf), 0x004E4942) + bytes(buf)
    with open(path, "wb") as f:
        f.write(out)


def matte(name, r, g, b, roughness=0.95):
    return {
        "name": name,
        "pbrMetallicRoughness": {"baseColorFactor": [r, g, b, 1.0], "metallicFactor": 0.0, "roughnessFactor": roughness},
    }


def cornell(path):
    WHITE, RED, GREEN, BLOCK = 0, 1, 2, 3
    p = {}
    # Inward-facing walls (counter-clockwise seen from inside).
    face(p, WHITE, [(-1, 0, 1), (1, 0, 1), (1, 0, -1), (-1, 0, -1)])  # floor, up
    face(p, WHITE, [(-1, 2, -1), (1, 2, -1), (1, 2, 1), (-1, 2, 1)])  # ceiling, down
    face(p, WHITE, [(-1, 0, -1), (1, 0, -1), (1, 2, -1), (-1, 2, -1)])  # back, +z
    face(p, RED, [(-1, 0, 1), (-1, 0, -1), (-1, 2, -1), (-1, 2, 1)])  # left, +x
    face(p, GREEN, [(1, 0, -1), (1, 0, 1), (1, 2, 1), (1, 2, -1)])  # right, -x
    block(p, BLOCK, -0.38, -0.3, 0.6, 1.2, 0.6, math.radians(18))
    block(p, BLOCK, 0.4, 0.35, 0.6, 0.6, 0.6, math.radians(-17))
    glb(path, p, [
        matte("white", 0.73, 0.73, 0.73),
        matte("red", 0.63, 0.065, 0.05),
        matte("green", 0.14, 0.45, 0.091),
        matte("block", 0.73, 0.73, 0.73),
    ])


def window_room(path):
    WALL, FLOOR, FRAME = 0, 1, 2
    p = {}
    W, H, D = 4.0, 4.0, 4.0  # half extents x, z; height
    def both(mat, corners):
        face(p, mat, corners)
        face(p, mat, corners[::-1])
    both(FLOOR, [(-W, 0, D), (W, 0, D), (W, 0, -D), (-W, 0, -D)])
    both(WALL, [(-W, H, -D), (W, H, -D), (W, H, D), (-W, H, D)])
    both(WALL, [(-W, 0, -D), (W, 0, -D), (W, H, -D), (-W, H, -D)])
    both(WALL, [(-W, 0, D), (-W, 0, -D), (-W, H, -D), (-W, H, D)])
    # The +x wall with a window from z -1.4..1.4, y 1.2..3.2, in panes.
    x = W
    zs, ys = (-D, -1.4, 1.4, D), (0, 1.2, 3.2, H)
    for i in range(3):
        for j in range(3):
            if i == 1 and j == 1:
                continue
            both(WALL, [(x, ys[j], zs[i + 1]), (x, ys[j], zs[i]), (x, ys[j + 1], zs[i]), (x, ys[j + 1], zs[i + 1])])
    # The window's bars: a cross, 0.12 m thick, as thin boxes.
    t = 0.06
    def bar(z0, z1, y0, y1):
        for (a, b) in ((x - t, x + t),):
            both(FRAME, [(b, y0, z1), (b, y0, z0), (b, y1, z0), (b, y1, z1)])
            both(FRAME, [(a, y0, z1), (a, y0, z0), (a, y1, z0), (a, y1, z1)])
            both(FRAME, [(a, y0, z0), (b, y0, z0), (b, y1, z0), (a, y1, z0)])
            both(FRAME, [(a, y0, z1), (b, y0, z1), (b, y1, z1), (a, y1, z1)])
            both(FRAME, [(a, y0, z0), (b, y0, z0), (b, y0, z1), (a, y0, z1)])
            both(FRAME, [(a, y1, z0), (b, y1, z0), (b, y1, z1), (a, y1, z1)])
    bar(-t, t, 1.2, 3.2)
    bar(-1.4, 1.4, 2.2 - t, 2.2 + t)
    glb(path, p, [
        matte("plaster", 0.62, 0.6, 0.56),
        matte("boards", 0.42, 0.3, 0.2, 0.35),
        matte("frame", 0.2, 0.16, 0.12),
    ])


def lattice(x, y):
    bar = (x % 16) < 3 or (y % 16) < 3
    return (30, 24, 20, 255) if bar else (255, 244, 220, 255)


def lantern(x, y):
    face, fy = divmod(y, 64)
    cx, cy = (x % 16) - 7.5, (fy % 16) - 7.5
    hole = cx * cx + cy * cy < 18
    if face == 3:  # -Y: below, dimmer
        return (120, 90, 50, 255) if hole else (10, 8, 6, 255)
    if face == 2:  # +Y: above, closed
        return (12, 10, 8, 255)
    return (255, 196, 120, 255) if hole else (16, 12, 8, 255)


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "assets"
    os.makedirs(os.path.join(root, "models"), exist_ok=True)
    cornell(os.path.join(root, "models", "cornell.glb"))
    window_room(os.path.join(root, "models", "window-room.glb"))
    png(os.path.join(root, "textures", "lattice.png"), 64, 64, lattice)
    png(os.path.join(root, "textures", "lantern.png"), 64, 384, lantern)


main()
