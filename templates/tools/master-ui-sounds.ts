// Masters the default UI sound set. Raw <style>/<event>.mp3 clips become
// mono 48 kHz Vorbis, trimmed, faded and levelled to the event's active RMS.
//   bun templates/tools/master-ui-sounds.ts <raw dir> [repo] [--only=tick,step]
// Prompts and original platform media IDs are recorded below.
import { mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const RATE = 48000;

/** The same 5 ms envelope windows and f64 arithmetic as the original master. */
export function master(input: Float32Array, rmsDb: number, longest: number): Float32Array {
  const mean = input.reduce((a, b) => a + b, 0) / input.length;
  let x = Float64Array.from(input, (v) => v - mean);
  const peak = (a: Float64Array) => a.reduce((p, v) => Math.max(p, Math.abs(v)), 0);
  const envelope = (a: Float64Array) =>
    Float64Array.from({ length: Math.floor(a.length / 240) }, (_, i) => {
      let sum = 0;
      for (let j = i * 240; j < (i + 1) * 240; j++) sum += a[j]! ** 2;
      return Math.sqrt(sum / 240 + 1e-15);
    });
  let env = envelope(x);
  const p = peak(x);
  const start =
    Math.max(
      0,
      env.findIndex((v) => v > p * 10 ** (-40 / 20)),
    ) * 240;
  const tail = env.findLastIndex((v) => v > p * 10 ** (-50 / 20));
  if (tail < 0) throw new Error("clip has no audible windows");
  const end = Math.min((tail + 1) * 240, start + Math.floor(longest * RATE));
  x = x.slice(Math.max(0, start - 48), end);
  const fade = Math.min(Math.floor(x.length / 4), Math.floor(0.012 * RATE));
  for (let i = 0; i < fade; i++) x[x.length - fade + i]! *= (1 - i / (fade - 1)) ** 2;
  for (let i = 0; i < 48; i++) x[i]! *= i / 47;
  env = envelope(x);
  const threshold = Math.max(...env) * 10 ** (-30 / 20);
  let sum = 0,
    count = 0;
  for (let i = 0; i < env.length; i++)
    if (env[i]! > threshold) {
      for (let j = i * 240; j < (i + 1) * 240; j++) sum += x[j]! ** 2;
      count += 240;
    }
  const rms = Math.sqrt(sum / count);
  const gain = Math.min(10 ** (rmsDb / 20) / rms, 10 ** (-1 / 20) / peak(x));
  return Float32Array.from(x, (v) => v * gain);
}

export function run(sets: Record<string, string>, mix: Record<string, number[]>): void {
  const args = process.argv.slice(2).filter((a) => !a.startsWith("--only="));
  const only = process.argv
    .find((a) => a.startsWith("--only="))
    ?.slice(7)
    .split(",");
  if (!args[0]) throw new Error("expected <raw dir> [repo] [--only=tick,step]");
  const repo = args[1] ?? resolve(import.meta.dir, "../..");
  for (const set of Object.keys(sets))
    for (const [event, [rms, longest]] of Object.entries(mix)) {
      if (only && !only.includes(event)) continue;
      const decoded = Bun.spawnSync(
        [
          "ffmpeg",
          "-nostdin",
          "-v",
          "error",
          "-i",
          join(args[0], set, `${event}.mp3`),
          "-ac",
          "1",
          "-ar",
          String(RATE),
          "-f",
          "f32le",
          "-",
        ],
        { maxBuffer: 64 * 1024 * 1024 },
      );
      if (decoded.exitCode) throw new Error(decoded.stderr.toString());
      const bytes = decoded.stdout;
      const input = new Float32Array(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength));
      const x = master(input, rms!, longest!);
      const path = join(repo, "templates/styles/shared/sfx", set, `${event}.ogg`);
      mkdirSync(dirname(path), { recursive: true });
      const encoded = Bun.spawnSync(
        [
          "ffmpeg",
          "-nostdin",
          "-v",
          "error",
          "-y",
          "-f",
          "f32le",
          "-ar",
          String(RATE),
          "-ac",
          "1",
          "-i",
          "-",
          "-c:a",
          "libvorbis",
          "-q:a",
          "5",
          "-map_metadata",
          "-1",
          "-fflags",
          "+bitexact",
          "-flags:a",
          "+bitexact",
          path,
        ],
        { stdin: new Uint8Array(x.buffer) },
      );
      if (encoded.exitCode) throw new Error(encoded.stderr.toString());
      console.log(`${set}/${event}: ${(x.length / RATE).toFixed(2)} s`);
    }
}

export const MIX = {
  tap: [-22, 0.15],
  toggle: [-22, 0.2],
  tab: [-22, 0.25],
  countdown: [-20, 0.25],
  back: [-21, 0.4],
  close: [-21, 0.45],
  open: [-21, 0.5],
  confirm: [-19, 0.6],
  error: [-19, 0.6],
  purchase: [-18, 1.0],
  reward: [-18, 1.3],
  levelup: [-17, 2.0],
  tick: [-24, 0.12],
  step: [-24, 0.12],
  swipe: [-22, 0.3],
  notify: [-20, 0.6],
  unlock: [-19, 0.9],
};

export const STYLES = {
  default:
    "clean modern mobile RPG menu UI (Genshin Impact / Honkai Star Rail style): soft glassy chimes, crystal and airy synth bells, polished, gentle",
};

export const EVENTS = {
  tap: "a single very short soft button tap, one tiny click with a hint of tone, under 0.1 seconds, nothing after it",
  confirm: "a short bright positive confirm sound, two quick rising notes, ends within 0.4 seconds",
  back: "a short soft falling two-note cancel or back sound, ends within 0.3 seconds",
  error: "a short gentle error or denied buzz, two low quick notes, not harsh, ends within 0.4 seconds",
  open: "a short soft panel open whoosh rising upward, ends within 0.4 seconds",
  close: "a short soft panel close whoosh falling downward, ends within 0.35 seconds",
  toggle: "a tiny crisp switch toggle click, very short, nothing after it",
  tab: "a tiny soft tab switch swish with a light tick, very short",
  reward: "a rewarding item collect jingle, sparkling ascending arpeggio with a shimmering tail, about 1 second",
  levelup: "a triumphant level up fanfare stinger, rising arpeggio landing on a bright major chord with sparkle, about 1.6 seconds",
  purchase: "a satisfying purchase sound, coins clinking with a cash register ka-ching chime, about 0.8 seconds",
  countdown: "a single short countdown tick beep for a 3-2-1 timer, one clean beep only, under 0.2 seconds",
  tick: "a tiny single soft tick for a number counting up, one tiny click, extremely short, nothing after it",
  step: "a tiny dry detent click for a slider moving one notch, extremely short, nothing after it",
  swipe: "a quick light swipe whoosh for turning a page or swiping a card sideways, ends within 0.25 seconds",
  notify: "a short friendly notification ping, two light ascending notes, ends within 0.5 seconds",
  unlock: "a satisfying unlock sound, a latch clicks open followed by a short bright shimmer, about 0.6 seconds",
};

export const PROMPT = "{event}. Style: {style}. Single isolated sound effect, dry, no music bed, no voice, no background noise.";

export const MEDIA = {
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
};

if (import.meta.main) run(STYLES, MIX);
