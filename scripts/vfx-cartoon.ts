// The cartoon effect pack, written from one table of recipes.
//
//   bun scripts/vfx-cartoon.ts [<out dir> [<pack>...] [--spec=<dir>]]
//
// The out dir defaults to packages/vfx-packs; `--spec=<dir>` also writes each
// pack's `vfx-pack.ts manifest` spec there.
//
// Writes `cartoon-2d/` for the cartoon-2d look.
// Each effect is one checked `file.vfx2d` Pixel VFX Forge declaration
// (`content/<name>.ts`), drawn supersampled (`supersample`)
// so its edges are smooth, its fills flat bands of two to four tones and its
// contour a 2 px dark warm brown.
//
// Positions are in half-canvas units of the shorter side,
// y down; an effect that stands on the ground has its pivot on the ground
// line, so the entity's position is where the feet are.
//
// After writing, `bun scripts/vfx-pack.ts manifest <pack> --spec
// <dir>/<pack>.json` and `preview` fill assets.json and preview/ as for every
// pack.
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type Layer = { [key: string]: Json };
type Recipe = { [key: string]: Json };

const OUT = process.argv[2] ?? resolve(import.meta.dir, "..", "packages", "vfx-packs");

// ---------- colour ramps, dark to light ----------

const RAMPS: Record<string, string[]> = {
  "fire": ["#b8321c", "#ee6a1e", "#ffb22e", "#ffe9a0"],
  "hot": ["#d2461e", "#ff8a24", "#ffd040", "#fff6c8"],
  "smoke": ["#8e7f78", "#bcaea4", "#e9e0d6"],
  "soot": ["#4e4440", "#7a6c66", "#a69890"],
  "dust": ["#a37650", "#cfa274", "#f0d6ae"],
  "water": ["#2a6cc0", "#3f9fe6", "#8fd8ff", "#eefaff"],
  "foam": ["#7cc4ee", "#c6ecff", "#ffffff"],
  "gold": ["#b8700f", "#ecaa1e", "#ffdc58", "#fff6c8"],
  "star": ["#e09a14", "#ffd23c", "#fff3a8"],
  "heal": ["#2a9046", "#56cc68", "#aef29a", "#effff0"],
  "magic": ["#5e34c0", "#9466ec", "#cfb0ff", "#f6eeff"],
  "ice": ["#3482c8", "#7cc6ee", "#cdf2ff", "#ffffff"],
  "poison": ["#4e7e1a", "#8cc236", "#d0ee76"],
  "volt": ["#2f5ed8", "#62bcff", "#dff7ff"],
  "love": ["#cc3466", "#f2688e", "#ffbcd2"],
  "dark": ["#3e2a66", "#6c4a9c", "#a68cd0"],
  "leaf": ["#4f8a26", "#86bf3a", "#c8e46a"],
  "autumn": ["#b4501c", "#e88a2a", "#f6c04a"],
  "wood": ["#6e4024", "#a4683a", "#d29a5e"],
  "rock": ["#5c5854", "#8c8680", "#bcb6ae"],
  "white": ["#d8cfc4", "#f4efe8", "#ffffff"],
  "red": ["#b42a2a", "#e65a3e", "#ff9c6a"],
  "blue": ["#2856b8", "#4a8ee8", "#9cd0ff"],
  "teal": ["#1e8a86", "#3cc0b4", "#9cf0e0"],
  "pink": ["#c23a8e", "#ee6ac0", "#ffb6e6"],
  "firefly": ["#a8b81c", "#e6f04a", "#fbffc8"],
};

// ---------- layers ----------

/** A layer: its type and fields, as the recipe writes them. */
const L = (kind: string, f: Layer = {}): Layer => ({ type: kind, ...f });

/** Python's round(): halves to even, as the first packs were written with. */
function roundHalfEven(x: number): number {
  const f = Math.floor(x);
  const d = x - f;
  return d > 0.5 ? f + 1 : d < 0.5 ? f : f % 2 === 0 ? f : f + 1;
}
/** Python's round(x, digits): an exact decimal half goes to the even digit
 * (toFixed would round it up). */
function roundTo(x: number, digits: number): number {
  const exact = x.toFixed(60);
  const cut = exact.indexOf(".") + 1 + digits;
  if (/^50*$/.test(exact.slice(cut))) {
    const kept = exact.slice(0, cut);
    return +kept[kept.length - 1] % 2 === 0 ? Number(kept) : Number(x.toFixed(digits));
  }
  return Number(x.toFixed(digits));
}

/** A recipe; `ground` is the anchor y of the ground line (half-canvas
 * units, y down), which then becomes the sprite's pivot. */
function recipe(
  name: string, w: number, h: number, frames: number, fps: number, layers: Layer[], ramps: Record<string, string[]>,
  { loop = false, seed = 1, ground, impact }: { loop?: boolean; seed?: number; ground?: number; impact?: number } = {},
): Recipe {
  const r: Recipe = { name, w, h, frames, fps, loop, seed, ramps: { ...ramps }, layers };
  if (ground !== undefined) {
    const s = Math.min(w, h) / 2;
    r.pivot = [0.5, roundTo(0.5 + (ground * s) / h, 4)];
  }
  if (impact !== undefined) r.impact = impact;
  return r;
}

/** Ramps named from RAMPS: main, alt, smoke, spark. */
const R = (k: Record<string, string>) => Object.fromEntries(Object.entries(k).map(([slot, v]) => [slot, RAMPS[v]]));

// Every effect: name, title, category, element, placement, recipe.
type Effect = [name: string, title: string, category: string, element: string, placement: string, recipe: Recipe];
const EFFECTS: Effect[] = [];

// Output pixels of margin on each side, for effects whose rings, debris or
// drift would otherwise reach the canvas edge (a straight cut in the game):
// the canvas grows and `scale` keeps the drawing's pixel size.
const PAD: Record<string, number> = {
  "hit": 16, "hit-heavy": 20, "stomp": 16, "explosion": 20, "dash": 16,
  "jump-dust": 24, "land-dust": 24, "splash": 20, "powerup": 14,
  "confetti": 12, "fireworks": 12, "checkpoint": 16, "heal": 16, "buff": 16,
  "poison": 12, "burning": 16, "frozen": 16, "shocked": 14, "teleport": 16,
  "campfire": 14, "break-crate": 24, "break-ice": 24, "splash-small": 12,
  "shield": 8, "twinkle": 8, "whirlwind": 12,
};

function padded(name: string, r: Recipe): Recipe {
  const p = PAD[name] ?? 0;
  if (!p) return r;
  const [w0, h0] = [r.w as number, r.h as number];
  r.w = w0 + 2 * p;
  r.h = h0 + 2 * p;
  r.scale = roundTo(Math.min(w0, h0) / Math.min(r.w, r.h), 6);
  const pivot = r.pivot as number[] | undefined;
  if (pivot) pivot[1] = roundTo(0.5 + ((pivot[1] - 0.5) * h0) / r.h, 4);
  return r;
}

function fx(name: string, title: string, category: string, element: string, placement: string, r: Recipe) {
  EFFECTS.push([name, title, category, element, placement, padded(name, r)]);
}

// --- hits and combat ---
fx("hit", "Pow Hit", "hit-spark", "", "Where the hit lands; a struck enemy or wall.", recipe(
  "Pow Hit", 112, 112, 10, 18, [
    L("star", { ramp: "$main", points: 8, r: 0.78, inner: 0.42, sharp: 0.6, jitter: 0.25, pop: 0.22, hollow: 0.9 }),
    L("ring", { ramp: "$alt", r0: 0.25, r1: 0.95, th: 0.12, breakup: 0.5, t0: 0.05, t1: 0.7, gain: 0.8 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 7, speed: [1.6, 2.4], drag: 2.6, life: [0.35, 0.6], shape: "star", size: 6, t0: 0.05 }),
    L("flash", { ramp: "$spark", r: 0.45, dur: 0.12 }),
  ], R({ main: "star", alt: "white", spark: "star" }), { seed: 11 }));
fx("hit-small", "Tap Hit", "hit-spark", "", "Where a light hit lands.", recipe(
  "Tap Hit", 72, 72, 8, 18, [
    L("star", { ramp: "$main", points: 5, r: 0.7, inner: 0.45, pop: 0.25, hollow: 0.9 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 5, speed: [1.4, 2.0], drag: 3, life: [0.3, 0.5], shape: "circle", size: 4 }),
  ], R({ main: "white", spark: "star" }), { seed: 12 }));
fx("hit-heavy", "Smash Hit", "hit-spark", "", "Where a heavy hit lands; pair it with og.time.hitStop on its impact.", recipe(
  "Smash Hit", 160, 160, 12, 18, [
    L("burst", { ramp: "$alt", r0: 0.12, r1: 0.5, grow: 0.25, rough: 0.35, cool: 0.55, erode: 1, erodeStart: 0.35, gain: 0.9, t1: 0.8 }),
    L("star", { ramp: "$main", points: 10, r: 0.9, inner: 0.36, sharp: 0.7, jitter: 0.3, pop: 0.2, hollow: 0.9 }),
    L("ring", { ramp: "$spark", r0: 0.3, r1: 1.0, th: 0.1, breakup: 0.6, t0: 0.08, t1: 0.75 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 10, speed: [1.8, 2.8], drag: 2.4, life: [0.4, 0.7], shape: "star", size: 7, t0: 0.05 }),
    L("flash", { ramp: "$spark", r: 0.55, dur: 0.12 }),
  ], R({ main: "star", alt: "hot", spark: "white" }), { seed: 13 }));
fx("slash", "Swoosh Slash", "slash", "", "At the swing's centre, +x forward; turn it with the sprite.", recipe(
  "Swoosh Slash", 128, 128, 12, 18, [
    L("slash", { ramp: "$main", r: 0.72, th: 0.24, a0: -150, a1: 40, sweep: 0.3, trail: 0.7, erode: 0.6 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 4, x: 0.55, y: -0.25, speed: [0.8, 1.4], drag: 3, life: [0.3, 0.5], shape: "star", size: 5, t0: 0.18 }),
  ], R({ main: "white", spark: "star" }), { seed: 14 }));
fx("stomp", "Stomp", "hit-spark", "earth", "On the ground under a stomped enemy.", recipe(
  "Stomp", 144, 96, 12, 18, [
    L("puffs", { ramp: "$smoke", count: 6, spread: 0.9, flat: 0.25, rise: 0.1, r0: 0.14, r1: 0.28, life: [0.5, 0.9], emit: 0.08, dir: 0, cone: 180, speed: 1.2, decay: 0.55, y: 0.6 }),
    L("star", { ramp: "$main", points: 6, r: 0.6, inner: 0.45, pop: 0.25, hollow: 0.9, y: 0.15 }),
    L("ring", { ramp: "$alt", r0: 0.2, r1: 1.2, th: 0.12, squash: 0.3, breakup: 0.4, y: 0.65, t1: 0.6 }),
  ], R({ main: "star", alt: "white", smoke: "dust" }), { seed: 15, ground: 0.75 }));
fx("block", "Block", "shield", "", "In front of a guarding character, +x toward the attack.", recipe(
  "Block", 128, 144, 10, 18, [
    L("bubble", { ramp: "$main", r: 0.62, th: 0.08, hex: 0, fill: 0, shimmer: 1, x: -0.15 }),
    L("star", { ramp: "$spark", points: 5, r: 0.42, inner: 0.5, pop: 0.25, hollow: 0.9, x: 0.55, t1: 0.45 }),
    L("particles", { ramp: "$spark", mode: "spray", count: 8, x: 0.45, dir: 0, spread: 110, speed: [0.9, 1.4], drag: 3.5, life: [0.3, 0.55], shape: "star", size: 7 }),
    L("flash", { ramp: "$spark", r: 0.55, x: 0.4, squash: 1.6, dur: 0.1 }),
  ], R({ main: "blue", spark: "white" }), { seed: 16 }));

// --- shooting ---
fx("muzzle", "Pop Muzzle", "gunfire", "fire", "At the barrel's mouth, +x forward.", recipe(
  "Pop Muzzle", 96, 64, 6, 20, [
    L("star", { ramp: "$main", points: 6, r: 0.62, inner: 0.42, pop: 0.3, hollow: 0.8, x: -0.2 }),
    L("burst", { ramp: "$main", r0: 0.1, r1: 0.34, grow: 0.2, rough: 0.2, cool: 0.6, x: -0.1, t1: 0.7 }),
    L("puffs", { ramp: "$smoke", count: 3, spread: 0.15, rise: 0.15, r0: 0.08, r1: 0.18, life: [0.5, 0.8], emit: 0.2, dir: 0, speed: 0.6, decay: 0.5, x: 0.1, t0: 0.3 }),
  ], R({ main: "hot", smoke: "smoke" }), { seed: 21, impact: 0.0 }));
fx("bullet", "Pellet", "projectile", "", "Loops on the moving shot; +x is where it flies.", recipe(
  "Pellet", 96, 48, 8, 16, [
    L("comet", { ramp: "$main", r: 0.32, len: 1.3, speed: 2, rough: 0.2, x: 0.6 }),
  ], R({ main: "hot" }), { loop: true, seed: 22 }));
fx("bullet-hit", "Pellet Pop", "hit-spark", "", "Where a shot ends.", recipe(
  "Pellet Pop", 80, 80, 8, 18, [
    L("ring", { ramp: "$main", r0: 0.2, r1: 0.6, th: 0.14, breakup: 0.6, t1: 0.5 }),
    L("particles", { ramp: "$main", mode: "burst", count: 6, speed: [1.4, 2.0], drag: 2.8, life: [0.3, 0.55], shape: "circle", size: 5 }),
    L("flash", { ramp: "$spark", r: 0.4, dur: 0.12 }),
  ], R({ main: "hot", spark: "white" }), { seed: 23 }));
fx("charge", "Charge Ball", "magic-impact", "lightning", "Loops at the hand or barrel while a shot charges.", recipe(
  "Charge Ball", 96, 96, 12, 12, [
    L("particles", { ramp: "$spark", mode: "implode", count: 10, implodeR: 0.95, life: [0.5, 1.0], shape: "star", size: 4, heat: [0.6, 1.05] }),
    L("orb", { ramp: "$main", r: 0.34, pulse: 0.15, pulseRate: 2, spin: 1, rim: 0.2 }),
  ], R({ main: "volt", spark: "white" }), { loop: true, seed: 24 }));
fx("laser", "Ray Beam", "beam", "lightning", "Loops from the emitter along +x; scale x for length.", recipe(
  "Ray Beam", 192, 64, 8, 12, [
    L("beam", { ramp: "$main", x: -1.45, len: 2.9, th: 0.28, cap: 0.3, wob: 0.12, speed: 2, grow: 0 }),
    L("particles", { ramp: "$spark", mode: "ambient", count: 6, area: 0.9, spanX: 0.85, spanY: 0.25, speed: [0, 0.1], wiggle: 0.04, life: [0.3, 0.6], shape: "star", size: 4, twinkle: 2 }),
  ], R({ main: "pink", spark: "white" }), { loop: true, seed: 25 }));
fx("fuse", "Fuse Sparks", "fire", "fire", "Loops at a lit fuse's tip.", recipe(
  "Fuse Sparks", 64, 64, 12, 12, [
    L("particles", { ramp: "$main", mode: "spray", count: 10, dir: -90, spread: 160, speed: [0.8, 1.6], gravity: 1.2, drag: 1.2, life: [0.3, 0.6], shape: "star", size: 4 }),
    L("orb", { ramp: "$main", r: 0.16, pulse: 0.3, pulseRate: 3, spin: 1 }),
  ], R({ main: "hot" }), { loop: true, seed: 26 }));

// --- explosions ---
fx("explosion", "Boom", "explosion", "fire", "On the ground where it goes off.", recipe(
  "Boom", 192, 192, 18, 15, [
    L("puffs", { ramp: "$smoke", count: 9, spread: 0.55, rise: 0.35, r0: 0.18, r1: 0.36, life: [0.55, 0.95], emit: 0.12, gain: 0.8, decay: 0.5, flat: 0.6, t0: 0.18, y: 0.1 }),
    L("burst", { ramp: "$main", r0: 0.12, r1: 0.66, grow: 0.35, rough: 0.28, cool: 0.5, erode: 0.85, erodeStart: 0.45, t1: 0.8, y: 0.15 }),
    L("ring", { ramp: "$alt", r0: 0.2, r1: 0.95, th: 0.08, squash: 0.3, breakup: 0.5, y: 0.75, t1: 0.55 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 12, speed: [1.6, 2.6], drag: 2, gravity: 1.2, life: [0.4, 0.7], shape: "star", size: 7, t0: 0.05, y: 0.15 }),
    L("flash", { ramp: "$spark", r: 0.6, dur: 0.1, y: 0.15 }),
  ], R({ main: "fire", alt: "white", smoke: "smoke", spark: "star" }), { seed: 31, ground: 0.8 }));
fx("explosion-small", "Pop Boom", "explosion", "fire", "On the ground or a body where a small charge goes off.", recipe(
  "Pop Boom", 120, 120, 14, 16, [
    L("puffs", { ramp: "$smoke", count: 6, spread: 0.4, rise: 0.3, r0: 0.16, r1: 0.3, life: [0.5, 0.9], emit: 0.1, decay: 0.5, flat: 0.7, t0: 0.2 }),
    L("burst", { ramp: "$main", r0: 0.12, r1: 0.6, grow: 0.3, rough: 0.25, cool: 0.5, erode: 0.9, erodeStart: 0.4, t1: 0.8 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 7, speed: [1.4, 2.2], drag: 2.4, life: [0.35, 0.6], shape: "star", size: 6, t0: 0.05 }),
    L("flash", { ramp: "$spark", r: 0.5, dur: 0.1 }),
  ], R({ main: "fire", smoke: "smoke", spark: "star" }), { seed: 32 }));
fx("poof", "Poof", "smoke", "", "Where something appears or vanishes.", recipe(
  "Poof", 112, 112, 14, 16, [
    L("puffs", { ramp: "$main", count: 8, spread: 0.35, rise: 0.15, r0: 0.2, r1: 0.4, life: [0.6, 1.0], emit: 0.08, decay: 0.45, flat: 0.85 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 5, speed: [1.4, 2.0], drag: 2.6, life: [0.35, 0.6], shape: "star", size: 5, t0: 0.1 }),
  ], R({ main: "white", spark: "star" }), { seed: 33 }));

// --- movement ---
fx("jump-dust", "Jump Dust", "movement", "earth", "At the feet as a jump leaves the ground.", recipe(
  "Jump Dust", 112, 64, 10, 18, [
    L("puffs", { ramp: "$main", count: 6, spread: 0.5, flat: 0.2, rise: 0.12, r0: 0.12, r1: 0.26, life: [0.5, 0.9], emit: 0.06, dir: 180, cone: 200, speed: 0.9, decay: 0.55, y: 0.55 }),
  ], R({ main: "dust" }), { seed: 41, ground: 0.75 }));
fx("land-dust", "Land Dust", "movement", "earth", "At the feet on landing.", recipe(
  "Land Dust", 144, 64, 12, 18, [
    L("puffs", { ramp: "$main", count: 8, spread: 0.9, flat: 0.2, rise: 0.12, r0: 0.12, r1: 0.28, life: [0.5, 0.9], emit: 0.05, dir: -90, cone: 180, speed: 1.4, decay: 0.5, y: 0.55 }),
    L("particles", { ramp: "$main", mode: "burst", count: 6, y: 0.55, dir: -90, spread: 140, speed: [1.0, 1.6], gravity: 2.4, drag: 1.2, ground: 0.7, life: [0.35, 0.6], shape: "circle", size: 4 }),
  ], R({ main: "dust" }), { seed: 42, ground: 0.75 }));
fx("run-dust", "Run Dust", "movement", "earth", "Loops behind running feet; +x is the way the runner faces.", recipe(
  "Run Dust", 96, 64, 12, 12, [
    L("puffs", { ramp: "$main", count: 4, spread: 0.2, flat: 0.4, rise: 0.25, r0: 0.1, r1: 0.2, life: [0.5, 0.9], dir: 180, cone: 40, speed: 0.8, decay: 0.6, x: 0.3, y: 0.55 }),
  ], R({ main: "dust" }), { loop: true, seed: 43, ground: 0.75 }));
fx("dash", "Dash Lines", "movement", "wind", "At the dashing body; +x is the way it dashes.", recipe(
  "Dash Lines", 160, 96, 10, 18, [
    L("particles", { ramp: "$main", mode: "spray", count: 12, x: 0.35, area: 0.4, areaX: 0.3, dir: 180, spread: 12, speed: [1.6, 2.4], drag: 3, life: [0.35, 0.6], emit: 0.35, shape: "streak", size: 11 }),
    L("puffs", { ramp: "$smoke", count: 4, spread: 0.25, rise: 0.05, r0: 0.12, r1: 0.22, life: [0.4, 0.8], dir: 180, speed: 0.6, decay: 0.6, x: -0.35, t0: 0.1 }),
  ], R({ main: "white", smoke: "white" }), { seed: 44 }));
fx("double-jump", "Air Jump", "movement", "wind", "At the feet of a second jump in the air.", recipe(
  "Air Jump", 112, 80, 10, 18, [
    L("ring", { ramp: "$main", r0: 0.2, r1: 0.95, th: 0.16, squash: 0.35, breakup: 0.3, y: 0.4 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, y: 0.4, squash: 0.4, speed: [1.2, 1.8], drag: 3, life: [0.3, 0.5], shape: "star", size: 4 }),
  ], R({ main: "white", spark: "star" }), { seed: 45 }));
fx("skid", "Skid Dust", "movement", "earth", "Loops at the feet while skidding; +x is the way it slides.", recipe(
  "Skid Dust", 112, 64, 12, 12, [
    L("puffs", { ramp: "$main", count: 5, spread: 0.15, flat: 0.3, rise: 0.3, r0: 0.1, r1: 0.24, life: [0.5, 0.9], dir: -150, cone: 50, speed: 1.0, decay: 0.55, x: 0.4, y: 0.55 }),
    L("particles", { ramp: "$main", mode: "spray", count: 6, x: 0.45, y: 0.55, dir: -135, spread: 50, speed: [0.8, 1.4], gravity: 2, life: [0.3, 0.5], shape: "circle", size: 3 }),
  ], R({ main: "dust" }), { loop: true, seed: 46, ground: 0.75 }));

// --- water ---
fx("splash", "Splash", "splash", "water", "On the water line where something falls in.", recipe(
  "Splash", 144, 144, 14, 16, [
    L("pillar", { ramp: "$main", w: 0.32, hgt: 0.9, speed: 1, rough: 0.35, grow: 0.3, y: 0.7, t1: 0.6 }),
    L("particles", { ramp: "$main", mode: "spray", count: 14, y: 0.6, dir: -90, spread: 110, speed: [1.4, 2.4], gravity: 3.2, drag: 0.6, life: [0.5, 0.9], shape: "drop", size: 6 }),
    L("ring", { ramp: "$alt", r0: 0.2, r1: 1.08, th: 0.1, squash: 0.25, breakup: 0.3, y: 0.75 }),
  ], R({ main: "water", alt: "foam" }), { seed: 51, ground: 0.8 }));
fx("splash-small", "Plop", "splash", "water", "On the water line where something small drops in.", recipe(
  "Plop", 96, 96, 12, 16, [
    L("particles", { ramp: "$main", mode: "spray", count: 8, y: 0.55, dir: -90, spread: 90, speed: [1.0, 1.8], gravity: 3.2, drag: 0.6, life: [0.45, 0.8], shape: "drop", size: 5 }),
    L("ring", { ramp: "$alt", r0: 0.15, r1: 0.9, th: 0.12, squash: 0.28, breakup: 0.2, y: 0.65 }),
  ], R({ main: "water", alt: "foam" }), { seed: 52, ground: 0.7 }));
fx("ripple", "Ripples", "splash", "water", "Loops on the water line round a wading body or a float.", recipe(
  "Ripples", 128, 48, 16, 12, [
    L("ring", { ramp: "$main", r0: 0.2, r1: 1.6, th: 0.12, squash: 0.22, breakup: 0.15, fade: 1 }),
    L("ring", { ramp: "$main", r0: 0.2, r1: 1.6, th: 0.12, squash: 0.22, breakup: 0.15, fade: 1, t0: 0.5, t1: 1.5 }),
  ], R({ main: "foam" }), { loop: true, seed: 53 }));
fx("bubbles", "Bubbles", "ambient", "water", "Loops under water at a diver or a vent.", recipe(
  "Bubbles", 64, 128, 16, 12, [
    L("particles", { ramp: "$main", mode: "rise", count: 8, y: 0.8, area: 0.3, speed: [0.8, 1.4], life: [0.6, 1.0], wiggle: 0.3, shape: "bubble", size: [3, 6] }),
  ], R({ main: "foam" }), { loop: true, seed: 54 }));

// --- pickups and rewards ---
fx("coin", "Coin Grab", "pickup", "", "Where a coin is collected.", recipe(
  "Coin Grab", 96, 112, 12, 18, [
    L("rays", { ramp: "$smoke", count: 8, r: 0.8, width: 0.16, gain: 0.55, t1: 0.6 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, speed: [1.2, 1.8], drag: 3, life: [0.35, 0.6], shape: "star", size: 5, t0: 0.1 }),
    L("particles", { ramp: "$main", mode: "burst", count: 1, speed: [0, 0], life: [0.55, 0.6], shape: "coin", size: 11, gravity: -2.5 }),
  ], R({ main: "gold", smoke: "white", spark: "star" }), { seed: 61 }));
fx("gem", "Gem Grab", "pickup", "", "Where a gem is collected.", recipe(
  "Gem Grab", 96, 112, 12, 18, [
    L("ring", { ramp: "$spark", r0: 0.2, r1: 0.8, th: 0.1, breakup: 0.4, t1: 0.6 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, speed: [1.2, 1.8], drag: 3, life: [0.35, 0.6], shape: "star", size: 5, t0: 0.1 }),
    L("particles", { ramp: "$main", mode: "burst", count: 1, speed: [0, 0], life: [0.55, 0.6], shape: "diamond", size: 10, gravity: -2.5 }),
  ], R({ main: "teal", spark: "white" }), { seed: 62 }));
fx("heart", "Heart Grab", "pickup", "", "Where a life or health pickup is collected.", recipe(
  "Heart Grab", 96, 112, 12, 18, [
    L("ring", { ramp: "$main", r0: 0.2, r1: 0.8, th: 0.1, breakup: 0.3, t1: 0.6 }),
    L("particles", { ramp: "$main", mode: "rise", count: 4, area: 0.5, speed: [0.8, 1.4], life: [0.45, 0.8], shape: "heart", size: 5, t0: 0.1 }),
    L("particles", { ramp: "$main", mode: "burst", count: 1, speed: [0, 0], life: [0.55, 0.6], shape: "heart", size: 11, gravity: -2.5 }),
  ], R({ main: "love", spark: "white" }), { seed: 63 }));
fx("powerup", "Power Up", "buff", "", "Round a character taking a power-up.", recipe(
  "Power Up", 128, 160, 16, 15, [
    L("rays", { ramp: "$main", count: 12, r: 1.0, width: 0.18, gain: 0.7, spin: 0.2, t1: 0.8 }),
    L("ring", { ramp: "$main", r0: 0.3, r1: 1.0, th: 0.14, breakup: 0.3, t1: 0.7 }),
    L("particles", { ramp: "$spark", mode: "rise", count: 10, area: 0.7, speed: [0.8, 1.5], life: [0.45, 0.8], emit: 0.5, shape: "star", size: 5, twinkle: 2 }),
  ], R({ main: "star", spark: "white" }), { seed: 64 }));
fx("star-collect", "Star Grab", "pickup", "", "Where a big collectible is taken.", recipe(
  "Star Grab", 144, 144, 16, 15, [
    L("star", { ramp: "$main", points: 5, r: 0.8, inner: 0.48, sharp: 0.2, pop: 0.3, hollow: 0.6 }),
    L("rays", { ramp: "$spark", count: 10, r: 1.0, width: 0.14, gain: 0.7, t1: 0.7 }),
    L("particles", { ramp: "$main", mode: "burst", count: 10, speed: [1.4, 2.2], drag: 2.2, life: [0.4, 0.7], shape: "star", size: 6, t0: 0.1 }),
  ], R({ main: "star", spark: "white" }), { seed: 65 }));
fx("chest-open", "Treasure", "pickup", "", "At an opened chest's lid line.", recipe(
  "Treasure", 128, 160, 18, 15, [
    L("rays", { ramp: "$smoke", count: 9, r: 1.1, width: 0.2, gain: 0.55, y: 0.2, jitter: 0.2, t1: 0.85 }),
    L("particles", { ramp: "$spark", mode: "rise", count: 8, y: 0.3, area: 0.5, speed: [0.6, 1.2], life: [0.4, 0.8], emit: 0.6, shape: "star", size: 5, twinkle: 2 }),
    L("particles", { ramp: "$main", mode: "burst", count: 8, y: 0.4, dir: -90, spread: 70, speed: [2.0, 2.8], gravity: 4.5, drag: 0.3, life: [0.6, 0.95], emit: 0.2, shape: "coin", size: 8, ground: 0.55 }),
  ], R({ main: "gold", smoke: "white", spark: "star" }), { seed: 66, ground: 0.6 }));
fx("confetti", "Confetti Pop", "ui", "", "Where a win is celebrated; project a UI point into the scene.", recipe(
  "Confetti Pop", 160, 160, 20, 15, [
    L("particles", { ramp: "$main", mode: "spray", count: 10, dir: -90, spread: 120, speed: [1.8, 2.8], gravity: 1.6, drag: 1.8, life: [0.6, 1.0], shape: "leaf", size: 9 }),
    L("particles", { ramp: "$alt", mode: "spray", count: 10, dir: -90, spread: 120, speed: [1.8, 2.8], gravity: 1.6, drag: 1.8, life: [0.6, 1.0], shape: "leaf", size: 9, seed: 3 }),
    L("particles", { ramp: "$smoke", mode: "spray", count: 10, dir: -90, spread: 120, speed: [1.8, 2.8], gravity: 1.6, drag: 1.8, life: [0.6, 1.0], shape: "leaf", size: 9, seed: 5 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, speed: [1.4, 2.0], drag: 2.6, life: [0.3, 0.5], shape: "star", size: 5 }),
  ], R({ main: "red", alt: "blue", smoke: "star", spark: "white" }), { seed: 67 }));
fx("confetti-rain", "Confetti Rain", "ui", "", "Loops over a win screen's play area.", recipe(
  "Confetti Rain", 192, 192, 24, 12, [
    L("particles", { ramp: "$main", mode: "fall", y: 0.3, fadeIn: 0.15, fallScale: 0.62, driftX: 0.06, count: 8, spanX: 0.68, speed: [0.6, 1.0], life: [0.8, 0.95], wiggle: 0.2, shape: "leaf", size: 9 }),
    L("particles", { ramp: "$alt", mode: "fall", y: 0.3, fadeIn: 0.15, fallScale: 0.62, driftX: 0.06, count: 8, spanX: 0.68, speed: [0.6, 1.0], life: [0.8, 0.95], wiggle: 0.2, shape: "leaf", size: 9, seed: 3 }),
    L("particles", { ramp: "$smoke", mode: "fall", y: 0.3, fadeIn: 0.15, fallScale: 0.62, driftX: 0.06, count: 8, spanX: 0.68, speed: [0.6, 1.0], life: [0.8, 0.95], wiggle: 0.2, shape: "leaf", size: 9, seed: 5 }),
  ], R({ main: "red", alt: "blue", smoke: "star" }), { loop: true, seed: 68 }));
fx("fireworks", "Firework", "ui", "fire", "In the sky over a win.", recipe(
  "Firework", 192, 192, 22, 15, [
    L("particles", { ramp: "$spark", mode: "burst", count: 1, y: 1.0, dir: -90, spread: 0, speed: [2.2, 2.2], life: [0.25, 0.25], shape: "streak", size: 4 }),
    L("particles", { ramp: "$main", mode: "burst", count: 18, speed: [1.6, 2.0], drag: 1.6, gravity: 0.8, life: [0.5, 0.75], shape: "star", size: 7, t0: 0.25, y: -0.1 }),
    L("ring", { ramp: "$main", r0: 0.1, r1: 0.6, th: 0.06, breakup: 0.8, t0: 0.25, t1: 0.5, y: -0.1 }),
    L("flash", { ramp: "$spark", r: 0.35, dur: 0.08, t0: 0.25, y: -0.1 }),
  ], R({ main: "pink", spark: "star" }), { seed: 69 }));
fx("checkpoint", "Checkpoint", "marker", "", "On the ground at a reached checkpoint.", recipe(
  "Checkpoint", 112, 160, 16, 15, [
    L("pillar", { ramp: "$main", w: 0.45, hgt: 2.2, speed: 2, rough: 0.12, grow: 0.15, gain: 0.7, y: 1.1 }),
    L("ring", { ramp: "$main", r0: 0.25, r1: 0.9, th: 0.14, squash: 0.32, breakup: 0.3, y: 1.1, t1: 0.6 }),
    L("particles", { ramp: "$spark", mode: "rise", count: 8, y: 1.0, area: 0.6, speed: [1.0, 1.6], life: [0.45, 0.8], emit: 0.5, shape: "star", size: 4 }),
  ], R({ main: "teal", spark: "white" }), { seed: 70, ground: 1.15 }));

// --- status ---
fx("heal", "Heal", "heal", "nature", "On the ground under the healed character.", recipe(
  "Heal", 112, 160, 18, 15, [
    L("ring", { ramp: "$main", r0: 0.3, r1: 0.95, th: 0.14, squash: 0.32, breakup: 0.25, y: 1.1 }),
    L("particles", { ramp: "$main", mode: "rise", count: 8, y: 1.0, area: 0.65, areaY: 0.25, speed: [0.8, 1.5], life: [0.5, 0.85], emit: 0.5, shape: "plus", size: 5, heat: [1.05, 0.7] }),
    L("particles", { ramp: "$spark", mode: "rise", count: 6, y: 1.0, area: 0.7, speed: [0.5, 1.0], life: [0.4, 0.8], emit: 0.6, shape: "star", size: 3, twinkle: 2 }),
  ], R({ main: "heal", spark: "white" }), { seed: 71, ground: 1.15 }));
fx("buff", "Buff", "buff", "", "On the ground under a strengthened character.", recipe(
  "Buff", 112, 160, 16, 15, [
    L("ring", { ramp: "$main", r0: 0.3, r1: 0.9, th: 0.12, squash: 0.3, breakup: 0.3, y: 1.1, t1: 0.6 }),
    L("particles", { ramp: "$spark", mode: "rise", count: 9, y: 1.0, area: 0.6, areaY: 0.2, speed: [1.2, 1.9], life: [0.45, 0.8], emit: 0.5, shape: "chevron", size: 10 }),
    L("particles", { ramp: "$main", mode: "rise", count: 6, y: 1.0, area: 0.7, speed: [0.7, 1.2], life: [0.4, 0.8], emit: 0.6, shape: "star", size: 5, twinkle: 2 }),
  ], R({ main: "star", spark: "red" }), { seed: 72, ground: 1.15 }));
fx("debuff", "Debuff", "status", "dark", "Over a weakened character's head.", recipe(
  "Debuff", 112, 144, 16, 15, [
    L("puffs", { ramp: "$main", count: 5, spread: 0.4, rise: -0.2, r0: 0.14, r1: 0.26, life: [0.5, 0.9], emit: 0.4, decay: 0.5, flat: 0.8, y: -0.4 }),
    L("particles", { ramp: "$main", mode: "burst", count: 6, y: -0.4, dir: 90, spread: 50, speed: [0.4, 0.8], gravity: 2.5, life: [0.5, 0.8], emit: 0.5, shape: "drop", size: 6 }),
  ], R({ main: "dark" }), { seed: 73 }));
fx("stun", "Dizzy Stars", "status", "", "Loops over a stunned character's head.", recipe(
  "Dizzy Stars", 112, 64, 16, 12, [
    L("particles", { ramp: "$main", mode: "orbit", count: 4, orbitR: 0.75, turns: 1, squash: 0.35, life: [1, 1], shape: "star", size: 7 }),
  ], R({ main: "star" }), { loop: true, seed: 74 }));
fx("poison", "Poisoned", "status", "poison", "Loops round a poisoned character.", recipe(
  "Poisoned", 96, 144, 16, 12, [
    L("particles", { ramp: "$main", mode: "rise", count: 8, y: 0.6, area: 0.6, speed: [0.6, 1.2], life: [0.6, 0.95], wiggle: 0.3, shape: "bubble", size: [3, 6] }),
    L("puffs", { ramp: "$main", count: 3, spread: 0.4, rise: 0.4, r0: 0.1, r1: 0.2, life: [0.6, 0.95], decay: 0.6, gain: 0.6, y: 0.4 }),
  ], R({ main: "poison" }), { loop: true, seed: 75 }));
fx("burning", "On Fire", "status", "fire", "Loops on a burning character; it stands on the ground.", recipe(
  "On Fire", 96, 160, 16, 12, [
    L("flame", { ramp: "$main", w: 0.75, hgt: 2.0, taper: 0.4, freq: 2, rough: 0.35, speed: 1, flick: 0.3, y: 1.1 }),
    L("particles", { ramp: "$main", mode: "rise", count: 5, y: 0.4, area: 0.5, speed: [0.8, 1.4], life: [0.4, 0.8], shape: "circle", size: 3 }),
  ], R({ main: "fire" }), { loop: true, seed: 76, ground: 1.15 }));
fx("frozen", "Frozen", "status", "ice", "On the ground under a frozen character.", recipe(
  "Frozen", 128, 144, 14, 15, [
    L("spikes", { ramp: "$main", count: 6, spread: 0.8, hgt: 1.2, width: 0.22, lean: 0.35, fan: 0.6, grow: 0.2, hold: 0.6, y: 0.95 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, y: 0.4, speed: [0.8, 1.4], drag: 3, life: [0.3, 0.6], shape: "diamond", size: 3, t0: 0.1 }),
  ], R({ main: "ice", spark: "white" }), { seed: 77, ground: 1.0 }));
fx("shocked", "Zapped", "status", "lightning", "Round a shocked character.", recipe(
  "Zapped", 112, 144, 12, 18, [
    L("bolt", { ramp: "$main", mode: "radial", count: 5, len: 0.95, jag: 0.4, depth: 4, branch: 0.25, thick: 4, flicker: 0.12, hold: 2, fadeStart: 0.7 }),
    L("flash", { ramp: "$spark", r: 0.45, dur: 0.1 }),
  ], R({ main: "volt", spark: "white" }), { seed: 78 }));

// --- magic ---
fx("magic", "Sparkle Cast", "magic-impact", "arcane", "At the hand or wand casting.", recipe(
  "Sparkle Cast", 112, 112, 14, 16, [
    L("ring", { ramp: "$main", r0: 0.15, r1: 0.85, th: 0.14, breakup: 0.4, t1: 0.7 }),
    L("particles", { ramp: "$main", mode: "burst", count: 9, speed: [1.2, 2.0], drag: 2.4, life: [0.4, 0.7], shape: "star", size: 6 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 6, speed: [0.6, 1.2], drag: 2.4, life: [0.3, 0.6], shape: "circle", size: 3, t0: 0.1 }),
  ], R({ main: "magic", spark: "white" }), { seed: 81 }));
fx("magic-bolt", "Magic Bolt", "magic-projectile", "arcane", "Loops on the moving spell; +x is where it flies.", recipe(
  "Magic Bolt", 128, 64, 10, 15, [
    L("comet", { ramp: "$main", r: 0.36, len: 1.4, speed: 2, rough: 0.3, x: 0.7 }),
    L("particles", { ramp: "$spark", mode: "ambient", count: 5, x: -0.2, area: 0.4, spanX: 1.2, spanY: 0.3, life: [0.3, 0.6], shape: "star", size: 4, twinkle: 2 }),
  ], R({ main: "magic", spark: "white" }), { loop: true, seed: 82 }));
fx("magic-circle", "Spell Circle", "magic-impact", "arcane", "Loops on the ground under a caster.", recipe(
  "Spell Circle", 160, 80, 16, 12, [
    L("glyph", { ramp: "$main", r: 0.95, squash: 0.35, rings: [1, 0.82, 0.55], star: 6, step: 2, ticks: 0, runes: 0, spin: 1 }),
  ], R({ main: "magic" }), { loop: true, seed: 83 }));
fx("portal", "Portal", "magic-impact", "arcane", "Loops where a portal stands; its centre at the anchor.", recipe(
  "Portal", 128, 160, 16, 12, [
    L("vortex", { ramp: "$main", r: 0.85, rIn: 0.25, arms: 3, twist: 1.2, spin: 1, dir: 1, squash: 1.3 }),
    L("particles", { ramp: "$spark", mode: "implode", count: 8, implodeR: 1.0, life: [0.5, 0.9], shape: "star", size: 4 }),
  ], R({ main: "magic", spark: "white" }), { loop: true, seed: 84 }));
fx("teleport", "Teleport", "movement", "arcane", "On the ground where a character vanishes or arrives.", recipe(
  "Teleport", 112, 176, 18, 15, [
    L("ring", { ramp: "$main", r0: 0.3, r1: 0.95, th: 0.14, squash: 0.32, breakup: 0.2, y: 1.3, t1: 0.7 }),
    L("pillar", { ramp: "$main", w: 0.4, hgt: 2.8, speed: 3, rough: 0.12, grow: 0.1, y: 1.3, t0: 0.25, t1: 0.8 }),
    L("particles", { ramp: "$spark", mode: "burst", count: 8, y: 0.3, t0: 0.35, speed: [1.0, 1.8], drag: 2.5, life: [0.3, 0.6], shape: "star", size: 5 }),
    L("flash", { ramp: "$spark", y: 0.3, r: 0.6, squash: 1.6, t0: 0.3, dur: 0.12 }),
  ], R({ main: "blue", spark: "white" }), { seed: 85, ground: 1.35 }));
fx("shield", "Shield Bubble", "shield", "arcane", "Loops round a shielded character's middle.", recipe(
  "Shield Bubble", 128, 128, 16, 12, [
    L("bubble", { ramp: "$main", r: 0.85, th: 0.09, hex: 0, fill: 0, shimmer: 1 }),
  ], R({ main: "blue" }), { loop: true, seed: 86 }));
fx("whirlwind", "Whirlwind", "magic-impact", "wind", "Loops on the ground under a whirlwind.", recipe(
  "Whirlwind", 112, 176, 16, 12, [
    L("tornado", { ramp: "$main", w0: 0.25, w1: 0.8, hgt: 2.6, twist: 2, spin: 2, sway: 0.15, rough: 0.3, y: 1.3 }),
    L("puffs", { ramp: "$smoke", count: 9, spread: 0.95, flat: 0.35, rise: 0.04, r0: 0.24, r1: 0.32, life: [0.9, 1.0], dir: 0, cone: 360, speed: 0.1, decay: 0.15, y: -1.2 }),
    L("particles", { ramp: "$alt", mode: "orbit", count: 4, y: 0.6, orbitR: 0.6, turns: 2, squash: 0.3, life: [1, 1], shape: "leaf", size: 4 }),
  ], R({ main: "ice", alt: "leaf", smoke: "white" }), { loop: true, seed: 87, ground: 1.35 }));

// --- environment ---
fx("campfire", "Campfire", "fire", "fire", "Loops on the ground at the logs.", recipe(
  "Campfire", 96, 144, 16, 12, [
    L("flame", { ramp: "$main", w: 0.8, hgt: 1.6, taper: 0.5, freq: 2, rough: 0.35, speed: 1, flick: 0.3, y: 0.9 }),
    L("particles", { ramp: "$alt", mode: "rise", count: 5, y: 0.2, area: 0.3, speed: [0.8, 1.4], life: [0.4, 0.8], wiggle: 0.3, shape: "circle", size: 3 }),
  ], R({ main: "fire", alt: "hot" }), { loop: true, seed: 91, ground: 0.95 }));
fx("torch", "Torch Flame", "fire", "fire", "Loops at a torch's head.", recipe(
  "Torch Flame", 64, 112, 16, 12, [
    L("flame", { ramp: "$main", w: 0.6, hgt: 1.5, taper: 0.5, freq: 2, rough: 0.3, speed: 1, flick: 0.3, y: 0.75 }),
  ], R({ main: "fire" }), { loop: true, seed: 92, ground: 0.8 }));
fx("chimney-smoke", "Chimney Smoke", "smoke", "", "Loops at a chimney's top.", recipe(
  "Chimney Smoke", 96, 176, 24, 12, [
    L("puffs", { ramp: "$main", count: 6, spread: 0.15, rise: 1.8, r0: 0.18, r1: 0.42, life: [0.8, 1.0], dir: -90, cone: 20, speed: 0.5, drift: 0.3, decay: 0.4, flat: 1, y: 1.1 }),
  ], R({ main: "smoke" }), { loop: true, seed: 93, ground: 1.2 }));
fx("leaves", "Falling Leaves", "ambient", "nature", "Loops over a play area under trees.", recipe(
  "Falling Leaves", 192, 192, 24, 12, [
    L("particles", { ramp: "$main", mode: "fall", y: 0.3, fadeIn: 0.15, fallScale: 0.62, driftX: 0.06, count: 6, spanX: 0.68, speed: [0.4, 0.7], life: [0.8, 0.95], wiggle: 0.2, shape: "leaf", size: 14 }),
    L("particles", { ramp: "$alt", mode: "fall", y: 0.3, fadeIn: 0.15, fallScale: 0.62, driftX: 0.06, count: 5, spanX: 0.68, speed: [0.4, 0.7], life: [0.8, 0.95], wiggle: 0.2, shape: "leaf", size: 14, seed: 4 }),
  ], R({ main: "leaf", alt: "autumn" }), { loop: true, seed: 94 }));
fx("fireflies", "Fireflies", "ambient", "nature", "Loops in a dark play area.", recipe(
  "Fireflies", 192, 128, 24, 12, [
    L("particles", { ramp: "$main", mode: "ambient", count: 10, spanX: 0.85, spanY: 0.6, speed: [0, 0.15], life: [0.8, 1.0], wiggle: 0.2, twinkle: 1, shape: "circle", size: 9, glow: 1 }),
  ], R({ main: "firefly" }), { loop: true, seed: 95 }));
fx("twinkle", "Twinkle", "ambient", "", "Loops on something shiny: a gem, a star, a secret.", recipe(
  "Twinkle", 112, 112, 16, 12, [
    L("particles", { ramp: "$main", mode: "ambient", count: 4, area: 0.6, spanX: 0.62, spanY: 0.62, speed: [0, 0.1], wiggle: 0.04, life: [0.5, 0.8], twinkle: 2, shape: "star", size: 16 }),
  ], R({ main: "star" }), { loop: true, seed: 96 }));

// --- destruction ---
fx("break-crate", "Crate Break", "destruction", "nature", "At the centre of a broken crate.", recipe(
  "Crate Break", 160, 144, 16, 16, [
    L("puffs", { ramp: "$smoke", count: 5, spread: 0.4, rise: 0.2, r0: 0.14, r1: 0.28, life: [0.5, 0.9], decay: 0.55, flat: 0.8, t0: 0.05, t1: 0.85 }),
    L("star", { ramp: "$spark", points: 6, r: 0.55, inner: 0.45, pop: 0.25, hollow: 0.9, t1: 0.35 }),
    L("particles", { ramp: "$main", mode: "burst", count: 9, dir: -90, spread: 200, speed: [1.3, 2.1], gravity: 3, drag: 1.8, life: [0.6, 1.0], shape: "chunk", size: [8, 12] }),
  ], R({ main: "wood", smoke: "smoke", spark: "white" }), { seed: 101 }));
fx("break-rock", "Rock Break", "destruction", "earth", "At the centre of a broken rock.", recipe(
  "Rock Break", 160, 144, 16, 16, [
    L("puffs", { ramp: "$smoke", count: 6, spread: 0.45, rise: 0.25, r0: 0.16, r1: 0.3, life: [0.5, 0.9], decay: 0.5, flat: 0.8, t0: 0.05, t1: 0.85 }),
    L("star", { ramp: "$spark", points: 6, r: 0.55, inner: 0.45, pop: 0.25, hollow: 0.9, t1: 0.35 }),
    L("particles", { ramp: "$main", mode: "burst", count: 9, dir: -90, spread: 200, speed: [1.3, 2.0], gravity: 3.2, drag: 1.8, life: [0.6, 1.0], shape: "chunk", size: [7, 12] }),
  ], R({ main: "rock", smoke: "smoke", spark: "white" }), { seed: 102 }));
fx("break-ice", "Ice Shatter", "destruction", "ice", "At the centre of shattered ice or glass.", recipe(
  "Ice Shatter", 160, 144, 14, 16, [
    L("flash", { ramp: "$spark", r: 0.25, dur: 0.08 }),
    L("ring", { ramp: "$spark", r0: 0.12, r1: 0.55, th: 0.07, breakup: 0.7, t1: 0.4 }),
    L("particles", { ramp: "$main", mode: "burst", count: 14, dir: -90, spread: 220, speed: [1.3, 2.1], gravity: 3, drag: 1.8, life: [0.5, 0.9], shape: "shard", size: [7, 11] }),
  ], R({ main: "ice", spark: "white" }), { seed: 103 }));


interface Pack {
  description: string;
  looks: string[];
  style: Record<string, Json>;
  title: string;
}

const PACKS: Record<string, Pack> = {
  "cartoon-2d": {
    description: "Flat cartoon effects for cartoon-2d: clean shapes in two to four flat tones with a 2 px dark warm contour, drawn smooth.",
    looks: ["cartoon-2d"],
    style: { outline: "outside", outlineColor: "#3a2416", line: 2, innerLine: 1 },
    title: "Cartoon",
  },
};

const keywords = (spec: Pack) => [...new Set(["2d", "cartoon", "flat", ...spec.looks])].sort();

function write(pack: string, spec: Pack): number {
  const d = join(OUT, pack);
  mkdirSync(join(d, "content"), { recursive: true });
  const cats = new Set<string>();
  const elems = new Set<string>();
  const rows: string[] = [];
  for (const [name, title, category, element, placement, r0] of EFFECTS) {
    const r: Recipe = structuredClone(r0);
    Object.assign(r, { supersample: 4, cleanup: false, ...spec.style });
    writeFileSync(join(d, "content", `${name}.ts`), `import { file } from '@origingame/engine/author';\n// ${title}: ${placement}\nexport default file.vfx2d(${JSON.stringify({ recipe: r }, null, 1)});\n`);
    cats.add(category);
    if (element) elems.add(element);
    rows.push(`| ${name} | ${r.loop ? "loops" : "once"} | ${placement} |`);
  }
  const ext = {
    format: 1,
    kind: "vfx",
    name: pack,
    version: "1.0.0",
    description: spec.description,
    minEngine: "0.9.0",
    tags: { style: "stylised", dim: "2d", category: [...cats].sort(), element: [...elems].sort() },
    keywords: keywords(spec),
  };
  writeFileSync(join(d, "extension.json"), JSON.stringify(ext, null, 2) + "\n");
  writeFileSync(join(d, "README.md"), `# ${spec.title} 2D effects

Install with \`og vfx add ${pack}\`; spawn \`base/vfx/${pack}/<name>\` as an
og.Sprite flipbook (\`og api show anim\`). ${spec.description}

Every effect is a \`file.vfx2d\` recipe drawn supersampled (\`supersample: 4\`):
the cook renders it at four canvas pixels an output pixel, outlines it and
shrinks it, so its edges are smooth and its sheet is filtered linearly.
An effect's pixels are the character's (96 to 160 px): draw the sprite at
1 world unit a pixel of the game's own scale, never resampled.

Each one-shot effect sends an og.FlipbookEvent \`impact\`.
No gameplay, scoring, damage or UI logic; text is left to the
game's UI.

| Effect | Plays | Placement |
|---|---|---|
${rows.join("\n")}

All recipes are original MIT work, Origin Engine, written by
\`scripts/vfx-cartoon.ts\` for the Pixel VFX Forge renderer the cook runs
(\`third_party/scenario-pixel-vfx\`). The outline is an exact Euclidean
distance transform (Felzenszwalb and Huttenlocher, "Distance Transforms of
Sampled Functions", Theory of Computing 8, 2012; method only).
`);
  return EFFECTS.length;
}

/** The `vfx-pack.ts manifest --spec` input for a pack. */
function manifestSpec(spec: Pack) {
  return {
    minEngine: "0.9.0",
    style: "stylised",
    dim: "2d",
    keywords: keywords(spec),
    files: [{
      glob: "*",
      licence: "MIT",
      source: { url: "https://github.com/originai-game/origin-engine" },
      credit: "Origin Engine",
      derived: "Original recipes written by scripts/vfx-cartoon.ts for the Pixel VFX Forge renderer the cook runs; no external images, models or code.",
    }],
    effects: Object.fromEntries(EFFECTS.map(([name, title, category, element]) =>
      [`content/${name}.ts`, { title, category, element, keywords: ["cartoon", category] }])),
  };
}

const args = process.argv.slice(3);
const only = args.filter((a) => !a.startsWith("--spec="));
const specs = args.find((a) => a.startsWith("--spec="))?.slice(7);
for (const [pack, spec] of Object.entries(PACKS)) {
  if (only.length && !only.includes(pack)) continue;
  console.log(pack, write(pack, spec));
  if (specs) {
    mkdirSync(specs, { recursive: true });
    writeFileSync(join(specs, `${pack}.json`), JSON.stringify(manifestSpec(spec), null, 1) + "\n");
  }
}
