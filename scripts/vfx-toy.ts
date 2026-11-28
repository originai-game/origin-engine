// The toy and craft effect packs, written from one table of recipes.
//
//   bun scripts/vfx-toy.ts [<out dir> [<pack>...] [--spec=<dir>]]
//
// The out dir defaults to packages/vfx-packs; `--spec=<dir>` also writes each
// pack's `vfx-pack.ts manifest` spec there.
//
// Writes `low-poly/`, the chunky mesh effects of the low-poly look.
// Every effect is solid matter, the way a toy world is built: smoke and dust
// are puffs that pop up and shrink away instead of fading, debris and sparks
// are pieces, water is glossy drops, fire is flame-shaped pieces lit from
// inside. The geometric mechanisms share this pack under geometric/.
//
// Shapes the engine has no mesh for (a star, a coin, a heart, a flame, a
// drop, a ribbon, a cloud cutout) are small glTF models the script
// writes itself (models/*.glb), each declared by a `file.import` with
// `conform: "effect"` (content/models/<name>.ts).
// Scale: metres, a figure about 1 m tall. Positions are in the effect's
// space, y up; an effect stands on its origin unless its placement says.
//
// After writing, `bun scripts/vfx-pack.ts manifest <pack> --spec
// <dir>/<pack>.json` and `preview` fill assets.json and preview/ as for every
// pack.
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const OUT = process.argv[2] && !process.argv[2].startsWith("--")
  ? resolve(process.argv[2])
  : resolve(import.meta.dir, "..", "packages", "vfx-packs");

// ---------- numbers ----------

const f = (x: number) => {
  const r = Math.round(x * 1000) / 1000;
  return Object.is(r, -0) ? "0" : String(r);
};

// ---------- colour: sRGB hex in the recipes, linear in the files ----------

type RGB = [number, number, number];
const parse = (c: string): RGB => [1, 3, 5].map((i) => parseInt(c.slice(i, i + 2), 16) / 255) as RGB;
const toLinear = (c: number) => (c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));

// ---------- the variants ----------

/** What a piece of matter is in a variant: the mesh a particle draws. */
type Kind = "puff" | "chip" | "bit" | "drop" | "star" | "coin" | "heart" | "flame" | "ribbon" | "gem";

interface Variant {
  pack: string;
  title: string;
  looks: string[];
  description: string;
  /** The mesh a kind draws: an engine mesh or one of the pack's models. */
  mesh: (k: Kind) => ModelName | "box" | "sphere";
  /** Height over width of a kind's piece (a card is thin), or undefined. */
  thin: (k: Kind) => number | undefined;
  /** Solid pieces' surface roughness. */
  roughness: number;
  /** Glossier pieces (water, ice, gems). */
  gloss: number;
  /** The recipe's sRGB colour as the variant paints it. */
  paint: (c: RGB) => RGB;
  /** Light-emitting colours (fire, magic): how hot they read. */
  glow: number;
  /** Ring segments. */
  segments: number;
  /** Lit pieces' share of the light behind them that passes through (thin card), 0 for solid ones. */
  translucency: number;
  /** Kinds cut flat that stand up facing the eye, rocking a little, instead of tumbling: a card cloud seen edge on is a sliver. */
  stand?: Kind[];
  /** The models the pack ships. */
  models: Record<string, () => Mesh>;
}

// ---------- the declarations ----------

/** A declaration's value: written into its content/*.ts file as JSON, refs as ref(). */
type J = { [k: string]: unknown };
const ref = (id: string) => ({ $ogRef: id });
/** The pack-relative id of a declaration or model: `base/vfx/<pack>/<stem>`. */
const id = (pack: string, stem: string) => ref(`base/vfx/${pack}/${stem}`);
/** Leaves out the keys whose value is undefined. */
const obj = (o: J): J => Object.fromEntries(Object.entries(o).filter(([, x]) => x !== undefined));
const n3 = (x: number) => {
  const r = Math.round(x * 1000) / 1000;
  return Object.is(r, -0) ? 0 : r;
};
const nums = (s: string) => s.trim().split(/\s+/).map(Number);
/** "t v t v ..." as [[t, v], ...]. */
const curve = (s: string) => {
  const n = nums(s), out: number[][] = [];
  for (let i = 0; i < n.length; i += 2) out.push([n3(n[i]), n3(n[i + 1])]);
  return out;
};
const keyed = (s: string) => ({ kind: "curve", keys: curve(s) });

/** "point", "sphere <r> [surface]", "cone <deg> <r>" or "box <x> <y> <z>" (half extents). */
function shapeOf(s: string): J {
  const [kind, ...rest] = s.trim().split(/\s+/);
  const n = rest.filter((x) => x !== "surface").map(Number);
  if (kind === "point") return { kind };
  if (kind === "sphere") return obj({ kind, radius: n[0], surface: rest.includes("surface") || undefined });
  if (kind === "cone") return { kind, angleDegrees: n[0], radius: n[1] };
  if (kind === "box") return { kind, halfExtents: n };
  throw new Error(`shape ${s}`);
}

/** The engine mesh or model a kind draws, as a particle's render mesh. */
const meshOf = (v: Variant, k: Kind) => {
  const m = v.mesh(k);
  return m === "box" || m === "sphere" ? m : id(v.pack, `models/${m}`);
};

/** A declaration file: the licence line, a note and the value. */
const declaration = (fn: "particles" | "effect" | "vfx", note: string, value: J) =>
  `import { file, ref } from '@origingame/engine/author';\n\n// MIT Origin Engine (scripts/vfx-toy.ts). ${note}\nexport default file.${fn}(${
    JSON.stringify(value, null, 2).replace(/\{\s*"\$ogRef": ("[^"]+")\s*\}/g, "ref($1)")
  });\n`.replace("{ file, ref }", JSON.stringify(value).includes('"$ogRef"') ? "{ file, ref }" : "{ file }");

interface Piece {
  kind: Kind;
  /** sRGB colours: one, or a start and an end over the life. */
  col: string | [string, string];
  /** Emits light: unlit, colour times the variant's glow. */
  hot?: number;
  n: number;
  at?: number;
  /** Bursts more: [time, count] within the cycle. */
  more?: [number, number][];
  rate?: number;
  duration?: number;
  loop?: boolean;
  life: [number, number];
  speed: [number, number];
  size: [number, number];
  shape: string;
  spread?: number;
  gravity?: number;
  drag?: number;
  spin?: number;
  grow?: string;
  height?: string;
  speedOverLife?: string;
  noise?: string;
  collide?: string;
  trail?: string;
  /** A flame thrown in a direction points along its velocity instead of standing up. */
  along?: boolean;
  alpha?: string;
  space?: "local";
  attractor?: string;
  inherit?: number;
  wind?: number;
  gloss?: boolean;
  /** A second colour each piece lies between (sRGB). */
  vary?: string;
  /** Drops born where each piece dies: an emitter of the pack's. */
  sub?: { emitter: string; count: number; inherit: number };
}

/** Pop in, hold, shrink away: how a toy puff lives without fading. */
const POP = "0 0.25 0.12 1 0.7 0.9 1 0";
const PIECE_OUT = "0 1 0.8 1 1 0";

function emitter(v: Variant, p: Piece): J {
  const paint = (c: string) => v.paint(parse(c)).map(toLinear) as RGB;
  const k = p.hot ? p.hot * v.glow : 1;
  const c0 = paint(Array.isArray(p.col) ? p.col[0] : p.col).map((x) => x * k);
  const c1 = Array.isArray(p.col) ? paint(p.col[1]).map((x) => x * k) : c0;
  const alpha = curve(p.alpha ?? "0 1 1 1");
  const keys = (from: number[], to: number[]) =>
    alpha.map(([t, a]) => [t, [...from.map((x, j) => n3(x + (to[j] - x) * t)), a]]);
  const vary = p.vary ? paint(p.vary).map((x) => x * k) : undefined;
  const size = p.size;
  const thin = v.thin(p.kind);
  const stand = !p.along && (v.stand ?? []).includes(p.kind);
  const height = p.height ?? (thin !== undefined ? `0 ${f(thin)} 1 ${f(thin)}` : undefined);
  const lit = !p.hot;
  const max = Math.min(4096, Math.ceil((p.n + (p.more ?? []).reduce((s, [, c]) => s + c, 0) + (p.rate ?? 0) * p.life[1] * 1.3) * 1.25) + 2);
  const bursts = [...(p.n > 0 ? [[p.at ?? 0, p.n]] : []), ...(p.more ?? [])].map(([time, count]) => ({ time: n3(time), count }));
  const spin = stand ? 30 : p.spin ?? 120;
  const collide = p.collide && nums(p.collide.replace(/depth|die/g, ""));
  const noise = p.noise && nums(p.noise);
  const attractor = p.attractor && nums(p.attractor);
  const trail = p.trail && p.trail.trim().split(/\s+/);
  return obj({
    maxParticles: max,
    shape: shapeOf(p.shape),
    spread: p.spread,
    rate: p.rate ?? 0,
    bursts: bursts.length ? bursts : undefined,
    duration: p.duration ?? 0.05,
    looping: !!p.loop,
    lifetime: p.life,
    speed: p.speed,
    size: size.map(n3),
    sizeOverLife: curve(p.grow ?? PIECE_OUT),
    heightOverLife: height && curve(height),
    speedOverLife: p.speedOverLife && curve(p.speedOverLife),
    // Flames keep their point up (a mesh's y); a standing cutout rocks; everything else tumbles.
    rotation: p.kind === "flame" ? [-8, 8] : stand ? [-20, 20] : [0, 360],
    align: p.along ? { kind: "velocity", stretch: 0 } : stand ? { kind: "upright", axis: [0, 1, 0] } : undefined,
    angularVelocity: [-spin, spin],
    gravity: [0, p.gravity ?? 0, 0],
    drag: p.drag,
    wind: p.wind,
    noise: noise && { strength: noise[0], frequency: noise[1], scroll: noise[2] },
    attractors: attractor && [{ position: attractor.slice(0, 3), strength: attractor[3], radius: attractor[4] }],
    inheritVelocity: p.inherit,
    space: p.space,
    render: { kind: "mesh", mesh: meshOf(v, p.kind) },
    lit: lit || undefined,
    roughness: lit ? (p.gloss ? v.gloss : v.roughness) : undefined,
    translucency: lit && v.translucency > 0 ? v.translucency : undefined,
    blend: "alpha",
    trail: trail && obj({ lifetime: Number(trail[0]), width: trail[1] && trail[1] !== "only" ? Number(trail[1]) : undefined, head: trail.includes("only") ? false : undefined }),
    colorOverLife: keys(c0, c1),
    randomColorOverLife: vary && keys(vary, vary),
    collision: collide && obj({ bounce: collide[0], friction: collide[1], against: "depth", die: p.collide!.includes("die") || undefined }),
    subEmitters: p.sub && [{ trigger: "death", emitter: id(v.pack, `emitters/${p.sub.emitter}`), count: p.sub.count, inheritVelocity: p.sub.inherit }],
  });
}

// ---------- composed effects ----------

type El = (v: Variant, emit: (name: string, p: Piece, note: string) => J) => J;

interface Effect {
  name: string;
  title: string;
  category: string;
  element?: string;
  placement: string;
  duration: number;
  loop?: boolean;
  impact?: number;
  els: El[];
}

const EFFECTS: Effect[] = [];
const fx = (e: Effect) => EFFECTS.push(e);

/** Where and when an element plays, around its node. */
interface Place { at?: number; life?: number; until?: number; offset?: number[]; rotate?: number[]; move?: number[] }
const element = (o: Place, node: J): J =>
  obj({ at: o.at, life: o.life, until: o.until, offset: o.offset, rotate: o.rotate, velocity: o.move, node });

/** A colour as the variant paints it, linear, times its glow when it emits light. */
const painted = (v: Variant, col: string, hot?: number) =>
  v.paint(parse(col)).map(toLinear).map((x) => n3(x * (hot ? hot * v.glow : 1)));

/** An emitter element. */
const E = (name: string, p: Piece, note: string, opts: Omit<Place, "life"> = {}): El =>
  (_v, emit) => element(opts, { kind: "emitter", asset: emit(name, p, note) });

/** A flat ring: a shockwave on the ground, a halo, a ripple. */
const RING = (o: { at?: number; life: number; scale: string; col: string; alpha?: number; inner?: number; offset?: number[]; rotate?: number[]; hot?: number; spin?: number[]; fade?: boolean }): El =>
  (v) => element({ at: o.at, life: o.life, offset: o.offset ?? [0, 0.02, 0], rotate: o.rotate ?? [-90, 0, 0] }, obj({
    kind: "mesh",
    shape: { kind: "ring", ring: { inner: o.inner ?? 0.82, segments: v.segments } },
    effect: id(v.pack, o.fade === false ? "solid" : "fade"),
    scale: keyed(o.scale),
    spin: o.spin,
    color: [...painted(v, o.col, o.hot), o.alpha ?? 1],
    blend: "alpha",
  }));

/** A solid mesh of the pack (a coin, a star, a gem) that pops, spins and leaves. */
const SHOW = (o: { model: Kind; at?: number; life: number; scale: string; col: string; offset?: number[]; spin?: number[]; move?: number[]; hot?: number }): El =>
  (v) => {
    const m = v.mesh(o.model);
    return element(o, obj({
      kind: "mesh",
      shape: m === "box" || m === "sphere" ? { kind: m } : { kind: "model", asset: id(v.pack, `models/${m}`) },
      effect: id(v.pack, "solid"),
      scale: keyed(o.scale),
      spin: o.spin,
      color: [...painted(v, o.col, o.hot), 1],
      blend: "alpha",
    }));
  };

const LIGHT = (o: { at?: number; life: number; col: string; i: string; d: number; offset?: number[] }): El =>
  () => element({ ...o, offset: o.offset ?? [0, 0.4, 0] }, {
    kind: "light", color: parse(o.col).map(toLinear).map(n3), intensity: keyed(o.i), distance: o.d,
  });

const SHAKE = (t: number): El => () => element({}, { kind: "shake", trauma: t });

const BEAM = (o: { life: number; to: number[]; width: number; col: string; hot: number; at?: number; offset?: number[] }): El =>
  (v) => element(o, { kind: "beam", to: o.to, width: o.width, effect: id(v.pack, "solid"), color: [...painted(v, o.col, o.hot), 1] });

const TRAIL = (o: { life: number; lifetime: number; width: number[]; col: string; hot?: number; orbit?: number[]; rise?: number; offset?: number[]; rotate?: number[]; at?: number }): El =>
  (v) => element(o, obj({
    kind: "trail", lifetime: o.lifetime, width: o.width, orbit: o.orbit, rise: o.rise,
    effect: id(v.pack, "solid"), color: [...painted(v, o.col, o.hot), 1],
  }));

// ---------- palette (sRGB): bright, clean toy colours ----------

const C = {
  white: "#fbf7f0", cream: "#f3e8d2", smoke: "#e8e4df", grey: "#a9a7a4", soot: "#5b5653",
  dust: "#d9b98c", sand: "#e6cf9e", dirt: "#a8794e",
  red: "#e8402f", orange: "#ff8a1f", yellow: "#ffd23a", gold: "#f6b91a", lemon: "#fff28a",
  green: "#4cc45a", mint: "#8ee6a0", lime: "#b5e24a", teal: "#2bc0b4",
  blue: "#3a86f0", sky: "#8cc8ff", water: "#4aa8f0", foam: "#e6f6ff", ice: "#a8e2ff",
  purple: "#8a5ae6", violet: "#b58cff", pink: "#ff6fa8", rose: "#ff9cc0",
  wood: "#c58a52", woodDark: "#8e5a32", rock: "#8f8a85", rockLight: "#b9b3ab",
  poison: "#86d13a", poisonDark: "#4f9a2a", leaf: "#6cbf3a", autumn: "#f09a2a",
};

// ---------- building blocks ----------

const puffs = (col: string, n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "puff", col, n, life: [0.5, 0.9], speed: [0.6, 1.6], size: [0.18, 0.32],
  shape: "sphere 0.12", spread: 0, gravity: 0.6, drag: 3.5, spin: 40, grow: POP, wind: 0.6, ...o,
});
const chips = (col: string, n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "chip", col, n, life: [0.7, 1.2], speed: [2, 4.2], size: [0.06, 0.12],
  shape: "cone 55 0.08", gravity: -9.8, drag: 0.6, spin: 420, collide: "0.35 0.5 depth", ...o,
});
const stars = (col: string, n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "star", col, n, life: [0.35, 0.6], speed: [2.2, 3.6], size: [0.1, 0.16],
  shape: "sphere 0.05", spread: 180, gravity: -2, drag: 4, spin: 360, ...o,
});
const drops = (n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "drop", col: C.water, n, life: [0.45, 0.8], speed: [1.6, 3.2], size: [0.05, 0.09],
  shape: "cone 35 0.08", gravity: -9.8, drag: 0.4, spin: 60, gloss: true, collide: "0 0 depth die", ...o,
});
const flames = (n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "flame", col: [C.yellow, C.red], hot: 1.35, n, life: [0.35, 0.6], speed: [0.8, 1.5], size: [0.14, 0.24],
  shape: "sphere 0.08", spread: 0, gravity: 1.5, drag: 1.5, spin: 10, grow: "0 0.3 0.15 1 1 0", ...o,
});
const bits = (col: string, n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "bit", col, n, life: [0.3, 0.6], speed: [1.5, 3], size: [0.035, 0.06],
  shape: "sphere 0.05", spread: 180, gravity: -3, drag: 2, spin: 0, ...o,
});
const confettiOf = (col: string, n: number, o: Partial<Piece> = {}): Piece => ({
  kind: "ribbon", col, n, life: [1.6, 2.4], speed: [3, 5.5], size: [0.12, 0.17],
  shape: "cone 30 0.1", gravity: -4, drag: 2.6, spin: 540, noise: "1.2 1.5 0.5", ...o,
});
const RAINBOW = [C.red, C.yellow, C.blue, C.green, C.pink];

// ---------- the recipes ----------

// Explosions
fx({ name: "explosion", title: "Explosion", category: "explosion", element: "fire", placement: "on the ground", duration: 2, impact: 0, els: [
  E("explosion-core", flames(16, { along: true, col: [C.lemon, C.orange], hot: 1.7, life: [0.18, 0.32], speed: [2.5, 4.5], size: [0.32, 0.5], shape: "sphere 0.2", spread: 180, gravity: 0, drag: 6, grow: "0 0.4 0.2 1 1 0" }), "The hot heart of a toy blast: fat flame pieces thrown out and burning down."),
  E("explosion-puffs", puffs(C.smoke, 18, { col: [C.orange, C.smoke], life: [0.9, 1.4], speed: [2.4, 4.2], size: [0.42, 0.7], shape: "sphere 0.3", spread: 180, gravity: 0.8, drag: 4.5, vary: C.grey }), "The cotton-ball cloud a toy blast leaves: puffs that pop out, hang and shrink away.", { at: 0.04 }),
  E("explosion-soot", puffs(C.soot, 8, { life: [0.8, 1.2], speed: [1.6, 3], size: [0.3, 0.5], shape: "sphere 0.25", spread: 180, gravity: 1.4, drag: 4 }), "Darker puffs that rise through the cloud.", { at: 0.08, offset: [0, 0.2, 0] }),
  E("explosion-chips", chips(C.orange, 14, { vary: C.yellow, speed: [3.5, 6], size: [0.07, 0.12] }), "Bright chips flung out and bouncing on the table."),
  RING({ life: 0.35, scale: "0 0.3 0.4 1.9 1 2.4", col: C.cream, inner: 0.86 }),
  LIGHT({ life: 0.4, col: C.orange, i: "0 8 1 0", d: 6, offset: [0, 0.6, 0] }),
  SHAKE(0.4),
] });
fx({ name: "explosion-small", title: "Small Explosion", category: "explosion", element: "fire", placement: "on the ground", duration: 1.4, impact: 0, els: [
  E("small-core", flames(9, { along: true, col: [C.lemon, C.orange], hot: 1.6, life: [0.14, 0.26], speed: [1.8, 3], size: [0.2, 0.32], shape: "sphere 0.12", spread: 180, gravity: 0, drag: 6, grow: "0 0.4 0.2 1 1 0" }), "A small blast's flame pieces."),
  E("small-puffs", puffs(C.smoke, 10, { col: [C.orange, C.smoke], life: [0.6, 1], speed: [1.6, 3], size: [0.26, 0.42], shape: "sphere 0.15", spread: 180, gravity: 0.6, vary: C.grey }), "A small blast's puffs.", { at: 0.03 }),
  E("small-chips", chips(C.yellow, 7, { vary: C.orange, speed: [2.5, 4.5] }), "A few chips."),
  RING({ life: 0.28, scale: "0 0.2 0.4 1.1 1 1.4", col: C.cream, inner: 0.86 }),
  LIGHT({ life: 0.3, col: C.orange, i: "0 4 1 0", d: 4 }),
  SHAKE(0.2),
] });
fx({ name: "poof", title: "Poof", category: "smoke", placement: "centred on what vanishes or appears", duration: 1.1, impact: 0, els: [
  E("poof-puffs", puffs(C.white, 14, { life: [0.45, 0.75], speed: [1.8, 2.8], size: [0.28, 0.42], shape: "sphere 0.15 surface", spread: 0, gravity: 0.4, drag: 6, vary: C.smoke }), "A ring of puffs: something vanished or appeared."),
  E("poof-stars", stars(C.yellow, 5, { size: [0.08, 0.12], speed: [1.6, 2.4], life: [0.4, 0.6] }), "Sparkle stars thrown from the puff."),
] });

// Hits
fx({ name: "hit", title: "Hit", category: "hit-spark", placement: "at the point struck", duration: 0.8, impact: 0, els: [
  E("hit-stars", stars(C.yellow, 7, { vary: C.lemon }), "Stars knocked out of a hit."),
  E("hit-bits", bits(C.white, 8, { speed: [2.5, 4], life: [0.15, 0.3], size: [0.03, 0.05] }), "White flecks of the impact."),
  RING({ life: 0.18, scale: "0 0.05 1 0.5", col: C.white, inner: 0.6, rotate: [0, 0, 0], offset: [0, 0, 0] }),
] });
fx({ name: "hit-small", title: "Small Hit", category: "hit-spark", placement: "at the point struck", duration: 0.6, impact: 0, els: [
  E("small-stars", stars(C.lemon, 4, { size: [0.07, 0.1], speed: [1.6, 2.6], life: [0.25, 0.45] }), "A tap's few stars."),
  RING({ life: 0.14, scale: "0 0.04 1 0.3", col: C.white, inner: 0.6, rotate: [0, 0, 0], offset: [0, 0, 0] }),
] });
fx({ name: "hit-heavy", title: "Heavy Hit", category: "hit-spark", placement: "at the point struck", duration: 1, impact: 0, els: [
  E("heavy-stars", stars(C.yellow, 12, { size: [0.12, 0.2], speed: [3, 5], life: [0.4, 0.7], vary: C.orange }), "A heavy hit's big stars."),
  E("heavy-bits", bits(C.white, 14, { speed: [3.5, 6], life: [0.15, 0.35], size: [0.035, 0.06] }), "Flecks."),
  E("heavy-puffs", puffs(C.white, 6, { life: [0.3, 0.5], speed: [1.5, 2.5], size: [0.18, 0.28], spread: 180, gravity: 0 }), "A pop of puffs at the impact."),
  RING({ life: 0.22, scale: "0 0.08 1 0.9", col: C.white, inner: 0.7, rotate: [0, 0, 0], offset: [0, 0, 0] }),
  RING({ at: 0.04, life: 0.25, scale: "0 0.1 1 1.2", col: C.yellow, inner: 0.85, rotate: [0, 0, 0], offset: [0, 0, 0] }),
  SHAKE(0.3),
] });
fx({ name: "stomp", title: "Stomp", category: "hit-spark", element: "earth", placement: "on the ground under the foot", duration: 1, impact: 0, els: [
  E("stomp-puffs", puffs(C.dust, 12, { life: [0.4, 0.7], speed: [2.2, 3.2], size: [0.2, 0.32], shape: "cone 88 0.15", gravity: 0.2, drag: 5, vary: C.sand }), "Dust puffs squeezed out flat along the ground."),
  E("stomp-stars", stars(C.yellow, 5, { speed: [2, 3], shape: "cone 30 0.05", spread: 0, gravity: -6 }), "Stars bounced up from the stomp."),
  RING({ life: 0.3, scale: "0 0.2 1 1.3", col: C.cream, inner: 0.84 }),
  SHAKE(0.25),
] });
fx({ name: "stun", title: "Stunned", category: "status", placement: "above the head", duration: 1.2, loop: true, els: [
  ...[0, 120, 240].map((turn) => TRAIL({ life: 1.2, lifetime: 0.3, width: [0.05, 0], col: C.lemon, orbit: [0.24, 0.85], rotate: [0, turn, 0] })),
  E("stun-stars", stars(C.yellow, 0, { rate: 7, duration: 1.2, loop: true, life: [0.35, 0.5], speed: [0, 0], size: [0.1, 0.13], shape: "cone 90 0.24", spread: 0, gravity: 0, drag: 0, spin: 200, grow: "0 0 0.25 1 0.75 1 1 0", vary: C.lemon }), "Stars blinking on the ring round a dizzy head."),
] });

// Weapons and projectiles
fx({ name: "muzzle", title: "Toy Blaster", category: "gunfire", placement: "at the muzzle, +y along the barrel", duration: 0.7, impact: 0, els: [
  E("muzzle-flame", flames(5, { along: true, hot: 1.7, life: [0.1, 0.14], speed: [3, 4.5], size: [0.16, 0.24], shape: "cone 12 0.02", spread: 0, gravity: 0, drag: 8 }), "A short pop of flame from a toy barrel."),
  E("muzzle-puffs", puffs(C.white, 5, { life: [0.35, 0.55], speed: [1.2, 2.2], size: [0.1, 0.16], shape: "cone 20 0.02", spread: 0, gravity: 0.3, drag: 6 }), "The cork pop's puffs.", { at: 0.02, offset: [0, 0.04, 0] }),
  E("muzzle-stars", stars(C.yellow, 3, { size: [0.05, 0.08], speed: [1.5, 2.5], life: [0.15, 0.25], shape: "cone 50 0.02", spread: 0, gravity: 0 }), "Spark stars at the muzzle."),
  LIGHT({ life: 0.08, col: C.orange, i: "0 1.5 1 0", d: 2, offset: [0, 0.15, 0] }),
] });
fx({ name: "bullet", title: "Pellet Trail", category: "projectile", placement: "on the pellet, moved by the game", duration: 0.5, loop: true, els: [
  SHOW({ model: "bit", life: 0.5, scale: "0 0.14 1 0.14", col: C.yellow }),
  TRAIL({ life: 0.5, lifetime: 0.16, width: [0.1, 0], col: C.white }),
  E("bullet-puffs", puffs(C.white, 0, { rate: 30, duration: 0.5, loop: true, life: [0.2, 0.35], speed: [0, 0.2], size: [0.08, 0.12], shape: "point", gravity: 0, drag: 2 }), "Little puffs left behind a pellet."),
] });
fx({ name: "rocket", title: "Toy Rocket Trail", category: "projectile", element: "fire", placement: "at the rocket's tail, -y behind it", duration: 1, loop: true, els: [
  E("rocket-flame", flames(0, { along: true, rate: 40, duration: 1, loop: true, hot: 1.5, life: [0.1, 0.18], speed: [1.5, 2.5], size: [0.1, 0.16], shape: "cone 8 0.02", gravity: 0, drag: 2, inherit: 0.3 }), "The flame out of a toy rocket.", { rotate: [180, 0, 0] }),
  E("rocket-puffs", puffs(C.white, 0, { rate: 26, duration: 1, loop: true, life: [0.6, 1], speed: [0.4, 0.8], size: [0.12, 0.2], shape: "cone 10 0.03", gravity: 0.2, drag: 2, vary: C.smoke }), "A trail of puffs behind it.", { rotate: [180, 0, 0], offset: [0, -0.12, 0] }),
  LIGHT({ life: 1, col: C.orange, i: "0 1 1 1", d: 2, offset: [0, -0.1, 0] }),
] });
fx({ name: "magic-bolt", title: "Magic Bolt", category: "projectile", element: "arcane", placement: "on the bolt, moved by the game", duration: 0.6, loop: true, els: [
  SHOW({ model: "star", life: 0.6, scale: "0 0.24 1 0.24", col: C.violet, hot: 1.6, spin: [0, 0, 600] }),
  TRAIL({ life: 0.6, lifetime: 0.24, width: [0.14, 0], col: C.purple, hot: 1.6 }),
  E("bolt-bits", stars(C.violet, 0, { rate: 30, duration: 0.6, loop: true, hot: 1.6, life: [0.25, 0.45], speed: [0.2, 0.6], size: [0.06, 0.1], gravity: -1 }), "Little stars shed by a magic bolt."),
  LIGHT({ life: 0.6, col: C.violet, i: "0 1 1 1", d: 2, offset: [0, 0, 0] }),
] });
fx({ name: "laser", title: "Toy Laser", category: "beam", element: "lightning", placement: "from the emitter along +y, 4 m", duration: 0.6, impact: 0, els: [
  BEAM({ life: 0.45, to: [0, 4, 0], width: 0.06, col: C.red, hot: 2.2 }),
  BEAM({ life: 0.45, to: [0, 4, 0], width: 0.025, col: C.white, hot: 2 }),
  E("laser-bits", bits(C.rose, 10, { hot: 2, speed: [1, 2], life: [0.2, 0.35] }), "Sparkles at the laser's mouth."),
  E("laser-hit", stars(C.yellow, 6, { speed: [1.5, 2.5], life: [0.25, 0.4] }), "Stars where it lands.", { offset: [0, 4, 0] }),
  LIGHT({ life: 0.45, col: C.red, i: "0 1.5 1 0", d: 2.5, offset: [0, 0.2, 0] }),
] });

// Pickups and rewards
fx({ name: "coin", title: "Coin Collect", category: "pickup", element: "light", placement: "where the coin was", duration: 1, impact: 0, els: [
  SHOW({ model: "coin", life: 0.45, scale: "0 0.3 0.4 0.36 1 0", col: C.gold, move: [0, 1.6, 0], spin: [0, 1100, 0] }),
  E("coin-stars", stars(C.lemon, 7, { size: [0.06, 0.1], speed: [1.4, 2.4], life: [0.3, 0.5], gravity: -1 }), "Sparkle stars of a coin collected.", { at: 0.35, offset: [0, 0.55, 0] }),
] });
fx({ name: "gem", title: "Gem Collect", category: "pickup", element: "light", placement: "where the gem was", duration: 1.1, impact: 0, els: [
  SHOW({ model: "gem", life: 0.5, scale: "0 0.3 0.4 0.36 1 0", col: C.teal, move: [0, 1.4, 0], spin: [0, 900, 0] }),
  E("gem-bits", bits(C.ice, 10, { kind: "gem", gloss: true, size: [0.05, 0.08], speed: [1.5, 2.6], life: [0.35, 0.55], gravity: -2 }), "Glinting gem chips.", { at: 0.4, offset: [0, 0.6, 0] }),
  RING({ at: 0.4, life: 0.25, scale: "0 0.1 1 0.5", col: C.ice, inner: 0.75, rotate: [0, 0, 0], offset: [0, 0.6, 0] }),
] });
fx({ name: "star-collect", title: "Star Collect", category: "pickup", element: "light", placement: "where the star was", duration: 1.4, impact: 0, els: [
  SHOW({ model: "star", life: 0.7, scale: "0 0.45 0.3 0.6 0.8 0.5 1 0", col: C.yellow, move: [0, 1.1, 0], spin: [0, 720, 0] }),
  E("star-ring", stars(C.lemon, 10, { size: [0.08, 0.12], speed: [2, 2.4], life: [0.4, 0.6], shape: "sphere 0.05 surface", spread: 0, gravity: 0, drag: 3 }), "A ring of little stars bursting out.", { at: 0.6, offset: [0, 0.75, 0] }),
  RING({ at: 0.6, life: 0.3, scale: "0 0.15 1 0.9", col: C.white, inner: 0.85, rotate: [0, 0, 0], offset: [0, 0.75, 0] }),
  LIGHT({ life: 0.9, col: C.yellow, i: "0 1.2 0.7 1 1 0", d: 2.5, offset: [0, 0.6, 0] }),
] });
fx({ name: "heart", title: "Heart Pickup", category: "pickup", element: "life", placement: "where the heart was", duration: 1.2, impact: 0, els: [
  SHOW({ model: "heart", life: 0.6, scale: "0 0.3 0.2 0.42 0.4 0.34 1 0", col: C.red, move: [0, 1.2, 0] }),
  E("heart-little", bits(C.pink, 6, { kind: "heart", size: [0.08, 0.12], speed: [0.6, 1.2], life: [0.6, 0.9], gravity: 1.2, drag: 2, spin: 60, shape: "sphere 0.15", vary: C.rose }), "Little hearts floating up.", { at: 0.4, offset: [0, 0.6, 0] }),
] });
fx({ name: "powerup", title: "Power-up", category: "buff", element: "light", placement: "on the ground at the figure's feet", duration: 1.6, impact: 0, els: [
  RING({ life: 0.8, scale: "0 0.2 0.5 0.7 1 0.8", col: C.yellow, inner: 0.8, hot: 1.2, spin: [0, 0, 180] }),
  RING({ at: 0.15, life: 0.7, scale: "0 0.2 0.5 0.55 1 0.6", col: C.orange, inner: 0.8, offset: [0, 0.6, 0], hot: 1.2 }),
  E("power-stars", stars(C.yellow, 0, { rate: 30, duration: 0.8, life: [0.6, 0.9], speed: [1.2, 2], size: [0.07, 0.11], shape: "cone 5 0.5", spread: 0, gravity: 0, drag: 1.5, vary: C.lemon }), "Stars rising round the figure.", { rotate: [0, 0, 0] }),
  LIGHT({ life: 1.2, col: C.yellow, i: "0 0 0.15 2 1 0", d: 3, offset: [0, 0.8, 0] }),
] });
fx({ name: "chest-open", title: "Chest Open", category: "pickup", element: "light", placement: "in the chest's mouth", duration: 1.8, impact: 0, els: [
  E("chest-coins", bits(C.gold, 10, { kind: "coin", size: [0.1, 0.14], speed: [3, 4.2], life: [0.7, 1], shape: "cone 22 0.12", spread: 0, gravity: -9, drag: 0.5, spin: 600, collide: "0.4 0.5 depth" }), "Coins jumping out of an opened chest."),
  E("chest-stars", stars(C.lemon, 10, { size: [0.07, 0.11], speed: [1.6, 2.6], life: [0.5, 0.8], shape: "cone 30 0.15", spread: 0, gravity: 0.5 }), "Sparkle stars rising."),
  LIGHT({ life: 1.2, col: C.yellow, i: "0 3 1 0", d: 3 }),
] });
fx({ name: "checkpoint", title: "Checkpoint", category: "ui", element: "light", placement: "on the ground at the flag", duration: 2, impact: 0, els: [
  RING({ life: 0.8, scale: "0 0.2 1 1.4", col: C.yellow, inner: 0.84 }),
  RING({ at: 0.15, life: 0.8, scale: "0 0.2 1 1.1", col: C.white, inner: 0.84 }),
  E("check-confetti-a", confettiOf(C.green, 22, { shape: "cone 25 0.1", speed: [3.5, 5], size: [0.13, 0.18] }), "Confetti from a checkpoint.", { offset: [0, 0.1, 0] }),
  E("check-confetti-b", confettiOf(C.yellow, 22, { shape: "cone 25 0.1", speed: [3.5, 5], size: [0.13, 0.18] }), "Confetti from a checkpoint.", { offset: [0, 0.1, 0] }),
  E("check-confetti-c", confettiOf(C.white, 10, { shape: "cone 25 0.1", speed: [3.5, 5] }), "Confetti from a checkpoint.", { offset: [0, 0.1, 0] }),
] });
fx({ name: "level-up", title: "Level Up", category: "buff", element: "light", placement: "on the ground at the figure's feet", duration: 2.2, impact: 0, els: [
  RING({ life: 0.9, scale: "0 0.2 0.3 0.9 1 1", col: C.yellow, inner: 0.84, hot: 1.3 }),
  RING({ at: 0.2, life: 0.8, scale: "0 0.2 0.3 0.75 1 0.8", col: C.lemon, inner: 0.84, offset: [0, 0.7, 0], hot: 1.3 }),
  RING({ at: 0.4, life: 0.7, scale: "0 0.15 0.3 0.6 1 0.65", col: C.white, inner: 0.84, offset: [0, 1.4, 0], hot: 1.3 }),
  E("level-stars", stars(C.yellow, 0, { rate: 45, duration: 1, life: [0.8, 1.1], speed: [1.6, 2.6], size: [0.08, 0.13], shape: "cone 6 0.6", spread: 0, gravity: 0, drag: 1.2, vary: C.lemon }), "Stars rising round a levelled figure."),
  E("level-confetti", confettiOf(C.yellow, 16, { shape: "cone 20 0.15", speed: [4, 6], vary: C.orange }), "A pop of confetti.", { at: 0.9, offset: [0, 1.6, 0] }),
  LIGHT({ life: 1.6, col: C.yellow, i: "0 0 0.2 3 1 0", d: 4, offset: [0, 1, 0] }),
] });
fx({ name: "reward-burst", title: "Reward Burst", category: "ui", element: "light", placement: "where the reward is shown", duration: 1.6, impact: 0, els: [
  E("reward-coins", bits(C.gold, 12, { kind: "coin", size: [0.1, 0.14], speed: [3, 4.5], life: [0.9, 1.3], shape: "cone 40 0.1", spread: 0, gravity: -8, drag: 0.5, spin: 700, collide: "0.4 0.5 depth" }), "Coins fountaining from a reward."),
  E("reward-stars", stars(C.lemon, 12, { size: [0.07, 0.11], speed: [2.5, 3.5], life: [0.5, 0.8] }), "Stars of a reward."),
  RING({ life: 0.3, scale: "0 0.15 1 1.2", col: C.white, inner: 0.86, rotate: [0, 0, 0], offset: [0, 0.3, 0] }),
] });

// Magic
fx({ name: "magic", title: "Magic Hit", category: "magic-impact", element: "arcane", placement: "at the point struck", duration: 1.2, impact: 0, els: [
  E("magic-stars", stars(C.violet, 14, { hot: 1.5, size: [0.08, 0.14], speed: [2, 3.5], life: [0.45, 0.7], vary: C.pink }), "Magic stars thrown from a spell's hit."),
  E("magic-puffs", puffs(C.violet, 8, { life: [0.4, 0.7], speed: [1.2, 2], size: [0.2, 0.3], spread: 180, gravity: 0.4, vary: C.rose }), "Coloured puffs of the spell."),
  RING({ life: 0.3, scale: "0 0.1 1 1.1", col: C.violet, inner: 0.84, rotate: [0, 0, 0], offset: [0, 0, 0], hot: 1.4 }),
  LIGHT({ life: 0.4, col: C.violet, i: "0 3 1 0", d: 3, offset: [0, 0, 0] }),
] });
fx({ name: "charge", title: "Charge", category: "magic-impact", element: "arcane", placement: "where the power gathers", duration: 1.2, loop: true, els: [
  SHOW({ model: "bit", life: 1.2, scale: "0 0.12 0.5 0.22 1 0.12", col: C.sky, hot: 1.6 }),
  E("charge-in", stars(C.sky, 0, { rate: 36, duration: 1.2, loop: true, hot: 1.6, life: [0.45, 0.6], speed: [0, 0], size: [0.06, 0.09], shape: "sphere 0.6 surface", spread: 0, gravity: 0, drag: 0, attractor: "0 0 0 14 1", grow: "0 0 0.2 1 0.8 1 1 0", vary: C.white }), "Stars pulled in to a charging spell."),
  LIGHT({ life: 1.2, col: C.sky, i: "0 0.8 0.5 1.6 1 0.8", d: 2.5, offset: [0, 0, 0] }),
] });
fx({ name: "magic-circle", title: "Magic Circle", category: "magic-impact", element: "arcane", placement: "on the ground", duration: 3, impact: 0.3, els: [
  RING({ life: 3, scale: "0 0 0.1 1 0.9 1 1 0", col: C.violet, inner: 0.9, hot: 1.3, spin: [0, 0, 40], fade: false }),
  RING({ life: 3, scale: "0 0 0.12 0.72 0.9 0.72 1 0", col: C.pink, inner: 0.86, hot: 1.3, spin: [0, 0, -60], fade: false }),
  RING({ life: 3, scale: "0 0 0.14 0.45 0.9 0.45 1 0", col: C.violet, inner: 0.7, hot: 1.3, spin: [0, 0, 90], fade: false }),
  E("circle-stars", stars(C.violet, 0, { rate: 22, duration: 2.6, hot: 1.5, life: [0.8, 1.2], speed: [0.6, 1], size: [0.06, 0.1], shape: "cone 2 0.9", spread: 0, gravity: 0, drag: 0.6, vary: C.pink }), "Stars rising off a magic circle.", { at: 0.2 }),
  LIGHT({ life: 3, col: C.violet, i: "0 0 0.1 1.6 0.9 1.6 1 0", d: 3.5, offset: [0, 0.3, 0] }),
] });
fx({ name: "portal", title: "Portal", category: "magic-impact", element: "arcane", placement: "its foot on the ground, facing +z", duration: 2, loop: true, els: [
  RING({ life: 2, scale: "0 0.8 1 0.8", col: C.purple, inner: 0.82, offset: [0, 0.85, 0], rotate: [0, 0, 0], hot: 1.4, spin: [0, 0, 90], fade: false }),
  RING({ life: 2, scale: "0 0.62 1 0.62", col: C.violet, inner: 0, offset: [0, 0.85, -0.01], rotate: [0, 0, 0], hot: 0.9, fade: false }),
  E("portal-stars", stars(C.white, 0, { rate: 24, duration: 2, loop: true, hot: 1.4, life: [0.6, 0.9], speed: [0, 0], size: [0.05, 0.08], shape: "sphere 0.75 surface", spread: 0, gravity: 0, drag: 0, attractor: "0 0 0 6 1", grow: "0 0 0.2 1 1 0", vary: C.violet }), "Stars swirling into a portal.", { offset: [0, 0.85, 0] }),
  LIGHT({ life: 2, col: C.violet, i: "0 1.5 1 1.5", d: 3, offset: [0, 0.85, 0.3] }),
] });
fx({ name: "teleport", title: "Teleport", category: "magic-impact", element: "arcane", placement: "on the ground at the figure's feet", duration: 1.4, impact: 0.3, els: [
  RING({ life: 0.7, scale: "0 0.2 0.4 0.55 1 0.6", col: C.sky, inner: 0.8, hot: 1.3 }),
  E("teleport-column", stars(C.sky, 0, { rate: 60, duration: 0.5, hot: 1.5, life: [0.4, 0.6], speed: [2.5, 3.5], size: [0.06, 0.1], shape: "cone 2 0.4", spread: 0, gravity: 0, drag: 0.5, vary: C.white }), "A column of stars lifting the figure away."),
  E("teleport-poof", puffs(C.white, 10, { life: [0.35, 0.6], speed: [1.6, 2.4], size: [0.22, 0.34], shape: "sphere 0.2 surface", spread: 0, gravity: 0.6, drag: 6 }), "The puff it leaves.", { at: 0.3, offset: [0, 0.5, 0] }),
  LIGHT({ life: 0.8, col: C.sky, i: "0 3 1 0", d: 3, offset: [0, 0.6, 0] }),
] });
fx({ name: "shield", title: "Shield", category: "buff", element: "light", placement: "centred on the figure", duration: 2, loop: true, els: [
  (v) => element({ life: 2 }, {
    kind: "mesh",
    shape: { kind: "sphere" },
    effect: id(v.pack, "bubble"),
    scale: keyed("0 1.1 0.5 1.14 1 1.1"),
    color: [...painted(v, C.sky), 0.5],
    blend: "alpha",
  }),
  RING({ life: 2, scale: "0 0.56 0.5 0.6 1 0.56", col: C.white, inner: 0.9, offset: [0, -0.5, 0], hot: 1.2, spin: [0, 0, 60], fade: false }),
  E("shield-glints", stars(C.white, 0, { rate: 6, duration: 2, loop: true, life: [0.4, 0.6], speed: [0, 0], size: [0.06, 0.09], shape: "sphere 0.56 surface", spread: 0, gravity: 0, drag: 0, grow: "0 0 0.3 1 1 0", hot: 1.4 }), "Glints twinkling on the shield."),
] });
fx({ name: "heal", title: "Heal", category: "heal", element: "life", placement: "on the ground at the figure's feet", duration: 1.8, impact: 0, els: [
  RING({ life: 0.7, scale: "0 0.2 0.4 0.6 1 0.65", col: C.green, inner: 0.82 }),
  E("heal-hearts", bits(C.mint, 0, { kind: "heart", rate: 14, duration: 1, life: [0.7, 1], speed: [0.8, 1.3], size: [0.1, 0.14], shape: "cone 4 0.4", spread: 0, gravity: 0.3, drag: 1, spin: 60, vary: C.green }), "Green hearts rising round a healed figure."),
  E("heal-plus", stars(C.lemon, 0, { rate: 18, duration: 1, life: [0.5, 0.8], speed: [0.8, 1.4], size: [0.05, 0.08], shape: "cone 4 0.45", spread: 0, gravity: 0, drag: 1 }), "Little sparkles among them."),
  LIGHT({ life: 1.2, col: C.green, i: "0 0 0.2 1.5 1 0", d: 3, offset: [0, 0.6, 0] }),
] });
fx({ name: "buff", title: "Buff", category: "buff", placement: "on the ground at the figure's feet", duration: 1.4, impact: 0, els: [
  RING({ life: 0.6, scale: "0 0.2 0.5 0.6 1 0.65", col: C.orange, inner: 0.82 }),
  E("buff-arrows", bits(C.orange, 0, { kind: "drop", rate: 22, duration: 0.8, life: [0.5, 0.7], speed: [1.6, 2.4], size: [0.08, 0.12], shape: "cone 3 0.42", spread: 0, gravity: 0, drag: 1, spin: 0, vary: C.yellow }), "Drops shooting up round a strengthened figure."),
] });
fx({ name: "debuff", title: "Debuff", category: "debuff", element: "shadow", placement: "above the figure's head", duration: 1.4, impact: 0, els: [
  E("debuff-drops", bits(C.purple, 0, { kind: "drop", rate: 18, duration: 0.8, life: [0.5, 0.7], speed: [0.6, 1], size: [0.08, 0.12], shape: "cone 4 0.35", spread: 0, gravity: -3, drag: 0.5, spin: 0, vary: C.soot }), "Dark drops falling round a weakened figure.", { rotate: [180, 0, 0] }),
  E("debuff-puffs", puffs(C.purple, 6, { life: [0.6, 0.9], speed: [0.3, 0.6], size: [0.16, 0.26], spread: 180, gravity: -0.3, vary: C.soot }), "A dull cloud over its head."),
] });

// Status
fx({ name: "burning", title: "Burning", category: "status", element: "fire", placement: "on the figure, up to 1 m", duration: 1, loop: true, els: [
  E("burning-flames", flames(0, { rate: 40, duration: 1, loop: true, life: [0.35, 0.55], speed: [0.4, 0.8], size: [0.22, 0.34], shape: "box 0.18 0.4 0.18", gravity: 1 }), "Flames licking up a burning figure.", { offset: [0, 0.5, 0] }),
  E("burning-puffs", puffs(C.soot, 0, { rate: 6, duration: 1, loop: true, life: [0.6, 0.9], speed: [0.6, 1], size: [0.12, 0.2], shape: "sphere 0.15", gravity: 1 }), "Little sooty puffs.", { offset: [0, 1, 0] }),
  LIGHT({ life: 1, col: C.orange, i: "0 1.4 0.5 1.8 1 1.4", d: 2.5, offset: [0, 0.6, 0] }),
] });
fx({ name: "poison", title: "Poisoned", category: "status", element: "poison", placement: "on the figure", duration: 1.2, loop: true, els: [
  E("poison-bubbles", bits(C.poison, 0, { kind: "puff", rate: 14, duration: 1.2, loop: true, gloss: true, life: [0.6, 0.9], speed: [0.4, 0.8], size: [0.07, 0.13], shape: "box 0.2 0.35 0.2", spread: 0, gravity: 0.6, drag: 1, grow: POP, vary: C.lime }), "Green bubbles rising off a poisoned figure.", { offset: [0, 0.5, 0] }),
] });
fx({ name: "frozen", title: "Frozen", category: "status", element: "ice", placement: "on the figure", duration: 1.5, loop: true, els: [
  E("frozen-crystals", bits(C.ice, 0, { kind: "gem", rate: 10, duration: 1.5, loop: true, gloss: true, life: [0.8, 1.2], speed: [0.1, 0.3], size: [0.08, 0.14], shape: "box 0.25 0.45 0.25", spread: 180, gravity: 0, drag: 2, spin: 40, grow: POP, vary: C.white }), "Ice crystals growing on a frozen figure.", { offset: [0, 0.5, 0] }),
  E("frozen-flakes", stars(C.white, 0, { rate: 8, duration: 1.5, loop: true, life: [0.8, 1.2], speed: [0.1, 0.3], size: [0.04, 0.06], shape: "box 0.3 0.1 0.3", gravity: -0.4, drag: 1 }), "Flakes drifting off it.", { offset: [0, 1, 0] }),
] });
fx({ name: "shocked", title: "Shocked", category: "status", element: "lightning", placement: "on the figure", duration: 0.6, loop: true, els: [
  BEAM({ life: 0.08, offset: [-0.2, 0.4, 0], to: [0.4, 0.5, 0.1], width: 0.07, col: C.lemon, hot: 2.4 }),
  BEAM({ at: 0.15, life: 0.08, offset: [0.2, 1.1, 0], to: [-0.35, -0.5, 0.1], width: 0.07, col: C.lemon, hot: 2.4 }),
  BEAM({ at: 0.32, life: 0.08, offset: [0, 0.2, 0.15], to: [0.1, 0.9, -0.25], width: 0.07, col: C.white, hot: 2.4 }),
  BEAM({ at: 0.46, life: 0.08, offset: [-0.15, 1.2, 0], to: [0.3, -0.4, 0], width: 0.07, col: C.lemon, hot: 2.4 }),
  E("shock-stars", stars(C.lemon, 0, { more: [[0, 4], [0.15, 4], [0.32, 4], [0.46, 4]], duration: 0.6, loop: true, hot: 1.8, size: [0.09, 0.13], speed: [1.5, 2.5], life: [0.15, 0.3], shape: "sphere 0.3" }), "Sparks popping off a shocked figure.", { offset: [0, 0.6, 0] }),
  LIGHT({ life: 0.6, col: C.lemon, i: "0 1.6 0.12 0.2 0.25 1.6 0.4 0.2 0.77 1.6 1 0.2", d: 3, offset: [0, 0.7, 0] }),
] });

// Smoke and dust
fx({ name: "dust", title: "Dust Cloud", category: "smoke", element: "earth", placement: "on the ground", duration: 1.6, impact: 0, els: [
  E("dust-puffs", puffs(C.dust, 14, { life: [0.7, 1.1], speed: [1.4, 2.4], size: [0.26, 0.42], shape: "cone 70 0.2", gravity: 0.3, drag: 3.5, vary: C.sand }), "A cloud of dust puffs."),
  E("dust-grit", chips(C.dirt, 8, { kind: "bit", size: [0.03, 0.05], speed: [1.6, 2.8], life: [0.5, 0.8] }), "Grit bouncing out of it."),
] });
fx({ name: "run-dust", title: "Run Dust", category: "smoke", element: "earth", placement: "at the foot, moved with the figure", duration: 0.4, loop: true, els: [
  E("run-puffs", puffs(C.dust, 0, { rate: 14, duration: 0.4, loop: true, life: [0.35, 0.55], speed: [0.3, 0.7], size: [0.1, 0.16], shape: "cone 60 0.05", gravity: 0.4, drag: 3, vary: C.sand }), "Little puffs kicked up by running feet."),
] });
fx({ name: "jump-dust", title: "Jump Dust", category: "smoke", element: "earth", placement: "on the ground under the jump", duration: 0.8, impact: 0, els: [
  E("jump-puffs", puffs(C.dust, 8, { life: [0.35, 0.55], speed: [1.4, 2], size: [0.14, 0.22], shape: "cone 85 0.1", gravity: 0.2, drag: 6, vary: C.sand }), "Puffs pushed out by a jump."),
] });
fx({ name: "land-dust", title: "Land Dust", category: "smoke", element: "earth", placement: "on the ground under the landing", duration: 1, impact: 0, els: [
  E("land-puffs", puffs(C.dust, 12, { life: [0.45, 0.7], speed: [2, 2.8], size: [0.18, 0.28], shape: "cone 88 0.15", gravity: 0.2, drag: 6, vary: C.sand }), "A ring of puffs flattened out by a landing."),
  RING({ life: 0.25, scale: "0 0.2 1 0.9", col: C.cream, inner: 0.86 }),
] });
fx({ name: "double-jump", title: "Double Jump", category: "movement", placement: "under the figure, in the air", duration: 0.8, impact: 0, els: [
  RING({ life: 0.3, scale: "0 0.15 1 0.7", col: C.white, inner: 0.82, offset: [0, 0, 0] }),
  E("double-puffs", puffs(C.white, 8, { life: [0.3, 0.5], speed: [1.6, 2.2], size: [0.12, 0.18], shape: "cone 88 0.12", gravity: 0, drag: 6 }), "Air puffs pushed off a mid-air jump."),
  E("double-stars", stars(C.yellow, 4, { size: [0.06, 0.09], speed: [1.2, 1.8], life: [0.3, 0.45], shape: "cone 70 0.1", spread: 0, gravity: -2 }), "A few stars."),
] });
fx({ name: "dash", title: "Dash", category: "movement", placement: "behind the figure, +z the way it dashes", duration: 0.6, impact: 0, els: [
  E("dash-puffs", puffs(C.white, 10, { life: [0.3, 0.5], speed: [2, 3], size: [0.14, 0.22], shape: "cone 18 0.25", spread: 0, gravity: 0, drag: 5 }), "A burst of puffs behind a dash.", { rotate: [-90, 0, 0], offset: [0, 0.5, 0] }),
  E("dash-lines", bits(C.white, 8, { kind: "ribbon", life: [0.15, 0.25], speed: [5, 7], size: [0.06, 0.08], shape: "box 0.3 0.35 0.05", spread: 0, gravity: 0, drag: 0, spin: 0, height: "0 4 1 4" }), "Speed streaks behind a dash.", { rotate: [-90, 0, 0], offset: [0, 0.5, 0] }),
] });
fx({ name: "skid", title: "Skid", category: "smoke", element: "earth", placement: "at the sliding foot or wheel, moved by the game", duration: 0.5, loop: true, els: [
  E("skid-puffs", puffs(C.dust, 0, { rate: 22, duration: 0.5, loop: true, life: [0.35, 0.55], speed: [0.6, 1.2], size: [0.12, 0.18], shape: "cone 70 0.05", gravity: 0.3, drag: 3, vary: C.sand }), "Puffs ploughed up by a skid."),
  E("skid-grit", chips(C.dirt, 0, { kind: "bit", rate: 14, duration: 0.5, loop: true, size: [0.025, 0.04], speed: [1, 2], life: [0.3, 0.5] }), "Grit."),
] });
fx({ name: "chimney-smoke", title: "Chimney Smoke", category: "smoke", placement: "at the chimney's top", duration: 2, loop: true, els: [
  E("chimney-puffs", puffs(C.smoke, 0, { rate: 8, duration: 2, loop: true, life: [2.4, 3], speed: [0.6, 0.9], size: [0.24, 0.34], shape: "cone 6 0.06", gravity: 0.1, drag: 0.4, grow: "0 0.3 0.15 0.8 0.7 1.2 1 0", wind: 0.8, vary: C.white }), "A slow string of puffs from a chimney."),
] });
fx({ name: "steam", title: "Steam", category: "smoke", element: "water", placement: "at the vent, +y the way it blows", duration: 1, loop: true, els: [
  E("steam-puffs", puffs(C.white, 0, { rate: 24, duration: 1, loop: true, life: [0.5, 0.8], speed: [2.2, 3], size: [0.12, 0.2], shape: "cone 8 0.04", gravity: 0.5, drag: 2, grow: "0 0.3 0.15 0.8 0.7 1.4 1 0" }), "Puffs of steam hissing out of a vent."),
] });

// Water
fx({ name: "splash", title: "Splash", category: "splash", element: "water", placement: "on the water's surface", duration: 1.4, impact: 0, els: [
  E("splash-crown", drops(18, { speed: [2.4, 3.6], size: [0.07, 0.12], shape: "cone 22 0.2", life: [0.6, 0.9] }), "A crown of drops thrown up by a splash."),
  E("splash-column", drops(8, { speed: [3.6, 4.6], size: [0.09, 0.14], shape: "cone 6 0.04", life: [0.7, 1] }), "The column up the middle."),
  E("splash-foam", puffs(C.foam, 8, { life: [0.4, 0.7], speed: [1.2, 1.8], size: [0.14, 0.22], shape: "cone 88 0.15", gravity: 0, drag: 5, gloss: true }), "Foam puffs on the surface."),
  RING({ life: 0.9, scale: "0 0.2 1 1.4", col: C.foam, inner: 0.88 }),
  RING({ at: 0.2, life: 0.9, scale: "0 0.15 1 1", col: C.foam, inner: 0.88 }),
] });
fx({ name: "splash-small", title: "Small Splash", category: "splash", element: "water", placement: "on the water's surface", duration: 1, impact: 0, els: [
  E("small-crown", drops(10, { speed: [1.6, 2.4], size: [0.05, 0.08], shape: "cone 22 0.1", life: [0.4, 0.6] }), "A small splash's drops."),
  RING({ life: 0.7, scale: "0 0.1 1 0.8", col: C.foam, inner: 0.86 }),
] });
fx({ name: "ripple", title: "Ripple", category: "splash", element: "water", placement: "on the water's surface", duration: 1.8, loop: true, els: [
  RING({ life: 1.2, scale: "0 0.1 1 0.9", col: C.foam, inner: 0.9 }),
  RING({ at: 0.6, life: 1.2, scale: "0 0.1 1 0.9", col: C.foam, inner: 0.9 }),
] });
fx({ name: "bubbles", title: "Bubbles", category: "splash", element: "water", placement: "under water, rising", duration: 1.5, loop: true, els: [
  E("bubbles-rise", bits(C.sky, 0, { kind: "puff", rate: 10, duration: 1.5, loop: true, gloss: true, life: [1, 1.4], speed: [0.5, 0.8], size: [0.05, 0.11], shape: "sphere 0.12", spread: 20, gravity: 0.2, drag: 1, noise: "0.4 2 0.4", grow: POP, alpha: "0 0.8 1 0.8" }), "Glossy bubbles wobbling up."),
] });
fx({ name: "drip", title: "Drip", category: "splash", element: "water", placement: "the drop falls 1.5 m to the origin", duration: 1.2, loop: true, els: [
  E("drip-drop", drops(1, { speed: [0, 0], size: [0.1, 0.1], shape: "point", spread: 0, life: [0.56, 0.56], duration: 1.2, loop: true, spin: 0, grow: "0 1 1 1", sub: { emitter: "drip-plip", count: 6, inherit: 0 } }), "One drop let go each cycle.", { offset: [0, 1.5, 0] }),
  RING({ at: 0.55, life: 0.6, scale: "0 0.04 1 0.3", col: C.foam, inner: 0.84 }),
] });
fx({ name: "fountain", title: "Fountain", category: "splash", element: "water", placement: "at the spout, +y up", duration: 1, loop: true, els: [
  E("fountain-jet", drops(0, { rate: 46, duration: 1, loop: true, speed: [3.6, 4.2], size: [0.06, 0.1], shape: "cone 5 0.03", life: [0.75, 0.9], collide: undefined }), "A toy fountain's jet of drops."),
  RING({ life: 1, scale: "0 0.5 0.5 0.6 1 0.5", col: C.foam, inner: 0.85, offset: [0, 0.01, 0], fade: false }),
] });
fx({ name: "wake", title: "Boat Wake", category: "splash", element: "water", placement: "at the stern on the water, moved by the game", duration: 0.5, loop: true, els: [
  E("wake-foam", puffs(C.foam, 0, { rate: 30, duration: 0.5, loop: true, life: [0.6, 0.9], speed: [0.5, 1], size: [0.12, 0.2], shape: "cone 80 0.15", gravity: 0, drag: 2, gloss: true }), "Foam puffs churned behind a toy boat."),
  E("wake-drops", drops(0, { rate: 18, duration: 0.5, loop: true, speed: [1, 1.8], size: [0.04, 0.07], shape: "cone 40 0.1", life: [0.3, 0.5] }), "Spray."),
] });

// Fire
fx({ name: "campfire", title: "Campfire", category: "fire", element: "fire", placement: "on the ground", duration: 1, loop: true, els: [
  E("camp-flames", flames(0, { rate: 36, duration: 1, loop: true, life: [0.45, 0.7], speed: [0.3, 0.7], size: [0.3, 0.46], shape: "sphere 0.12", spread: 15, gravity: 1 }), "Flame pieces licking up from a campfire.", { offset: [0, 0.05, 0] }),
  E("camp-embers", bits(C.orange, 0, { rate: 6, duration: 1, loop: true, hot: 2, life: [0.8, 1.2], speed: [0.6, 1.2], size: [0.025, 0.04], shape: "sphere 0.15", spread: 20, gravity: 0.8, drag: 0.5, noise: "0.6 2 0.4" }), "Embers drifting up.", { offset: [0, 0.2, 0] }),
  E("camp-puffs", puffs(C.grey, 0, { rate: 2.5, duration: 1, loop: true, life: [1.4, 2], speed: [0.4, 0.6], size: [0.14, 0.2], shape: "sphere 0.05", gravity: 0.2, drag: 0.4, grow: "0 0.3 0.15 0.8 0.7 1.1 1 0" }), "A thin string of smoke puffs.", { offset: [0, 0.6, 0] }),
  LIGHT({ life: 1, col: C.orange, i: "0 2 0.3 2.6 0.6 1.8 1 2", d: 4 }),
] });
fx({ name: "torch", title: "Torch", category: "fire", element: "fire", placement: "on the torch head", duration: 1, loop: true, els: [
  E("torch-flames", flames(0, { rate: 26, duration: 1, loop: true, life: [0.3, 0.45], speed: [0.3, 0.6], size: [0.17, 0.26], shape: "sphere 0.04", spread: 10, gravity: 1 }), "A torch's small flame."),
  LIGHT({ life: 1, col: C.orange, i: "0 1.2 0.5 1.5 1 1.2", d: 3, offset: [0, 0.15, 0] }),
] });
fx({ name: "fuse", title: "Fuse", category: "fire", element: "fire", placement: "at the fuse's end", duration: 0.5, loop: true, els: [
  E("fuse-stars", stars(C.lemon, 0, { rate: 40, duration: 0.5, loop: true, hot: 2, life: [0.15, 0.3], speed: [1.2, 2], size: [0.04, 0.07], shape: "sphere 0.02", gravity: -3, vary: C.orange }), "Sparks spitting from a lit fuse."),
  LIGHT({ life: 0.5, col: C.orange, i: "0 0.35 0.5 0.5 1 0.35", d: 1.2, offset: [0, 0.05, 0] }),
] });

// Vehicles
fx({ name: "exhaust", title: "Kart Exhaust", category: "exhaust", placement: "at the exhaust pipe, +y out of it", duration: 0.6, loop: true, els: [
  E("exhaust-puffs", puffs(C.smoke, 0, { rate: 18, duration: 0.6, loop: true, life: [0.4, 0.65], speed: [0.8, 1.2], size: [0.08, 0.13], shape: "cone 10 0.02", gravity: 0.6, drag: 3, inherit: 0.2, vary: C.grey }), "Puffs popping out of a kart's exhaust."),
] });
fx({ name: "boost", title: "Kart Boost", category: "exhaust", element: "fire", placement: "at the exhaust pipe, +y out of it", duration: 1, impact: 0, els: [
  E("boost-flames", flames(0, { along: true, rate: 90, duration: 0.9, col: [C.yellow, C.red], hot: 1.5, life: [0.12, 0.2], speed: [2.5, 3.5], size: [0.22, 0.3], shape: "cone 6 0.02", gravity: 0, drag: 3, inherit: 0.2 }), "A boost's flame out of the pipe."),
  E("boost-puffs", puffs(C.white, 6, { life: [0.3, 0.5], speed: [1.6, 2.4], size: [0.14, 0.2], shape: "cone 45 0.04", gravity: 0.3 }), "The pop of puffs as it kicks in."),
  E("boost-stars", stars(C.yellow, 6, { size: [0.05, 0.08], speed: [1.6, 2.6], life: [0.25, 0.4], shape: "cone 30 0.02", spread: 0, gravity: -2 }), "Stars at the kick."),
  LIGHT({ life: 0.9, col: C.orange, i: "0 2 1 0", d: 2.5, offset: [0, 0.2, 0] }),
] });
fx({ name: "drift-sparks", title: "Drift Sparks", category: "exhaust", element: "lightning", placement: "at the rear wheel on the ground, moved by the game", duration: 0.5, loop: true, els: [
  E("drift-sparks", stars(C.sky, 0, { rate: 40, duration: 0.5, loop: true, hot: 2, life: [0.18, 0.32], speed: [1.6, 2.8], size: [0.05, 0.08], shape: "cone 55 0.05", spread: 0, gravity: -6, drag: 1, vary: C.white }), "Blue sparks thrown off a drifting wheel: a mini-turbo charging."),
  E("drift-puffs", puffs(C.white, 0, { rate: 10, duration: 0.5, loop: true, life: [0.35, 0.5], speed: [0.4, 0.8], size: [0.1, 0.15], shape: "cone 60 0.05", gravity: 0.3 }), "Tyre puffs."),
] });
fx({ name: "tire-smoke", title: "Tyre Smoke", category: "smoke", placement: "at the wheel on the ground, moved by the game", duration: 0.5, loop: true, els: [
  E("tire-puffs", puffs(C.white, 0, { rate: 24, duration: 0.5, loop: true, life: [0.6, 0.9], speed: [0.5, 1], size: [0.16, 0.26], shape: "cone 60 0.08", gravity: 0.4, drag: 2, vary: C.smoke }), "Puffs of a spinning tyre."),
] });
fx({ name: "wheel-dust", title: "Wheel Dust", category: "smoke", element: "earth", placement: "behind the wheel on the ground, moved by the game", duration: 0.5, loop: true, els: [
  E("wheel-puffs", puffs(C.dust, 0, { rate: 20, duration: 0.5, loop: true, life: [0.5, 0.8], speed: [0.6, 1.2], size: [0.14, 0.22], shape: "cone 50 0.08", gravity: 0.3, drag: 2.5, vary: C.sand }), "Dust kicked up by a wheel."),
  E("wheel-grit", chips(C.dirt, 0, { kind: "bit", rate: 10, duration: 0.5, loop: true, size: [0.025, 0.04], speed: [1.2, 2], life: [0.35, 0.5] }), "Grit."),
] });
fx({ name: "afterburner", title: "Toy Jet Flame", category: "exhaust", element: "fire", placement: "at the nozzle, +y out of it", duration: 1, loop: true, els: [
  E("jet-flames", flames(0, { along: true, rate: 110, duration: 1, loop: true, col: [C.yellow, C.red], hot: 1.5, life: [0.14, 0.22], speed: [3, 3.8], size: [0.24, 0.32], shape: "cone 3 0.03", gravity: 0, drag: 4, grow: "0 1 1 0" }), "A toy jet's flame cone."),
  E("jet-puffs", puffs(C.white, 0, { rate: 12, duration: 1, loop: true, life: [0.4, 0.6], speed: [1.2, 1.8], size: [0.1, 0.16], shape: "cone 6 0.05", gravity: 0.2 }), "Puffs off its end.", { offset: [0, 0.5, 0] }),
  LIGHT({ life: 1, col: C.orange, i: "0 1.6 1 1.6", d: 2.5, offset: [0, 0.3, 0] }),
] });

// Destruction and building
fx({ name: "break-block", title: "Block Break", category: "destruction", element: "earth", placement: "centred on the block", duration: 1.6, impact: 0, els: [
  E("block-chunks", chips(C.orange, 8, { kind: "chip", size: [0.2, 0.24], speed: [2, 3.4], life: [0.9, 1.2], shape: "box 0.25 0.25 0.25", spread: 70, spin: 300, vary: C.yellow }), "A toy block bursting into chunks."),
  E("block-bits", chips(C.yellow, 10, { size: [0.08, 0.1], speed: [2.4, 4], shape: "box 0.25 0.25 0.25", spread: 80 }), "Smaller bits."),
  E("block-puffs", puffs(C.white, 6, { life: [0.35, 0.55], speed: [1, 1.6], size: [0.2, 0.3], spread: 180, gravity: 0.2 }), "A puff where it was."),
  SHAKE(0.15),
] });
fx({ name: "break-crate", title: "Crate Break", category: "destruction", element: "wood", placement: "centred on the crate", duration: 1.6, impact: 0, els: [
  E("crate-planks", chips(C.wood, 8, { kind: "chip", size: [0.12, 0.16], height: "0 0.3 1 0.3", speed: [2, 3.4], life: [0.9, 1.2], shape: "box 0.25 0.25 0.25", spread: 70, spin: 360, vary: C.woodDark }), "Planks of a broken toy crate."),
  E("crate-splinters", chips(C.wood, 12, { size: [0.05, 0.07], speed: [2.4, 4], shape: "box 0.25 0.25 0.25", spread: 80 }), "Splinters."),
  E("crate-puffs", puffs(C.dust, 6, { life: [0.35, 0.55], speed: [1, 1.6], size: [0.2, 0.3], spread: 180, gravity: 0.2 }), "A dusty puff."),
  SHAKE(0.15),
] });
fx({ name: "break-rock", title: "Rock Break", category: "destruction", element: "earth", placement: "centred on the rock", duration: 1.6, impact: 0, els: [
  E("rock-chunks", chips(C.rock, 9, { kind: "chip", size: [0.14, 0.2], speed: [2, 3.2], life: [0.9, 1.2], shape: "sphere 0.25", spread: 70, spin: 240, vary: C.rockLight }), "Rock lumps."),
  E("rock-bits", chips(C.rockLight, 12, { size: [0.05, 0.08], speed: [2.4, 4], shape: "sphere 0.25", spread: 80 }), "Gravel."),
  E("rock-puffs", puffs(C.grey, 8, { life: [0.5, 0.8], speed: [1, 1.8], size: [0.24, 0.36], spread: 180, gravity: 0.3, vary: C.smoke }), "A grey puff of stone dust."),
  SHAKE(0.2),
] });
fx({ name: "break-ice", title: "Ice Break", category: "destruction", element: "ice", placement: "centred on the ice", duration: 1.4, impact: 0, els: [
  E("ice-shards", chips(C.ice, 14, { kind: "gem", gloss: true, size: [0.08, 0.16], speed: [2, 3.6], life: [0.8, 1.1], shape: "sphere 0.25", spread: 80, spin: 360, vary: C.white }), "Glossy ice shards."),
  E("ice-glints", stars(C.white, 8, { size: [0.05, 0.08], hot: 1.4 }), "Glints."),
  SHAKE(0.12),
] });
fx({ name: "build-place", title: "Build Place", category: "destruction", element: "earth", placement: "on the ground under the placed piece", duration: 1, impact: 0, els: [
  E("place-puffs", puffs(C.white, 10, { life: [0.35, 0.55], speed: [1.6, 2.2], size: [0.14, 0.22], shape: "cone 88 0.35", gravity: 0, drag: 6 }), "Puffs squeezed out as a piece snaps down."),
  RING({ life: 0.25, scale: "0 0.4 1 0.9", col: C.white, inner: 0.88 }),
  E("place-stars", stars(C.yellow, 4, { size: [0.06, 0.09], speed: [1.6, 2.2], life: [0.3, 0.45], shape: "cone 60 0.3", spread: 0, gravity: -3 }), "A few stars."),
] });
fx({ name: "build-remove", title: "Build Remove", category: "destruction", element: "earth", placement: "centred on the removed piece", duration: 1, impact: 0, els: [
  E("remove-bits", chips(C.white, 12, { size: [0.06, 0.09], speed: [1, 2], life: [0.5, 0.8], shape: "box 0.25 0.25 0.25", spread: 180, gravity: -2, drag: 2 }), "Bits a removed piece falls apart into."),
  E("remove-puffs", puffs(C.white, 8, { life: [0.3, 0.5], speed: [1, 1.6], size: [0.16, 0.24], spread: 180, gravity: 0.2 }), "A puff."),
] });

// Celebration
const confetti = (name: string, title: string, at: number[], cone: string, n: number, speed: [number, number], placement: string, extra: El[] = []) => fx({
  name, title, category: "ui", placement, duration: 2.6, impact: 0, els: [
    ...RAINBOW.map((c, i) => E(`${name}-${i}`, confettiOf(c, n, { shape: cone, speed }), `${title}: one colour of the confetti.`, { offset: at })),
    ...extra,
  ],
});
confetti("confetti", "Confetti Pop", [0, 0.1, 0], "cone 30 0.1", 12, [4, 6.5], "on the ground, +y up", [RING({ life: 0.25, scale: "0 0.1 1 0.6", col: C.white, inner: 0.84 })]);
fx({ name: "confetti-rain", title: "Confetti Rain", category: "ui", placement: "3 m above the area it covers", duration: 2, loop: true, els: RAINBOW.map((c, i) =>
  E(`confetti-rain-${i}`, confettiOf(c, 0, { rate: 14, duration: 2, loop: true, life: [2.6, 3.2], speed: [0, 0.3], size: [0.15, 0.2], shape: "box 2 0.05 2", gravity: -3, drag: 1.5, collide: "0.1 0.9 depth" }), "Confetti fluttering down over an area.", { offset: [0, 3, 0] })) });
fx({ name: "fireworks", title: "Fireworks", category: "ui", element: "fire", placement: "on the ground: it bursts 6 m up", duration: 3, impact: 0.7, els: [
  E("fw-rise", stars(C.lemon, 0, { rate: 40, duration: 0.7, hot: 2, life: [0.2, 0.3], speed: [0, 0.3], size: [0.05, 0.07], shape: "point", gravity: -1 }), "The rocket's spark trail.", { move: [0, 8.6, 0], until: 0.7 }),
  ...[C.red, C.yellow, C.blue].map((c, i) => E(`fw-burst-${i}`, stars(c, 30, { hot: 1.8, size: [0.12, 0.18], speed: [4.5, 5.5], life: [1.1, 1.5], shape: "sphere 0.1 surface", spread: 0, gravity: -1.5, drag: 1.6, trail: "0.18 0.4" }), "A burst of coloured stars.", { at: 0.7 + i * 0.12, offset: [i * 0.8 - 0.8, 6 + (i % 2) * 0.5, 0] })),
  LIGHT({ at: 0.7, life: 0.8, col: C.yellow, i: "0 6 1 0", d: 10, offset: [0, 6, 0] }),
] });
fx({ name: "twinkle", title: "Twinkle", category: "ambient", element: "light", placement: "on what sparkles", duration: 1.6, loop: true, els: [
  E("twinkle-stars", stars(C.white, 0, { rate: 7, duration: 1.6, loop: true, hot: 1.6, life: [0.5, 0.7], speed: [0, 0], size: [0.12, 0.18], shape: "sphere 0.3", gravity: 0, drag: 0, spin: 90, grow: "0 0 0.4 1 1 0", vary: C.lemon }), "Stars blinking on something shiny."),
] });

// Ambience
fx({ name: "leaves", title: "Falling Leaves", category: "ambient", element: "nature", placement: "3 m above the area it covers", duration: 3, loop: true, els: [
  E("leaves-fall", bits(C.leaf, 0, { kind: "ribbon", rate: 10, duration: 3, loop: true, life: [3.6, 4.4], speed: [0, 0.2], size: [0.15, 0.2], shape: "box 2 0.05 2", spread: 0, gravity: -1.6, drag: 1.2, spin: 140, noise: "0.8 0.8 0.3", grow: "0 0 0.08 1 0.92 1 1 0", vary: C.autumn, collide: "0 0.9 depth" }), "Leaves fluttering down.", { offset: [0, 3, 0] }),
] });
fx({ name: "fireflies", title: "Fireflies", category: "ambient", element: "nature", placement: "over the area, 0.2 to 1.5 m up", duration: 3, loop: true, els: [
  E("fireflies-drift", bits(C.lime, 0, { rate: 10, duration: 3, loop: true, hot: 2.4, life: [2.4, 3.2], speed: [0.05, 0.15], size: [0.06, 0.09], shape: "box 1.5 0.6 1.5", gravity: 0, drag: 0.5, noise: "0.4 0.6 0.2", grow: "0 0 0.08 1 0.5 0.5 0.7 1 1 0", vary: C.lemon }), "Glowing bugs drifting about.", { offset: [0, 0.85, 0] }),
] });

// ---------- the models: tiny glTF meshes the effects draw ----------

interface Mesh { pos: number[]; nrm: number[]; idx: number[] }

function addTri(m: Mesh, a: number[], b: number[], c: number[]) {
  const u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]], w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
  let n = [u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]];
  const l = Math.hypot(...n) || 1;
  n = n.map((x) => x / l);
  const i = m.pos.length / 3;
  m.pos.push(...a, ...b, ...c);
  m.nrm.push(...n, ...n, ...n);
  m.idx.push(i, i + 1, i + 2);
}

/** A closed outline (x, y) extruded `depth` along z, its faces bulged by `bulge` at the centre. */
function extrude(outline: [number, number][], depth: number, bulge: number, centre: [number, number] = [0, 0]): Mesh {
  const m: Mesh = { pos: [], nrm: [], idx: [] };
  const h = depth / 2, n = outline.length;
  const front = [centre[0], centre[1], h + bulge], back = [centre[0], centre[1], -h - bulge];
  for (let i = 0; i < n; i++) {
    const [x0, y0] = outline[i], [x1, y1] = outline[(i + 1) % n];
    addTri(m, front, [x0, y0, h], [x1, y1, h]);
    addTri(m, back, [x1, y1, -h], [x0, y0, -h]);
    addTri(m, [x0, y0, h], [x0, y0, -h], [x1, y1, -h]);
    addTri(m, [x0, y0, h], [x1, y1, -h], [x1, y1, h]);
  }
  return m;
}

/** A closed surface from a (u, v) grid, u around and v pole to pole, smooth normals. */
function surface(fn: (u: number, v: number) => number[], nu: number, nv: number): Mesh {
  const P: number[][][] = [];
  for (let j = 0; j <= nv; j++) {
    P.push([]);
    for (let i = 0; i < nu; i++) P[j].push(fn((i / nu) * Math.PI * 2, (j / nv) * Math.PI));
  }
  // Smooth normals: the sum of the faces' about each grid point.
  const N = P.map((row) => row.map(() => [0, 0, 0]));
  const acc = (j: number, i: number, n: number[]) => { for (let k = 0; k < 3; k++) N[j][i % nu][k] += n[k]; };
  for (let j = 0; j < nv; j++) for (let i = 0; i < nu; i++) {
    const a = P[j][i], b = P[j][(i + 1) % nu], c = P[j + 1][(i + 1) % nu], d = P[j + 1][i];
    const u = [c[0] - a[0], c[1] - a[1], c[2] - a[2]], w = [d[0] - b[0], d[1] - b[1], d[2] - b[2]];
    const n = [u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]];
    for (const [jj, ii] of [[j, i], [j, i + 1], [j + 1, i + 1], [j + 1, i]]) acc(jj, ii, n);
  }
  // The poles: every point of a pole row shares one normal.
  for (const j of [0, nv]) {
    const s = N[j].reduce((t, n) => t.map((x, k) => x + n[k]), [0, 0, 0]);
    N[j] = N[j].map(() => s.slice());
  }
  const m: Mesh = { pos: [], nrm: [], idx: [] };
  for (let j = 0; j <= nv; j++) for (let i = 0; i < nu; i++) {
    const n = N[j][i], l = Math.hypot(...n) || 1;
    m.pos.push(...P[j][i]);
    m.nrm.push(n[0] / l, n[1] / l, n[2] / l);
  }
  const at = (j: number, i: number) => j * nu + (i % nu);
  for (let j = 0; j < nv; j++) for (let i = 0; i < nu; i++) {
    const a = at(j, i), b = at(j, i + 1), c = at(j + 1, i + 1), d = at(j + 1, i);
    // Counter-clockwise seen from outside.
    m.idx.push(a, c, d, a, b, c);
  }
  return m;
}

/** Scaled and centred to fit a unit cube. */
function fit(m: Mesh): Mesh {
  const lo = [Infinity, Infinity, Infinity], hi = [-Infinity, -Infinity, -Infinity];
  for (let i = 0; i < m.pos.length; i += 3) for (let k = 0; k < 3; k++) {
    lo[k] = Math.min(lo[k], m.pos[i + k]);
    hi[k] = Math.max(hi[k], m.pos[i + k]);
  }
  const s = 1 / Math.max(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]);
  const c = lo.map((l, k) => (l + hi[k]) / 2);
  return { ...m, pos: m.pos.map((p, i) => (p - c[i % 3]) * s) };
}

// Points on the unit sphere, y up: u around, v from the top.
const sph = (u: number, v: number) => [Math.sin(v) * Math.cos(u), Math.cos(v), Math.sin(v) * Math.sin(u)];
const starOutline = (points: number, inner: number): [number, number][] =>
  Array.from({ length: points * 2 }, (_, i) => {
    const r = i % 2 ? inner : 0.5, a = Math.PI / 2 + (i * Math.PI) / points;
    return [r * Math.cos(a), r * Math.sin(a)];
  });
const heartOutline = (): [number, number][] =>
  Array.from({ length: 40 }, (_, i) => {
    const t = (i / 40) * Math.PI * 2;
    return [-16 * Math.pow(Math.sin(t), 3) / 34, (13 * Math.cos(t) - 5 * Math.cos(2 * t) - 2 * Math.cos(3 * t) - Math.cos(4 * t)) / 34];
  });
const circleOutline = (n: number, r = 0.5, wob = 0, k = 0): [number, number][] =>
  Array.from({ length: n }, (_, i) => {
    const a = (i / n) * Math.PI * 2, rr = r * (1 + wob * Math.abs(Math.sin(k * a / 2)));
    return [rr * Math.cos(a), rr * Math.sin(a)];
  });
/** A teardrop, its point up: the side profile r(v). */
const dropR = (v: number, sharp: number) => Math.pow(Math.sin(v / 2), sharp) * Math.sin(v);
const gemOutline = (): [number, number][] => [[0, 0.5], [-0.32, 0.2], [-0.42, 0.05], [0, -0.5], [0.42, 0.05], [0.32, 0.2]];

const MODELS = {
  puff: () => fit(surface(sph, 20, 12)),
  block: () => fit(surface((u, v) => {
    // A superellipsoid: a cube with its edges rounded, as a moulded toy block.
    const e = 0.3, sg = (x: number) => Math.sign(x) * Math.pow(Math.abs(x), e);
    return [sg(Math.sin(v)) * sg(Math.cos(u)), sg(Math.cos(v)), sg(Math.sin(v)) * sg(Math.sin(u))];
  }, 24, 16)),
  star: () => fit(extrude(starOutline(5, 0.22), 0.16, 0.1)),
  coin: () => fit(extrude(circleOutline(24), 0.12, 0.02)),
  heart: () => fit(extrude(heartOutline(), 0.18, 0.1, [0, -0.04])),
  drop: () => fit(surface((u, v) => { const r = dropR(v, 1.2); return [r * Math.cos(u), -Math.cos(v) * 1.15, r * Math.sin(u)].map((x, k) => (k === 1 ? -x : x)); }, 16, 12)),
  flame: () => fit(surface((u, v) => { const r = dropR(v, 2) * (1 + 0.08 * Math.sin(3 * u)); return [r * Math.cos(u) * 0.8, Math.cos(v) * 1.5, r * Math.sin(u) * 0.8]; }, 14, 12)),
  ribbon: () => fit(surface((u, v) => {
    // A confetti strip, curled: a flat rounded box bent along its length.
    const p = [Math.sin(v) * Math.cos(u), Math.cos(v), Math.sin(v) * Math.sin(u)];
    const x = p[0] * 0.5, y = p[1] * 0.18, z = p[2] * 0.03, a = x * 1.6;
    return [Math.sin(a) / 1.6, y, z + (1 - Math.cos(a)) / 1.6];
  }, 16, 8)),
  gem: () => fit(extrude(gemOutline(), 0.3, 0.12, [0, 0.05])),
};
type ModelName = keyof typeof MODELS;

/** A glTF 2.0 binary of one mesh: positions, normals, 16-bit indices, no material. */
function glb(m: Mesh): Buffer {
  const nv = m.pos.length / 3;
  if (nv > 65535) throw new Error("too many vertices");
  const pos = new Float32Array(m.pos), nrm = new Float32Array(m.nrm), idx = new Uint16Array(m.idx);
  const pad4 = (n: number) => (n + 3) & ~3;
  const bin = Buffer.alloc(pad4(pos.byteLength) + pad4(nrm.byteLength) + pad4(idx.byteLength));
  Buffer.from(pos.buffer).copy(bin, 0);
  Buffer.from(nrm.buffer).copy(bin, pad4(pos.byteLength));
  Buffer.from(idx.buffer).copy(bin, pad4(pos.byteLength) + pad4(nrm.byteLength));
  const lo = [0, 1, 2].map((k) => Math.min(...m.pos.filter((_, i) => i % 3 === k)));
  const hi = [0, 1, 2].map((k) => Math.max(...m.pos.filter((_, i) => i % 3 === k)));
  const json = {
    asset: { version: "2.0", generator: "origin-engine scripts/vfx-toy.ts" },
    scene: 0, scenes: [{ nodes: [0] }], nodes: [{ mesh: 0 }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0, NORMAL: 1 }, indices: 2 }] }],
    buffers: [{ byteLength: bin.length }],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: pos.byteLength, target: 34962 },
      { buffer: 0, byteOffset: pad4(pos.byteLength), byteLength: nrm.byteLength, target: 34962 },
      { buffer: 0, byteOffset: pad4(pos.byteLength) + pad4(nrm.byteLength), byteLength: idx.byteLength, target: 34963 },
    ],
    accessors: [
      { bufferView: 0, componentType: 5126, count: nv, type: "VEC3", min: lo, max: hi },
      { bufferView: 1, componentType: 5126, count: nv, type: "VEC3" },
      { bufferView: 2, componentType: 5123, count: idx.length, type: "SCALAR" },
    ],
  };
  let j = Buffer.from(JSON.stringify(json));
  j = Buffer.concat([j, Buffer.alloc(pad4(j.length) - j.length, 0x20)]);
  const head = Buffer.alloc(12);
  head.writeUInt32LE(0x46546c67, 0);
  head.writeUInt32LE(2, 4);
  head.writeUInt32LE(12 + 8 + j.length + 8 + bin.length, 8);
  const chunk = (b: Buffer, type: number) => { const h = Buffer.alloc(8); h.writeUInt32LE(b.length, 0); h.writeUInt32LE(type, 4); return Buffer.concat([h, b]); };
  return Buffer.concat([head, chunk(j, 0x4e4f534a), chunk(bin, 0x004e4942)]);
}

// ---------- the packs ----------

function variant(pack: string, o: Omit<Variant, "pack" | "mesh" | "models"> & { use: Record<Kind, ModelName | "box" | "sphere"> }): Variant {
  const used = new Set(Object.values(o.use).filter((x): x is ModelName => x !== "box" && x !== "sphere"));
  return {
    ...o,
    pack,
    mesh: (k) => o.use[k],
    models: Object.fromEntries([...used].map((n) => [n, MODELS[n]])),
  };
}

const VARIANTS: Variant[] = [
  variant("low-poly", {
    title: "Low-poly",
    looks: ["low-poly"],
    description: "Chunky low-poly effects: satin puffs, bevelled blocks, stars, coins and hearts in bright clean colour; smoke and dust pop up and shrink away instead of fading.",
    use: { puff: "puff", chip: "block", bit: "puff", drop: "drop", star: "star", coin: "coin", heart: "heart", flame: "flame", ribbon: "ribbon", gem: "gem" },
    thin: () => undefined,
    roughness: 0.38, gloss: 0.12, glow: 1, segments: 48, translucency: 0,
    paint: (c) => c,
  }),
];

// The effect materials every pack ships.
const FADE_OUT = { kind: "curve", keys: [[0, 1], [0.5, 0.9], [1, 0]] };
const MATERIALS: Record<string, [string, J]> = {
  solid: ["Solid unlit colour: the pack's shapes supply the form.", { blend: "alpha", doubleSided: true }],
  fade: ["Solid colour fading out over the last half of its life: rings and ripples.", { blend: "alpha", doubleSided: true, alpha: FADE_OUT }],
  bubble: ["A glossy clear bubble: brighter at its rim.", { blend: "alpha", doubleSided: false, fresnel: { power: 2.5, strength: 1.6 } }],
};

function write(v: Variant): { effects: number; files: number } {
  const d = join(OUT, v.pack);
  // The geometric/ sources are authored separately; preserve them.
  for (const sub of ["content/emitters", "content/models", "models"]) rmSync(join(d, sub), { recursive: true, force: true });
  mkdirSync(join(d, "content", "emitters"), { recursive: true });
  mkdirSync(join(d, "content", "models"), { recursive: true });
  mkdirSync(join(d, "models"), { recursive: true });
  let files = 0;
  const put = (stem: string, text: string) => { writeFileSync(join(d, "content", `${stem}.ts`), text); files++; };
  const seen = new Set<string>();
  const emit = (name: string, p: Piece, note: string) => {
    if (seen.has(name)) throw new Error(`emitter ${name} written twice`);
    seen.add(name);
    put(`emitters/${name}`, declaration("particles", note, emitter(v, p)));
    return id(v.pack, `emitters/${name}`);
  };
  for (const [name, [note, value]] of Object.entries(MATERIALS)) put(name, declaration("effect", note, value));
  for (const [name, make] of Object.entries(v.models)) {
    writeFileSync(join(d, "models", `${name}.glb`), glb(make()));
    files++;
    // The raw model cooks only through its import declaration: an effect's mesh, no face budget.
    put(`models/${name}`, `import { file } from '@origingame/engine/author';\n\n// MIT Origin Engine (scripts/vfx-toy.ts). The ${name} mesh effects of the pack draw.\nexport default file.import(${JSON.stringify({ kind: "model", source: `assets/vfx/${v.pack}/models/${name}.glb`, conform: "effect" }, null, 2)});\n`);
  }
  emit("drip-plip", drops(0, { speed: [0.8, 1.4], size: [0.025, 0.04], shape: "cone 55 0.01", life: [0.2, 0.35], collide: undefined }), "The little crown of drops where a drip lands.");
  const cats = new Set<string>(), elems = new Set<string>();
  const rows: string[] = [];
  for (const e of EFFECTS) {
    const value = obj({
      duration: e.duration,
      looping: e.loop || undefined,
      events: e.impact !== undefined ? [{ name: "impact", time: e.impact }] : undefined,
      elements: e.els.map((el) => el(v, emit)),
    });
    put(e.name, declaration("vfx", `${e.title}: ${e.placement}.`, value));
    cats.add(e.category);
    if (e.element) elems.add(e.element);
    rows.push(`| ${e.name} | ${e.loop ? "loops" : "once"} | ${e.placement} |`);
  }
  const ext = {
    format: 1,
    kind: "vfx",
    name: v.pack,
    version: "1.0.0",
    description: v.description,
    minEngine: "0.9.0",
    tags: { style: "stylised", dim: "3d", category: [...cats].sort(), element: [...elems].sort() },
    keywords: keywords(v),
  };
  writeFileSync(join(d, "extension.json"), JSON.stringify(ext, null, 2) + "\n");
  writeFileSync(join(d, "README.md"), `# ${v.title} 3D effects

Install with \`og vfx add ${v.pack}\`; spawn \`base/vfx/${v.pack}/<name>\`
with og.vfx.spawn. ${v.description}

Every effect is solid matter, the way a toy world is built: puffs, pieces,
drops and flame shapes drawn as small lit mesh particles (\`file.particles\`
with a mesh \`render\`), with ring meshes and point lights in each
\`file.vfx\` (\`content/<name>.ts\`). Sizes are in metres for a figure about
1 m tall. The geometric/ directory adds nine texture-free mechanisms.
Effects that play once send their
impact at the time in \`assets.json\`. No gameplay, scoring, damage or UI
logic, and no sound.

| Effect | Plays | Placement |
|---|---|---|
${rows.join("\n")}

All files are original MIT work, Origin Engine, written by
\`scripts/vfx-toy.ts\`: the effects and emitters from one table of recipes,
the models (\`models/*.glb\`) from its own geometry (extrusions of star,
heart, coin, gem and drop outlines; lathed drops and flames; a
superellipsoid block; flat card cutouts), with no external images, models or
code.
`);
  files += 2;
  return { effects: EFFECTS.length, files };
}

const keywords = (v: Variant) => [...new Set(["3d", "toy", "craft", ...v.looks])].sort();

/** The `vfx-pack.ts manifest --spec` input for a pack. */
function manifestSpec(v: Variant) {
  const geometric = [
    ["celebration", "Coral Confetti", "ui"],
    ["dust", "Slate Dust", "smoke"],
    ["explosion", "Facet Burst", "explosion"],
    ["impact", "Angular Impact", "hit-spark"],
    ["magic", "Open Triangles", "magic-impact"],
    ["muzzle", "Geometric Pulse", "gunfire"],
    ["pickup", "Rising Cubes", "buff"],
    ["splash", "Facet Splash", "splash"],
    ["trail", "Facet Trail", "ambient"],
  ];
  return {
    minEngine: "0.9.0",
    style: "stylised",
    dim: "3d",
    keywords: keywords(v),
    files: [{
      glob: "content/geometric/*",
      licence: "MIT",
      source: { url: "https://github.com/originai-game/origin-engine" },
      credit: "Origin Engine",
      derived: "Original texture-free native mesh-particle and polygon-ring composition, represented as checked TypeScript FX data. No external models, textures or code.",
    }, {
      glob: "*",
      licence: "MIT",
      source: { url: "https://github.com/originai-game/origin-engine" },
      credit: "Origin Engine",
      derived: "Original effects, emitters and models written by scripts/vfx-toy.ts; no external images, models or code.",
    }],
    effects: Object.fromEntries([
      ...EFFECTS.map((e) =>
        [`content/${e.name}.ts`, { title: e.title, category: e.category, ...(e.element ? { element: e.element } : {}), keywords: ["toy", e.category, ...e.name.split("-")].filter((x, i, a) => a.indexOf(x) === i) }]),
      ...geometric.map(([name, title, category]) =>
        [`content/geometric/${name}.ts`, { title, category, keywords: ["geometric", name], ...(name === "dust" ? { element: "earth" } : name === "splash" ? { element: "water" } : {}) }]),
    ]),
  };
}

const args = process.argv.slice(2).filter((a, i) => !(i === 0 && !a.startsWith("--")));
const only = args.filter((a) => !a.startsWith("--spec="));
const specs = args.find((a) => a.startsWith("--spec="))?.slice(7);
for (const v of VARIANTS) {
  if (only.length && !only.includes(v.pack)) continue;
  console.log(v.pack, write(v));
  if (specs) {
    mkdirSync(specs, { recursive: true });
    writeFileSync(join(specs, `${v.pack}.json`), JSON.stringify(manifestSpec(v), null, 1) + "\n");
  }
}
