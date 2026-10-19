#!/usr/bin/env python3
"""Writes the retargeting assets of templates/fx-showcase (self-made, CC0 1.0):

  assets/anims/humanoid-clips.glb  an animation library: a 20-node humanoid
                                   skeleton with plain Blender names (hips,
                                   spine, chest, neck, head, shoulder.L,
                                   upper_arm.L, forearm.L, hand.L, thigh.L,
                                   shin.L, foot.L, toe.L and the right side),
                                   T-posed, identity rest rotations, legs
                                   0.45 + 0.45 m, and three clips (walk with
                                   footstep markers, idle, run); no mesh.
  assets/models/strider.glb        a tall character with Mixamo names
                                   (mixamorig:Hips, ...LeftUpLeg, ...), an
                                   Armature node scaled by 0.01, each bone's
                                   +Y along the bone, arms in an A-pose, a
                                   thigh of 0.70 and a shin of 0.44 m; no clips.
  assets/models/puppet2d.glb       a side-on 2D figure for the 2D ragdoll: every
                                   bone in the xy plane, near limbs light and far
                                   ones dark, a walk in place turning about z.
  assets/models/stout.glb          a short wide character with Unreal
                                   mannequin names (pelvis, thigh_l, calf_l,
                                   ...), each bone's +X along the bone, a thigh
                                   of 0.30 and a shin of 0.32 m; no clips.

The two characters are skinned boxes, one per bone. Deterministic: the same
bytes every run.

    python3 crates/og-anim/tools/make_humanoids.py [templates/fx-showcase]
"""
import json, math, os, struct, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "templates/fx-showcase"


# ── quaternions (x, y, z, w) ─────────────────────────────────────────────

def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return (aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz)


def qconj(q):
    return (-q[0], -q[1], -q[2], q[3])


def qrot(q, v):
    p = qmul(qmul(q, (v[0], v[1], v[2], 0.0)), qconj(q))
    return (p[0], p[1], p[2])


def qaxis(axis, angle):
    s = math.sin(angle / 2)
    return (axis[0] * s, axis[1] * s, axis[2] * s, math.cos(angle / 2))


def norm(v):
    l = math.sqrt(sum(c * c for c in v))
    return tuple(c / l for c in v)


def qarc(a, b):
    a, b = norm(a), norm(b)
    d = sum(x * y for x, y in zip(a, b))
    if d < -0.999999:
        axis = norm((0, 0, 1) if abs(a[2]) < 0.9 else (1, 0, 0))
        c = (a[1] * axis[2] - a[2] * axis[1], a[2] * axis[0] - a[0] * axis[2], a[0] * axis[1] - a[1] * axis[0])
        c = norm(c)
        return (c[0], c[1], c[2], 0.0)
    c = (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
    q = (c[0], c[1], c[2], 1 + d)
    l = math.sqrt(sum(x * x for x in q))
    return tuple(x / l for x in q)


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def scale(a, s):
    return tuple(x * s for x in a)


def r6(v):
    """Rounds to 6 decimals (stable bytes, exact zeros)."""
    return [0.0 if abs(x) < 5e-7 else round(x, 6) for x in v]


# ── a rig: nodes with model-space rest positions and bone frames ──────────

class Rig:
    def __init__(self, root_scale):
        self.names, self.parents, self.pos, self.rot = [], [], [], []
        self.root_scale = root_scale

    def node(self, name, parent, pos, rot=(0, 0, 0, 1)):
        self.names.append(name)
        self.parents.append(parent)
        self.pos.append(tuple(pos))
        self.rot.append(tuple(rot))
        return len(self.names) - 1

    def local(self, i):
        """Local translation and rotation (glTF TRS) of node i at rest."""
        p = self.parents[i]
        if p is None:
            return (0, 0, 0), (0, 0, 0, 1)
        if self.parents[p] is None:
            # Children of the scaled root: positions / scale, rotation as is.
            t = scale(self.pos[i], 1 / self.root_scale)
            return t, self.rot[i]
        inv = qconj(self.rot[p])
        t = scale(qrot(inv, sub(self.pos[i], self.pos[p])), 1 / self.root_scale)
        return t, qmul(inv, self.rot[i])

    def inverse_bind(self, i):
        """The inverse of node i's model matrix (rotation r, translation p,
        uniform scale s): scale 1/s, rotate by r^-1, translate -r^-1 p / s."""
        s = self.root_scale
        r = qconj(self.rot[i])
        x, y, z, w = r
        m = [[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
             [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
             [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]]
        t = scale(qrot(r, self.pos[i]), -1 / s)
        cols = []
        for c in range(3):
            cols.extend([m[0][c] / s, m[1][c] / s, m[2][c] / s, 0.0])
        cols.extend([t[0], t[1], t[2], 1.0])
        return r6(cols)


def humanoid(r, style, hips_y, thigh, shin, ankle, hip_w, spine, chest, neck, head,
             shoulder_w, upper_arm, forearm, hand, arm_down_deg, names, root_name):
    """Builds a humanoid into rig r. style: 'blender' (identity frames),
    'mixamo' (+Y along each bone), 'ue' (+X along each bone)."""
    def frame(direction):
        if style == "blender":
            return (0, 0, 0, 1)
        return qarc((0, 1, 0) if style == "mixamo" else (1, 0, 0), direction)

    root = r.node(root_name, None, (0, 0, 0))
    pos = {}
    pos["hips"] = (0, hips_y, 0)
    pos["spine"] = (0, hips_y + spine, 0)
    pos["chest"] = (0, hips_y + spine + chest, 0)
    pos["neck"] = (0, hips_y + spine + chest + neck, 0)
    pos["head"] = (0, hips_y + spine + chest + neck + head * 0.3, 0)
    down = math.radians(arm_down_deg)
    for side, sx in (("L", 1), ("R", -1)):
        sh = (sx * shoulder_w * 0.35, pos["neck"][1] - 0.04, 0)
        ua = (sx * shoulder_w, sh[1], 0)
        d = (sx * math.cos(down), -math.sin(down), 0)
        pos["shoulder." + side] = sh
        pos["upper_arm." + side] = ua
        pos["forearm." + side] = add(ua, scale(d, upper_arm))
        pos["hand." + side] = add(ua, scale(d, upper_arm + forearm))
        pos["hand_end." + side] = add(ua, scale(d, upper_arm + forearm + hand))
        pos["thigh." + side] = (sx * hip_w, hips_y, 0)
        pos["shin." + side] = (sx * hip_w, hips_y - thigh, 0)
        pos["foot." + side] = (sx * hip_w, hips_y - thigh - shin, 0)
        pos["toe." + side] = (sx * hip_w, ankle * 0.25, ankle * 1.1)
        pos["toe_end." + side] = (sx * hip_w, ankle * 0.25, ankle * 1.8)
    child = {"hips": "spine", "spine": "chest", "chest": "neck", "neck": "head"}
    for side in "LR":
        child.update({"shoulder." + side: "upper_arm." + side, "upper_arm." + side: "forearm." + side,
                      "forearm." + side: "hand." + side, "hand." + side: "hand_end." + side,
                      "thigh." + side: "shin." + side, "shin." + side: "foot." + side,
                      "foot." + side: "toe." + side, "toe." + side: "toe_end." + side})
    parent = {"hips": None, "spine": "hips", "chest": "spine", "neck": "chest", "head": "neck"}
    for side in "LR":
        parent.update({"shoulder." + side: "chest", "upper_arm." + side: "shoulder." + side,
                       "forearm." + side: "upper_arm." + side, "hand." + side: "forearm." + side,
                       "thigh." + side: "hips", "shin." + side: "thigh." + side,
                       "foot." + side: "shin." + side, "toe." + side: "foot." + side})
    order = ["hips", "spine", "chest", "neck", "head"]
    for side in "LR":
        order += ["shoulder." + side, "upper_arm." + side, "forearm." + side, "hand." + side]
    for side in "LR":
        order += ["thigh." + side, "shin." + side, "foot." + side, "toe." + side]
    index = {}
    for b in order:
        if b == "hips":
            rot = (0, 0, 0, 1)
        elif b == "head":
            rot = frame((0, 1, 0))
        else:
            rot = frame(sub(pos[child[b]], pos[b]))
        p = root if parent[b] is None else index[parent[b]]
        index[b] = r.node(names[b], p, pos[b], rot)
    return index, pos, child


BLENDER = {b: b for b in ["hips", "spine", "chest", "neck", "head"]}
for s in "LR":
    for b in ["shoulder", "upper_arm", "forearm", "hand", "thigh", "shin", "foot", "toe"]:
        BLENDER[f"{b}.{s}"] = f"{b}.{s}"

MIXAMO = {"hips": "mixamorig:Hips", "spine": "mixamorig:Spine", "chest": "mixamorig:Spine1",
          "neck": "mixamorig:Neck", "head": "mixamorig:Head"}
for s, side in (("L", "Left"), ("R", "Right")):
    MIXAMO.update({f"shoulder.{s}": f"mixamorig:{side}Shoulder", f"upper_arm.{s}": f"mixamorig:{side}Arm",
                   f"forearm.{s}": f"mixamorig:{side}ForeArm", f"hand.{s}": f"mixamorig:{side}Hand",
                   f"thigh.{s}": f"mixamorig:{side}UpLeg", f"shin.{s}": f"mixamorig:{side}Leg",
                   f"foot.{s}": f"mixamorig:{side}Foot", f"toe.{s}": f"mixamorig:{side}ToeBase"})

UE = {"hips": "pelvis", "spine": "spine_01", "chest": "spine_02", "neck": "neck_01", "head": "head"}
for s, side in (("L", "l"), ("R", "r")):
    UE.update({f"shoulder.{s}": f"clavicle_{side}", f"upper_arm.{s}": f"upperarm_{side}",
               f"forearm.{s}": f"lowerarm_{side}", f"hand.{s}": f"hand_{side}",
               f"thigh.{s}": f"thigh_{side}", f"shin.{s}": f"calf_{side}",
               f"foot.{s}": f"foot_{side}", f"toe.{s}": f"ball_{side}"})


# ── glTF writing ──────────────────────────────────────────────────────────

class Gltf:
    def __init__(self):
        self.buf = bytearray()
        self.views, self.accessors = [], []

    def add(self, data, fmt, count, ctype, atype, minmax=None, target=None):
        while len(self.buf) % 4:
            self.buf.append(0)
        off = len(self.buf)
        raw = struct.pack("<%d%s" % (len(data), fmt), *data)
        self.buf.extend(raw)
        v = {"buffer": 0, "byteOffset": off, "byteLength": len(raw)}
        if target:
            v["target"] = target
        self.views.append(v)
        a = {"bufferView": len(self.views) - 1, "componentType": ctype, "count": count, "type": atype}
        if minmax:
            a["min"], a["max"] = minmax
        self.accessors.append(a)
        return len(self.accessors) - 1

    def write(self, path, doc):
        doc["buffers"] = [{"byteLength": len(self.buf)}]
        doc["bufferViews"] = self.views
        doc["accessors"] = self.accessors
        js = json.dumps(doc, separators=(",", ":"), sort_keys=True).encode()
        js += b" " * ((4 - len(js) % 4) % 4)
        while len(self.buf) % 4:
            self.buf.append(0)
        glb = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(self.buf))
        glb += struct.pack("<II", len(js), 0x4E4F534A) + js
        glb += struct.pack("<II", len(self.buf), 0x004E4942) + bytes(self.buf)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        open(path, "wb").write(glb)
        return len(glb)


def nodes_of(r, extra_children=None):
    nodes = []
    for i, name in enumerate(r.names):
        n = {"name": name}
        t, q = r.local(i)
        t, q = r6(t), r6(q)
        if any(t):
            n["translation"] = t
        if q != [0.0, 0.0, 0.0, 1.0]:
            n["rotation"] = q
        if r.parents[i] is None and r.root_scale != 1:
            n["scale"] = [r.root_scale] * 3
        kids = [j for j, p in enumerate(r.parents) if p == i]
        if extra_children and i in extra_children:
            kids += extra_children[i]
        if kids:
            n["children"] = kids
        nodes.append(n)
    return nodes


FACES = [((1, 0, 0), (0, 1, 0), (0, 0, 1)), ((-1, 0, 0), (0, 0, 1), (0, 1, 0)),
         ((0, 1, 0), (0, 0, 1), (1, 0, 0)), ((0, -1, 0), (1, 0, 0), (0, 0, 1)),
         ((0, 0, 1), (1, 0, 0), (0, 1, 0)), ((0, 0, -1), (0, 1, 0), (1, 0, 0))]


def character(path, r, index, pos, child, color, widths, title):
    """A skinned character: a box along each bone (rigid to it), the head a cube."""
    g = Gltf()
    joints = list(range(1, len(r.names)))  # every node but the root
    joint_of = {n: j for j, n in enumerate(joints)}
    P, N, J, W, I = [], [], [], [], []

    def box(center, half, rot, joint):
        for n, u, v in FACES:
            base = len(P) // 3
            for su, sv in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
                local = [half[k] * (n[k] + su * u[k] + sv * v[k]) for k in range(3)]
                p = add(center, qrot(rot, local))
                P.extend(r6(p))
                N.extend(r6(qrot(rot, n)))
                J.extend((joint, 0, 0, 0))
                W.extend((1.0, 0.0, 0.0, 0.0))
            a, b, c, d = base, base + 1, base + 2, base + 3
            cross = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
            if sum(cross[k] * n[k] for k in range(3)) > 0:
                I.extend((a, b, c, a, c, d))
            else:
                I.extend((a, c, b, a, d, c))

    for b, node in index.items():
        j = joint_of[node]
        if b == "head":
            box(add(pos["head"], (0, widths["head"] * 0.9, 0.0)), (widths["head"],) * 3, (0, 0, 0, 1), j)
            continue
        if b not in child:
            continue
        a, c = pos[b], pos[child[b]]
        d = sub(c, a)
        length = math.sqrt(sum(x * x for x in d))
        rot = qarc((0, 1, 0), d)
        w = widths.get(b.split(".")[0], 0.05)
        box(add(a, scale(d, 0.5)), (w, length * 0.5, w * (0.8 if "foot" not in b else 1.2)), rot, j)
    nv = len(P) // 3
    lo = [min(P[k::3]) for k in range(3)]
    hi = [max(P[k::3]) for k in range(3)]
    a_pos = g.add(P, "f", nv, 5126, "VEC3", (lo, hi), 34962)
    a_nrm = g.add(N, "f", nv, 5126, "VEC3", None, 34962)
    a_jnt = g.add(J, "H", nv, 5123, "VEC4", None, 34962)
    a_wgt = g.add(W, "f", nv, 5126, "VEC4", None, 34962)
    a_idx = g.add(I, "I", len(I), 5125, "SCALAR", None, 34963)
    ibm = []
    for n in joints:
        ibm.extend(r.inverse_bind(n))
    a_ibm = g.add(ibm, "f", len(joints), 5126, "MAT4")
    body = len(r.names)
    doc = {
        "asset": {"version": "2.0", "generator": "og-anim make_humanoids.py", "copyright": "CC0 1.0, Origin Engine authors"},
        "scene": 0,
        "scenes": [{"name": title, "nodes": [0]}],
        "nodes": nodes_of(r, {0: [body]}) + [{"name": "body", "mesh": 0, "skin": 0}],
        "meshes": [{"name": title, "primitives": [{
            "attributes": {"POSITION": a_pos, "NORMAL": a_nrm, "JOINTS_0": a_jnt, "WEIGHTS_0": a_wgt},
            "indices": a_idx, "material": 0}]}],
        "materials": [{"name": title, "pbrMetallicRoughness": {"baseColorFactor": color, "metallicFactor": 0, "roughnessFactor": 0.7}}],
        "skins": [{"name": title, "joints": joints, "inverseBindMatrices": a_ibm, "skeleton": joints[0]}],
    }
    size = g.write(path, doc)
    print(f"{path}: {size} bytes, {len(r.names)} nodes, {nv} vertices")


# ── the library ───────────────────────────────────────────────────────────

TAU = 2 * math.pi


def library(path):
    r = Rig(1.0)
    index, pos, _ = humanoid(r, "blender", 1.0, 0.45, 0.45, 0.1, 0.15, 0.25, 0.2, 0.18, 0.2,
                             0.22, 0.28, 0.26, 0.08, 0, BLENDER, "Rig")
    g = Gltf()
    animations = []

    def clip(name, duration, rate, chans, events=None):
        samplers, channels = [], []
        n = int(round(duration * rate)) + 1
        times = [i / rate for i in range(n)]
        for node, path_, f in chans:
            vals = []
            for t in times:
                vals.extend(r6(f(t)))
            lanes = len(f(0.0))
            ti = g.add(times, "f", n, 5126, "SCALAR", ([min(times)], [max(times)]))
            vi = g.add(vals, "f", n, 5126, {3: "VEC3", 4: "VEC4"}[lanes])
            samplers.append({"input": ti, "output": vi, "interpolation": "LINEAR"})
            channels.append({"sampler": len(samplers) - 1, "target": {"node": node, "path": path_}})
        a = {"name": name, "samplers": samplers, "channels": channels}
        if events:
            a["extras"] = {"og.events": [{"name": nm, "time": t} for nm, t in events]}
        animations.append(a)

    X, Y, Z = (1, 0, 0), (0, 1, 0), (0, 0, 1)

    def gait(name, duration, rate, stride, stance, lift, height, bob, arm, events):
        """A gait made by inverse kinematics: the hips travel `stride` metres
        a cycle at a constant speed; each foot is planted (still, flat, at
        the ankle's rest height) for `stance` of its cycle, under the hips at
        mid-stance, then swings forward by a stride, lifted by `lift`; each
        leg is solved in its sagittal plane, the knee bending forward."""
        l1, l2 = 0.45, 0.45
        ankle = 0.1

        def hips_y(t):
            u = t / duration
            return height + bob * math.cos(4 * math.pi * (u - stance / 2))

        def foot(t, offset):
            u = (t / duration + offset) % 1.0
            cycle_start = t - u * duration
            planted_z = stride * (cycle_start + stance / 2 * duration) / duration
            if u < stance:
                return planted_z, ankle
            s_ = (u - stance) / (1 - stance)
            e = s_ * s_ * (3 - 2 * s_)
            return planted_z + stride * e, ankle + lift * math.sin(math.pi * s_)

        def leg(t, offset):
            fz, fy = foot(t, offset)
            hz, hy = stride * t / duration, hips_y(t)
            vz, vy = fz - hz, fy - hy
            d = min(math.hypot(vz, vy), l1 + l2 - 1e-5)
            psi = math.atan2(vz, -vy)
            beta = math.acos(max(-1.0, min(1.0, (l1 * l1 + d * d - l2 * l2) / (2 * l1 * d))))
            gamma = math.acos(max(-1.0, min(1.0, (l1 * l1 + l2 * l2 - d * d) / (2 * l1 * l2))))
            thigh = psi + beta
            knee = math.pi - gamma
            return (qaxis(X, -thigh), qaxis(X, knee), qaxis(X, thigh - knee))

        ph = lambda t: t / duration * TAU
        chans = [
            (index["hips"], "translation", lambda t: (0.0, hips_y(t), stride * t / duration)),
            (index["chest"], "rotation", lambda t: qaxis(Y, -0.1 * math.sin(ph(t)))),
            (index["upper_arm.L"], "rotation", lambda t: qmul(qaxis(X, arm * math.sin(ph(t))), qaxis(Z, -1.3))),
            (index["upper_arm.R"], "rotation", lambda t: qmul(qaxis(X, -arm * math.sin(ph(t))), qaxis(Z, 1.3))),
            (index["forearm.L"], "rotation", lambda t: qaxis(Y, -0.3)),
            (index["forearm.R"], "rotation", lambda t: qaxis(Y, 0.3)),
        ]
        for side, offset in (("L", 0.0), ("R", 0.5)):
            for k, part in enumerate(("thigh", "shin", "foot")):
                chans.append((index[f"{part}.{side}"], "rotation",
                              (lambda o, kk: lambda t: leg(t, o)[kk])(offset, k)))
        clip(name, duration, rate, chans, events)

    gait("walk", 1.0, 30, 1.2, 0.6, 0.12, 0.915, 0.015, 0.35, [("footL", 0.0), ("footR", 0.5)])
    gait("run", 0.66, 30, 1.8, 0.45, 0.2, 0.88, 0.02, 0.7, [("footL", 0.0), ("footR", 0.33)])
    clip("idle", 2.0, 15, [
        (index["hips"], "translation", lambda t: (0.0, 1.0 - 0.01 * math.sin(t * TAU / 2), 0.0)),
        (index["chest"], "rotation", lambda t: qaxis(X, 0.03 * math.sin(t * TAU / 2))),
        (index["upper_arm.L"], "rotation", lambda t: qaxis(Z, -1.3)),
        (index["upper_arm.R"], "rotation", lambda t: qaxis(Z, 1.3)),
    ])
    doc = {
        "asset": {"version": "2.0", "generator": "og-anim make_humanoids.py", "copyright": "CC0 1.0, Origin Engine authors"},
        "scene": 0,
        "scenes": [{"name": "humanoid-clips", "nodes": [0]}],
        "nodes": nodes_of(r),
        "animations": animations,
    }
    size = g.write(path, doc)
    print(f"{path}: {size} bytes, {len(r.names)} nodes, {len(animations)} clips")


library(os.path.join(ROOT, "assets/anims/humanoid-clips.glb"))

strider = Rig(0.01)
index, pos, child = humanoid(strider, "mixamo", 1.3, 0.70, 0.44, 0.16, 0.12, 0.2, 0.15, 0.2, 0.24,
                             0.24, 0.32, 0.28, 0.1, 45, MIXAMO, "Armature")
character(os.path.join(ROOT, "assets/models/strider.glb"), strider, index, pos, child,
          [0.22, 0.45, 0.8, 1], {"hips": 0.12, "spine": 0.13, "chest": 0.15, "neck": 0.05, "head": 0.12,
                                  "shoulder": 0.05, "upper_arm": 0.05, "forearm": 0.045, "hand": 0.04,
                                  "thigh": 0.07, "shin": 0.055, "foot": 0.05, "toe": 0.045}, "strider")

stout = Rig(1.0)
index, pos, child = humanoid(stout, "ue", 0.72, 0.30, 0.32, 0.1, 0.2, 0.22, 0.2, 0.12, 0.26,
                             0.3, 0.24, 0.22, 0.09, 10, UE, "root")
character(os.path.join(ROOT, "assets/models/stout.glb"), stout, index, pos, child,
          [0.85, 0.5, 0.15, 1], {"hips": 0.2, "spine": 0.22, "chest": 0.24, "neck": 0.07, "head": 0.16,
                                   "shoulder": 0.07, "upper_arm": 0.075, "forearm": 0.07, "hand": 0.06,
                                   "thigh": 0.1, "shin": 0.085, "foot": 0.07, "toe": 0.06}, "stout")


# ── a side-on 2D puppet for the 2D ragdoll ────────────────────────────────

def puppet(path):
    """A side-on figure facing +x with every bone in the xy plane (identity
    rest rotations, Blender names), the near limbs at z +0.06 and the far
    ones at -0.06, skinned flat boxes in two materials (near light, far
    dark), and a walk in place whose rotations are all about z."""
    r = Rig(1.0)
    root = r.node("Puppet", None, (0, 0, 0))
    pos = {"hips": (0, 0.95, 0), "spine": (0, 1.2, 0), "head": (0, 1.55, 0)}
    for side, z in (("L", 0.06), ("R", -0.06)):
        pos.update({f"upper_arm.{side}": (0, 1.5, z), f"forearm.{side}": (0, 1.22, z), f"hand.{side}": (0, 0.97, z),
                    f"thigh.{side}": (0, 0.92, z * 0.6), f"shin.{side}": (0, 0.5, z * 0.6), f"foot.{side}": (0, 0.1, z * 0.6)})
    parent = {"hips": None, "spine": "hips", "head": "spine"}
    for s_ in "LR":
        parent.update({f"upper_arm.{s_}": "spine", f"forearm.{s_}": f"upper_arm.{s_}", f"hand.{s_}": f"forearm.{s_}",
                       f"thigh.{s_}": "hips", f"shin.{s_}": f"thigh.{s_}", f"foot.{s_}": f"shin.{s_}"})
    order = ["hips", "spine", "head"] + [f"{b}.{s_}" for s_ in "LR" for b in ("upper_arm", "forearm", "hand")] \
        + [f"{b}.{s_}" for s_ in "LR" for b in ("thigh", "shin", "foot")]
    index = {}
    for b in order:
        index[b] = r.node(b, root if parent[b] is None else index[parent[b]], pos[b])
    g = Gltf()
    joints = list(range(1, len(r.names)))
    joint_of = {n: j for j, n in enumerate(joints)}
    # Segments: (bone, from, to, half width).
    ends = {"hips": ("hips", "spine", 0.13), "spine": ("spine", "head", 0.12)}
    for s_ in "LR":
        ends.update({f"upper_arm.{s_}": (f"upper_arm.{s_}", f"forearm.{s_}", 0.05),
                     f"forearm.{s_}": (f"forearm.{s_}", f"hand.{s_}", 0.045),
                     f"thigh.{s_}": (f"thigh.{s_}", f"shin.{s_}", 0.07),
                     f"shin.{s_}": (f"shin.{s_}", f"foot.{s_}", 0.055)})
    prims = []
    for near in (True, False):
        P, N, J, W, I = [], [], [], [], []

        def quad(cx, cy, cz, hx, hy, joint):
            """A flat box (2 cm deep) centred on (cx, cy, cz)."""
            for n, u, v in FACES:
                base = len(P) // 3
                for su, sv in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
                    half = (hx, hy, 0.01)
                    p = [(cx, cy, cz)[k] + half[k] * (n[k] + su * u[k] + sv * v[k]) for k in range(3)]
                    P.extend(r6(p))
                    N.extend(n)
                    J.extend((joint, 0, 0, 0))
                    W.extend((1.0, 0.0, 0.0, 0.0))
                a, b, c, d = base, base + 1, base + 2, base + 3
                cross = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
                if sum(cross[k] * n[k] for k in range(3)) > 0:
                    I.extend((a, b, c, a, c, d))
                else:
                    I.extend((a, c, b, a, d, c))

        for bone, (a, b, w) in ends.items():
            if bone.endswith(".R") == near:
                continue
            pa, pb = pos[a], pos[b]
            quad(pa[0], (pa[1] + pb[1]) / 2, pa[2], w, abs(pa[1] - pb[1]) / 2 + 0.02, joint_of[index[bone]])
        if near:
            quad(0.02, 1.68, 0.0, 0.13, 0.14, joint_of[index["head"]])
            for s_ in "L":
                h = pos[f"hand.{s_}"]
                quad(0.0, h[1] - 0.05, h[2], 0.045, 0.06, joint_of[index[f"hand.{s_}"]])
                f = pos[f"foot.{s_}"]
                quad(0.07, f[1] - 0.05, f[2], 0.12, 0.045, joint_of[index[f"foot.{s_}"]])
        else:
            for s_ in "R":
                h = pos[f"hand.{s_}"]
                quad(0.0, h[1] - 0.05, h[2], 0.045, 0.06, joint_of[index[f"hand.{s_}"]])
                f = pos[f"foot.{s_}"]
                quad(0.07, f[1] - 0.05, f[2], 0.12, 0.045, joint_of[index[f"foot.{s_}"]])
        nv = len(P) // 3
        lo = [min(P[k::3]) for k in range(3)]
        hi = [max(P[k::3]) for k in range(3)]
        prims.append({
            "attributes": {"POSITION": g.add(P, "f", nv, 5126, "VEC3", (lo, hi), 34962),
                           "NORMAL": g.add(N, "f", nv, 5126, "VEC3", None, 34962),
                           "JOINTS_0": g.add(J, "H", nv, 5123, "VEC4", None, 34962),
                           "WEIGHTS_0": g.add(W, "f", nv, 5126, "VEC4", None, 34962)},
            "indices": g.add(I, "I", len(I), 5125, "SCALAR", None, 34963),
            "material": 0 if near else 1,
        })
    ibm = []
    for n in joints:
        ibm.extend(r.inverse_bind(n))
    a_ibm = g.add(ibm, "f", len(joints), 5126, "MAT4")
    # The walk in place: every rotation about z (the plane's normal).
    animations = []
    Z = (0, 0, 1)
    rate, duration = 30, 1.0
    n = int(round(duration * rate)) + 1
    times = [i / rate for i in range(n)]
    ph = lambda t: t / duration * TAU
    chans = [
        (index["hips"], "translation", lambda t: (0.0, 0.95 + 0.02 * math.cos(2 * ph(t)), 0.0)),
        (index["spine"], "rotation", lambda t: qaxis(Z, -0.06 + 0.03 * math.cos(2 * ph(t)))),
    ]
    for s_, o in (("L", 0.0), ("R", math.pi)):
        chans += [
            (index[f"thigh.{s_}"], "rotation", (lambda oo: lambda t: qaxis(Z, 0.45 * math.sin(ph(t) + oo)))(o)),
            (index[f"shin.{s_}"], "rotation", (lambda oo: lambda t: qaxis(Z, -0.6 * max(0.0, math.cos(ph(t) + oo)) ** 2))(o)),
            (index[f"upper_arm.{s_}"], "rotation", (lambda oo: lambda t: qaxis(Z, -0.4 * math.sin(ph(t) + oo)))(o)),
            (index[f"forearm.{s_}"], "rotation", (lambda oo: lambda t: qaxis(Z, 0.35 + 0.15 * math.sin(ph(t) + oo)))(o)),
        ]
    samplers, channels = [], []
    for node, path_, f in chans:
        vals = []
        for t in times:
            vals.extend(r6(f(t)))
        lanes = len(f(0.0))
        ti = g.add(times, "f", n, 5126, "SCALAR", ([min(times)], [max(times)]))
        vi = g.add(vals, "f", n, 5126, {3: "VEC3", 4: "VEC4"}[lanes])
        samplers.append({"input": ti, "output": vi, "interpolation": "LINEAR"})
        channels.append({"sampler": len(samplers) - 1, "target": {"node": node, "path": path_}})
    animations.append({"name": "walk", "samplers": samplers, "channels": channels})
    body = len(r.names)
    doc = {
        "asset": {"version": "2.0", "generator": "og-anim make_humanoids.py", "copyright": "CC0 1.0, Origin Engine authors"},
        "scene": 0,
        "scenes": [{"name": "puppet", "nodes": [0]}],
        "nodes": nodes_of(r, {0: [body]}) + [{"name": "body", "mesh": 0, "skin": 0}],
        "meshes": [{"name": "puppet", "primitives": prims}],
        "materials": [
            {"name": "near", "pbrMetallicRoughness": {"baseColorFactor": [0.95, 0.62, 0.25, 1], "metallicFactor": 0, "roughnessFactor": 0.8}},
            {"name": "far", "pbrMetallicRoughness": {"baseColorFactor": [0.55, 0.32, 0.14, 1], "metallicFactor": 0, "roughnessFactor": 0.8}},
        ],
        "skins": [{"name": "puppet", "joints": joints, "inverseBindMatrices": a_ibm, "skeleton": joints[0]}],
        "animations": animations,
    }
    size = g.write(path, doc)
    print(f"{path}: {size} bytes, {len(r.names)} nodes, 1 clip")


puppet(os.path.join(ROOT, "assets/models/puppet2d.glb"))
