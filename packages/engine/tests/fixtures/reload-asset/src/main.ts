// The game of tests/reload-asset.test.ts: the scene's dropship
// (assets/scenes/main.ogt) under a sun and the sky, the camera 8 m in
// front of it, where the old game template looked at it.
import { og } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.position = { x: 0.4, y: 1.8, z: 8 };
    camera.quaternion = { x: 0, y: 0, z: 0, w: 1 };
    must(world.resource(og.RenderSettings)).sky = true;
    const sun = must(world.spawn({ name: "sun" }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -1.5, z: -0.6 }) });
  },
});
