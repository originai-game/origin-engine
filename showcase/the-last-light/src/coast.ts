// The coast: the ground, the buildings and rocks on it, what grows there,
// and the sea.
import { og } from "@origingame/engine";
import type { Entity, TerrainData, World } from "@origingame/engine";
import { COTTAGE, LIGHTHOUSE, RUIN } from "./layout.ts";
import { must } from "./must.ts";

export interface Coast {
  ground: Entity;
  data: TerrainData;
  sea: Entity;
  /** The lighthouse's lamp, in the world. */
  lamp: { x: number; y: number; z: number };
  heightAt(x: number, z: number): number;
}

export function buildCoast(world: World): Coast {
  const terrain = must(og.terrain.load(world, "terrain/coast.ogt"));
  const ground = must(world.spawn({ name: "coast" }));
  must(world.insert(ground, og.Terrain, { data: terrain.data }));
  must(world.insert(ground, og.Collider, { shape: "heightfield", mesh: terrain.geometry }));
  const heightAt = (x: number, z: number) => must(og.terrain.sample(world, { data: terrain.data, origin: { x: 0, y: 0, z: 0 }, x, z })).height;

  // A generated model is a unit cube about its centre: `size` is its
  // longest side in metres, `half` how far its foot is under its centre
  // (in the cube's units), `sink` how far the foot goes into the ground.
  const place = (name: string, path: string, x: number, z: number, size: number, half: number, turn = 0, sink = 0.3) => {
    const k = size / 1.9;
    const y = heightAt(x, z) + half * k - sink;
    const e = must(world.spawn({ name, position: { x, y, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn), scale: { x: k, y: k, z: k } }));
    must(world.insert(e, og.Model, { asset: must(og.load.gltf(world, path)) }));
    return y;
  };
  const base = place("lighthouse", "models/lighthouse.glb", LIGHTHOUSE.x, LIGHTHOUSE.z, LIGHTHOUSE.height, 0.95, 0.4);
  place("cottage", "models/cottage.glb", COTTAGE.x, COTTAGE.z, COTTAGE.length, 0.43, 0.2, 0.5);
  place("ruin", "models/ruin.glb", RUIN.x, RUIN.z, RUIN.length, 0.55, -0.3, 0.5);
  place("crag", "models/rock-cliff.glb", -8.5, -5.5, 9, 0.6, 2.2, 1.2);
  place("boulder", "models/boulder.glb", 2.6, -2.4, 2.4, 0.27, 0.6, 0.15);
  // Stacks and skerries off the point and under the cliffs, where the swell breaks.
  place("stack-point", "models/sea-stack.glb", 58, -80, 15, 0.33, 0.3, 2.2);
  place("stack-near", "models/sea-stack.glb", 29, -28, 11, 0.33, 2.4, 2.0);
  place("stack-cove", "models/sea-stack.glb", 40, -50, 14, 0.33, 4.1, 2.2);
  place("crag-point", "models/rock-cliff.glb", 43, -84, 11, 0.6, 3.6, 2.5);

  // The hawthorn on the crag, its crown combed inland over the path.
  const tree = must(world.spawn({ name: "hawthorn", position: { x: -6.2, y: heightAt(-6.2, -1.6) + 1.2, z: -1.6 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, 2.6) }));
  must(world.insert(tree, og.Model, { asset: must(og.load.gltf(world, "plants/hawthorn.ogt")) }));

  // Turf: fine grass everywhere it holds, tall plumed grass in drifts, and
  // the cliff-top flowers in their own patches.
  must(world.insert(ground, og.Grass, { data: terrain.data, layer: 0, density: 700, height: 0.55, distance: 120 }));
  const grow = (name: string, path: string, density: number, patch: number, threshold: number, seed: number, distance = 45, scale: readonly [number, number] = [0.8, 1.25]) => {
    const e = must(world.spawn({ name }));
    must(world.insert(e, og.TerrainScatter, {
      data: terrain.data, model: must(og.load.gltf(world, path)), layer: 0, density, distance, noiseScale: patch, noiseThreshold: threshold,
      seed, minScale: scale[0], maxScale: scale[1], castShadows: false, avoidWater: 3,
    }));
  };
  grow("plume-grass", "plants/plume-grass.ogt", 0.7, 9, 0.46, 3, 70, [0.75, 1.3]);
  grow("oxeye", "plants/oxeye.ogt", 5, 5, 0.6, 5);
  grow("buttercup", "plants/buttercup.ogt", 5, 4, 0.62, 8);
  grow("thrift", "plants/thrift.ogt", 6, 6, 0.6, 13);
  grow("foxglove", "plants/foxglove.ogt", 0.8, 7, 0.55, 21, 45, [0.8, 1.2]);

  const sea = must(world.spawn({ name: "sea" }));
  must(world.insert(sea, og.Water, {
    size: { x: 6000, y: 6000 }, segments: 256, endless: true, seabed: ground,
    // A fresh breeze off the sea: a long swell that shoals on the reef
    // and breaks on the rocks.
    wind: { x: -11, y: 5.5 }, swell: 1.25, choppiness: 0.85, shore: 2, shoreFoam: 1.6, foamWidth: 1.2,
    deep: { r: 0.01, g: 0.06, b: 0.09 }, shallow: { r: 0.2, g: 0.5, b: 0.55 }, depthFade: 12, waveStrength: 0.25,
  }));

  // The lamp room, an eighth of the tower below its top.
  const lamp = { x: LIGHTHOUSE.x, y: base + LIGHTHOUSE.height * 0.38, z: LIGHTHOUSE.z };
  return { ground, data: terrain.data, sea, lamp, heightAt };
}
