#!/usr/bin/env python3
"""Masters the UI sound set: each pack style's raw clip of each event, as
generated (origin's generate_sfx, the prompts below), trimmed, faded and
levelled to the event's loudness, written as Ogg Vorbis to where the packs
name them:

    templates/styles/shared/sfx/default/<event>.ogg   (the default pack's)
    templates/styles/<style>/sfx/<event>.ogg           (scifi, fantasy, cartoon)

    python3 templates/tools/master-ui-sounds.py <raw dir> [repo]

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
}

STYLES = {
    "default": "clean modern mobile RPG menu UI (Genshin Impact / Honkai Star Rail style): soft glassy chimes, crystal and airy synth bells, polished, gentle",
    "scifi": "sci-fi holographic interface UI: crisp digital blips, synth tones, servo and energy shimmer, futuristic HUD",
    "fantasy": "fantasy RPG menu UI: warm magical harp and celesta, soft metal and wood, enchanted sparkle",
    "cartoon": "cartoon casual mobile game UI (Clash Royale / Brawl Stars style): bouncy bubbly pops, marimba and xylophone, playful and punchy",
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
}
PROMPT = "{event}. Style: {style}. Single isolated sound effect, dry, no music bed, no voice, no background noise."

# style/event: the generated original's media id on the platform.
MEDIA = {
    "cartoon/back": "2495d45c5db0e594310add3bc8d91818",
    "cartoon/close": "b5def7d0c1afbc472cc3721ff01d775f",
    "cartoon/confirm": "a5eff643d00634ec237f518e4d7825a8",
    "cartoon/countdown": "c97efc2ec481aa73b6f231ab6feae731",
    "cartoon/error": "29a9313989f1319d8091ea0bc62ed562",
    "cartoon/levelup": "c617b166b3413f073dc58c03d11c9552",
    "cartoon/open": "d7532ffb7337f96578b4837f65eaf9a0",
    "cartoon/purchase": "0b74aae291687c864f09990961191bae",
    "cartoon/reward": "056563c4c4da78ea3d1785c39ea8dac1",
    "cartoon/tab": "3106574e854d73ea988bafa93d23a29f",
    "cartoon/tap": "26274e80d2051527a18b4943ff060773",
    "cartoon/toggle": "0d9bde364e35fa55d8e62f6c567e4507",
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
    "fantasy/back": "707ca5b1ac8ab2978f63552c1093ed6f",
    "fantasy/close": "e2f3883d8b866dae4d4c792168a4e554",
    "fantasy/confirm": "0e1cbc05216e47650c9774dd8b605f13",
    "fantasy/countdown": "73545422b66c0c8f04285ba1b43b0dd2",
    "fantasy/error": "f3a0b71caac94554f3212207a8085b13",
    "fantasy/levelup": "f220236673fd2ab7d76be4aebd3826ac",
    "fantasy/open": "6604c47ee21b4632b3e15fe23bf6b16d",
    "fantasy/purchase": "5bc2d85cbd9458a961e8263fbf13600f",
    "fantasy/reward": "9b4284bb3b749c4fa0785064525e2395",
    "fantasy/tab": "7fb5aaef910492d891066c629985c518",
    "fantasy/tap": "25c9cd70273765a8ae2db2a8a0ddc5ca",
    "fantasy/toggle": "6e0016b05f0b0ca646a23bd78e5863f8",
    "scifi/back": "d4287a064c581d8959542bf36e60131f",
    "scifi/close": "e018cda0a52e000d580dcd588f028598",
    "scifi/confirm": "edaf6a8a67e7f3385b310406b0f83aa8",
    "scifi/countdown": "caa9e43c141cdd054ac6b6697ed94d79",
    "scifi/error": "6d0e2e5441a5e2d6f2697505f4988a5a",
    "scifi/levelup": "94c7727f29bef1531c685acf83b22259",
    "scifi/open": "532436b0cbe31b29973808a91ede65c9",
    "scifi/purchase": "d05c0a52192cb2727c151913a476caa9",
    "scifi/reward": "24a501233f21a529c2527d40d3d244f5",
    "scifi/tab": "bc843acdcc90f544724880b190e4d742",
    "scifi/tap": "4a73a04aeda318eea159a1bf734ccb7c",
    "scifi/toggle": "0a0dc1edff8c4d5ec12dc82d0c055f00",
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
    for style in STYLES:
        out = os.path.join(repo, "templates", "styles", *(["shared", "sfx", "default"] if style == "default" else [style, "sfx"]))
        for event, (rms_db, longest) in MIX.items():
            x = master(load(os.path.join(raw, style, f"{event}.mp3")), rms_db, longest)
            write(os.path.join(out, f"{event}.ogg"), x)
            print(f"{style}/{event}: {len(x) / RATE:.2f} s")


if __name__ == "__main__":
    main()
