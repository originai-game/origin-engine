// The IDL's anim module on both hosts: og.load.animation registers the
// template walker's rig and clips, og.anim poses skeletons from players and
// graphs, writes the fields it owns, fires clip markers as events, solves an
// IK chain, refuses bad graphs by name, and gives the same world hash on the
// native addon and the wasm module. Each host runs in its own process.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "game");

function run(host: string, ...extra: string[]) {
  const r = Bun.spawnSync(["bun", "tests/support/anim-hash.ts", "--host", host, "--project", project, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Record<string, any>;
}

test("og.anim poses the walker the same way on both hosts", () => {
  if (!existsSync(join(project, "cooked", ".cook.ogt"))) {
    const c = Bun.spawnSync([ogBin(), "cook", project], { cwd: repo });
    if (c.exitCode !== 0) throw new Error(new TextDecoder().decode(c.stderr));
  }
  const node = run("node");
  const web = run("web");
  expect(node.tick).toBe(300);
  // The rig and its clips, and the same handles for a second load.
  expect(node.animation.clips.length).toBeGreaterThan(0);
  expect(node.animation.skins[0]).toBeGreaterThan(0);
  expect(node.sameHandles).toBe(true);
  expect(node.uncooked).toMatch(/models\/nothing\.glb/);
  // Written by og.anim: the palette length, the playback time, the graph's state after p0 rose.
  expect(node.skeleton.jointCount).toBe(node.animation.skins[0]);
  expect(node.player.time).toBeGreaterThan(0);
  expect(node.graph.state).toBe(1);
  expect(node.graph.stateTime).toBeGreaterThan(0);
  expect(node.shapes.w0).toBeCloseTo(0.75, 5);
  expect(Number.isFinite(node.chain.error)).toBe(true);
  // Markers fire as og.AnimationEvent when the walker's clips carry them.
  const markers = node.animation.clips.reduce((n: number, c: { markers: string[] }) => n + c.markers.length, 0);
  if (markers > 0) expect(node.events).toBeGreaterThan(0);
  // Refusals name the fix.
  expect(node.missingClip).toMatch(/no-such-clip/);
  expect(node.brokenGraph).toMatch(/line|nowhere/);
  // One engine, two hosts.
  expect(web.hash).toBe(node.hash);
  expect(web.skeleton).toEqual(node.skeleton);
  expect(web.graph).toEqual(node.graph);
  // The pose moved the world: a different tick count is another hash.
  expect(run("node", "--ticks", "301").hash).not.toBe(node.hash);
}, 240_000);

test("a ragdoll at weight 0 follows the animation and goes limp from it", () => {
  const r = Bun.spawnSync(["bun", "tests/support/ragdoll-follow.ts", "--project", project, "--ticks", "40"], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr) + out);
  type P = { x: number; y: number; z: number };
  const { a, b, rest, limp } = JSON.parse(out.trim().split("\n").pop()!) as { a: P[]; b: P[]; rest: P[]; limp: P[] };
  const far = (u: P[], v: P[]) => Math.max(...u.map((p, i) => Math.hypot(p.x - v[i]!.x, p.y - v[i]!.y, p.z - v[i]!.z)));
  expect(a.length).toBeGreaterThan(3);
  // The walk moves the bones from tick to tick, away from the rig's rest pose.
  expect(far(a, b)).toBeGreaterThan(0.005);
  expect(far(b, rest)).toBeGreaterThan(0.02);
  // Going limp starts from the animated pose, not the rest pose.
  expect(far(limp, b)).toBeLessThan(far(limp, rest));
}, 120_000);
