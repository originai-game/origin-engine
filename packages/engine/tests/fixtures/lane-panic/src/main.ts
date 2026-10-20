// A crowd for the lanes: 2,304 boxes under a camera and a light, so every
// frame's CPU frustum cull tests enough bounds (two of its 1,024-bound
// grains; under GPU culling's 16,384 instances) to split across the lanes
// pool when the page boots the lanes variant
// (docs/web-export.md §Lanes). tests/web/lane-panic.ts injects a panic into
// one of those lanes and checks that the page reports it.
import { og, type World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

export const SIDE = 48;

function start(world: World): void {
  const camera = must(world.resource(og.Camera));
  camera.position = { x: 0, y: 30, z: 40 };
  camera.quaternion = og.transform.lookAt({ x: 0, y: 30, z: 40 }, { x: 0, y: 0, z: 0 });
  const geometry = must(og.render.boxGeometry(world, { width: 0.6, height: 0.6, depth: 0.6 }));
  const material = must(og.render.standardMaterial(world, { color: { r: 0.9, g: 0.55, b: 0.2 } }));
  for (let i = 0; i < SIDE * SIDE; i++) {
    const x = (i % SIDE) - SIDE / 2;
    const z = Math.floor(i / SIDE) - SIDE / 2;
    const box = must(world.spawn({ name: `box-${i}`, position: { x, y: 0, z } }));
    must(world.insert(box, og.Mesh, { geometry, material }));
  }
}

export default og.game({ tick: 60, start });
