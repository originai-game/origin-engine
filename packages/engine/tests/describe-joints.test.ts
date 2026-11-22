// The probe's describe lists a posed skeleton's joints on screen: the cooked
// walker (tests/fixtures/cook-assets) played on an og.AnimationPlayer, seen
// from the front, its joints named by humanoid role and projected where the
// camera puts the posed nodes.
//
// Ways it could fail, and the check that catches each:
// - joints from the rest pose, not the evaluated one → the feet's points
//   miss og.anim.getNodeMatrix's projection mid-stride
// - the node's frame without the entity's Transform → every point is off by
//   the walker's place (it stands away from the origin)
// - roles not read (node names only) → no `head` or `leftFoot`
// - the humanoid filter ignored → `Rig` listed with it; `none` still lists
// - y measured up instead of down the surface → the head below the feet
import { beforeAll, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem, mat4, type Entity, type World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "tests", "fixtures", "cook-assets");
const AT = { x: 1.5, y: 0, z: -2 };

interface Joint { name: string; point: { x: number; y: number }; depth: number; visible: boolean }
interface Seen { name: string | null; joints?: Joint[] }

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

beforeAll(() => {
  loadNativeHost();
});

// The cook runs in the test, not a hook: a hook's timeout would not cover a cold cook.
test("describe lists the walker's posed joints where the camera puts them", () => {
  cookProject(project, repo);
  let nodes: readonly string[] = [];
  const Place = defineSystem("DjPlace", { reads: [], writes: [og.Transform, og.Model, og.Skeleton, og.AnimationPlayer] }, (_q, _t, world: World) => {
    if (world.find("walker")) return;
    const model = must(og.load.gltf(world, "models/walker.glb"));
    if (og.load.status(world, model) !== "ready") return;
    const anim = must(og.load.animation(world, "models/walker.glb"));
    nodes = anim.nodes;
    const e = must(world.instantiate(model, { name: "walker", position: AT })) as Entity;
    must(world.insert(e, og.Skeleton, { asset: anim.skeleton }));
    must(world.insert(e, og.AnimationPlayer, { clip: anim.clips[0]!.clip, speed: 1 }));
  });
  const game = og.game({
    systems: [Place],
    start(world) {
      const camera = must(world.resource(og.Camera));
      camera.position = { x: AT.x, y: 1.2, z: AT.z + 5 };
      camera.lookAt({ x: AT.x, y: 1, z: AT.z });
    },
  });
  const engine = must(createEngine(game as unknown as GameToken, { root: project }));
  must(engine.advance(40));
  const world = engine.world;
  const walker = world.find("walker")!;
  expect(walker).toBeTruthy();

  const find = (d: unknown) => (d as { entities: Seen[] }).entities.find((e) => e.name === "walker")!;
  const humanoid = find(engine.probe.describe("humanoid")).joints!;
  const named = Object.fromEntries(humanoid.map((j) => [j.name, j]));
  for (const role of ["hips", "head", "leftFoot", "rightFoot", "leftHand"]) expect(named[role]).toBeDefined();
  expect(humanoid.some((j) => j.name === "Rig")).toBe(false);

  // Each where the camera projects the posed node, at the walker's place.
  const camera = must(world.resource(og.Camera));
  const t = must(world.get(walker, og.Transform));
  for (const [role, node] of [["head", "head"], ["leftFoot", "foot.L"], ["rightFoot", "foot.R"]] as const) {
    const m = must(og.anim.getNodeMatrix(world, walker, nodes.indexOf(node))).elements;
    const local = { x: m[12]!, y: m[13]!, z: m[14]! };
    const p = og.transform.compose(t.position, t.quaternion, t.scale);
    const w = mat4.transformPoint(p, local);
    const screen = camera.worldToScreen(w)!;
    const j = named[role]!;
    expect(Math.abs(j.point.x - screen.x)).toBeLessThan(1);
    expect(Math.abs(j.point.y - screen.y)).toBeLessThan(1);
    expect(j.visible).toBe(true);
    expect(j.depth).toBeGreaterThan(3);
  }
  expect(named.head!.point.y).toBeLessThan(named.leftFoot!.point.y - 50);

  // Every joint by default, at least the humanoid ones; none on request.
  expect(find(engine.probe.describe()).joints!.length).toBeGreaterThanOrEqual(humanoid.length);
  expect(find(engine.probe.describe("none")).joints).toBeUndefined();
}, 600_000);
