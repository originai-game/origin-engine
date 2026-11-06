// Every sound of Redline Ring, synthesised: `bun tools/make-assets.ts`
// writes assets/audio/*.wav byte for byte (seeded noise, no clock). The
// engine and tyre loops, the contacts, the race's cues and "Redline", a
// 132 bpm arcade loop of eight bars. The engine, tyres, contacts and the
// cue tones follow examples/meridian-loop's tools/audio.ts (the engine's
// own CC0 work); the boost, the finish and the music are this game's.
//
// The circuit, the cars and the scenery are not assets: src/track.ts and
// src/mesh.ts build them in `start`. The UI style is assets/styles/redline,
// a copy of the engine's broadcast pack; the smoke and spark textures and
// emitters are examples/meridian-loop's.

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const RATE = 48000;
const MUSIC_RATE = 32000;
const root = join(dirname(fileURLToPath(import.meta.url)), "..", "assets");

/** mulberry32: a small seeded generator, the same numbers on every machine. */
function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function lowpass(buf: Float32Array, cutoff: number, rate = RATE): Float32Array {
  const a = 1 - Math.exp((-2 * Math.PI * cutoff) / rate);
  let y = 0;
  for (let i = 0; i < buf.length; i++) {
    y += a * (buf[i]! - y);
    buf[i] = y;
  }
  return buf;
}

function highpass(buf: Float32Array, cutoff: number): Float32Array {
  const lp = lowpass(Float32Array.from(buf), cutoff);
  for (let i = 0; i < buf.length; i++) buf[i] = buf[i]! - lp[i]!;
  return buf;
}

function normalise(buf: Float32Array, peak: number): Float32Array {
  let m = 0;
  for (const v of buf) m = Math.max(m, Math.abs(v));
  if (m > 0) for (let i = 0; i < buf.length; i++) buf[i] = (buf[i]! / m) * peak;
  return buf;
}

/** Crossfades the last `fade` samples into the first, so the buffer loops without a click. */
function loopable(buf: Float32Array, fade: number): Float32Array {
  const n = buf.length - fade;
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = buf[i]!;
  for (let i = 0; i < fade; i++) {
    const t = i / fade;
    out[i] = buf[n + i]! * (1 - t) + buf[i]! * t;
  }
  return out;
}

/** The engine at pitch 1: a 55 Hz four-cylinder burble, one second, looping (55 whole cycles). */
function engine(): Float32Array {
  const n = RATE;
  const out = new Float32Array(n);
  const r = rng(5);
  const noise = new Float32Array(n);
  for (let i = 0; i < n; i++) noise[i] = r() * 2 - 1;
  lowpass(noise, 900);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    let v = 0;
    for (let h = 1; h <= 14; h++) v += (Math.sin(2 * Math.PI * 55 * h * t + h * 0.7) * (h === 2 ? 1.3 : h === 4 ? 0.8 : 1)) / h ** 1.1;
    const pulse = 0.5 + 0.5 * Math.sin(2 * Math.PI * 110 * t);
    v += noise[i]! * pulse * pulse * 1.4;
    out[i] = Math.tanh(v * 0.9);
  }
  return normalise(out, 0.8);
}

/** Tyres sliding: a squeal over hiss, one second, looping. */
function skid(): Float32Array {
  const n = RATE + 4800;
  const out = new Float32Array(n);
  const r = rng(17);
  for (let i = 0; i < n; i++) out[i] = r() * 2 - 1;
  highpass(lowpass(out, 3200), 900);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const wob = 1 + 0.03 * Math.sin(2 * Math.PI * 5.3 * t) + 0.015 * Math.sin(2 * Math.PI * 13.1 * t);
    out[i] = out[i]! * 0.9 + Math.sin(2 * Math.PI * 1180 * wob * t) * 0.25 + Math.sin(2 * Math.PI * 2360 * wob * t) * 0.08;
  }
  return normalise(loopable(out, 4800), 0.7);
}

/** A knock between cars, or a barrier brushed: a low thump and a scrape. */
function bump(): Float32Array {
  const n = Math.round(RATE * 0.4);
  const out = new Float32Array(n);
  const r = rng(23);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const thump = Math.sin(2 * Math.PI * (70 - t * 60) * t) * Math.exp(-t * 14);
    const grit = (r() * 2 - 1) * Math.exp(-t * 11) * 0.5;
    const ring = Math.sin(2 * Math.PI * 1730 * t) * Math.exp(-t * 18) * 0.2;
    out[i] = thump + grit + ring;
  }
  return normalise(lowpass(out, 5000), 0.85);
}

/** A barrier hit hard: a heavier thump, crunching metal and a long scrape. */
function crash(): Float32Array {
  const n = Math.round(RATE * 0.8);
  const out = new Float32Array(n);
  const r = rng(31);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const thump = Math.sin(2 * Math.PI * (55 - t * 40) * t) * Math.exp(-t * 8) * 1.2;
    const crunch = (r() * 2 - 1) * Math.exp(-t * 6) * (0.6 + 0.4 * Math.sin(2 * Math.PI * 23 * t));
    const metal = (Math.sin(2 * Math.PI * 1310 * t) + 0.7 * Math.sin(2 * Math.PI * 2077 * t) + 0.5 * Math.sin(2 * Math.PI * 2930 * t)) * Math.exp(-t * 7) * 0.22;
    out[i] = thump + crunch + metal;
  }
  return normalise(lowpass(out, 6500), 0.95);
}

/** A tone with a quick attack and an exponential tail, notes `gap` seconds apart. */
function tone(freqs: readonly number[], length: number, decay: number, gap = 0, peak = 0.7): Float32Array {
  const n = Math.round(RATE * (length + gap * (freqs.length - 1)));
  const out = new Float32Array(n);
  freqs.forEach((f, k) => {
    const start = Math.round(RATE * gap * k);
    for (let i = 0; start + i < n; i++) {
      const t = i / RATE;
      const env = Math.min(1, t * 400) * Math.exp(-t * decay);
      out[start + i] += env * (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(2 * Math.PI * f * 2 * t) + 0.12 * Math.sin(2 * Math.PI * f * 3 * t));
    }
  });
  return normalise(out, peak);
}

/** The boost's whoosh: rising filtered noise and a lifting tone. */
function boost(): Float32Array {
  const n = Math.round(RATE * 0.7);
  const out = new Float32Array(n);
  const r = rng(41);
  let y = 0;
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const cut = 300 + 5000 * Math.min(1, t / 0.35);
    const a = 1 - Math.exp((-2 * Math.PI * cut) / RATE);
    y += a * (r() * 2 - 1 - y);
    const env = Math.min(1, t * 30) * Math.exp(-Math.max(0, t - 0.25) * 6);
    out[i] = y * env + Math.sin(2 * Math.PI * (300 + 900 * t) * t) * env * 0.25;
  }
  return normalise(out, 0.75);
}

/** The finish: a bright fanfare climbing to a held chord. */
function finish(): Float32Array {
  const notes = [523.25, 659.25, 783.99, 1046.5];
  const head = tone(notes, 0.25, 9, 0.11, 0.7);
  const n = head.length + Math.round(RATE * 1.1);
  const out = new Float32Array(n);
  out.set(head);
  const at = Math.round(RATE * 0.44);
  for (const f of [523.25, 659.25, 783.99, 1046.5]) {
    for (let i = 0; at + i < n; i++) {
      const t = i / RATE;
      const env = Math.min(1, t * 60) * Math.exp(-t * 2.2);
      out[at + i] += env * 0.3 * (Math.sin(2 * Math.PI * f * t) + 0.4 * Math.sin(2 * Math.PI * f * 2.005 * t));
    }
  }
  return normalise(out, 0.8);
}

/** "Redline": eight bars at 132 bpm, mono: a driving bass, four-on-the-floor, a stab on the offbeats and a lead hook. */
function music(): Float32Array {
  const rate = MUSIC_RATE;
  const beat = 60 / 132;
  const bars = 8;
  const len = Math.round(bars * 4 * beat * rate);
  const out = new Float32Array(len);
  const r = rng(77);
  const mtof = (m: number) => 440 * 2 ** ((m - 69) / 12);
  const put = (at: number, v: number) => {
    if (at >= 0 && at < len) out[at] += v;
  };
  // E minor's i - VI - III - VII: Em C G D, a bar each, twice round.
  const roots = [40, 36, 43, 38];
  const chords = [[64, 67, 71], [60, 64, 67], [67, 71, 74], [62, 66, 69]];
  for (let bar = 0; bar < bars; bar++) {
    const c = bar % 4;
    const barStart = bar * 4 * beat;
    for (let b = 0; b < 4; b++) {
      const t0 = Math.round((barStart + b * beat) * rate);
      for (let i = 0; i < rate * 0.25; i++) {
        const t = i / rate;
        put(t0 + i, Math.sin(2 * Math.PI * (120 * Math.exp(-t * 22) + 45) * t) * Math.exp(-t * 10) * 0.6);
      }
      if (b === 1 || b === 3) {
        for (let i = 0; i < rate * 0.2; i++) {
          const t = i / rate;
          put(t0 + i, ((r() * 2 - 1) * Math.exp(-t * 16) + Math.sin(2 * Math.PI * 200 * t) * Math.exp(-t * 35)) * 0.25);
        }
      }
      for (let k = 0; k < 2; k++) {
        const th = t0 + Math.round(((k + 0.5) * beat * rate) / 2);
        for (let i = 0; i < rate * 0.04; i++) put(th + i, (r() * 2 - 1) * Math.exp(-(i / rate) * 90) * 0.08);
      }
      // Bass: eighths on the root, the octave on the off eighth.
      for (let k = 0; k < 2; k++) {
        const ts = t0 + Math.round((k * beat * rate) / 2);
        const f = mtof(roots[c]! + (k === 1 ? 12 : 0));
        for (let i = 0; i < (beat / 2) * rate * 0.92; i++) {
          const t = i / rate;
          const saw = 2 * ((f * t) % 1) - 1;
          put(ts + i, Math.tanh(saw * 1.8) * Math.exp(-t * 5) * 0.22);
        }
      }
      // The stab on the offbeat: the chord, short and bright.
      const ts = t0 + Math.round((beat * rate) / 2);
      for (const note of chords[c]!) {
        const f = mtof(note);
        for (let i = 0; i < beat * rate * 0.3; i++) {
          const t = i / rate;
          const sq = ((f * t) % 1) < 0.5 ? 1 : -1;
          put(ts + i, sq * Math.exp(-t * 14) * 0.035);
        }
      }
    }
    // The hook in the second half: a rising figure on the chord's tones.
    if (bar >= 4) {
      const line = [chords[c]![0]! + 12, chords[c]![1]! + 12, chords[c]![2]! + 12, chords[c]![1]! + 12, chords[c]![2]! + 12, chords[c]![0]! + 24, chords[c]![2]! + 12, chords[c]![1]! + 12];
      line.forEach((note, k) => {
        const ts = Math.round((barStart + (k * beat) / 2) * rate);
        const f = mtof(note);
        for (let i = 0; i < (beat / 2) * rate * 0.9; i++) {
          const t = i / rate;
          const vib = 1 + 0.003 * Math.sin(2 * Math.PI * 6 * t);
          const v = (2 * ((f * vib * t) % 1) - 1) * Math.min(1, t * 80) * Math.exp(-t * 4);
          put(ts + i, v * 0.05);
        }
      });
    }
  }
  return normalise(lowpass(out, 7500, rate), 0.85);
}

function wav(samples: Float32Array, rate: number): Uint8Array {
  const pcm = new Int16Array(samples.length);
  for (let i = 0; i < samples.length; i++) pcm[i] = Math.round(Math.max(-1, Math.min(1, samples[i]!)) * 32767);
  const out = new Uint8Array(44 + pcm.byteLength);
  const dv = new DataView(out.buffer);
  const tag = (at: number, s: string) => {
    for (let i = 0; i < 4; i++) out[at + i] = s.charCodeAt(i);
  };
  tag(0, "RIFF");
  dv.setUint32(4, 36 + pcm.byteLength, true);
  tag(8, "WAVE");
  tag(12, "fmt ");
  dv.setUint32(16, 16, true);
  dv.setUint16(20, 1, true);
  dv.setUint16(22, 1, true);
  dv.setUint32(24, rate, true);
  dv.setUint32(28, rate * 2, true);
  dv.setUint16(32, 2, true);
  dv.setUint16(34, 16, true);
  tag(36, "data");
  dv.setUint32(40, pcm.byteLength, true);
  out.set(new Uint8Array(pcm.buffer), 44);
  return out;
}

const write = (name: string, f: () => Float32Array, rate = RATE) => {
  const path = join(root, "audio", name);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, wav(f(), rate));
};

write("engine.wav", engine);
write("skid.wav", skid);
write("bump.wav", bump);
write("crash.wav", crash);
write("beep.wav", () => tone([660], 0.2, 16));
write("go.wav", () => tone([1320, 1980], 0.5, 6, 0, 0.75));
write("sector.wav", () => tone([988, 1319], 0.3, 12, 0.07, 0.6));
write("lap.wav", () => tone([784, 988, 1175], 0.45, 8, 0.09));
write("best.wav", () => tone([523, 659, 784, 1047, 1319], 0.6, 5, 0.1, 0.8));
write("boost.wav", boost);
write("finish.wav", finish);
write("redline.wav", music, MUSIC_RATE);
