// The design's hello world (docs/design.md §3.2), written against the
// generated declarations. `bun run typecheck` in packages/engine checks it.
import { og, Vector3, Quaternion, defineComponent, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const Health = defineComponent("Health", { hp: f32(100, { min: 0, max: 1000, tunable: true }), max: f32(100) });
export const Velocity = defineComponent("Velocity", { v: Vector3() });
export const Spin = defineComponent("Spin", { rotation: Quaternion(), turns: u32() });

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {            // one call per archetype
    const p = a.column(og.Transform).position; // Float32Array, stride 3
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count; i++) {
      p[i*3] += v[i*3] * time.dt; p[i*3+1] += v[i*3+1] * time.dt; p[i*3+2] += v[i*3+2] * time.dt;
    }
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
  scenes: { main: "scenes/main.ogt" },
  systems: [Move],
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0])) },  // registered into the command channel
});
