#!/usr/bin/env python3
"""Regenerates Saltwind Isle's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/terrain/isle.png: a 257 x 257 16-bit grey heightmap of the 256 m
  square the isle stands in (one sample a metre; black -10 m, white 30 m,
  as terrain/isle.ogt says): the main island (a central highland, the
  watchtower's hill in the west, the windmill's meadow in the east, the
  camp's beach in the south) and, across a deep strait, the north isle
  with the well and the lighthouse's crag. The sea (og.Water at y 0) fills
  the rest.
- assets/terrain/isle-control.png: its 512 x 512 RGBA control map: grass in
  R (except on the paths), the paths in G (from the camp to every site and
  the bridge), rock in B and sand in A (both everywhere: their slope and
  altitude rules place them).
- assets/maps/isle.png: the island's map for the HUD's minimap (512 x 512:
  the sea's depth, the sand, the grass shaded by the hills, the rock and
  the paths), north up.
- assets/textures/item-wood.png, item-stone.png: the bag's two items.
- src/layout.ts: where the trees, the clusters of woods, the rocks, the
  sites, the camp and the bridge stand (positions only: the game samples
  the terrain for their heights).
- assets/audio/*.wav: the chop, the stone, the claim, the discovery, the
  bridge, the dawn and night stingers, the collapse, the splash, the jump,
  the surf's loop and the island's music; assets/audio/*.ogt: og sfx
  presets for the interface and the footsteps.

Standard library only; deterministic.
"""
import math, os, struct, sys, zlib

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_wav, tone, noise, mix, concat  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")

# The isle: keep these in step with src/layout.ts (written below) and terrain/isle.ogt.
LOW, HIGH = -10.0, 30.0
N = 257  # samples per side, one a metre
HALF = (N - 1) / 2
SEA = 0.0
CAMP = (0.0, 82.0)
# The five sites (src/layout.ts SITES order): x, z, the pad's radius.
SITES = [
    ("watchtower", -62.0, 18.0),
    ("windmill", 58.0, 40.0),
    ("chapel", -6.0, -6.0),
    ("well", -16.0, -78.0),
    ("lighthouse", 12.0, -100.0),
]
BRIDGE_X = 4.0


def write(path, data):
    path = os.path.join(root, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


# ── noise ──────────────────────────────────────────────────────────────────

def hash01(x, y, s=0):
    h = (x * 374761393 + y * 668265263 + s * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) / 0xFFFFFFFF


def smooth(t):
    return t * t * (3 - 2 * t)


def value(x, y, s):
    xi, yi = math.floor(x), math.floor(y)
    fx, fy = smooth(x - xi), smooth(y - yi)
    a, b = hash01(xi, yi, s), hash01(xi + 1, yi, s)
    c, d = hash01(xi, yi + 1, s), hash01(xi + 1, yi + 1, s)
    return a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy


def fbm(x, y, s, octaves=4):
    total, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        total += amp * value(x * (1 << o), y * (1 << o), s + o * 17)
        norm += amp
        amp *= 0.5
    return total / norm


def gauss(x, z, cx, cz, r):
    return math.exp(-((x - cx) ** 2 + (z - cz) ** 2) / (2 * r * r))


def step(a, b, t):
    return smooth(max(0.0, min(1.0, (t - a) / (b - a))))


# ── the terrain ────────────────────────────────────────────────────────────

def land(x, z):
    """Above 0 on land: the main island's and the north isle's blobs, their coasts ragged by noise."""
    m = 1 - math.sqrt((x / 100) ** 2 + ((z - 20) / 68) ** 2)
    n = 1 - math.sqrt(((x - 0) / 40) ** 2 + ((z + 85) / 26) ** 2)
    rag = 0.16 * (fbm(x / 22 + 11.3, z / 22 + 4.1, 3) - 0.5)
    return max(m, n) + rag


def raw_height(x, z):
    """Metres at (x, z) before the sites' pads."""
    l = land(x, z)
    h = -9 + 10.5 * step(-0.2, 0.08, l)
    inland = step(0.04, 0.34, l)
    rolling = 1.2 + 3.0 * fbm(x / 34 + 1.7, z / 34 + 9.2, 7)
    hills = (14 * gauss(x, z, -8, -6, 24)       # the central highland (the chapel)
             + 11 * gauss(x, z, -62, 18, 13)    # the watchtower's hill
             + 2.5 * gauss(x, z, 58, 40, 12)    # the windmill's rise
             + 8 * gauss(x, z, 12, -100, 10)     # the lighthouse's crag
             + 3 * gauss(x, z, -16, -78, 12)    # the well's rise
             + 6 * gauss(x, z, 40, -6, 16)      # an eastern down
             + 5 * gauss(x, z, -38, 54, 14))    # a south-western knoll
    ridge = 1 - abs(2 * fbm(x / 16, z / 16, 5) - 1)
    return h + inland * (rolling + hills * (0.85 + 0.25 * ridge))


def pad_height(name, sx, sz):
    return max(raw_height(sx, sz), 1.2)


PADS = [(sx, sz, pad_height(n, sx, sz), 6.0) for n, sx, sz in SITES] + [(CAMP[0], CAMP[1], max(raw_height(*CAMP), 1.0), 7.0)]


def height(x, z):
    """Metres at local (x, z), the terrain's centre at 0: the sites and the camp stand on level pads."""
    h = raw_height(x, z)
    for px, pz, ph, r in PADS:
        d = math.hypot(x - px, z - pz)
        k = 1 - step(r, r + 6, d)
        h = h + (ph - h) * k
    return h


def bridge_ends():
    """Where the bridge lands: the last metre of the main island's shore and the first of the north isle's, along BRIDGE_X."""
    z = 0.0
    while height(BRIDGE_X, z - 1) > 0.5:
        z -= 1
    south = z
    z -= 4
    while height(BRIDGE_X, z) < 0.5:
        z -= 1
    return south + 2.0, z - 2.0


def segments():
    """The paths: from the camp to the chapel and on to each site, and from the bridge on the north isle."""
    s, n = bridge_ends()
    nodes = {"camp": CAMP, "bridgeS": (BRIDGE_X, s), "bridgeN": (BRIDGE_X, n)}
    for name, x, z in SITES:
        nodes[name] = (x, z)
    pairs = [("camp", "chapel"), ("chapel", "watchtower"), ("chapel", "windmill"), ("chapel", "bridgeS"), ("bridgeN", "well"), ("well", "lighthouse"), ("camp", "windmill")]
    return [(nodes[a], nodes[b]) for a, b in pairs]


def path(x, z, segs):
    best = 1e9
    for (ax, az), (bx, bz) in segs:
        vx, vz = bx - ax, bz - az
        t = max(0.0, min(1.0, ((x - ax) * vx + (z - az) * vz) / (vx * vx + vz * vz)))
        # A path that wanders: bent sideways by a gentle wave along its length.
        L = math.hypot(vx, vz)
        bend = 3.0 * math.sin(t * math.pi) * math.sin(t * L / 9)
        px = ax + vx * t - vz / L * bend
        pz = az + vz * t + vx / L * bend
        best = min(best, math.hypot(x - px, z - pz))
    return max(0.0, 1 - best / 1.6)


def png16_grey(w, h, rows):
    raw = b"".join(b"\x00" + struct.pack(f">{w}H", *r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 16, 0, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def png_rgb(w, h, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def terrain(H):
    rows = [[max(0, min(65535, round((H[r][c] - LOW) / (HIGH - LOW) * 65535))) for c in range(N)] for r in range(N)]
    write("assets/terrain/isle.png", png16_grey(N, N, rows))
    segs = segments()
    M = 512
    rows = []
    for r in range(M):
        row = bytearray()
        for c in range(M):
            x = (c + 0.5) / M * (N - 1) - HALF
            z = (r + 0.5) / M * (N - 1) - HALF
            p = path(x, z, segs)
            row += bytes([round(255 * (1 - p)), round(255 * p), 255, 255])
        rows.append(bytes(row))
    write("assets/terrain/isle-control.png", png_bytes(M, M, rows))


def sample(H, x, z):
    """The heightmap bilinearly, as og.terrain.sample reads it."""
    fx, fz = x + HALF, z + HALF
    c, r = max(0, min(N - 2, int(math.floor(fx)))), max(0, min(N - 2, int(math.floor(fz))))
    tx, tz = fx - c, fz - r
    a, b = H[r][c], H[r][c + 1]
    d, e = H[r + 1][c], H[r + 1][c + 1]
    return a + (b - a) * tx + (d - a) * tz + (a - b - d + e) * tx * tz


def slope(H, x, z):
    gx = (sample(H, x + 1, z) - sample(H, x - 1, z)) / 2
    gz = (sample(H, x, z + 1) - sample(H, x, z - 1)) / 2
    return math.degrees(math.atan(math.hypot(gx, gz)))


def island_map(H):
    """The minimap's picture: 512 px for the 256 m square, north up."""
    M = 512
    segs = segments()
    rows = []
    for r in range(M):
        row = []
        for c in range(M):
            x = (c + 0.5) / M * (N - 1) - HALF
            z = (r + 0.5) / M * (N - 1) - HALF
            h = sample(H, x, z)
            if h < SEA:
                k = max(0.0, min(1.0, -h / 7))
                col = (0.36 - 0.22 * k, 0.70 - 0.30 * k, 0.78 - 0.18 * k)
                if h > -0.8:
                    col = (0.62, 0.84, 0.84)
            else:
                sl = slope(H, x, z)
                if h < 1.6:
                    col = (0.93, 0.85, 0.62)
                elif sl > 30:
                    col = (0.55, 0.53, 0.5)
                else:
                    g = min(1.0, h / 22)
                    col = (0.47 + 0.12 * g, 0.70 - 0.05 * g, 0.36 + 0.06 * g)
                # Light from the north-west.
                shade = (sample(H, x - 1, z - 1) - sample(H, x + 1, z + 1)) * 0.06
                col = tuple(v * (1 + shade) for v in col)
                p = path(x, z, segs)
                if p > 0.2:
                    col = tuple(a + (b - a) * min(1, p * 1.4) for a, b in zip(col, (0.82, 0.70, 0.5)))
            row += [round(max(0, min(1, v)) * 255) for v in col]
        rows.append(row)
    write("assets/maps/isle.png", png_rgb(M, M, rows))


# ── where things stand ─────────────────────────────────────────────────────

def layout(H):
    s, n = bridge_ends()
    keep = [(x, z, 11.0) for _, x, z in SITES] + [(CAMP[0], CAMP[1], 13.0), (BRIDGE_X, s, 9.0), (BRIDGE_X, n, 9.0)]
    segs = segments()

    def clear(x, z, r):
        return all(math.hypot(x - kx, z - kz) > kr + r for kx, kz, kr in keep) and path(x, z, segs) == 0.0

    trees, woods, rocks = [], [], []
    cell = 7.0
    i = 0
    gz = -HALF + cell / 2
    while gz < HALF:
        gx = -HALF + cell / 2
        while gx < HALF:
            i += 1
            x = gx + (hash01(int(gx), int(gz), 41) - 0.5) * cell * 0.8
            z = gz + (hash01(int(gx), int(gz), 43) - 0.5) * cell * 0.8
            h = sample(H, x, z)
            forest = fbm(x / 30 + 5.5, z / 30 + 2.2, 9)
            if 1.9 < h < 20 and slope(H, x, z) < 24 and clear(x, z, 2.5):
                if forest > 0.62 and hash01(int(gx), int(gz), 47) < 0.45:
                    woods.append((round(x, 2), round(z, 2), round(hash01(i, 1, 53) * 6.283, 3)))
                elif forest > 0.5 and hash01(int(gx), int(gz), 49) < 0.6:
                    trees.append((round(x, 2), round(z, 2), round(hash01(i, 2, 53) * 6.283, 3)))
            if 1.2 < h < 24 and hash01(int(gx), int(gz), 59) < 0.075 and clear(x, z, 1.5):
                rocks.append((round(x, 2), round(z, 2), round(hash01(i, 3, 53) * 6.283, 3)))
            gx += cell
        gz += cell
    fmt = lambda rows: "".join(f"  [{x}, {z}, {a}],\n" for x, z, a in rows)  # noqa: E731
    sites = "".join(f"  {{ name: \"{nm}\", x: {x}, z: {z} }},\n" for nm, x, z in SITES)
    write("src/layout.ts", f"""// Where Saltwind Isle's things stand: written by tools/make-assets.py
// (deterministic) beside the heightmap it places them on; the game samples
// the terrain for each one's height. Edit the script, not this file.

/** The sea's surface (og.Water at y 0) and the terrain's size. */
export const SEA = {SEA};
export const ISLE_SIZE = {N - 1};
/** The camp on the south beach: where a run starts and a fainted seat wakes. */
export const CAMP = {{ x: {CAMP[0]}, z: {CAMP[1]} }};
/** The five sites, west to east on the main island, then the north isle's two. */
export const SITES: readonly {{ name: string; x: number; z: number }}[] = [
{sites}];
/** The bridge across the strait: its x and the ends of its span (z, south then north). */
export const BRIDGE = {{ x: {BRIDGE_X}, south: {s}, north: {n} }};
/** Trees a seat can fell for wood: [x, z, heading]. */
export const TREES: readonly (readonly [number, number, number])[] = [
{fmt(trees)}];
/** Deep woods (a clump of trees, scenery): [x, z, heading]. */
export const WOODS: readonly (readonly [number, number, number])[] = [
{fmt(woods)}];
/** Boulders a seat can break for stone: [x, z, heading]. */
export const ROCKS: readonly (readonly [number, number, number])[] = [
{fmt(rocks)}];
""")
    return len(trees), len(woods), len(rocks), s, n


# ── the bag's items ────────────────────────────────────────────────────────

def items():
    S = 96
    wood = Canvas(S, S)
    for i, (cx, cy) in enumerate([(30, 60), (60, 62), (45, 38)]):
        wood.poly([(cx - 26, cy - 8), (cx + 22, cy - 14), (cx + 26, cy + 4), (cx - 22, cy + 10)], "#7a4c29")
        wood.ellipse(cx + 23, cy - 5, 8, 10, "#dbad6b")
        wood.ring(cx + 23, cy - 5, 4.5, 1.6, "#9e6b38")
        wood.line(cx - 20, cy - 2, cx + 14, cy - 8, 2, "#5c381f")
    write("assets/textures/item-wood.png", png_bytes(S, S, wood.rgba()))
    stone = Canvas(S, S)
    stone.poly([(16, 66), (24, 34), (48, 20), (74, 28), (84, 56), (70, 78), (34, 80)], "#858a8f")
    stone.poly([(24, 34), (48, 20), (74, 28), (56, 44), (34, 46)], "#b8bdc2")
    stone.poly([(56, 44), (74, 28), (84, 56), (66, 60)], "#999ea3")
    stone.line(36, 58, 58, 64, 2, "#61666b")
    write("assets/textures/item-stone.png", png_bytes(S, S, stone.rgba()))


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def music():
    """Sixteen bars at 96 BPM in D major: a soft pad, a lilting plucked figure and a whistled tune over it, looped seamlessly."""
    beat = 60 / 96
    bars = 16
    total = int(bars * 4 * beat * RATE)
    out = [0.0] * total
    chords = [(146.83, 185.0, 220.0), (196.0, 246.94, 293.66), (164.81, 220.0, 277.18), (146.83, 185.0, 220.0)]
    for i in range(total):
        t = i / RATE
        ch = chords[int(t / (4 * beat * 4)) % 4]
        swell = 0.55 + 0.45 * math.sin(2 * math.pi * t / (8 * beat))
        out[i] += 0.022 * swell * sum(math.sin(2 * math.pi * f * t) for f in ch)
    for b in range(bars * 4):
        ch = chords[(b // 16) % 4]
        for k, off in enumerate((0.0, 0.5)):
            f = ch[(b + k) % 3] * (2 if k else 1)
            s0 = int((b + off) * beat * RATE)
            for i in range(int(0.5 * beat * RATE)):
                t = i / RATE
                out[(s0 + i) % total] += 0.045 * math.exp(-t / 0.18) * (math.sin(2 * math.pi * f * t) + 0.25 * math.sin(2 * math.pi * 3 * f * t))
    tune = [(0, 587.33, 2), (2, 659.25, 1), (3, 739.99, 1), (4, 880.0, 3), (8, 783.99, 2), (10, 739.99, 2), (12, 659.25, 4),
            (16, 587.33, 2), (18, 659.25, 2), (20, 739.99, 3), (24, 659.25, 2), (26, 587.33, 2), (28, 554.37, 4),
            (36, 739.99, 2), (38, 783.99, 2), (40, 880.0, 4), (48, 987.77, 2), (50, 880.0, 2), (52, 739.99, 4), (58, 587.33, 6)]
    for b, f, d in tune:
        s0 = int(b * beat * RATE)
        n = int(d * beat * RATE)
        for i in range(n):
            t = i / RATE
            env = min(1.0, t / 0.06) * math.exp(-t / (d * beat * 0.9))
            vib = 1 + 0.004 * math.sin(2 * math.pi * 5.5 * t)
            out[(s0 + i) % total] += 0.04 * env * math.sin(2 * math.pi * f * vib * t)
    return out


def surf():
    """Eight seconds of surf, looped seamlessly: filtered noise swelling with each wave."""
    total = int(8 * RATE)
    x = 7
    out = []
    lp = 0.0
    for i in range(total):
        x ^= (x << 13) & 0xFFFFFFFF; x ^= x >> 17; x ^= (x << 5) & 0xFFFFFFFF
        lp += 0.08 * (((x / 0xFFFFFFFF) * 2 - 1) - lp)
        t = i / RATE
        swell = 0.35 + 0.65 * (0.5 - 0.5 * math.cos(2 * math.pi * t / 4)) ** 2
        out.append(lp * swell * 0.9)
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("chop.wav", mix(noise(0.18, decay=0.04, gain=0.45, seed=3), tone(0.2, 180, 90, shape="tri", decay=0.05, gain=0.4)))
    w("stone.wav", mix(noise(0.12, decay=0.02, gain=0.35, seed=9), tone(0.25, 1250, 1180, shape="sine", decay=0.06, gain=0.25), tone(0.25, 1870, 1800, shape="sine", decay=0.04, gain=0.12)))
    w("pickup.wav", concat(tone(0.06, 660, 700, shape="tri", decay=0.05, gain=0.3), tone(0.12, 990, 1040, shape="tri", decay=0.08, gain=0.3)))
    w("claim.wav", mix(concat(*[tone(0.14, f, shape="tri", gain=0.38, decay=0.12) for f in (392.0, 493.88, 587.33)], tone(0.9, 783.99, shape="tri", gain=0.4, decay=0.45)), noise(0.6, decay=0.25, gain=0.12, seed=13)))
    w("discover.wav", concat(tone(0.18, 880.0, shape="sine", gain=0.3, decay=0.2), tone(0.5, 1318.5, shape="sine", gain=0.28, decay=0.35)))
    w("build.wav", concat(*[mix(noise(0.1, decay=0.025, gain=0.4, seed=17 + k), tone(0.12, 240, 160, shape="tri", decay=0.04, gain=0.3)) + [0.0] * int(0.08 * RATE) for k in range(4)], tone(0.7, 587.33, shape="tri", gain=0.35, decay=0.35)))
    w("dawn.wav", concat(*[tone(0.16, f, shape="tri", gain=0.36, decay=0.14) for f in (587.33, 739.99, 880.0)], tone(0.9, 1174.66, shape="tri", gain=0.36, decay=0.5)))
    w("night.wav", concat(*[tone(0.3, f, shape="tri", gain=0.36, decay=0.22) for f in (440.0, 369.99, 293.66)], tone(1.1, 220.0, shape="tri", gain=0.36, decay=0.55)))
    w("collapse.wav", mix(tone(0.8, 320, 90, shape="sine", decay=0.4, gain=0.4), noise(0.5, decay=0.2, gain=0.15, seed=21)))
    w("splash.wav", mix(noise(0.5, decay=0.14, gain=0.4, seed=5), tone(0.2, 600, 200, shape="sine", decay=0.06, gain=0.15)))
    w("jump.wav", tone(0.12, 300, 520, shape="tri", decay=0.06, gain=0.3))
    w("surf.wav", surf())
    w("isle.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup", "step": "footstep"}
    for name, preset in presets.items():
        write(f"assets/audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


def main():
    H = [[height(c - HALF, r - HALF) for c in range(N)] for r in range(N)]
    terrain(H)
    island_map(H)
    counts = layout(H)
    items()
    sounds()
    print("trees %d, woods %d, rocks %d, bridge %.1f to %.1f" % counts)
    for name, x, z in SITES:
        print("site %s at %.1f, %.1f: %.2f m" % (name, x, z, sample(H, x, z)))
    print("camp %.2f m" % sample(H, *CAMP))


if __name__ == "__main__":
    main()
