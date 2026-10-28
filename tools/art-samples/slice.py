#!/usr/bin/env python3
"""Turns generated images into a sample scene's sources (README.md).

    slice.py kit <kit.png> <out-dir> <long-side> [pixel|paint]
        a 2 x 2 sheet (hero, tree / crate, lantern) cut into its quadrants,
        each trimmed to its subject and scaled so its long side is at most
        <long-side> (point sampling for pixel art, so blocks stay one colour)
    slice.py image <in> <out> <WxH> [pixel|paint] [crop-top-fraction]
        one image scaled to exactly WxH (a ground tile, a far layer)

Needs ImageMagick (`magick`). Pixel art is scaled with -filter point to a
few times its logical grid; the cook then finds the grid itself
(og-cook README §Art conformance) or takes the brief's (`conform { size }`).
"""
import subprocess, sys, os


def magick(*args):
    subprocess.run(["magick", *args], check=True)


def kit(src, out, long_side, mode):
    os.makedirs(out, exist_ok=True)
    w, h = map(int, subprocess.run(["magick", "identify", "-format", "%w %h", src], capture_output=True, text=True, check=True).stdout.split())
    filt = ["-filter", "point"] if mode == "pixel" else []
    for name, (qx, qy) in {"hero": (0, 0), "tree": (1, 0), "crate": (0, 1), "lantern": (1, 1)}.items():
        crop = f"{w // 2}x{h // 2}+{qx * w // 2}+{qy * h // 2}"
        magick(src, "-crop", crop, "+repage", "-alpha", "on", "-channel", "A", "-threshold", "50%", "+channel",
               "-trim", "+repage", *filt, "-resize", f"{long_side}x{long_side}>", "-define", "png:compression-level=9",
               os.path.join(out, f"{name}.png"))


def image(src, dst, size, mode, top=0.0):
    filt = ["-filter", "point"] if mode == "pixel" else []
    crop = []
    if top:
        h = int(subprocess.run(["magick", "identify", "-format", "%h", src], capture_output=True, text=True, check=True).stdout)
        crop = ["-gravity", "south", "-crop", f"100%x{int(h * (1 - top))}+0+0", "+repage"]
    quality = ["-quality", "86"] if dst.endswith(".jpg") else ["-define", "png:compression-level=9"]
    magick(src, *crop, *filt, "-resize", f"{size}!", *quality, dst)


if __name__ == "__main__":
    what = sys.argv[1]
    if what == "kit":
        kit(sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5] if len(sys.argv) > 5 else "pixel")
    elif what == "image":
        image(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5] if len(sys.argv) > 5 else "pixel", float(sys.argv[6]) if len(sys.argv) > 6 else 0.0)
    else:
        sys.exit(__doc__)
