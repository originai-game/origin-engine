#!/usr/bin/env python3
"""Masters the theme sound sets (the `sound` dimension, docs: the theme plan
§2): each set's raw clip of each event, as generated (origin's
generate_sfx, ElevenLabs eleven_text_to_sound_v3, the prompts below),
trimmed, faded and levelled to the event's loudness, written as Ogg Vorbis:

    templates/styles/shared/sfx/<set>/<event>.ogg

    python3 templates/tools/master-theme-sounds.py <raw dir> [repo]

`<raw dir>/<set>/<event>.mp3` are the generated originals (MEDIA names each
one's media id on the platform). Needs numpy and ffmpeg (libvorbis).

The mix: every clip starts at its first sound, ends at its tail's -50 dB
with a 12 ms fade, is cut to its event's longest length, and is levelled
so its active part (the frames within 30 dB of its peak) has its event's
RMS, its peak held at -1 dBFS. Quiet events (tap, toggle, tab) sit 5 dB
under the stingers (reward, levelup), so a theme's `volume` sets taste,
not balance; every set mixes the same.
"""
import os, subprocess, sys
import numpy as np

RATE = 48000

# event: (active RMS dBFS, longest seconds)
MIX = {
    "tap": (-22, 0.15), "toggle": (-22, 0.2), "tab": (-22, 0.25), "countdown": (-20, 0.25),
    "back": (-21, 0.4), "close": (-21, 0.45), "open": (-21, 0.5), "confirm": (-19, 0.6),
    "error": (-19, 0.6), "purchase": (-18, 1.0), "reward": (-18, 1.3), "levelup": (-17, 2.0),
}

SETS = {
    "soft": "soft warm UI: muted felt-mallet tones and gentle wooden taps, calm and cozy, quiet",
    "glass": "glass UI: crystal chimes and airy synth bells, polished and shimmering, premium RPG menu",
    "mechanical": "mechanical UI: tactile metal switches, servo clicks, latches and short hydraulic hisses, military instrument panel",
    "wood": "wooden UI: warm hollow wood knocks, marimba and wooden blocks, natural fantasy tavern feel",
    "chiptune": "8-bit chiptune UI: square and pulse wave blips, retro NES game console, crisp and simple",
    "arcade": "neon arcade UI: punchy synthwave zaps, bright digital stabs and laser sweeps, rhythm game energy",
    "cute": "cute casual mobile UI: bubbly pops, squeaky boings, toy xylophone and kalimba, playful and sweet",
}
EVENTS = {
    "tap": "a single very short soft button tap, one tiny click with a hint of tone, under 0.1 seconds, nothing after it",
    "confirm": "a short bright positive confirm sound, two quick rising notes, ends within 0.4 seconds",
    "back": "a short soft falling two-note cancel or back sound, ends within 0.3 seconds",
    "error": "a short gentle error or denied buzz, two low quick notes, not harsh, ends within 0.4 seconds",
    "open": "a short soft panel open whoosh rising upward, ends within 0.4 seconds",
    "close": "a short soft panel close whoosh falling downward, ends within 0.35 seconds",
    "toggle": "a tiny crisp switch toggle click, very short, nothing after it",
    "tab": "a tiny soft tab switch swish with a light tick, very short",
    "reward": "a rewarding item collect jingle, sparkling ascending arpeggio with a shimmering tail, about 1 second",
    "levelup": "a triumphant level up fanfare stinger, rising arpeggio landing on a bright major chord with sparkle, about 1.6 seconds",
    "purchase": "a satisfying purchase sound, coins clinking with a short ka-ching chime, about 0.8 seconds",
    "countdown": "a single short countdown tick beep for a 3-2-1 timer, one clean beep only, under 0.2 seconds",
}
PROMPT = "{event}. Style: {style}. Single isolated sound effect, dry, no music bed, no voice, no background noise."

# set/event: the generated original's media id on the platform.
MEDIA = {
    "arcade/back": "db684d42af5ea29d8dc59cb7589cf3b3",
    "arcade/close": "bdde1c9556989450c39273177c7b0d32",
    "arcade/confirm": "d1b32bf45f233a832faa0d0baaf73368",
    "arcade/countdown": "89474cc5a12fbfbed6dca4d72ef1a66a",
    "arcade/error": "129a9163886e9f1bc90f9dd7ba120b44",
    "arcade/levelup": "96f77ed17b8512756b4fc9a4ddb50ad7",
    "arcade/open": "b8355ae899288b5ac3e74fc0ff07fade",
    "arcade/purchase": "6c0e3cb34a09411a8c83491676a5e75a",
    "arcade/reward": "d96a9196ace03d740e48280723676a60",
    "arcade/tab": "e6d820a637461aa465b7e0bc078e7db0",
    "arcade/tap": "8d7e95ce84bfe3a0ef773afb55591755",
    "arcade/toggle": "5e1416bdac6d95729fe7ca28c86633f7",
    "chiptune/back": "9ba5bde09bb0f6f0e5ae5bb6452fa7f9",
    "chiptune/close": "9328d82ac7979c39c4b2c8d4a74916df",
    "chiptune/confirm": "7217c11321ab61903edb571eea00070e",
    "chiptune/countdown": "166734d09c2e1e02bddc82c8b3bce3bc",
    "chiptune/error": "4831c148a9b24e8e8252ef981880db70",
    "chiptune/levelup": "a9046c97a251ec6acc4c70c1dd68e3ba",
    "chiptune/open": "c8ea7f25603976a0268dc92b77eef189",
    "chiptune/purchase": "391c0c39ea2f69dfea1c9e407bb10255",
    "chiptune/reward": "dd9f90cff599544b29603df446931876",
    "chiptune/tab": "ab9d5be97e981e09e4b536a874b051cc",
    "chiptune/tap": "fd00de8c5625e410323343dc9ee210c2",
    "chiptune/toggle": "930c648e97db91c0cd11eb0cc516af13",
    "cute/back": "24dd806088fa1c6a0cc640131ffb29a5",
    "cute/close": "e018d5ab0c7aa25c1d8254c5206aa4c5",
    "cute/confirm": "f55edff8fd95531857bd66f7bbdc99d3",
    "cute/countdown": "fe9493b3505c7cba3b212c5341be0b70",
    "cute/error": "4028c8d2f1e08745d61ca4dd4c47fcca",
    "cute/levelup": "0ec533784149852bc5f1336a345860da",
    "cute/open": "e55c8701dd520bf09c33907bfb3bc26e",
    "cute/purchase": "655e9e0e70ec1e37ad9711969e60a541",
    "cute/reward": "e847bc630d63e227e1b0bd2850564209",
    "cute/tab": "cb55e17235164d82b9bbb037f5dea199",
    "cute/tap": "5dd82046af7ff291664b7e759bb20ec9",
    "cute/toggle": "6cbff20ac83a96404905c83fb3ea1490",
    "glass/back": "7bd47d16d097f6d14fa7290cbb289edb",
    "glass/close": "7f19342215453d22272d98960d68179d",
    "glass/confirm": "287d26f71cfd2682b47733755d3e6581",
    "glass/countdown": "9f40aa441419c3f25f09f8bb27f26f22",
    "glass/error": "507207289686ee33fb8baa1f8d87b26e",
    "glass/levelup": "1267b9d503f83e6e5021e794160c8018",
    "glass/open": "ab1df0944f62b31740b36f701373707d",
    "glass/purchase": "fd7f693042766a5e8430c4310a7fa710",
    "glass/reward": "b1ab70d6afb29582df67008db21d9075",
    "glass/tab": "a264bdbe35a72574e1ba5b6c5b9e4617",
    "glass/tap": "8b612c3bbdb57f416dc28c4cac7e6974",
    "glass/toggle": "64e198903f0bc7434ec38a5b40aa177c",
    "mechanical/back": "18ca7b61a8bb1dc5a0145e0b7c686293",
    "mechanical/close": "d8f357ac842ce8d689aac7893b2aa25d",
    "mechanical/confirm": "651fd292cb88697c5e2b8323b2721ac1",
    "mechanical/countdown": "619303f14b2ec82cc842803673f29108",
    "mechanical/error": "30cc43109ba4e4cc2076672dd90c48b2",
    "mechanical/levelup": "d34118fd48909fbffb1845bfc09c3c9c",
    "mechanical/open": "04c4bf50d1cc26b22979aa390286a49f",
    "mechanical/purchase": "564ad022891e0f92746a6630681c1c43",
    "mechanical/reward": "178a321f8f187fe48b933c4bfc65b228",
    "mechanical/tab": "896a9edc8eb6b2a2b6bcc0b1ae673456",
    "mechanical/tap": "d7805342a3c078e8c94fac56e18b4948",
    "mechanical/toggle": "1f631c4810fb5a5c3babc86612e4006f",
    "soft/back": "075c41647d96eaff8b0ae233d6605921",
    "soft/close": "e72e951240ec88c2b9403fb2d6ef3c41",
    "soft/confirm": "a1f7a63589bef9264e94b7ff5e8cb6c3",
    "soft/countdown": "be4aeafbf1b47b0db56b0278c8734217",
    "soft/error": "cf3e1ad5bf672b72070be12861164711",
    "soft/levelup": "cf04c020d2c86f970d39aa675bfb3dc0",
    "soft/open": "174bccb10e083071a9827f1b0405db39",
    "soft/purchase": "047526bb71dbc3157e10d08f65e879df",
    "soft/reward": "9df881c38604073241584c08d98247d5",
    "soft/tab": "2db1b7eae8b1682cf66148550bdef3ae",
    "soft/tap": "0e72a4e0c5441bd954ef089466feb62b",
    "soft/toggle": "96e7e157b2f4a2cb3958428b36e90c85",
    "wood/back": "fa5af1c6546c617bcf6bbb296759fbe6",
    "wood/close": "2ed1ec5744f947e81b0ef5db0b116784",
    "wood/confirm": "c5851876b23c10dd6c1e8d22289ef6ca",
    "wood/countdown": "f0d2cdd73a6040bc53afcb1e6cbfe954",
    "wood/error": "871e06cb8c9b6beaa26884991da83921",
    "wood/levelup": "3f997d4165ec511783090dfa1d406059",
    "wood/open": "cc843cd85c1c45eef7864e37f43d6096",
    "wood/purchase": "b3306065a4c12a30edc852ba87e22a49",
    "wood/reward": "76a8f0165677006d2a81710eb1d8bb7c",
    "wood/tab": "2f5e0fa00a17eaae90a3e178a9777f00",
    "wood/tap": "03b3d79d9ee91a4cab53af3ec3c80229",
    "wood/toggle": "d7ef59b152cb8f10b28d3df0b97b07d3",
}


def load(path):
    raw = subprocess.run(["ffmpeg", "-nostdin", "-v", "error", "-i", path, "-ac", "1", "-ar", str(RATE),
                          "-f", "f32le", "-"], capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype=np.float32).astype(np.float64)


def master(x, rms_db, longest):
    x = x - x.mean()
    peak = np.abs(x).max()
    win = 240
    n = len(x) // win
    env = np.sqrt((x[: n * win].reshape(n, win) ** 2).mean(axis=1) + 1e-15)
    start = int(np.argmax(env > peak * 10 ** (-40 / 20))) * win
    loud = np.nonzero(env > peak * 10 ** (-50 / 20))[0]
    end = min((loud[-1] + 1) * win, start + int(longest * RATE))
    x = x[max(0, start - 48):end].copy()
    fade = min(len(x) // 4, int(0.012 * RATE))
    x[-fade:] *= np.linspace(1, 0, fade) ** 2
    x[:48] *= np.linspace(0, 1, 48)
    n = len(x) // win
    env = np.sqrt((x[: n * win].reshape(n, win) ** 2).mean(axis=1) + 1e-15)
    act = env > env.max() * 10 ** (-30 / 20)
    rms = np.sqrt((x[: n * win].reshape(n, win)[act] ** 2).mean())
    gain = min(10 ** (rms_db / 20) / rms, 10 ** (-1 / 20) / np.abs(x).max())
    return (x * gain).astype(np.float32)


def write(path, x):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    subprocess.run(["ffmpeg", "-nostdin", "-v", "error", "-y", "-f", "f32le", "-ar", str(RATE), "-ac", "1",
                    "-i", "-", "-c:a", "libvorbis", "-q:a", "5", "-map_metadata", "-1",
                    "-fflags", "+bitexact", "-flags:a", "+bitexact", path], input=x.tobytes(), check=True)


def main():
    raw = sys.argv[1]
    repo = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(__file__), "..", "..")
    for name in SETS:
        out = os.path.join(repo, "templates", "styles", "shared", "sfx", name)
        for event, (rms_db, longest) in MIX.items():
            x = master(load(os.path.join(raw, name, f"{event}.mp3")), rms_db, longest)
            write(os.path.join(out, f"{event}.ogg"), x)
            print(f"{name}/{event}: {len(x) / RATE:.2f} s")


if __name__ == "__main__":
    main()
