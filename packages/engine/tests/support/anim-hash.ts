// One host per process: the template's walker (templates/game/assets/models/
// walker.glb, cooked) loaded with og.load.animation and posed by og.anim:
// one walker on an AnimationPlayer, one on a two-state AnimationGraph that a
// system drives with p0, one with blendshape weights set by game code, and
// an IK chain on its own entity. Advances --ticks and prints JSON for
// anim.test.ts: the world hash, what the components were written, the
// animation events a system read, and the refusals.
//
//   bun tests/support/anim-hash.ts --host node|web --project <templates/game> [--ticks 300]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const which = opt("--host", "node");
const ticks = Number(opt("--ticks", "300"));
const project = opt("--project", "");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

let root = project;
if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = (await import(join(pkg, "dist", "web", "origin-engine.js"))) as Record<string, (...a: unknown[]) => unknown>;
  await loadWebHost(glue as never, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
  // The Worker's preload, by hand: the manifest and every blob under <root>/cooked/.
  root = "game";
  const manifest = readFileSync(join(project, "cooked", ".cook.ogt"), "utf8");
  glue["og_web_put_file"]!(`${root}/cooked/.cook.ogt`, new TextEncoder().encode(manifest));
  const listed = JSON.parse(glue["og_web_bundle_files"]!(manifest) as string) as Array<{ file: string }>;
  for (const o of listed) glue["og_web_put_file"]!(`${root}/cooked/${o.file}`, new Uint8Array(readFileSync(join(project, "cooked", o.file))));
}

const Tally = defineResource("AnTally", { events: u32(), switched: u32() });
const out: Record<string, unknown> = { host: which };

/** p0 rises after one second: the graph's base layer moves from its first state to its second. */
const Drive = defineSystem("AnDrive", { writes: [og.AnimationGraph] }, (q, time) => {
  for (const a of q.archetypes()) {
    const g = a.column(og.AnimationGraph);
    for (let i = 0; i < a.count; i++) g.p0[i] = time.tick > 60 ? 1 : 0;
  }
});

/** Counts the markers og.anim fired last tick. */
const Count = defineSystem("AnCount", { reads: [og.AnimationEvent], writes: [Tally] }, (_q, _t, world) => {
  const t = world.resource(Tally);
  const n = world.events(og.AnimationEvent).length;
  if (t.ok && n) t.value.events += n;
});

function start(world: World): void {
  const loaded = og.load.animation(world, "models/walker.glb");
  if (!loaded.ok) throw new Error(loaded.reason);
  const again = og.load.animation(world, "models/walker.glb#anim");
  const a = loaded.value;
  out.animation = { skeleton: a.skeleton, clips: a.clips.map((c) => ({ name: c.name, duration: +c.duration.toFixed(4), markers: c.markers })), nodes: a.nodes.length, skins: a.skins, morphTargets: a.morphTargets };
  out.sameHandles = again.ok && again.value.skeleton === a.skeleton && again.value.clips[0]?.clip === a.clips[0]?.clip;
  out.uncooked = (() => {
    const r = og.load.animation(world, "models/nothing.glb");
    return r.ok ? null : r.reason;
  })();
  const [first, second] = [a.clips[0]!, a.clips[1] ?? a.clips[0]!];
  const text = `type animgraph\nversion 1\n---\nparam go\nclip one "${first.name}"\nclip two "${second.name}"\nlayer base {\n  state rest one\n  state move two\n  entry rest\n  transition rest move {\n    when go gt 0.5\n    fade 0.2\n  }\n}\n`;
  const graph = og.anim.graph(world, { text, clips: a.clips });
  if (!graph.ok) throw new Error(graph.reason);
  const missing = og.anim.graph(world, { text: text.replace(`"${second.name}"`, '"no-such-clip"'), clips: a.clips });
  out.missingClip = missing.ok ? null : missing.reason;
  const broken = og.anim.graph(world, { text: "type animgraph\nversion 1\n---\nlayer base {\n  state rest nowhere\n}\n", clips: a.clips });
  out.brokenGraph = broken.ok ? null : broken.reason;

  const spawn = (name: string, x: number) => {
    const e = world.spawn({ name, position: { x, y: 0, z: 0 } });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, og.Skeleton, { asset: a.skeleton, rootMotion: "off" });
    return e.value;
  };
  const player = spawn("player", 0);
  world.insert(player, og.AnimationPlayer, { clip: first.clip, speed: 1 });
  const grapher = spawn("graph", 2);
  world.insert(grapher, og.AnimationGraph, { asset: graph.value });
  const shaper = spawn("shapes", 4);
  world.insert(shaper, og.AnimationPlayer, { clip: second.clip, speed: 0.5 });
  world.insert(shaper, og.Blendshapes, { source: "component", w0: 0.75 });
  const chain = world.spawn({ name: "chain" });
  if (!chain.ok) throw new Error(chain.reason);
  const joints = a.skins[0] ?? 0;
  world.insert(chain.value, og.IkChain, { skeleton: player, tip: Math.max(0, joints - 1), bones: 2, solver: "fabrik", target: { x: 0.3, y: 0.8, z: 0.2 } });
}

const game = og.game({ tick: 60, seed: 3, resources: [Tally], systems: [Drive, Count], start }) as unknown as GameToken;
const created = createEngine(game, { root });
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const advanced = engine.advance(ticks);
if (!advanced.ok) throw new Error(advanced.reason);
const w = engine.world;
const get = <T>(name: string, token: unknown) => {
  const r = w.get(w.find(name)!, token as never) as { ok: boolean; value?: T };
  return r.ok ? r.value! : null;
};
out.tick = advanced.value;
out.hash = engine.hash();
out.skeleton = get("player", og.Skeleton);
out.player = get("player", og.AnimationPlayer);
out.graph = get("graph", og.AnimationGraph);
out.shapes = get("shapes", og.Blendshapes);
out.chain = get("chain", og.IkChain);
const tally = w.resource(Tally);
out.events = tally.ok ? tally.value.events : null;
console.log(JSON.stringify(out));
