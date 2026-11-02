// The R7 decals sample (batch 1): 600 normal-mapped og.Decal bullet holes
// on a wall and the ground before it, and two large ones under a low sun.
// tests/evidence/r7/NOTES.md has the commands and what each frame shows.

import { og } from "@origingame/engine";
import type { Entity, Geometry, Material, Quaternion, Vector3, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: Vector3, scale: Vector3): Entity {
  const e = must(world.spawn({ name, position }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

/** A deterministic 0..1 hash of an index and a salt. */
const hash = (k: number, salt: number): number => {
  let h = Math.imul(k + 1, 0x8da6b343) ^ Math.imul(salt + 7, 0xd8163841);
  h = Math.imul(h ^ (h >>> 13), 0x85ebca6b);
  h ^= h >>> 16;
  return (h & 0xffff) / 65535;
};

/** The Hamilton product a × b (b's rotation first). */
const qmul = (a: Quaternion, b: Quaternion): Quaternion => ({
  x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
  y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
  z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
  w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
});

// The box's local +Z is the surface normal it covers: the ground's is +Y.
const flat = (turn: number): Quaternion =>
  qmul(og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn), og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2));
const upright = (turn: number): Quaternion => og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, turn);

function hole(world: World, name: string, at: Vector3, q: Quaternion, size: number, normalScale = 1): void {
  const e = must(world.spawn({ name, position: at, quaternion: q }));
  world.set(e, og.Transform, { scale: { x: size, y: size, z: 0.3 } });
  world.insert(e, og.Decal, {
    uvMin: { x: 0, y: 0 },
    uvMax: { x: 0.5, y: 1 },
    normalMin: { x: 0.5, y: 0 },
    normalMax: { x: 1, y: 1 },
    normalScale,
    roughness: 0.95,
  });
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0, y: 3, z: 5.6 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 0.6, z: -0.8 });
    cam.fov = 50;
    const settings = must(world.resource(og.RenderSettings));
    settings.decalAtlas = must(og.load.texture(world, "textures/bullet-holes.png"));

    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.8, y: -0.45, z: -0.4 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3, color: { r: 1, g: 0.95, b: 0.88 }, castShadow: true });

    const box = must(og.render.boxGeometry(world, {}));
    const plaster = must(og.render.standardMaterial(world, { color: { r: 0.82, g: 0.78, b: 0.7 }, roughness: 0.9 }));
    const ground = must(og.render.standardMaterial(world, { color: { r: 0.42, g: 0.38, b: 0.33 }, roughness: 0.95 }));
    mesh(world, "ground", box, ground, { x: 0, y: -0.5, z: 0 }, { x: 20, y: 1, z: 14 });
    mesh(world, "wall", box, plaster, { x: 0, y: 1.5, z: -2.2 }, { x: 8, y: 3, z: 0.4 });

    for (let k = 0; k < 600; k++) {
      const i = k % 32;
      const j = Math.floor(k / 32);
      const size = 0.12 + 0.1 * hash(k, 3);
      const turn = hash(k, 9) * Math.PI * 2;
      const jx = (hash(k, 1) - 0.5) * 0.1;
      const jy = (hash(k, 2) - 0.5) * 0.1;
      if (k < 384) {
        hole(world, `wall-hole-${k}`, { x: -3.8 + i * 0.245 + jx, y: 0.2 + j * 0.22 + jy, z: -2 }, upright(turn), size);
      } else {
        hole(world, `ground-hole-${k}`, { x: -3.8 + i * 0.245 + jx, y: 0, z: -1.6 + (j - 12) * 0.3 + jy }, flat(turn), size);
      }
    }
    // Two large ones where the relief reads: the right one twice as strong.
    hole(world, "crater-a", { x: -1.5, y: 0, z: 1 }, flat(0), 1.6, 1);
    hole(world, "crater-b", { x: 1.5, y: 0, z: 1 }, flat(0), 1.6, 2);
  },
});
