#!/usr/bin/env python3
"""Regenerates Three-Lane Beat's self-made assets (CC0), byte for byte:

    python3 tools/make-assets.py            # from the archetype directory

- assets/charts/neon-pulse.ogt: the song's chart, a scene of one `Song`
  and a `Note` per note (lane, beat), written from the same note lists as
  the stems, so every note is on a sound,
- assets/audio/*.wav: the song "Neon Pulse" (120 BPM, 24 bars) as stems
  the game starts on bar lines through og.Music (drums: one bar; bass,
  chords and two lead phrases: two bars each), the metronome's click, a
  tap per lane, the miss, the clear and the fail; *.ogt: the UI's og sfx
  presets,
- assets/sprites/*.png: the note, the receptor ring, the lane's glow and
  the spark,
- assets/fx/*.ogt: the hit's burst and the beat's dust,
- assets/fonts/NotoSansSC-subset.ttf + .ogt: Noto Sans SC subset to the
  characters of assets/strings/zh-CN.ogt (needs fontTools and the project
  template's font; skipped with a note when either is missing).

Standard library only except the font subset; deterministic.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import Canvas, png_bytes, write_wav, tone, noise, mix, concat, subset_font  # noqa: E402

root = os.path.dirname(here)
assets = os.path.join(root, "assets")
RATE = 22050
BPM = 120
BEAT = 60 / BPM
BAR = 4 * BEAT
BARS = 24


def write(path, data):
    path = os.path.join(assets, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb" if isinstance(data, bytes) else "w") as f:
        f.write(data)


def canvas_png(c):
    return png_bytes(c.w, c.h, c.rgba())


# ── the song: note lists the stems and the chart share ─────────────────────

# A minor: A C D E G (MIDI numbers).
def hz(midi):
    return 440.0 * 2 ** ((midi - 69) / 12)


# Two-bar chords (root, third, fifth) for the bass and the pads: Am, F, C, G.
CHORDS = [(57, 60, 64), (53, 57, 60), (48, 52, 55), (55, 59, 62)]
# Two lead phrases of sixteen eighths (None rests), two bars each.
LEAD_A = [69, None, 72, 74, 76, None, 74, 72, 69, None, 67, 69, 72, None, None, None]
LEAD_B = [76, 79, 76, 74, 72, None, 74, 76, 81, None, 79, 76, 74, 72, 69, None]

def lane_of(midi):
    """A lead note's lane by pitch: low, middle, high."""
    return 0 if midi < 71 else (1 if midi < 76 else 2)


def chart_notes():
    """(beat, lane) from 0 = bar 1's first beat. Follows the music: the
    count-in bars' quarters, then kick and snare, then the lead's melody,
    then chords of two lanes, then the last hit on all three."""
    notes = []
    for bar in range(1, BARS + 1):
        b0 = (bar - 1) * 4
        if bar <= 2:
            for k in range(4):
                notes.append((b0 + k, 1 if bar == 1 else (0, 2, 0, 2)[k]))
        elif bar <= 4:
            notes += [(b0, 0), (b0 + 1, 2), (b0 + 2, 0), (b0 + 3, 2)]
            if bar == 4:
                notes.append((b0 + 3.5, 1))
        elif bar <= 8:
            notes += [(b0, 0), (b0 + 1, 2), (b0 + 2, 0), (b0 + 3, 2)]
            notes += [(b0 + 1.5, 1)] if bar % 2 == 0 else [(b0 + 2.5, 1)]
        elif bar <= 20:
            phrase = LEAD_A if ((bar - 9) // 2) % 2 == 0 else LEAD_B
            half = ((bar - 9) % 2) * 8
            for k in range(8):
                m = phrase[half + k]
                if m is not None:
                    notes.append((b0 + k * 0.5, lane_of(m)))
        elif bar <= 22:
            notes += [(b0, 0), (b0, 2), (b0 + 1, 1), (b0 + 2, 0), (b0 + 2, 1), (b0 + 3, 2)]
        else:
            notes += [(b0, 0), (b0 + 1, 1), (b0 + 2, 2), (b0 + 3, 1)]
    end = BARS * 4
    notes += [(end, 0), (end, 1), (end, 2)]
    return sorted(notes)


def chart_text():
    notes = chart_notes()
    out = [
        "type scene", "version 1", "---",
        "# Neon Pulse's chart (tools/make-assets.py writes it from the song's note lists).",
        "# A chart is a scene: one entity with a Song (tempo, metre, length in bars) and an",
        "# entity per note, its Note's lane (0 left, 1 middle, 2 right) and beat (from 0 at bar",
        "# 1's first beat; 3.5 is the 'and' of beat 4). Edit it by hand: the game reads it as is.",
        'name "Neon Pulse"',
        'entity "song" {',
        "  Song {",
        f"    bpm {BPM}",
        "    beatsPerBar 4",
        f"    bars {BARS}",
        f"    notes {len(notes)}",
        "  }",
        "}",
    ]
    for i, (beat, lane) in enumerate(notes):
        out += [f'entity "note-{i}" {{', "  Note {", f"    index {i}", f"    lane {lane}", f"    beat {beat:g}", "  }", "}"]
    return "\n".join(out) + "\n"


# ── synthesis ──────────────────────────────────────────────────────────────

def silence(seconds):
    return [0.0] * int(seconds * RATE)


def place(out, sound, at):
    s0 = int(round(at * RATE))
    for i, v in enumerate(sound):
        if s0 + i < len(out):
            out[s0 + i] += v


def kick():
    n = int(0.25 * RATE)
    out, phase = [], 0.0
    for i in range(n):
        t = i / RATE
        f = 50 + 110 * math.exp(-t / 0.03)
        phase += 2 * math.pi * f / RATE
        out.append(0.9 * math.sin(phase) * math.exp(-t / 0.12))
    return out


def snare():
    return mix(noise(0.18, decay=0.06, gain=0.45, seed=7), tone(0.12, 190, 160, shape="tri", decay=0.04, gain=0.3))


def hat(open_=False):
    return noise(0.12 if open_ else 0.05, decay=0.04 if open_ else 0.012, gain=0.16, seed=13)


def saw(seconds, f, gain, decay):
    n = int(seconds * RATE)
    out = []
    for i in range(n):
        t = i / RATE
        ph = (f * t) % 1.0
        env = min(1.0, t / 0.005) * math.exp(-t / decay)
        out.append(gain * env * (2 * ph - 1))
    return out


def lowpass(x, a=0.2):
    y, out = 0.0, []
    for v in x:
        y += a * (v - y)
        out.append(y)
    return out


def drums():
    out = silence(BAR)
    for k in range(4):
        if k % 2 == 0:
            place(out, kick(), k * BEAT)
        else:
            place(out, snare(), k * BEAT)
    for k in range(8):
        place(out, hat(k == 7), k * BEAT / 2)
    return out


def stem_bass(chord):
    out = silence(2 * BAR)
    root = chord[0] - 12
    for k in range(16):
        f = hz(root + (12 if k % 4 == 3 else 0))
        place(out, lowpass(saw(BEAT / 2 * 0.95, f, 0.35, 0.18), 0.12), k * BEAT / 2)
    return out


def stem_chords(chord):
    out = silence(2 * BAR)
    for k in (0, 1.5, 3, 4, 5.5, 7):
        for m in chord:
            place(out, lowpass(saw(BEAT * 1.2, hz(m), 0.07, 0.35), 0.18), k * BEAT)
    return out


def stem_lead(phrase):
    out = silence(2 * BAR)
    for k, m in enumerate(phrase):
        if m is None:
            continue
        f = hz(m)
        n = int(BEAT / 2 * 0.9 * RATE)
        note = []
        for i in range(n):
            t = i / RATE
            env = min(1.0, t / 0.004) * math.exp(-t / 0.22)
            note.append(0.22 * env * (math.sin(2 * math.pi * f * t) + 0.45 * math.sin(4 * math.pi * f * t + 0.4) + 0.2 * math.sin(6 * math.pi * f * t)))
        place(out, note, k * BEAT / 2)
    return out


def song_stems():
    """Each stem one or two bars long; the game starts each on its bar line.
    The bass and the chords follow the two-bar chords in turn, so each is
    four two-bar stems."""
    stems = {"drums": drums()}
    for i, ch in enumerate(CHORDS):
        stems[f"bass{i}"] = stem_bass(ch)
        stems[f"chords{i}"] = stem_chords(ch)
    stems["leadA"] = stem_lead(LEAD_A)
    stems["leadB"] = stem_lead(LEAD_B)
    return stems


def sounds():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    for name, s in song_stems().items():
        w(f"{name}.wav", s)
    w("click.wav", concat(tone(0.03, 1760, shape="square", decay=0.012, gain=0.3)))
    for lane, f in enumerate((523.25, 659.25, 783.99)):
        w(f"tap{lane}.wav", mix(tone(0.09, f, shape="tri", decay=0.035, gain=0.35), noise(0.02, decay=0.006, gain=0.12, seed=lane + 1)))
    w("miss.wav", mix(tone(0.18, 140, 90, shape="square", decay=0.06, gain=0.22), noise(0.1, decay=0.03, gain=0.12, seed=31)))
    w("clear.wav", concat(*[tone(0.11, hz(m), shape="tri", gain=0.38, decay=0.09) for m in (69, 72, 76, 79)], tone(0.6, hz(81), shape="tri", gain=0.4, decay=0.35)))
    w("fail.wav", concat(*[tone(0.2, hz(m), shape="tri", gain=0.38, decay=0.16) for m in (64, 62, 60)], tone(0.7, hz(57), shape="tri", gain=0.38, decay=0.4)))
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup"}
    for name, preset in presets.items():
        write(f"audio/{name}.ogt", f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook (the UI\'s {name[3:]} sound).\npreset "{preset}"\nseed 0\npitch 1\nlength 1\nvolume 0.8\n')


# ── art ────────────────────────────────────────────────────────────────────

def note_png():
    """A note: a rounded bar, white (the sprite tints it by lane), a bright core."""
    c = Canvas(96, 40)
    c.rect(2, 4, 92, 32, "#ffffff", radius=14, alpha=0.35)
    c.rect(6, 8, 84, 24, "#ffffff", radius=11)
    c.rect(14, 13, 68, 6, "#ffffff", radius=3, alpha=0.6)
    return c


def ring_rgba():
    """The receptor: an outline of the note's shape, clear inside."""
    n_w, n_h = 96, 40
    rows = []
    for y in range(n_h):
        row = bytearray()
        for x in range(n_w):
            # Signed distance to a rounded rectangle 88 x 28 at (4, 6), radius 12.
            cx = min(max(x + 0.5, 16), 80)
            cy = min(max(y + 0.5, 18), 22)
            d = math.hypot(x + 0.5 - cx, y + 0.5 - cy) - 12
            a = max(0.0, 1 - abs(d + 1.5) / 1.8)
            glow = max(0.0, 1 - abs(d) / 6) * 0.25
            row += bytes([255, 255, 255, round(min(1.0, a + glow) * 255)])
        rows.append(bytes(row))
    return png_bytes(n_w, n_h, rows)


def lane_rgba():
    """A lane's glow: bright at the bottom, fading up, soft at the sides."""
    w, h = 32, 128
    rows = []
    for y in range(h):
        row = bytearray()
        for x in range(w):
            side = 1 - abs((x + 0.5) / w * 2 - 1)
            up = (y + 0.5) / h
            a = (side ** 1.5) * (up ** 2.0) * 0.9
            row += bytes([255, 255, 255, round(a * 255)])
        rows.append(bytes(row))
    return png_bytes(w, h, rows)


def spark():
    c = Canvas(16, 16)
    for r, a in ((7.5, 0.15), (5, 0.35), (3, 0.8), (1.6, 1.0)):
        c.circle(8, 8, r, "#ffffff", a)
    return c


LINEAR = """type import
version 1
---
# Drawn art: lossless, sampled linear.
texture {
  kind color
}
"""

PIXEL = """type import
version 1
---
# A soft gradient: kept exact (RGBA8, no block compression), sampled nearest.
texture {
  kind pixel
}
"""

HIT = """type particles
version 1
---
# A hit: sparks thrown up from the receptor in the lane's colour (the sprite tints them).
maxParticles 40
shape box 0.5 0.05 0
spread 60
burst 0 14
duration 0.1
lifetime 0.25 0.45
speed 3 6
size 0.08 0.16
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 1 1 1
  key 1 1 1 1 0
}
gravity 0 -8 0
drag 2
texture "sprites/spark.png"
blend additive
render sprite 20
"""


# ── the font ───────────────────────────────────────────────────────────────

def zh_characters():
    text = open(os.path.join(assets, "strings", "zh-CN.ogt"), encoding="utf-8").read()
    chars = sorted({ch for ch in text if ord(ch) > 0x2E7F})
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
    write("charts/neon-pulse.ogt", chart_text())
    write("sprites/note.png", canvas_png(note_png()))
    write("sprites/note.png.ogt", LINEAR)
    write("sprites/ring.png", ring_rgba())
    write("sprites/ring.png.ogt", LINEAR)
    write("sprites/lane.png", lane_rgba())
    write("sprites/lane.png.ogt", PIXEL)
    write("sprites/spark.png", canvas_png(spark()))
    write("sprites/spark.png.ogt", LINEAR)
    write("fx/hit.ogt", HIT)
    sounds()
    font()


if __name__ == "__main__":
    main()
