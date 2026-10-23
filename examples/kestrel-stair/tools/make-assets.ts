// The Kestrel Stair's asset generator: every source asset of the game is
// made here, deterministically (seeded generators, no clock), with nothing
// downloaded but the Japanese font's source (tools/fetch-font.ts).
//
//   bun tools/make-assets.ts            everything, the music included
//   bun tools/make-assets.ts --no-music everything but assets/music/stair.wav
//
// What it writes under assets/:
// - models/stair.glb: the level (tools/level.ts from src/layout.ts): the
//   court, the ramps, the crypt terrace, the crypt, the ledges, the stairs,
//   the high terrace, the gate wall, the summit; textured stone, paving,
//   grass, plaster and wood.
// - nav/stair.ogt and ai/<foe>.ogt: the navmesh baked from the level and the
//   foes' behaviours, their rounds from src/layout.ts.
// - anims/humanoid.glb, models/courier.glb, imp.glb, warden.glb (+ the
//   courier's import settings and rigs/warden.ogt): tools/rig.ts.
// - textures/*.png: the cape and the banners' cloth, the particles' spark,
//   flame and dust (the level's own textures are inside models/stair.glb).
// - env/dusk.hdr: a 64 x 32 synthetic late-afternoon sky.
// - fx/*.ogt: brazier fire, sigil burst, hit sparks, slam dust.
// - audio/*.ogt: og sfx presets; audio/*.wav: a brazier's crackle, the
//   crypt's drip, the imps' chitter and the summit bell, synthesised here.
// - music/stair.wav: "The Kestrel Stair", 2 minutes 40 at 84 bpm, cooked in
//   4-second chunks (music/stair.wav.ogt). About 15 MB: not committed.
// - skins/kestrel/: the engine's paper style pack, copied and retinted.

import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { FOES } from "../src/layout.ts";
import { behaviour, level, navmesh } from "./level.ts";
import { clamp, draw, fbm, hash2, hdr, hex, mix, rng, smoothstep, vnoise, wav } from "./media.ts";
import { character, library, wardenRig } from "./rig.ts";

const ROOT = resolve(import.meta.dir ?? ".", "..");
const ASSETS = join(ROOT, "assets");
const ENGINE = resolve(ROOT, "..", "..");

function write(rel: string, data: Uint8Array | string): void {
  const p = join(ASSETS, rel);
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, data);
}

// ----------------------------------------------------------------- textures

/** A tileable texture: `colour(u, v)` in 0..1, wrapped at `period` noise cells. */
function tileable(size: number, colour: (u: number, v: number, x: number, y: number) => [number, number, number]): Uint8Array {
  return draw(size, (u, v, x, y) => [...colour(u, v, x, y), 255]);
}

function shade(a: string, b: string, t: number): [number, number, number] {
  const x = hex(a);
  const y = hex(b);
  return [mix(x[0], y[0], t), mix(x[1], y[1], t), mix(x[2], y[2], t)];
}

/** A wash: soft pigment blotches pooling darker at their rims, over paper-light ground. */
function wash(u: number, v: number, s: number, period: number): number {
  const f = fbm(u * period, v * period, s, 4, period);
  const rim = 1 - Math.abs(f - 0.5) * 4;
  return clamp(f + 0.25 * smoothstep(0.6, 1, rim), 0, 1);
}

export function textures(): Record<"paving" | "stone" | "grass" | "plaster" | "wood", Uint8Array> {
  const paving = tileable(256, (u, v) => {
    // Flagstones of two sizes, washed, with dark joints.
    const cu = u * 4;
    const row = Math.floor(v * 4);
    const off = row % 2 === 0 ? 0 : 0.5;
    const fu = (cu + off) % 1;
    const fv = (v * 4) % 1;
    const joint = Math.min(fu, 1 - fu, fv, 1 - fv);
    const cell = hash2(Math.floor(cu + off) % 4, row, 3);
    const w = wash(u, v, 11, 6);
    const base = shade("#d8cdb8", "#b9a98d", 0.35 * cell + 0.4 * w);
    const j = smoothstep(0.015, 0.05, joint);
    return [mix(110, base[0], j), mix(96, base[1], j), mix(80, base[2], j)];
  });
  const stone = tileable(256, (u, v) => {
    // Coursed ashlar: long blocks, washed grey-ochre.
    const rowH = 1 / 6;
    const row = Math.floor(v / rowH);
    const off = row % 2 === 0 ? 0 : 0.25;
    const fu = ((u + off) * 2) % 1;
    const fv = (v % rowH) / rowH;
    const joint = Math.min(fu, 1 - fu, fv * 0.33, (1 - fv) * 0.33);
    const cell = hash2(Math.floor((u + off) * 2) % 2, row, 7);
    const w = wash(u, v, 21, 5);
    const base = shade("#c9c0ae", "#9c917c", 0.3 * cell + 0.5 * w);
    const j = smoothstep(0.01, 0.035, joint);
    return [mix(96, base[0], j), mix(88, base[1], j), mix(78, base[2], j)];
  });
  const grass = tileable(256, (u, v) => {
    const w = wash(u, v, 31, 4);
    const tuft = fbm(u * 24, v * 24, 41, 2, 24);
    const base = shade("#a9b47a", "#6f8a52", 0.55 * w + 0.3 * tuft);
    const flower = hash2(Math.floor(u * 64), Math.floor(v * 64), 51) > 0.985 ? 1 : 0;
    return flower ? [232, 214, 160] : base;
  });
  const plaster = tileable(256, (u, v) => {
    // Lime plaster: a warm wash, faint hairline cracks, a darker dado band at the foot.
    const w = wash(u, v, 61, 3);
    const n = vnoise(u * 5, v * 5, 71, 5);
    const crack = smoothstep(0.495, 0.5, n) * (1 - smoothstep(0.5, 0.505, n)) * smoothstep(0.4, 0.7, fbm(u * 3, v * 3, 72, 2, 3));
    const base = shade("#e6d3ab", "#c9ab7f", 0.55 * w);
    return [base[0] - 28 * crack, base[1] - 28 * crack, base[2] - 24 * crack];
  });
  const wood = tileable(128, (u, v) => {
    const grain = Math.sin((u * 18 + fbm(u * 4, v * 2, 81, 3, 4) * 3) * Math.PI * 2) * 0.5 + 0.5;
    return shade("#8a5b37", "#5e3b22", 0.5 * grain + 0.3 * wash(u, v, 91, 2));
  });
  return { paving, stone, grass, plaster, wood };
}

function extraTextures(): void {
  // The cape: a deep red wool with a gold border at its hem.
  write("textures/cape.png", tileable(128, (u, v) => {
    const w = wash(u, v, 101, 2);
    const hem = v > 0.88 ? 1 : 0;
    const base = hem ? shade("#c9a14a", "#9c7a30", w) : shade("#a3271c", "#701812", 0.6 * w);
    return base;
  }));
  // The banners: ochre cloth with the kestrel's wedge in dark ink.
  write("textures/banner.png", tileable(128, (u, v) => {
    const w = wash(u, v, 111, 2);
    const cx = u - 0.5;
    const cy = v - 0.42;
    const wing = Math.abs(cy + Math.abs(cx) * 0.7) < 0.05 && Math.abs(cx) < 0.32;
    const body = Math.hypot(cx * 1.6, cy - 0.04) < 0.08;
    const border = u < 0.06 || u > 0.94 || v < 0.05;
    if (wing || body) return shade("#2a2a3a", "#40405a", w);
    if (border) return shade("#5a3a8a", "#3e2766", w);
    return shade("#e2b860", "#c99a3f", 0.7 * w);
  }));
  // Particles: a soft spark, a flame tongue and a dust puff (white: the emitters tint them).
  write("textures/spark.png", draw(64, (u, v) => {
    const d = Math.hypot(u - 0.5, v - 0.5) * 2;
    const a = clamp(1 - d, 0, 1) ** 2.2;
    return [255, 255, 255, a * 255];
  }));
  write("textures/flame.png", draw(64, (u, v) => {
    const x = (u - 0.5) * 2;
    const y = v;
    const shape = clamp(1 - Math.abs(x) / (0.15 + 0.6 * y * (1 - y) * 2.2), 0, 1) * smoothstep(0, 0.25, y) * (1 - smoothstep(0.85, 1, y));
    return [255, 255, 255, shape ** 1.4 * 255];
  }));
  write("textures/dust.png", draw(64, (u, v) => {
    const d = Math.hypot(u - 0.5, v - 0.5) * 2;
    const n = fbm(u * 4, v * 4, 121, 3, 4);
    return [255, 255, 255, clamp((1 - d) * (0.5 + n), 0, 1) * 200];
  }));
}

// -------------------------------------------------------------------- HDR

function environment(): void {
  // Late afternoon over the hills: a warm low sun to the west, a pale blue zenith.
  write("env/dusk.hdr", hdr(64, 32, (u, v) => {
    const el = (0.5 - v) * Math.PI;
    const az = u * Math.PI * 2;
    const up = Math.max(0, Math.sin(el));
    const sky = [mix(1.1, 0.35, up), mix(0.95, 0.55, up), mix(0.75, 0.95, up)];
    const sunAz = Math.PI * 1.5;
    const d = Math.acos(clamp(Math.cos(el) * Math.cos(az - sunAz) * Math.cos(0.15) + Math.sin(el) * Math.sin(0.15), -1, 1));
    const sun = 30 * Math.exp(-((d / 0.06) ** 2)) + 1.5 * Math.exp(-((d / 0.5) ** 2));
    if (el < 0) return [0.32, 0.28, 0.22];
    return [sky[0]! + sun, sky[1]! + sun * 0.8, sky[2]! + sun * 0.55];
  }));
  write("env/dusk.ogt", `type environment\nversion 1\n---\n# The late-afternoon sky (tools/make-assets.ts), prefiltered by og cook.\nsource "dusk.hdr"\nsize 64\nintensity 1\n`);
}

// --------------------------------------------------------------- particles

function effects(): void {
  write("fx/fire.ogt", `type particles
version 1
---
# A brazier's fire: flame tongues rising and fading, a few sparks.
maxParticles 64
shape cone 12 0.22
rate 30
lifetime 0.45 0.8
speed 0.9 1.6
size 0.28 0.45
sizeOverLife 0 0.6 0.4 1 1 0.2
colorOverLife {
  key 0 1 0.85 0.4 0.9
  key 0.5 1 0.45 0.12 0.7
  key 1 0.6 0.15 0.05 0
}
color 3 2 1.2 1
texture "textures/flame.png"
blend additive
noise 0.6 1.5 0.8
`);
  write("fx/sigil.ogt", `type particles
version 1
---
# A sigil taken: a burst of gold motes rising round the courier.
maxParticles 96
shape sphere 0.4
rate 0
burst 0 60
duration 0.2
looping false
lifetime 0.6 1.2
speed 1.5 3
size 0.06 0.12
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.9 0.5 1
  key 1 1 0.7 0.2 0
}
color 3 2.4 1.2 1
gravity 0 1.5 0
drag 2
texture "textures/spark.png"
blend additive
`);
  write("fx/hit.ogt", `type particles
version 1
---
# A sword's hit: sparks flung out, stretched along their flight.
maxParticles 48
shape point
spread 180
rate 0
burst 0 22
duration 0.1
looping false
lifetime 0.15 0.35
speed 3 7
size 0.04 0.08
colorOverLife {
  key 0 1 1 0.85 1
  key 1 1 0.5 0.15 0
}
color 4 3 2 1
gravity 0 -6 0
drag 3
align velocity 0.05
texture "textures/spark.png"
blend additive
`);
  write("fx/dust.ogt", `type particles
version 1
---
# A warden's slam: a ring of dust thrown low across the ground.
maxParticles 80
shape cone 85 0.6
rate 0
burst 0 50
duration 0.15
looping false
lifetime 0.6 1.1
speed 2 4.5
size 0.5 0.9
sizeOverLife 0 0.5 1 1.6
colorOverLife {
  key 0 0.72 0.66 0.56 0.55
  key 1 0.72 0.66 0.56 0
}
gravity 0 -0.5 0
drag 3
texture "textures/dust.png"
blend alpha
`);
}

// ------------------------------------------------------------------- sounds

/** og sfx presets: og cook synthesises each into a sound. */
function presets(): void {
  const p: [string, string, number, number, number, number][] = [
    // file, preset, seed, pitch, length, volume
    ["step", "footstep", 3, 1, 1, 0.8],
    ["swing", "whoosh", 2, 1.25, 0.7, 0.8],
    ["hit", "hit", 4, 1, 1, 1],
    ["hurt", "hurt", 1, 1, 1, 1],
    ["death", "death", 2, 0.9, 1, 1],
    ["jump", "jump", 5, 0.8, 1, 0.6],
    ["land", "land", 1, 1, 1, 0.8],
    ["sigil", "powerup", 3, 1, 1, 0.9],
    ["herb", "pickup", 2, 1, 1, 0.8],
    ["heal", "levelup", 4, 1.2, 0.8, 0.7],
    ["gate", "door", 2, 0.6, 1.6, 1],
    ["waystone", "select", 6, 0.8, 1.4, 0.9],
    ["slam", "explosion", 7, 0.7, 1, 1],
    ["alert", "laser", 5, 0.5, 0.8, 0.6],
    ["bite", "hit", 9, 1.6, 0.6, 0.8],
    ["win", "levelup", 1, 1, 1.4, 1],
    ["lose", "death", 5, 0.7, 1.6, 1],
    ["ui-click", "click", 0, 1, 1, 0.6],
    ["ui-hover", "click", 3, 1.4, 0.5, 0.25],
    ["ui-open", "select", 1, 1, 1, 0.6],
    ["ui-close", "back", 0, 1, 1, 0.6],
    ["ui-error", "error", 0, 1, 1, 0.7],
  ];
  for (const [file, preset, seed, pitch, length, volume] of p) {
    write(`audio/${file}.ogt`, `type sfx\nversion 1\n---\n# og sfx's "${preset}" (tools/make-assets.ts): og.load.sound(world, "audio/${file}.ogt") plays it.\npreset "${preset}"\nseed ${seed}\npitch ${pitch}\nlength ${length}\nvolume ${volume}\n`);
  }
}

/** Sounds synthesised here: loops the spatial sources play, and the bell. */
function synthSounds(): void {
  const RATE = 44100;
  // A brazier's crackle: brown noise with random pops (a 4 s loop).
  {
    const r = rng(7);
    const n = RATE * 4;
    const out = new Float32Array(n);
    let b = 0;
    for (let i = 0; i < n; i++) {
      b = b * 0.98 + (r() * 2 - 1) * 0.02;
      out[i] = b * 2.2;
    }
    for (let k = 0; k < 70; k++) {
      const at = Math.floor(r() * (n - 2000));
      const amp = 0.2 + r() * 0.5;
      for (let i = 0; i < 600; i++) out[at + i]! += (r() * 2 - 1) * amp * Math.exp(-i / 90);
    }
    // Fade the loop's seam.
    for (let i = 0; i < 2000; i++) {
      const f = i / 2000;
      out[i] = out[i]! * f + out[n - 2000 + i]! * (1 - f);
    }
    write("audio/crackle.wav", wav(out.subarray(0, n - 2000), RATE, 1));
  }
  // The crypt's drip: a few drops ringing in the dark (a 6 s loop).
  {
    const n = RATE * 6;
    const out = new Float32Array(n);
    for (const [t, f] of [[0.4, 1450], [1.9, 1180], [3.1, 1620], [4.6, 1320]] as const) {
      const at = Math.floor(t * RATE);
      for (let i = 0; i < RATE * 0.5; i++) {
        const s = i / RATE;
        const pitch = f * (1 + 0.6 * Math.exp(-s * 40));
        out[at + i]! += Math.sin(2 * Math.PI * pitch * s) * Math.exp(-s * 14) * 0.6;
      }
    }
    write("audio/drip.wav", wav(out, RATE, 1));
  }
  // The imps' chitter: clicks in a burst, then quiet (a 3 s loop).
  {
    const r = rng(17);
    const n = RATE * 3;
    const out = new Float32Array(n);
    for (let k = 0; k < 14; k++) {
      const at = Math.floor((0.2 + k * 0.07 + r() * 0.03) * RATE);
      const f = 2200 + r() * 1400;
      for (let i = 0; i < 900; i++) out[at + i]! += Math.sin((2 * Math.PI * f * i) / RATE) * Math.exp(-i / 160) * 0.5;
    }
    write("audio/chitter.wav", wav(out, RATE, 1));
  }
  // The summit bell: inharmonic partials ringing out over 5 s.
  {
    const n = RATE * 5;
    const out = new Float32Array(n);
    const f0 = 196;
    const partials: [number, number, number][] = [[0.5, 0.5, 0.6], [1, 1, 0.8], [1.19, 0.6, 1.2], [1.5, 0.4, 1.6], [2, 0.5, 2], [2.52, 0.3, 2.6], [3.0, 0.2, 3.4]];
    for (let i = 0; i < n; i++) {
      const s = i / RATE;
      let v = 0;
      for (const [ratio, amp, decay] of partials) v += amp * Math.sin(2 * Math.PI * f0 * ratio * s) * Math.exp(-s * decay);
      out[i] = v * 0.25 * Math.min(1, s / 0.004);
    }
    write("audio/bell.wav", wav(out, RATE, 1));
  }
}

/**
 * "The Kestrel Stair": 2:40 at 84 bpm in A dorian, 56 bars of 4/4. A
 * drone and a bowed pad under a plucked ostinato from the start; frame
 * drums from bar 9; a recorder-like melody (a seeded walk over the
 * pentatonic) from bar 17 to 48; the last 8 bars thin out. 48 kHz stereo,
 * the pluck panned left and the melody right.
 */
function music(): Uint8Array {
  const RATE = 48000;
  const BPM = 84;
  const BARS = 56;
  const beat = 60 / BPM;
  const N = Math.round(BARS * 4 * beat * RATE);
  const L = new Float32Array(N);
  const R = new Float32Array(N);
  const midi = (m: number) => 440 * 2 ** ((m - 69) / 12);
  // A dorian: Am7, G6, Fmaj7, Em7 (two bars each).
  const CHORDS = [
    [45, 57, 60, 64, 67],
    [43, 55, 59, 62, 64],
    [41, 57, 60, 64, 65],
    [40, 55, 59, 62, 64],
  ];
  const PENTA = [64, 67, 69, 71, 72, 74, 76, 79];
  const r = rng(84);
  const add = (start: number, dur: number, f: number, amp: number, pan: number, kind: "pad" | "pluck" | "drone" | "flute" | "drum") => {
    const s0 = Math.floor(start * RATE);
    const len = Math.floor(dur * RATE);
    const gl = Math.cos(((pan + 1) * Math.PI) / 4);
    const gr = Math.sin(((pan + 1) * Math.PI) / 4);
    for (let i = 0; i < len && s0 + i < N; i++) {
      const t = i / RATE;
      let v: number;
      if (kind === "pad") {
        const env = Math.min(1, t / 1.2) * Math.min(1, (dur - t) / 1.2);
        v = env * (Math.sin(2 * Math.PI * f * t) + 0.4 * Math.sin(2 * Math.PI * f * 2.003 * t) + 0.2 * Math.sin(2 * Math.PI * f * 3.01 * t)) * (0.85 + 0.15 * Math.sin(2 * Math.PI * 5 * t));
      } else if (kind === "drone") {
        const env = Math.min(1, t / 2) * Math.min(1, (dur - t) / 2);
        v = env * (Math.sin(2 * Math.PI * f * t) + 0.5 * Math.sin(2 * Math.PI * f * 1.5 * t));
      } else if (kind === "pluck") {
        const env = Math.min(1, t / 0.003) * Math.exp(-t * 6);
        v = env * (Math.sin(2 * Math.PI * f * t) + 0.4 * Math.sin(4 * Math.PI * f * t) * Math.exp(-t * 10));
      } else if (kind === "flute") {
        const env = Math.min(1, t / 0.08) * Math.min(1, (dur - t) / 0.2);
        const vib = 1 + 0.004 * Math.sin(2 * Math.PI * 5.5 * t) * Math.min(1, t / 0.4);
        v = env * (Math.sin(2 * Math.PI * f * vib * t) + 0.15 * Math.sin(4 * Math.PI * f * vib * t) + 0.04 * (r() * 2 - 1));
      } else {
        const env = Math.exp(-t * 18);
        v = env * (Math.sin(2 * Math.PI * f * t * (1 + 0.5 * Math.exp(-t * 30))) + 0.3 * (r() * 2 - 1) * Math.exp(-t * 40));
      }
      L[s0 + i]! += v * amp * gl;
      R[s0 + i]! += v * amp * gr;
    }
  };
  let note = 2;
  for (let bar = 0; bar < BARS; bar++) {
    const chord = CHORDS[Math.floor(bar / 2) % CHORDS.length]!;
    const t0 = bar * 4 * beat;
    const level = Math.min(1, (bar + 1) / 3) * Math.min(1, (BARS - bar) / 6);
    if (bar % 8 === 0) add(t0, 32 * beat + 1, midi(33), 0.06 * level, 0, "drone");
    if (bar % 2 === 0) for (const m of chord.slice(1)) add(t0, 8 * beat + 0.6, midi(m), 0.035 * level, (m % 3) - 1, "pad");
    for (let e = 0; e < 8; e++) {
      const m = chord[1 + ((e * 2 + Math.floor(bar / 4)) % 4)]! + 12;
      add(t0 + e * beat * 0.5, 1.2, midi(m), 0.045 * level * (e % 2 === 0 ? 1 : 0.65), -0.5, "pluck");
    }
    if (bar >= 8 && bar < 52) {
      for (const [b, f, a] of [[0, 70, 0.22], [1.5, 110, 0.1], [2, 70, 0.16], [3, 130, 0.09], [3.5, 110, 0.07]] as const) add(t0 + b * beat, 0.4, f, a * level, 0.15, "drum");
    }
    if (bar >= 16 && bar < 48) {
      for (let q = 0; q < 4; q++) {
        if (r() < 0.35) continue;
        note = clamp(note + Math.floor(r() * 5) - 2, 0, PENTA.length - 1);
        const len = r() < 0.4 ? 2 : 1;
        add(t0 + q * beat, len * beat * 0.95, midi(PENTA[note]!), 0.05 * level, 0.5, "flute");
      }
    }
  }
  // A gentle low-pass, then normalise both channels together to -3 dBFS.
  let ll = 0;
  let lr = 0;
  let peak = 0;
  for (let i = 0; i < N; i++) {
    ll += (L[i]! - ll) * 0.4;
    lr += (R[i]! - lr) * 0.4;
    L[i] = ll;
    R[i] = lr;
    peak = Math.max(peak, Math.abs(ll), Math.abs(lr));
  }
  const gain = 0.7 / peak;
  const inter = new Float32Array(N * 2);
  for (let i = 0; i < N; i++) {
    inter[i * 2] = L[i]! * gain;
    inter[i * 2 + 1] = R[i]! * gain;
  }
  return wav(inter, RATE, 2);
}

// --------------------------------------------------------------------- skin

/**
 * The paper style pack (templates/ui-kit/assets/skins/paper), copied as
 * skins/kestrel: the accent turned from the pack's one red to the courier's
 * cape red, the ground a warmer paper, and the CJK fallback pointed at the
 * game's own Japanese subset (og-taste §1).
 */
function skin(): void {
  const from = join(ENGINE, "templates", "ui-kit", "assets", "skins", "paper");
  const to = join(ASSETS, "skins", "kestrel");
  mkdirSync(to, { recursive: true });
  for (const f of readdirSync(from)) if (f !== "skin.ogt") copyFileSync(join(from, f), join(to, f));
  let text = readFileSync(join(from, "skin.ogt"), "utf8");
  const swap = (line: string, value: string) => {
    const re = new RegExp(`^(\\s*${line}\\s+)"[^"]*"`, "m");
    if (!re.test(text)) throw new Error(`skin.ogt has no ${line} line`);
    text = text.replace(re, `$1"${value}"`);
  };
  text = text.replace(/^name "paper"/m, 'name "kestrel"');
  text = text.replace(/^# The paper style pack.*$/m, "# The Kestrel Stair's skin: the engine's paper style pack (templates/ui-kit/assets/skins/paper), copied and retinted by tools/make-assets.ts.");
  swap("backdrop", "#efe6d2");
  swap("well", "#e4d9c1");
  swap("primary", "#9c2a1d");
  swap("primaryHover", "#8f2618");
  swap("primaryActive", "#7f2013");
  swap("highlight", "#86210f");
  // The pack's fgSubtle measures 3.5:1 as an item's count on a slot (og polish): a shade darker.
  swap("inkFaint", "#57524a");
  swap("fallback", "fonts/NotoSansJP-kestrel.otf");
  writeFileSync(join(to, "skin.ogt"), text);
  // The copy's licences: the pack's, its fallback named as this game's.
  let lic = readFileSync(join(from, "LICENSES.md"), "utf8");
  lic = lic.replace("# The paper style pack: licences", "# The Kestrel Stair's skin (the engine's paper style pack, retinted): licences");
  lic = lic.replace(/Characters these fonts lack[\s\S]*$/, "Characters these fonts lack (the Japanese table's) are drawn with the\nskin's `fallback` font, `fonts/NotoSansJP-kestrel.otf` (Noto Sans JP subset\nto the string tables, OFL 1.1, `assets/fonts/OFL.txt`).\n");
  writeFileSync(join(to, "LICENSES.md"), lic);
}

// --------------------------------------------------------------------- main

const tex = textures();
extraTextures();
write("models/stair.glb", level(tex));
write("nav/stair.ogt", navmesh());
for (const f of FOES) write(`ai/${f.name}.ogt`, behaviour(f.name));
write("anims/humanoid.glb", library());
write("models/courier.glb", character("courier"));
write("models/courier.glb.ogt", `type import\nversion 1\n---\n# The courier has no clips of its own: the cook retargets the clip\n# library's onto its skeleton (Mixamo names, detected), tools/rig.ts.\nanimation {\n  library "anims/humanoid.glb"\n}\n`);
write("models/imp.glb", character("imp"));
write("models/warden.glb", character("warden"));
write("rigs/warden.ogt", wardenRig());
environment();
effects();
presets();
synthSounds();
skin();
if (!process.argv.includes("--no-music")) write("music/stair.wav", music());
write("music/stair.wav.ogt", `type import\nversion 1\n---\n# "The Kestrel Stair" (tools/make-assets.ts): cooked in 4-second chunks and\n# streamed, never resident whole (og-audio §5).\nsound {\n  stream true\n  chunk 4\n}\n`);
console.log(`made The Kestrel Stair's assets under ${ASSETS}`);
