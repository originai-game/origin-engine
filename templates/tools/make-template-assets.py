#!/usr/bin/env python3
"""Regenerates the template's self-made assets (CC0), byte for byte:
textures/stone.png, textures/stone_n.png and audio/blip.wav.

    python3 templates/tools/make-template-assets.py templates/game/assets

Standard library only (zlib, wave); seeded, so the output is reproducible.
"""
import math, os, random, struct, sys, wave, zlib

out = sys.argv[1] if len(sys.argv) > 1 else "templates/game/assets"
random.seed(20261017)
N = 512


def png(path, w, h, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def lattice(n):
    return [[random.random() for _ in range(n + 1)] for _ in range(n + 1)]


octs = [(lattice(4 * 2 ** o), 4 * 2 ** o) for o in range(6)]


def noise(x, y):
    v = 0; a = 0.5; tot = 0
    for g, n in octs:
        fx = x * n; fy = y * n; ix = int(fx) % n; iy = int(fy) % n; tx = fx - int(fx); ty = fy - int(fy)
        sx = tx * tx * (3 - 2 * tx); sy = ty * ty * (3 - 2 * ty)
        a0 = g[iy][ix] + (g[iy][ix + 1] - g[iy][ix]) * sx
        a1 = g[iy + 1][ix] + (g[iy + 1][ix + 1] - g[iy + 1][ix]) * sx
        v += a * (a0 + (a1 - a0) * sy); tot += a; a *= 0.5
    return v / tot


h = [[0.0] * N for _ in range(N)]
for y in range(N):
    for x in range(N):
        u, v = x / N, y / N
        gx = abs((u * 4) % 1 - 0.5); gy = abs((v * 4) % 1 - 0.5)
        grout = 1.0 if max(gx, gy) > 0.46 else 0.0
        h[y][x] = noise(u, v) * 0.8 + 0.2 - grout * 0.5
crow = []; nrow = []
for y in range(N):
    c = []; n = []
    for x in range(N):
        t = h[y][x]; tile = int(x / N * 4) + int(y / N * 4) * 4
        hue = [(0.62, 0.55, 0.48), (0.55, 0.57, 0.6), (0.66, 0.6, 0.5)][tile % 3]
        k = 0.55 + 0.6 * t
        c += [max(0, min(255, int(255 * q * k))) for q in hue]
        dx = (h[y][(x + 1) % N] - h[y][(x - 1) % N]) * 4; dy = (h[(y + 1) % N][x] - h[(y - 1) % N][x]) * 4
        l = math.sqrt(dx * dx + dy * dy + 1)
        n += [int((-dx / l * 0.5 + 0.5) * 255), int((-dy / l * 0.5 + 0.5) * 255), int((1 / l * 0.5 + 0.5) * 255)]
    crow.append(c); nrow.append(n)
png(os.path.join(out, "textures/stone.png"), N, N, crow)
png(os.path.join(out, "textures/stone_n.png"), N, N, nrow)

sr = 44100; count = int(sr * 0.5); ph = 0; frames = bytearray()
for i in range(count):
    t = i / sr; f = 660 * (2 ** (t / 0.25)) if t < 0.25 else 1320
    ph += 2 * math.pi * f / sr
    env = math.exp(-t * 7) * min(1, t * 400)
    s = env * (0.7 * math.sin(ph) + 0.2 * math.sin(2 * ph) + 0.1 * math.sin(3 * ph))
    frames += struct.pack("<h", int(max(-1, min(1, s)) * 30000))
w = wave.open(os.path.join(out, "audio/blip.wav"), "wb")
w.setnchannels(1); w.setsampwidth(2); w.setframerate(sr); w.writeframes(bytes(frames)); w.close()
