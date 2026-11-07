// The game the committed old saves were taken from (tests/save-versions.test.ts):
// written against the API every release since 0.7.7 has, so the same file
// runs on the engine that wrote a save and on this one. make.ts runs it on
// a published engine and writes `<version>.save` beside it.
//
// It keeps what an engine upgrade could break in a player's save: game
// components and a resource, engine components (og.Transform, og.Parent,
// a physics body and its collider), og.Rng draws, names, and the UI's
// state block (a button, a slider, a toast).
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const Wallet = defineComponent("Wallet", { gold: u32(), rate: f32(1) });
export const Run = defineResource("Run", { score: u32(), volume: f32(0.5), presses: u32() });

const Earn = defineSystem("Earn", { reads: [Wallet], writes: [Run, og.Rng] }, (q, _time, world) => {
  const run = world.resource(Run);
  const rng = world.resource(og.Rng);
  if (!run.ok || !rng.ok) return;
  for (const a of q.archetypes()) {
    const w = a.column(Wallet);
    for (let i = 0; i < a.count; i++) run.value.score += w.gold[i]! + rng.value.integer(0, 2);
  }
});

const Menu = defineSystem("Menu", { writes: [Run] }, (_q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok) return;
  if (og.ui.button("Play", "Play", {}).clicked) run.value.presses += 1;
  const v = og.ui.valueSlider("Volume", run.value.volume, { min: 0, max: 1 });
  run.value.volume = v.value;
});

export function start(world: World): void {
  for (let i = 0; i < 12; i++) {
    const e = world.spawn({ name: `coin-${i}`, position: { x: i, y: 1 + i * 0.5, z: 0 } });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Wallet, { gold: 10 * i, rate: 1 + i / 4 });
    if (i % 3 === 0) {
      world.insert(e.value, og.RigidBody, {});
      world.insert(e.value, og.Collider, { shape: 1, radius: 0.25 });
    }
  }
  const floor = world.spawn({ name: "floor" });
  if (!floor.ok) throw new Error(floor.reason);
  world.insert(floor.value, og.RigidBody, { kind: 1 });
  world.insert(floor.value, og.Collider, { size: { x: 40, y: 0.2, z: 4 } });
  const child = world.spawn({ name: "badge", parent: world.find("coin-3")!, position: { x: 0, y: 0.5, z: 0 } });
  if (!child.ok) throw new Error(child.reason);
  og.ui.toast(world, "Welcome back", {});
}

export default og.game({ tick: 60, seed: 7, systems: [Earn, Menu], resources: [Run], start });
