// The game of tests/scene-write-back.test.ts: a boot scene that sets a
// resource, a second scene instantiated under a root the game spawns, and
// entities the game spawns at run time (a bolt with a spark under it). Each
// is spawned only when no scene made it already, so the world reloads from
// a written-back scene without spawning anything twice.
import { og, defineComponent, defineResource, f32, u32 } from "@origingame/engine";

const Score = defineResource("Score", { best: f32(), lives: u32() });
const Glow = defineComponent("Glow", { level: f32() });
const Charge = defineComponent("Charge", { amount: f32() });

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Score],
  start(world) {
    const room = must(og.load.scene(world, "scenes/room.ogt"));
    const hall = world.find("hall") ?? must(world.spawn({ name: "hall", position: { x: 5, y: 0, z: 0 } }));
    must(world.instantiate(room, { parent: hall }));
    if (world.find("bolt") === null) {
      const bolt = must(world.spawn({ name: "bolt", position: { x: 0.125, y: 2.3333333, z: -4 } }));
      world.insert(bolt, Charge, { amount: 0.75 });
      must(world.spawn({ name: "spark", parent: bolt, position: { x: 0, y: 0.25, z: 0 } }));
    }
  },
});
