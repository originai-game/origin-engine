// @bun
// packages/engine/tests/fixtures/video/src/main.ts
import { og } from "/home/user/workspace/repo/packages/engine/src/index.ts";
function must(r) {
  if (!r.ok)
    throw new Error(r.reason);
  return r.value;
}
var main_default = og.game({
  tick: 30,
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = 9;
    camera.position = { x: 0, y: 0, z: 50 };
    const info = must(og.load.video(world, "clips/count.ogt"));
    const screen = must(world.spawn({ name: "screen", position: { x: 0, y: 0, z: 0 } }));
    must(world.insert(screen, og.Sprite, { texture: info.texture, width: 16, height: 9 }));
    must(world.insert(screen, og.VideoPlayer, { video: info.video }));
    must(world.insert(screen, og.AudioSource, {}));
  }
});
export {
  main_default as default
};
