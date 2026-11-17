// The land: the valley's terrain, the lake, the grass and everything
// scattered over the ground (woods, bushes, heather, flowers).
import { og } from "@origingame/engine";
import type { TerrainData, TerrainScatter, World } from "@origingame/engine";
import { SIZE } from "./layout.ts";
import { ORIGIN, must } from "./util.ts";

/** The terrain's layers, as assets/terrain/valley.ogt lists them. */
export const LAYER = { meadow: 0, rock: 1, shore: 2, snow: 3, forest: 4, path: 5, field: 6, alpine: 7 } as const;

/** The scatters that are woods, by name. */
export const WOODS = ["pines", "pines-tall", "oaks", "birches", "bushes"] as const;

export function buildLand(world: World): TerrainData {
  const valley = must(og.terrain.load(world, "terrain/valley.ogt"));
  const ground = must(world.spawn({ name: "valley", position: ORIGIN }));
  must(world.insert(ground, og.Terrain, { data: valley.data }));
  must(world.insert(ground, og.Collider, { shape: "heightfield", mesh: valley.geometry }));

  const lake = must(world.spawn({ name: "lake", position: ORIGIN }));
  must(world.insert(lake, og.Water, {
    size: { x: SIZE, y: SIZE },
    segments: 8,
    waveStrength: 0.1,
    waveScale: 0.3,
    waveSpeed: 0.5,
    depthFade: 5,
    foamWidth: 0.12,
    shallow: { r: 0.36, g: 0.55, b: 0.52 },
    deep: { r: 0.03, g: 0.1, b: 0.14 },
  }));

  // The meadow, and the wheat on the field below the mill.
  const meadow = must(world.spawn({ name: "meadow" }));
  must(world.insert(meadow, og.Grass, { data: valley.data, layer: LAYER.meadow, density: 300, distance: 120, height: 0.42, dryness: 0.1 }));
  const wheat = must(world.spawn({ name: "wheat" }));
  must(world.insert(wheat, og.Grass, {
    data: valley.data, layer: LAYER.field, density: 300, distance: 200, height: 0.95, heightVariation: 0.15, width: 0.016, curve: 0.25, lean: 0.12, clumpLean: 0.1,
    baseColor: { r: 0.3, g: 0.24, b: 0.07 }, tipColor: { r: 0.72, g: 0.55, b: 0.17 }, dryColor: { r: 0.75, g: 0.62, b: 0.25 }, dryness: 0.5, seed: 3,
  }));

  // The woods: conifers and broadleaves over the forest floor, each species
  // in its own groves, drawn as impostors beyond the nearest of them.
  const scatter = (name: string, model: string, desc: Partial<TerrainScatter>) => {
    const e = must(world.spawn({ name }));
    must(world.insert(e, og.TerrainScatter, { data: valley.data, model: must(og.load.gltf(world, model)), ...desc }));
  };
  const wood = { layer: LAYER.forest, distance: 2000, fade: 0.05, impostorDistance: 110, footprint: 3.2, priority: 3, sway: 0.25, align: 0 };
  scatter("pines", "plants/pine.ogt", { ...wood, density: 0.012, minScale: 0.85, maxScale: 1.5, seed: 1 });
  scatter("pines-tall", "plants/pine-tall.ogt", { ...wood, density: 0.008, minScale: 0.9, maxScale: 1.35, seed: 2, noiseThreshold: 0.35, noiseScale: 90 });
  scatter("oaks", "plants/oak.ogt", { ...wood, density: 0.006, minScale: 0.8, maxScale: 1.3, seed: 3, noiseThreshold: 0.45, noiseScale: 120, footprint: 5 });
  scatter("birches", "plants/birch.ogt", { ...wood, density: 0.006, minScale: 0.8, maxScale: 1.3, seed: 4, noiseThreshold: 0.5, noiseScale: 70 });
  scatter("bushes", "plants/shrub.ogt", { layer: LAYER.forest, distance: 160, density: 0.004, minScale: 0.6, maxScale: 1.1, seed: 5, footprint: 1.5, priority: 2, sway: 0.4 });

  // The meadow's own: heather in drifts and daisies between.
  scatter("heather", "plants/heather.ogt", { layer: LAYER.meadow, distance: 70, density: 0.5, minScale: 0.5, maxScale: 1, seed: 6, noiseThreshold: 0.6, noiseScale: 9, footprint: 0.3, priority: 1, sway: 0.6, castShadows: false });
  scatter("daisies", "plants/daisy.ogt", { layer: LAYER.meadow, distance: 45, density: 1.4, minScale: 0.7, maxScale: 1.2, seed: 7, noiseThreshold: 0.4, noiseScale: 6, sway: 0.8, castShadows: false });
  return valley.data;
}
