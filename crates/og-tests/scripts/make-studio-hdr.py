#!/usr/bin/env python3
"""Writes a synthetic equirectangular HDR "studio" as a Radiance .hdr (flat
RGBE scanlines): a warm floor, a sky from a pale horizon to a blue zenith, a
warm key softbox in front, up and to the left, and a cool rim strip behind
and to the right. The environment-map gfx scene and the fx-showcase's
environment station light by it.

  python3 crates/og-tests/scripts/make-studio-hdr.py <out.hdr> [width]

Column u looks towards longitude (u - 0.5) * 360 degrees from +x towards +z
(the image's centre is +x), rows run from +y to -y: og-render's (and
three.js's) equirectangular mapping. Deterministic: the same bytes every run.
"""
import math
import struct
import sys


def lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def smooth(e0, e1, x):
    t = min(max((x - e0) / (e1 - e0), 0.0), 1.0)
    return t * t * (3.0 - 2.0 * t)


def norm(v):
    n = math.sqrt(sum(c * c for c in v))
    return tuple(c / n for c in v)


KEY = norm((-0.5, 0.55, 0.65))
RIM = norm((0.75, 0.25, -0.6))


def radiance(d):
    x, y, z = d
    horizon = (0.95, 0.88, 0.78)
    zenith = (0.30, 0.45, 0.85)
    floor = (0.16, 0.13, 0.10)
    sky = lerp(horizon, zenith, math.sqrt(max(y, 0.0)))
    c = lerp(floor, sky, smooth(-0.04, 0.04, y))
    key = smooth(math.cos(math.radians(14)), math.cos(math.radians(9)), sum(a * b for a, b in zip(d, KEY)))
    rim = smooth(math.cos(math.radians(11)), math.cos(math.radians(6)), sum(a * b for a, b in zip(d, RIM)))
    return tuple(c[i] + key * (36.0, 30.0, 22.0)[i] + rim * (6.0, 9.0, 14.0)[i] for i in range(3))


def rgbe(c):
    v = max(c)
    if v < 1e-32:
        return bytes(4)
    m, e = math.frexp(v)
    s = m * 256.0 / v
    return bytes([min(255, int(c[0] * s)), min(255, int(c[1] * s)), min(255, int(c[2] * s)), e + 128])


def main():
    out = sys.argv[1]
    w = int(sys.argv[2]) if len(sys.argv) > 2 else 64
    h = w // 2
    data = bytearray()
    for row in range(h):
        lat = (0.5 - (row + 0.5) / h) * math.pi
        for col in range(w):
            lon = ((col + 0.5) / w - 0.5) * 2.0 * math.pi
            d = (math.cos(lat) * math.cos(lon), math.sin(lat), math.cos(lat) * math.sin(lon))
            # 4x4 supersampling keeps the small lights' energy at low resolution.
            acc = [0.0, 0.0, 0.0]
            for sy in range(4):
                for sx in range(4):
                    la = (0.5 - (row + (sy + 0.5) / 4) / h) * math.pi
                    lo = ((col + (sx + 0.5) / 4) / w - 0.5) * 2.0 * math.pi
                    dd = (math.cos(la) * math.cos(lo), math.sin(la), math.cos(la) * math.sin(lo))
                    r = radiance(dd)
                    for i in range(3):
                        acc[i] += r[i] / 16.0
            data += rgbe(acc)
    header = f"#?RADIANCE\n# og-tests make-studio-hdr.py\nFORMAT=32-bit_rle_rgbe\n\n-Y {h} +X {w}\n".encode()
    with open(out, "wb") as f:
        f.write(header + bytes(data))


main()
