// The sprite sheet: assets/models/sprites.glb holds one unit quad per sprite,
// each with its own material, in this order (tools/make-assets.py writes it).
// A sprite is drawn by an og.Mesh with that geometry and material; its size
// is the entity's Transform.scale.

import { og } from "@origingame/engine";
import type { Geometry, Material, World } from "@origingame/engine";

export const SPRITES = [
  "sky",
  "moon",
  "cloud",
  "building_a",
  "building_b",
  "building_c",
  "ground",
  "crate",
  "fence",
  "coin",
  "runner_idle",
  "runner_run_a",
  "runner_run_b",
  "runner_jump",
  "runner_crash",
] as const;

export type SpriteName = (typeof SPRITES)[number];

export interface Sprite {
  readonly geometry: Geometry;
  readonly material: Material;
}

export type Sheet = Readonly<Record<SpriteName, Sprite>>;

/** Loads the sheet once, in `start`; every sprite is ready at once from the cooked bundle. */
export function loadSheet(world: World): Sheet {
  const asset = og.load.gltf(world, "models/sprites.glb");
  if (!asset.ok) throw new Error(asset.reason);
  const info = og.load.gltfInfo(world, asset.value);
  if (!info.ok) throw new Error(info.reason);
  const out: Partial<Record<SpriteName, Sprite>> = {};
  SPRITES.forEach((name, i) => {
    const geometry = info.value.geometries[i];
    const material = info.value.materials[i];
    if (geometry === undefined || material === undefined) {
      throw new Error(`models/sprites.glb has no sprite ${i} (${name}); run tools/make-assets.py and og cook`);
    }
    out[name] = { geometry, material };
  });
  return out as Sheet;
}
