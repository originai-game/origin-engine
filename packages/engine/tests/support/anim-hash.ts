// One host per process: the template's walker (tests/fixtures/cook-assets/assets/models/
// walker.glb, cooked) loaded with og.load.animation and posed by og.anim:
// one walker on an AnimationPlayer, one on a two-state AnimationGraph that a
// system drives with p0, one with blendshape weights set by game code, and
// an IK chain on its own entity. Advances --ticks and prints JSON for
// anim.test.ts: the world hash, what the components were written, the
// animation events a system read (with where they came from), and the refusals.
//
//   bun tests/support/anim-hash.ts --host node|web --project <the former test sample> [--ticks 300]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
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

const Tally = defineResource("AnTally", { events: u32(), switched: u32(), dominant: u32(), states: u32() });
const out: Record<string, unknown> = { host: which };
let queryRig = 0 as Entity;
let queryNode = 0;
const querySamples: unknown[] = [];

/** p0 rises after one second: the graph's base layer moves from its first state to its second. */
const Drive = defineSystem("AnDrive", { writes: [og.AnimationGraph] }, (q, time) => {
  for (const a of q.archetypes()) {
    const g = a.column(og.AnimationGraph);
    for (let i = 0; i < a.count; i++) g.p0[i] = time.tick > 60 ? 1 : 0;
  }
});

/** Counts the markers og.anim fired last tick. */
const Count = defineSystem("AnCount", { reads: [og.AnimationEvent, og.Skeleton], writes: [Tally] }, (_q, _t, world) => {
  const t = world.resource(Tally);
  const events = world.events(og.AnimationEvent);
  if (t.ok && events.length) {
    t.value.events += events.length;
    // Where each came from, so both hosts are compared on it too.
    for (const e of events) {
      if (e.dominant) t.value.dominant += 1;
      t.value.states += e.state + 2 * e.layer;
    }
  }
  const a = og.anim.getNodeMatrix(world, queryRig, queryNode);
  const b = og.anim.getNodeMatrix(world, queryRig, queryNode);
  if (!a.ok) throw new Error(a.reason);
  if (!b.ok) throw new Error(b.reason);
  if (JSON.stringify(a.value) !== JSON.stringify(b.value)) throw new Error("node query advanced animation");
  if (querySamples.length < 2) querySamples.push(a.value);
});

function start(world: World): void {
  const loaded = og.load.animation(world, "models/walker.glb");
  if (!loaded.ok) throw new Error(loaded.reason);
  const again = og.load.animation(world, "models/walker.glb#anim");
  const a = loaded.value;
  out.animation = { skeleton: a.skeleton, clips: a.clips.map((c) => ({ name: c.name, duration: +c.duration.toFixed(4), markers: c.markers })), nodes: a.nodes, skins: a.skins, skinJoints: a.skinJoints, morphTargets: a.morphTargets };
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
  // A sprite graph names flipbook or rig2d clips by name, resolved as it plays.
  const sprite = og.anim.graph(world, { sprite: true, text: "type animgraph\nversion 1\n---\nparam speed\nclip idle \"idle\"\nclip run \"run\"\nlayer base {\n  state idle idle\n  state run run\n  entry idle\n  transition idle run {\n    when speed gt 0.5\n  }\n}\n" });
  out.spriteGraph = sprite.ok ? typeof sprite.value : sprite.reason;
  const spriteBroken = og.anim.graph(world, { sprite: true, text: "type animgraph\nversion 1\n---\nlayer base {\n  state idle nowhere\n}\n" });
  out.spriteBroken = spriteBroken.ok ? null : spriteBroken.reason;

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

  // A posed hand and a non-joint glTF node use the same socket API. The
  // rotated, nonuniform parent makes a missing bone frame unambiguous.
  const carrier = spawn("socket-rig", 6);
  queryRig = carrier;
  queryNode = a.nodes.indexOf("hand.L");
  world.set(carrier, og.Transform, { quaternion: { x: 0, y: 0, z: Math.SQRT1_2, w: Math.SQRT1_2 }, scale: { x: 2, y: 3, z: 4 } });
  world.insert(carrier, og.AnimationPlayer, { clip: first.clip });
  const grip = world.spawn({ name: "socket-ik" });
  if (!grip.ok) throw new Error(grip.reason);
  world.insert(grip.value, og.IkChain, { skeleton: carrier, tip: joints - 1, bones: 2, solver: "fabrik", space: "model", target: { x: 0.65, y: 1.6, z: 0.3 }, tolerance: 0.000001 });
  for (const [name, node, position] of [
    ["lantern", a.nodes.indexOf("hand.L"), { x: 0, y: 0, z: 0 }],
    ["helper-socket", a.nodes.indexOf("body"), { x: 0.1, y: 0.2, z: 0.3 }],
  ] as const) {
    const child = world.spawn({ name, parent: carrier, position });
    if (!child.ok) throw new Error(child.reason);
    const attached = world.insert(child.value, og.BoneAttachment, { node });
    if (!attached.ok) throw new Error(attached.reason);
    if (name === "lantern") {
      const body = world.insert(child.value, og.RigidBody, { kind: "kinematic" });
      if (!body.ok) throw new Error(body.reason);
      world.insert(child.value, og.Collider, { shape: "sphere", radius: 0.03 });
    }
  }
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
out.attachment = get("lantern", og.MatrixWorld);
out.helperSocket = get("helper-socket", og.MatrixWorld);
out.gripOffset = get("lantern", og.Transform);
out.querySamples = querySamples;
out.nodeMatrix = og.anim.getNodeMatrix(w, queryRig, queryNode);
out.badQuery = og.anim.getNodeMatrix(w, queryRig, 9999);
const hit = og.physics.castRay(w, { origin: { x: 1.2, y: 1.3, z: 2.2 }, direction: { x: 0, y: 0, z: -1 }, maxDistance: 2 });
out.socketCollider = hit.ok && hit.value?.entity === w.find("lantern");
const badNode = w.set(w.find("lantern")!, og.BoneAttachment, { node: 9999 });
out.badSocket = badNode.ok ? null : badNode.reason;
const dynamic = w.set(w.find("lantern")!, og.RigidBody, { kind: "dynamic" });
out.dynamicSocket = dynamic.ok ? null : dynamic.reason;
const tally = w.resource(Tally);
out.events = tally.ok ? tally.value.events : null;
out.markerSources = tally.ok ? { dominant: tally.value.dominant, states: tally.value.states } : null;
console.log(JSON.stringify(out));
