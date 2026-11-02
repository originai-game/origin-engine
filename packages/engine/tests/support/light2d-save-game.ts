// Two games for tests/light2d-save.test.ts, which restores saves written by
// an engine from before R11 (og.Lighting2D, og.Light2D and og.Occluder2D at
// version 1, registered in every world), and tests/support/light2d-save-v1.ts,
// which runs on that engine to write them. `lit` turns 2D lighting on with a
// lamp and a wall; `plain` has none. Both count their ticks. They import
// only the package's public entries, so they run on either build.
import { og, defineResource, defineSystem, u32, type World } from "@origingame/engine";

export const Ticks = defineResource("Ticks", { n: u32() });

const Count = defineSystem("Count", { writes: [Ticks] }, (_q, _time, world) => {
  const t = world.resource(Ticks);
  if (t.ok) t.value.n += 1;
});

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

export const lit = og.game({
  tick: 30,
  seed: 3,
  systems: [Count],
  resources: [Ticks],
  start(world: World) {
    const l = must(world.resource(og.Lighting2D));
    l.enabled = true;
    l.ambient = { r: 0.2, g: 0.25, b: 0.3 };
    l.maxLayer = 40;
    const lamp = must(world.spawn({ name: "lamp", position: { x: -3, y: 1, z: 0 } }));
    must(world.insert(lamp, og.Light2D, { radius: 6, intensity: 2, falloff: 1.5, sourceRadius: 0.5, castShadows: false }));
    const wall = must(world.spawn({ name: "wall", position: { x: 1, y: 0, z: 0 } }));
    must(world.insert(wall, og.Occluder2D, { width: 2, height: 0.5 }));
  },
});

export const plain = og.game({ tick: 30, seed: 3, systems: [Count], resources: [Ticks] });

/** The tick both saves are taken at. */
export const SAVE_AT = 30;
