// Rumpus Isle's asset generator: every source asset the game cooks is built
// here, deterministically (seeded, no clock, no download).
//
//   bun tools/make-assets.ts             everything, the music included
//   bun tools/make-assets.ts --no-music  everything but assets/music/romp.wav
//
// What it writes under assets/:
// - models/island.glb: the round island the rounds are played on (a grass
//   top with a painted ring, a sand rim and a rounded rock side), and
//   models/islet.glb, models/palm.glb, models/tile.glb (a bevelled tile the
//   crumble round lays out and tints per row), models/podium.glb (three
//   steps), models/berry.glb (a berry with a leaf).
// - audio/*.ogt: og sfx presets (og-audio §2), synthesised by og cook;
//   audio/splash.wav: a splash, noise through a falling band-pass.
// - music/romp.wav: "Rumpus Romp", 16 bars at 128 bpm (30 s, 44.1 kHz
//   stereo), cooked in chunks and streamed (music/romp.wav.ogt). About 5 MB,
//   so it is not committed (.gitignore).
// - fx/*.ogt: the emitters (a splash, dash dust, a berry's sparkle, a tile's
//   crumbs, the podium's confetti).
// - styles/rumpus/: the engine's cartoon style pack copied from
//   templates/ui-kit (its atlas, icons and fonts are not committed:
//   .gitignore), recoloured sea and coral, its CJK fallback line dropped.

import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { band, bevelBox, Builder, disc, glb, ring, type Mat, type Part, type V3 } from "./glb.ts";

const ROOT = resolve(import.meta.dir ?? ".", "..");
const ASSETS = join(ROOT, "assets");
const ENGINE = resolve(ROOT, "..", "..");

function write(rel: string, data: Uint8Array | string): void {
  const p = join(ASSETS, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, data);
}

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

const hex = (h: string): [number, number, number] => {
  // sRGB to linear, as glTF's baseColorFactor is linear.
  const c = [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
  return c.map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4)) as [number, number, number];
};

// -------------------------------------------------------------------- models

/** The island: radius 7.2 at the top (y 0), a sand rim from 6.6, a rounded rock side down to y -1.6. */
function island(): void {
  const mats: Mat[] = [
    { name: "grass", color: hex("#7cc35a"), roughness: 0.92 },
    { name: "ring", color: hex("#a6dc7c"), roughness: 0.9 },
    { name: "sand", color: hex("#f1d494"), roughness: 0.95 },
    { name: "rock", color: hex("#b88a64"), roughness: 0.9 },
  ];
  const seg = 96;
  const wob = (a: number) => 1 + 0.012 * Math.sin(a * 5) + 0.008 * Math.sin(a * 11 + 1);
  const grass = new Builder(0);
  disc(grass, 5.6, 0, seg, true, wob);
  ring(grass, 5.95, 6.45, 0, seg, wob);
  const paint = new Builder(1);
  ring(paint, 5.6, 5.95, 0, seg, wob);
  const sand = new Builder(2);
  ring(sand, 6.45, 6.9, 0, seg, wob);
  // The rounded edge: a quarter round from the top's rim down the side.
  const steps = 5;
  for (let i = 0; i < steps; i++) {
    const a0 = (i / steps) * (Math.PI / 2), a1 = ((i + 1) / steps) * (Math.PI / 2);
    const r = 0.3;
    const p0: [number, number] = [6.9 + Math.sin(a0) * r, -r + Math.cos(a0) * r];
    const p1: [number, number] = [6.9 + Math.sin(a1) * r, -r + Math.cos(a1) * r];
    // Going outwards and down: draw from the lower point up so the band faces out.
    band(sand, p1[0], p1[1], p0[0], p0[1], seg, [Math.sin(a1), Math.cos(a1), 0], [Math.sin(a0), Math.cos(a0), 0], wob);
  }
  const rock = new Builder(3);
  band(rock, 6.5, -1.7, 7.2, -0.3, seg, undefined, undefined, wob);
  disc(rock, 6.5, -1.7, seg, false, wob);
  write("models/island.glb", glb([{ name: "island", parts: [grass.part(), paint.part(), sand.part(), rock.part()] }], mats));
}

/** A small sandy islet with a rock, for the sea around. */
function islet(): void {
  const mats: Mat[] = [
    { name: "sand", color: hex("#f1d494"), roughness: 0.95 },
    { name: "rock", color: hex("#9d8577"), roughness: 0.9 },
  ];
  const seg = 40;
  const sand = new Builder(0);
  disc(sand, 1.4, 0.15, seg);
  band(sand, 1.9, -0.6, 1.4, 0.15, seg);
  const rock = new Builder(1);
  bevelBox(rock, [0.7, 0.55, 0.6], 0.12, [0.35, 0.4, -0.2]);
  bevelBox(rock, [0.4, 0.3, 0.45], 0.08, [-0.45, 0.28, 0.35]);
  write("models/islet.glb", glb([{ name: "islet", parts: [sand.part(), rock.part()] }], mats));
}

/** A palm: a leaning trunk of rings and six drooping fronds. */
function palm(): void {
  const mats: Mat[] = [
    { name: "trunk", color: hex("#a2774f"), roughness: 0.9 },
    { name: "frond", color: hex("#3f9d4b"), roughness: 0.85 },
  ];
  const trunk = new Builder(0);
  const rings = 6;
  for (let i = 0; i < rings; i++) {
    const y0 = i * 0.42, y1 = (i + 1) * 0.42;
    const lean = (y: number) => 0.08 * y * y;
    const t0 = trunk.positions.length / 3;
    band(trunk, 0.17 - i * 0.012, y0, 0.13 - i * 0.012, y1, 12);
    // Lean the ring along +x.
    for (let v = t0; v < trunk.positions.length / 3; v++) trunk.positions[v * 3] = trunk.positions[v * 3]! + lean(trunk.positions[v * 3 + 1]!);
  }
  const frond = new Builder(1);
  const top: V3 = [0.08 * 2.52 * 2.52, 2.52, 0];
  for (let k = 0; k < 6; k++) {
    const a = (k / 6) * Math.PI * 2 + 0.3;
    const dir: V3 = [Math.cos(a), 0, Math.sin(a)];
    const side: V3 = [-dir[2], 0, dir[0]];
    const pts: V3[] = [];
    for (let s = 0; s <= 4; s++) {
      const d = s * 0.32;
      pts.push([top[0] + dir[0] * d, top[1] + 0.25 * s - 0.11 * s * s, top[2] + dir[2] * d]);
    }
    for (let s = 0; s < 4; s++) {
      const w0 = 0.22 * Math.sin(((s + 0.5) / 4) * Math.PI), w1 = 0.22 * Math.sin(((s + 1.5) / 4) * Math.PI) * (s === 3 ? 0 : 1);
      const p0 = pts[s]!, p1 = pts[s + 1]!;
      const n: V3 = [0, 1, 0];
      const a0 = frond.vertex([p0[0] + side[0] * w0, p0[1], p0[2] + side[2] * w0], n);
      const b0 = frond.vertex([p0[0] - side[0] * w0, p0[1], p0[2] - side[2] * w0], n);
      const a1 = frond.vertex([p1[0] + side[0] * w1, p1[1], p1[2] + side[2] * w1], n);
      const b1 = frond.vertex([p1[0] - side[0] * w1, p1[1], p1[2] - side[2] * w1], n);
      frond.quad(a0, a1, b1, b0);
      frond.quad(b0, b1, a1, a0);
    }
  }
  write("models/palm.glb", glb([{ name: "palm", parts: [trunk.part(), frond.part()] }], mats));
}

/** The crumble round's tile: 1.5 m square, 0.5 m thick, top at y 0; white, tinted per row by og.Model.tint. */
function tile(): void {
  const mats: Mat[] = [
    { name: "top", color: [1, 1, 1], roughness: 0.75 },
    { name: "side", color: hex("#d9cbb4"), roughness: 0.9 },
  ];
  const top = new Builder(0);
  bevelBox(top, [1.44, 0.16, 1.44], 0.06, [0, -0.08, 0]);
  const side = new Builder(1);
  bevelBox(side, [1.32, 0.4, 1.32], 0.05, [0, -0.36, 0]);
  write("models/tile.glb", glb([{ name: "tile", parts: [top.part(), side.part()] }], mats));
}

/** The podium: the winner's step in the middle (1.2 m), second left (0.8), third right (0.5); the front faces +z. */
function podium(): void {
  const mats: Mat[] = [
    { name: "gold", color: hex("#f2c14e"), roughness: 0.45, metalness: 0.2 },
    { name: "silver", color: hex("#c9d3dc"), roughness: 0.45, metalness: 0.2 },
    { name: "bronze", color: hex("#d08a54"), roughness: 0.5, metalness: 0.2 },
  ];
  const steps: [number, number, number][] = [[0, 1.2, 0], [-1.7, 0.8, 1], [1.7, 0.5, 2]];
  const parts: Part[] = steps.map(([x, h, m]) => {
    const b = new Builder(m);
    bevelBox(b, [1.6, h, 1.6], 0.1, [x, h / 2, 0]);
    return b.part();
  });
  write("models/podium.glb", glb([{ name: "podium", parts }], mats));
}

/** A berry: a round fruit with two leaves, 0.5 m across, resting on y 0. */
function berry(): void {
  const mats: Mat[] = [
    { name: "fruit", color: hex("#e2365b"), roughness: 0.35 },
    { name: "leaf", color: hex("#3f9d4b"), roughness: 0.8 },
  ];
  const fruit = new Builder(0);
  const rings = 8, seg = 16, r = 0.25;
  for (let i = 0; i < rings; i++) {
    const a0 = (i / rings) * Math.PI, a1 = ((i + 1) / rings) * Math.PI;
    // From the bottom pole up, so each band faces out.
    const y0 = r - Math.cos(a0) * r, y1 = r - Math.cos(a1) * r;
    band(fruit, Math.sin(a0) * r + 1e-4, y0, Math.sin(a1) * r + 1e-4, y1, seg, [Math.sin(a0), -Math.cos(a0), 0], [Math.sin(a1), -Math.cos(a1), 0]);
  }
  const leaf = new Builder(1);
  for (const s of [1, -1]) {
    const n: V3 = [0, 1, 0];
    const a = leaf.vertex([0, 0.48, 0], n);
    const b = leaf.vertex([0.12 * s, 0.56, 0.07], n);
    const c = leaf.vertex([0.26 * s, 0.52, 0], n);
    const d = leaf.vertex([0.12 * s, 0.56, -0.07], n);
    leaf.quad(a, b, c, d);
    leaf.quad(a, d, c, b);
  }
  write("models/berry.glb", glb([{ name: "berry", parts: [fruit.part(), leaf.part()] }], mats));
}

// -------------------------------------------------------------------- sound

/** og sfx presets (og-audio §2): each a text asset og cook synthesises. */
function sfx(): void {
  const presets: [string, string, number, number, number, number][] = [
    // file, preset, seed, pitch, length, volume
    ["bump", "hit", 4, 1.35, 0.5, 0.8],
    ["dash", "whoosh", 2, 1.4, 0.45, 0.55],
    ["berry", "coin", 3, 1.1, 0.8, 0.6],
    ["gold", "powerup", 5, 1.2, 0.7, 0.6],
    ["crack", "door", 6, 1.6, 0.4, 0.5],
    ["crumble", "explosion", 7, 1.8, 0.35, 0.45],
    ["beep", "select", 1, 1.0, 0.7, 0.55],
    ["go", "levelup", 2, 1.3, 0.8, 0.7],
    ["whistle", "laser", 9, 0.6, 0.9, 0.45],
    ["win", "levelup", 4, 1.0, 1.4, 0.75],
    ["join", "pickup", 2, 1.0, 0.8, 0.6],
    ["leave", "back", 1, 0.9, 0.8, 0.5],
    ["ready", "select", 3, 1.25, 0.8, 0.55],
    ["ui-hover", "click", 2, 1.4, 1, 0.25],
    ["ui-click", "click", 1, 1.0, 1, 0.55],
    ["ui-select", "select", 1, 1.0, 1, 0.55],
    ["ui-back", "back", 1, 1.0, 1, 0.5],
    ["ui-error", "error", 1, 1.0, 1, 0.5],
  ];
  for (const [file, preset, seed, pitch, length, volume] of presets) {
    write(`audio/${file}.ogt`, `type sfx\nversion 1\n---\n# og sfx's ${preset} preset (tools/make-assets.ts): og.load.sound(world, "audio/${file}.ogt") plays it.\npreset "${preset}"\nseed ${seed}\npitch ${pitch}\nlength ${length}\nvolume ${volume}\n`);
  }
}

function wav(rate: number, channels: number, samples: Float32Array): Uint8Array {
  const n = samples.length;
  const out = new Uint8Array(44 + n * 2);
  const dv = new DataView(out.buffer);
  const str = (at: number, s: string) => { for (let i = 0; i < s.length; i++) out[at + i] = s.charCodeAt(i); };
  str(0, "RIFF");
  dv.setUint32(4, 36 + n * 2, true);
  str(8, "WAVE");
  str(12, "fmt ");
  dv.setUint32(16, 16, true);
  dv.setUint16(20, 1, true);
  dv.setUint16(22, channels, true);
  dv.setUint32(24, rate, true);
  dv.setUint32(28, rate * channels * 2, true);
  dv.setUint16(32, channels * 2, true);
  dv.setUint16(34, 16, true);
  str(36, "data");
  dv.setUint32(40, n * 2, true);
  for (let i = 0; i < n; i++) dv.setInt16(44 + i * 2, Math.round(Math.max(-1, Math.min(1, samples[i]!)) * 32767), true);
  return out;
}

/** A splash: a noise burst through a band-pass falling from 2.4 kHz, and a few bubbles after. */
function splash(): Uint8Array {
  const rate = 44100, n = Math.round(rate * 0.75);
  const out = new Float32Array(n);
  const r = rng(11);
  let lp = 0, bp = 0;
  for (let i = 0; i < n; i++) {
    const t = i / rate;
    const f = 300 + 2100 * Math.exp(-t * 7);
    const k = Math.min(0.9, (2 * Math.PI * f) / rate);
    const w = r() * 2 - 1;
    lp += (w - lp) * k;
    bp += (lp - bp) * k * 0.5;
    const env = Math.min(1, t * 200) * Math.exp(-t * 6);
    out[i] = (lp - bp) * env * 1.8;
  }
  for (let b = 0; b < 5; b++) {
    const t0 = 0.18 + b * 0.09 + r() * 0.04;
    const f0 = 500 + r() * 500;
    for (let i = Math.round(t0 * rate); i < Math.min(n, Math.round((t0 + 0.08) * rate)); i++) {
      const t = i / rate - t0;
      out[i] = out[i]! + 0.18 * Math.sin(2 * Math.PI * (f0 + 2500 * t) * t) * Math.exp(-t * 40);
    }
  }
  let peak = 0;
  for (const v of out) peak = Math.max(peak, Math.abs(v));
  for (let i = 0; i < n; i++) out[i] = (out[i]! / peak) * 0.8;
  return wav(rate, 1, out);
}

/**
 * "Rumpus Romp": 16 bars at 128 bpm in F major (I vi IV V), a loop that ends
 * where it starts: a bouncy bass, marimba chords on the off-beats, a whistled
 * tune over the middle eight bars, a kick, a clap and a shaker.
 */
function music(): Uint8Array {
  const RATE = 44100, BPM = 128, BARS = 16;
  const beat = 60 / BPM;
  const N = Math.round(BARS * 4 * beat * RATE);
  const L = new Float32Array(N), R = new Float32Array(N);
  const r = rng(7);
  const midi = (m: number) => 440 * 2 ** ((m - 69) / 12);
  const add = (t0: number, dur: number, voice: (t: number) => number, pan: number) => {
    const a = Math.round(t0 * RATE), len = Math.round(dur * RATE);
    const gl = Math.cos(((pan + 1) * Math.PI) / 4), gr = Math.sin(((pan + 1) * Math.PI) / 4);
    for (let i = 0; i < len; i++) {
      const v = voice(i / RATE);
      const k = (a + i) % N;
      L[k] = L[k]! + v * gl;
      R[k] = R[k]! + v * gr;
    }
  };
  const marimba = (f: number, amp: number) => (t: number) => amp * Math.exp(-t * 7) * (Math.sin(2 * Math.PI * f * t) + 0.25 * Math.sin(2 * Math.PI * f * 4 * t) * Math.exp(-t * 30));
  const bass = (f: number, amp: number) => (t: number) => amp * Math.exp(-t * 3.5) * Math.tanh(2.2 * Math.sin(2 * Math.PI * f * t));
  const whistle = (f: number, amp: number, dur: number) => (t: number) => amp * Math.min(1, t * 30) * Math.min(1, (dur - t) * 20) * Math.sin(2 * Math.PI * f * t + 0.004 * f * Math.sin(2 * Math.PI * 5.5 * t));
  const kick = (amp: number) => (t: number) => amp * Math.exp(-t * 18) * Math.sin(2 * Math.PI * (50 + 120 * Math.exp(-t * 35)) * t);
  const noise = (amp: number, decay: number, seed: number) => {
    const g = rng(seed);
    let lp = 0;
    return (t: number) => {
      const w = g() * 2 - 1;
      lp += (w - lp) * 0.6;
      return amp * Math.exp(-t * decay) * (w - lp);
    };
  };
  // F, Dm, Bb, C: root and the chord tones above.
  const CHORDS = [[53, 65, 69, 72], [50, 62, 65, 69], [46, 62, 65, 70], [48, 64, 67, 72]];
  const TUNE = [77, 76, 74, 72, 74, 76, 77, 79, 81, 79, 77, 76, 74, 72, 74, 72];
  for (let bar = 0; bar < BARS; bar++) {
    const chord = CHORDS[bar % 4]!;
    const t0 = bar * 4 * beat;
    for (const [b, oct] of [[0, 0], [1.5, 12], [2, 0], [3, 7]] as const) add(t0 + b * beat, beat * 1.2, bass(midi(chord[0]! - 12 + oct), 0.2), 0);
    for (let e = 0; e < 4; e++) for (const [i, m] of chord.slice(1).entries()) add(t0 + (e + 0.5) * beat, 0.7, marimba(midi(m), 0.045), i === 0 ? -0.4 : i === 1 ? 0 : 0.4);
    for (let q = 0; q < 4; q++) add(t0 + q * beat, 0.3, kick(q % 2 === 0 ? 0.5 : 0.3), 0);
    for (const q of [1, 3]) add(t0 + q * beat, 0.25, noise(0.22, 22, bar * 4 + q), 0.15);
    for (let h = 0; h < 8; h++) add(t0 + h * beat * 0.5, 0.06, noise(h % 2 ? 0.05 : 0.08, 70, 900 + bar * 8 + h), -0.3);
    if (bar >= 4 && bar < 12) {
      for (let q = 0; q < 2; q++) {
        const m = TUNE[((bar - 4) * 2 + q) % TUNE.length]!;
        const dur = beat * (r() < 0.3 ? 1.5 : 1.8);
        add(t0 + q * 2 * beat, dur, whistle(midi(m), 0.1, dur), 0.25);
        if (r() < 0.5) add(t0 + (q * 2 + 1.5) * beat, beat * 0.45, whistle(midi(m + 2), 0.08, beat * 0.45), 0.25);
      }
    }
  }
  let peak = 0;
  for (let i = 0; i < N; i++) peak = Math.max(peak, Math.abs(L[i]!), Math.abs(R[i]!));
  const gain = 0.7 / peak;
  const inter = new Float32Array(N * 2);
  for (let i = 0; i < N; i++) {
    inter[i * 2] = L[i]! * gain;
    inter[i * 2 + 1] = R[i]! * gain;
  }
  return wav(RATE, 2, inter);
}

// -------------------------------------------------------------------- effects

function emitter(file: string, lines: string): void {
  write(`fx/${file}.ogt`, `type particles\nversion 1\n---\n${lines.trim()}\n`);
}

function fx(): void {
  // A critter hits the sea: a crown of droplets and a puff of spray.
  emitter("splash", `
# A critter hits the sea (tools/make-assets.ts).
maxParticles 96
shape cone 35 0.4
rate 0
burst 0 60
duration 0.1
looping false
lifetime 0.5 0.9
speed 3.5 6.5
size 0.12 0.24
sizeOverLife 0 1 1 0.4
colorOverLife {
  key 0 0.85 0.95 1 1
  key 0.7 0.6 0.85 1 0.8
  key 1 0.6 0.85 1 0
}
gravity 0 -14 0
drag 0.6
blend alpha
`);
  // A dash's dust at the feet.
  emitter("dust", `
# A dash's dust at the feet (tools/make-assets.ts).
maxParticles 48
shape sphere 0.25
rate 0
burst 0 14
duration 0.1
looping false
lifetime 0.35 0.6
speed 0.6 1.6
size 0.18 0.3
sizeOverLife 0 0.6 1 1.4
colorOverLife {
  key 0 1 0.95 0.85 0.8
  key 1 1 0.95 0.85 0
}
gravity 0 0.6 0
drag 3
blend alpha
`);
  // A berry taken: a ring of sparkles.
  emitter("sparkle", `
# A berry taken (tools/make-assets.ts).
maxParticles 48
shape sphere 0.2 surface
rate 0
burst 0 22
duration 0.1
looping false
lifetime 0.35 0.6
speed 1.5 3
size 0.08 0.14
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.95 0.6 1
  key 1 1 0.5 0.6 0
}
color 2 1.6 1.2 1
gravity 0 1.5 0
drag 2
blend additive
`);
  // A tile breaking up.
  emitter("crumbs", `
# A tile breaking up (tools/make-assets.ts).
maxParticles 64
shape box 0.7 0.05 0.7
rate 0
burst 0 26
duration 0.1
looping false
lifetime 0.5 0.9
speed 0.5 1.8
size 0.1 0.2
colorOverLife {
  key 0 0.85 0.78 0.66 1
  key 1 0.7 0.62 0.5 0
}
gravity 0 -9 0
blend alpha
`);
  // Confetti over the podium, while the winners stand there.
  emitter("confetti", `
# Confetti over the podium (tools/make-assets.ts).
maxParticles 400
shape box 3 0.2 1.2
spread 25
rate 90
lifetime 2.5 3.5
speed 0.5 1.5
size 0.1 0.16
rotation 0 360
angularVelocity -400 400
colorOverLife {
  key 0 1 1 1 1
  key 0.85 1 1 1 1
  key 1 1 1 1 0
}
gravity 0 -2.2 0
noise 0.8 0.6 0.4
drag 0.8
blend alpha
`);
}

// ------------------------------------------------------------------- style

/**
 * The cartoon style pack (the catalogue's style for miniature-diorama), copied
 * from the engine's templates/ui-kit as og-taste §1 says: its ground a deep
 * sea blue, its accent a coral. Its CJK `fallback` line is dropped (English
 * and French only, Latin-1).
 */
function style(): void {
  const from = join(ENGINE, "templates", "styles", "cartoon");
  const to = join(ASSETS, "styles", "rumpus");
  mkdirSync(to, { recursive: true });
  for (const f of ["atlas.png", "atlas.png.ogt", "icons.png", "icons.png.ogt", "Fredoka-SemiBold.ttf", "Fredoka-SemiBold.ttf.ogt", "LilitaOne-Regular.ttf", "LilitaOne-Regular.ttf.ogt", "OFL.txt", "LICENSES.md"]) copyFileSync(join(from, f), join(to, f));
  let text = readFileSync(join(from, "style.ogt"), "utf8");
  const colours: Record<string, string> = {
    backdrop: "#123a5c",
    well: "#174870",
    primary: "#b23a1e",
    primaryHover: "#b9401f",
    primaryActive: "#962f17",
    highlight: "#a3341a",
    primarySoft: "#b23a1e40",
    scrim: "#0b2238cc",
    gold: "#b23a1e",
  };
  for (const [k, v] of Object.entries(colours)) text = text.replace(new RegExp(`^(  ${k}) "#[0-9a-fA-F]+"`, "m"), `$1 "${v}"`);
  text = text.replace(/^name "cartoon"$/m, 'name "rumpus"');
  text = text.replace(/^# The cartoon style pack.*$/m, "# Rumpus Isle's style: the engine's cartoon style pack (templates/ui-kit, CC0 art, OFL fonts:\n# LICENSES.md), copied by tools/make-assets.ts, its ground a deep sea blue and its accent coral;\n# the CJK fallback line dropped, the game shows Latin-1 only.");
  text = text.replace(/^  fallback .*\n/m, "");
  writeFileSync(join(to, "style.ogt"), text);
}

// -------------------------------------------------------------------- main

island();
islet();
palm();
tile();
podium();
berry();
sfx();
fx();
style();
write("audio/splash.wav", splash());
write("music/romp.wav.ogt", "type import\nversion 1\n---\n# The music (tools/make-assets.ts): cooked in 4-second chunks and\n# streamed, never resident whole (og-audio §5).\nsound {\n  stream true\n  chunk 4\n}\n");
if (!process.argv.includes("--no-music")) write("music/romp.wav", music());
console.log(`made Rumpus Isle's assets under ${ASSETS}`);
