// The IDL's anim module on both hosts: og.load.animation registers the
// template walker's rig and clips, og.anim poses skeletons from players and
// graphs, writes the fields it owns, fires clip markers as events, solves an
// IK chain, refuses bad graphs by name, and gives the same world hash on the
// native addon and the wasm module. Each host runs in its own process.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject } from "./support/og-bin.ts";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tests", "fixtures", "cook-assets");

function run(host: string, ...extra: string[]) {
  const r = Bun.spawnSync(["bun", "tests/support/anim-hash.ts", "--host", host, "--project", project, ...extra], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Record<string, any>;
}

// Every test that reads the template's cooked/ cooks it first (incremental:
// free when current, and a type-hash move re-cooks instead of reading a
// stale blob). In the test, not a hook: a hook's timeout would not cover a
// cold cook, and the first test skips in the functional lane, so a cook
// inside it left the ragdoll test on the stale blob (Metal lane, abab174e).
test.skipIf(STALE_WASM)("og.anim poses the walker the same way on both hosts", () => {
  cookProject(project, repo);
  const node = run("node");
  const web = run("web");
  expect(node.tick).toBe(300);
  // The rig and its clips, and the same handles for a second load.
  expect(node.animation.clips.length).toBeGreaterThan(0);
  expect(node.animation.skins[0]).toBeGreaterThan(0);
  // Derive joint order independently from the fixture's original glTF,
  // translating by name because the cook topologically reorders nodes.
  const glb = readFileSync(join(project, "assets/models/walker.glb"));
  const jsonLength = new DataView(glb.buffer, glb.byteOffset, glb.byteLength).getUint32(12, true);
  const gltf = JSON.parse(new TextDecoder().decode(glb.subarray(20, 20 + jsonLength)));
  const expectedJoints = gltf.skins.map((skin: { joints: number[] }) =>
    skin.joints.map((j) => node.animation.nodes.indexOf(gltf.nodes[j].name)));
  expect(expectedJoints.flat().every((j: number) => j >= 0)).toBe(true);
  expect(node.animation.skinJoints).toEqual(expectedJoints);
  expect(web.animation.skinJoints).toEqual(expectedJoints);
  // The named hand is NOT its skin joint index in this fixture.
  const handNode = node.animation.nodes.indexOf("hand.L");
  expect(expectedJoints[0].indexOf(handNode)).not.toBe(handNode);
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
  // The graph's walk markers come from its second state, dominant past its fade.
  if (markers > 0) expect(node.markerSources.dominant).toBeGreaterThan(0);
  if (markers > 0) expect(node.markerSources.states).toBeGreaterThan(0);
  expect(web.markerSources).toEqual(node.markerSources);
  // Refusals name the fix.
  expect(node.missingClip).toMatch(/no-such-clip/);
  expect(node.brokenGraph).toMatch(/line|nowhere/);
  expect(node.spriteGraph).toBe("number");
  expect(node.spriteBroken).toMatch(/line|nowhere/);
  // FABRIK's model-space target (0.65, 1.6, 0.3), transformed by
  // Rz(90) * scale(2,3,4) + (6,0,0), is (1.2,1.3,1.2).
  const hand = node.attachment.matrix.elements;
  for (const [axis, expected] of [[12, 1.2], [13, 1.3], [14, 1.2]]) {
    expect(hand[axis!]).toBeCloseTo(expected!, 3);
  }
  for (const [axis, expected] of [[12, 5.4], [13, 0.2], [14, 1.2]]) {
    expect(node.helperSocket.matrix.elements[axis!]).toBeCloseTo(expected!, 5);
  }
  expect(node.gripOffset.position).toEqual({ x: 0, y: 0, z: 0 });
  expect(node.socketCollider).toBe(true);
  expect(node.badSocket).toMatch(/Animation.nodes/);
  expect(node.dynamicSocket).toMatch(/kinematic/);
  expect(node.badQuery.ok).toBe(false);
  expect(node.badQuery.reason).toMatch(/Animation.nodes/);
  // Game systems first read rest, then the completed first tick's IK.
  for (const [axis, rest, solved] of [[12, 0.85, 0.65], [13, 1.6, 1.6], [14, 0, 0.3]]) {
    expect(node.querySamples[0].elements[axis!]).toBeCloseTo(rest!, 5);
    expect(node.querySamples[1].elements[axis!]).toBeCloseTo(solved!, 3);
    expect(node.nodeMatrix.value.elements[axis!]).toBeCloseTo(solved!, 3);
  }
  expect(node.nodeMatrix.ok).toBe(true);
  expect(web.nodeMatrix).toEqual(node.nodeMatrix);
  expect(web.querySamples).toEqual(node.querySamples);
  // One engine, two hosts.
  expect(web.hash).toBe(node.hash);
  expect(web.skeleton).toEqual(node.skeleton);
  expect(web.graph).toEqual(node.graph);
  expect(web.spriteBroken).toBe(node.spriteBroken);
  expect(web.attachment).toEqual(node.attachment);
  expect(web.helperSocket).toEqual(node.helperSocket);
  expect(web.socketCollider).toBe(true);
  expect(web.badSocket).toBe(node.badSocket);
  expect(web.dynamicSocket).toBe(node.dynamicSocket);
  // The pose moved the world: a different tick count is another hash.
  expect(run("node", "--ticks", "301").hash).not.toBe(node.hash);
}, 240_000);

test("a ragdoll at weight 0 follows the animation and goes limp from it", () => {
  cookProject(project, repo);
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
