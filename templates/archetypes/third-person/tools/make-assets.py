#!/usr/bin/env python3
"""Regenerates Ember Vale's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/terrain/vale.png: a 129 x 129 16-bit grey heightmap of the 128 m
  vale (one sample a metre; black -4 m, white 20 m, as terrain/vale.ogt
  says): rolling meadow, a lake in the middle (og.Water at y 1 fills it),
  a knoll under each of the four beacons and a ring of cliffs round it all.
- assets/terrain/vale-control.png: its 256 x 256 RGBA control map: grass in
  R (except on the paths), the paths in G (a ring round the lake through the
  four beacons and a spur from the south where the lamplighter starts), rock
  in B and sand in A (both everywhere: their slope and altitude rules place
  them).
- assets/audio/*.wav: the beacon's ignition, the sentinel's alarm and grab,
  the splash, the dawn and the night stingers, and the vale's music loop;
  assets/audio/*.ogt: og sfx presets for the interface and the footsteps.
- assets/fonts/NotoSansSC-subset.ttf + .ogt: Noto Sans SC subset to the
  characters of assets/strings/zh-CN.ogt (needs fontTools and the project
  template's font). When either is missing both stay as committed, and the
  script fails naming any character of assets/strings/*.ogt the committed
  font has no glyph for.

Standard library only except the font subset; deterministic.
"""
import math, os, struct, sys, zlib

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")

# The vale: keep these in step with src/world.ts (BEACONS, LAKE, WATER_Y, START).
LOW, HIGH = -4.0, 20.0
N = 129  # samples per side, one a metre
HALF = (N - 1) / 2
LAKE = (0.0, -6.0)
BEACONS = [(-34.0, -30.0), (34.0, -32.0), (36.0, 26.0), (-36.0, 24.0)]
RING = 31.0  # the path's radius round the lake's centre
START_Z = 36.0


def write(path, data):
    path = os.path.join(assets, path)
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

def height(x, z):
    """Metres at local (x, z), the terrain's centre at 0."""
    h = 3.2 + 2.4 * (fbm(x / 40 + 3.1, z / 40 + 7.3, 1) - 0.5)
    ridge = 1 - abs(2 * fbm(x / 18, z / 18, 5) - 1)
    h += 15 * step(44, 62, max(abs(x), abs(z))) * (0.7 + 0.5 * ridge)
    h -= 3.4 * gauss(x, z, LAKE[0], LAKE[1], 14)
    for bx, bz in BEACONS:
        h += 2.6 * gauss(x, z, bx, bz, 5)
    return h


def path(x, z):
    """How much of a path is at (x, z): the ring round the lake and the spur from the south."""
    d = abs(math.hypot(x - LAKE[0], z - LAKE[1]) - RING)
    ring = max(0.0, 1 - d / 1.4)
    spur = max(0.0, 1 - abs(x - 1.5 * math.sin(z / 7)) / 1.2) * step(RING + LAKE[1] - 2, RING + LAKE[1] + 1, z) * (1 - step(START_Z + 2, START_Z + 5, z))
    return max(ring, spur)


def png16_grey(w, h, rows):
    raw = b"".join(b"\x00" + struct.pack(f">{w}H", *r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 16, 0, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def terrain():
    rows = []
    for r in range(N):
        rows.append([max(0, min(65535, round((height(c - HALF, r - HALF) - LOW) / (HIGH - LOW) * 65535))) for c in range(N)])
    write("terrain/vale.png", png16_grey(N, N, rows))
    M = 256
    rows = []
    for r in range(M):
        row = bytearray()
        for c in range(M):
            x = (c + 0.5) / M * (N - 1) - HALF
            z = (r + 0.5) / M * (N - 1) - HALF
            p = path(x, z)
            row += bytes([round(255 * (1 - p)), round(255 * p), 255, 255])
        rows.append(bytes(row))
    write("terrain/vale-control.png", png_bytes(M, M, rows))


# ── sounds ─────────────────────────────────────────────────────────────────

RATE = 22050


def music():
    """Eight bars at 72 BPM in A minor: a soft drone, a plucked ostinato and a high answer, looped seamlessly."""
    beat = 60 / 72
    total = int(32 * beat * RATE)
    out = [0.0] * total
    for i in range(total):
        t = i / RATE
        swell = 0.6 + 0.4 * math.sin(2 * math.pi * t / (16 * beat))
        out[i] += 0.045 * swell * (math.sin(2 * math.pi * 110.0 * t) + 0.5 * math.sin(2 * math.pi * 164.81 * t))
    pluck = [220.0, 261.63, 329.63, 261.63, 196.0, 246.94, 293.66, 246.94]
    for b in range(32):
        f = pluck[b % 8]
        s0 = int(b * beat * RATE)
        for i in range(int(beat * RATE)):
            t = i / RATE
            out[(s0 + i) % total] += 0.05 * math.exp(-t / 0.25) * (math.sin(2 * math.pi * f * t) + 0.3 * math.sin(2 * math.pi * 2 * f * t))
    answer = [(4, 659.25), (6, 587.33), (12, 523.25), (14, 493.88), (20, 659.25), (22, 783.99), (28, 659.25), (30, 587.33)]
    for b, f in answer:
        s0 = int(b * beat * RATE)
        for i in range(int(2 * beat * RATE)):
            t = i / RATE
            out[(s0 + i) % total] += 0.05 * math.exp(-t / 0.9) * (math.sin(2 * math.pi * f * t) + 0.35 * math.sin(2 * math.pi * f * 2.76 * t))
    return out


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("ignite.wav", mix(noise(0.9, decay=0.35, gain=0.35, seed=7), tone(0.9, 180, 420, shape="tri", decay=0.4, gain=0.35)))
    w("alarm.wav", concat(tone(0.12, 220, 330, shape="square", decay=0.08, gain=0.25), tone(0.3, 330, 165, shape="square", decay=0.15, gain=0.25)))
    w("grab.wav", mix(tone(0.4, 140, 60, shape="square", decay=0.15, gain=0.35), noise(0.3, decay=0.08, gain=0.35, seed=11)))
    w("splash.wav", mix(noise(0.5, decay=0.14, gain=0.4, seed=5), tone(0.2, 600, 200, shape="sine", decay=0.06, gain=0.15)))
    w("jump.wav", tone(0.12, 300, 520, shape="tri", decay=0.06, gain=0.3))
    w("dawn.wav", concat(*[tone(0.16, f, shape="tri", gain=0.4, decay=0.14) for f in (440.0, 523.25, 659.25, 783.99)], tone(0.9, 880.0, shape="tri", gain=0.4, decay=0.5)))
    w("night.wav", concat(*[tone(0.3, f, shape="tri", gain=0.4, decay=0.22) for f in (329.63, 293.66, 246.94)], tone(1.0, 220.0, shape="tri", gain=0.4, decay=0.5)))
    w("vale.wav", music())
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup", "step": "footstep"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


# ── the font ───────────────────────────────────────────────────────────────

# ── the sentinels ──────────────────────────────────────────────────────────

# Each sentinel's patrol round the lake's path (keep in step with
# src/sentinels.ts SENTINELS, which spawns them at the first point).
ROUTES = {
    "west": [(-27.0, 12.0), (-31.0, -6.0), (-27.0, -22.0)],
    "north": [(-20.0, -31.0), (0.0, -37.0), (20.0, -31.0)],
    "east": [(27.0, -22.0), (31.0, -6.0), (27.0, 12.0)],
}


def behaviours():
    for name, route in ROUTES.items():
        points = "".join(f"  point {x:g} {height(x, z):.2f} {z:g}\n" for x, z in route)
        write(f"ai/sentinel-{name}.ogt", f"""type behaviour
version 1
---
# The {name} sentinel (og.ai; generated by tools/make-assets.py, which keeps
# the route on the terrain). It walks its round of the lake's path; when it
# sees the hero (og.Perception: within its sight, in its cone, not
# behind the terrain) it calls the alarm and chases, grabs within reach
# (the game's `grab` costs a heart and throws the hero back), stands
# stunned a moment, and searches where it last saw the hero before going
# back to its round. The hero wading in the lake is out of its reach: the
# navmesh stops at the shore.
name "sentinel-{name}"
route post {{
{points}}}
machine {{
  initial patrol
  state patrol {{
    patrol post {{
      wait 1.5
      speed Tuning.patrolSpeed
    }}
    to alert {{
      check sees
      check Game.phase is 1
    }}
  }}
  state alert {{
    enter {{
      command alarm
    }}
    chase {{
      speed Tuning.chaseSpeed
      reach 1.1
    }}
    to grab {{
      check targetDistance below 1.5
    }}
    to search {{
      check sees is false
    }}
    to patrol {{
      check Game.phase not 1
    }}
  }}
  state grab {{
    enter {{
      command grab
    }}
    stop
    to search {{
      after 1.6
    }}
  }}
  state search {{
    move_to lastSeen {{
      speed Tuning.patrolSpeed
    }}
    to alert {{
      check sees
      check Game.phase is 1
    }}
    to patrol {{
      after 4
    }}
  }}
}}
""")


def cmap_chars(path):
    """The code points a TTF's cmap maps (formats 4 and 12; standard library only)."""
    data = open(path, "rb").read()
    tables = {}
    for i in range(struct.unpack(">H", data[4:6])[0]):
        tag, _, offset, _ = struct.unpack(">4sIII", data[12 + 16 * i:28 + 16 * i])
        tables[tag] = offset
    if b"cmap" not in tables:
        sys.exit(f"{path} has no cmap table")
    cmap = tables[b"cmap"]
    found = set()
    for i in range(struct.unpack(">H", data[cmap + 2:cmap + 4])[0]):
        _, _, sub = struct.unpack(">HHI", data[cmap + 4 + 8 * i:cmap + 12 + 8 * i])
        at = cmap + sub
        fmt = struct.unpack(">H", data[at:at + 2])[0]
        if fmt == 4:
            segs = struct.unpack(">H", data[at + 6:at + 8])[0] // 2
            ends = struct.unpack(f">{segs}H", data[at + 14:at + 14 + 2 * segs])
            base = at + 16 + 2 * segs
            starts = struct.unpack(f">{segs}H", data[base:base + 2 * segs])
            deltas = struct.unpack(f">{segs}h", data[base + 2 * segs:base + 4 * segs])
            ranges_at = base + 4 * segs
            ranges = struct.unpack(f">{segs}H", data[ranges_at:ranges_at + 2 * segs])
            for k in range(segs):
                for c in range(starts[k], ends[k] + 1):
                    if c == 0xFFFF:
                        continue
                    if ranges[k] == 0:
                        glyph = (c + deltas[k]) & 0xFFFF
                    else:
                        g_at = ranges_at + 2 * k + ranges[k] + 2 * (c - starts[k])
                        glyph = struct.unpack(">H", data[g_at:g_at + 2])[0]
                        glyph = (glyph + deltas[k]) & 0xFFFF if glyph else 0
                    if glyph:
                        found.add(c)
        elif fmt == 12:
            groups = struct.unpack(">I", data[at + 12:at + 16])[0]
            for k in range(groups):
                first, last, glyph = struct.unpack(">III", data[at + 16 + 12 * k:at + 28 + 12 * k])
                found.update(c for c in range(first, last + 1) if glyph + c - first)
    return found


def font():
    text = open(os.path.join(assets, "strings", "zh-CN.ogt"), encoding="utf-8").read()
    chars = "".join(sorted({ch for ch in text if ord(ch) > 0x2E7F}))
    src = os.path.join(root, "..", "..", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf")
    dst = os.path.join(assets, "fonts", "NotoSansSC-subset.ttf")
    try:
        subset_font(src, dst, chars)
    except ImportError:
        print("font subset skipped: fontTools is not installed (pip install fonttools); checking the committed font")
    except FileNotFoundError:
        print(f"font subset skipped: no source font at {os.path.normpath(src)}; checking the committed font")
    else:
        write("fonts/NotoSansSC-subset.ttf.ogt", "type import\nversion 1\n---\n"
              "# Latin-1, CJK punctuation and every character of assets/strings/zh-CN.ogt (tools/make-assets.py).\n"
              f'font {{\n  charset latin cjk-punct\n  chars "{chars}"\n  em_px 32\n  px_range 4\n}}\n')
        return
    # Not subset: the committed font must already draw every CJK character the strings use.
    strings = os.path.join(assets, "strings")
    need = set()
    for name in sorted(os.listdir(strings)):
        if name.endswith(".ogt"):
            need.update(ch for ch in open(os.path.join(strings, name), encoding="utf-8").read() if ord(ch) > 0x2E7F)
    missing = "".join(sorted(ch for ch in need if ord(ch) not in cmap_chars(dst)))
    if missing:
        sys.exit(f"{os.path.relpath(dst, root)} has no glyph for {missing} (used in assets/strings); "
                 "install fontTools and the project template's font and run tools/make-assets.py again")


def main():
    terrain()
    sounds()
    behaviours()
    if os.path.exists(os.path.join(assets, "strings", "zh-CN.ogt")):
        font()


if __name__ == "__main__":
    main()
