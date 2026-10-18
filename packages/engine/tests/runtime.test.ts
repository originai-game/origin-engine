// The runtime on the native host: systems write engine memory through typed
// views, stale views throw with their tick and system, the dispatcher refuses
// an unknown system id, declared order holds, refusals are Results.
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineSystem, f32, Vector3 } from "@origingame/engine";
import type { World, Entity } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { boundHost, createEngine, type Engine, type GameToken } from "../src/runtime.ts";

const Pos = defineComponent("RtPos", { p: Vector3() });
const Vel = defineComponent("RtVel", { v: Vector3({ x: 0, y: 0, z: 1 }) });
const Pair = defineComponent("RtPair", { a: f32(), b: f32() });

const Integrate = defineSystem("RtIntegrate", { reads: [Vel], writes: [Pos] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(Pos).p;
    const v = a.column(Vel).v;
    for (let i = 0; i < a.count * 3; i++) p[i] = p[i]! + v[i]! * time.dt;
  }
});

let kept: { view: unknown; array: Float32Array } | null = null;
const Keep = defineSystem("RtKeep", { reads: [Vel] }, (q) => {
  for (const a of q.archetypes()) kept = { view: a, array: a.column(Vel).v };
});

// B copies a into b; A sets a. Declared B first, but B runs after A.
const SetA = defineSystem("RtSetA", { writes: [Pair] }, (q, time) => {
  for (const a of q.archetypes()) a.column(Pair).a.fill(time.tick + 1);
});
const CopyB = defineSystem("RtCopyB", { writes: [Pair], after: ["RtSetA"] }, (q) => {
  for (const a of q.archetypes()) a.column(Pair).b.set(a.column(Pair).a);
});

const Sneaky = defineSystem("RtSneaky", { reads: [Vel] }, (q, _time, world) => {
  for (const a of q.archetypes()) a.column(Pos);
  void world;
});

const spawnMovers = (world: World, args: readonly string[]) => {
  for (let i = 0; i < Number(args[0]); i++) {
    const e = world.spawn({ name: `mover-${i}`, position: { x: i, y: 0, z: 0 } });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Pos, { p: { x: i, y: 0, z: 0 } });
    world.insert(e.value, Vel);
    world.insert(e.value, Pair);
  }
};

function start(game: GameToken): Engine {
  const r = createEngine(game);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

beforeAll(() => {
  loadNativeHost();
});

test("a system writes engine memory through its views, every tick", () => {
  const engine = start(og.game({ systems: [Integrate], commands: { spawn: spawnMovers } }) as unknown as GameToken);
  expect(engine.command("spawn", ["3"]).ok).toBeTrue();
  expect(engine.advance(120)).toEqual({ ok: true, value: 120 });
  let z = 0;
  for (let t = 0; t < 120; t++) z = Math.fround(z + Math.fround(1 / 60));
  const e = engine.world.find("mover-2") as Entity;
  const got = engine.world.get(e, Pos);
  if (!got.ok) throw new Error(got.reason);
  expect(got.value.p.x).toBe(2);
  expect(got.value.p.z).toBe(z);
});

test("a stale view throws with the tick and the system it belonged to", () => {
  const engine = start(og.game({ systems: [Keep], commands: { spawn: spawnMovers } }) as unknown as GameToken);
  engine.command("spawn", ["2"]);
  engine.advance(3);
  const view = kept!.view as { column(c: unknown): unknown; entities: Uint32Array; tick: number };
  expect(() => view.column(Vel)).toThrow(/stale ArchetypeView: it belonged to tick 2, system 'RtKeep'/);
  expect(() => view.entities).toThrow(/tick 2/);
  expect(() => view.tick).toThrow(/tick 2/);
  // On the native host a kept typed array aliases the column only while the
  // column stays put: a structural change detaches it, so it can never reach
  // memory that moved.
  if (native().og_node_views_mode() === "alias") {
    expect(kept!.array.length).toBe(6);
    expect(engine.world.despawn(engine.world.find("mover-0")!).ok).toBeTrue();
    expect(kept!.array.length).toBe(0);
  }
});

test("the dispatcher refuses a system id it has no system for", () => {
  const abi = boundHost()!.abi;
  const ghost = abi.og_system_define_system("RtGhost", JSON.stringify({ reads: [], writes: [] }), 9999);
  const game = abi.og_game_game(JSON.stringify({ systems: [ghost] }));
  const created = JSON.parse(abi.og_engine_create(game, "{}")) as { ok: boolean; value: number };
  expect(created.ok).toBeTrue();
  const advanced = JSON.parse(abi.og_engine_advance(created.value, 1)) as { ok: boolean; reason: string };
  expect(advanced.ok).toBeFalse();
  expect(advanced.reason).toMatch(/tick 0, system 'RtGhost': the dispatcher has no system with callback id 9999/);
});

test("after/before order systems regardless of declaration order", () => {
  const engine = start(og.game({ systems: [CopyB, SetA], commands: { spawn: spawnMovers } }) as unknown as GameToken);
  engine.command("spawn", ["1"]);
  engine.advance(5);
  const e = engine.world.find("mover-0") as Entity;
  const got = engine.world.get(e, Pair);
  if (!got.ok) throw new Error(got.reason);
  expect(got.value).toEqual({ a: 5, b: 5 });
});

test("an undeclared column throws inside the system and fails the tick with its name", () => {
  const engine = start(og.game({ systems: [Sneaky], commands: { spawn: spawnMovers } }) as unknown as GameToken);
  engine.command("spawn", ["1"]);
  const r = engine.advance(1);
  expect(r.ok).toBeFalse();
  if (!r.ok) expect(r.reason).toMatch(/system 'RtSneaky' did not declare 'RtPos'/);
});

test("refusals are Results that name the reason", () => {
  const engine = start(og.game({ commands: { spawn: spawnMovers } }) as unknown as GameToken);
  engine.command("spawn", ["1"]);
  const dup = engine.world.spawn({ name: "mover-0" });
  expect(dup.ok).toBeFalse();
  if (!dup.ok) expect(dup.reason).toMatch(/already exists/);
  const dead = engine.world.despawn(12345 as Entity);
  expect(dead.ok).toBeFalse();
  expect(engine.command("nope").ok).toBeFalse();
  const rng = engine.world.resource(og.Rng);
  expect(rng.ok).toBeTrue();
});

test("GameDesc.commandHelp gives a game's actions their usage in help, and get reads a resource", () => {
  const engine = start(
    og.game({
      commands: { spawn: spawnMovers, other: () => {} },
      commandHelp: { spawn: { usage: "<count>", doc: "Spawn that many movers." }, ghost: { usage: "x" } },
    }) as unknown as GameToken,
  );
  const help = og.commands.help(engine.world as never) as Array<{ name: string; usage: string; doc: string; core: boolean }>;
  const spawn = help.find((c) => c.name === "spawn");
  expect(spawn).toEqual({ name: "spawn", usage: "spawn:<count>", doc: "Spawn that many movers.", core: false });
  expect(help.find((c) => c.name === "other")?.usage).toBe("other");
  expect(help.find((c) => c.name === "get")?.core).toBeTrue();
  const log = engine.takeLog().map((l) => l.text);
  expect(log.some((t) => t.includes("GameDesc.commandHelp names 'ghost', which GameDesc.commands does not declare"))).toBeTrue();
  expect(og.commands.schedule(engine.world as never, "0:set:og.Camera.fov=70;0:get:og.Camera.fov").ok).toBeTrue();
  engine.advance(1);
  expect(engine.takeLog().map((l) => l.text)).toContain("get og.Camera.fov = 70.0");
  expect(engine.takeOutputs().some((o) => o.action === "get")).toBeTrue();
});
