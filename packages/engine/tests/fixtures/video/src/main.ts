// A cooked video on a sprite that fills the view, its sound on the same
// entity (og-video.test.ts): og.load.video in start, og.VideoPlayer plays it.
// `pair` halves it and adds a second player of the same video half a second ahead on
// the right; Show puts each player's own texture (VideoPlayer.texture) on
// its sprite, so the two halves show different frames.
import { og, defineSystem } from "@origingame/engine";
import type { Video, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Each sprite shows its player's texture (written by og.video). */
export const Show = defineSystem("Show", { reads: [og.VideoPlayer], writes: [og.Sprite] }, (q) => {
  for (const a of q.archetypes()) {
    const from = a.column(og.VideoPlayer).texture;
    const to = a.column(og.Sprite).texture;
    for (let k = 0; k < a.count; k++) {
      if (from[2 * k] === 0 && from[2 * k + 1] === 0) continue;
      to[2 * k] = from[2 * k]!;
      to[2 * k + 1] = from[2 * k + 1]!;
    }
  }
});

let video = 0 as unknown as Video;

export default og.game({
  tick: 30,
  systems: [Show],
  start(world) {
    // An orthographic view 9 units tall, the screen filling it.
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = 9;
    camera.position = { x: 0, y: 0, z: 50 };
    const info = must(og.load.video(world, "clips/count.ogt"));
    video = info.video;
    const screen = must(world.spawn({ name: "screen", position: { x: 0, y: 0, z: 0 } }));
    must(world.insert(screen, og.Sprite, { texture: info.texture, width: 16, height: 9 }));
    must(world.insert(screen, og.VideoPlayer, { video: info.video }));
    must(world.insert(screen, og.AudioSource, {}));
  },
  commands: {
    pair: (world: World) => {
      const screen = world.find("screen")!;
      must(world.set(screen, og.Transform, { position: { x: -4, y: 0, z: 0 } }) as never);
      must(world.set(screen, og.Sprite, { width: 8, height: 4.5 }) as never);
      const time = must(world.get(screen, og.VideoPlayer)).time;
      const second = must(world.spawn({ name: "screen2", position: { x: 4, y: 0, z: 0 } }));
      must(world.insert(second, og.Sprite, { width: 8, height: 4.5 }));
      must(world.insert(second, og.VideoPlayer, { video, time: time + 0.5 }));
    },
  },
});
