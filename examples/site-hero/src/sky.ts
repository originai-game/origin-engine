// The sky: the authored islands, the ruined arch and the far tower placed in
// the shot.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { must, v3 } from "./mesh.ts";
import type { V3 } from "./mesh.ts";

/** Places an authored model (assets/models/*.glb, ~1.9 units tall, centred on its origin) at `at`, scaled and turned. */
function place(world: World, path: string, name: string, at: V3, scale: number, yaw: number): Entity {
  const e = must(world.instantiate(must(og.load.gltf(world, path)), { name, position: at }));
  must(world.set(e, og.Transform, { position: at, scale: { x: scale, y: scale, z: scale }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, yaw) }));
  return e;
}

/**
 * The sky props, placed about `eye` in the shot's frame (the camera looks down -z):
 * one hero island with the ruined arch centre right, one
 * medium and two small islands lower and left of centre (never in the
 * headline's block), the far tower 2 km out, 6° left of the shots' -z:
 * about 20 % across the hero and 27 % across the wide shot, clear of both
 * edges (at 3.3 km and 19° left it fell outside the hero's left edge and
 * thinned to a flat silhouette in the haze), its foot sunk in the cloud deck
 * so it rises out of it.
 * All authored (Meshy, assets/models/SOURCE.md).
 */
export function buildSky(world: World, eye: V3): void {
  const hero = v3(eye.x + 60, eye.y - 70, eye.z - 420);
  const heroScale = 55;
  place(world, "models/island.glb", "island-hero", hero, heroScale, 0.4);
  // The arch on the hero island's cap (its top about 0.6 of the model's half height above its centre).
  place(world, "models/arch.glb", "arch", v3(hero.x - 10, hero.y + heroScale * 0.62, hero.z - 4), 9, -0.5);
  place(world, "models/island.glb", "island-medium", v3(eye.x - 120, eye.y - 120, eye.z - 640), 30, 2.1);
  place(world, "models/islandSmall.glb", "island-small-a", v3(eye.x + 160, eye.y - 90, eye.z - 300), 9, 0.9);
  place(world, "models/islandSmall.glb", "island-small-b", v3(eye.x - 30, eye.y - 140, eye.z - 340), 7, 3.3);
  place(world, "models/tower.glb", "tower", v3(eye.x - 216, eye.y - 345, eye.z - 1990), 160, 0.3);
}

