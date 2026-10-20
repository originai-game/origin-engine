// What the lanes test and `bench/lanes.ts --crowd` add to the reference
// scene (crates/og-tests/reference-scene) so every lane has more than two
// grains of work: 2000 small boxes on a wide grid, most of them outside the
// view (the frustum cull splits from 2048 items; extraction reads them from
// the columns),
// 160 more walkers (og-anim splits from 128 skeletons) and 48 boxes dropped
// on a floor collider (the rapier step).

import { og, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export function crowd(world: World): void {
  const cube = must(og.render.boxGeometry(world, { width: 0.3, height: 0.3, depth: 0.3 }));
  const paint = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.5, b: 0.9 }, roughness: 0.6 }));
  for (let i = 0; i < 2000; i++) {
    const position = { x: ((i % 50) - 24.5) * 2.4, y: 0.15, z: -Math.floor(i / 50) * 2.4 - 4 };
    const e = must(world.spawn({ name: `far-${i}`, position }));
    world.insert(e, og.Mesh, { geometry: cube, material: paint, castShadow: false, receiveShadow: true });
  }
  const model = must(og.load.gltf(world, "models/walker.glb"));
  const anim = must(og.load.animation(world, "models/walker.glb"));
  const clips = ["walk", "run", "idle", "wave"].map((n) => anim.clips.find((c) => c.name === n)!.clip);
  for (let i = 0; i < 160; i++) {
    const position = { x: ((i % 20) - 9.5) * 0.9, y: 0, z: 16.5 + Math.floor(i / 20) * 0.6 };
    const e = must(world.instantiate(model, { name: `crowd-${i}`, position }));
    world.insert(e, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
    world.insert(e, og.AnimationPlayer, { clip: clips[i % clips.length]!, speed: 0.7 + (i % 9) * 0.05, time: (i * 0.071) % 1, loop: true, fade: 0 });
  }
  const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: 0 } }));
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 80, y: 1, z: 80 } });
  const crate = must(og.render.boxGeometry(world, { width: 0.6, height: 0.6, depth: 0.6 }));
  for (let i = 0; i < 48; i++) {
    const position = { x: ((i % 6) - 2.5) * 0.9 + (i % 3) * 0.05, y: 3 + Math.floor(i / 6) * 0.8, z: 14 + ((i * 7) % 5) * 0.2 };
    const e = must(world.spawn({ name: `crate-${i}`, position }));
    world.insert(e, og.RigidBody, {});
    world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.6, y: 0.6, z: 0.6 } });
    world.insert(e, og.Mesh, { geometry: crate, material: paint, castShadow: true, receiveShadow: true });
  }
}
