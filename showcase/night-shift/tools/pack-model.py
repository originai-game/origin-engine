#!/usr/bin/env python3
"""Repack a generated GLB for the repository: textures capped at a size and
stored as JPEG (PNG is kept only where the image has real alpha). Geometry,
skins and clips are copied byte for byte.

    tools/pack-model.py <in.glb> <out.glb> [max-texture-size, default 2048] [glow]

With a fourth argument, every material also glows in its own colours: its
base colour texture becomes its emissive texture at that strength
(KHR_materials_emissive_strength), as a paper lantern lit from inside.

Needs ImageMagick (`magick`).
"""
import json
import struct
import subprocess
import sys


def magick(args, data):
    return subprocess.run(["magick", *args], input=data, stdout=subprocess.PIPE, check=True).stdout


def main():
    src, dst = sys.argv[1], sys.argv[2]
    cap = int(sys.argv[3]) if len(sys.argv) > 3 else 2048
    glow = float(sys.argv[4]) if len(sys.argv) > 4 else 0.0
    raw = open(src, "rb").read()
    magic, _version, _length = struct.unpack_from("<III", raw, 0)
    assert magic == 0x46546C67, "not a GLB"
    json_len, _ = struct.unpack_from("<II", raw, 12)
    doc = json.loads(raw[20 : 20 + json_len])
    bin_at = 20 + json_len
    bin_len, _ = struct.unpack_from("<II", raw, bin_at)
    blob = raw[bin_at + 8 : bin_at + 8 + bin_len]

    if glow > 0:
        for material in doc.get("materials", []):
            base = material.get("pbrMetallicRoughness", {}).get("baseColorTexture")
            if base is None:
                continue
            material["emissiveTexture"] = dict(base)
            material["emissiveFactor"] = [1, 1, 1]
            material.setdefault("extensions", {})["KHR_materials_emissive_strength"] = {"emissiveStrength": glow}
        used = doc.setdefault("extensionsUsed", [])
        if "KHR_materials_emissive_strength" not in used:
            used.append("KHR_materials_emissive_strength")

    image_views = {img["bufferView"]: img for img in doc.get("images", []) if "bufferView" in img}
    out = bytearray()
    for index, view in enumerate(doc["bufferViews"]):
        data = blob[view.get("byteOffset", 0) : view.get("byteOffset", 0) + view["byteLength"]]
        image = image_views.get(index)
        if image is not None:
            opaque = magick(["-", "-format", "%[opaque]", "info:"], data).decode().strip().lower() == "true"
            resize = ["-resize", f"{cap}x{cap}>"]
            if opaque:
                data = magick(["-", *resize, "-strip", "-sampling-factor", "4:4:4", "-quality", "90", "jpg:-"], data)
                image["mimeType"] = "image/jpeg"
            else:
                data = magick(["-", *resize, "-strip", "png:-"], data)
                image["mimeType"] = "image/png"
        while len(out) % 4:
            out.append(0)
        view["byteOffset"] = len(out)
        view["byteLength"] = len(data)
        out += data
    while len(out) % 4:
        out.append(0)
    doc["buffers"][0]["byteLength"] = len(out)

    text = json.dumps(doc, separators=(",", ":")).encode()
    text += b" " * (-len(text) % 4)
    total = 12 + 8 + len(text) + 8 + len(out)
    with open(dst, "wb") as f:
        f.write(struct.pack("<III", 0x46546C67, 2, total))
        f.write(struct.pack("<II", len(text), 0x4E4F534A))
        f.write(text)
        f.write(struct.pack("<II", len(out), 0x004E4942))
        f.write(out)
    print(f"{dst}: {len(raw)} -> {total} bytes")


main()
