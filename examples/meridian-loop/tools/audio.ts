// Every sound of the game, synthesised: the engine and tyre loops (pitched
// and faded by src/sound.ts), the impacts, the race's cues, the UI and the
// music ("Meridian", a 100 bpm synthwave loop of 24 bars, 57.6 s, stereo
// 48 kHz). Deterministic: seeded noise, no clock.

import { rng } from "./media.ts";

export const RATE = 48000;

/** A one-pole low-pass over a buffer, in place. */
function lowpass(buf: Float32Array, cutoff: number): Float32Array {
  const a = 1 - Math.exp((-2 * Math.PI * cutoff) / RATE);
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
export function engine(): Float32Array {
  const n = RATE;
  const out = new Float32Array(n);
  const r = rng(5);
  const noise = new Float32Array(n);
  for (let i = 0; i < n; i++) noise[i] = r() * 2 - 1;
  lowpass(noise, 900);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const f = 55;
    let v = 0;
    for (let h = 1; h <= 14; h++) {
      const amp = (h === 2 ? 1.3 : h === 4 ? 0.8 : 1) / h ** 1.1;
      v += Math.sin(2 * Math.PI * f * h * t + h * 0.7) * amp;
    }
    // Firing pulses: the noise gated twice a cycle.
    const pulse = 0.5 + 0.5 * Math.sin(2 * Math.PI * f * 2 * t);
    v += noise[i]! * pulse * pulse * 1.4;
    out[i] = Math.tanh(v * 0.9);
  }
  return normalise(out, 0.8);
}

/** Tyres sliding: a squeal over hiss, one second, looping. */
export function skid(): Float32Array {
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

/** A struck wall: a low thump and a scrape of metal. */
export function bump(): Float32Array {
  const n = Math.round(RATE * 0.45);
  const out = new Float32Array(n);
  const r = rng(23);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    const thump = Math.sin(2 * Math.PI * (70 - t * 60) * t) * Math.exp(-t * 14);
    const grit = (r() * 2 - 1) * Math.exp(-t * 9) * 0.6;
    const ring = Math.sin(2 * Math.PI * 1730 * t) * Math.exp(-t * 18) * 0.25 + Math.sin(2 * Math.PI * 2410 * t) * Math.exp(-t * 22) * 0.18;
    out[i] = thump + grit + ring;
  }
  return normalise(lowpass(out, 5000), 0.9);
}

/** A landing after the jump: the suspension bottoming out. */
export function land(): Float32Array {
  const n = Math.round(RATE * 0.4);
  const out = new Float32Array(n);
  const r = rng(29);
  for (let i = 0; i < n; i++) {
    const t = i / RATE;
    out[i] = Math.sin(2 * Math.PI * (55 - t * 30) * t) * Math.exp(-t * 9) + (r() * 2 - 1) * Math.exp(-t * 30) * 0.5;
  }
  return normalise(lowpass(out, 1800), 0.95);
}

/** A tone with a quick attack and an exponential tail. */
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

export const beep = () => tone([660], 0.18, 18);
export const go = () => tone([1320, 1980], 0.45, 7, 0, 0.75);
export const sector = () => tone([988, 1319], 0.3, 12, 0.07);
export const lap = () => tone([784, 988, 1175], 0.45, 8, 0.09);
export const best = () => tone([523, 659, 784, 1047, 1319], 0.6, 5, 0.1, 0.8);
export const slower = () => tone([440, 370], 0.3, 10, 0.1, 0.55);
export const hover = () => tone([1760], 0.05, 60, 0, 0.18);
export const click = () => tone([1200, 1600], 0.07, 40, 0.03, 0.4);
export const open = () => tone([660, 990], 0.12, 22, 0.05, 0.35);
export const close = () => tone([990, 660], 0.12, 22, 0.05, 0.35);
export const error = () => tone([220, 207], 0.16, 14, 0.06, 0.5);

/** "Meridian": interleaved stereo, 24 bars at 100 bpm. */
export function music(): Float32Array {
  const bpm = 100;
  const beat = 60 / bpm;
  const bars = 24;
  const len = Math.round(bars * 4 * beat * RATE);
  const L = new Float32Array(len);
  const R = new Float32Array(len);
  const r = rng(77);
  const mtof = (m: number) => 440 * 2 ** ((m - 69) / 12);
  // Am F C G, two bars each, three times round.
  const roots = [57, 53, 48, 55];
  const chords = [
    [57, 60, 64],
    [53, 57, 60],
    [48, 52, 55],
    [55, 59, 62],
  ];
  const put = (buf: Float32Array, at: number, v: number) => {
    if (at >= 0 && at < len) buf[at] += v;
  };
  for (let bar = 0; bar < bars; bar++) {
    const c = Math.floor(bar / 2) % 4;
    const barStart = bar * 4 * beat;
    const intro = bar < 4;
    for (let b = 0; b < 4; b++) {
      const t0 = Math.round((barStart + b * beat) * RATE);
      // Kick on every beat (from bar 2), snare with a long noise tail on 2 and 4 (from bar 4).
      if (bar >= 2) {
        for (let i = 0; i < RATE * 0.3; i++) {
          const t = i / RATE;
          const v = Math.sin(2 * Math.PI * (110 * Math.exp(-t * 18) + 42) * t) * Math.exp(-t * 9) * 0.55;
          put(L, t0 + i, v);
          put(R, t0 + i, v);
        }
      }
      if (!intro && (b === 1 || b === 3)) {
        for (let i = 0; i < RATE * 0.35; i++) {
          const t = i / RATE;
          const v = ((r() * 2 - 1) * Math.exp(-t * 11) + Math.sin(2 * Math.PI * 190 * t) * Math.exp(-t * 30)) * 0.22;
          put(L, t0 + i, v);
          put(R, t0 + i, v * 0.9);
        }
      }
      // Hats on the off-beats.
      if (bar >= 2) {
        const th = t0 + Math.round(beat * 0.5 * RATE);
        for (let i = 0; i < RATE * 0.05; i++) {
          const v = (r() * 2 - 1) * Math.exp(-(i / RATE) * 80) * 0.07;
          put(L, th + i, v * 0.7);
          put(R, th + i, v);
        }
      }
      // The bass: sixteenths of the root, octave jumps on the off sixteenth.
      for (let k = 0; k < 4; k++) {
        const ts = t0 + Math.round((k * beat * RATE) / 4);
        const note = roots[c]! - 24 + (k % 2 === 1 ? 12 : 0);
        const f = mtof(note);
        const n = Math.round((beat / 4) * RATE * 0.9);
        for (let i = 0; i < n; i++) {
          const t = i / RATE;
          const ph = (f * t) % 1;
          const saw = 2 * ph - 1;
          const env = Math.exp(-t * 7) * (intro ? 0.5 : 1);
          const v = Math.tanh(saw * 1.5) * env * 0.2;
          put(L, ts + i, v);
          put(R, ts + i, v);
        }
      }
    }
    // The pad: the chord held across the bar, detuned saws, slow attack.
    const p0 = Math.round(barStart * RATE);
    const pn = Math.round(4 * beat * RATE);
    for (const note of chords[c]!) {
      const f = mtof(note);
      for (let i = 0; i < pn; i++) {
        const t = i / RATE;
        const env = Math.min(1, t * 2) * Math.min(1, (pn - i) / (RATE * 0.2));
        const a = ((f * 1.003 * t) % 1) * 2 - 1;
        const b = ((f * 0.997 * t) % 1) * 2 - 1;
        put(L, p0 + i, a * env * 0.035);
        put(R, p0 + i, b * env * 0.035);
      }
    }
    // A lead line in the last eight bars of each sixteen: a falling figure on the chord's tones.
    if (bar % 16 >= 8 || bar >= 16) {
      const line = [chords[c]![2]! + 12, chords[c]![1]! + 12, chords[c]![0]! + 12, chords[c]![1]! + 12];
      line.forEach((note, k) => {
        const ts = p0 + Math.round(k * beat * RATE);
        const f = mtof(note);
        for (let i = 0; i < beat * RATE * 0.95; i++) {
          const t = i / RATE;
          const vib = 1 + 0.004 * Math.sin(2 * Math.PI * 5.5 * t);
          const sq = Math.sin(2 * Math.PI * f * vib * t) > 0 ? 1 : -1;
          const env = Math.min(1, t * 60) * Math.exp(-t * 1.8);
          put(L, ts + i, sq * env * 0.035);
          put(R, ts + i + 2400, sq * env * 0.03);
        }
      });
    }
  }
  lowpass(L, 7000);
  lowpass(R, 7000);
  const out = new Float32Array(len * 2);
  let peak = 0;
  for (let i = 0; i < len; i++) peak = Math.max(peak, Math.abs(L[i]!), Math.abs(R[i]!));
  const g = 0.85 / peak;
  for (let i = 0; i < len; i++) {
    out[i * 2] = L[i]! * g;
    out[i * 2 + 1] = R[i]! * g;
  }
  return out;
}
