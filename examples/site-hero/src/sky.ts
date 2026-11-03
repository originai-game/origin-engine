// The sky: the authored islands, the ruined arch and the far tower placed in
// the shot, and the hero island's waterfall (a water sheet).

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { MeshBuilder, must, v3 } from "./mesh.ts";
import type { Rgb, V3 } from "./mesh.ts";

/** Places an authored model (assets/models/*.glb, ~1.9 units tall, centred on its origin) at `at`, scaled and turned. */
function place(world: World, path: string, name: string, at: V3, scale: number, yaw: number): Entity {
  const e = must(world.instantiate(must(og.load.gltf(world, path)), { name, position: at }));
  must(world.set(e, og.Transform, { position: at, scale: { x: scale, y: scale, z: scale }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) }));
  return e;
}

export interface Sky {
  falls: Entity;
  /** Where the hero island's waterfall leaves its lip (world). */
  lip: V3;
}

/**
 * The sky props, placed about `eye` in the shot's frame (the camera looks down -z):
 * one hero island with the ruined arch and the waterfall centre right, one
 * medium and two small islands lower and left of centre (never in the
 * headline's block), the far tower soft on the horizon at about 38 % across.
 * All authored (Meshy, assets/models/SOURCE.md).
 */
export function buildSky(world: World, eye: V3): Sky {
  const hero = v3(eye.x + 60, eye.y - 70, eye.z - 420);
  const heroScale = 55;
  place(world, "models/island.glb", "island-hero", hero, heroScale, 0.4);
  // The arch on the hero island's cap (its top about 0.6 of the model's half height above its centre).
  place(world, "models/arch.glb", "arch", v3(hero.x - 10, hero.y + heroScale * 0.62, hero.z - 4), 9, -0.5);
  place(world, "models/island.glb", "island-medium", v3(eye.x - 120, eye.y - 120, eye.z - 640), 30, 2.1);
  place(world, "models/islandSmall.glb", "island-small-a", v3(eye.x + 160, eye.y - 90, eye.z - 300), 9, 0.9);
  place(world, "models/islandSmall.glb", "island-small-b", v3(eye.x - 30, eye.y - 140, eye.z - 340), 7, 3.3);
  place(world, "models/tower.glb", "tower", v3(eye.x - 1100, eye.y - 140, eye.z - 3200), 160, 0.3);

  // The waterfall: a thin bright sheet from the hero island's lip down into the deck.
  const lip = v3(hero.x - heroScale * 0.55, hero.y + heroScale * 0.45, hero.z + heroScale * 0.4);
  const f = new MeshBuilder();
  const drop = 420;
  const rows = 24;
  const first = f.positions.length / 3;
  for (let k = 0; k <= rows; k++) {
    const t = k / rows;
    const w = 3.5 + t * 9;
    const y = lip.y - t * drop;
    const out = 4 + Math.sqrt(t) * 14;
    const c: Rgb = [0.92, 0.95, 0.98];
    f.vertex(v3(lip.x - w * 0.5, y, lip.z + out), c);
    f.vertex(v3(lip.x + w * 0.5, y, lip.z + out), c);
  }
  f.grid(first, rows + 1, 2, false);
  const water = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.9, b: 0.95 }, emissive: { r: 0.55, g: 0.6, b: 0.65 }, emissiveIntensity: 0.6, roughness: 0.3, transparent: true, opacity: 0.82, side: "double" }));
  const falls = must(world.spawn({ name: "falls" }));
  must(world.insert(falls, og.Mesh, { geometry: f.geometry(world), material: water }));
  return { falls, lip };
}

