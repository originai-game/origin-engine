import { og, defineResource, defineSystem, f32 } from "@origingame/engine";

function must<T>(result: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

const Sample = defineResource("Sample", { height: f32() });
const sample = defineSystem("SampleTerrain", { reads: [og.Terrain], writes: [Sample] }, (_query, time, world) => {
  // The terrain's data from its entity, not a module variable: a restore brings it back.
  const terrain = must(world.get(world.find("terrain")!, og.Terrain));
  const ground = must(og.terrain.sample(world, {
    data: terrain.data, origin: { x: 3000, y: 0, z: 0 }, x: 2960 + time.tick / 60, z: 36,
  }));
  must(world.resource(Sample)).height = ground.height;
});

// The camera dives from far above into the valley, so finer tiles stream
// in and fade over their parents as it comes down (the presentation
// checkpoint's terrain fade: packages/engine/tests/presentation.test.ts).
const fly = defineSystem("Fly", { writes: [og.Camera] }, (_query, time, world) => {
  const camera = must(world.resource(og.Camera));
  const k = Math.max(0, 1 - time.tick / 120);
  camera.position = { x: 2962, y: 21 + 2400 * k * k, z: 64 + 1200 * k * k };
  camera.quaternion = og.transform.lookAt(camera.position, { x: 2992, y: 1, z: -16 });
});

export default og.game({
  resources: [Sample],
  systems: [sample, fly],
  start(world) {
    const info = must(og.terrain.load(world, "terrain/valley.ogt"));
    const entity = must(world.spawn({ name: "terrain", position: { x: 3000, y: 0, z: 0 } }));
    world.insert(entity, og.Terrain, { data: info.data });
    const camera = must(world.resource(og.Camera));
    camera.position = { x: 2962, y: 21, z: 64 };
    camera.quaternion = og.transform.lookAt(camera.position, { x: 2992, y: 1, z: -16 });
  },
});
