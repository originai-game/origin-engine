#!/usr/bin/env python3
"""Writes templates/game/assets/models/walker.glb: the engine's self-made
(CC0 1.0) rigged test model. A 13-node walker (hips, spine, head, two
three-joint legs, a three-joint left arm) skinned to one mesh of boxes, the
head with two morph targets, and six clips covering every sampler:
idle and walk LINEAR (walk keyed at 24 Hz, with footstep markers in
extras."og.events"), run CUBICSPLINE (10 Hz keys with tangents), wave STEP,
blink a LINEAR weights channel, turn a 120-degree LINEAR rotation. Deterministic: the same bytes every run.

    python3 crates/og-anim/tools/make_rig.py [out.glb]
"""
import json, math, struct, sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "templates/game/assets/models/walker.glb"

# name, parent, translation
NODES = [
    ("Rig", None, (0, 0, 0)),
    ("hips", 0, (0, 1.0, 0)),
    ("spine", 1, (0, 0.3, 0)),
    ("head", 2, (0, 0.4, 0)),
    ("thigh.L", 1, (0.15, 0, 0)),
    ("shin.L", 4, (0, -0.45, 0)),
    ("foot.L", 5, (0, -0.45, 0)),
    ("thigh.R", 1, (-0.15, 0, 0)),
    ("shin.R", 7, (0, -0.45, 0)),
    ("foot.R", 8, (0, -0.45, 0)),
    ("arm.L", 2, (0.25, 0.3, 0)),
    ("forearm.L", 10, (0.3, 0, 0)),
    ("hand.L", 11, (0.3, 0, 0)),
]
JOINTS = list(range(1, 13))  # skin joint j is node j + 1


def world(i):
    x, y, z = NODES[i][2]
    p = NODES[i][1]
    if p is None:
        return (x, y, z)
    px, py, pz = world(p)
    return (px + x, py + y, pz + z)


buf = bytearray()
views, accessors = [], []


def add(data, fmt, count, ctype, atype, minmax=None, target=None):
    while len(buf) % 4:
        buf.append(0)
    off = len(buf)
    raw = struct.pack("<%d%s" % (len(data), fmt), *data)
    buf.extend(raw)
    v = {"buffer": 0, "byteOffset": off, "byteLength": len(raw)}
    if target:
        v["target"] = target
    views.append(v)
    a = {"bufferView": len(views) - 1, "componentType": ctype, "count": count, "type": atype}
    if minmax:
        a["min"], a["max"] = minmax
    accessors.append(a)
    return len(accessors) - 1


# ── the mesh: one box per bone, rigid to the bone's joint; knee and elbow
# boxes blend half into the next joint at their lower end.
pos, nrm, jnt, wgt, idx = [], [], [], [], []
morph1, morph2 = [], []
FACES = [((1, 0, 0), (0, 1, 0), (0, 0, 1)), ((-1, 0, 0), (0, 0, 1), (0, 1, 0)),
         ((0, 1, 0), (0, 0, 1), (1, 0, 0)), ((0, -1, 0), (1, 0, 0), (0, 0, 1)),
         ((0, 0, 1), (1, 0, 0), (0, 1, 0)), ((0, 0, -1), (0, 1, 0), (1, 0, 0))]


def box(center, half, joint, blend_to=None, blend_below=None, head=False):
    for n, u, v in FACES:
        base = len(pos) // 3
        for su, sv in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            p = [center[k] + half[k] * (n[k] + su * u[k] + sv * v[k]) for k in range(3)]
            pos.extend(p)
            nrm.extend(n)
            if blend_to is not None and p[1] < blend_below:
                jnt.extend((joint, blend_to, 0, 0)); wgt.extend((0.5, 0.5, 0, 0))
            else:
                jnt.extend((joint, 0, 0, 0)); wgt.extend((1, 0, 0, 0))
            if head:
                morph1.extend((0, -0.5 * (p[1] - center[1]), 0))   # blink: squeeze vertically
                morph2.extend((0.3 * (p[0] - center[0]), 0, 0.3 * (p[2] - center[2])))  # squash
            else:
                morph1.extend((0, 0, 0)); morph2.extend((0, 0, 0))
        # winding: counter-clockwise seen from outside
        a, b, c, d = base, base + 1, base + 2, base + 3
        cross = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
        if sum(cross[k] * n[k] for k in range(3)) > 0:
            idx.extend((a, b, c, a, c, d))
        else:
            idx.extend((a, c, b, a, d, c))


def seg(node, child, r, blend=False):
    a, b = world(node), world(child)
    c = [(a[k] + b[k]) / 2 for k in range(3)]
    h = [max(abs(b[k] - a[k]) / 2, r) for k in range(3)]
    j = node - 1
    box(c, h, j, blend_to=(child - 1) if blend else None, blend_below=c[1] - h[1] * 0.5)


seg(1, 2, 0.12)
seg(2, 3, 0.1)
box(world(3), (0.12, 0.12, 0.12), 2, head=True)
for t, s, f in ((4, 5, 6), (7, 8, 9)):
    seg(t, s, 0.06, blend=True)
    seg(s, f, 0.05)
seg(10, 11, 0.05)
seg(11, 12, 0.04)
nv = len(pos) // 3
lo = [min(pos[k::3]) for k in range(3)]
hi = [max(pos[k::3]) for k in range(3)]
a_pos = add(pos, "f", nv, 5126, "VEC3", (lo, hi), 34962)
a_nrm = add(nrm, "f", nv, 5126, "VEC3", None, 34962)
a_jnt = add(jnt, "H", nv, 5123, "VEC4", None, 34962)
a_wgt = add(wgt, "f", nv, 5126, "VEC4", None, 34962)
a_idx = add(idx, "I", len(idx), 5125, "SCALAR", None, 34963)


def mm(v):
    return ([min(v[k::3]) for k in range(3)], [max(v[k::3]) for k in range(3)])


a_m1 = add(morph1, "f", nv, 5126, "VEC3", mm(morph1))
a_m2 = add(morph2, "f", nv, 5126, "VEC3", mm(morph2))

# inverse bind matrices: translation by minus the joint's rest position (column-major)
ibm = []
for n in JOINTS:
    x, y, z = world(n)
    ibm.extend((1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -x, -y, -z, 1))
a_ibm = add(ibm, "f", len(JOINTS), 5126, "MAT4")


def quat(axis, angle):
    s = math.sin(angle / 2)
    return (axis[0] * s, axis[1] * s, axis[2] * s, math.cos(angle / 2))


animations = []


def channel(samplers, channels, node, path, times, values, interp, lanes):
    ti = add(times, "f", len(times), 5126, "SCALAR", ([min(times)], [max(times)]))
    count = len(values) // lanes
    atype = {1: "SCALAR", 3: "VEC3", 4: "VEC4"}.get(lanes, "SCALAR")
    vi = add(values, "f", count if lanes in (1, 3, 4) else len(values), 5126, atype)
    samplers.append({"input": ti, "output": vi, "interpolation": interp})
    channels.append({"sampler": len(samplers) - 1, "target": {"node": node, "path": path}})


def clip(name, duration, rate, interp, chans, events=None):
    samplers, channels = [], []
    n = int(round(duration * rate)) + 1
    times = [i / rate for i in range(n)]
    for node, path, f in chans:
        vals = []
        for t in times:
            v = f(t)
            if interp == "CUBICSPLINE":
                # tangents: central differences (units per second)
                e = 1e-3
                d = [(a - b) / (2 * e) for a, b in zip(f(t + e), f(t - e))]
                vals.extend(d); vals.extend(v); vals.extend(d)
            else:
                vals.extend(v)
        lanes = len(f(0.0))
        channel(samplers, channels, node, path, times, vals, interp, lanes)
    a = {"name": name, "samplers": samplers, "channels": channels}
    if events:
        a["extras"] = {"og.events": [{"name": n, "time": t} for n, t in events]}
    animations.append(a)


TAU = 2 * math.pi


def gait(name, duration, rate, interp, swing, stride, events):
    ph = lambda t: t / duration * TAU
    clip(name, duration, rate, interp, [
        (1, "translation", lambda t: (0.0, 1.0 + 0.03 * math.cos(2 * ph(t)), stride * t / duration)),
        (1, "rotation", lambda t: quat((0, 1, 0), 0.1 * math.sin(ph(t)))),
        (4, "rotation", lambda t: quat((1, 0, 0), swing * math.sin(ph(t)))),
        (5, "rotation", lambda t: quat((1, 0, 0), 0.6 * swing * (1 + math.sin(ph(t) - 1.2)))),
        (7, "rotation", lambda t: quat((1, 0, 0), -swing * math.sin(ph(t)))),
        (8, "rotation", lambda t: quat((1, 0, 0), 0.6 * swing * (1 - math.sin(ph(t) - 1.2)))),
    ], events)


clip("idle", 2.0, 30, "LINEAR", [
    (2, "rotation", lambda t: quat((0, 0, 1), 0.05 * math.sin(t * TAU / 2))),
    (1, "translation", lambda t: (0.0, 1.0 - 0.01 * math.sin(t * TAU / 2), 0.0)),
])
gait("walk", 1.0, 24, "LINEAR", 0.5, 1.2, [("footL", 0.26), ("footR", 0.76)])
gait("run", 0.6, 10, "CUBICSPLINE", 0.9, 1.5, [("footL", 0.16), ("footR", 0.46)])
clip("wave", 1.0, 4, "STEP", [
    (10, "rotation", lambda t: quat((0, 0, 1), 1.2 * math.sin(t * TAU))),
], [("wave", 0.5)])
# turn: two LINEAR keys 120 degrees apart, so slerp and nlerp differ visibly.
samplers, channels = [], []
channel(samplers, channels, 1, "rotation", [0.0, 1.0],
        list(quat((0, 1, 0), 0.0)) + list(quat((0, 1, 0), 2 * math.pi / 3)), "LINEAR", 4)
animations.append({"name": "turn", "samplers": samplers, "channels": channels})
clip("blink", 0.5, 30, "LINEAR", [
    (13, "weights", lambda t: (math.sin(t / 0.5 * math.pi), 0.5 * t / 0.5)),
])

nodes = []
for i, (name, parent, t) in enumerate(NODES):
    n = {"name": name}
    if any(t):
        n["translation"] = list(t)
    kids = [j for j, nd in enumerate(NODES) if nd[1] == i]
    if i == 0:
        kids.append(len(NODES))
    if kids:
        n["children"] = kids
    nodes.append(n)
nodes.append({"name": "body", "mesh": 0, "skin": 0})

doc = {
    "asset": {"version": "2.0", "generator": "og-anim make_rig.py", "copyright": "CC0 1.0, Origin Engine authors"},
    "scene": 0,
    "scenes": [{"name": "walker", "nodes": [0]}],
    "nodes": nodes,
    "meshes": [{
        "name": "walker",
        "primitives": [{
            "attributes": {"POSITION": a_pos, "NORMAL": a_nrm, "JOINTS_0": a_jnt, "WEIGHTS_0": a_wgt},
            "indices": a_idx,
            "material": 0,
            "targets": [{"POSITION": a_m1}, {"POSITION": a_m2}],
        }],
        "weights": [0.0, 0.0],
        "extras": {"targetNames": ["blink", "squash"]},
    }],
    "materials": [{"name": "walker", "pbrMetallicRoughness": {"baseColorFactor": [0.8, 0.45, 0.2, 1], "metallicFactor": 0, "roughnessFactor": 0.7}}],
    "skins": [{"name": "walker", "joints": JOINTS, "inverseBindMatrices": a_ibm, "skeleton": 1}],
    "animations": animations,
    "buffers": [{"byteLength": len(buf)}],
    "bufferViews": views,
    "accessors": accessors,
}
js = json.dumps(doc, separators=(",", ":"), sort_keys=True).encode()
js += b" " * ((4 - len(js) % 4) % 4)
while len(buf) % 4:
    buf.append(0)
glb = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(buf))
glb += struct.pack("<II", len(js), 0x4E4F534A) + js
glb += struct.pack("<II", len(buf), 0x004E4942) + bytes(buf)
open(OUT, "wb").write(glb)
print(f"{OUT}: {len(glb)} bytes, {nv} vertices, {len(idx)//3} triangles, {len(animations)} clips")
