// One native engine per process on the reference scene (crates/og-tests/
// reference-scene: 500 turning objects, 20 skinned walkers, two shadowed
// lights, post) plus ./lanes-crowd.ts (2000 more boxes, 160 more walkers, 48
// falling crates), so every lane has work to split. The lanes are whatever
// OG_LANES gave the process. Prints JSON for lanes.test.ts: at ticks 60, 180
// and 300 the world hash and the SHA-256 of a captured frame (anim palettes,
// extraction and cull reach the pixels, not the world hash); then a replay:
// restore the snapshot of tick 60, switch lanes with the probe's
// lanes:<on|off>, advance to 180 again; and the dump_lanes report.
//
//   OG_LANES=off|<n> bun tests/support/lanes-hash.ts --replay-lanes on|off

import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";
import { loadNativeHost, native } from "../../src/node/index.ts";
import { bundleGame } from "../../src/node/build.ts";
import { decodeCapture } from "./png.ts";
import { crowd } from "./lanes-crowd.ts";

declare const Bun: { CryptoHasher: new (alg: string) => { update(d: Uint8Array): void; digest(enc: "hex"): string } };

const argv = process.argv.slice(2);
const replayLanes = argv.includes("--replay-lanes") ? argv[argv.indexOf("--replay-lanes") + 1]! : "off";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const repo = join(pkg, "..", "..");
const scene = join(repo, "crates", "og-tests", "reference-scene");
const W = 320;
const H = 180;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

loadNativeHost();
const bundled = join(repo, "target", "tmp", "lanes-test", `scene-${process.pid}.js`);
const b = await bundleGame(join(scene, "src", "main.ts"), bundled, "native");
if (!b.ok) throw new Error(b.reason);
const ref = (await import(bundled)) as { build(world: World): void; Turn: unknown };

const game = og.game({
  seed: 1,
  systems: [ref.Turn as never],
  render: { antialias: "off" },
  start(world: World) {
    ref.build(world);
    crowd(world);
  },
}) as unknown as GameToken;
const engine = must(createEngine(game, { root: scene }));
const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);

const frame = () => {
  const img = decodeCapture(engine.probe.capture(W, H));
  const h = new Bun.CryptoHasher("sha256");
  h.update(img.rgba);
  return h.digest("hex");
};
const at = (tick: number) => {
  const now = engine.world.tick;
  if (tick > now) must(engine.advance(tick - now));
  return { tick: engine.world.tick, hash: engine.hash(), frame: frame() };
};

const out: Record<string, unknown> = { tier: attached.value };
const first = at(60);
const snap = engine.probe.snapshot() as { id: string; hash: string };
out.checkpoints = [first, at(180), at(300)];
if (!engine.probe.restore(snap.id)) throw new Error("restore refused");
out.restored = engine.hash();
engine.probe.commands(`0:lanes:${replayLanes}`);
out.replay = at(180);
engine.probe.commands("0:dump_lanes:lanes.json");
must(engine.advance(1));
const dump = engine.takeOutputs().find((o) => o.file === "lanes.json");
out.lanes = dump ? JSON.parse(dump.data) : null;
engine.destroy();
console.log(JSON.stringify(out));
