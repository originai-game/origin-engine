import { og, defineSystem } from "@origingame/engine";

const view = defineSystem("View", { writes: [og.Camera] }, (_query, time, world) => {
  const camera = world.resource(og.Camera);
  if (!camera.ok) throw new Error(camera.reason);
  if (time.tick < 180) {
    const x = 75 + time.tick * 0.05;
    camera.value.position = { x, y: 1.7, z: 80 };
    camera.value.quaternion = og.transform.lookAt(camera.value.position, { x: x + 35, y: 4, z: 72 });
    og.assert.reachable("street view");
  } else {
    camera.value.position = { x: 90, y: 55, z: -25 };
    camera.value.quaternion = og.transform.lookAt(camera.value.position, { x: 90, y: 0, z: 40 });
    og.assert.reachable("block view");
  }
  og.assert.always(camera.value.position.y > 1.6, "camera stays above the road");
});

export default og.game({
  scenes: { main: "districts/town.ogt" },
  systems: [view],
  start(world) {
    og.stream.setBudget(world, 512 * 1024 * 1024);
    const land = og.terrain.load(world, "base/terrain/land");
    if (!land.ok) throw new Error(land.reason);
    const ground = world.spawn({ name: "land" });
    if (!ground.ok) throw new Error(ground.reason);
    const terrain = world.insert(ground.value, og.Terrain, { data: land.value.data });
    if (!terrain.ok) throw new Error(terrain.reason);
    const collider = world.insert(ground.value, og.Collider, { shape: "heightfield", mesh: land.value.geometry });
    if (!collider.ok) throw new Error(collider.reason);
    const sky = world.resource(og.PhysicalSky);
    if (sky.ok) {
      sky.value.enabled = true;
      sky.value.haze = 4;
    }
  },
});
