// The runtime on the native host: systems write engine memory through typed
// views, stale views throw with their tick and system, the dispatcher refuses
// an unknown system id, declared order holds, refusals are Results.
import { beforeAll, expect, test } from "bun:test";
import { og, defineComponent, defineResource, defineSystem, f32, u32, Vector3 } from "@origingame/engine";
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

// A resource a system counts up, read through handles taken before and outside it.
const Score = defineResource("RtScore", { points: u32(0), spawnX: f32(0) });
const Count = defineSystem("RtCount", { writes: [Score] }, (_q, _time, world) => {
  const score = world.resource(Score);
  if (!score.ok) throw new Error(score.reason);
  (score.value as { points: number }).points += 1;
});
let perTick: number[] = [];
const ReadEveryTick = defineSystem("RtReadEveryTick", { reads: [Score] }, (_q, _time, world) => {
  const score = world.resource(Score);
  if (!score.ok) throw new Error(score.reason);
  perTick.push((score.value as { points: number }).points);
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

test("with deferStart, GameDesc.start waits for the first tick boundary; asking the probe who is speaking is not one", () => {
  let starts = 0;
  const game = og.game({ start: () => void starts++ }) as unknown as GameToken;
  const created = createEngine(game, { deferStart: true });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  // The web Worker attaches its renderer and hands over the transcoded
  // textures here, between createEngine and the first tick boundary.
  expect(starts).toBe(0);
  expect((engine.probe.hello() as { substrate?: string }).substrate).toBe("origin-engine");
  expect(starts).toBe(0);
  expect(engine.advance(0).ok).toBeTrue();
  expect(starts).toBe(1);
  expect(engine.advance(2).ok).toBeTrue();
  expect(starts).toBe(1);
});

test("a throw in GameDesc.start fails the run: the error with its stack, and no tick on the half-built world", () => {
  function buildTheVillage(): void {
    throw new Error("the village has no well");
  }
  // Without deferStart start runs inside createEngine, which refuses.
  const eager = createEngine(og.game({ start: () => buildTheVillage() }) as unknown as GameToken);
  expect(eager.ok).toBeFalse();
  if (!eager.ok) {
    expect(eager.reason).toContain("GameDesc.start threw");
    expect(eager.reason).toContain("the village has no well");
    expect(eager.reason).toContain("buildTheVillage");
  }
  // With deferStart (og run with a renderer, og dev, the web Worker) at the
  // first tick boundary, and every advance, frame and probe call after it.
  const created = createEngine(og.game({ start: () => buildTheVillage() }) as unknown as GameToken, { deferStart: true });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const first = engine.advance(0);
  expect(first.ok).toBeFalse();
  if (!first.ok) {
    expect(first.reason).toContain("the village has no well");
    expect(first.reason).toContain("buildTheVillage");
  }
  expect(engine.advance(1).ok).toBeFalse();
  const frame = engine.frame(1000 / 60);
  expect(frame.ok).toBeFalse();
  expect(frame.reason).toContain("GameDesc.start threw");
  expect(engine.world.tick).toBe(0);
  const answer = JSON.parse(engine.call(JSON.stringify({ jsonrpc: "2.0", id: 7, method: "state", params: {} }))) as { error?: { message: string } };
  expect(answer.error?.message).toContain("the village has no well");
  engine.destroy();
});

test("project.ogt fills the GameDesc fields a game leaves unset, and one set in both differently refuses", () => {
  const project = 'type project\nversion 1\n---\nname "P"\ntick 30\nseed 5\n';
  const created = createEngine(og.game({}) as unknown as GameToken, { project });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  expect(engine.rate).toBe(30);
  const time = engine.world.resource(og.Time);
  expect(time.ok && (time.value as { rate: number }).rate).toBe(30);
  const rng = engine.world.resource(og.Rng);
  expect(rng.ok && rng.value.seed).toBe(5);
  engine.destroy();
  // Set alike in both: fine; differently: refused, naming both.
  expect(createEngine(og.game({ tick: 30 }) as unknown as GameToken, { project }).ok).toBeTrue();
  const clash = createEngine(og.game({ seed: 1 }) as unknown as GameToken, { project });
  expect(clash.ok).toBeFalse();
  if (!clash.ok) expect(clash.reason).toContain("GameDesc.seed is 1 but project.ogt's `seed` is 5");
  // Without a project: GameDesc, else the defaults.
  const bare = createEngine(og.game({}) as unknown as GameToken);
  expect(bare.ok && bare.value.rate).toBe(60);
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

test("a resource handle is live: two handles agree, and a handle taken earlier reads what a system wrote since", () => {
  const engine = start(og.game({ systems: [Count], resources: [Score] }) as unknown as GameToken);
  const a = engine.world.resource(Score);
  const b = engine.world.resource(Score);
  if (!a.ok || !b.ok) throw new Error("no RtScore");
  const ha = a.value as { points: number; spawnX: number };
  const hb = b.value as { points: number; spawnX: number };
  // Hollowbell FINDINGS #6: b's write was invisible to a, which answered its fetch-time copy.
  hb.spawnX = 5;
  expect(ha.spawnX).toBe(5);
  expect({ ...ha }).toEqual({ points: 0, spawnX: 5 });
  engine.advance(3);
  expect(ha.points).toBe(3);
  expect(hb.points).toBe(3);
  // What the probe's get answers (at the tick's start, before RtCount) is what the handle answered.
  expect(og.commands.schedule(engine.world as never, "0:get:RtScore.points").ok).toBeTrue();
  engine.advance(1);
  expect(engine.takeLog().map((l) => l.text)).toContain("get RtScore.points = 3");
  expect(ha.points).toBe(4);
  // A write from outside the game (the probe's set) reaches a handle taken before it.
  expect(og.commands.schedule(engine.world as never, "0:set:RtScore.spawnX=7").ok).toBeTrue();
  engine.advance(1);
  expect(ha.spawnX).toBe(7);
  // og.Time's handle moves with the ticks too.
  const time = engine.world.resource(og.Time);
  if (!time.ok) throw new Error(time.reason);
  const tick = (time.value as { tick: number }).tick;
  engine.advance(2);
  expect((time.value as { tick: number }).tick).toBe(tick + 2);
  engine.destroy();
});

test("a handle read in a per-tick system costs one engine call, reads in a loop none more", () => {
  perTick = [];
  const engine = start(og.game({ systems: [Count, ReadEveryTick], resources: [Score] }) as unknown as GameToken);
  engine.advance(5);
  // Each tick's system read what RtCount wrote earlier in the same tick.
  expect(perTick).toEqual([1, 2, 3, 4, 5]);
  const a = engine.world.resource(Score);
  if (!a.ok) throw new Error(a.reason);
  const h = a.value as { points: number };
  const reads = 20000;
  // Cold: an engine call between reads (a resource write elsewhere would be one), so each read goes to the engine.
  let t0 = performance.now();
  let sum = 0;
  for (let i = 0; i < reads; i++) {
    engine.world.alive(0 as Entity);
    sum += h.points;
  }
  const cold = ((performance.now() - t0) * 1000) / reads;
  // Warm: a loop over rows reading a tuning field, nothing else reaching the engine.
  t0 = performance.now();
  for (let i = 0; i < reads; i++) sum += h.points;
  const warm = ((performance.now() - t0) * 1000) / reads;
  expect(sum).toBe(10 * reads);
  console.log(`resource handle field read: ${cold.toFixed(2)} µs after an engine call, ${warm.toFixed(3)} µs in a loop`);
  expect(cold).toBeLessThan(100);
  expect(warm).toBeLessThan(1);
  engine.destroy();
});
