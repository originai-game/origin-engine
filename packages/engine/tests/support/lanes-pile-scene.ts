// The physics pile of the lanes tests and benches: `bodies` boxes stacked
// ten layers high over a floor collider, each layer shifted so the columns
// topple into each other. tests/support/lanes-pile.ts steps it headless
// natively (lanes.test.ts); bench/lanes.ts --pile builds it into a game,
// drawn (a camera on the pile, a sun, a mesh on every box) so the Chrome
// modes can compare their captures. og-physics steps it on the lanes from
// `og_physics::pool_bodies` for the pool's thread count (500 bodies on two
// to four lanes, 1,000 on five or six, 1,500 from seven).

import { og, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Spawns the pile of `bodies` boxes; `drawn` adds the camera, a sun and the boxes' meshes. */
export function pile(world: World, bodies: number, drawn = false): void {
  const layers = 10;
  const perLayer = Math.ceil(bodies / layers);
  const side = Math.ceil(Math.sqrt(perLayer));
  const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: 0 } }));
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 80, y: 1, z: 80 } });
  let draw: ((e: Parameters<World["insert"]>[0]) => void) | null = null;
  if (drawn) {
    const camera = must(world.resource(og.Camera));
    const eye = { x: 0, y: side * 0.55 + 6, z: side * 0.9 + 8 };
    camera.position = eye;
    camera.quaternion = og.transform.lookAt(eye, { x: 0, y: 1, z: 0 });
    camera.fov = 50;
    camera.far = 200;
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.45, y: -0.75, z: -0.5 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 2.6 });
    const geometry = must(og.render.boxGeometry(world, { width: 0.5, height: 0.5, depth: 0.5 }));
    const material = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.55, b: 0.25 }, roughness: 0.6 }));
    draw = (e) => world.insert(e, og.Mesh, { geometry, material, castShadow: false, receiveShadow: true });
    const floor = must(og.render.boxGeometry(world, { width: 80, height: 1, depth: 80 }));
    const grey = must(og.render.standardMaterial(world, { color: { r: 0.35, g: 0.36, b: 0.4 }, roughness: 0.9 }));
    world.insert(ground, og.Mesh, { geometry: floor, material: grey, castShadow: false, receiveShadow: true });
  }
  for (let i = 0; i < bodies; i++) {
    const layer = Math.floor(i / perLayer);
    const k = i % perLayer;
    const shift = (layer % 3) * 0.17;
    const position = { x: ((k % side) - side / 2) * 0.7 + shift, y: 0.3 + layer * 0.62, z: (Math.floor(k / side) - side / 2) * 0.7 - shift };
    const e = must(world.spawn({ name: `box-${i}`, position }));
    world.insert(e, og.RigidBody, {});
    world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.5, y: 0.5, z: 0.5 } });
    draw?.(e);
  }
}
