// og explore's deep-state fixture: a corridor of 10 cells per level, walked
// with `right` and `left`; `use` at the end descends. The planted bug is on
// level 5: its vault door puts the hero one cell past the corridor, which the
// default range invariant on Dungeon.x (0..10) reports. Reaching it takes at
// least 44 presses in order; the smoke playtest walks level 1 and misses it.
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

export const Dungeon = defineResource("Dungeon", {
  x: u32(0, { min: 0, max: 10 }),
  level: u32(1, { min: 1, max: 9 }),
});

const Walk = defineSystem("Walk", { writes: [Dungeon] }, (_q, _time, world) => {
  const d = world.resource(Dungeon);
  if (!d.ok) return;
  const v = d.value;
  if (og.input.isActionJustPressed(world, "right") && v.x < 10) v.x += 1;
  if (og.input.isActionJustPressed(world, "left") && v.x > 0) v.x -= 1;
  if (og.input.isActionJustPressed(world, "use") && v.x === 10 && v.level < 9) {
    v.level += 1;
    v.x = 0;
    og.assert.sometimes(v.level >= 3, "descends to level 3");
    // The bug: level 5's vault door opens one cell past the corridor.
    if (v.level === 5) v.x = 11;
  }
});

export default og.game({
  tick: 60,
  resources: [Dungeon],
  systems: [Walk],
  start(world) {
    og.input.defineAction(world, { name: "right", bindings: [{ control: "KeyD" }] });
    og.input.defineAction(world, { name: "left", bindings: [{ control: "KeyA" }] });
    og.input.defineAction(world, { name: "use", bindings: [{ control: "KeyE" }] });
  },
});
