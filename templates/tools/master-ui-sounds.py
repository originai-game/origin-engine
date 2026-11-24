#!/usr/bin/env python3
"""Masters the UI sound set: the default pack's raw clip of each event, as
generated (origin's generate_sfx, the prompts below), trimmed, faded and
levelled to the event's loudness, written as Ogg Vorbis to where the pack
names them:

    templates/styles/shared/sfx/default/<event>.ogg

    python3 templates/tools/master-ui-sounds.py <raw dir> [repo] [--only=tick,step]

`<raw dir>/<style>/<event>.mp3` are the generated originals (MEDIA names
each one's media id on the platform). Needs numpy and ffmpeg (libvorbis).

The mix: every clip starts at its first sound (no lead silence, so a press
sounds on the frame it lands), ends at its tail's -50 dB with a 12 ms
fade, is cut to its event's longest length, and is levelled so its active
part (the frames within 30 dB of its peak) has its event's RMS, its peak
held at -1 dBFS. Quiet events (tap, toggle, tab) sit 5 dB under the
stingers (reward, levelup), so a pack's `volume` sets taste, not balance.
"""
import os, subprocess, sys
import numpy as np

RATE = 48000

# event: (active RMS dBFS, longest seconds)
MIX = {
    "tap": (-22, 0.15), "toggle": (-22, 0.2), "tab": (-22, 0.25), "countdown": (-20, 0.25),
    "back": (-21, 0.4), "close": (-21, 0.45), "open": (-21, 0.5), "confirm": (-19, 0.6),
    "error": (-19, 0.6), "purchase": (-18, 1.0), "reward": (-18, 1.3), "levelup": (-17, 2.0),
    "tick": (-24, 0.12), "step": (-24, 0.12), "swipe": (-22, 0.3), "notify": (-20, 0.6),
    "unlock": (-19, 0.9),
}

STYLES = {
    "default": "clean modern mobile RPG menu UI (Genshin Impact / Honkai Star Rail style): soft glassy chimes, crystal and airy synth bells, polished, gentle",
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
    "purchase": "a satisfying purchase sound, coins clinking with a cash register ka-ching chime, about 0.8 seconds",
    "countdown": "a single short countdown tick beep for a 3-2-1 timer, one clean beep only, under 0.2 seconds",
    "tick": "a tiny single soft tick for a number counting up, one tiny click, extremely short, nothing after it",
    "step": "a tiny dry detent click for a slider moving one notch, extremely short, nothing after it",
    "swipe": "a quick light swipe whoosh for turning a page or swiping a card sideways, ends within 0.25 seconds",
    "notify": "a short friendly notification ping, two light ascending notes, ends within 0.5 seconds",
    "unlock": "a satisfying unlock sound, a latch clicks open followed by a short bright shimmer, about 0.6 seconds",
}
PROMPT = "{event}. Style: {style}. Single isolated sound effect, dry, no music bed, no voice, no background noise."

# style/event: the generated original's media id on the platform.
MEDIA = {
    "default/back": "edf7a74e01f2eb0e55bcd4113db9e9bf",
    "default/close": "46503c25c31ca4a6e409aabac5ab4bb9",
    "default/confirm": "a136d9fed6c172dbf7cbea0a7fb23826",
    "default/countdown": "3af3e6b9c95b8d5e8bc3f93b5f027279",
    "default/error": "30a2b3638a72e3dbef690c31b9da23c5",
    "default/levelup": "08cc0bdbf5fe6f2e9f6c0f6acec7cad9",
    "default/open": "f8bd9c0022fdb65c4137ba9bfe0039af",
    "default/purchase": "f89dc9ce95d47ce87ed6a3280bcefb9e",
    "default/reward": "7673620657d029661dbd64b7f5b4d6b6",
    "default/tab": "3ffc01053eff577cdf4c86c4a9a51981",
    "default/tap": "9587e27d9e9e39a1f3176588c71801ba",
    "default/toggle": "0840d149f3f29a0873c57e7036f4a792",
    "default/notify": "69711efff1e8658fb934d796acb155a4",
    "default/step": "c28c7691bf4f05fcfd8e47e1e06e24e6",
    "default/swipe": "86b0770b94ef7d93a2164ba1a6bdfb70",
    "default/tick": "5c79a66870b162f07ec6294ed1192bfd",
    "default/unlock": "4d5d5c62df1773f0fc82640655667941",
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
    args = [a for a in sys.argv[1:] if not a.startswith("--only=")]
    only = [a.split("=", 1)[1].split(",") for a in sys.argv[1:] if a.startswith("--only=")]
    raw = args[0]
    repo = args[1] if len(args) > 1 else os.path.join(os.path.dirname(__file__), "..", "..")
    for style in STYLES:
        out = os.path.join(repo, "templates", "styles", "shared", "sfx", style)
        for event, (rms_db, longest) in MIX.items():
            if only and event not in only[0]:
                continue
            x = master(load(os.path.join(raw, style, f"{event}.mp3")), rms_db, longest)
            write(os.path.join(out, f"{event}.ogg"), x)
            print(f"{style}/{event}: {len(x) / RATE:.2f} s")


if __name__ == "__main__":
    main()
