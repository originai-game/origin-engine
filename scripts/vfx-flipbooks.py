#!/usr/bin/env python3
"""Offline flipbooks for the 3D effect packs: fire, smoke and explosions
simulated as a small 3D fluid and rendered as sheets of frames.

    python3 scripts/vfx-flipbooks.py <out-dir> [name ...]

Real-time volumetric simulation is out of the engine's scope; this is the
offline step that EmberGen or Houdini does for a commercial game, written
small enough to rerun. Each effect is a stable-fluids solve (Stam 1999:
semi-Lagrangian advection, a Jacobi pressure projection) on a 64^3 grid with
buoyancy from temperature and vorticity confinement (Fedkiw, Stam and
Jensen 2001), raymarched along the view axis with single scattering from
one light (a shadow volume marched toward it) and blackbody emission
from temperature. Seeded: the same arguments write the same bytes.

Sheets are 8x8 frames of 128 px (1024^2), straight alpha: smoke is
lit colour over alpha (draw it alpha-blended), fire and explosion cores are
emission (their alpha is the smoke's cover, draw them additive or
premultiplied). A stylised variant of each bands the same frames into
three tones with a hard alpha cut.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter, map_coordinates

N = 64
FRAMES = 64
CELL = 128
GRID = 8


def advect(field, u, v, w, dt):
    """Semi-Lagrangian advection of a scalar on the cell centres."""
    z, y, x = np.meshgrid(np.arange(N), np.arange(N), np.arange(N), indexing="ij")
    coords = np.array([z - dt * w, y - dt * v, x - dt * u])
    return map_coordinates(field, coords, order=1, mode="constant", cval=0.0)


def project(u, v, w, iters=30):
    """Make the velocity divergence-free (Jacobi on the pressure Poisson equation)."""
    div = np.zeros_like(u)
    div[1:-1, 1:-1, 1:-1] = -0.5 * (
        u[1:-1, 1:-1, 2:] - u[1:-1, 1:-1, :-2]
        + v[1:-1, 2:, 1:-1] - v[1:-1, :-2, 1:-1]
        + w[2:, 1:-1, 1:-1] - w[:-2, 1:-1, 1:-1]
    )
    p = np.zeros_like(u)
    for _ in range(iters):
        p[1:-1, 1:-1, 1:-1] = (
            div[1:-1, 1:-1, 1:-1]
            + p[1:-1, 1:-1, 2:] + p[1:-1, 1:-1, :-2]
            + p[1:-1, 2:, 1:-1] + p[1:-1, :-2, 1:-1]
            + p[2:, 1:-1, 1:-1] + p[:-2, 1:-1, 1:-1]
        ) / 6.0
    u[1:-1, 1:-1, 1:-1] -= 0.5 * (p[1:-1, 1:-1, 2:] - p[1:-1, 1:-1, :-2])
    v[1:-1, 1:-1, 1:-1] -= 0.5 * (p[1:-1, 2:, 1:-1] - p[1:-1, :-2, 1:-1])
    w[1:-1, 1:-1, 1:-1] -= 0.5 * (p[2:, 1:-1, 1:-1] - p[:-2, 1:-1, 1:-1])


def vorticity(u, v, w, eps, dt):
    """Vorticity confinement: puts back the small curls advection smears out."""
    g = lambda f, a: np.gradient(f, axis=a)
    wx = g(w, 1) - g(v, 0)
    wy = g(u, 0) - g(w, 2)
    wz = g(v, 2) - g(u, 1)
    mag = np.sqrt(wx * wx + wy * wy + wz * wz) + 1e-6
    nx, ny, nz = g(mag, 2), g(mag, 1), g(mag, 0)
    nl = np.sqrt(nx * nx + ny * ny + nz * nz) + 1e-6
    nx, ny, nz = nx / nl, ny / nl, nz / nl
    # f = eps (N x omega); axes: x=2, y=1 (up is +y index), z=0
    u += dt * eps * (ny * wz - nz * wy)
    v += dt * eps * (nz * wx - nx * wz)
    w += dt * eps * (nx * wy - ny * wx)


def blackbody(t):
    """Fire colour from a 0..1 temperature: deep red, orange, yellow, white."""
    t = np.clip(t, 0, 1)[..., None]
    stops = np.array([
        [0.0, 0.0, 0.0],
        [0.55, 0.05, 0.01],
        [1.6, 0.42, 0.06],
        [3.2, 1.6, 0.35],
        [5.0, 4.2, 2.6],
    ])
    x = t[..., 0] * (len(stops) - 1)
    i = np.clip(np.floor(x).astype(int), 0, len(stops) - 2)
    f = (x - i)[..., None]
    return stops[i] * (1 - f) + stops[i + 1] * f


def render(d, temp, smoke_col, light=(0.8, 1.0, 0.05), absorb=6.0, emit=1.0):
    """Raymarch along +z (toward the viewer) with one shadowing light.

    Returns straight-alpha RGBA (float, HDR before the sheet tone map).
    The y axis of the grid is up; rows are flipped for the image.
    """
    sigma = absorb * d / N * 8.0
    # Shadow toward the light: march the cumulative optical depth along the
    # light's dominant axes (up and to the side), smoothed for soft shadows.
    up = np.cumsum(sigma[:, ::-1, :], axis=1)[:, ::-1, :] * light[1]
    side = np.cumsum(sigma[:, :, ::-1], axis=2)[:, :, ::-1] * light[0]
    front = np.cumsum(sigma[::-1, :, :], axis=0)[::-1, :, :] * light[2]
    shadow = np.exp(-gaussian_filter(up + side + front, 1.2))
    ambient = 0.16 + 0.12 * (np.arange(N)[None, :, None] / N)
    lit = (shadow * 1.0 + ambient)[..., None] * np.array(smoke_col)
    e = blackbody(temp) * emit
    # Front-to-back along z (index 0 is far, N-1 near the camera).
    trans = np.ones((N, N))
    col = np.zeros((N, N, 3))
    step_t = np.exp(-sigma)
    for k in range(N):
        a = 1.0 - step_t[k]
        col += trans[..., None] * (a[..., None] * lit[k] + e[k] * (1.0 / N) * 8.0)
        trans = trans * step_t[k]
    alpha = 1.0 - trans
    rgb = np.where(alpha[..., None] > 1e-4, col / np.maximum(alpha[..., None], 1e-4), 0.0)
    # Pure emission where there is no smoke (thin flame): keep it visible.
    glow = np.clip(col - rgb * alpha[..., None], 0, None)
    return rgb[::-1], alpha[::-1], glow[::-1]


def tone(x):
    """ACES-ish filmic curve to fit HDR into 8 bits."""
    a, b, c, d, e = 2.51, 0.03, 2.43, 0.59, 0.14
    return np.clip((x * (a * x + b)) / (x * (c * x + d) + e), 0, 1)


def simulate(kind, seed):
    rng = np.random.default_rng(seed)
    u = np.zeros((N, N, N)); v = np.zeros_like(u); w = np.zeros_like(u)
    d = np.zeros_like(u); t = np.zeros_like(u)
    z, y, x = np.meshgrid(np.arange(N), np.arange(N), np.arange(N), indexing="ij")
    cx = cz = N / 2
    noise = gaussian_filter(rng.standard_normal((N, N, N)), 2.0)
    noise /= np.abs(noise).max()
    params = {
        "fire": dict(steps=2, buoy=0.55, cool=0.035, fade=0.985, eps=1.6, smoke=(0.25, 0.22, 0.2), absorb=2.0, emit=1.4),
        "smoke": dict(steps=2, buoy=0.55, cool=0.012, fade=0.996, eps=1.4, smoke=(0.78, 0.78, 0.8), absorb=7.0, emit=0.0),
        "explosion": dict(steps=2, buoy=0.9, cool=0.011, fade=0.996, eps=1.6, smoke=(0.32, 0.29, 0.27), absorb=6.0, emit=1.6),
        "dust": dict(steps=2, buoy=0.15, cool=0.02, fade=0.994, eps=1.0, smoke=(0.62, 0.52, 0.4), absorb=5.0, emit=0.0),
    }[kind]
    edge = np.minimum.reduce([x, N - 1 - x, z, N - 1 - z, N - 1 - y]).astype(float)
    sponge = np.clip(edge / 8.0, 0, 1) ** 0.25
    frames = []
    total = FRAMES * params["steps"]
    for s in range(total):
        f = s / total
        dt = 0.9
        if kind == "fire":
            # A steady base of fuel: a flickering disc near the floor.
            r = np.sqrt((x - cx) ** 2 + (z - cz) ** 2)
            jitter = 1.0 + 0.4 * np.sin(s * 0.37 + noise[:, 6, :] * 6)
            src = (r < 10.0 * jitter + 2.5 * noise) & (y > 3) & (y < 8)
            t[src] = np.maximum(t[src], 0.95)
            d[src] = np.maximum(d[src], 0.25)
            v[src] = np.maximum(v[src], 0.5)
        elif kind == "smoke":
            if s < total * 0.5:
                r = np.sqrt((x - cx) ** 2 + (y - 8) ** 2 + (z - cz) ** 2)
                src = r < 5.5 + 2.5 * noise
                d[src] = np.maximum(d[src], 0.9 + 0.1 * noise[src])
                t[src] = np.maximum(t[src], 0.5)
                v[src] = np.maximum(v[src], 0.45)
        elif kind == "explosion":
            if s == 0:
                r = np.sqrt((x - cx) ** 2 + (y - 24) ** 2 + (z - cz) ** 2)
                src = r < 7 + 2.5 * noise
                d[src] = 1.0
                t[src] = 1.0
                k = np.where(src, 0.9 / (r + 1.0), 0.0)
                u += (x - cx) * k; v += (y - 24) * k; w += (z - cz) * k
        elif kind == "dust":
            if s == 0:
                r = np.sqrt((x - cx) ** 2 + (z - cz) ** 2)
                src = (r < 9) & (y < 8)
                d[src] = 0.9
                k = np.where(src, 2.2 / (r + 1.0), 0.0)
                u += (x - cx) * k; w += (z - cz) * k; v += np.where(src, 0.4, 0)
        v += dt * params["buoy"] * (t - 0.05 * d) * 0.45
        # Turbulence: a smooth random force field that drifts with time, so
        # the plume breaks into billows instead of rising as one blob.
        if s % 4 == 0:
            turb = [gaussian_filter(rng.standard_normal((N, N, N)), 3.0) for _ in range(3)]
            turb = [f / (np.abs(f).max() + 1e-6) for f in turb]
        u += dt * 0.35 * turb[0] * (d > 0.02)
        v += dt * 0.2 * turb[1] * (d > 0.02)
        w += dt * 0.35 * turb[2] * (d > 0.02)
        u *= 0.995; v *= 0.995; w *= 0.995
        vorticity(u, v, w, params["eps"], dt)
        project(u, v, w)
        u, v, w = advect(u, u, v, w, dt), advect(v, u, v, w, dt), advect(w, u, v, w, dt)
        project(u, v, w)
        d = advect(d, u, v, w, dt) * params["fade"]
        # A sponge under every face: what reaches the edge thins out rather
        # than piling up against the wall.
        d *= sponge; t *= sponge; v *= sponge
        t = np.maximum(advect(t, u, v, w, dt) - params["cool"] * dt * (0.3 + t), 0)
        if (s + 1) % params["steps"] == 0:
            rgb, a, glow = render(d, t, params["smoke"], absorb=params["absorb"], emit=params["emit"])
            frames.append((rgb, a, glow, f))
    return frames


def sheet(frames, stylised):
    img = np.zeros((GRID * CELL, GRID * CELL, 4))
    for i, (rgb, a, glow, _) in enumerate(frames):
        # Up-sample the 64^2 render to the cell with a cubic filter, then a
        # little blur against the grid's blockiness.
        ch = []
        for c in range(3):
            ch.append(Image.fromarray((rgb[..., c]).astype(np.float32), "F").resize((CELL, CELL), Image.BICUBIC))
        col = np.stack([np.asarray(c) for c in ch], -1)
        gl = np.stack([np.asarray(Image.fromarray(glow[..., c].astype(np.float32), "F").resize((CELL, CELL), Image.BICUBIC)) for c in range(3)], -1)
        al = np.asarray(Image.fromarray(a.astype(np.float32), "F").resize((CELL, CELL), Image.BICUBIC))
        al = np.clip(al, 0, 1)
        # Fold the emission into straight colour where there is cover, and
        # into alpha where there is only light (a flame with no smoke).
        lum = gl.max(-1)
        a2 = np.clip(np.maximum(al, np.clip(lum, 0, 1)), 0, 1)
        col = (col * al[..., None] + gl) / np.maximum(a2[..., None], 1e-4)
        col = tone(col)
        if stylised:
            l = col.mean(-1, keepdims=True)
            band = np.where(l > 0.62, 1.0, np.where(l > 0.3, 0.62, 0.32))
            col = np.clip(col / np.maximum(l, 1e-3) * band, 0, 1)
            a2 = np.where(a2 > 0.35, 1.0, 0.0) * np.clip((a2 - 0.25) * 6, 0, 1)
        # Fade the edge of each cell so nothing clips at the cell border.
        yy, xx = np.mgrid[0:CELL, 0:CELL] / (CELL - 1)
        edge = np.clip(np.minimum.reduce([xx, 1 - xx, yy, 1 - yy]) * 14, 0, 1)
        a2 = a2 * edge
        gy, gx = divmod(i, GRID)
        img[gy * CELL:(gy + 1) * CELL, gx * CELL:(gx + 1) * CELL, :3] = col
        img[gy * CELL:(gy + 1) * CELL, gx * CELL:(gx + 1) * CELL, 3] = a2
    return Image.fromarray((np.clip(img, 0, 1) * 255 + 0.5).astype(np.uint8), "RGBA")


def main():
    out = Path(sys.argv[1])
    names = sys.argv[2:] or ["fire", "smoke", "explosion", "dust"]
    out.mkdir(parents=True, exist_ok=True)
    for i, name in enumerate(names):
        frames = simulate(name, seed=1000 + i)
        for stylised in (False, True):
            path = out / f"{name}{'_toon' if stylised else ''}_8x8.png"
            sheet(frames, stylised).save(path, optimize=True, compress_level=9)
            print(path)


if __name__ == "__main__":
    main()
