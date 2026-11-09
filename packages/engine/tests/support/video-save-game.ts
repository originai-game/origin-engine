// Two games for tests/video-save.test.ts. `player` has a paused player at
// TIME at half speed, looping; `plain` has none. Both count their ticks.
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

/** A time an f32 cannot hold: 2.5 + 2^-30 s. */
export const TIME = 2.5 + 2 ** -30;

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
    const r = world.insert(screen.value, og.VideoPlayer, { time: TIME, playing: false, speed: 0.5, loop: true });
    if (!r.ok) throw new Error(r.reason);
  },
});

export const plain = og.game({ tick: 30, seed: 3, systems: [Count], resources: [Ticks] });

/** The tick both saves are taken at. */
export const SAVE_AT = 30;
