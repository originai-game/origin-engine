// og.platform's services on the native host (docs/platform-asks.md #18):
// purchases and entitlements, ads, achievements, the player, friends,
// invites and share, answered by the local platform's deterministic fake.
// Outcomes come from the fake's queues; a pending purchase is settled by a
// notice (og.PlatformNotice); an ad pauses the game for a frame and its
// answer arrives after the resume; a method the host lacks is left out of
// its capabilities and answered `unavailable`; a host with no fake (a
// published page without a Player) fakes nothing; og.platform.grant grants
// a purchase once and refuses a system that did not declare the grants.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { localPlatform, memoryPersistence, type LocalFake } from "../src/platform-local.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

function start(game: unknown, fake?: LocalFake): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  r.value.attachPlatform(localPlatform(memoryPersistence(), fake));
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Res = { method: string; ok: boolean; data: Record<string, unknown>; error?: { code: string; retryable: boolean } } | null;
const result = (w: World, r: number) => og.platform.result(w, r) as unknown as Res;

const Heard = defineResource("PsHeard", { notices: u32(), last: u32() });
const Listener = defineSystem("PsListener", { writes: [Heard], reads: [og.PlatformNotice] }, (_q, _time, world) => {
  const h = must(world.resource(Heard));
  for (const e of world.events(og.PlatformNotice)) {
    h.notices += 1;
    h.last = e.notice;
  }
});

test("the fake answers purchases from its scripted outcomes, and a pending one is settled by a notice", () => {
  const fake: LocalFake = {
    products: [
      { id: "gems.100", title: "100 gems", price: { amount: 1.99, currency: "EUR" } },
      { id: "no.ads", kind: "durable" },
    ],
    outcomes: { "purchase.buy": ["cancelled", "ok", "pending"] },
  };
  const engine = start(og.game({ resources: [Heard], systems: [Listener] }), fake);
  const w = engine.world;
  expect(og.platform.supports(w, "purchase.buy")).toBeTrue();
  const products = og.platform.products(w);
  const cancelled = must(og.platform.purchase(w, { product: "gems.100" }));
  const bought = must(og.platform.purchase(w, { product: "gems.100", quantity: 3 }));
  const pending = must(og.platform.purchase(w, { product: "no.ads" }));
  const missing = must(og.platform.purchase(w, { product: "gold" }));
  engine.advance(1);
  const p = result(w, must(products))!.data as { products: Array<{ id: string; kind: string; price: { amount: number; currency: string; display: string } }> };
  expect(p.products.map((x) => [x.id, x.kind, x.price.display])).toEqual([
    ["gems.100", "consumable", "1.99 EUR"],
    ["no.ads", "durable", "0.99 USD"],
  ]);
  expect([result(w, cancelled)!.ok, result(w, cancelled)!.error?.code]).toEqual([false, "cancelled"]);
  const got = result(w, bought)!.data;
  expect(got).toEqual({ purchase: "purchase-1", product: "gems.100", quantity: 3, state: "completed" });
  expect(result(w, pending)!.data["state"]).toBe("pending");
  expect(result(w, missing)!.error?.code).toBe("not_found");

  // The pending purchase settles at a later pump, as a notice at a tick's start.
  engine.advance(2);
  const notice = og.platform.notice(w, 1) as unknown as { kind: string; data: Record<string, unknown> } | null;
  expect(notice?.kind).toBe("purchase");
  expect(notice?.data).toEqual({ purchase: "purchase-2", product: "no.ads", quantity: 1, state: "completed" });
  engine.advance(1);
  const heard = must(w.resource(Heard));
  expect([heard.notices, heard.last]).toEqual([1, 1]);

  // Entitlements: both, until the consumable is consumed; consuming twice is not_found.
  const before = og.platform.entitlements(w);
  const used = must(og.platform.consume(w, "purchase-1"));
  const twice = must(og.platform.consume(w, "purchase-1"));
  const after = og.platform.entitlements(w);
  const owned = must(og.platform.purchase(w, { product: "no.ads" }));
  engine.advance(1);
  const products_of = (r: number) => (result(w, r)!.data["entitlements"] as Array<{ product: string }>).map((e) => e.product);
  expect(products_of(before)).toEqual(["gems.100", "no.ads"]);
  expect(result(w, used)!.data).toEqual({ purchase: "purchase-1", product: "gems.100", consumed: true });
  expect(result(w, twice)!.error?.code).toBe("not_found");
  expect(products_of(after)).toEqual(["no.ads"]);
  expect(result(w, owned)!.error?.code).toBe("conflict");
});

test("an ad pauses the game for a frame and its answer arrives after the resume", () => {
  const engine = start(og.game({}), { outcomes: { "ad.show": ["skipped"], "ad.ready": ["not_ready"] } });
  const w = engine.world;
  const ready = must(og.platform.adReady(w, { kind: "rewarded" as never }));
  const skipped = must(og.platform.showAd(w, { kind: "rewarded" as never, placement: "continue" }));
  const frames: Array<{ ticks: number; paused: boolean }> = [];
  for (let i = 0; i < 4; i++) {
    const f = engine.frame(1000 / 60);
    frames.push({ ticks: f.ticks ?? 0, paused: f.paused === true });
    if (i === 0) expect(result(w, skipped)).toBeNull();
  }
  // The first frame is paused (no tick); the game resumes and the answer is applied at the next tick.
  expect(frames[0]).toEqual({ ticks: 0, paused: true });
  expect(frames.slice(1).some((f) => f.ticks > 0)).toBeTrue();
  expect(result(w, skipped)!.data).toEqual({ kind: "rewarded", placement: "continue", outcome: "skipped" });
  expect(result(w, ready)!.data["ready"]).toBeFalse();
  // The defaults: a rewarded ad completed, an interstitial shown.
  const done = must(og.platform.showAd(w, {}));
  const shown = must(og.platform.showAd(w, { kind: "interstitial" as never }));
  for (let i = 0; i < 6; i++) engine.frame(1000 / 60);
  expect(result(w, done)!.data["outcome"]).toBe("completed");
  expect(result(w, shown)!.data["outcome"]).toBe("shown");
});

test("achievements, the player, friends, invites and share answer from the fake", () => {
  const engine = start(og.game({}), {
    player: { player: "p-me", name: "Ada" },
    friends: [{ player: "p-bo", name: "Bo", online: true, playing: true }],
    invites: [{ invite: "inv-9", from: { player: "p-bo", name: "Bo" }, payload: { room: "r1" }, opened: true }],
    outcomes: { share: ["cancelled"] },
  });
  const w = engine.world;
  const half = must(og.platform.achievementProgress(w, { id: "collector", percent: 50 }));
  const lower = must(og.platform.achievementProgress(w, { id: "collector", percent: 20 }));
  const unlock = must(og.platform.achievementUnlock(w, "collector"));
  const again = must(og.platform.achievementUnlock(w, "collector"));
  const list = og.platform.achievements(w);
  const me = og.platform.player(w);
  const friends = og.platform.friends(w, { count: 5 });
  const sent = must(og.platform.inviteSend(w, { to: "p-bo", payload: { room: "r2" } }));
  const invites = og.platform.invites(w);
  const closed = must(og.platform.share(w, { text: "I cleared level 3!", path: "level=3" }));
  const shared = must(og.platform.share(w, { text: "Again", image: true }));
  engine.advance(1);
  expect(result(w, half)!.data).toEqual({ id: "collector", percent: 50, unlocked: false, new: false });
  expect(result(w, lower)!.data["percent"]).toBe(50);
  expect(result(w, unlock)!.data).toEqual({ id: "collector", percent: 100, unlocked: true, new: true });
  expect(result(w, again)!.data["new"]).toBeFalse();
  expect(result(w, list)!.data).toEqual({ achievements: [{ id: "collector", percent: 100, unlocked: true, time: null }] });
  expect(result(w, me)!.data).toEqual({ player: "p-me", name: "Ada", guest: false });
  expect(result(w, friends)!.data).toEqual({ friends: [{ player: "p-bo", name: "Bo", online: true, playing: true }] });
  expect(result(w, sent)!.data).toEqual({ invite: "invite-1", to: "p-bo" });
  expect(result(w, invites)!.data).toEqual({ invites: [{ invite: "inv-9", from: { player: "p-bo", name: "Bo" }, payload: { room: "r1" }, time: null, opened: true }] });
  expect(result(w, closed)!.error?.code).toBe("cancelled");
  expect(result(w, shared)!.data).toEqual({ shared: true, target: null });
});

test("a method the host lacks is not a capability and answers unavailable; without a fake nothing is faked", () => {
  const lacking = start(og.game({}), { unavailable: ["ad.show"] });
  expect(og.platform.supports(lacking.world, "ad.show")).toBeFalse();
  expect(og.platform.supports(lacking.world, "ad.ready")).toBeTrue();
  const ad = must(og.platform.showAd(lacking.world, {}));
  lacking.advance(1);
  const r = result(lacking.world, ad)!;
  expect([r.ok, r.error?.code, r.error?.retryable]).toEqual([false, "unavailable", true]);

  // A published page without a Player: the store answers, the services never do.
  const page = start(og.game({}));
  expect(og.platform.supports(page.world, "purchase.buy")).toBeFalse();
  expect(og.platform.supports(page.world, "store.get")).toBeTrue();
  const buy = must(og.platform.purchase(page.world, { product: "gems" }));
  page.advance(1);
  expect(result(page.world, buy)!.error?.code).toBe("unavailable");
});

const Gems = defineResource("PsGems", { gems: u32(), refused: u32() });
const Undeclared = defineSystem("PsUndeclared", { writes: [Gems] }, (_q, time, world) => {
  if (time.tick !== 1) return;
  const g = must(world.resource(Gems));
  if (!og.platform.grant(world, "purchase-1").ok) g.refused += 1;
});
const Granter = defineSystem("PsGranter", { writes: [Gems, og.PlatformGrants] }, (_q, time, world) => {
  if (time.tick < 2 || time.tick > 3) return;
  const g = must(world.resource(Gems));
  // The same purchase seen on two ticks (an answer, then its notice): granted once.
  if (must(og.platform.grant(world, "purchase-1"))) g.gems += 100;
});

test("og.platform.grant grants a purchase once, and a system must declare og.PlatformGrants", () => {
  const engine = start(og.game({ resources: [Gems], systems: [Undeclared, Granter] }), {});
  engine.advance(5);
  const g = must(engine.world.resource(Gems));
  expect([g.gems, g.refused]).toEqual([100, 1]);
  expect(og.platform.granted(engine.world, "purchase-1")).toBeTrue();
  expect(og.platform.granted(engine.world, "purchase-2")).toBeFalse();
});
