#!/usr/bin/env python3
"""Smooths the wanderer's body skin weights where og-cook's skin probe sees
stretched triangles, until none of its probe poses stretches any.

    python3 tools/smooth-skin.py <in.glb> <out.glb>      (needs numpy)

The probe is og-cook's (crates/og-cook/src/cook/formats/model/skin_report.rs):
rotations about the bind joints on model x, composed down the hierarchy,
flagging a triangle whose longest edge grows past 2.5x its bind length and
0.08 model units. Only mesh 0 (the body) is changed; the cloak keeps its
weights. Vertices sharing a position are averaged together, so seams stay
closed; each pass blends the flagged region with its neighbours and widens
it. Run it after tools/cloak.ts.
"""
import json, struct, sys
import numpy as np

CT = {5126: np.float32, 5121: np.uint8, 5123: np.uint16, 5125: np.uint32}
NC = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
POSES = {
    "rightArmForward": [("RightArm", -70), ("RightForeArm", -40), ("LeftArm", 10)],
    "leftArmForward": [("LeftArm", -70), ("LeftForeArm", -40), ("RightArm", 10)],
    "stride": [("LeftUpLeg", -35), ("LeftLeg", 30), ("RightUpLeg", 25)],
}

def accessor(b, j, base, i):
    a = j["accessors"][i]; v = j["bufferViews"][a["bufferView"]]
    o = base + v.get("byteOffset", 0) + a.get("byteOffset", 0)
    dt = CT[a["componentType"]]; c = NC[a["type"]]; size = np.dtype(dt).itemsize * c
    stride = v.get("byteStride") or size
    raw = np.frombuffer(b, np.uint8, stride * (a["count"] - 1) + size, o)
    rows = np.lib.stride_tricks.as_strided(raw, (a["count"], size), (stride, 1))
    return np.ascontiguousarray(rows).view(dt).reshape(a["count"], c), o, stride

def main(src, dst):
    b = bytearray(open(src, "rb").read()); n = struct.unpack_from("<I", b, 12)[0]
    j = json.loads(b[20:20 + n]); base = 20 + n + 8
    prim = j["meshes"][0]["primitives"][0]; at = prim["attributes"]
    P = accessor(b, j, base, at["POSITION"])[0].astype(np.float64)
    J, oj, sj = accessor(b, j, base, at["JOINTS_0"])
    W, ow, sw = accessor(b, j, base, at["WEIGHTS_0"])
    T = accessor(b, j, base, prim["indices"])[0].reshape(-1, 3)
    skin = j["skins"][0]; joints = skin["joints"]
    names = [j["nodes"][x].get("name") for x in joints]
    ibm = accessor(b, j, base, skin["inverseBindMatrices"])[0].reshape(-1, 4, 4).transpose(0, 2, 1)
    pivots = [np.linalg.inv(m)[:3, 3] for m in ibm]
    parent = {c: i for i, nd in enumerate(j["nodes"]) for c in nd.get("children", [])}

    def longest(X):
        a, c, d = X[T[:, 0]], X[T[:, 1]], X[T[:, 2]]
        return np.maximum.reduce([np.linalg.norm(a - c, axis=1), np.linalg.norm(c - d, axis=1), np.linalg.norm(d - a, axis=1)])
    bind = longest(P); H = np.c_[P, np.ones(len(P))]

    def stretched(Jn, Wn):
        ws = Wn / np.maximum(Wn.sum(1, keepdims=True), 1e-12); found = {}
        for pose, turns in POSES.items():
            angle = {names.index(k): a for k, a in turns}; delta = {}
            def d(k):
                if k not in delta:
                    r = np.radians(angle.get(k, 0.0)); L = np.eye(4)
                    L[1, 1] = L[2, 2] = np.cos(r); L[1, 2] = -np.sin(r); L[2, 1] = np.sin(r)
                    A = np.eye(4); A[:3, 3] = pivots[k]; B = np.eye(4); B[:3, 3] = -pivots[k]
                    p = parent.get(joints[k]); L = A @ L @ B
                    delta[k] = d(joints.index(p)) @ L if p in joints else L
                return delta[k]
            pal = np.stack([d(k) for k in range(len(joints))])
            posed = sum(np.einsum("nij,nj->ni", pal[Jn[:, i]], H)[:, :3] * ws[:, i:i + 1] for i in range(4))
            e = longest(posed); found[pose] = np.where((e > 0.08) & (e > 2.5 * bind))[0]
        return found

    key = {}; weld = np.array([key.setdefault(tuple(p), len(key)) for p in np.round(P, 5)])
    near = [set() for _ in key]
    for a, c, d in weld[T]: near[a] |= {c, d}; near[c] |= {a, d}; near[d] |= {a, c}
    dense = np.zeros((len(key), len(joints)))
    for i in range(4): np.add.at(dense, (weld, J[:, i]), W[:, i])
    dense /= np.bincount(weld)[:, None]
    Jn, Wn = J.copy(), W.copy(); found = stretched(Jn, Wn)
    for step in range(12):
        print(f"pass {step}:", {k: len(v) for k, v in found.items()})
        bad = sorted(set(np.concatenate(list(found.values())).tolist()))
        if not bad: break
        region = set(weld[T[bad]].ravel().tolist())
        for _ in range(1 + step // 3): region |= {m for r in list(region) for m in near[r]}
        for _ in range(4):
            new = dense.copy()
            for r in region: new[r] = 0.5 * dense[r] + 0.5 * dense[list(near[r])].mean(0)
            dense = new
        top = np.argsort(-dense, 1)[:, :4]; w = np.take_along_axis(dense, top, 1); w /= w.sum(1, keepdims=True)
        Jn = top[weld].astype(J.dtype); Wn = w[weld].astype(W.dtype); found = stretched(Jn, Wn)
    else:
        sys.exit("smooth-skin: still stretched after 12 passes")
    for v in range(len(P)):
        b[oj + v * sj:oj + v * sj + Jn.itemsize * 4] = Jn[v].tobytes()
        b[ow + v * sw:ow + v * sw + 16] = Wn[v].astype(np.float32).tobytes()
    open(dst, "wb").write(b)
    print("smooth-skin:", int((np.abs(Wn - W).sum(1) > 1e-4).sum()), "of", len(P), "body vertices reweighted")

if __name__ == "__main__":
    if len(sys.argv) != 3: sys.exit(__doc__)
    main(*sys.argv[1:])
