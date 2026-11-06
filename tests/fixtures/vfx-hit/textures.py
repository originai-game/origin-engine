# Writes assets/textures/*.png, the fixture's only images (README.md):
#   python3 tests/fixtures/vfx-hit/textures.py
import math, os, random
from PIL import Image

here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets", "textures")

def smooth(a, b, x):
    t = min(max((x - a) / (b - a), 0.0), 1.0)
    return t * t * (3 - 2 * t)

def save(name, size, f, mode="L", height=None):
    h = height or size
    img = Image.new(mode, (size, h))
    img.putdata([f((x + 0.5) / size, (y + 0.5) / h) for y in range(h) for x in range(size)])
    img.save(os.path.join(here, name), optimize=True)

def lattice(period, seed):
    r = random.Random(seed)
    return [[r.random() for _ in range(period)] for _ in range(period)]

def value_noise(g, period, u, v):
    x, y = u * period, v * period
    i, j = int(math.floor(x)), int(math.floor(y))
    fx, fy = x - i, y - j
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a = g[j % period][i % period]; b = g[j % period][(i + 1) % period]
    c = g[(j + 1) % period][i % period]; d = g[(j + 1) % period][(i + 1) % period]
    return (a + (b - a) * fx) * (1 - fy) + (c + (d - c) * fx) * fy

octaves = [(lattice(4 << k, 7 + k), 4 << k, 0.5 ** k) for k in range(4)]
def fbm(u, v):
    return sum(w * value_noise(g, p, u, v) for g, p, w in octaves) / sum(w for _, _, w in octaves)

def white(a):
    return (255, 255, 255, int(255 * a))

# A hot dot with a soft halo, white in alpha: sparks, embers, glints.
save("spark.png", 64, lambda u, v: white((lambda d: 0.75 * (1 - smooth(0, 0.25, d)) ** 2 + 0.25 * (1 - smooth(0.1, 1, d)) ** 2)(math.hypot(u - .5, v - .5) * 2)), "RGBA")
# Tiling fbm: erosion, scrolling streaks, smoke shapes.
save("noise.png", 128, lambda u, v: int(255 * min(max((fbm(u, v) - 0.2) / 0.6, 0), 1)))
# A soft puff, white in alpha: smoke billboards.
save("puff.png", 64, lambda u, v: white((1 - smooth(0.1, 1, math.hypot(u - .5, v - .5) * 2)) ** 1.5 * (0.6 + 0.4 * fbm(u, v))), "RGBA")
# The decal atlas: a scorch (left half) and a heal sigil (right half), RGBA.
def atlas(u, v):
    if u < 0.5:
        x, y = (u - 0.25) * 4, (v - 0.5) * 2
        d = math.hypot(x, y); a = math.atan2(y, x)
        rays = 0.5 + 0.5 * math.cos(a * 9 + 3 * fbm(u, v))
        reach = 0.55 + 0.4 * rays * fbm(u * 2, v * 2)
        alpha = (1 - smooth(reach * 0.6, reach, d)) * 0.95
        burn = 0.03 + 0.05 * fbm(u * 3, v * 3)
        return (int(255 * burn), int(255 * burn * 0.8), int(255 * burn * 0.7), int(255 * alpha))
    x, y = (u - 0.75) * 4, (v - 0.5) * 2
    d = math.hypot(x, y); a = math.atan2(y, x)
    ring = max(1 - abs(d - 0.85) / 0.05, 0) + max(1 - abs(d - 0.62) / 0.03, 0)
    spokes = max(1 - abs(math.sin(a * 3)) * d * 12, 0) * (d < 0.62)
    g = min(ring + spokes, 1) * (1 - smooth(0.9, 0.98, d))
    return (int(255 * 0.55), 255, int(255 * 0.7), int(255 * g))
save("atlas.png", 512, atlas, "RGBA", 256)
