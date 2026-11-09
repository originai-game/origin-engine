// A shop on og.platform's services (docs/platform-asks.md #19), for the
// playtests under tests/: it asks for its entitlements at start and
// delivers any consumable not yet delivered (a purchase a quit cut short),
// buys gems on `buy`, grants each purchase once (og.platform.grant) and then
// consumes it, credits a pending purchase when its notice settles it, shows
// a rewarded ad on `ad` and pays only a completed one, and unlocks an
// achievement at 100 gems. Entitlements are the host's word, never a save.
import { og, defineResource, defineSystem, list, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const Wallet = defineResource("Wallet", { gems: u32(), cancelled: u32(), skipped: u32(), delivered: u32(), unlocked: u32() });

/** Requests in flight, by what the game asked: world state, so a save, a restore or a replay keeps them. */
const Asked = defineResource("Asked", { entitlements: u32(), buys: list(u32(), 64), ads: list(u32(), 64) });

/** Takes `request` off the list it was asked on; whether it was there. */
function settle(asked: { buys: readonly number[]; ads: readonly number[] }, key: "buys" | "ads", request: number): boolean {
  if (!asked[key].includes(request)) return false;
  asked[key] = asked[key].filter((r) => r !== request);
  return true;
}

type Purchase = { purchase: string; product: string; quantity: number; state?: string };

/** Grants a purchase once: gems, then consume (the host drops it from the entitlements). */
function deliver(world: World, w: { gems: number; delivered: number; unlocked: number }, p: Purchase): void {
  const first = og.platform.grant(world, p.purchase);
  if (!first.ok || !first.value) return;
  w.gems += 100 * p.quantity;
  w.delivered += 1;
  og.platform.consume(world, p.purchase);
  if (w.gems >= 100 && w.unlocked === 0) {
    w.unlocked = 1;
    og.platform.achievementUnlock(world, "first.gems");
  }
}

const Shop = defineSystem("Shop", { reads: [og.PlatformAnswered, og.PlatformNotice], writes: [Wallet, Asked, og.PlatformGrants] }, (_q, time, world) => {
  const w = world.resource(Wallet);
  const a = world.resource(Asked);
  if (!w.ok || !a.ok) return;
  const asked = a.value;
  if (time.tick === 1) asked.entitlements = og.platform.entitlements(world);
  for (const e of world.events(og.PlatformAnswered)) {
    const r = og.platform.result(world, e.request);
    if (!r) continue;
    if (e.request === asked.entitlements && r.ok) {
      for (const p of (r.data as { entitlements: Purchase[] }).entitlements) deliver(world, w.value, p);
    } else if (settle(asked, "buys", e.request)) {
      if (!r.ok) w.value.cancelled += r.error?.code === "cancelled" ? 1 : 0;
      else if ((r.data as Purchase).state === "completed") deliver(world, w.value, r.data as Purchase);
    } else if (settle(asked, "ads", e.request) && r.ok) {
      if ((r.data as { outcome: string }).outcome === "completed") w.value.gems += 10;
      else w.value.skipped += 1;
    }
  }
  for (const n of world.events(og.PlatformNotice)) {
    const notice = og.platform.notice(world, n.notice);
    const p = notice?.data as Purchase | undefined;
    if (notice?.kind === "purchase" && p?.state === "completed") deliver(world, w.value, p);
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Wallet, Asked],
  systems: [Shop],
  commands: {
    buy(world) {
      const r = og.platform.purchase(world, { product: "gems" });
      const a = world.resource(Asked);
      if (r.ok && a.ok) a.value.buys = [...a.value.buys, r.value];
    },
    ad(world) {
      const r = og.platform.showAd(world, { kind: "rewarded" as never, placement: "shop" });
      const a = world.resource(Asked);
      if (r.ok && a.ok) a.value.ads = [...a.value.ads, r.value];
    },
  },
});
