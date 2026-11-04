#!/usr/bin/env python3
"""The demolition yard's textures, drawn from seeded noise (CC0, the engine's).

    python3 examples/demolition-yard/tools/textures.py

writes assets/textures/*.png: each a tile a metre wide (the props' uvs
repeat once a metre), albedo in sRGB and normal maps in glTF's tangent
frame (red towards +u, green up the image). The brick tile is the running
bond of the wall's `fracture { brick 0.25 0.1 }`: 4 bricks a course, 10
courses, the bottom course starting at a joint, so the cracks follow the
mortar drawn here.
"""
import os
import numpy as np
from PIL import Image

OUT = os.path.join(os.path.dirname(__file__), "..", "assets", "textures")
rng = np.random.default_rng(7)


def noise(n, scale, octaves=4):
    """Tileable value noise in 0..1 on an n×n grid, `scale` cells across."""
    out = np.zeros((n, n))
    amp, total = 1.0, 0.0
    for o in range(octaves):
        cells = scale * 2**o
        g = rng.random((cells, cells))
        x = np.arange(n) * cells / n
        i = np.floor(x).astype(int)
        f = x - i
        f = f * f * (3 - 2 * f)
        i0, i1 = i % cells, (i + 1) % cells
        a = g[np.ix_(i0, i0)] * (1 - f)[None, :] + g[np.ix_(i0, i1)] * f[None, :]
        b = g[np.ix_(i1, i0)] * (1 - f)[None, :] + g[np.ix_(i1, i1)] * f[None, :]
        out += amp * (a * (1 - f)[:, None] + b * f[:, None])
        total += amp
        amp *= 0.5
    return out / total


def normal_map(h, strength):
    """A tangent-space normal map of height `h` (0..1, image rows down)."""
    dx = (np.roll(h, -1, 1) - np.roll(h, 1, 1)) * 0.5 * strength
    dy = (np.roll(h, -1, 0) - np.roll(h, 1, 0)) * 0.5 * strength
    # Up the image is +v: a height rising down the rows tilts the normal up.
    n = np.stack([-dx, dy, np.ones_like(h)], -1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    return n * 0.5 + 0.5


def save(name, rgb, srgb=True):
    rgb = np.clip(rgb, 0, 1)
    if srgb:
        rgb = np.where(rgb <= 0.0031308, rgb * 12.92, 1.055 * rgb ** (1 / 2.4) - 0.055)
    Image.fromarray((rgb * 255 + 0.5).astype(np.uint8)).save(os.path.join(OUT, name), optimize=True)


def brick():
    n = 1024
    courses, per = 10, 4
    ch = n // courses  # course height in px
    bw = n // per
    y = np.arange(n)
    # Rows from the bottom: course 0 is the image's last rows.
    up = n - 1 - y
    course = up * courses // n
    yin = up - course * n // courses
    x = np.arange(n)
    off = np.where(course % 2 == 1, bw // 2, 0)
    xs = (x[None, :] + off[:, None]) % n
    bi = xs // bw
    xin = xs % bw
    mortar_px = 6
    edge = np.minimum(np.minimum(xin, bw - 1 - xin), np.minimum(yin, ch - 1 - yin)[:, None])
    in_brick = edge >= mortar_px // 2
    # Per brick: a colour from a red-brown family.
    ids = course[:, None] * per + bi
    hue = rng.random(courses * per)
    base = np.array([[0.42, 0.13, 0.07], [0.5, 0.2, 0.1], [0.36, 0.12, 0.08], [0.55, 0.26, 0.15], [0.3, 0.1, 0.06]])
    pick = base[(hue * len(base)).astype(int) % len(base)]
    tint = 0.85 + 0.3 * rng.random(courses * per)
    col = pick[ids] * tint[ids][..., None]
    grain = noise(n, 24, 5)
    speck = (noise(n, 96, 2) > 0.72).astype(float)
    col = col * (0.75 + 0.45 * grain[..., None]) * (1 - 0.25 * speck[..., None])
    # Burnt ends: darker towards a brick's edges.
    col *= (0.82 + 0.18 * np.clip(edge / 18.0, 0, 1))[..., None]
    mortar = np.array([0.3, 0.29, 0.27]) * (0.75 + 0.4 * noise(n, 64, 3))[..., None]
    rgb = np.where(in_brick[..., None], col, mortar)
    save("brick.png", rgb)
    # Height: bricks proud of the mortar with rounded arrises, pitted faces.
    h = np.clip((edge - mortar_px // 2 + 1) / 6.0, 0, 1) ** 0.6 * 0.8
    h += 0.12 * grain - 0.15 * speck * in_brick
    h = np.where(in_brick, h, 0.05 * noise(n, 64, 3))
    save("brick_n.png", normal_map(h, 14.0), srgb=False)


def brick_interior():
    n = 512
    g = noise(n, 12, 5)
    crumbs = noise(n, 80, 2)
    rgb = np.array([0.36, 0.27, 0.22]) * (0.8 + 0.3 * g[..., None])
    # Lighter dust and darker grit.
    rgb = np.where((crumbs > 0.7)[..., None], rgb * 1.25 + 0.05, rgb)
    rgb = np.where((crumbs < 0.22)[..., None], rgb * 0.7, rgb)
    save("brick_cut.png", rgb)
    h = 0.6 * g + 0.4 * crumbs
    save("brick_cut_n.png", normal_map(h, 10.0), srgb=False)


def concrete():
    n = 512
    g = noise(n, 8, 6)
    stains = noise(n, 3, 3)
    rgb = np.array([0.3, 0.29, 0.27]) * (0.85 + 0.2 * g[..., None]) * (0.85 + 0.2 * stains[..., None])
    pores = (noise(n, 120, 1) > 0.8)
    rgb = np.where(pores[..., None], rgb * 0.7, rgb)
    save("concrete.png", rgb)
    save("concrete_n.png", normal_map(0.5 * g - 0.3 * pores, 6.0), srgb=False)


def aggregate():
    """A concrete cut: grey cement with stones of gravel, light and dark."""
    n = 512
    rgb = np.array([0.3, 0.29, 0.27]) * (0.85 + 0.25 * noise(n, 16, 5)[..., None])
    h = 0.2 * noise(n, 16, 4)
    yy, xx = np.mgrid[0:n, 0:n]
    for _ in range(260):
        cx, cy = rng.random(2) * n
        r = 4 + rng.random() ** 2 * 16
        ax = 0.6 + rng.random() * 0.8
        t = rng.random() * np.pi
        dx = (xx - cx + n / 2) % n - n / 2
        dy = (yy - cy + n / 2) % n - n / 2
        u = (dx * np.cos(t) + dy * np.sin(t)) / (r * ax)
        v = (-dx * np.sin(t) + dy * np.cos(t)) / (r / ax)
        d = u * u + v * v
        m = d < 1
        stone = [np.array([0.16, 0.155, 0.15]), np.array([0.42, 0.38, 0.33]), np.array([0.25, 0.22, 0.19]), np.array([0.1, 0.1, 0.11])][rng.integers(4)]
        rgb[m] = stone * (0.85 + 0.3 * rng.random())
        h[m] = 0.35 + 0.4 * np.sqrt(1 - d[m])
    save("concrete_cut.png", rgb)
    save("concrete_cut_n.png", normal_map(h, 8.0), srgb=False)


def planks():
    """The crate's faces: four boards across, grain along u, a frame."""
    n = 512
    boards = 4
    bh = n // boards
    y = np.arange(n)
    board = y // bh
    yin = y % bh
    x = np.arange(n)
    shift = rng.random(boards) * 7
    # Grain: stretched noise along u, with a wavy ring figure.
    streak = noise(n, 6, 5)
    streak = np.repeat(noise(n, 8, 4)[:, :1], n, 1) * 0.5 + streak * 0.5
    fine = np.zeros((n, n))
    for b in range(boards):
        rows = slice(b * bh, (b + 1) * bh)
        v = (yin[rows][:, None] / bh) * 14 + shift[b] + 0.5 * np.sin(x[None, :] / n * np.pi * 2 * (1 + b % 2) + shift[b])
        fine[rows] = 0.5 + 0.5 * np.sin(v * 2 * np.pi)
    tone = np.array([0.3, 0.18, 0.09]) * (0.85 + 0.3 * rng.random(boards))[board][:, None, None]
    rgb = tone * (0.72 + 0.14 * fine[..., None] + 0.3 * streak[..., None])
    gap = np.minimum(yin, bh - 1 - yin) < 3
    # The frame: darker battens round the face.
    rim = np.minimum(np.minimum(x[None, :], n - 1 - x[None, :]), np.minimum(y, n - 1 - y)[:, None]) < 34
    rgb = np.where(rim[..., None], rgb * 0.78, rgb)
    rgb = np.where(gap[:, None, None] & ~rim[..., None], rgb * 0.25, rgb)
    save("planks.png", rgb)
    h = 0.6 + 0.08 * fine + 0.05 * streak
    h = np.where(gap[:, None] & ~rim, 0.0, h)
    h = np.where(rim, h + 0.25, h)
    save("planks_n.png", normal_map(h, 10.0), srgb=False)


def wood_cut():
    n = 512
    x = np.arange(n)
    streak = noise(n, 4, 5)
    fibres = 0.5 + 0.5 * np.sin((x[None, :] / n * 40 + 3 * streak) * 2 * np.pi)
    rgb = np.array([0.5, 0.36, 0.2]) * (0.82 + 0.15 * fibres[..., None] + 0.12 * noise(n, 32, 3)[..., None])
    save("wood_cut.png", rgb)
    save("wood_cut_n.png", normal_map(0.3 * fibres + 0.3 * streak, 8.0), srgb=False)


def ground():
    """The yard: worn concrete slabs with dirt in their joints."""
    n = 1024
    g = noise(n, 6, 6)
    dirt = noise(n, 3, 4)
    rgb = np.array([0.19, 0.185, 0.17]) * (0.8 + 0.3 * g[..., None])
    rgb = rgb * (1 - 0.35 * np.clip(dirt - 0.5, 0, 1)[..., None] * 2) + np.array([0.06, 0.05, 0.03]) * np.clip(dirt - 0.55, 0, 1)[..., None] * 3
    yy, xx = np.mgrid[0:n, 0:n]
    joint = (np.minimum(xx % (n // 2), yy % (n // 2)) < 4)
    rgb = np.where(joint[..., None], rgb * 0.45, rgb)
    pebbles = noise(n, 160, 1) > 0.8
    rgb = np.where(pebbles[..., None], rgb * 0.8, rgb)
    save("ground.png", rgb)
    h = 0.5 + 0.3 * g - 0.4 * joint - 0.1 * pebbles
    save("ground_n.png", normal_map(h, 8.0), srgb=False)


os.makedirs(OUT, exist_ok=True)
for f in (brick, brick_interior, concrete, aggregate, planks, wood_cut, ground):
    f()
