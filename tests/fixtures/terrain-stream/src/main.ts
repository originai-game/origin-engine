import { og, defineResource, defineSystem, f32 } from "@origingame/engine";
import type { TerrainData } from "@origingame/engine";

function must<T>(result: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

const Sample = defineResource("Sample", { height: f32() });
let terrain: TerrainData;
const sample = defineSystem("SampleTerrain", { writes: [Sample] }, (_query, time, world) => {
  const ground = must(og.terrain.sample(world, {
    data: terrain, origin: { x: 3000, y: 0, z: 0 }, x: 2960 + time.tick / 60, z: 36,
  }));
  must(world.resource(Sample)).height = ground.height;
});

export default og.game({
  resources: [Sample],
  systems: [sample],
  start(world) {
    const info = must(og.terrain.load(world, "terrain/valley.ogt"));
    terrain = info.data;
    const entity = must(world.spawn({ name: "terrain", position: { x: 3000, y: 0, z: 0 } }));
    world.insert(entity, og.Terrain, { data: terrain });
    const camera = must(world.resource(og.Camera));
    camera.position = { x: 2962, y: 21, z: 64 };
    camera.quaternion = og.transform.lookAt(camera.position, { x: 2992, y: 1, z: -16 });
  },
});
