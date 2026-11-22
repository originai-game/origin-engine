// Gameplay only: the engine loads, starts and draws the flow
// (project.ogt `flow`). Two games run while their segments hold:
// - the level: `push` (D) rolls the ball; off the set's edge it is won,
//   and `give-up` (Q) loses it;
// - the arcade: the puck falls in 2D; once it rests on the floor it has
//   landed.
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";

function held(world: World, node: string) {
  const flow = og.flow.current(world);
  if (flow == null) return null;
  const v = og.flow.view(world, flow);
  return v && v.held && v.node === node ? flow : null;
}

function y(world: World, name: string): number | null {
  const e = world.find(name);
  if (e == null) return null;
  const t = world.get(e, og.Transform);
  return t.ok ? t.value.position.y : null;
}

export const Level = defineSystem("Level", { reads: [og.Transform] }, (_q, _time, world) => {
  const flow = held(world, "level");
  const ball = world.find("ball");
  if (flow == null || ball == null) return;
  if (og.input.isActionJustPressed(world, "give-up")) {
    og.flow.advance(world, flow, "lost");
  } else if ((y(world, "ball") ?? 0) < -2) {
    og.flow.advance(world, flow, "won");
  } else if (og.input.isActionDown(world, "push")) {
    og.physics.applyImpulse(world, ball, { x: 0.05, y: 0, z: 0 });
  }
});

export const Arcade = defineSystem("Arcade", { reads: [og.Transform, og.Velocity] }, (_q, _time, world) => {
  const flow = held(world, "arcade");
  const puck = world.find("puck");
  if (flow == null || puck == null) return;
  const v = world.get(puck, og.Velocity);
  const at = y(world, "puck") ?? 3;
  if (at < 0.45 && v.ok && Math.abs(v.value.linear.y) < 0.01) og.flow.advance(world, flow, "landed");
});

export default og.game({
  systems: [Level, Arcade],
  start(world) {
    og.input.defineAction(world, { name: "push", bindings: [{ control: "KeyD" }] });
    og.input.defineAction(world, { name: "give-up", bindings: [{ control: "KeyQ" }] });
  },
});
