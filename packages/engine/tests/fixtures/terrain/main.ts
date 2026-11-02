// The fixture terrain.test.ts cooks for its assets (the tests drive their
// own worlds). Run on its own it shows the hill from above, and `hole`
// cuts a pit into it (R6's holes):
// (--no-check: the fixture has no src/ for og check to read)
//   og run packages/engine/tests/fixtures/terrain --no-check --size 1280x720 --out og-out \
//     --commands "0.2:screenshot:hill.png;0.3:hole:20:20:4;0.6:screenshot:hole.png;0.7:quit"
import { og, SHAPE_HEIGHTFIELD } from "@origingame/engine";

let terrain = 0;

export default og.game({
  start(world) {
    const info = og.terrain.load(world, "terrain/hill.ogt");
    if (!info.ok) throw new Error(info.reason);
    terrain = info.value.data as unknown as number;
    const hill = world.spawn({ name: "hill" });
    if (!hill.ok) throw new Error(hill.reason);
    world.insert(hill.value, og.Terrain, { data: info.value.data });
    world.insert(hill.value, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.value.geometry });
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      const eye = { x: 20, y: 22, z: 38 };
      cam.value.position = eye;
      cam.value.quaternion = og.transform.lookAt(eye, { x: 20, y: 2, z: 20 });
      cam.value.fov = 50;
      cam.value.far = 500;
    }
    const sun = world.spawn({ name: "sun" });
    if (sun.ok) {
      world.insert(sun.value, og.DirectionalLight, { intensity: 3 });
      world.set(sun.value, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.3, y: -2, z: -0.4 }) });
    }
  },
  commands: {
    /** `hole:<x>:<z>:<radius>`: cuts a round pit into the hill. */
    hole: (world, args) => {
      const r = og.terrain.edit(world, terrain as never, {
        mode: "hole" as never,
        center: { x: Number(args[0] ?? 20), y: 0, z: Number(args[1] ?? 20) },
        radius: Number(args[2] ?? 4),
        falloff: 0,
      });
      if (!r.ok) throw new Error(r.reason);
    },
  },
  commandHelp: {
    hole: { usage: "<x>:<z>:<radius>", doc: "Cut a round pit into the hill (og.terrain.edit's hole brush)." },
  },
});
