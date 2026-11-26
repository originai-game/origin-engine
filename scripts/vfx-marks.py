#!/usr/bin/env python3
"""The cinematic-realism pack's surface marks, generated (MIT, Origin Engine):

    uv run --with numpy --with pillow scripts/vfx-marks.py [out-dir]

writes textures/{scorch,scorch-normal,embers,chip,chip-normal}.png, the
other marks (skid, gouge, fracture, dent, wet) and the drawn sprites
(sigil, streak, taper, leaves, flames) under packages/vfx-packs/cinematic-realism (or
out-dir). Colour images are sRGB
RGBA, their alpha the mark's coverage; normal maps are tangent space, +x
along u and +y up the image (glTF), from each mark's height field.
"""
import os
import sys

import numpy as np
from PIL import Image

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "packages", "vfx-packs", "cinematic-realism", "textures")


def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def lattice(seed, n=64):
    return np.random.default_rng(seed).random((n, n))


def value_noise(g, x, y):
    n = g.shape[0]
    i, j = np.floor(x).astype(int), np.floor(y).astype(int)
    fx, fy = x - i, y - j
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a, b = g[j % n, i % n], g[j % n, (i + 1) % n]
    c, d = g[(j + 1) % n, i % n], g[(j + 1) % n, (i + 1) % n]
    return (a + (b - a) * fx) * (1 - fy) + (c + (d - c) * fx) * fy


def fbm(x, y, seed, octaves=5):
    total, norm = 0.0, 0.0
    for k in range(octaves):
        w = 0.5 ** k
        # Each octave's lattice is turned, so no shape keeps the grid's axes.
        c, t = np.cos(0.5 + 0.6435 * k), np.sin(0.5 + 0.6435 * k)
        u, v = (x * c - y * t) * 2 ** k + 17.3 * k, (x * t + y * c) * 2 ** k + 9.1 * k
        total = total + w * value_noise(lattice(seed + k), u, v)
        norm += w
    return total / norm


def grid(size):
    c = (np.arange(size) + 0.5) / size * 2 - 1
    x, y = np.meshgrid(c, c)
    return x, y, np.hypot(x, y), np.arctan2(y, x)


def around(a, k, seed, octaves=3):
    """Noise of the angle alone, seamless round the circle."""
    return fbm(np.cos(a) * k + 31.0, np.sin(a) * k + 31.0, seed, octaves)


def normal_map(h, strength):
    """Height (image rows down) to a glTF tangent-space normal map."""
    size = h.shape[0]
    dy, dx = np.gradient(h, 2.0 / size)
    n = np.stack([-dx * strength, dy * strength, np.ones_like(h)], -1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    return np.round((n * 0.5 + 0.5) * 255).astype(np.uint8)


def save(name, rgb, alpha=None):
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    if alpha is None:
        img = Image.fromarray(rgb, "RGB")
    else:
        a = np.clip(alpha * 255, 0, 255).astype(np.uint8)
        img = Image.fromarray(np.dstack([rgb, a]), "RGBA")
    img.save(os.path.join(OUT, name), optimize=True)


def scorch(size=512):
    """A blast's mark: a blotchy burnt core, soot thinning outwards in a few
    short uneven tongues, a pale ash ring where the heat was highest and
    spatter thrown past it, over a shallow crater with a broken lip."""
    x, y, r, a = grid(size)
    warp = fbm(x * 2.2 + 5, y * 2.2 + 5, 11) - 0.5
    blotch = fbm(x * 5 + 2, y * 5 - 3, 13, 4)
    grain = fbm(x * 40, y * 40, 23, 3)
    # The edge wanders: a few broad tongues, never a regular star.
    tongue = around(a, 1.3, 31, 2) * 0.6 + around(a + warp, 3.1, 37, 2) * 0.4
    edge = 0.44 + 0.3 * tongue
    d = r + warp * 0.22 + (blotch - 0.5) * 0.16
    core = 1 - smooth(0.12, 0.42, d)
    stain = (1 - smooth(edge * 0.5, edge + 0.14, d)) * (0.5 + 0.5 * smooth(0.3, 0.7, blotch))
    # Fine streaks only inside the stain's outer half, dragged outwards.
    fine = smooth(0.4, 0.8, around(a + warp * 0.6, 20.0, 41, 3))
    streaks = fine * smooth(edge * 0.5, edge * 0.8, d) * (1 - smooth(edge * 0.85, edge + 0.16, d)) * 0.3
    spatter = smooth(0.6, 0.85, fbm(x * 34, y * 34, 43, 3)) * (1 - smooth(edge * 0.8, edge + 0.25, d)) * 0.4
    alpha = np.clip(np.maximum.reduce([core * 0.95, stain * 0.88, streaks, spatter]), 0, 1)
    alpha *= (0.8 + 0.2 * grain) * (1 - smooth(0.88, 0.98, r))
    # Soot nearly black and slightly warm; ash a blotchy grey round the core.
    ash = np.exp(-((d - 0.24) / 0.08) ** 2) * smooth(0.45, 0.7, fbm(x * 9, y * 9, 47))
    tone = 14 + 10 * grain + 9 * blotch + 52 * ash
    rgb = np.dstack([tone * 1.06, tone * 0.98, tone * 0.9])
    save("scorch.png", rgb, alpha)
    # Height: the bowl, its broken lip and loose grit.
    lip = np.exp(-((d - 0.24) / 0.05) ** 2) * smooth(0.3, 0.7, fbm(x * 4, y * 4, 53, 2)) * smooth(0.08, 0.18, r)
    bowl = -(1 - smooth(0.0, 0.26, d))
    grit = (fbm(x * 26, y * 26, 59, 3) - 0.5) * (1 - smooth(0.3, 0.75, r))
    h = 0.5 * bowl + 0.28 * lip + 0.12 * grit
    save("scorch-normal.png", normal_map(h, 0.2))


def embers(size=512):
    """What still glows in a fresh scorch: scattered hot flecks and a few
    short burning seams, thickest just inside the lip. Its colour is the
    glow's (the decal's `emissive` multiplies it); its alpha where it is."""
    x, y, r, a = grid(size)
    seams = 1 - smooth(0.0, 0.02, np.abs(fbm(x * 7, y * 7, 61, 4) - 0.5))
    patch = smooth(0.56, 0.66, fbm(x * 3 + 3, y * 3, 67, 3))
    flecks = smooth(0.7, 0.8, fbm(x * 36, y * 36, 71, 3))
    where = (1 - smooth(0.25, 0.5, r)) * (0.3 + 0.7 * smooth(0.04, 0.2, r))
    alpha = np.clip(seams * patch * 0.7 + flecks * 0.85, 0, 1) * where
    heat = np.clip(0.4 + 0.6 * flecks + 0.3 * seams * patch, 0, 1)
    rgb = np.dstack([255 * np.ones_like(r), 70 + 130 * heat, 14 + 70 * heat ** 2])
    save("embers.png", rgb, alpha)


def chip(size=256):
    """A hard surface struck by a bullet or a fragment: a dark pit, the
    pale flake spalled off round it, hairline cracks and a breath of dust."""
    x, y, r, a = grid(size)
    edge = 0.2 + 0.16 * around(a, 3.0, 67) + 0.05 * (around(a, 11.0, 71) - 0.5)
    spall = 1 - smooth(edge - 0.035, edge, r)
    pit = 1 - smooth(0.07, 0.12, r + 0.03 * (around(a, 6.0, 73) - 0.5))
    cracks = np.zeros_like(r)
    rng = np.random.default_rng(79)
    for k in range(7):
        base = k * 2 * np.pi / 7 + (rng.random() - 0.5) * 0.6
        reach = 0.4 + 0.45 * rng.random()
        wobble = (fbm(r * 6, np.full_like(r, k * 3.7), 83, 3) - 0.5) * 0.9 * np.sqrt(r)
        d = np.abs((a - base - wobble + np.pi) % (2 * np.pi) - np.pi) * r
        width = 0.007 * (1 - smooth(edge, reach, r)) + 0.0025
        cracks = np.maximum(cracks, (1 - smooth(0.0, width, d)) * (1 - smooth(reach - 0.1, reach, r)))
    cracks *= 1 - spall
    grain = fbm(x * 30, y * 30, 89, 3)
    dust = (1 - smooth(edge, 0.85, r)) * 0.16 * (0.5 + grain)
    alpha = np.maximum.reduce([pit * 0.97, spall * 0.62, cracks * 0.8, dust])
    tone = np.where(pit > 0.5, 26 + 10 * grain, np.where(spall > 0.5, 118 + 46 * grain, 150))
    tone = np.where((cracks > dust) & (spall < 0.5), 32, tone)
    rgb = np.dstack([tone, tone * 0.97, tone * 0.93])
    save("chip.png", rgb, alpha * (1 - smooth(0.9, 0.99, r)))
    h = -0.35 * spall - 0.65 * pit - 0.25 * cracks + 0.05 * (grain - 0.5) * spall
    save("chip-normal.png", normal_map(h, 0.16))


def skid(w=512, h=128):
    """A tyre's mark, u along the travel: rubber laid in lengthwise streaks
    between the tread's grooves, thin where the tyre skipped, feathered at
    both ends so marks laid end to end read as one."""
    u, v = np.meshgrid((np.arange(w) + 0.5) / w, (np.arange(h) + 0.5) / h * 2 - 1)
    drift = (fbm(u * 3, v * 0.2, 71, 3) - 0.5) * 0.12
    vv = v + drift
    band = 1 - smooth(0.62, 0.86, np.abs(vv))
    grooves = 0.55 + 0.45 * smooth(0.15, 0.45, np.abs(np.sin(vv * 9.5)))
    streak = fbm(u * 2.5, v * 26, 73, 4)
    skip = smooth(0.25, 0.6, fbm(u * 9, v * 1.5, 79, 3))
    ends = smooth(0.0, 0.1, u) * (1 - smooth(0.9, 1.0, u))
    alpha = band * grooves * (0.45 + 0.55 * streak) * (0.5 + 0.5 * skip) * ends * 0.9
    tone = 16 + 10 * streak
    save("skid.png", np.dstack([tone, tone, tone * 1.02]), alpha)


def gouge(size=256):
    """A round's gouge in timber, u along the grain: pale torn fibres in a
    ragged slot, darker where it is deepest, splinters lifted at its ends."""
    x, y, r, a = grid(size)
    fibre = fbm(x * 3, y * 38, 83, 4)
    rag = (fbm(x * 7, y * 30, 89, 3) - 0.5) * 0.5
    d = np.hypot(x / (0.62 + rag * 0.5), y / 0.2)
    slot = 1 - smooth(0.55, 1.0, d)
    pit = 1 - smooth(0.0, 0.5, np.hypot(x / 0.22, y / 0.12))
    splinter = smooth(0.62, 0.8, fibre) * (1 - smooth(0.9, 1.5, d)) * smooth(0.5, 0.9, d)
    alpha = np.clip(np.maximum(slot * (0.75 + 0.25 * fibre), splinter * 0.8), 0, 1)
    tone = 0.55 + 0.45 * fibre - 0.6 * pit
    rgb = np.dstack([205 * tone + 20, 170 * tone + 14, 118 * tone + 8])
    save("gouge.png", rgb, alpha)
    hgt = -0.8 * slot - 0.6 * pit + 0.35 * splinter + 0.15 * (fibre - 0.5) * slot
    save("gouge-normal.png", normal_map(hgt, 0.14))


def fracture(size=512):
    """A round through glass: a crushed white eye, long radial cracks that
    wander, and short arcs joining neighbours near the eye."""
    x, y, r, a = grid(size)
    rng = np.random.default_rng(97)
    n = 13
    angles = np.sort((np.arange(n) + rng.uniform(-0.35, 0.35, n)) * (2 * np.pi / n))
    reach = rng.uniform(0.45, 0.95, n)
    lines = np.zeros_like(r)
    for k in range(n):
        wander = (fbm(r * 5 + 3.1 * k, r * 0 + 1.7 * k, 101 + k, 3) - 0.5) * 0.5 * smooth(0.05, 0.6, r)
        da = np.angle(np.exp(1j * (a - angles[k] - wander)))
        d = np.abs(da) * r
        width = 0.0035 + 0.004 * (1 - smooth(0.0, reach[k], r))
        lines = np.maximum(lines, (1 - smooth(width * 0.4, width, d)) * (1 - smooth(reach[k] * 0.9, reach[k], r)) * (da * 0 + 1) * (np.abs(da) < 1.0))
    arcs = np.zeros_like(r)
    for j, rj in enumerate([0.16, 0.27, 0.41]):
        wob = (around(a, 2.0, 131 + j, 3) - 0.5) * 0.09
        gate = smooth(0.52, 0.6, around(a, 3.5 + j, 137 + j, 2))
        arcs = np.maximum(arcs, (1 - smooth(0.002, 0.006, np.abs(r - rj - wob))) * gate)
    eye = 1 - smooth(0.035, 0.085, r + (fbm(x * 30, y * 30, 139, 3) - 0.5) * 0.04)
    frost = (1 - smooth(0.05, 0.2, r)) * smooth(0.45, 0.8, fbm(x * 60, y * 60, 149, 2)) * 0.5
    alpha = np.clip(np.maximum.reduce([lines * 0.9, arcs * 0.7, eye * 0.95, frost]), 0, 1) * (1 - smooth(0.92, 0.99, r))
    tone = 228 - 150 * eye * (1 - smooth(0.0, 0.03, r))
    save("fracture.png", np.dstack([tone, tone * 1.01, tone * 1.02]), alpha)
    save("fracture-normal.png", normal_map(-lines - 0.7 * arcs - 0.8 * eye, 0.05))


def dent(size=128):
    """A round's strike on plate: a shallow dish, bare bright metal on its
    rim and soot in its eye."""
    x, y, r, a = grid(size)
    wob = (around(a, 2.4, 151, 3) - 0.5) * 0.12
    d = r + wob
    rim = np.exp(-((d - 0.42) / 0.05) ** 2) * (0.35 + 0.65 * smooth(0.35, 0.7, around(a, 5.0, 153, 3)))
    soot = 1 - smooth(0.05, 0.36, d)
    scratch = smooth(0.55, 0.8, around(a, 16.0, 157, 3)) * smooth(0.4, 0.5, d) * (1 - smooth(0.6, 0.85, d))
    alpha = np.clip(np.maximum.reduce([rim * 0.8, soot * 0.85, scratch * 0.6]), 0, 1) * (1 - smooth(0.9, 0.99, r))
    tone = 40 + 130 * np.maximum(rim, scratch) * (1 - soot) + 10 * fbm(x * 20, y * 20, 163, 3)
    save("dent.png", np.dstack([tone, tone * 0.99, tone * 0.97]), alpha)
    save("dent-normal.png", normal_map(-(1 - smooth(0.0, 0.5, d)) + 0.25 * rim, 0.3))


def wet(size=256):
    """A splash's wet patch: a blotch with lobes thrown outwards and
    separate drops past them."""
    x, y, r, a = grid(size)
    lobes = 0.42 + 0.3 * around(a, 2.2, 167, 3) + 0.12 * around(a, 7.0, 173, 2)
    body = 1 - smooth(lobes - 0.12, lobes, r + (fbm(x * 6, y * 6, 179, 3) - 0.5) * 0.12)
    drops = smooth(0.74, 0.8, fbm(x * 17, y * 17, 181, 2)) * (1 - smooth(0.55, 0.95, r)) * smooth(0.25, 0.5, r)
    alpha = np.clip(np.maximum(body, drops), 0, 1) * (1 - smooth(0.9, 0.99, r))
    # Near white: a wet decal multiplies the surface by it, so the effect's
    # `color` alone says how much darker the wet ground is.
    tone = np.full_like(r, 236.0)
    save("wet.png", np.dstack([tone, tone * 1.01, tone * 1.03]), alpha)


def sigil(size=512):
    """A drawn circle: two rings, ticks and short strokes between them, a
    six-pointed star and an inner ring, each line with a faint halo."""
    x, y, r, a = grid(size)

    def line(d, w):
        return np.maximum(1 - smooth(w * 0.5, w, d), 0.22 * np.exp(-(d / (w * 5)) ** 2))

    def tri(turn):
        d = np.full_like(r, 9.0)
        for k in range(3):
            t = turn + k * 2 * np.pi / 3
            d = np.minimum(d, np.abs(x * np.cos(t) + y * np.sin(t) - 0.36))
        return d

    rings = np.maximum.reduce([line(np.abs(r - 0.9), 0.009), line(np.abs(r - 0.76), 0.006), line(np.abs(r - 0.2), 0.006)])
    ticks = (1 - smooth(0.02, 0.05, np.abs(np.sin(a * 12)))) * smooth(0.775, 0.785, r) * (1 - smooth(0.845, 0.855, r))
    seg = np.floor((a + np.pi) / (2 * np.pi) * 36)
    on = np.random.default_rng(191).random(36)[seg.astype(int) % 36] > 0.45
    strokes = line(np.abs(r - 0.83 - 0.015 * np.sin(a * 36 * 2)), 0.004) * on * (np.abs(np.sin(a * 18)) > 0.35)
    star = line(np.minimum(tri(0.0), tri(np.pi)), 0.005) * (1 - smooth(0.72, 0.75, r))
    alpha = np.clip(np.maximum.reduce([rings, ticks * 0.9, strokes * 0.8, star]), 0, 1)
    tone = np.full_like(r, 255.0)
    save("sigil.png", np.dstack([tone, tone, tone]), alpha)


def streak(w=256, h=32):
    """A beam's or a tracer's body: a hot core and a soft edge across it,
    faint unevenness along it."""
    u, v = np.meshgrid((np.arange(w) + 0.5) / w, (np.arange(h) + 0.5) / h * 2 - 1)
    core = np.exp(-(v / 0.16) ** 2)
    halo = np.exp(-(v / 0.5) ** 2) * (1 - smooth(0.8, 1.0, np.abs(v)))
    uneven = 0.85 + 0.15 * value_noise(lattice(193), u * 16, v * 0 + 0.5)
    alpha = np.clip((core + 0.4 * halo) * uneven, 0, 1)
    tone = np.full_like(u, 255.0)
    save("streak.png", np.dstack([tone, tone, tone]), alpha)


def taper(w=128, h=4):
    """A swing's falloff along its arc (a mask, read in red): full just
    behind the leading end at u 0, gone at the tail."""
    u = np.tile((np.arange(w) + 0.5) / w, (h, 1))
    tone = smooth(0.0, 0.07, u) * (1 - u) ** 1.6 * 255
    save("taper.png", np.dstack([tone, tone, tone]))


def leaves(size=256):
    """Four fallen leaves, 2 x 2: a pointed blade with a midrib and veins,
    each its own shape and its own dry colour."""
    cell = size // 2
    c = (np.arange(cell) + 0.5) / cell * 2 - 1
    x, y = np.meshgrid(c, c)
    rgb = np.zeros((size, size, 3))
    alpha = np.zeros((size, size))
    tints = [(150, 96, 40), (120, 110, 48), (164, 70, 34), (96, 82, 44)]
    for k in range(4):
        wide = [0.42, 0.5, 0.36, 0.46][k]
        bend = [0.12, -0.1, 0.05, -0.16][k]
        yy = y / 0.86
        xx = x - bend * (1 - yy * yy)
        # A blade: widest below the middle, a point at the tip.
        half = wide * np.clip(1 - yy * yy, 0, 1) ** 0.75 * (1 - 0.35 * yy)
        ragged = (fbm(x * 9 + 4 * k, y * 9, 197 + k, 3) - 0.5) * 0.07
        blade = 1 - smooth(-0.03, 0.02, np.abs(xx) - half - ragged)
        blade *= (np.abs(yy) < 1)
        rib = 1 - smooth(0.01, 0.03, np.abs(xx))
        veins = (1 - smooth(0.03, 0.08, np.abs(np.sin((yy - np.abs(xx) * 1.3) * 11)))) * 0.5
        mottle = fbm(x * 6 + 9 * k, y * 6, 211 + k, 4)
        tone = 0.75 + 0.35 * mottle - 0.22 * np.maximum(rib, veins)
        oy, ox = (k // 2) * cell, (k % 2) * cell
        for ch in range(3):
            rgb[oy:oy + cell, ox:ox + cell, ch] = tints[k][ch] * tone
        alpha[oy:oy + cell, ox:ox + cell] = blade
    save("leaves.png", rgb, alpha)


def flames(cell=128, cols=8, rows=4):
    """A flame that loops, 8 x 4 frames: a body widest at its foot, torn
    into tongues by noise that climbs it and comes round after the last
    frame, coloured by how hot each point is. For additive particles."""
    frames = cols * rows
    c = (np.arange(cell) + 0.5) / cell
    x, y = np.meshgrid(c * 2 - 1, 1 - c)
    sheet = np.zeros((rows * cell, cols * cell, 4))

    def climbing(seed, freq, n, turns, t):
        # A lattice n cells high that climbs `turns` of its height a loop.
        g = lattice(seed, n)
        return value_noise(g, x * freq + 3.7, y * freq * 1.5 - t * n * turns)

    keys = np.array([0.0, 0.2, 0.45, 0.7, 1.0])
    ramp = np.array([[0, 0, 0], [0.55, 0.06, 0.0], [1.0, 0.36, 0.03], [1.0, 0.72, 0.22], [1.0, 0.93, 0.62]])
    for f in range(frames):
        t = f / frames
        big = climbing(223, 2.0, 4, 1, t)
        mid = climbing(227, 4.0, 8, 1, t) * 0.6 + climbing(229, 8.0, 16, 2, t) * 0.4
        fine = climbing(233, 16.0, 32, 2, t)
        sway = (big - 0.5) * 0.5 * y
        width = 0.5 * (1 - y) ** 0.55 + 0.04
        core = 1 - np.abs(x + sway) / width
        heat = core * 1.15 + (mid - 0.5) * (0.9 + 1.3 * y) + (fine - 0.5) * 0.4 - y * 0.8
        heat *= smooth(0.0, 0.06, y) * (1 - smooth(0.9, 1.0, y)) * (1 - smooth(0.85, 1.0, np.abs(x)))
        v = np.clip(heat / 1.35, 0, 1) ** 1.25
        rgb = np.stack([np.interp(v, keys, ramp[:, k]) for k in range(3)], -1)
        oy, ox = (f // cols) * cell, (f % cols) * cell
        sheet[oy:oy + cell, ox:ox + cell, :3] = rgb ** (1 / 2.2) * 255
        sheet[oy:oy + cell, ox:ox + cell, 3] = smooth(0.0, 0.45, heat)
    save("flames.png", sheet[..., :3], sheet[..., 3])


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    scorch()
    embers()
    chip()
    skid()
    gouge()
    fracture()
    dent()
    wet()
    sigil()
    streak()
    taper()
    leaves()
    flames()
