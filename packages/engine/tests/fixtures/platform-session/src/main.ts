// Two peers in lockstep, each buying the same product at tick 100 from its
// own platform, which answers one `ok` and the other `cancelled`
// (--platform-fake). Credit applies the answer to the shared Wallet: in a
// running session it hears nothing (a platform answer is this peer's
// alone), so the worlds keep one hash; Toast writes no shared state and
// hears it (rooms.test.ts).
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";

/** Shared state: hashed and compared across the room. */
const Wallet = defineResource("Wallet", { gems: u32(), answers: u32() });
let request = 0;

const Buy = defineSystem("Buy", {}, (_q, time, world) => {
  if (time.tick !== 100) return;
  const r = og.platform.purchase(world, { product: "gems" });
  if (r.ok) request = r.value;
});

// The bug the engine prevents: a peer's own answer written into the shared world.
const Credit = defineSystem("Credit", { reads: [og.PlatformAnswered], writes: [Wallet] }, (_q, _time, world) => {
  const w = world.resource(Wallet);
  if (!w.ok) return;
  for (const e of world.events(og.PlatformAnswered)) {
    w.value.answers += 1;
    if (e.ok) w.value.gems += 100;
  }
  if (request !== 0 && og.platform.result(world, request)?.ok) w.value.gems += 1;
});

// What a session-local reader may do: show it (a toast), here a count only this peer keeps.
let heard = 0;
/** Toast's count, copied in by the `heard` action between ticks: one answer each, ok or cancelled, so equal on both peers. */
const Heard = defineResource("Heard", { count: u32() });
const Toast = defineSystem("Toast", { reads: [og.PlatformAnswered] }, (_q, _time, world) => {
  heard += world.events(og.PlatformAnswered).length;
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Wallet, Heard],
  systems: [Buy, Credit, Toast],
  commands: {
    heard(world) {
      const h = world.resource(Heard);
      if (h.ok) h.value.count = heard;
    },
  },
  start(world) {
    const lock = world.resource(og.Lockstep);
    if (!lock.ok) throw new Error(lock.reason);
    lock.value.enabled = true;
    lock.value.players = 2;
    lock.value.hashEvery = 10;
  },
});

