// A game that spawns a box in front of the camera (tests/sourcemap.test.ts):
// the probe's source for it must be this file's line, not the bundle's.
import { og, type World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  start(world: World) {
    const box = must(og.render.boxGeometry(world, {}));
    const grey = must(og.render.standardMaterial(world, {}));
    const crate = must(world.spawn({ name: "crate", position: { x: 0, y: 0, z: -4 } }));
    must(world.insert(crate, og.Mesh, { geometry: box, material: grey }));
  },
});
