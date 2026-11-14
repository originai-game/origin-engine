// The facing convention on the native host, as a game uses it: a
// third-person walker moved by og.input.moveRelative and turned by
// og.transform.face (the `facing` guide).
//
// Ways it could fail, and the check that catches each:
// - the action's y mapped to world +Z or to the camera's backwards → W
//   walks out of the screen (the first walk's z)
// - the camera's rotation ignored (a world-fixed mapping) → W still walks
//   to -Z after the camera has orbited to the other side
// - the screen's right taken from an object's right (-X) → D walks to the
//   screen's left
// - og.transform.face pointing -Z at the target, as og.transform.lookAt →
//   the walker's forward is opposite to its movement
// - a pitched camera shortening the step → the walked distance
// - og.transform.face used for a 2D thing → its +X leaves the xy plane
//   (the 2D test's forward z and its unchanged spin axis)
// - a 2D heading measured from +Y, or clockwise → the quarter turn's
//   forward; a 3D heading measured from +X → the quarter turn's forward
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem, type World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

const SPEED = 3;
const Walk = defineSystem("FcWalk", { reads: [og.Camera], writes: [og.Transform] }, (_q, time, world) => {
  const hero = world.find("hero")!;
  const camera = must(world.resource(og.Camera));
  const move = og.input.moveRelative(world, "move", camera.quaternion);
  if (move.x === 0 && move.z === 0) return;
  const t = must(world.get(hero, og.Transform));
  const position = { x: t.position.x + move.x * SPEED * time.dt, y: t.position.y, z: t.position.z + move.z * SPEED * time.dt };
  must(world.set(hero, og.Transform, { position, quaternion: og.transform.face({ x: 0, y: 0, z: 0 }, move) }));
});

beforeAll(() => {
  loadNativeHost();
});

/** Put the camera `behind` the hero (an offset from it), 3 m up, looking at it. */
function behind(world: World, offset: { x: number; z: number }) {
  const hero = must(world.get(world.find("hero")!, og.Transform)).position;
  const camera = must(world.resource(og.Camera));
  camera.position = { x: hero.x + offset.x, y: hero.y + 3, z: hero.z + offset.z };
  camera.lookAt(hero);
}

test("W walks into the screen and D to its right, before and after the camera orbits half a turn", () => {
  const game = og.game({
    systems: [Walk],
    start: (world: World) => {
      must(world.spawn({ name: "hero", position: { x: 0, y: 0, z: 0 } }));
      og.input.defineAction(world, { name: "move", kind: "axis2", bindings: [{ up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD" }] });
      behind(world, { x: 0, z: 8 });
    },
  });
  const engine = must(createEngine(game as unknown as GameToken));
  const world = engine.world;
  const hero = world.find("hero")!;
  const at = () => must(world.get(hero, og.Transform));
  const key = (k: string, action: "down" | "up") => engine.probe.input({ kind: "key", key: k, action } as never);
  const walk = (k: string) => {
    const from = at().position;
    key(k, "down");
    engine.advance(60);
    key(k, "up");
    engine.advance(1);
    const to = at().position;
    return { x: to.x - from.x, y: to.y - from.y, z: to.z - from.z };
  };

  // The camera is on the +Z side looking down -Z: into the screen is -Z.
  let step = walk("KeyW");
  expect(step.x).toBeCloseTo(0, 4);
  expect(step.y).toBe(0);
  expect(step.z).toBeCloseTo(-SPEED, 3); // one second at full speed, the camera's pitch notwithstanding
  let forward = og.transform.forward(at().quaternion);
  expect(forward.z).toBeCloseTo(-1, 5); // it faces the way it walked
  step = walk("KeyD");
  expect(step.x).toBeCloseTo(SPEED, 3);
  expect(step.z).toBeCloseTo(0, 4);
  expect(og.transform.right(at().quaternion).z).toBeCloseTo(1, 5); // facing +X, its right is +Z

  // Orbit half a turn: the camera is on the -Z side, so into the screen is +Z and its right is -X.
  behind(world, { x: 0, z: -8 });
  const camera = must(world.resource(og.Camera));
  expect(og.transform.cameraForward(camera.quaternion).z).toBeGreaterThan(0.9);
  expect(og.transform.cameraRight(camera.quaternion).x).toBeCloseTo(-1, 5);
  step = walk("KeyW");
  expect(step.x).toBeCloseTo(0, 4);
  expect(step.z).toBeCloseTo(SPEED, 3);
  forward = og.transform.forward(at().quaternion);
  expect(forward.z).toBeCloseTo(1, 5);
  step = walk("KeyD");
  expect(step.x).toBeCloseTo(-SPEED, 3);
  expect(step.z).toBeCloseTo(0, 4);
});

test("a 2D thing faces +X, turns about +Z towards its target and stays in the plane", () => {
  const identity = { x: 0, y: 0, z: 0, w: 1 };
  // Unturned, it faces +X.
  expect(og.transform.forward2d(identity)).toEqual({ x: 1, y: 0, z: 0 });
  expect(og.transform.heading2d(identity)).toBe(0);

  // Facing a target up and to the left: its +X points there, in the plane.
  const at = { x: 2, y: 1, z: 5 };
  const target = { x: -1, y: 5, z: -3 }; // the depths differ: only xy counts
  const q = og.transform.face2d(at, target);
  expect(q.x).toBe(0);
  expect(q.y).toBe(0); // a turn about +Z only
  const forward = og.transform.forward2d(q);
  expect(forward.x).toBeCloseTo(-3 / 5, 5);
  expect(forward.y).toBeCloseTo(4 / 5, 5);
  expect(forward.z).toBeCloseTo(0, 6);
  // The 3D call turns the same thing out of the plane: its +Z goes to the target.
  expect(Math.abs(og.transform.forward2d(og.transform.face(at, { x: -1, y: 5, z: 5 })).z)).toBeGreaterThan(0.9);

  // A 2D heading turns from +X towards +Y; the pair round-trips.
  const up = og.transform.forward2d(og.transform.fromHeading2d(Math.PI / 2));
  expect(up.x).toBeCloseTo(0, 5);
  expect(up.y).toBeCloseTo(1, 5);
  expect(og.transform.heading2d(og.transform.fromHeading2d(-2.2))).toBeCloseTo(-2.2, 5);
  expect(og.transform.heading2d(q)).toBeCloseTo(Math.atan2(4, -3), 5);
  // The same point twice: no turn.
  expect(og.transform.face2d(at, at)).toEqual(identity);
});

test("a 3D heading turns about +Y from +Z towards +X", () => {
  expect(og.transform.heading({ x: 0, y: 0, z: 0, w: 1 })).toBe(0);
  const quarter = og.transform.forward(og.transform.fromHeading(Math.PI / 2));
  expect(quarter.x).toBeCloseTo(1, 5);
  expect(quarter.z).toBeCloseTo(0, 5);
  expect(og.transform.heading(og.transform.fromHeading(1.2))).toBeCloseTo(1.2, 5);
  // An object turned to face a point has the heading of the way there.
  const q = og.transform.face({ x: 1, y: 0, z: 1 }, { x: 4, y: 0, z: -3 });
  expect(og.transform.heading(q)).toBeCloseTo(Math.atan2(3, -4), 5);
});
