// A cooked video on a sprite that fills the view, its sound on the same
// entity (og-video.test.ts): og.load.video in start, og.VideoPlayer plays it.
import { og } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  tick: 30,
  start(world) {
    // An orthographic view 9 units tall, the screen filling it.
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = 9;
    camera.position = { x: 0, y: 0, z: 50 };
    const info = must(og.load.video(world, "clips/count.ogt"));
    const screen = must(world.spawn({ name: "screen", position: { x: 0, y: 0, z: 0 } }));
    must(world.insert(screen, og.Sprite, { texture: info.texture, width: 16, height: 9 }));
    must(world.insert(screen, og.VideoPlayer, { video: info.video }));
    must(world.insert(screen, og.AudioSource, {}));
  },
});
