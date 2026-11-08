// A farm at dusk (README.md): a tilemap of grass, paths and tilled plots
// (tiles/farm-map.ogt), crops, fences, trees and props from loose PNGs the
// cook packs into one texture (props/props.ogt), a farmer walking down the
// path (a flipbook from a GIF) and warm lamps whose light the fences and the
// trunks shadow (og.Light2D, og.Occluder2D).
//
//   og run tests/fixtures/top-down --size 960x540 --commands "0.3:walk;2.5:quit"
//
// Measurement commands (README.md §Numbers): `crowd:N` adds N crop sprites
// in view, `lamps:N` swaps the lamps for N shadowed lights in view and
// `penumbra:R` shows one light of sourceRadius R behind one wall, far from
// the farm.
import { og, defineResource, defineSystem, u32, f32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { MAP, PROPS } from "./art.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A tile is a world unit and 16 texels; a texel three screen pixels at 960x540. */
const TEX = 1 / 16;
/** The farmer's start on the path and how fast she walks (tiles a second). */
const START = { x: 23, y: 22.5 };
const WALK = 1.9;
/** The tiles of tiles/farm.ogt the fixture reads: watered soil and the path. */
const WATERED = 7;
const PATH = [8, 9, 10, 11];

const Play = defineResource("Play", { walking: u32(), steps: u32(), tilled: u32(), crowd: u32(), lights: u32(), seed: f32() });

/** World y of a map row's centre, and the row of a world y (the map's top-left is at (0, MAP.h)). */
const rowY = (r: number) => MAP.h - r - 0.5;
const rowOf = (y: number) => Math.floor(MAP.h - y);

/**
 * Where a vfx2d effect goes once `type vfx2d` lands (og.vfx.spawn of a
 * cooked `fx/dust.ogt`): a dust puff at the farmer's feet on each step.
 * Until then it marks the spot with nothing drawn.
 */
function vfx2d(_world: World, _name: "dust", _at: { x: number; y: number }): void {
  // og.vfx.spawn(world, { asset: must(og.load.vfx(world, `fx/${name}.ogt`)), position: { ...at, z: 0 } });
}

type Opts = { shadow?: number; castShadow?: boolean; flipX?: boolean; glow?: number };

/** A prop of the packed sheet standing on its feet at (x, y), sorted by y. */
function prop(world: World, name: keyof typeof PROPS, x: number, y: number, opts: Opts = {}): void {
  // The same handles every call (og.load answers a path once loaded).
  const s = must(og.load.flipbook(world, "props/props.ogt"));
  const e = must(world.spawn({ position: { x, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { width: PROPS[name].w * TEX, height: PROPS[name].h * TEX, sortY: true, ...opts }));
  must(world.insert(e, og.Flipbook, { sheet: s.sheet, clip: og.anim.clip(s, name), playing: false }));
}

function tree(world: World, kind: "tree" | "pine", x: number, y: number): void {
  prop(world, kind, x, y, { shadow: 0.35 });
  const trunk = must(world.spawn({ position: { x, y: y + 0.2, z: 0 } }));
  must(world.insert(trunk, og.Occluder2D, { width: 0.45, height: 0.35 }));
}

function lamp(world: World, x: number, y: number): void {
  prop(world, "lamp", x, y, { shadow: 0.3, glow: 0.6 });
  const light = must(world.spawn({ name: `lamp${x},${y}`, position: { x, y: y + 1.7, z: 0 } }));
  must(world.insert(light, og.Light2D, { color: { r: 1, g: 0.7, b: 0.38 }, intensity: 1.4, radius: 4, falloff: 1.8, sourceRadius: 0.25, castShadows: true }));
}

const CROPS = [["parsnip_2", "cauliflower", "parsnip_1", "pumpkin"], ["wheat_a", "wheat_b", "sunflower", "tomato"], ["parsnip_0", "parsnip_1", "cauliflower", "tomato"]] as const;

function build(world: World): void {
  const map = must(og.load.tilemap(world, "tiles/farm-map.ogt"));
  const ground = must(world.spawn({ name: "farm", position: { x: 0, y: MAP.h, z: 0 } }));
  must(world.insert(ground, og.Tilemap, { map: map.map, layer: -10, collision: false }));

  // The plots: crops on their cells, the first columns watered and empty, a fence round them.
  MAP.plots.forEach(([px, py, pw, ph], i) => {
    for (let r = 0; r < ph; r++) {
      for (let c = 3; c < pw; c++) {
        const kind = CROPS[i]![(r * 7 + c * 3) % 4]!;
        prop(world, kind, px + c + 0.5, rowY(py + r) - 0.35);
      }
    }
    const [x0, x1, top, bottom] = [px - 0.5, px + pw + 0.5, rowY(py) + 0.9, rowY(py + ph - 1) - 0.8];
    for (let x = x0 + 0.5; x < x1; x += 1) {
      prop(world, "fence_h", x, top, { castShadow: true });
      if (Math.abs(x - (px + 1.5)) > 0.6) prop(world, "fence_h", x, bottom, { castShadow: true });
    }
    for (let y = bottom + 0.5; y < top; y += 0.5) {
      prop(world, "fence_post", x0, y, { castShadow: true });
      prop(world, "fence_post", x1, y, { castShadow: true });
    }
  });
  prop(world, "scarecrow", 33, rowY(8), { shadow: 0.3 });

  // Trees round the farm, a well, the yard and wild flowers on the grass.
  for (const [k, x, y] of [["tree", 4, 26], ["pine", 2, 21], ["tree", 19.5, 29], ["tree", 28, 28.5], ["pine", 42, 27], ["tree", 45, 20], ["pine", 44, 13], ["tree", 4, 13], ["pine", 6, 6], ["tree", 30, 9.5], ["pine", 40, 3], ["tree", 19.6, 15.2]] as const) tree(world, k, x, y);
  prop(world, "well", 25.4, 20.6, { shadow: 0.35 });
  prop(world, "mailbox", 24.8, 24.6, { shadow: 0.25 });
  prop(world, "sign", 21.2, 14.2, { shadow: 0.25 });
  for (const [k, x, y] of [["crate", 26.2, 15.6], ["barrel", 27.3, 15.8], ["chest", 28.4, 15.5], ["hay", 29.6, 15.9], ["crate", 26.5, 14.9]] as const) prop(world, k, x, y, { shadow: 0.3, castShadow: true });
  const rnd = (s: number) => { const v = Math.sin(s * 127.1) * 43758.5453; return v - Math.floor(v); };
  const wild = ["flowers_a", "flowers_b", "flowers_c", "bush_a", "bush_b", "rock", "stump", "flowers_a"] as const;
  for (let i = 0; i < 70; i++) {
    const x = 1 + rnd(i) * (MAP.w - 2);
    const y = 1 + rnd(i + 100) * (MAP.h - 2);
    const [c, r] = [Math.floor(x), rowOf(y)];
    const busy = MAP.pathX.includes(c as 22) || MAP.pathY.includes(r as 17) || MAP.plots.some(([px, py, pw, ph]) => c >= px - 1 && c <= px + pw && r >= py - 1 && r <= py + ph);
    if (!busy) prop(world, wild[i % wild.length]!, x, y, { flipX: i % 3 === 0 });
  }
  for (const [x, y] of [[21.2, 23.4], [24.8, 17.6], [20.8, 11.4], [12.5, 16.4], [33.5, 16.4]] as const) lamp(world, x, y);

  // The farmer, walking down the path towards the camera.
  const farmerSheet = must(og.load.flipbook(world, "farmer/farmer.ogt"));
  const farmer = must(world.spawn({ name: "farmer", position: { x: START.x, y: START.y, z: 0 } }));
  must(world.insert(farmer, og.Sprite, { width: farmerSheet.frameWidth * TEX, height: farmerSheet.frameHeight * TEX, sortY: true, shadow: 0.4 }));
  must(world.insert(farmer, og.Flipbook, { sheet: farmerSheet.sheet, clip: og.anim.clip(farmerSheet, "walk"), playing: false }));
  must(world.insert(farmer, og.Follow2D, { smoothTime: 0.3, offset: { x: 0, y: 0.6 }, clamp: true, boundsMin: { x: 10, y: 5.625 }, boundsMax: { x: MAP.w - 10, y: MAP.h - 5.625 } }));

  // Dusk: a violet ambient, warm lamps; a warm grade and a vignette.
  const l = must(world.resource(og.Lighting2D));
  l.enabled = true;
  l.ambient = { r: 0.24, g: 0.2, b: 0.36 };
  const post = must(world.resource(og.PostEffects));
  post.vignette = true;
  post.vignetteIntensity = 0.4;
  post.temperature = 0.15;
  post.glow = true;
  const cam = must(world.resource(og.Camera));
  cam.orthographic = true;
  cam.height = 540 / 48;
  cam.position = { x: START.x, y: START.y + 0.6, z: 10 };
  cam.background = { r: 0.05, g: 0.06, b: 0.04 };
}

const Walk = defineSystem("Walk", { reads: [og.FlipbookEvent], writes: [Play, og.Flipbook, og.Transform] }, (_q, time, world) => {
  const play = must(world.resource(Play));
  const farmer = world.find("farmer");
  if (farmer === null) return;
  const t = must(world.get(farmer, og.Transform));
  must(world.set(farmer, og.Flipbook, { playing: play.walking === 1 }));
  if (play.walking === 1) must(world.set(farmer, og.Transform, { position: { x: t.position.x, y: t.position.y - WALK * time.dt, z: 0 } }));
  for (const e of world.events(og.FlipbookEvent)) {
    if (e.entity !== farmer || e.name !== "step") continue;
    play.steps += 1;
    og.assert.reachable("the farmer steps");
    vfx2d(world, "dust", { x: t.position.x, y: t.position.y });
  }
});

/** The tile under the farmer's feet (read between ticks: og.tilemap reads the world). */
function underFoot(world: World): number {
  const farm = world.find("farm");
  const farmer = world.find("farmer");
  if (farm === null || farmer === null) return -1;
  const p = must(world.get(farmer, og.Transform)).position;
  return og.tilemap.getTile(world, farm, "ground", Math.floor(p.x), rowOf(p.y));
}

/** Spread `n` things over the camera's view with a seeded walk. */
function inView(world: World, n: number, each: (x: number, y: number, i: number) => void): void {
  const cam = must(world.resource(og.Camera));
  for (let i = 0; i < n; i++) {
    const fx = (i * 0.6180339887) % 1;
    const fy = (i * 0.7548776662) % 1;
    each(cam.position.x - 9.5 + fx * 19, cam.position.y - 5.2 + fy * 10.4, i);
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Play],
  systems: [...og.systems.cameras, Walk],
  start: build,
  commands: {
    walk: (world) => { must(world.resource(Play)).walking = 1; },
    stop: (world) => { must(world.resource(Play)).walking = 0; },
    till: (world, args) => {
      const farm = world.find("farm");
      if (farm === null) return;
      const [x, y] = [Number(args[0] ?? 12), Number(args[1] ?? 9)];
      must(og.tilemap.setTile(world, { entity: farm, layer: "ground", x, y, tile: WATERED }));
      must(world.resource(Play)).tilled += 1;
    },
    crowd: (world, args) => {
      const n = Number(args[0] ?? 1000);
      const kinds = ["parsnip_2", "cauliflower", "pumpkin", "wheat_a", "tomato", "sunflower", "flowers_a", "bush_a"] as const;
      inView(world, n, (x, y, i) => prop(world, kinds[i % kinds.length]!, x, y));
      must(world.resource(Play)).crowd += n;
    },
    lamps: (world, args) => {
      const n = Number(args[0] ?? 0);
      const lit = must(world.query({ with: [og.Light2D] })).archetypes().flatMap((a) => Array.from({ length: a.count }, (_, i) => a.entity(i)));
      for (const e of lit) must(world.despawn(e));
      inView(world, n, (x, y) => {
        const e = must(world.spawn({ position: { x, y, z: 0 } }));
        must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.7, b: 0.38 }, intensity: 1.2, radius: 5, falloff: 1.4, sourceRadius: 0.25, castShadows: true }));
      });
      must(world.resource(Play)).lights = n;
    },
    penumbra: (world, args) => {
      // A lit white floor far from the farm, one light and one wall: the
      // wall's shadow edge 6 units behind it is measured on the frame.
      const at = { x: 200, y: 200 };
      const floor = must(world.spawn({ position: { x: at.x, y: at.y, z: 0 } }));
      must(world.insert(floor, og.Sprite, { width: 24, height: 14, layer: -5 }));
      const light = must(world.spawn({ name: "probe-light", position: { x: at.x - 6, y: at.y, z: 0 } }));
      must(world.insert(light, og.Light2D, { intensity: 1, radius: 30, falloff: 0.1, sourceRadius: Number(args[0] ?? 0.25), castShadows: true }));
      const wall = must(world.spawn({ position: { x: at.x - 3, y: at.y + 1.5, z: 0 } }));
      must(world.insert(wall, og.Occluder2D, { width: 0.3, height: 3 }));
      const farmer = world.find("farmer");
      if (farmer !== null) must(world.remove(farmer, og.Follow2D));
      must(world.resource(og.Lighting2D)).ambient = { r: 0, g: 0, b: 0 };
      const post = must(world.resource(og.PostEffects));
      post.vignette = false;
      post.temperature = 0;
      must(world.resource(og.Camera)).position = { x: at.x, y: at.y, z: 10 };
    },
  },
  commandHelp: {
    walk: { usage: "", doc: "The farmer walks down the path." },
    stop: { usage: "", doc: "The farmer stops." },
    till: { usage: "[x y]", doc: "Water the soil at map cell (x, y)." },
    crowd: { usage: "N", doc: "Add N crop sprites in view (the sprite-count measurement)." },
    lamps: { usage: "N", doc: "Replace every light with N shadowed lights in view (the light-pass measurement)." },
    penumbra: { usage: "R", doc: "Show one light of sourceRadius R behind one wall, far from the farm (the penumbra measurement)." },
  },
  facts: (world) => {
    const play = world.resource(Play);
    const farmer = world.find("farmer");
    const farm = world.find("farm");
    const y = farmer === null ? 0 : must(world.get(farmer, og.Transform)).position.y;
    const tile = underFoot(world);
    return {
      steps: play.ok ? play.value.steps : 0,
      walked: Math.round((START.y - y) * 100) / 100,
      onPath: PATH.includes(tile) ? 1 : 0,
      watered: farm === null ? -1 : og.tilemap.getTile(world, farm, "ground", 12, 9),
    };
  },
});
