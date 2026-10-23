#!/usr/bin/env python3
"""Regenerates Spark Orb's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/sprites/orb.png: the orb, a glowing sphere of swirling fire,
  computed per pixel; glow.png (a soft light, dithered: the orb's glow and
  the sky's haze), spark.png (particles and the fixed stars) and
  mote-<generator>.png (the four motes: an ember, a bolt, a crystal, a
  star),
- assets/styles/sparkorb/atlas.png: the neon pack's atlas
  (templates/ui-kit) with its magenta frames turned violet,
- assets/fx/*.ogt: the particle emitters (the tap's burst, a milestone's
  ring, the supernova, the sparks drifting up from the orb, the stars),
- assets/audio/*.wav: the tap, a purchase, a milestone, the supernova and
  the orbit's music loop; *.ogt: the UI's og sfx presets,
- assets/fonts/NotoSansSC-subset.ttf + .ogt: Noto Sans SC subset to the
  characters of assets/strings/zh-CN.ogt (needs fontTools and the project
  template's font; skipped with a note when either is missing).

Standard library only except the font subset; deterministic.
"""
import colorsys, math, os, struct, sys, zlib

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


def canvas_png(c):
    return png_bytes(c.w, c.h, c.rgba())


def clamp(x):
    return max(0.0, min(1.0, x))


def lerp(a, b, t):
    return a + (b - a) * t


# ── the art ────────────────────────────────────────────────────────────────

def orb():
    """A sphere of fire: a white-gold core, orange flesh, a red rim, swirling
    bands turning about its axis, a highlight up and to the left, and a thin
    corona just outside the edge. Straight alpha."""
    n = 256
    R = 0.84  # the sphere's radius in half-sizes (the rest is the corona)
    rows = []
    for y in range(n):
        row = bytearray()
        for x in range(n):
            u = (x + 0.5) / (n / 2) - 1
            v = (y + 0.5) / (n / 2) - 1
            d = math.hypot(u, v)
            if d <= R:
                r = d / R
                z = math.sqrt(max(0.0, 1 - r * r))
                # Swirl: bands in longitude, bent by latitude.
                lon = math.atan2(v, u)
                band = 0.5 + 0.5 * math.sin(lon * 3 + r * 7.5 + math.sin(v * 5) * 1.2)
                heat = clamp(0.25 + 0.75 * z) * (0.82 + 0.18 * band)
                cr = lerp(0.85, 1.0, heat)
                cg = lerp(0.22, 0.92, heat ** 1.4)
                cb = lerp(0.08, 0.62, heat ** 3.2)
                # The highlight up and to the left.
                h = clamp(1 - math.hypot(u + 0.32, v + 0.36) / 0.34)
                cr, cg, cb = (lerp(c, 1.0, h * 0.8) for c in (cr, cg, cb))
                # A soft edge over the last pixel.
                a = clamp((R - d) * n / 2)
                row += bytes([round(cr * 255), round(cg * 255), round(cb * 255), round(a * 255)])
            else:
                c = clamp(1 - (d - R) / (1 - R))
                a = c ** 2.4 * 0.75
                row += bytes([255, 170, 70, round(a * 255)])
        rows.append(bytes(row))
    return png_bytes(n, n, rows)


def dither(x, y):
    """A hashed offset in [-0.5, 0.5) of one 8-bit step, so a slow gradient
    stored in 8 bits shows no rings once the sRGB curve stretches its dark end."""
    return ((hashn(x, y, 77) % 1024) / 1024 - 0.5) / 255


def glow():
    """A soft round glow, fully clear at its edge (a falloff, dithered)."""
    n = 256
    rows = []
    for y in range(n):
        row = bytearray()
        for x in range(n):
            d = math.hypot(x + 0.5 - n / 2, y + 0.5 - n / 2) / (n / 2)
            a = clamp(max(0.0, 1 - d) ** 2.2 + dither(x, y))
            row += bytes([255, 255, 255, round(a * 255)])
        rows.append(bytes(row))
    return png_bytes(n, n, rows)


def spark():
    c = Canvas(16, 16)
    for r, a in ((7.5, 0.15), (5, 0.35), (3, 0.8), (1.6, 1.0)):
        c.circle(8, 8, r, "#ffffff", a)
    return c


def hashn(*v):
    h = 2166136261
    for x in v:
        h = ((h ^ (x & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    return h


def mote(kind):
    """White shapes the sprite tints: an ember (a teardrop), a bolt, a
    crystal (a diamond), a star; each with a soft halo."""
    c = Canvas(32, 32)
    c.circle(16, 16, 15, "#ffffff", 0.12)
    c.circle(16, 16, 11, "#ffffff", 0.18)
    if kind == "ember":
        c.poly([(16, 5), (22, 16), (20, 23), (16, 26), (12, 23), (10, 16)], "#ffffff")
    elif kind == "storm":
        c.poly([(18, 4), (10, 17), (15, 17), (13, 28), (22, 13), (17, 13)], "#ffffff")
    elif kind == "crystal":
        c.poly([(16, 4), (24, 14), (16, 28), (8, 14)], "#ffffff")
        c.line(8, 14, 24, 14, 1.2, "#d9d9d9")
    else:
        pts = []
        for k in range(10):
            a = -math.pi / 2 + k * math.pi / 5
            r = 12 if k % 2 == 0 else 5
            pts.append((16 + r * math.cos(a), 16 + r * math.sin(a)))
        c.poly(pts, "#ffffff")
    return c


PIXEL = """type import
version 1
---
# A slow gradient or single-texel stars: kept exact (RGBA8, no block
# compression, which shows 4 x 4 blocks in dark gradients), sampled nearest.
texture {
  kind pixel
}
"""

LINEAR = """type import
version 1
---
# Drawn art: lossless, sampled linear.
texture {
  kind color
}
"""


def emitter(comment, body):
    return f"type particles\nversion 1\n---\n# {comment}\n{body.strip()}\n"


TAP = emitter("A tap: a handful of sparks thrown out from where the finger landed.", """
maxParticles 40
shape point
spread 180
burst 0 12
duration 0.1
lifetime 0.3 0.55
speed 2 4.2
size 0.1 0.2
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.92 0.6 1
  key 0.5 1 0.62 0.2 0.9
  key 1 0.9 0.3 0.1 0
}
gravity 0 -3 0
drag 2.5
texture "sprites/spark.png"
blend additive
render sprite 20
""")

MILESTONE = emitter("A milestone: a ring of sparks flung out from the orb.", """
maxParticles 72
shape sphere 0.3
spread 180
burst 0 48
duration 0.1
lifetime 0.5 0.8
speed 3 4.5
size 0.12 0.24
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 1 0.85 1
  key 1 1 0.55 0.2 0
}
drag 1.8
texture "sprites/spark.png"
blend additive
render sprite 21
""")

NOVA = emitter("The supernova: the orb bursts in every direction.", """
maxParticles 240
shape sphere 0.6
spread 180
burst 0 200
duration 0.15
lifetime 0.8 1.6
speed 4 11
size 0.15 0.4
sizeOverLife 0 1 0.6 0.8 1 0
colorOverLife {
  key 0 1 1 1 1
  key 0.3 1 0.8 0.4 1
  key 1 0.7 0.3 1 0
}
drag 1.4
texture "sprites/spark.png"
blend additive
render sprite 22
""")

DRIFT = emitter("Sparks drifting up off the orb's skin, always.", """
maxParticles 60
shape sphere 1.2
rate 14
lifetime 1.2 2.2
speed 0.2 0.6
size 0.06 0.14
sizeOverLife 0 0.4 0.3 1 1 0
colorOverLife {
  key 0 1 0.85 0.5 0
  key 0.2 1 0.75 0.35 0.9
  key 1 1 0.4 0.15 0
}
gravity 0 0.6 0
noise 0.4 0.5 0.4
texture "sprites/spark.png"
blend additive
render sprite 2
""")

STARS = emitter("The night's stars: faint, slow, twinkling in and out across the sky.", """
maxParticles 90
shape box 22 14 0
rate 9
lifetime 6 10
speed 0 0.03
size 0.04 0.1
sizeOverLife 0 0.6 0.5 1 1 0.6
colorOverLife {
  key 0 0.8 0.85 1 0
  key 0.25 0.85 0.9 1 0.8
  key 0.75 1 0.95 0.9 0.6
  key 1 1 0.95 0.9 0
}
texture "sprites/spark.png"
blend additive
render sprite 0
""")


# ── the style's atlas ───────────────────────────────────────────────────────

def read_png(path):
    """An 8-bit RGBA, non-interlaced PNG's rows of pixels (what make-styles.py writes)."""
    data = open(path, "rb").read()
    at, idat, w, h = 8, b"", 0, 0
    while at < len(data):
        n = struct.unpack(">I", data[at:at + 4])[0]
        kind = data[at + 4:at + 8]
        body = data[at + 8:at + 8 + n]
        if kind == b"IHDR":
            w, h, depth, colour = struct.unpack(">IIBB", body[:10])
            assert depth == 8 and colour == 6, "RGBA8 only"
        elif kind == b"IDAT":
            idat += body
        at += 12 + n
    raw = zlib.decompress(idat)
    stride = w * 4
    rows, prev = [], bytearray(stride)
    for y in range(h):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - 4] if i >= 4 else 0
            b = prev[i]
            c = prev[i - 4] if i >= 4 else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(line))
        prev = line
    return w, h, rows


def atlas():
    """The neon pack's atlas with its baked magenta frames turned to the
    night's violet (hue 300 to 262), the rest untouched."""
    src = os.path.join(root, "..", "..", "styles", "neon", "atlas.png")
    if not os.path.exists(src):
        print("atlas recolour skipped (no templates/ui-kit here); the committed atlas stays")
        return None
    w, h, rows = read_png(src)
    out = []
    for row in rows:
        line = bytearray(row)
        for x in range(w):
            r, g, b, a = line[x * 4:x * 4 + 4]
            if a == 0:
                continue
            hh, ss, vv = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
            if ss > 0.3 and 0.78 <= hh <= 0.9:
                nr, ng, nb = colorsys.hsv_to_rgb(262 / 360, ss * 0.85, vv)
                line[x * 4:x * 4 + 3] = bytes([round(nr * 255), round(ng * 255), round(nb * 255)])
        out.append(bytes(line))
    return png_bytes(w, h, out)


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def bell(seconds, f, gain=0.2, decay=0.6):
    n = int(seconds * RATE)
    out = []
    for i in range(n):
        t = i / RATE
        env = min(1.0, t / 0.004) * math.exp(-t / decay)
        out.append(gain * env * (math.sin(2 * math.pi * f * t) + 0.35 * math.sin(2 * math.pi * f * 2.76 * t) + 0.15 * math.sin(2 * math.pi * f * 5.4 * t)))
    return out


def pad(seconds, freqs, gain):
    n = int(seconds * RATE)
    out = [0.0] * n
    for f in freqs:
        for i in range(n):
            t = i / RATE
            env = min(1.0, t / 0.8) * min(1.0, (seconds - t) / 0.8)
            out[i] += gain * env * (math.sin(2 * math.pi * f * t) + 0.25 * math.sin(2 * math.pi * f * 1.005 * t + 1.3))
    return out


def music():
    """Eight bars at 72 BPM in D major: a slow pad under bell notes that circle like the motes; loops seamlessly."""
    beat = 60 / 72
    chords = [(146.83, 220.0, 293.66), (123.47, 185.0, 246.94), (98.0, 146.83, 196.0), (110.0, 164.81, 220.0)] * 2
    melody = [587.33, 739.99, 880.0, 739.99, 659.25, 587.33, 493.88, 587.33]
    total = int(32 * beat * RATE)
    out = [0.0] * total
    for b, ch in enumerate(chords):
        p = pad(4 * beat, ch, 0.045)
        s0 = int(b * 4 * beat * RATE)
        for i, s in enumerate(p):
            if s0 + i < total:
                out[s0 + i] += s
        for k in range(4):
            if (b + k) % 3 == 2:
                continue
            f = melody[(b * 3 + k) % 8] * (0.5 if b >= 4 and k == 3 else 1)
            note = bell(beat * 2, f, 0.07, 0.7)
            st = int((b * 4 + k) * beat * RATE)
            for i, s in enumerate(note):
                out[(st + i) % total] += s
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("tap.wav", mix(bell(0.35, 1046.5, 0.28, 0.12), tone(0.05, 2093, 1568, shape="tri", decay=0.02, gain=0.12)))
    w("buy.wav", concat(bell(0.08, 1318.5, 0.25, 0.05), bell(0.35, 1975.5, 0.25, 0.12)))
    w("milestone.wav", concat(*[bell(0.1, f, 0.25, 0.08) for f in (783.99, 987.77, 1174.66)], bell(0.7, 1567.98, 0.28, 0.3)))
    w("nova.wav", mix(noise(1.6, decay=0.45, gain=0.45, seed=11), tone(1.6, 110, 40, shape="sine", decay=0.5, gain=0.6), bell(1.6, 1760, 0.12, 0.6)))
    w("orbit.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook (the UI\'s {name[3:]} sound).\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


# ── the font ───────────────────────────────────────────────────────────────

def zh_characters():
    text = open(os.path.join(assets, "strings", "zh-CN.ogt"), encoding="utf-8").read()
    # And the punctuation the UI draws itself: the en dash (a grade or a
    # count not yet known) and the ellipsis (a cut label), which the zh-CN
    # screens showed as missing glyphs before og-ui warned about them.
    chars = sorted({ch for ch in text if ord(ch) > 0x2E7F} | set("–…"))
    return "".join(chars)


def font():
    chars = zh_characters()
    src = os.path.join(root, "..", "..", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf")
    try:
        subset_font(src, os.path.join(assets, "fonts", "NotoSansSC-subset.ttf"), chars)
    except Exception as e:  # noqa: BLE001
        print(f"font subset skipped ({e}); the committed assets/fonts/NotoSansSC-subset.ttf stays")
    write("fonts/NotoSansSC-subset.ttf.ogt", "type import\nversion 1\n---\n"
          "# Latin-1, CJK punctuation and every character of assets/strings/zh-CN.ogt (tools/make-assets.py).\n"
          f'font {{\n  charset latin cjk-punct\n  chars "{chars}"\n  em_px 32\n  px_range 4\n}}\n')


def main():
    write("sprites/orb.png", orb())
    write("sprites/orb.png.ogt", LINEAR)
    write("sprites/glow.png", glow())
    write("sprites/glow.png.ogt", PIXEL)
    write("sprites/spark.png", canvas_png(spark()))
    write("sprites/spark.png.ogt", LINEAR)
    for kind in ("ember", "storm", "crystal", "star"):
        write(f"sprites/mote-{kind}.png", canvas_png(mote(kind)))
        write(f"sprites/mote-{kind}.png.ogt", LINEAR)
    recoloured = atlas()
    if recoloured is not None:
        write("styles/sparkorb/atlas.png", recoloured)
    write("fx/tap.ogt", TAP)
    write("fx/milestone.ogt", MILESTONE)
    write("fx/nova.ogt", NOVA)
    write("fx/drift.ogt", DRIFT)
    write("fx/stars.ogt", STARS)
    sounds()
    font()


if __name__ == "__main__":
    main()
