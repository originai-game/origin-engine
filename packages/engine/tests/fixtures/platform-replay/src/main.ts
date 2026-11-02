// A game whose world follows its platform's answers and notices: it buys
// gems at tick 10 (credited only when the answer is ok) and counts the
// notices it hears. platform-replay.test.ts records it with a fake that
// cancels the purchase and sends a notice, and replays it with no fake:
// the trace's journal must answer as the recording's platform did.
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

const Wallet = defineResource("Wallet", { gems: u32(), cancelled: u32(), notices: u32() });
let request = 0;

const Buy = defineSystem("Buy", {}, (_q, time, world) => {
  if (time.tick !== 10) return;
  const r = og.platform.purchase(world, { product: "gems" });
  if (r.ok) request = r.value;
});

const Credit = defineSystem("Credit", { reads: [og.PlatformAnswered, og.PlatformNotice], writes: [Wallet] }, (_q, _time, world) => {
  const w = world.resource(Wallet);
  if (!w.ok) return;
  for (const e of world.events(og.PlatformAnswered)) {
    if (e.request !== request) continue;
    if (e.ok) w.value.gems += 100;
    else w.value.cancelled += 1;
  }
  w.value.notices += world.events(og.PlatformNotice).length;
});

export default og.game({ tick: 60, seed: 1, resources: [Wallet], systems: [Buy, Credit] });
