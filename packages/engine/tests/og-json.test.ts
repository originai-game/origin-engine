// The machine-readable output a tool that drives og relies on:
// `og test --json`'s events, `og probe`'s result and exit
// status, and `og build --json` with where the build lands. A change to
// any shape asserted here is a breaking change.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { appendFileSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";
import { gpuTest } from "./support/gpu.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "pipe"; stderr: "pipe"; timeout?: number }): { stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>; pid: number; signalCode: string | null; kill(): void };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
// No resident host: these tests pin output shapes, which a warm session does not change.
const env = { ...process.env, OG_BIN: ogBin(), OG_ENGINE_PACKAGE: pkg, OG_RENDERER: "none", OG_SESSION: "0" };
let root = "";
let game = "";

const og = (...args: string[]) => {
  const r = Bun.spawnSync([ogBin(), ...args], { cwd: repo, env });
  return { code: r.exitCode, stdout: new TextDecoder().decode(r.stdout), stderr: new TextDecoder().decode(r.stderr) };
};

const BUILD_WALL_MS = 180_000;

async function tracedBuild(name: string, deadline: number, ...args: string[]) {
  const logs = process.env.GATE_LOGS;
  if (!logs) return og("build", ...args);
  const trace = join(logs, `og-json-${name}.jsonl`);
  const child = Bun.spawn([ogBin(), "build", ...args], {
    cwd: repo, env: { ...env, OG_BUILD_TRACE: trace }, stdout: "pipe", stderr: "pipe",
    timeout: Math.max(1, Math.floor(deadline - performance.now())),
  });
  const record = (event: string, fields: Record<string, unknown>) => {
    try {
      appendFileSync(trace, JSON.stringify({ at: Date.now(), pid: child.pid, event, ...fields }) + "\n");
    } catch {
      // The contract is independent of evidence storage.
    }
  };
  record("spawn", {});
  const pipe = async (stream: "stdout" | "stderr") => {
    const reader = child[stream].getReader();
    const decoder = new TextDecoder();
    let output = "";
    for (;;) {
      const { done, value } = await reader.read();
      const chunk = done ? decoder.decode() : decoder.decode(value, { stream: true });
      if (chunk) record(stream, { chunk });
      output += chunk;
      if (done) return output;
    }
  };
  const sample = setInterval(() => {
    if (process.platform === "win32") return;
    const result = Bun.spawnSync(["ps", "-axo", "pid=,ppid=,stat=,pcpu=,rss=,time=,comm="], { cwd: repo, env });
    const rows = new TextDecoder().decode(result.stdout).trim().split("\n").map((row) => row.trim());
    const family = new Set([String(child.pid)]);
    for (let size = 0; size !== family.size;) {
      size = family.size;
      for (const row of rows) {
        const [pid, parent] = row.split(/\s+/, 3);
        if (pid && parent && family.has(parent)) family.add(pid);
      }
    }
    record("process-tree", { rows: rows.filter((row) => family.has(row.split(/\s+/, 1)[0]!)) });
  }, 15_000);
  try {
    const [code, stdout, stderr] = await Promise.all([child.exited, pipe("stdout"), pipe("stderr")]);
    record("exit", { status: code, signal: child.signalCode });
    return { code, stdout, stderr };
  } finally {
    clearInterval(sample);
  }
}
/** Every stdout line, each of which must be one JSON object. */
const lines = (stdout: string) => stdout.split("\n").filter((l) => l !== "").map((l) => JSON.parse(l) as Record<string, unknown>);

beforeAll(() => {
  root = mkdtempSync(join(tmpdir(), "og-json-"));
  game = join(root, "game");
  const created = Bun.spawnSync(["bun", join(repo, "packages/create/bin/create.ts"), game, "--name", "Json", "--no-prompt"], { cwd: repo, env });
  if (created.exitCode !== 0) throw new Error(new TextDecoder().decode(created.stderr));
  writeFileSync(join(game, "src/main.ts"), `import { og } from "@origingame/engine"; export default og.game({});`);
});
afterAll(() => rmSync(root, { recursive: true, force: true }));

test("og test --json prints only JSON Lines: check, one playtest per file in order, then test", () => {
  writeFileSync(join(game, "unit.test.ts"), `import { expect, test } from "bun:test"; test("unit", () => expect(1).toBe(1));`);
  const playtest = (ok: boolean) => `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("quits", async () => { expect((await runPlaytest({ commands: "0.1:quit" })).quit, "quit").toBe(${ok}); });`;
  writeFileSync(join(game, "tests/a.og.ts"), playtest(true));
  writeFileSync(join(game, "tests/b.og.ts"), playtest(false));
  rmSync(join(game, "tests/smoke.og.ts"), { force: true });
  const r = og("test", game, "--json");
  const events = lines(r.stdout);
  expect(events.map((e) => [e.event, e.playtest])).toEqual([["check", undefined], ["playtest", "a"], ["playtest", "b"], ["test", undefined]]);
  const [check, a, b, last] = events as [Record<string, any>, Record<string, any>, Record<string, any>, Record<string, any>];
  expect(check).toMatchObject({ ok: true, diagnostics: [] });
  expect(typeof check.ms).toBe("number");
  expect(a).toMatchObject({ ok: true, status: "passed", tests: 1, checks: 1, failures: [], uncheckedCount: 0, out: join(game, "og-out/test/a") });
  expect(b).toMatchObject({ ok: false, status: "failed", tests: 1 });
  expect(b.failures.join("\n")).toContain("quit");
  for (const key of ["file", "frameChecks", "unchecked", "failureCaptures", "ms", "assertions", "runs", "adapterErrors", "hash", "tier"]) expect(a).toHaveProperty(key);
  expect(last).toMatchObject({ ok: false, suites: 3, passed: 2, failed: 1, uncheckedSuites: 0, uncheckedCount: 0, skipped: 0, check: { ok: true, errors: 0 } });
  expect(r.code).toBe(1);
  // The bun tests' report and the summary are on stderr.
  expect(r.stderr).toContain("og test: 2 suite(s) passed, 1 failed");
  rmSync(join(game, "unit.test.ts"));
  rmSync(join(game, "tests/b.og.ts"));
}, 180_000);

/** A game that counts its ticks and restarts in its 30th, leaving the count as a first tick does (1), or by mistake 2. */
const restartGame = (good: boolean) => `import { defineResource, defineSystem, og, u32 } from "@origingame/engine";
const Count = defineResource("Count", { n: u32() });
const Tick = defineSystem("Tick", { reads: [], writes: [Count] }, (_q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok) return;
  if (c.value.n === 29) c.value.n = ${good ? 1 : 2};
  else c.value.n++;
});
export default og.game({ systems: [Tick] });
`;

/**
 * A game whose first tick spawns three coins and a hero aiming at the second, and whose restart in
 * its 30th tick despawns them all and spawns them again (new ids, the hero's Target a new handle),
 * or by mistake keeps one coin of the last game.
 */
/**
 * A game whose start (the setup's `start`) sends two Go intents and whose restart (`again`) sends
 * one: each Go sets Count.n back to 1. Its restarted world is a fresh start's but for the Go queue,
 * which holds the input that caused each start.
 */
const intentGame = `import { defineEvent, defineResource, defineSystem, og, u32 } from "@origingame/engine";
const Count = defineResource("Count", { n: u32() });
const Go = defineEvent("Go", { k: u32() });
const Tick = defineSystem("Tick", { reads: [Go], writes: [Count] }, (_q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok) return;
  if (world.events(Go).length > 0) c.value.n = 1;
  else c.value.n++;
});
const go = (world: any) => { og.input.intent(world, Go, { k: 1 }); };
export default og.game({ systems: [Tick], commands: { start: (w) => { go(w); go(w); }, again: (w) => go(w) } });
`;

const respawnGame = (leak: boolean) => `import { defineComponent, defineResource, defineSystem, Entity, og, u32 } from "@origingame/engine";
const Count = defineResource("Count", { n: u32() });
const Piece = defineComponent("Piece", { v: u32() });
const Target = defineComponent("Target", { coin: Entity() });
const spawnAll = (world: any) => {
  const coins = [1, 2, 3].map((v) => {
    const e = world.spawn({ bare: true });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Piece, { v });
    return e.value;
  });
  const hero = world.spawn({ name: "hero", bare: true });
  if (!hero.ok) throw new Error(hero.reason);
  world.insert(hero.value, Piece, { v: 9 });
  world.insert(hero.value, Target, { coin: coins[1] });
};
const Clear = defineSystem("Clear", { reads: [Count], writes: [Piece] }, (q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok || c.value.n !== 29) return;
  const all: any[] = [];
  for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) all.push(a.entity(i));
  for (const e of all.slice(${leak ? 1 : 0})) world.despawn(e);
});
const Round = defineSystem("Round", { reads: [], writes: [Count, Piece, Target] }, (_q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok) return;
  if (c.value.n === 0 || c.value.n === 29) spawnAll(world);
  c.value.n = c.value.n === 29 ? 1 : c.value.n + 1;
});
export default og.game({ systems: [Clear, Round] });
`;

/**
 * A game whose first tick spawns a ground, a falling crate (a dynamic body) and the template's walker
 * playing its first clip, and whose restart in its 30th tick despawns them all and spawns them again:
 * new ids, new bodies and skeletons in og.physics and og.anim. With `moving`, the restart leaves the
 * crate thrown upward.
 */
const physicsGame = (moving: boolean) => `import { ClipAsset, defineComponent, defineResource, defineSystem, og, SkeletonAsset, u32 } from "@origingame/engine";
const Count = defineResource("Count", { n: u32() });
const Piece = defineComponent("Piece", { v: u32() });
const Rig = defineResource("Rig", { skeleton: SkeletonAsset(), clip: ClipAsset() });
const spawnAll = (world: any, thrown: boolean) => {
  const rig = world.resource(Rig).value;
  const must = (r: any) => { if (!r.ok) throw new Error(r.reason); return r.value; };
  const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: 0 } }));
  must(world.insert(ground, og.Collider, { shape: "box", size: { x: 20, y: 1, z: 20 } }));
  must(world.insert(ground, Piece, { v: 1 }));
  const crate = must(world.spawn({ name: "crate", position: { x: 0, y: 2, z: 0 } }));
  must(world.insert(crate, og.RigidBody, { kind: "dynamic" }));
  must(world.insert(crate, og.Collider, { shape: "box", size: { x: 1, y: 1, z: 1 } }));
  must(world.insert(crate, og.Velocity, { linear: { x: 0, y: thrown ? 5 : 0, z: 0 } }));
  must(world.insert(crate, Piece, { v: 2 }));
  const walker = must(world.spawn({ name: "walker", position: { x: 3, y: 0, z: 0 } }));
  must(world.insert(walker, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  must(world.insert(walker, og.AnimationPlayer, { clip: rig.clip, speed: 1 }));
  must(world.insert(walker, Piece, { v: 3 }));
};
const Clear = defineSystem("Clear", { reads: [Count], writes: [Piece] }, (q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok || c.value.n !== 29) return;
  const all: any[] = [];
  for (const a of q.archetypes()) for (let i = 0; i < a.count; i++) all.push(a.entity(i));
  for (const e of all) world.despawn(e);
});
const Round = defineSystem("Round", { reads: [Rig], writes: [Count, Piece, og.Transform, og.RigidBody, og.Collider, og.Velocity, og.Skeleton, og.AnimationPlayer] }, (_q, _t, world) => {
  const c = world.resource(Count);
  if (!c.ok) return;
  if (c.value.n === 0 || c.value.n === 29) spawnAll(world, c.value.n === 29 && ${moving});
  c.value.n = c.value.n === 29 ? 1 : c.value.n + 1;
});
export default og.game({
  systems: [Clear, Round],
  start(world) {
    const a = og.load.animation(world, "base/models/walker");
    if (!a.ok) throw new Error(a.reason);
    const r = world.resource(Rig);
    if (!r.ok) throw new Error(r.reason);
    r.value.skeleton = a.value.skeleton;
    r.value.clip = a.value.clips[0]!.clip;
  },
});
`;

test("a playtest's timings are absent with the reason without a renderer, and a budget on them is unchecked; a restart check names the first difference", () => {
  const original = join(game, "src/main.ts");
  rmSync(join(game, "tests/smoke.og.ts"), { force: true });
  rmSync(join(game, "tests/a.og.ts"), { force: true });
  writeFileSync(join(game, "tests/budget.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("frames", async () => {
  const run = await runPlaytest({ commands: "0.3:quit", budget: { frameMs: { p95: 16.7 }, gpuPasses: { main: { p50: 2 } } } });
  expect(run.timings.frameMs, "no frame timings").toBeNull();
});`);
  writeFileSync(join(game, "tests/restart.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("restarts", async () => {
  const run = await runPlaytest({ commands: "1:quit", restart: { at: 0.5 } });
  expect(run.restart?.ok, "restarted").toBe(true);
});`);
  try {
    for (const good of [true, false]) {
      writeFileSync(original, restartGame(good));
      const ran = og("test", game, "--json");
      const events = lines(ran.stdout);
      if (!events.some((e) => e.playtest === "budget")) throw new Error(ran.stdout + ran.stderr);
      const budget = events.find((e) => e.playtest === "budget") as Record<string, any>;
      const restart = events.find((e) => e.playtest === "restart") as Record<string, any>;
      if (good) {
        expect(budget).toMatchObject({ ok: true, status: "unchecked" });
        expect(budget.timings).toMatchObject({ frameMs: null, gpuPasses: null });
        expect(budget.budget).toEqual({ frameMs: { p95: 16.7 }, gpuPasses: { main: { p50: 2 } } });
        expect(restart).not.toHaveProperty("budget");
        expect(budget.timings.missing.frameMs).toContain("no renderer");
        expect(budget.timings.missing.gpuPasses).toContain("no renderer");
        expect(budget.unchecked.filter((u: string) => u.startsWith("budget "))).toEqual([
          expect.stringContaining("budget frameMs.p95 <= 16.7 ms: no renderer"),
          expect.stringContaining("budget gpuPasses.main.p50 <= 2 ms: no renderer"),
        ]);
        expect(restart).toMatchObject({ ok: true, status: "passed", restart: { tick: 30, ok: true, first: null } });
        expect(restart.restart.hash).toBe(restart.restart.startHash);
      } else {
        expect(restart).toMatchObject({ ok: false, status: "failed", restart: { tick: 30, ok: false } });
        expect(restart.restart.first).toBe("resource Count.n is 2, a fresh start's 1");
        expect(restart.failures.join("\n")).toContain("restart: the world after 30 ticks is not a fresh start's");
      }
    }
    // The same restart keeping Count.n on purpose passes and says what it carried over; a kept name
    // the world does not declare fails, naming the fields it has.
    writeFileSync(join(game, "tests/restart.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("keeps", async () => {
  const run = await runPlaytest({ commands: "1:quit", restart: { at: 0.5, keep: ["Count.n"] } });
  expect(run.restart?.ok, "restarted").toBe(true);
});`);
    writeFileSync(join(game, "tests/miskept.og.ts"), `import { test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("keeps a field the world lacks", async () => {
  await runPlaytest({ commands: "1:quit", restart: { at: 0.5, keep: ["Count.m"] } });
});`);
    const events = lines(og("test", game, "--json").stdout) as Array<Record<string, any>>;
    const kept = events.find((e) => e.playtest === "restart"), wrong = events.find((e) => e.playtest === "miskept");
    rmSync(join(game, "tests/miskept.og.ts"));
    expect(kept, JSON.stringify(events)).toMatchObject({ ok: true, status: "passed", restart: { tick: 30, ok: true, first: null, kept: [{ name: "Count.n", start: 1, now: 2 }] } });
    expect(wrong?.ok, JSON.stringify(wrong)).toBe(false);
    expect(JSON.stringify(wrong)).toContain("restart keep `Count.m`: `Count` has no field `m` (its fields: n)");
    // An intent's queue is the player's input, as og.KeyDown's is: a start by two intents and a
    // restart by one compare equal.
    writeFileSync(original, intentGame);
    writeFileSync(join(game, "tests/restart.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("restarts by intent", async () => {
  const run = await runPlaytest({ setup: "start", commands: "0.49:again;1:quit", restart: { at: 0.5 } });
  expect(run.restart?.ok, "restarted").toBe(true);
});`);
    const byIntent = lines(og("test", game, "--json").stdout).find((e) => e.playtest === "restart") as Record<string, any> | undefined;
    expect(byIntent, JSON.stringify(byIntent?.restart ?? byIntent)).toMatchObject({ ok: true, status: "passed", restart: { ok: true, first: null } });
    writeFileSync(join(game, "tests/restart.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("restarts", async () => {
  const run = await runPlaytest({ commands: "1:quit", restart: { at: 0.5 } });
  expect(run.restart?.ok, "restarted").toBe(true);
});`);
    rmSync(join(game, "tests/budget.og.ts"));
    // Despawning everything and spawning it again is a restart; one entity left over is not, by name.
    for (const leak of [false, true]) {
      writeFileSync(original, respawnGame(leak));
      const ran = og("test", game, "--json");
      const restart = lines(ran.stdout).find((e) => e.playtest === "restart") as Record<string, any> | undefined;
      if (!restart) throw new Error(ran.stdout + ran.stderr);
      if (!leak) {
        expect(restart, JSON.stringify(restart.restart)).toMatchObject({ ok: true, status: "passed", restart: { tick: 30, ok: true, first: null, unchecked: [] } });
        expect(restart.restart.hash).toBe(restart.restart.startHash);
      } else {
        expect(restart).toMatchObject({ ok: false, status: "failed", restart: { tick: 30, ok: false } });
        expect(restart.restart.first).toMatch(/^entity #\d+\.\d+ is not in a fresh start \(1 extra in all\)$/);
        expect(restart.restart.diff.entities).toMatchObject({ missing: [] });
      }
    }
    // A physics and animated game respawned: og.physics and og.anim compare by entity through the
    // pairing; a crate left moving names its entity and field.
    mkdirSync(join(game, "assets/models"), { recursive: true });
    copyFileSync(join(repo, "tests/fixtures/cook-assets/assets/models/walker.glb"), join(game, "assets/models/walker.glb"));
    mkdirSync(join(game, "content/models"), { recursive: true });
    writeFileSync(join(game, "content/models/walker.ts"), `import { file } from "@origingame/engine/author";
export default file.import({ kind: "model", source: "assets/models/walker.glb" });`);
    for (const moving of [false, true]) {
      writeFileSync(original, physicsGame(moving));
      const ran = og("test", game, "--json");
      const restart = lines(ran.stdout).find((e) => e.playtest === "restart") as Record<string, any> | undefined;
      if (!restart?.restart) throw new Error(ran.stdout + ran.stderr);
      const blocks = (restart.restart.diff.records ?? []).map((r: { block: string }) => r.block);
      if (!moving) {
        expect(restart, JSON.stringify(restart.restart)).toMatchObject({ ok: true, status: "passed", restart: { tick: 30, ok: true, first: null, unchecked: [], diff: { records: [], blocks: [] } } });
      } else {
        expect(restart).toMatchObject({ ok: false, status: "failed", restart: { tick: 30, ok: false } });
        expect(restart.restart.first).toStartWith("crate og.");
        expect(restart.restart.diff.components).toContainEqual(expect.objectContaining({ entity: "crate", component: "og.Velocity", fields: [expect.objectContaining({ field: "linear" })] }));
        // og.physics's own body record says the same, by entity and field.
        const body = restart.restart.diff.records.find((r: Record<string, any>) => r.block === "og.physics" && r.entity === "crate" && r.record === "body");
        expect(body?.fields.map((f: { field: string }) => f.field)).toContain("linear");
        expect(blocks.filter((b: string) => b !== "og.physics")).toEqual([]);
      }
    }
    rmSync(join(game, "assets/models/walker.glb"));
    rmSync(join(game, "content/models/walker.ts"));
  } finally {
    writeFileSync(original, `import { og } from "@origingame/engine"; export default og.game({});`);
    rmSync(join(game, "tests/budget.og.ts"), { force: true });
    rmSync(join(game, "tests/restart.og.ts"));
    rmSync(join(game, "tests/miskept.og.ts"), { force: true });
  }
}, 360_000);

gpuTest("a playtest over its frame budget fails, naming the timing and its ceiling", () => {
  writeFileSync(join(game, "tests/over.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("frames", async () => { expect((await runPlaytest({ commands: "0.5:quit", budget: { frameMs: { p95: 0.001 } } })).quit).toBe(true); });`);
  try {
    const r = Bun.spawnSync([ogBin(), "test", game, "--json"], { cwd: repo, env: { ...env, OG_RENDERER: process.env.OG_RENDERER ?? "hardware" } });
    const over = lines(new TextDecoder().decode(r.stdout)).find((e) => e.playtest === "over") as Record<string, any>;
    expect(over.timings.frameMs.p95).toBeGreaterThan(0.001);
    expect(over).toMatchObject({ ok: false, status: "failed" });
    expect(over.failures.join("\n")).toMatch(/budget frameMs\.p95: [\d.]+ ms, over its 0\.001 ms ceiling/);
  } finally {
    rmSync(join(game, "tests/over.og.ts"));
  }
}, 180_000);

test("og probe prints the result as JSON, exits 1 on an error, and --raw prints the frame", async () => {
  const port = String(40000 + (process.pid % 20000));
  const child = Bun.spawn([ogBin(), "run", game, "--headless", "--no-check", "--port", port], { cwd: repo, env, stdout: "pipe", stderr: "pipe" });
  try {
    const reader = child.stdout.getReader();
    let out = "";
    while (!out.includes("probe: ws://")) {
      const { value, done } = await reader.read();
      if (done) throw new Error(`og run ended before the probe was up: ${out}${await new Response(child.stderr).text()}`);
      out += new TextDecoder().decode(value);
    }
    const hello = og("probe", "hello", "--port", port);
    expect(hello.code).toBe(0);
    const result = JSON.parse(hello.stdout);
    expect(result.members).toContain("hello");
    expect(typeof result.engine).toBe("string");
    // A refusal is a result.
    const refused = og("probe", "entity", '{"id":"no-such-entity"}', "--port", port);
    expect([refused.code, JSON.parse(refused.stdout)]).toEqual([0, null]);
    const unknown = og("probe", "no_such_member", "--port", port);
    expect([unknown.code, unknown.stdout]).toEqual([1, ""]);
    expect(unknown.stderr).toContain("og probe: no_such_member: error -32601");
    const raw = JSON.parse(og("probe", "tick", "--port", port, "--raw").stdout);
    expect(raw).toMatchObject({ jsonrpc: "2.0", id: 1 });
    expect(typeof raw.result).toBe("number");
    // The game ends through its probe; it then no longer answers.
    expect(og("probe", "commands", '{"script":"0:quit"}', "--port", port).code).toBe(0);
    expect(await child.exited).toBe(0);
  } finally {
    child.kill();
    await child.exited;
  }
  const gone = og("probe", "hello", "--port", port);
  expect([gone.code, gone.stdout]).toEqual([1, ""]);
}, 120_000);

test("og build --json prints check and build events; the build lands in dist/ or --out", async () => {
  const deadline = performance.now() + BUILD_WALL_MS;
  const events = (r: { stdout: string }) => lines(r.stdout);
  const r = await tracedBuild("default", deadline, game, "--json");
  expect(r.code).toBe(0);
  const [check, build] = events(r) as [Record<string, any>, Record<string, any>];
  expect(events(r).map((e) => e.event)).toEqual(["check", "build"]);
  expect(check.ok).toBe(true);
  expect(build).toMatchObject({ ok: true, out: join(game, "dist"), publish: false });
  for (const key of ["files", "total", "downloads", "probe"]) expect(build).toHaveProperty(key);
  expect(build.manifest.entry).toBe("index.html");
  expect(existsSync(join(game, "dist", "og-build.json"))).toBe(true);
  expect(existsSync(join(game, "dist", build.manifest.entry))).toBe(true);
  const elsewhere = join(root, "elsewhere");
  const moved = await tracedBuild("elsewhere", deadline, game, "--json", "--no-check", "--out", elsewhere);
  expect(moved.code).toBe(0);
  expect(events(moved).map((e) => [e.event, e.out])).toEqual([["build", elsewhere]]);
  expect(existsSync(join(elsewhere, "og-build.json"))).toBe(true);
}, BUILD_WALL_MS);

test("commands files preserve script steps and arguments across run and replay", () => {
  writeFileSync(join(game, "src/main.ts"), `import { og } from "@origingame/engine"; export default og.game({});`);
  const script = join(root, "commands.txt");
  writeFileSync(script, ["# A CRLF recording", "", "  # Indented comment", "0:dump_world:hash#\\;before.json;0:dump_ui:ui.json", "0.2:quit", "# End"].join("\r\n"));
  const out = join(root, "command-outputs");
  const r = og("run", game, "--headless", "--commands-file", script, "--out", out, "--json");
  expect(r.code, r.stderr).toBe(0);
  expect(lines(r.stdout).at(-1)).toMatchObject({ tick: 12, quit: true });
  expect(existsSync(join(out, "hash#;before.json"))).toBe(true);
  expect(existsSync(join(out, "ui.json"))).toBe(true);
  const trace = join(root, "recording.json");
  const recorded = og("replay", "record", game, "--commands-file", script, "--out", trace, "--json");
  expect(recorded.code, recorded.stderr).toBe(0);
  const compared = og("replay", "compare", trace, game, "--json");
  expect(compared.code, compared.stderr).toBe(0);
}, 120_000);

gpuTest("og render records its whole frame grid through New game's snapshot rewind", () => {
  const project = join(root, "render-rewind");
  const made = og("new", project, "--form", "2d", "--look", "pixel", "--no-prompt");
  expect(made.code, made.stderr).toBe(0);
  const out = join(root, "render-frames");
  const script = join(root, "render-commands.txt");
  writeFileSync(script, ['# Restore during the recording', '', '0.5:intent:ShellAction:{"action":"menu","value":""}',
    '1:intent:ShellAction:{"action":"new","value":""}', '  # Commands retain their original recording times',
    '2:dump_world:after#\\;new.json;2.2:expect:ui:text:=Pause', '3:quit'].join("\n"));
  const r = Bun.spawnSync([ogBin(), "render", project, "--commands-file", script,
    "--to", "2.5", "--fps", "30", "--size", "160x90", "--frames", "png", "--out", out, "--json"],
    { cwd: repo, env: { ...env, OG_RENDERER: process.env.OG_RENDERER ?? "hardware" } });
  expect(r.exitCode, new TextDecoder().decode(r.stderr)).toBe(0);
  const frames = JSON.parse(readFileSync(join(out, "frames.json"), "utf8"));
  expect(frames.frames).toHaveLength(75);
  expect(frames.frames.map((f: { index: number; time: number }) => [f.index, f.time])).toEqual(
    Array.from({ length: 75 }, (_, i) => [i, i / 30]),
  );
  expect(existsSync(join(out, frames.frames[74].file))).toBe(true);
  expect(JSON.parse(readFileSync(join(out, "outputs", "after#;new.json"), "utf8"))).toMatchObject({ tick: 59 });
}, 180_000);
