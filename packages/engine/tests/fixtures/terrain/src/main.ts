// The fixture terrain.test.ts cooks for its assets (the tests drive their
// own worlds). Run on its own it shows the hill from above, and `hole`
// cuts a pit into it (R6's holes):
//   og run packages/engine/tests/fixtures/terrain --size 1280x720 --out og-out \
//     --commands "0.2:screenshot:hill.png;0.3:hole:20:20:4;0.6:screenshot:hole.png;0.7:quit"
// `grass` grows a default og.Grass over it, and `wind` blows og.Wind (R6's grass):
//   og run packages/engine/tests/fixtures/terrain --size 1280x720 --out og-out \
//     --commands "0.2:grass;0.4:screenshot:grass.png;0.5:wind:6;1.0:screenshot:grass-wind.png;1.1:quit"
import { og, type World } from "@origingame/engine";

// The hill and its terrain data live in the world (the entity named "hill"
// and its og.Terrain), so a restore or a fork brings them back with it.
function hill(world: World) {
  const e = world.find("hill");
  if (e === null) throw new Error("no hill: start spawns it");
  const t = world.get(e, og.Terrain);
  if (!t.ok) throw new Error(t.reason);
  return { entity: e, data: t.value.data };
}

export default og.game({
  start(world) {
    const info = og.terrain.load(world, "terrain/hill.ogt");
    if (!info.ok) throw new Error(info.reason);
    const spawned = world.spawn({ name: "hill" });
    if (!spawned.ok) throw new Error(spawned.reason);
    world.insert(spawned.value, og.Terrain, { data: info.value.data });
    world.insert(spawned.value, og.Collider, { shape: "heightfield", mesh: info.value.geometry });
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
      const r = og.terrain.edit(world, hill(world).data, {
        mode: "hole" as never,
        center: { x: Number(args[0] ?? 20), y: 0, z: Number(args[1] ?? 20) },
        radius: Number(args[2] ?? 4),
        falloff: 0,
      });
      if (!r.ok) throw new Error(r.reason);
    },
    /** `grass`: a default og.Grass (a meadow) over the hill's first layer. */
    grass: (world) => {
      const h = hill(world);
      world.insert(h.entity, og.Grass, { data: h.data });
    },
    /** `wind:<speed>`: og.Wind along +x, in m/s. */
    wind: (world, args) => {
      const w = world.resource(og.Wind);
      if (!w.ok) throw new Error(w.reason);
      w.value.speed = Number(args[0] ?? 6);
    },
  },
  commandHelp: {
    hole: { usage: "<x>:<z>:<radius>", doc: "Cut a round pit into the hill (og.terrain.edit's hole brush)." },
    grass: { usage: "", doc: "Grow a default og.Grass over the hill." },
    wind: { usage: "<speed>", doc: "Blow og.Wind along +x at <speed> m/s." },
  },
});
