// The platform's first answers behind the play gate (src/web/settle.ts,
// examples/WEB-FINDINGS.md #1): the Worker takes the game's first tick, then
// one tick per round of answers, so the title a returning player sees behind
// the gate already shows the saved language and the slot's Continue. Run on
// the native host with the local platform answering, as the page's storage
// does; the ticks are ordinary ticks, so two runs with the same answers end
// on the same world.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { emptyDocument, localPlatform, memoryPersistence, type LocalDocument, type PlatformHost } from "../src/platform-local.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { PlatformCounter, settle } from "../src/web/settle.ts";

beforeAll(() => {
  loadNativeHost();
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** What the title shows: the language (0 English, 1 the saved Chinese), and whether Continue is on. */
const Title = defineResource("StTitle", { language: u32(), cont: u32(), settings: u32(), slot: u32(), told: u32() });

type Answer = { ok: boolean; data?: { value?: { language?: string } | null; meta?: { place?: number } | null; text?: string | null } } | null;

const TitleSystem = defineSystem("StTitleSystem", { writes: [Title] }, (_q, time, world: World) => {
  const t = must(world.resource(Title));
  if (time.tick === 0) {
    t.settings = og.platform.loadSettings(world);
    t.slot = must(og.platform.loadSlot(world, { slot: "run", metaOnly: false }));
    must(og.platform.telemetry(world, "title.shown", {}));
  }
  const s = t.settings ? (og.platform.result(world, t.settings) as unknown as Answer) : null;
  if (s) {
    t.settings = 0;
    t.language = s.data?.value?.language === "zh-CN" ? 1 : 0;
  }
  const slot = t.slot ? (og.platform.result(world, t.slot) as unknown as Answer) : null;
  if (slot) {
    t.slot = 0;
    t.cont = slot.ok && typeof slot.data?.text === "string" ? 1 + (slot.data.meta?.place ?? 0) : 0;
  }
});

const game = () => og.game({ resources: [Title], systems: [TitleSystem] });

/** A store as a returning player left it: Chinese settings and a two-part run slot. */
function saved(): LocalDocument {
  const doc = emptyDocument("You");
  const save = createEngine(game() as GameToken);
  if (!save.ok) throw new Error(save.reason);
  const host = localPlatform(memoryPersistence(doc));
  save.value.attachPlatform(host);
  must(og.platform.saveSettings(save.value.world, { language: "zh-CN" }));
  // Over one key's 64 KiB: two parts, read after the manifest.
  must(og.platform.saveSlot(save.value.world, { slot: "run", text: "x".repeat(70_000), meta: { place: 3 } }));
  // One tick an advance: the engine hands the host its calls between advances.
  for (let i = 0; i < 8; i++) save.value.advance(1);
  return host.document();
}

function boot(doc: LocalDocument, wrap: (h: PlatformHost) => PlatformHost): Engine {
  const e = createEngine(game() as GameToken);
  if (!e.ok) throw new Error(e.reason);
  e.value.attachPlatform(wrap(localPlatform(memoryPersistence(doc))));
  return e.value;
}

test("behind the gate, the first tick's reads are applied before the first frame: the saved language and the slot's Continue", async () => {
  const doc = saved();
  const counter = new PlatformCounter();
  const engine = boot(doc, (h) => counter.wrap(h));
  must(engine.advance(1)); // the primed tick: tick 0 asks
  const title = must(engine.world.resource(Title));
  // One tick alone shows the defaults (the finding).
  expect([title.language, title.cont]).toEqual([0, 0]);
  expect(counter.fresh).toBeTrue();
  const ticks = await settle(counter, () => engine.advance(1), { pending: () => engine.platformPending(), until: performance.now() + 2000 });
  const after = must(engine.world.resource(Title));
  expect([after.language, after.cont]).toEqual([1, 4]);
  // The settings on the first settling tick; the slot's parts after its manifest on the next.
  expect(ticks).toBe(2);
  expect(counter.waiting).toBe(0);
  // Telemetry is never answered and never waited for.
  expect(og.platform.info(engine.world).pending).toBe(0);
});

test("a fresh player's settling takes one tick and ends; the same answers give the same world", async () => {
  const hash = async (doc: LocalDocument) => {
    const counter = new PlatformCounter();
    const engine = boot(doc, (h) => counter.wrap(h));
    must(engine.advance(1));
    const ticks = await settle(counter, () => engine.advance(1), { pending: () => engine.platformPending(), until: performance.now() + 2000 });
    return { ticks, tick: engine.world.tick, hash: engine.hash() };
  };
  const fresh = await hash(emptyDocument("You"));
  expect(fresh.ticks).toBe(1);
  const doc = saved();
  const a = await hash(structuredClone(doc));
  const b = await hash(structuredClone(doc));
  expect(a).toEqual(b);
});

test("an answer that has not come by the deadline is applied by a later settling tick; play stops the settling", async () => {
  const doc = saved();
  const counter = new PlatformCounter();
  // A host that answers only when told (a slow play origin).
  const held: Array<() => void> = [];
  const engine = boot(doc, (h) => counter.wrap({ host: h.host, capabilities: h.capabilities, handle: (calls, answer) => held.push(() => h.handle(calls, answer)) }));
  must(engine.advance(1));
  expect(counter.waiting).toBe(2);
  // The first frame's wait runs out: nothing applied, no tick taken.
  expect(await settle(counter, () => engine.advance(1), { pending: () => engine.platformPending(), until: performance.now() + 20 })).toBe(0);
  let drawn = 0;
  const later = settle(counter, () => engine.advance(1), { pending: () => engine.platformPending(), until: performance.now() + 2000, drawn: () => drawn++ });
  for (const answer of held.splice(0)) answer();
  // The slot's parts are asked on the settling tick; answer them as they come.
  const pump = setInterval(() => {
    for (const answer of held.splice(0)) answer();
  }, 5);
  const ticks = await later;
  clearInterval(pump);
  expect(ticks).toBe(2);
  expect(drawn).toBe(2);
  expect(must(engine.world.resource(Title)).language).toBe(1);
  // Play began: the gate's tap stops the settling before any tick.
  const counter2 = new PlatformCounter();
  const engine2 = boot(doc, (h) => counter2.wrap(h));
  must(engine2.advance(1));
  expect(await settle(counter2, () => engine2.advance(1), { pending: () => engine2.platformPending(), until: performance.now() + 2000, stop: () => true })).toBe(0);
});
