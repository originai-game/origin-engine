// The R9 GPU crowd sample (batch 5): 400 template walkers on a plain, each
// playing the walk on the GPU (og.CrowdAnimation) at its own offset, and
// one at the front posed by og.anim (og.Skeleton and og.AnimationPlayer)
// for comparison. tests/evidence/r9/NOTES.md has the commands and what
// each frame shows.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** A deterministic 0..1 hash of an index and a salt. */
const hash = (k: number, salt: number): number => {
  let h = Math.imul(k + 1, 0x8da6b343) ^ Math.imul(salt + 7, 0xd8163841);
  h = Math.imul(h ^ (h >>> 13), 0x85ebca6b);
  h ^= h >>> 16;
  return (h & 0xffff) / 65535;
};

export default og.game({
  tick: 60,
  seed: 1,
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0, y: 6, z: 14 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 0.5, z: -6 });
    cam.fov = 50;

    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5, y: -0.7, z: -0.5 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3, color: { r: 1, g: 0.96, b: 0.9 }, castShadow: true });

    const box = must(og.render.boxGeometry(world, {}));
    const grass = must(og.render.standardMaterial(world, { color: { r: 0.32, g: 0.4, b: 0.26 }, roughness: 0.95 }));
    const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: -6 } }));
    world.set(ground, og.Transform, { scale: { x: 60, y: 1, z: 60 } });
    world.insert(ground, og.Mesh, { geometry: box, material: grass, receiveShadow: true });

    const anim = must(og.load.animation(world, "models/walker.glb"));
    const walk = anim.clips[0]!.clip;
    const walker = must(og.load.gltf(world, "models/walker.glb"));

    // 20 × 20, 1.2 m apart, each a little off its grid point, turned its
    // own way, and out of step with its neighbours.
    for (let k = 0; k < 400; k++) {
      const i = k % 20;
      const j = Math.floor(k / 20);
      const at = { x: -11.4 + i * 1.2 + (hash(k, 1) - 0.5) * 0.4, y: 0, z: 2 - j * 1.2 + (hash(k, 2) - 0.5) * 0.4 };
      const e = must(world.instantiate(walker, { name: `walker-${k}`, position: at }));
      world.set(e, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, hash(k, 3) * Math.PI * 2) });
      world.insert(e, og.CrowdAnimation, { skeleton: anim.skeleton, clip: walk, offset: hash(k, 4) * 4, speed: 0.8 + 0.4 * hash(k, 5) });
    }

    // The one og.anim poses, at the front, for comparison.
    const solo = must(world.instantiate(walker, { name: "solo", position: { x: 0, y: 0, z: 5 } }));
    world.insert(solo, og.Skeleton, { asset: anim.skeleton });
    world.insert(solo, og.AnimationPlayer, { clip: walk, loop: true });
  },
});
