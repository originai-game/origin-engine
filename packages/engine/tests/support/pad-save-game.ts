// A game whose state follows pad 1: what tests/gamepad-save.test.ts restores
// from a save a four-pad build wrote (og.Gamepad version 1), and what
// tests/support/pad-save-v1.ts runs on that engine to write it. It imports
// only the package's public entries, so it runs on either build.
import { og, defineResource, defineSystem, f32, u32 } from "@origingame/engine";

export const PadRun = defineResource("PadRun", { presses: u32(), x: f32(), connected: u32() });

const PadFollow = defineSystem("PadFollow", { reads: [og.Gamepad], writes: [PadRun] }, (_q, time, world) => {
  const pads = world.resource(og.Gamepad);
  const run = world.resource(PadRun);
  if (!pads.ok || !run.ok) return;
  if (pads.value.pressed("a", 1)) run.value.presses += 1;
  run.value.x += pads.value.axis("leftX", 1) * time.dt;
  run.value.connected = pads.value.connected(1) ? 1 : 0;
});

export const game = og.game({ tick: 60, seed: 5, systems: [PadFollow], resources: [PadRun] });

/** Pad 1's input on tick `t` of the script: A tapped every 10 ticks, the stick pushed right after tick 20. */
export function padAt(t: number): { kind: "gamepad"; pad: number; buttons: string[]; axes: number[] } {
  return { kind: "gamepad", pad: 1, buttons: t % 10 === 0 ? ["a"] : [], axes: [t >= 20 ? 0.5 : 0, 0, 0, 0] };
}

/** The ticks the save is taken at, and that the world goes on for after it. */
export const SAVE_AT = 30;
export const AFTER = 20;
