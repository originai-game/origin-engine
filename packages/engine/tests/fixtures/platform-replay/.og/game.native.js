// @bun
// src/main.ts
import { og, defineResource, defineSystem, u32 } from "/home/user/workspace/repo/packages/engine/src/index.ts";
var Wallet = defineResource("Wallet", { gems: u32(), cancelled: u32(), notices: u32() });
var request = 0;
var Buy = defineSystem("Buy", {}, (_q, time, world) => {
  if (time.tick !== 10)
    return;
  const r = og.platform.purchase(world, { product: "gems" });
  if (r.ok)
    request = r.value;
});
var Credit = defineSystem("Credit", { reads: [og.PlatformAnswered, og.PlatformNotice], writes: [Wallet] }, (_q, _time, world) => {
  const w = world.resource(Wallet);
  if (!w.ok)
    return;
  for (const e of world.events(og.PlatformAnswered)) {
    if (e.request !== request)
      continue;
    if (e.ok)
      w.value.gems += 100;
    else
      w.value.cancelled += 1;
  }
  w.value.notices += world.events(og.PlatformNotice).length;
});
var main_default = og.game({ tick: 60, seed: 1, resources: [Wallet], systems: [Buy, Credit] });
export {
  main_default as default
};

//# debugId=ACF43F66F918B7C064756E2164756E21
//# sourceMappingURL=game.native.js.map
