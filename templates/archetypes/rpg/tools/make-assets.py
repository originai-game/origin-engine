#!/usr/bin/env python3
"""Greywatch Keep's generated sounds (run from anywhere; writes under assets/):

  assets/audio/keep.wav     the yard's music: a slow modal loop (D dorian),
                            a drone, a lute-like ostinato and a high answer.
  assets/audio/won.wav      the quest done: a rising fanfare.
  assets/audio/fallen.wav   the party fallen: a falling toll.
  assets/audio/quest.wav    a quest stage done: a two-note bell.
  assets/audio/*.ogt        og sfx presets, synthesised by og cook: the
                            interface, the potion, a coin, a level-up, a
                            hurt cry and the risen's alarm.

The blows, the bones, the chest and the steps are Kenney's CC0 recordings
(LICENSES.md), not generated here.
"""
import math, os, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from oglite import write_wav, tone, mix, concat  # noqa: E402

RATE = 22050
assets = os.path.join(os.path.dirname(here), "assets")


def music():
    """Sixteen bars at 66 BPM in D dorian, looped seamlessly."""
    beat = 60 / 66
    total = int(64 * beat * RATE)
    out = [0.0] * total
    for i in range(total):
        t = i / RATE
        swell = 0.6 + 0.4 * math.sin(2 * math.pi * t / (32 * beat))
        out[i] += 0.04 * swell * (math.sin(2 * math.pi * 73.42 * t) + 0.5 * math.sin(2 * math.pi * 110.0 * t))
    pluck = [293.66, 349.23, 440.0, 349.23, 261.63, 329.63, 392.0, 329.63]
    for b in range(64):
        f = pluck[b % 8] * (0.5 if (b // 16) % 2 else 1.0) * 0.75
        s0 = int(b * beat * RATE)
        for i in range(int(beat * RATE)):
            t = i / RATE
            out[(s0 + i) % total] += 0.05 * math.exp(-t / 0.3) * (math.sin(2 * math.pi * f * t) + 0.35 * math.sin(2 * math.pi * 2 * f * t) + 0.12 * math.sin(2 * math.pi * 3 * f * t))
    answer = [(6, 587.33), (8, 523.25), (14, 493.88), (22, 659.25), (24, 587.33), (30, 523.25), (38, 698.46), (40, 659.25), (46, 587.33), (54, 523.25), (56, 493.88), (62, 440.0)]
    for b, f in answer:
        s0 = int(b * beat * RATE)
        for i in range(int(2 * beat * RATE)):
            t = i / RATE
            out[(s0 + i) % total] += 0.04 * math.exp(-t / 1.0) * (math.sin(2 * math.pi * f * t) + 0.3 * math.sin(2 * math.pi * f * 2.76 * t))
    return out


def main():
    a = os.path.join(assets, "audio")
    os.makedirs(a, exist_ok=True)
    w = lambda name, s: write_wav(os.path.join(a, name), s)  # noqa: E731
    w("keep.wav", music())
    w("won.wav", concat(*[tone(0.15, f, shape="tri", gain=0.4, decay=0.13) for f in (293.66, 369.99, 440.0, 587.33)], tone(1.1, 739.99, shape="tri", gain=0.4, decay=0.6)))
    w("fallen.wav", concat(*[tone(0.36, f, shape="tri", gain=0.4, decay=0.26) for f in (293.66, 261.63, 220.0)], tone(1.2, 146.83, shape="tri", gain=0.45, decay=0.6)))
    w("quest.wav", concat(tone(0.18, 659.25, shape="sine", gain=0.4, decay=0.3), mix(tone(0.9, 987.77, shape="sine", gain=0.35, decay=0.5), tone(0.9, 1975.5, shape="sine", gain=0.08, decay=0.3))))
    presets = {"ui-press": "click", "ui-open": "select", "ui-close": "back", "ui-error": "error", "ui-success": "levelup",
               "potion": "powerup", "coin": "coin", "levelup": "levelup", "hurt": "hurt", "alarm": "laser"}
    for name, preset in presets.items():
        with open(os.path.join(a, f"{name}.ogt"), "w") as f:
            f.write(f'type sfx\nversion 1\n---\n# og sfx\'s "{preset}" preset, synthesised by og cook.\npreset "{preset}"\nseed 0\npitch {0.6 if name == "alarm" else 1}\nlength 1\nvolume 0.8\n')


if __name__ == "__main__":
    main()
