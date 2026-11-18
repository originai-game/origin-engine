// The set: a temple's ruined forecourt on a clear morning. A cobbled way
// runs east past fallen drums and broken columns to a stone terrace 0.4 m
// high under an arch; grass, boulders and pines round it, hills beyond.
import { math, og } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { mat, must, pitch, place, rgb, shapes, tex, v3, yaw } from "./util.ts";

/** Where the terrace starts (its step) and how high it stands. */
export const TERRACE_X = 9.5;
export const TERRACE_TOP = 0.4;
/** The way's centre line (between the two lanes) and its width. */
export const WAY_Z = -0.8;
const WAY_WIDTH = 3.6;

/** A kit surface (textures/<name>.{color,normal,orm}.jpg) tiled every `tile` metres over `w` x `h`.
 * The kit's packed map holds roughness (green) and metalness (blue) only: its red is empty, so it is no occlusion map. */
function surface(world: World, name: string, w: number, h: number, tileM: number, tint = rgb(1, 1, 1)): Material {
  const repeat = { x: w / tileM, y: h / tileM };
  const orm = tex(world, `textures/${name}.orm.jpg`);
  return mat(world, {
    map: tex(world, `textures/${name}.color.jpg`), color: tint,
    normalMap: tex(world, `textures/${name}.normal.jpg`),
    roughnessMap: orm, roughness: 1, metalness: 0,
    mapRepeat: repeat, normalMapRepeat: repeat, metalnessMapRepeat: repeat,
  });
}

/** A kit model (models/<name>.glb, about its centre, `half` its half height) standing on y = `y`.
 * The set stands from the first frame: a model whose maps are still being prepared at the start is drawn whole when they are, not faded in. */
function model(world: World, name: string, half: number, x: number, y: number, z: number, scale: number, front = 0): Entity {
  const turn = yaw(front);
  const e = must(world.instantiate(must(og.load.gltf(world, `models/${name}.glb`)), {
    name: `${name}-${x}-${z}`, position: v3(x, y + half * scale, z), quaternion: turn, scale: v3(scale, scale, scale),
  }));
  must(og.render.reveal(world, e, 0));
  return e;
}

export function buildSet(world: World): void {
  const s = shapes(world);
  // The ground and its collider.
  place(world, s.quad, surface(world, "grass", 900, 900, 3, rgb(0.75, 0.95, 0.62)), { name: "meadow", at: v3(6, 0, -20), size: v3(900, 900, 1), turn: pitch(-90), shadow: false });
  const slab = must(world.spawn({ name: "ground-collider", position: v3(6, -0.5, -20) }));
  must(world.insert(slab, og.Collider, { shape: "box", size: v3(160, 1, 160) }));
  // The cobbled way, a hair above the grass.
  const wayLen = 30;
  place(world, s.quad, surface(world, "cobbles", wayLen, WAY_WIDTH, 1.6), { name: "way", at: v3(4, 0.005, WAY_Z), size: v3(wayLen, WAY_WIDTH, 1), turn: pitch(-90), shadow: false });

  // The terrace: dressed stone 0.4 m high, from the step east, with its collider.
  const tw = 6, td = 3.6;
  // Its top and its four faces, each tiled to its own size so the 0.4 m
  // riser shows the same blocks as the paving.
  const stoneTint = rgb(0.95, 0.9, 0.82);
  const cx = TERRACE_X + tw / 2, h = TERRACE_TOP;
  place(world, s.quad, surface(world, "stone", tw, td, 1.5, stoneTint), { name: "terrace-top", at: v3(cx, h, WAY_Z), size: v3(tw, td, 1), turn: pitch(-90) });
  const long = surface(world, "stone", tw, h, 1.5, stoneTint), short = surface(world, "stone", td, h, 1.5, stoneTint);
  place(world, s.quad, long, { name: "terrace-front", at: v3(cx, h / 2, WAY_Z + td / 2), size: v3(tw, h, 1) });
  place(world, s.quad, long, { name: "terrace-back", at: v3(cx, h / 2, WAY_Z - td / 2), size: v3(tw, h, 1), turn: yaw(180) });
  place(world, s.quad, short, { name: "terrace-step", at: v3(TERRACE_X, h / 2, WAY_Z), size: v3(td, h, 1), turn: yaw(-90) });
  place(world, s.quad, short, { name: "terrace-end", at: v3(TERRACE_X + tw, h / 2, WAY_Z), size: v3(td, h, 1), turn: yaw(90) });
  const terrace = must(world.spawn({ name: "terrace-collider", position: v3(TERRACE_X + tw / 2, TERRACE_TOP / 2, WAY_Z) }));
  must(world.insert(terrace, og.Collider, { shape: "box", size: v3(tw, TERRACE_TOP, td) }));
  // The arch over the terrace's far end, the course's goal.
  model(world, "arch", 0.95, TERRACE_X + 4.2, TERRACE_TOP, WAY_Z - 0.4, 2.6, 90);

  // The colonnade behind the way: standing columns and broken stumps.
  const row: Array<[number, string]> = [[-4, "column"], [0, "column_broken"], [4, "column"], [8, "column"], [12, "column_broken"], [16, "column"]];
  for (const [x, name] of row) model(world, name, 0.95, x, 0, -4.6, name === "column" ? 2.3 : 1.6, x * 17);
  // Two broken columns at the near edge, out of the way of the course.
  model(world, "column_broken", 0.95, -1.5, 0, 2.6, 1.2, 40);
  model(world, "column_broken", 0.95, 17.5, 0, 2.2, 1.3, -70);
  // The fallen drum the runner jumps: a low boulder lying in the lane.
  model(world, "boulder", 0.55, 5.8, -0.08, 0, 0.42, 30);

  // Boulders and pines round the forecourt; the pines thicken on the hill.
  const rocks: Array<[number, number, number]> = [[-6, -7, 1.3], [2, -9, 1.6], [10, -8, 1.2], [19, -6, 1.5], [21, 1.5, 1.0], [-7, 1.8, 0.9]];
  for (const [x, z, k] of rocks) model(world, "boulder", 0.55, x, -0.1, z, k, x * 31);
  const pines: Array<[number, number, number]> = [
    [-9, -12, 3.4], [-3, -15, 4.2], [5, -14, 3.8], [13, -13, 4.4], [20, -12, 3.6], [26, -9, 4.0],
    [-12, -22, 5.0], [1, -26, 5.4], [11, -24, 4.8], [22, -23, 5.2], [30, -18, 4.6],
  ];
  for (const [x, z, k] of pines) model(world, "pine", 0.95, x, 0, z, k, x * 13);
  // The forest beyond, out to 70 m, so the horizon is trees, not a plain:
  // rows staggered by a fixed hash, the trees taller further back.
  for (let row = 0; row < 4; row++) {
    for (let i = 0; i < 14; i++) {
      const h = math.sin(i * 12.9898 + row * 78.233) * 43758.5453;
      const jitter = h - Math.floor(h);
      const x = -24 + i * 5.2 + row * 2.4 + jitter * 3;
      const z = -32 - row * 11 - jitter * 6;
      model(world, "pine", 0.95, x, 0, z, 5 + row * 0.8 + jitter * 1.5, i * 47 + row * 11);
    }
  }
}
