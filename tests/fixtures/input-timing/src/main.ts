// Two actions (README.md): `tap` (Space) records when in its tick each
// press happened, and `jump` (KeyJ, buffered 6 ticks) is consumed on the
// ticks a character would land, LANDINGS.
//
//   og run tests/fixtures/input-timing --headless --commands "1.5:key_down:KeyJ;1.6:quit"
import { og, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The ticks the game can act on a jump; bit i of `taken` is LANDINGS[i]. */
export const LANDINGS = [94, 95, 127];

const Timing = defineResource("Timing", { taps: u32(), first: f32(), second: f32(), taken: u32(), jumps: u32() });

const Watch = defineSystem("Watch", { writes: [Timing] }, (_q, time, world) => {
  const t = must(world.resource(Timing));
  const tap = og.input.action(world, "tap");
  if (tap?.justPressed) {
    if (t.taps === 0) t.first = tap.pressedAt;
    else t.second = tap.pressedAt;
    t.taps += 1;
    og.assert.always(Math.abs(tap.pressedAt) <= time.dt, "pressedAt lies within a tick of its tick", { at: tap.pressedAt });
  }
  const landing = LANDINGS.indexOf(time.tick);
  if (landing >= 0 && og.input.consume(world, "jump")) {
    t.taken |= 1 << landing;
    t.jumps += 1;
    og.assert.reachable("a buffered jump is taken");
    og.assert.always(!og.input.consume(world, "jump"), "a press is taken once");
  }
});

function start(world: World): void {
  must(og.input.defineAction(world, { name: "tap", bindings: [{ control: "Space" }] }));
  must(og.input.defineAction(world, { name: "jump", bindings: [{ control: "KeyJ" }], bufferTicks: 6 }));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Timing],
  systems: [Watch],
  start,
  facts: (world) => {
    const t = world.resource(Timing);
    return t.ok ? { taps: t.value.taps, first: t.value.first, second: t.value.second, taken: t.value.taken, jumps: t.value.jumps } : {};
  },
});
