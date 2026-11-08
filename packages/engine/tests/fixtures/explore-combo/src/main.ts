// og explore's two-step fixture: `guard` raises the guard (an action), and a
// Taunt (an intent, as a UI button sends it through og.input.intent) within
// three ticks is a counter whose power divides by the ticks since the guard
// less one, floored at 0.01 instead of 1: a taunt the very next tick sends
// Fighter.power to 1000, over its maximum of 100, which the default range
// invariant reports. Any other gap, or either input alone, is fine; the smoke
// playtest guards and taunts a second apart.
import { og, defineEvent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";

export const Fighter = defineResource("Fighter", {
  power: f32(1, { min: 0, max: 100 }),
  guardTick: u32(),
  counters: u32(),
});
export const Taunt = defineEvent("Taunt", { level: u32(0, { min: 0, max: 3 }) });

const Fight = defineSystem("Fight", { reads: [Taunt], writes: [Fighter] }, (_q, time, world) => {
  const f = world.resource(Fighter);
  if (!f.ok) return;
  const v = f.value;
  if (og.input.isActionJustPressed(world, "guard")) v.guardTick = time.tick;
  for (const taunt of world.events(Taunt)) {
    const gap = time.tick - v.guardTick;
    if (v.guardTick > 0 && gap <= 3) {
      v.counters += 1;
      og.assert.sometimes(true, "counters a taunt");
      // The bug: the floor should be 1; a taunt the tick after the guard divides by 0.01.
      v.power = Math.min(10, v.power * (1 + taunt.level)) / Math.max(gap - 1, 0.01);
    } else v.power = 1;
  }
});

export default og.game({
  tick: 60,
  resources: [Fighter],
  systems: [Fight],
  start(world) {
    og.input.defineAction(world, { name: "guard", bindings: [{ control: "KeyQ" }] });
  },
  explore: { intents: [{ event: "Taunt" }] },
});
