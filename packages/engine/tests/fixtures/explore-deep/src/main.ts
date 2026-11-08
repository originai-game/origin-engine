// og explore's deep-state fixture: a corridor of 5 cells per level, walked
// with `right` and `left`; `use` at the end descends. The planted bug is on
// level 3: its vault door puts the hero one cell past the corridor, which the
// default range invariant on Dungeon.x (0..5) reports. Reaching it takes at
// least 12 presses in order (two levels of five steps and a descent); the
// smoke playtest walks level 1 and misses it.
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

export const Dungeon = defineResource("Dungeon", {
  x: u32(0, { min: 0, max: 5 }),
  level: u32(1, { min: 1, max: 9 }),
});

const Walk = defineSystem("Walk", { writes: [Dungeon] }, (_q, _time, world) => {
  const d = world.resource(Dungeon);
  if (!d.ok) return;
  const v = d.value;
  if (og.input.isActionJustPressed(world, "right") && v.x < 5) v.x += 1;
  if (og.input.isActionJustPressed(world, "left") && v.x > 0) v.x -= 1;
  if (og.input.isActionJustPressed(world, "use") && v.x === 5 && v.level < 9) {
    v.level += 1;
    v.x = 0;
    og.assert.sometimes(v.level >= 3, "descends to level 3");
    // The bug: level 3's vault door opens one cell past the corridor.
    if (v.level === 3) v.x = 6;
  }
});

export default og.game({
  tick: 60,
  resources: [Dungeon],
  systems: [Walk],
  // Where the hero is matters, not when: no time slices in the search's cells.
  explore: { ignore: ["@tick"] },
  start(world) {
    og.input.defineAction(world, { name: "right", bindings: [{ control: "KeyD" }] });
    og.input.defineAction(world, { name: "left", bindings: [{ control: "KeyA" }] });
    og.input.defineAction(world, { name: "use", bindings: [{ control: "KeyE" }] });
  },
});
