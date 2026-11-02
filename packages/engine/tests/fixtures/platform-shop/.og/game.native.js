// @bun
// packages/engine/tests/fixtures/platform-shop/src/main.ts
import { og, defineResource, defineSystem, u32 } from "/home/user/workspace/repo/packages/engine/src/index.ts";
var Wallet = defineResource("Wallet", { gems: u32(), cancelled: u32(), skipped: u32(), delivered: u32(), unlocked: u32() });
var asked = { entitlements: 0, buys: new Set, ads: new Set };
function deliver(world, w, p) {
  const first = og.platform.grant(world, p.purchase);
  if (!first.ok || !first.value)
    return;
  w.gems += 100 * p.quantity;
  w.delivered += 1;
  og.platform.consume(world, p.purchase);
  if (w.gems >= 100 && w.unlocked === 0) {
    w.unlocked = 1;
    og.platform.achievementUnlock(world, "first.gems");
  }
}
var Shop = defineSystem("Shop", { reads: [og.PlatformAnswered, og.PlatformNotice], writes: [Wallet, og.PlatformGrants] }, (_q, time, world) => {
  const w = world.resource(Wallet);
  if (!w.ok)
    return;
  if (time.tick === 1)
    asked.entitlements = og.platform.entitlements(world);
  for (const e of world.events(og.PlatformAnswered)) {
    const r = og.platform.result(world, e.request);
    if (!r)
      continue;
    if (e.request === asked.entitlements && r.ok) {
      for (const p of r.data.entitlements)
        deliver(world, w.value, p);
    } else if (asked.buys.delete(e.request)) {
      if (!r.ok)
        w.value.cancelled += r.error?.code === "cancelled" ? 1 : 0;
      else if (r.data.state === "completed")
        deliver(world, w.value, r.data);
    } else if (asked.ads.delete(e.request) && r.ok) {
      if (r.data.outcome === "completed")
        w.value.gems += 10;
      else
        w.value.skipped += 1;
    }
  }
  for (const n of world.events(og.PlatformNotice)) {
    const notice = og.platform.notice(world, n.notice);
    const p = notice?.data;
    if (notice?.kind === "purchase" && p?.state === "completed")
      deliver(world, w.value, p);
  }
});
var main_default = og.game({
  tick: 60,
  seed: 1,
  resources: [Wallet],
  systems: [Shop],
  commands: {
    buy(world) {
      const r = og.platform.purchase(world, { product: "gems" });
      if (r.ok)
        asked.buys.add(r.value);
    },
    ad(world) {
      const r = og.platform.showAd(world, { kind: "rewarded", placement: "shop" });
      if (r.ok)
        asked.ads.add(r.value);
    }
  }
});
export {
  Wallet,
  main_default as default
};

//# debugId=704567F86A0C5E3164756E2164756E21
//# sourceMappingURL=game.native.js.map
