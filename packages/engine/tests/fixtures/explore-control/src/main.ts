// og explore's no-bug control: explore-deep's corridor without the vault's
// bug. A search over it must report no violation.
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
  }
  og.assert.always(v.x <= 10, "the hero stays in the corridor", { x: v.x });
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
