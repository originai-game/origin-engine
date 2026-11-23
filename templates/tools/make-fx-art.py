#!/usr/bin/env python3
"""Draws the UI effects' particle textures (CC0), byte for byte, in
templates/styles/shared/fx/: white masks with alpha, 128 px, that the
element's fill colours (`nine-fx-sparkle bg-gold`).

    python3 templates/tools/make-fx-art.py [repo]

Needs numpy and Pillow.
"""
import os, sys
import numpy as np
from PIL import Image


def fx_canvas():
    n = 128
    ys, xs = np.mgrid[0:n, 0:n]
    return (xs + 0.5) / n * 2 - 1, (ys + 0.5) / n * 2 - 1


def fx_save(path, grey, alpha):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    g = np.clip(grey, 0, 1)
    a = np.clip(alpha, 0, 1)
    px = np.round(np.stack([g, g, g, a], -1) * 255).astype(np.uint8)
    Image.fromarray(px, "RGBA").save(path, optimize=True)


def fx_art(out):
    x, y = fx_canvas()
    r = np.hypot(x, y)
    edge = np.clip((1 - r) / 0.08, 0, 1)
    # sparkle: two crossed tapering beams, a smaller diagonal pair, a core
    beam = lambda u, v: np.exp(-np.abs(v) / (0.035 + 0.05 * np.clip(1 - np.abs(u), 0, 1))) * np.clip(1 - np.abs(u), 0, 1) ** 1.6
    star = np.maximum(beam(x, y), beam(y, x))
    d1, d2 = (x + y) / np.sqrt(2), (x - y) / np.sqrt(2)
    star = np.maximum(star, 0.45 * np.maximum(beam(d1 * 1.8, d2), beam(d2 * 1.8, d1)))
    core = np.exp(-(r / 0.16) ** 2)
    halo = np.exp(-(r / 0.42) ** 2) * 0.35
    a = np.clip(star + core + halo, 0, 1) * edge
    fx_save(os.path.join(out, "sparkle.png"), np.ones_like(a), a)
    # glint
    a = np.clip(np.exp(-(r / 0.22) ** 2) + np.exp(-(r / 0.6) ** 2) * 0.45, 0, 1) * edge
    fx_save(os.path.join(out, "glint.png"), np.ones_like(a), a)
    # shard: a tall rhombus, its left facets lit
    q = np.abs(x) / 0.42 + np.abs(y) / 0.92
    inside = np.clip((1 - q) * 30, 0, 1)
    lit = np.where(x < 0, 1.0, 0.62) * np.where(y < 0, 1.0, 0.85)
    rimline = np.clip(1 - np.abs(q - 0.93) * 18, 0, 1)
    grey = np.clip(lit + rimline * 0.5, 0, 1)
    fx_save(os.path.join(out, "shard.png"), grey, inside)
    # coin: a rim, a sunk face, a lit stamp and a gloss
    disc = np.clip((0.94 - r) * 40, 0, 1)
    rim = np.clip((r - 0.74) * 40, 0, 1)
    shade = 0.78 - 0.28 * y
    stamp = np.clip(1 - (np.abs(x) / 0.28 + np.abs(y) / 0.42), 0, 1) > 0
    grey = np.where(rim > 0.5, 0.96 - 0.3 * (y + 1) / 2, shade)
    grey = np.where(stamp, 1.0, grey)
    grey = grey + np.exp(-(((x + 0.32) / 0.22) ** 2 + ((y + 0.38) / 0.16) ** 2)) * 0.35
    fx_save(os.path.join(out, "coin.png"), grey, disc)
    # ray: a soft vertical beam, brightest in the middle
    a = np.exp(-(x / 0.16) ** 2) * np.clip(1 - np.abs(y), 0, 1) ** 0.8 + np.exp(-(x / 0.04) ** 2) * np.clip(1 - np.abs(y), 0, 1) * 0.6
    fx_save(os.path.join(out, "ray.png"), np.ones_like(a), np.clip(a, 0, 1))
    # ring
    a = np.exp(-((r - 0.82) / 0.035) ** 2) + np.exp(-((r - 0.82) / 0.14) ** 2) * 0.4
    fx_save(os.path.join(out, "ring.png"), np.ones_like(a), np.clip(a, 0, 1) * edge)


if __name__ == "__main__":
    repo = sys.argv[1] if len(sys.argv) > 1 else "."
    fx_art(os.path.join(repo, "templates/styles/shared/fx"))
