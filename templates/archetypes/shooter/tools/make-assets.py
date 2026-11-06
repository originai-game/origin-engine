#!/usr/bin/env python3
"""Regenerates Holdout's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/scenes/arena.ogt: the arena, from LAYOUT below: the floor's
  collider, the canyon walls round it with the three gates the drones and
  walkers come through, the cover (KayKit's Space Base Bits, CC0, scaled
  to the arena) with a box collider each, the lamp posts and the named
  points the game reads (spawn-<i>, gate-<i>, duel-<i>).
- assets/nav/arena.ogt: the walkers' and drones' navmesh, baked by og cook
  from that scene (its models and its colliders), bounded to the arena and
  the gates' pockets.
- assets/models/blaster.glb and drone.glb: the pulse rifle and the hover
  drone, faceted meshes of boxes, cylinders and an icosahedron in flat
  colours (the look's facets), forward along -Z (the rifle, as the view
  looks) and +Z (the drone, as og.NavAgent faces).
- assets/textures/floor.png: the landing field's 4 m deck panel, tiling;
  assets/textures/marks.png: the decal atlas (a scorched bullet hole, a
  blast scorch, hazard stripes and the gate's chevron).
- assets/audio/*.wav: the rifle's shot, the reload, the empty click, the
  drone's charge and bolt, the walker's swipe, the wave horn, the win and
  loss stingers and the arena's music loop; assets/audio/*.ogt: og sfx
  presets for the interface, the steps, the hits and the deaths.

Standard library only; deterministic.
"""
import json, math, os, struct, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_wav, tone, noise, mix, concat  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
RATE = 22050


def write(path, data):
    full = os.path.join(assets, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "wb" if isinstance(data, (bytes, bytearray)) else "w") as f:
        f.write(data)


# ── the arena ──────────────────────────────────────────────────────────────
#
# Metres, +x east, -z north. The play floor is 36 m square inside a ring of
# canyon blocks (KayKit's terrain block, 2 m, scaled 3: 6 m cubes); three
# gates (north, west, east) are 6 m notches in the ring the foes come
# through; the players start on the south pad. Keep src/world.ts's names
# (spawn-*, gate-*, duel-*) in step.

HALF = 18.0          # the floor's inner half width
WALL = 3.0           # the wall block's scale (6 m blocks)
RING = HALF + WALL   # the blocks' centres
GATES = [(0.0, -RING), (-RING, 0.0), (RING, 0.0)]

# KayKit models: (file, its own bounding box min y, the box (w, h, d) its collider takes at scale 1)
KIT = {
    "terrain-tall": (-1.0, (2.0, 2.0, 2.0)),
    "drill-structure": (-0.87, (1.15, 1.47, 1.15)),
    "cargo-a-stacked": (0.0, (1.0, 1.0, 1.0)),
    "cargo-b-stacked": (0.0, (1.0, 1.0, 1.0)),
    "structure-low": (0.0, (1.75, 1.0, 1.75)),
    "structure-tall": (0.0, (1.75, 2.0, 1.75)),
    "cargodepot-a": (0.0, (2.07, 1.12, 2.08)),
    "lander-a": (0.0, (1.3, 1.19, 1.3)),
    "lights": (0.0, (0.3, 1.0, 0.3)),
    "landingpad-large": (0.0, None),
    "solarpanel": (0.0, (0.9, 0.37, 0.45)),
    "rocks-b": (0.0, (1.7, 1.0, 1.6)),
    "rock-b": (0.0, None),
    "basemodule-a": (0.0, (2.25, 1.0, 2.18)),
    "basemodule-e": (0.0, (2.22, 1.65, 2.28)),
    "spacetruck-large": (-0.114, None),
    "containers-a": (0.0, None),
}

# (name, model, x, z, scale, yaw degrees[, scale y]): the cover and the dressing.
COVER = [
    ("core", "drill-structure", 0.0, 0.0, 2.3, 0.0),
    ("stack-nw", "cargo-a-stacked", -6.5, -6.0, 1.4, 18.0),
    ("stack-ne", "cargo-b-stacked", 6.0, -6.5, 1.4, -14.0),
    ("stack-sw", "cargo-b-stacked", -6.0, 6.5, 1.4, -8.0),
    ("stack-se", "cargo-a-stacked", 6.5, 6.0, 1.4, 27.0),
    ("low-n", "structure-low", 0.0, -11.5, 1.45, 0.0),
    ("low-w", "structure-low", -11.5, 0.0, 1.45, 90.0),
    ("low-e", "structure-low", 11.5, 0.0, 1.45, 90.0),
    ("tower-nw", "structure-tall", -12.5, -12.5, 1.55, 0.0),
    ("tower-ne", "structure-tall", 12.5, -12.5, 1.55, 90.0),
    ("depot", "cargodepot-a", 12.0, 11.5, 1.45, -90.0),
    ("lander", "lander-a", -12.0, 11.5, 1.7, 30.0),
    ("pad", "landingpad-large", 0.0, 13.0, 1.7, 0.0, 0.5),
    ("crate-s", "containers-a", 3.6, 15.6, 1.8, 10.0),
    ("solar-0", "solarpanel", -15.5, 3.8, 1.8, 90.0),
    ("solar-1", "solarpanel", -15.5, -3.8, 1.8, 90.0),
    ("rocks-0", "rocks-b", 15.2, -16.0, 1.6, 40.0),
    ("rocks-1", "rocks-b", -16.0, 16.0, 1.5, -20.0),
    ("rock-0", "rock-b", 8.5, -15.8, 1.8, 0.0),
    ("rock-1", "rock-b", -9.0, 15.9, 1.5, 70.0),
]
# Lamp posts beside each gate and at the pad: their lights are spawned by src/world.ts at LAMPS.
LAMPS = [(-4.6, -16.8), (4.6, -16.8), (-16.8, -4.6), (-16.8, 4.6), (16.8, -4.6), (16.8, 4.6), (-4.2, 16.4), (4.2, 16.4)]
# Beyond the ring: the outpost's skyline.
SKYLINE = [
    ("hab-s", "basemodule-a", 0.0, 30.0, 3.2, 180.0),
    ("hab-sw", "basemodule-e", -17.0, 31.0, 3.0, 160.0),
    ("tower-far-ne", "structure-tall", 22.0, -33.0, 3.4, 0.0),
    ("tower-far-w", "structure-tall", -33.0, -14.0, 3.0, 45.0),
    ("depot-far-e", "cargodepot-a", 33.0, 16.0, 3.0, 0.0),
    ("truck", "spacetruck-large", 27.0, 27.0, 9.0, 120.0),
    ("rocks-far-0", "rocks-b", -30.0, 26.0, 4.0, 10.0),
    ("rocks-far-1", "rocks-b", 31.0, -29.0, 4.5, 80.0),
]
SPAWNS = [(-1.6, 14.4), (1.6, 14.4)]
DUEL = [(0.0, 15.0), (0.0, -15.0), (-15.0, -9.0), (15.0, 9.0)]


def quat_y(deg):
    a = math.radians(deg) / 2
    return (0.0, math.sin(a), 0.0, math.cos(a))


def entity(name, x, y, z, yaw=0.0, scale=(1, 1, 1), model=None, body=""):
    q = quat_y(yaw)
    m = f'  model "models/kaykit/{model}.glb"\n' if model else ""
    return (f'entity "{name}" {{\n{m}  Transform {{\n    position {x:g} {y:g} {z:g}\n    quaternion {q[0]:g} {q[1]:.7g} {q[2]:g} {q[3]:.7g}\n'
            f'    scale {scale[0]:g} {scale[1]:g} {scale[2]:g}\n  }}\n{body}}}\n')


def solid(name, x, y, z, w, h, d, yaw=0.0):
    """A static box collider (its own entity, unscaled, so the box is in metres)."""
    return entity(name, x, y, z, yaw, body=f"  Collider {{\n    shape 0\n    size {w:g} {h:g} {d:g}\n  }}\n")


def placed(name, model, x, z, s, yaw, sy=None, collide=True):
    lo, box = KIT[model]
    sy = s if sy is None else s * sy
    out = entity(name, x, -lo * sy, z, yaw, (s, sy, s), model)
    if collide and box:
        w, h, d = box
        out += solid(f"{name}-solid", x, h * sy / 2, z, round(w * s, 3), round(h * sy, 3), round(d * s, 3), yaw)
    return out


def arena():
    parts = ['type scene\nversion 1\n---\n# Holdout\'s arena, written by tools/make-assets.py (edit LAYOUT there and\n'
             '# run it again): the floor, the canyon ring and its three gates, the cover\n'
             '# (KayKit Space Base Bits, CC0) with a box collider each, the lamp posts,\n'
             '# the skyline beyond the ring and the points src/world.ts reads by name.\n'
             '# assets/nav/arena.ogt bakes the navmesh from it.\nname "Arena"\n']
    parts.append(solid("floor", 0, -0.5, 0, 90, 1, 90))
    n = 0
    k = int(RING / (2 * WALL))
    ring = []
    for i in range(-k, k + 1):
        c = i * 2 * WALL
        ring += [(c, -RING), (c, RING), (-RING, c), (RING, c)]
    seen = set()
    for x, z in ring:
        if (x, z) in seen or (x, z) in GATES:
            continue
        seen.add((x, z))
        yaw = ((int(x * 7 + z * 13) % 4) + 4) % 4 * 90.0
        parts.append(placed(f"wall-{n}", "terrain-tall", x, z, WALL, yaw))
        n += 1
    # Each gate's pocket: a block behind it closes the notch 6 m deep.
    for i, (gx, gz) in enumerate(GATES):
        bx, bz = gx + math.copysign(2 * WALL, gx) if gx else gx, gz + math.copysign(2 * WALL, gz) if gz else gz
        parts.append(placed(f"gate-back-{i}", "terrain-tall", bx, bz, WALL, 0.0))
        # Where the foes appear: the pocket's middle, just inside the ring's line.
        px = gx - math.copysign(1.0, gx) if gx else 0.0
        pz = gz - math.copysign(1.0, gz) if gz else 0.0
        parts.append(entity(f"gate-{i}", px, 0, pz))
    for row in COVER:
        name, model, x, z, s, yaw, *rest = row
        parts.append(placed(name, model, x, z, s, yaw, rest[0] if rest else None))
    for i, (x, z) in enumerate(LAMPS):
        parts.append(placed(f"lamp-{i}", "lights", x, z, 2.2, 0.0))
    for name, model, x, z, s, yaw in SKYLINE:
        parts.append(placed(name, model, x, z, s, yaw, collide=False))
    for i, (x, z) in enumerate(SPAWNS):
        parts.append(entity(f"spawn-{i}", x, 0, z))
    for i, (x, z) in enumerate(DUEL):
        parts.append(entity(f"duel-{i}", x, 0, z))
    write("scenes/arena.ogt", "".join(parts))
    b = RING + WALL + 0.5
    write("nav/arena.ogt", f"""type navmesh
version 1
---
# The foes' navmesh, baked by `og cook` (og.nav.load("nav/arena.ogt")) from
# the arena's scene: its models and its box colliders, for a walker 0.45 m
# wide and 1.9 m tall (the drones fly the same mesh at their hover height).
# The bounds keep the floor inside the canyon ring and the gates' pockets.
# Written by tools/make-assets.py.
agent {{
  radius 0.45
  height 1.9
  climb 0.4
  slope 40
}}
cellSize 0.2
cellHeight 0.1
tileSize 48
scene "scenes/arena.ogt"
bounds {{
  min {-b:g} -0.6 {-b:g}
  max {b:g} 7 {b:g}
}}
""")


# ── meshes ─────────────────────────────────────────────────────────────────

class Mesh:
    """Flat-shaded triangles grouped by material, written as one glTF node."""

    def __init__(self):
        self.parts = {}

    def tri(self, mat, a, b, c):
        self.parts.setdefault(mat, []).append((a, b, c))

    def quad(self, mat, a, b, c, d):
        self.tri(mat, a, b, c)
        self.tri(mat, a, c, d)

    def box(self, mat, cx, cy, cz, w, h, d, rx=0.0):
        """An axis box, tipped rx radians about x round its centre."""
        x0, x1, y0, y1, z0, z1 = -w / 2, w / 2, -h / 2, h / 2, -d / 2, d / 2
        c, s = math.cos(rx), math.sin(rx)

        def p(x, y, z):
            return (cx + x, cy + y * c - z * s, cz + y * s + z * c)

        v = [p(x0, y0, z0), p(x1, y0, z0), p(x1, y1, z0), p(x0, y1, z0), p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)]
        for a, b_, c_, d_ in ((4, 5, 6, 7), (1, 0, 3, 2), (5, 1, 2, 6), (0, 4, 7, 3), (7, 6, 2, 3), (0, 1, 5, 4)):
            self.quad(mat, v[a], v[b_], v[c_], v[d_])

    def cylinder(self, mat, cx, cy, cz, r, length, axis="z", n=8, r2=None):
        """A prism of n sides along an axis, centred; r2 tapers the far end."""
        r2 = r if r2 is None else r2
        ring0, ring1 = [], []
        for i in range(n):
            a = 2 * math.pi * (i + 0.5) / n
            u, w = math.cos(a), math.sin(a)
            if axis == "z":
                ring0.append((cx + u * r, cy + w * r, cz + length / 2))
                ring1.append((cx + u * r2, cy + w * r2, cz - length / 2))
            elif axis == "y":
                ring0.append((cx + u * r, cy - length / 2, cz + w * r))
                ring1.append((cx + u * r2, cy + length / 2, cz + w * r2))
            else:
                ring0.append((cx - length / 2, cy + u * r, cz + w * r))
                ring1.append((cx + length / 2, cy + u * r2, cz + w * r2))
        c0 = tuple(sum(p[k] for p in ring0) / n for k in range(3))
        c1 = tuple(sum(p[k] for p in ring1) / n for k in range(3))
        for i in range(n):
            j = (i + 1) % n
            self.quad(mat, ring0[i], ring0[j], ring1[j], ring1[i])
            self.tri(mat, c0, ring0[j], ring0[i])
            self.tri(mat, c1, ring1[i], ring1[j])

    def ico(self, mat, cx, cy, cz, r, sy=1.0):
        t = (1 + 5 ** 0.5) / 2
        v = [(-1, t, 0), (1, t, 0), (-1, -t, 0), (1, -t, 0), (0, -1, t), (0, 1, t), (0, -1, -t), (0, 1, -t), (t, 0, -1), (t, 0, 1), (-t, 0, -1), (-t, 0, 1)]
        k = r / math.sqrt(1 + t * t)
        v = [(cx + x * k, cy + y * k * sy, cz + z * k) for x, y, z in v]
        f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
             (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)]
        for a, b, c in f:
            self.tri(mat, v[a], v[b], v[c])


def write_mesh_glb(path, mesh, materials):
    """materials: name -> (base rgb, metalness, roughness, emissive rgb or None, strength)."""
    bin_ = bytearray()
    views, accessors, prims = [], [], []
    names = [m for m in materials if m in mesh.parts]
    mats = []
    for mi, name in enumerate(names):
        base, metal, rough, emit, strength = materials[name]
        m = {"name": name, "pbrMetallicRoughness": {"baseColorFactor": [*base, 1], "metallicFactor": metal, "roughnessFactor": rough}}
        if emit:
            m["emissiveFactor"] = list(emit)
            m["extensions"] = {"KHR_materials_emissive_strength": {"emissiveStrength": strength}}
        mats.append(m)
        pos, nrm = [], []
        for a, b, c in mesh.parts[name]:
            ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
            vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
            nx, ny, nz = uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx
            ln = math.sqrt(nx * nx + ny * ny + nz * nz) or 1
            for p in (a, b, c):
                pos.append(p)
                nrm.append((nx / ln, ny / ln, nz / ln))
        for data, kind in ((pos, "p"), (nrm, "n")):
            while len(bin_) % 4:
                bin_.append(0)
            off = len(bin_)
            for p in data:
                bin_ += struct.pack("<3f", *[round(x, 5) for x in p])
            views.append({"buffer": 0, "byteOffset": off, "byteLength": len(data) * 12, "target": 34962})
            acc = {"bufferView": len(views) - 1, "componentType": 5126, "count": len(data), "type": "VEC3"}
            if kind == "p":
                acc["min"] = [min(p[k] for p in data) for k in range(3)]
                acc["max"] = [max(p[k] for p in data) for k in range(3)]
            accessors.append(acc)
        prims.append({"attributes": {"POSITION": len(accessors) - 2, "NORMAL": len(accessors) - 1}, "material": mi})
    while len(bin_) % 4:
        bin_.append(0)
    doc = {
        "asset": {"version": "2.0", "generator": "holdout make-assets.py"},
        "extensionsUsed": ["KHR_materials_emissive_strength"],
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"name": os.path.basename(path)[:-4], "mesh": 0}],
        "meshes": [{"primitives": prims}], "materials": mats,
        "accessors": accessors, "bufferViews": views, "buffers": [{"byteLength": len(bin_)}],
    }
    js = json.dumps(doc, separators=(",", ":")).encode()
    while len(js) % 4:
        js += b" "
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(bin_))
    out += struct.pack("<II", len(js), 0x4E4F534A) + js + struct.pack("<II", len(bin_), 0x004E4942) + bytes(bin_)
    write(path, out)


MATS = {
    "gunmetal": ((0.16, 0.17, 0.19), 0.7, 0.45, None, 0),
    "plate": ((0.62, 0.64, 0.66), 0.3, 0.5, None, 0),
    "grip": ((0.07, 0.07, 0.08), 0.0, 0.85, None, 0),
    "accent": ((0.95, 0.42, 0.08), 0.1, 0.5, None, 0),
    "glow": ((0.3, 0.9, 1.0), 0.0, 0.4, (0.3, 0.9, 1.0), 4.0),
    "eye": ((1.0, 0.18, 0.08), 0.0, 0.4, (1.0, 0.18, 0.06), 6.0),
}


def blaster():
    """The pulse rifle: grip at the origin, the muzzle 0.62 m along -Z."""
    m = Mesh()
    m.box("gunmetal", 0, 0.03, -0.1, 0.07, 0.1, 0.36)          # receiver
    m.box("plate", 0, 0.085, -0.12, 0.075, 0.02, 0.3)         # top plate
    m.box("glow", 0.039, 0.03, -0.14, 0.004, 0.022, 0.2)      # charge strip, right
    m.box("glow", -0.039, 0.03, -0.14, 0.004, 0.022, 0.2)     # and left
    m.box("plate", 0, 0.04, -0.33, 0.06, 0.07, 0.16)          # shroud
    m.cylinder("gunmetal", 0, 0.04, -0.52, 0.016, 0.22)       # barrel
    m.cylinder("accent", 0, 0.04, -0.615, 0.024, 0.03)        # muzzle ring
    m.box("grip", 0, -0.06, 0.01, 0.035, 0.11, 0.045, 0.25)   # grip
    m.box("accent", 0, -0.075, -0.13, 0.04, 0.13, 0.06, -0.2)  # magazine
    m.box("gunmetal", 0, 0.0, 0.17, 0.05, 0.08, 0.2)          # stock
    m.box("grip", 0, -0.02, 0.27, 0.055, 0.12, 0.03)          # butt pad
    m.box("gunmetal", 0, 0.115, -0.08, 0.03, 0.04, 0.06)      # sight
    m.box("glow", 0, 0.138, -0.08, 0.012, 0.008, 0.012)       # sight dot
    write_mesh_glb("models/blaster.glb", m, MATS)


def drone():
    """The hover drone: a faceted body, a red eye looking +Z, four rotor pods."""
    m = Mesh()
    m.ico("gunmetal", 0, 0, 0, 0.42, 0.62)
    m.box("plate", 0, 0.2, -0.02, 0.36, 0.08, 0.42)             # back plate
    m.cylinder("accent", 0, 0.0, 0.32, 0.15, 0.12, n=10)       # eye housing
    m.cylinder("eye", 0, 0.0, 0.39, 0.1, 0.03, n=10)           # eye
    for sx in (-1, 1):
        for sz in (-1, 1):
            x, z = 0.42 * sx, 0.36 * sz
            m.box("gunmetal", x / 2, 0.06, z / 2, abs(x) + 0.04, 0.04, 0.05)
            m.cylinder("plate", x, 0.08, z, 0.15, 0.04, axis="y", n=10)
            m.cylinder("glow", x, 0.11, z, 0.05, 0.02, axis="y", n=6)
    m.box("accent", 0, -0.27, 0, 0.05, 0.18, 0.18)              # fin
    write_mesh_glb("models/drone.glb", m, MATS)


# ── textures ───────────────────────────────────────────────────────────────

def floor_texture():
    """A 4 m deck panel (256 px): dusty plates, seams, bolts and a scuffed lane mark."""
    c = Canvas(256, 256)
    c.rect(0, 0, 256, 256, "#6f655a")
    for i, (x, y, w, h, col) in enumerate([(4, 4, 122, 122, "#7a6f62"), (130, 4, 122, 122, "#736859"), (4, 130, 122, 122, "#776b5d"), (130, 130, 122, 122, "#7d7264")]):
        c.rect(x, y, w, h, col)
        c.rect(x + 6, y + 6, w - 12, 3, "#8a7f70")
        for bx, by in ((x + 8, y + 8), (x + w - 8, y + 8), (x + 8, y + h - 8), (x + w - 8, y + h - 8)):
            c.circle(bx, by, 3, "#4d463e")
    c.rect(124, 0, 8, 256, "#4a433b")
    c.rect(0, 124, 256, 8, "#4a433b")
    for k in range(7):
        c.rect(20 + k * 14, 200, 8, 26, "#b08c3c", alpha=0.35)
    write("textures/floor.png", png_bytes(256, 256, c.rgba()))


def clip(pts, x0, y0, x1, y1):
    """A convex polygon clipped to a rectangle (Sutherland-Hodgman)."""
    for axis, lim, keep in ((0, x0, 1), (0, x1, -1), (1, y0, 1), (1, y1, -1)):
        out = []
        for i, a in enumerate(pts):
            b = pts[(i + 1) % len(pts)]
            ina, inb = (a[axis] - lim) * keep >= 0, (b[axis] - lim) * keep >= 0
            if ina:
                out.append(a)
            if ina != inb:
                t = (lim - a[axis]) / (b[axis] - a[axis])
                out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))
        pts = out
        if not pts:
            break
    return pts


def marks_texture():
    """The decal atlas, four 128 px cells: a bullet hole, a scorch, hazard stripes, the gate's chevron."""
    c = Canvas(256, 256)
    c.circle(64, 64, 34, "#2a2520", alpha=0.55)
    c.circle(64, 64, 18, "#15110e", alpha=0.9)
    c.circle(64, 64, 8, "#050403")
    c.circle(192, 64, 56, "#1d1814", alpha=0.45)
    c.circle(192, 64, 38, "#120e0b", alpha=0.6)
    c.circle(192, 64, 20, "#0a0806", alpha=0.75)
    c.rect(4, 132, 120, 120, "#e8b21a")
    for k in range(-4, 8):
        stripe = clip([(4 + k * 30, 252), (24 + k * 30, 252), (24 + k * 30 + 120, 132), (4 + k * 30 + 120, 132)], 4, 132, 124, 252)
        if len(stripe) >= 3:
            c.poly(stripe, "#1a1714")
    c.poly([(150, 160), (192, 200), (234, 160), (234, 186), (192, 226), (150, 186)], "#ff5a1f")
    write("textures/marks.png", png_bytes(256, 256, c.rgba()))


# ── sounds ─────────────────────────────────────────────────────────────────

def lowpass(s, k):
    out, y = [], 0.0
    for x in s:
        y += k * (x - y)
        out.append(y)
    return out


def shot():
    """The pulse rifle: a sharp crack, a low thump and a short zap tail."""
    crack = noise(0.12, decay=0.02, gain=0.75, seed=3)
    body = tone(0.18, 160, 55, shape="sine", decay=0.05, gain=0.7)
    zap = tone(0.14, 1400, 380, shape="square", decay=0.035, gain=0.16)
    tail = lowpass(noise(0.35, decay=0.1, gain=0.3, seed=9), 0.12)
    return mix(crack, body, zap, tail)


def music():
    """Sixteen bars at 112 BPM in D minor: a pulsing bass, a hat on the off-beats and a cold pad, looped."""
    beat = 60 / 112
    total = int(64 * beat * RATE)
    out = [0.0] * total
    roots = [73.42, 73.42, 87.31, 65.41]  # D2, D2, F2, C2: four bars each
    for b in range(64):
        f = roots[(b // 4) % 4]
        for e in range(2):
            s0 = int((b + e * 0.5) * beat * RATE)
            for i in range(int(0.45 * beat * RATE)):
                t = i / RATE
                ph = 2 * math.pi * f * t
                saw = 2 * ((f * t) % 1) - 1
                out[(s0 + i) % total] += 0.11 * math.exp(-t / 0.12) * (0.6 * math.sin(ph) + 0.4 * saw)
        s0 = int((b + 0.5) * beat * RATE)
        x = 0x9E3779B1 ^ b
        for i in range(int(0.05 * RATE)):
            x ^= (x << 13) & 0xFFFFFFFF; x ^= x >> 17; x ^= (x << 5) & 0xFFFFFFFF
            out[(s0 + i) % total] += 0.035 * ((x / 0xFFFFFFFF) * 2 - 1) * math.exp(-(i / RATE) / 0.012)
    chords = [(293.66, 349.23, 440.0), (293.66, 349.23, 440.0), (349.23, 440.0, 523.25), (261.63, 329.63, 392.0)]
    for i in range(total):
        t = i / RATE
        bar = int(t / (4 * beat)) % 4
        swell = 0.5 + 0.5 * math.sin(2 * math.pi * t / (8 * beat))
        out[i] += 0.022 * swell * sum(math.sin(2 * math.pi * f * t) for f in chords[bar])
    return out


def sounds():
    w = lambda name, s: write_wav(os.path.join(assets, "audio", name), s, RATE)  # noqa: E731
    os.makedirs(os.path.join(assets, "audio"), exist_ok=True)
    w("shot.wav", shot())
    w("reload.wav", concat(mix(noise(0.05, decay=0.01, gain=0.5, seed=21), tone(0.05, 900, 600, shape="square", decay=0.015, gain=0.2)),
                           [0.0] * int(0.32 * RATE),
                           mix(noise(0.07, decay=0.015, gain=0.6, seed=22), tone(0.09, 520, 260, shape="square", decay=0.03, gain=0.25)),
                           [0.0] * int(0.18 * RATE),
                           tone(0.16, 600, 1300, shape="tri", decay=0.08, gain=0.3)))
    w("empty.wav", mix(noise(0.04, decay=0.008, gain=0.45, seed=31), tone(0.04, 1800, 1500, shape="square", decay=0.01, gain=0.12)))
    w("charge.wav", tone(0.7, 220, 880, shape="square", attack=0.05, decay=0.9, gain=0.14))
    w("bolt.wav", mix(tone(0.3, 900, 120, shape="square", decay=0.09, gain=0.28), noise(0.2, decay=0.05, gain=0.25, seed=41)))
    w("swipe.wav", mix(lowpass(noise(0.3, decay=0.09, gain=0.6, seed=51), 0.25), tone(0.25, 140, 70, shape="square", decay=0.08, gain=0.2)))
    w("horn.wav", mix(tone(1.1, 196, 196, shape="square", attack=0.08, decay=0.7, gain=0.16), tone(1.1, 293.66, 293.66, shape="square", attack=0.1, decay=0.7, gain=0.12)))
    w("win.wav", concat(*[tone(0.15, f, shape="tri", gain=0.4, decay=0.13) for f in (392.0, 493.88, 587.33, 783.99)], tone(1.0, 987.77, shape="tri", gain=0.4, decay=0.6)))
    w("lose.wav", concat(*[tone(0.28, f, shape="square", gain=0.22, decay=0.2) for f in (293.66, 261.63, 220.0)], tone(1.1, 146.83, shape="square", gain=0.22, decay=0.6)))
    w("arena.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup",
               "step": "footstep", "jump": "jump", "land": "land", "hit": "hit", "hurt": "hurt", "pop": "explosion", "down": "death", "clear": "powerup"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


def main():
    arena()
    blaster()
    drone()
    floor_texture()
    marks_texture()
    sounds()


if __name__ == "__main__":
    main()
