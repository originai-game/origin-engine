// @bun
// packages/engine/tests/fixtures/platform-session/src/main.ts
import { og, defineResource, defineSystem, u32 } from "/Volumes/app/origin-engine-lane-27/packages/engine/src/index.ts";
var Wallet = defineResource("Wallet", { gems: u32(), answers: u32() });
var request = 0;
var Buy = defineSystem("Buy", {}, (_q, time, world) => {
  if (time.tick !== 100)
    return;
  const r = og.platform.purchase(world, { product: "gems" });
  if (r.ok)
    request = r.value;
});
var Credit = defineSystem("Credit", { reads: [og.PlatformAnswered], writes: [Wallet] }, (_q, _time, world) => {
  const w = world.resource(Wallet);
  if (!w.ok)
    return;
  for (const e of world.events(og.PlatformAnswered)) {
    w.value.answers += 1;
    if (e.ok)
      w.value.gems += 100;
  }
  if (request !== 0 && og.platform.result(world, request)?.ok)
    w.value.gems += 1;
});
var heard = 0;
var Heard = defineResource("Heard", { count: u32() });
var Toast = defineSystem("Toast", { reads: [og.PlatformAnswered] }, (_q, _time, world) => {
  heard += world.events(og.PlatformAnswered).length;
});
var main_default = og.game({
  tick: 60,
  seed: 1,
  resources: [Wallet, Heard],
  systems: [Buy, Credit, Toast],
  commands: {
    heard(world) {
      const h = world.resource(Heard);
      if (h.ok)
        h.value.count = heard;
    }
  },
  start(world) {
    const lock = world.resource(og.Lockstep);
    if (!lock.ok)
      throw new Error(lock.reason);
    lock.value.enabled = true;
    lock.value.players = 2;
    lock.value.hashEvery = 10;
  }
});
export {
  main_default as default
};

//# debugId=174F6467D88C0A0B64756E2164756E21
//# sourceMappingURL=game.native.js.map
