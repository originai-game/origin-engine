// The ECS's reactivity and spatial queries through the API on the native
// host: change filters (changed, added, removed) across ticks and across a
// save and restore, hook order, state machines entering and leaving once,
// run conditions, world.children, and world.spatial against brute force.
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineHook, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

const Hp = defineComponent("RxHp", { hp: u32(10) });
const Tag = defineComponent("RxTag", { n: u32() });

let seen: string[] = [];

const tagsOf = (q: { archetypes(): readonly { rows: Uint32Array; column(c: typeof Tag): { n: Uint32Array } }[] }) => {
  const out: number[] = [];
  for (const a of q.archetypes()) {
    const n = a.column(Tag).n;
    for (const r of a.rows) out.push(n[r]!);
  }
  return out.sort((x, y) => x - y).join(",");
};

// Hurts tag 1 on even ticks; rewrites tag 2's value unchanged; spawns one
// entity per tick from tick 3; despawns tag 1 at tick 5.
const Damage = defineSystem("RxDamage", { reads: [Tag], writes: [Hp, og.Transform] }, (q, time, world) => {
  for (const a of q.archetypes()) {
    const n = a.column(Tag).n;
    const hp = a.column(Hp).hp;
    for (let i = 0; i < a.count; i++) {
      if (n[i] === 1 && time.tick % 2 === 0) hp[i] = hp[i]! - 1;
      if (n[i] === 2) hp[i] = hp[i]!;
      if (n[i] === 1 && time.tick === 5) world.despawn(a.entity(i));
    }
  }
  if (time.tick >= 3) {
    const e = world.spawn({});
    if (!e.ok) throw new Error(e.reason);
  }
});

const Watch = defineSystem("RxWatch", { reads: [Tag], changed: [Hp] }, (q, time, world) => {
  const removed = world.removed(Hp);
  if (!removed.ok) throw new Error(removed.reason);
  seen.push(`${time.tick} changed ${tagsOf(q as never)} removed ${removed.value.length}`);
});

const tagAll = (world: World) => {
  for (let n = 1; n <= 2; n++) {
    const e = world.spawn({ name: `rx-${n}` });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Tag, { n });
    world.insert(e.value, Hp);
  }
};

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

beforeAll(() => {
  loadNativeHost();
});

test("a changed filter sees exactly the changes since the system last ran", () => {
  seen = [];
  const engine = start(og.game({ systems: [Damage, Watch], start: tagAll }));
  expect(engine.advance(7).ok).toBeTrue();
  expect(seen).toEqual([
    "0 changed 1,2 removed 0",
    "1 changed  removed 0",
    "2 changed 1 removed 0",
    "3 changed  removed 0",
    "4 changed 1 removed 0",
    "5 changed  removed 1",
    "6 changed  removed 0",
  ]);
});

test("change filters and the hash go on alike after a save and restore", () => {
  for (const at of [0, 2, 5]) {
    seen = [];
    const engine = start(og.game({ systems: [Damage, Watch], start: tagAll }));
    expect(engine.advance(at).ok).toBeTrue();
    const save = engine.world.snapshot();
    if (!save.ok) throw new Error(save.reason);
    const hash = engine.hash();
    expect(engine.advance(4).ok).toBeTrue();
    const straight = seen.slice(at);
    const after = engine.hash();
    expect(engine.world.restore(save.value).ok).toBeTrue();
    expect(engine.hash()).toBe(hash);
    seen = [];
    expect(engine.advance(4).ok).toBeTrue();
    expect(seen).toEqual(straight);
    expect(engine.hash()).toBe(after);
  }
});

let hookLog: string[] = [];
const OnHp = defineHook("RxOnHp", Hp, { on: "add", reads: [Hp, Tag], writes: [Tag] }, (world, entity) => {
  const hp = world.get(entity, Hp);
  hookLog.push(`add ${hp.ok ? (hp.value as { hp: number }).hp : "?"}`);
  if (!world.has(entity, Tag)) world.insert(entity, Tag, { n: 9 });
});
const OnTag = defineHook("RxOnTag", Tag, { on: "add", reads: [Tag] }, (world, entity) => {
  const t = world.get(entity, Tag);
  hookLog.push(`tag ${t.ok ? (t.value as { n: number }).n : "?"}`);
});
const OnHpGone = defineHook("RxOnHpGone", Hp, { on: "remove", reads: [Hp] }, (world, entity) => {
  const hp = world.get(entity, Hp);
  hookLog.push(`gone ${hp.ok ? (hp.value as { hp: number }).hp : "?"}`);
});

test("hooks run in GameDesc.hooks order, nest, and read a removed component before it goes", () => {
  hookLog = [];
  const engine = start(og.game({ hooks: [OnHp, OnTag, OnHpGone] }));
  const w = engine.world;
  const e = w.spawn({});
  if (!e.ok) throw new Error(e.reason);
  expect(w.insert(e.value, Hp, { hp: 4 }).ok).toBeTrue();
  expect(hookLog).toEqual(["add 4", "tag 9"]);
  expect(w.remove(e.value, Hp).ok).toBeTrue();
  expect(w.despawn(e.value).ok).toBeTrue();
  expect(hookLog).toEqual(["add 4", "tag 9", "gone 4"]);
});

let modeLog: string[] = [];
const log = (s: string) => (_q: unknown, time: { tick: number }) => void modeLog.push(`${time.tick} ${s}`);
const EnterPlay = defineSystem("RxEnterPlay", { state: "Mode", value: "play", when: "enter" }, log("enter play"));
const ExitMenu = defineSystem("RxExitMenu", { state: "Mode", value: "menu", when: "exit" }, log("exit menu"));
const InMenu = defineSystem("RxInMenu", { state: "Mode", value: "menu" }, (_q, time, world) => {
  modeLog.push(`${time.tick} menu`);
  if (time.tick === 1) world.setState("Mode", "play");
});
const Odd = defineSystem("RxOdd", { runIf: (world: World) => world.state("Mode") === "play" && world.tick % 2 === 1 }, log("odd"));

test("a state transition runs its exit and enter systems once, and run conditions gate", () => {
  modeLog = [];
  const engine = start(og.game({ states: [{ name: "Mode", values: ["menu", "play"] }], systems: [EnterPlay, ExitMenu, InMenu, Odd] }));
  expect(engine.advance(6).ok).toBeTrue();
  expect(modeLog).toEqual(["0 menu", "1 menu", "2 exit menu", "2 enter play", "3 odd", "5 odd"]);
  expect(engine.world.state("Mode")).toBe("play");
  expect(engine.world.setState("Mode", "lobby").ok).toBeFalse();
  expect(engine.world.state("Nope")).toBeNull();
});

test("world.children lists an entity's children in entity order", () => {
  const engine = start(og.game({}));
  const w = engine.world;
  const root = w.spawn({});
  if (!root.ok) throw new Error(root.reason);
  const kids = [0, 1, 2].map(() => {
    const k = w.spawn({ parent: root.value });
    if (!k.ok) throw new Error(k.reason);
    return k.value;
  });
  w.spawn({ parent: kids[0] });
  expect(w.children(root.value)).toEqual(kids);
  expect(w.children(kids[1]!)).toEqual([]);
});

test("world.spatial answers as brute force, nearest first, ties in entity order", () => {
  const engine = start(og.game({}));
  const w = engine.world;
  const points: { e: number; x: number; y: number; z: number }[] = [];
  let s = 7;
  const rand = () => ((s = (s * 1103515245 + 12345) % 2147483648) / 2147483648) * 20 - 10;
  for (let i = 0; i < 200; i++) {
    const p = { x: Math.round(rand()), y: Math.round(rand()), z: i % 3 === 0 ? 0 : Math.round(rand()) };
    const e = w.spawn({ position: p });
    if (!e.ok) throw new Error(e.reason);
    points.push({ e: e.value, x: Math.fround(p.x), y: Math.fround(p.y), z: Math.fround(p.z) });
  }
  const index = w.spatial({});
  if (!index.ok) throw new Error(index.reason);
  expect(index.value.count).toBe(200);
  for (let k = 0; k < 20; k++) {
    const c = { x: Math.round(rand()), y: Math.round(rand()), z: Math.round(rand()) };
    const want = points
      .map((p) => ({ entity: p.e, d2: (p.x - c.x) ** 2 + (p.y - c.y) ** 2 + (p.z - c.z) ** 2 }))
      .sort((a, b) => a.d2 - b.d2 || a.entity - b.entity);
    const r = 1 + (k % 6);
    expect<unknown>(index.value.within(c, r).map((n) => n.entity)).toEqual(want.filter((n) => n.d2 <= r * r).map((n) => n.entity));
    expect<unknown>(index.value.nearest(c, 5).map((n) => n.entity)).toEqual(want.slice(0, 5).map((n) => n.entity));
  }
});
