#!/usr/bin/env python3
"""The pixel top-down sample's tile set (README.md): a generated 4 x 4
sheet of swatches made into an atlas of plain tiles and three blob47
auto-tiling terrains, with its `type tileset` text.

    tiles.py sheet <generated.png> <out.png>
        the generated sheet (drawn at about 16x, not a whole scale) sampled
        at each logical pixel's centre onto its 64 x 64 grid: the source
        the atlas is made from (examples/art-pixel-topdown/sources/)
    tiles.py atlas <sheet.png> <atlas.png> <tileset.ogt> <texture-id> [scale]
        the atlas: tiles 0-15 the sheet's own (grass, path, cliff face and
        steps, water, bush, stump), then 47 tiles per terrain: `path`
        (packed dirt on grass), `pond` (water inside a dark bank) and
        `ledge` (raised grass with a rock lip; the cliff face below it is
        tiles 8-11, placed by the map), each tile drawn from the swatches
        for its neighbour mask. Written at `scale` texels per logical pixel
        (default 2): the cook brings every tile to ArtPixels.unit itself.

Needs ImageMagick (`magick`); no Python packages.
"""
import subprocess, sys

T = 16  # logical pixels per tile
N, NE, E, SE, S, SW, W, NW = 1, 2, 4, 8, 16, 32, 64, 128


def read(path):
    w, h = map(int, subprocess.run(["magick", "identify", "-format", "%w %h", path], capture_output=True, text=True, check=True).stdout.split())
    raw = subprocess.run(["magick", path, "-depth", "8", "rgb:-"], capture_output=True, check=True).stdout
    return w, h, [tuple(raw[i:i + 3]) for i in range(0, len(raw), 3)]


def write(path, w, h, px, scale=1):
    raw = bytes(c for p in px for c in p)
    subprocess.run(["magick", "-size", f"{w}x{h}", "-depth", "8", "rgb:-", "-filter", "point", "-resize", f"{w * scale}x{h * scale}",
                    "-define", "png:compression-level=9", path], input=raw, check=True)


def sheet(src, out):
    w, h, px = read(src)
    g = 4 * T
    res = [px[int((y + 0.5) * h / g) * w + int((x + 0.5) * w / g)] for y in range(g) for x in range(g)]
    write(out, g, g, res)


def blob_masks():
    out = set()
    for m in range(256):
        r = m & (N | E | S | W)
        for c, a, b in ((NE, N, E), (SE, S, E), (SW, S, W), (NW, N, W)):
            if m & c and m & a and m & b:
                r |= c
        out.add(r)
    return sorted(out)


# A ragged edge that starts and ends level, so tiles meet along it.
JIT = [0, 0, 1, 1, 2, 1, 0, 0, 1, 2, 2, 1, 0, 1, 1, 0]


def inside(x, y, m, e):
    """Whether logical pixel (x, y) of a filled cell with neighbour mask `m` lies in the terrain."""
    if not m & N and y < e + JIT[x]:
        return False
    if not m & S and y > T - 1 - e - JIT[(x + 5) % T]:
        return False
    if not m & W and x < e + JIT[(y + 9) % T]:
        return False
    if not m & E and x > T - 1 - e - JIT[(y + 3) % T]:
        return False
    # Outer corners round off; inner corners (both edges filled, the corner not) take a notch.
    for a, b, c, cx, cy in ((N, W, NW, 0, 0), (N, E, NE, T - 1, 0), (S, W, SW, 0, T - 1), (S, E, SE, T - 1, T - 1)):
        d = abs(x - cx) + abs(y - cy)
        if not m & a and not m & b and d < 2 * e + 3:
            return False
        if m & a and m & b and not m & c and d < e + 1:
            return False
    return True


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c)


def atlas(src, out, ogt, texture, scale=2):
    _, _, px = read(src)
    g = 4 * T
    tile = lambda i: [[px[((i // 4) * T + y) * g + (i % 4) * T + x] for x in range(T)] for y in range(T)]
    # A fill's top rows are the sheet's band between its rows (the grass fringe over the path): rows from its middle take their place.
    clean = lambda t: [t[y + 8] if y < 2 else t[y] for y in range(T)]
    grass, dirt, water, cliff = tile(0), clean(tile(4)), clean(tile(12)), tile(8)
    masks = blob_masks()
    tiles = [tile(i) for i in range(16)]

    def terrain(fill, back, e, rim):
        for m in masks:
            ins = [[inside(x, y, m, e) for x in range(T)] for y in range(T)]
            t = []
            for y in range(T):
                row = []
                for x in range(T):
                    row.append(rim(x, y, m, ins) if rim(x, y, m, ins) else (fill if ins[y][x] else back)[y][x])
                t.append(row)
            tiles.append(t)

    def near(ins, x, y, want):
        return any(0 <= x + dx < T and 0 <= y + dy < T and ins[y + dy][x + dx] == want for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))

    # path: dirt; the grass that borders it darkens (its overhang).
    def path_rim(x, y, m, ins):
        return shade(grass[y][x], 0.5) if not ins[y][x] and near(ins, x, y, True) else None
    terrain(dirt, grass, 3, path_rim)

    # pond: water inside a bank: earth on the north side (the bank's face, seen from above), a dark lip elsewhere.
    bank = shade(dirt[8][8], 0.55)
    def pond_rim(x, y, m, ins):
        if not ins[y][x]:
            return None
        if y >= 1 and not ins[y - 1][x]:
            return bank
        if y >= 2 and not ins[y - 2][x]:
            return shade(bank, 0.75)
        return shade(water[y][x], 0.6) if near(ins, x, y, False) else None
    terrain(water, grass, 3, pond_rim)

    # ledge: raised grass whose open sides end in a rock lip (the face below it is the cliff tiles).
    def ledge_rim(x, y, m, ins):
        if ins[y][x]:
            return shade(grass[y][x], 1.12) if near(ins, x, y, False) else None
        return cliff[(y + 4) % T][x] if (y >= 1 and ins[y - 1][x]) or near(ins, x, y, True) else shade(cliff[y][x], 0.7)
    terrain(grass, grass, 2, ledge_rim)

    # The cliff face's west end (157 face, 158 foot): its side turns away from the light.
    for i in (8, 11):
        tiles.append([[shade(c, 0.55 if x < 3 else 1) for x, c in enumerate(row)] for row in tiles[i]])

    cols = 16
    rows = (len(tiles) + cols - 1) // cols
    while len(tiles) < cols * rows:
        tiles.append([[(0, 0, 0)] * T for _ in range(T)])
    w, h = cols * T, rows * T
    img = [tiles[(y // T) * cols + x // T][y % T][x % T] for y in range(h) for x in range(w)]
    write(out, w, h, img, scale)

    lines = [
        "type tileset", "version 1", "---",
        "# Written by tools/art-samples/tiles.py from sources/terrain-sheet.png: tiles 0-3 grass,",
        "# 4-7 path, 8-9 cliff face, 10 steps, 11 the cliff's foot, 12-13 water, 14 bush, 15 stump;",
        "# 16-62 the path terrain's 47 blob shapes, 63-109 the pond's, 110-156 the ledge's;",
        "# 157-158 the cliff face's west end (face, foot).",
        f'texture "{texture}"', f"grid {T * scale} {T * scale} {cols} {rows}", "filter nearest",
        "solid 8 9 11 12 13 14 15 157 158",
    ]
    for name, base in (("path", 16), ("pond", 63), ("ledge", 110)):
        lines += [f"terrain {name} {{", "  mode blob47"] + [f"  rule {m} {base + i}" for i, m in enumerate(masks)] + [f"  default {base + masks.index(255)}", "  edge filled", "}"]
    open(ogt, "w").write("\n".join(lines) + "\n")


if __name__ == "__main__":
    cmd, *a = sys.argv[1:]
    if cmd == "sheet":
        sheet(*a)
    elif cmd == "atlas":
        atlas(a[0], a[1], a[2], a[3], int(a[4]) if len(a) > 4 else 2)
    else:
        raise SystemExit(__doc__)
