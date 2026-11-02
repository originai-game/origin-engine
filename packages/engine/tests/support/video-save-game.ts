// Two games for tests/video-save.test.ts, which restores saves an engine
// with og.VideoPlayer version 1 (an f32 `time`, registered in every world)
// wrote, and tests/support/video-save-v1.ts, which runs on that engine to
// write them. `player` has a paused player 2.5 s in at half speed, looping;
// `plain` has none. Both count their ticks. They import only the package's
// public entries, so they run on either build.
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

export const Ticks = defineResource("Ticks", { n: u32() });

const Count = defineSystem("Count", { writes: [Ticks] }, (_q, _time, world) => {
  const t = world.resource(Ticks);
  if (t.ok) t.value.n += 1;
});

export const player = og.game({
  tick: 30,
  seed: 3,
  systems: [Count],
  resources: [Ticks],
  start(world) {
    const screen = world.spawn({ name: "screen" });
    if (!screen.ok) throw new Error(screen.reason);
    const r = world.insert(screen.value, og.VideoPlayer, { time: 2.5, playing: false, speed: 0.5, loop: true });
    if (!r.ok) throw new Error(r.reason);
  },
});

export const plain = og.game({ tick: 30, seed: 3, systems: [Count], resources: [Ticks] });

/** The tick both saves are taken at. */
export const SAVE_AT = 30;
