// A list field through TypeScript on the host named by --host (node: the
// napi addon; web: the wasm module, run here outside a browser, as
// strings-hash.ts does), one process per run: a component and a resource
// with list fields and the engine's PostEffects.custom, written with
// world.insert/world.set and a resource's setter, read back with world.get,
// a resource handle, a system's typed view and the probe's get:, then
// --ticks fixed steps. Prints JSON { host, hash, ... }.
//
//   bun tests/support/lists-hash.ts --host node|web [--ticks 30]

import { og, defineComponent, defineResource, defineSystem, list, f32, u32, Asset } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const host = opt("--host", "node");
const ticks = Number(opt("--ticks", "30"));
if (host === "node") (await import("../../src/node/index.ts")).loadNativeHost();
else {
  const { readFileSync } = await import("node:fs");
  const { dirname, join } = await import("node:path");
  const { fileURLToPath } = await import("node:url");
  const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}

export const Chain = defineComponent("ListChain", { passes: list(u32(), 4), weight: f32(1) });
export const Grades = defineResource("ListGrades", { weights: list(f32(), 3), assets: list(Asset(), 2), turns: u32() });

const out: Record<string, unknown> = { host };
const attempt = (f: () => void): string => {
  try {
    f();
    return "accepted";
  } catch (e) {
    return (e as Error).message;
  }
};

/** On tick 5 the grades gain a weight, written whole; every tick the chains are read through the view. */
const Grade = defineSystem("ListGrade", { reads: [Chain], writes: [Grades] }, (q, time, world) => {
  const g = world.resource(Grades);
  if (!g.ok) return;
  if (time.tick === 5) {
    g.value.weights = [...g.value.weights, 0.25];
    g.value.turns += 1;
  }
  const lanes: number[][] = [];
  for (const a of q.archetypes()) {
    const passes = a.column(Chain).passes;
    out.viewLength = passes.length;
    out.viewCount = a.count;
    // Five u32 lanes per entity: the count, then the four elements.
    for (let i = 0; i < a.count; i++) lanes.push([...passes.subarray(i * 5, i * 5 + 5)]);
  }
  out.viewLanes = lanes;
});

function start(world: World): void {
  const ids: Entity[] = [];
  for (const passes of [[3, 0], [], [0]]) {
    const e = world.spawn({ name: `chain-${ids.length}` });
    if (!e.ok) throw new Error(e.reason);
    const inserted = world.insert(e.value, Chain, { passes, weight: ids.length + 1 });
    if (!inserted.ok) throw new Error(inserted.reason);
    ids.push(e.value);
  }
  out.got = ids.map((e) => {
    const v = world.get(e, Chain);
    return v.ok ? v.value : v.reason;
  });
  // Over the capacity: an error value, nothing truncated, nothing written.
  const first = ids[0]!;
  const long = world.set(first, Chain, { passes: [1, 2, 3, 4, 5] });
  out.long = long.ok ? "accepted" : long.reason;
  const after = world.get(first, Chain);
  out.afterLong = after.ok ? after.value : after.reason;

  // A resource handle: a frozen copy to read, a whole array to write.
  const g = world.resource(Grades);
  if (!g.ok) throw new Error(g.reason);
  const weights = g.value.weights;
  out.defaultWeights = weights;
  out.frozen = Object.isFrozen(weights);
  out.push = attempt(() => (weights as number[]).push(1));
  out.indexWrite = attempt(() => {
    (weights as number[])[0] = 1;
  });
  out.afterPush = g.value.weights;
  g.value.weights = [0.5, 2];
  out.written = g.value.weights;
  out.copy = g.value.weights !== g.value.weights;
  out.resourceLong = attempt(() => {
    g.value.weights = [1, 2, 3, 4];
  });
  out.resourceBadElement = attempt(() => {
    g.value.assets = [1.5 as never];
  });
  out.afterLongWeights = g.value.weights;
  // A handle past 2^32 keeps its high word.
  g.value.assets = [4294967298 as never];
  out.assets = g.value.assets;

  // The engine's PostEffects.custom: empty by default, its capacity the bound.
  const post = world.resource(og.PostEffects);
  if (!post.ok) throw new Error(post.reason);
  out.customDefault = post.value.custom;
  out.customFrozen = Object.isFrozen(post.value.custom);
  out.customLong = attempt(() => {
    post.value.custom = [1, 2, 3, 4, 5, 6, 7, 8, 9] as never;
  });
}

const game = og.game({ tick: 60, systems: [Grade], resources: [Grades], start }) as unknown as GameToken;
const created = createEngine(game);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const advanced = engine.advance(ticks);
if (!advanced.ok) throw new Error(advanced.reason);
const world = engine.world;
const g = world.resource(Grades);
out.grades = g.ok ? { weights: g.value.weights, assets: g.value.assets, turns: g.value.turns } : g.reason;
// `[]` and `[0]` are different world states.
const hashBefore = engine.hash();
const snap = engine.probe.snapshot() as { id?: string } | string;
const sid = typeof snap === "string" ? snap : String(snap.id ?? "");
const zero = world.find("chain-2")!;
world.set(zero, Chain, { passes: [] });
out.hashChanged = engine.hash() !== hashBefore;
out.restored = engine.probe.restore(sid);
out.hashRestored = engine.hash() === hashBefore;
const back = world.get(zero, Chain);
out.afterRestore = back.ok ? back.value : back.reason;
// The probe's command channel reads a list as a JSON array and writes one whole.
out.probeCommands = engine.probe.commands("0:get:chain-0.ListChain.passes:passes.json;0:set:ListGrades.weights=[4,5]");
engine.advance(1);
out.log = engine.takeLog().filter((l) => l.warning).map((l) => l.text);
out.outputs = engine.takeOutputs().filter((o) => o.kind === "json").map((o) => JSON.parse(o.data) as unknown);
const g2 = world.resource(Grades);
out.probeWeights = g2.ok ? g2.value.weights : g2.reason;
out.tick = advanced.value;
out.hash = engine.hash();
console.log(JSON.stringify(out));
