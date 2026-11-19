// examples/hello.ts with its Move system as ArchetypeView.addScaled: the same
// integrate over the same entities, one engine call per archetype instead of
// a JavaScript loop over them. The game-frame benchmark compares the two:
//
//   OG_BENCH_COMMANDS="spawn_wave 10000" with hello.ts and with this file.
import { og, defineSystem } from "@origingame/engine";
import { Health, Velocity } from "../../examples/hello.ts";
import type { World } from "@origingame/engine";

const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.addScaled(og.Transform, "position", Velocity, "v", time.dt);
    if (!r.ok) throw new Error(r.reason);
  }
});

function spawnWave(world: World, count: number): void {
  const rng = world.resource(og.Rng);
  if (!rng.ok) return;
  for (let i = 0; i < count; i++) {
    const e = world.spawn({ name: `enemy-${world.tick}-${i}`, position: { x: rng.value.range(-10, 10), y: 0, z: -20 } });
    if (!e.ok) return;
    world.insert(e.value, Velocity, { v: { x: 0, y: 0, z: 1 } });
    world.insert(e.value, Health);
  }
}

export default og.game({
  tick: 60,
  systems: [Move],
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0])) },
});
