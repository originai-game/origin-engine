"""Composes Greywatch's characters: a KayKit character (a skinned .glb on
the Rig_Medium skeleton) with the weapons it carries (KayKit .gltf props)
skinned rigidly to its hand slots, written as one .glb under
assets/models/. The engine draws a glTF as one model and has no joint
attachments, so a weapon that swings with the hand is a mesh of the skin
whose every vertex follows the hand slot's joint.

    python3 tools/compose.py <kaykit dir> assets/models

<kaykit dir> holds the catalog's KayKit files as `assets_fetch_plan`
places them (KayKit_Adventurers_2.0_FREE/..., KayKit_Skeletons_1.1_FREE/...):
see LICENSES.md for each group. Standard library only; the same input
writes the same bytes.
"""

import json
import os
import struct
import sys

CHARACTERS = {
    "knight.glb": ("KayKit_Adventurers_2.0_FREE/Characters/gltf/Knight.glb", [
        ("KayKit_Adventurers_2.0_FREE/Assets/gltf/sword_1handed.gltf", "handslot.r"),
        ("KayKit_Adventurers_2.0_FREE/Assets/gltf/shield_round.gltf", "handslot.l"),
    ]),
    "barbarian.glb": ("KayKit_Adventurers_2.0_FREE/Characters/gltf/Barbarian.glb", [
        ("KayKit_Adventurers_2.0_FREE/Assets/gltf/axe_1handed.gltf", "handslot.r"),
    ]),
    "mage.glb": ("KayKit_Adventurers_2.0_FREE/Characters/gltf/Mage.glb", [
        ("KayKit_Adventurers_2.0_FREE/Assets/gltf/staff.gltf", "handslot.r"),
    ]),
    "skeleton.glb": ("KayKit_Skeletons_1.1_FREE/characters/gltf/Skeleton_Minion.glb", [
        ("KayKit_Skeletons_1.1_FREE/assets/gltf/Skeleton_Blade.gltf", "handslot.r"),
        ("KayKit_Skeletons_1.1_FREE/assets/gltf/Skeleton_Shield_Small_A.gltf", "handslot.l"),
    ]),
    "captain.glb": ("KayKit_Skeletons_1.1_FREE/characters/gltf/Skeleton_Warrior.glb", [
        ("KayKit_Skeletons_1.1_FREE/assets/gltf/Skeleton_Axe.gltf", "handslot.r"),
    ]),
}

FLOAT, U16, U32 = 5126, 5123, 5125


def read_glb(path):
    b = open(path, "rb").read()
    magic, _, _ = struct.unpack("<III", b[:12])
    assert magic == 0x46546C67, path
    at = 12
    doc, binary = None, b""
    while at < len(b):
        n, kind = struct.unpack("<II", b[at:at + 8])
        chunk = b[at + 8:at + 8 + n]
        if kind == 0x4E4F534A:
            doc = json.loads(chunk)
        elif kind == 0x004E4942:
            binary = chunk
        at += 8 + n
    return doc, bytearray(binary)


def write_glb(path, doc, binary):
    while len(binary) % 4:
        binary.append(0)
    doc["buffers"] = [{"byteLength": len(binary)}]
    text = json.dumps(doc, separators=(",", ":"), sort_keys=True).encode()
    text += b" " * ((4 - len(text) % 4) % 4)
    total = 12 + 8 + len(text) + 8 + len(binary)
    with open(path, "wb") as f:
        f.write(struct.pack("<III", 0x46546C67, 2, total))
        f.write(struct.pack("<II", len(text), 0x4E4F534A) + text)
        f.write(struct.pack("<II", len(binary), 0x004E4942) + bytes(binary))


def quat_mat(t, r, s):
    x, y, z, w = r
    sx, sy, sz = s
    m = [
        [(1 - 2 * (y * y + z * z)) * sx, (2 * (x * y - z * w)) * sy, (2 * (x * z + y * w)) * sz, t[0]],
        [(2 * (x * y + z * w)) * sx, (1 - 2 * (x * x + z * z)) * sy, (2 * (y * z - x * w)) * sz, t[1]],
        [(2 * (x * z - y * w)) * sx, (2 * (y * z + x * w)) * sy, (1 - 2 * (x * x + y * y)) * sz, t[2]],
        [0, 0, 0, 1],
    ]
    return m


def mul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]


def local(node):
    return quat_mat(node.get("translation", [0, 0, 0]), node.get("rotation", [0, 0, 0, 1]), node.get("scale", [1, 1, 1]))


def world_of(doc, index):
    parent = {c: i for i, n in enumerate(doc["nodes"]) for c in n.get("children", [])}
    m = local(doc["nodes"][index])
    at = index
    while at in parent:
        at = parent[at]
        m = mul(local(doc["nodes"][at]), m)
    return m


def accessor(doc, binary, i):
    a = doc["accessors"][i]
    v = doc["bufferViews"][a["bufferView"]]
    width = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}[a["type"]]
    fmt = {FLOAT: "f", U16: "H", U32: "I"}[a["componentType"]]
    size = struct.calcsize(fmt)
    stride = v.get("byteStride", size * width)
    base = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    out = []
    for k in range(a["count"]):
        out.append(struct.unpack_from("<" + fmt * width, binary, base + k * stride))
    return out


def append(doc, binary, values, kind, ctype, target):
    while len(binary) % 4:
        binary.append(0)
    fmt = {FLOAT: "f", U16: "H", U32: "I", 5121: "B"}[ctype]
    start = len(binary)
    for v in values:
        binary.extend(struct.pack("<" + fmt * len(v), *v))
    doc["bufferViews"].append({"buffer": 0, "byteOffset": start, "byteLength": len(binary) - start, "target": target})
    acc = {"bufferView": len(doc["bufferViews"]) - 1, "componentType": ctype, "count": len(values), "type": kind}
    if kind == "VEC3" and ctype == FLOAT:
        acc["min"] = [min(v[i] for v in values) for i in range(3)]
        acc["max"] = [max(v[i] for v in values) for i in range(3)]
    doc["accessors"].append(acc)
    return len(doc["accessors"]) - 1


def embedded_image(doc, binary, i):
    img = doc["images"][i]
    v = doc["bufferViews"][img["bufferView"]]
    return bytes(binary[v.get("byteOffset", 0):v.get("byteOffset", 0) + v["byteLength"]])


def compose(src, out, body, weapons):
    doc, binary = read_glb(os.path.join(src, body))
    names = [n.get("name") for n in doc["nodes"]]
    skin = doc["skins"][0]
    root = next(i for i, n in enumerate(doc["nodes"]) if n.get("name") == "Rig_Medium")
    for path, slot in weapons:
        w = json.load(open(os.path.join(src, path)))
        wbin = open(os.path.join(os.path.dirname(os.path.join(src, path)), w["buffers"][0]["uri"]), "rb").read()
        joint = skin["joints"].index(names.index(slot))
        m = world_of(doc, names.index(slot))
        prim = w["meshes"][0]["primitives"][0]
        pos = [tuple(sum(m[r][c] * p[c] for c in range(3)) + m[r][3] for r in range(3)) for p in accessor(w, wbin, prim["attributes"]["POSITION"])]
        nrm = []
        for n in accessor(w, wbin, prim["attributes"]["NORMAL"]):
            v = [sum(m[r][c] * n[c] for c in range(3)) for r in range(3)]
            l = sum(x * x for x in v) ** 0.5 or 1
            nrm.append(tuple(x / l for x in v))
        uv = accessor(w, wbin, prim["attributes"]["TEXCOORD_0"])
        idx = accessor(w, wbin, prim["indices"])
        # The weapon's texture: the character's own when the bytes agree, else a new image of the skin's file.
        image = open(os.path.join(os.path.dirname(os.path.join(src, path)), w["images"][0]["uri"]), "rb").read()
        material = None
        for mi, mat in enumerate(doc["materials"]):
            tex = mat.get("pbrMetallicRoughness", {}).get("baseColorTexture")
            if tex is not None and embedded_image(doc, binary, doc["textures"][tex["index"]]["source"]) == image:
                material = mi
                break
        if material is None:
            while len(binary) % 4:
                binary.append(0)
            start = len(binary)
            binary.extend(image)
            doc["bufferViews"].append({"buffer": 0, "byteOffset": start, "byteLength": len(image)})
            doc["images"].append({"bufferView": len(doc["bufferViews"]) - 1, "mimeType": "image/png"})
            doc["textures"].append({"sampler": doc["textures"][0].get("sampler", 0), "source": len(doc["images"]) - 1})
            mat = dict(w["materials"][0])
            mat["pbrMetallicRoughness"] = dict(mat["pbrMetallicRoughness"], baseColorTexture={"index": len(doc["textures"]) - 1})
            doc["materials"].append(mat)
            material = len(doc["materials"]) - 1
        attributes = {
            "POSITION": append(doc, binary, pos, "VEC3", FLOAT, 34962),
            "NORMAL": append(doc, binary, nrm, "VEC3", FLOAT, 34962),
            "TEXCOORD_0": append(doc, binary, uv, "VEC2", FLOAT, 34962),
            "JOINTS_0": append(doc, binary, [(joint, 0, 0, 0)] * len(pos), "VEC4", 5121, 34962),
            "WEIGHTS_0": append(doc, binary, [(1.0, 0.0, 0.0, 0.0)] * len(pos), "VEC4", FLOAT, 34962),
        }
        indices = append(doc, binary, idx, "SCALAR", U16 if len(pos) < 65536 else U32, 34963)
        name = os.path.splitext(os.path.basename(path))[0]
        doc["meshes"].append({"name": name, "primitives": [{"attributes": attributes, "indices": indices, "material": material}]})
        doc["nodes"].append({"name": name, "mesh": len(doc["meshes"]) - 1, "skin": 0})
        doc["nodes"][root].setdefault("children", []).insert(0, len(doc["nodes"]) - 1)
    for bv in doc["bufferViews"]:
        bv["buffer"] = 0
    write_glb(out, doc, binary)


def main():
    src, dst = sys.argv[1], sys.argv[2]
    os.makedirs(dst, exist_ok=True)
    for name, (body, weapons) in CHARACTERS.items():
        compose(src, os.path.join(dst, name), body, weapons)
        print(f"{dst}/{name}")


if __name__ == "__main__":
    main()
