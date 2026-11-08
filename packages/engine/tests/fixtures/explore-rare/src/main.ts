// og explore's rare-branch fixture: a slot machine whose reels draw from
// og.Rng every tick; `spin` stops them, so when the player presses decides the
// draw. One stop in 500 is the jackpot, whose payout has its sign wrong: the
// coins go negative, which the default range invariant (Slots.coins >= 0)
// reports. The smoke playtest spins twenty times and never hits it.
import { og, defineResource, defineSystem, f64, i32, u32 } from "@origingame/engine";

export const Slots = defineResource("Slots", {
  reel: f64(0, { min: 0, max: 1 }),
  coins: i32(50, { min: 0 }),
  spins: u32(),
});

const Reels = defineSystem("Reels", { writes: [Slots, og.Rng] }, (_q, _time, world) => {
  const s = world.resource(Slots);
  const rng = world.resource(og.Rng);
  if (!s.ok || !rng.ok) return;
  const v = s.value;
  v.reel = rng.value.next();
  if (!og.input.isActionJustPressed(world, "spin") || v.coins <= 0) return;
  v.coins -= 1;
  v.spins += 1;
  if (v.reel < 0.002) {
    og.assert.reachable("jackpot");
    // The bug: the payout is subtracted.
    v.coins -= 1000;
  } else if (v.reel < 0.45) v.coins += 2;
});

export default og.game({
  tick: 60,
  resources: [Slots],
  systems: [Reels],
  start(world) {
    og.input.defineAction(world, { name: "spin", bindings: [{ control: "Space" }] });
  },
});
