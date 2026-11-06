// Local multiplayer and first-person look on the native host: several
// seats on one device give their inputs through og.net.submit's `player`,
// each reaching its own og.PlayerInput the next tick (the same path a room
// takes, so a game reads seats alone); og.FirstPersonCamera turns og.Camera
// by the locked mouse and holds its pitch.
//
// Ways it could fail, and the check that catches each:
// - every seat's input landing on seat 0 (the old lobby path) → seat 1's x
// - a seat past the device's pads accepted → the refusal
// - yaw turning the wrong way, or pitch not clamped → the camera's forward
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const refusals: string[] = [];
const Submit = defineSystem("LsSubmit", { reads: [og.Input] }, (_q, _time, world) => {
  for (const [player, x] of [[0, -0.5], [1, 0.75]] as const) {
    const r = og.net.submit(world, { player, x, buttons: 1 << player });
    if (!r.ok) throw new Error(r.reason);
  }
  const bad = og.net.submit(world, { player: 9, x: 1 });
  if (!bad.ok) refusals.push(bad.reason);
});

beforeAll(() => {
  loadNativeHost();
});

test("og.net.submit's player drives each local seat's og.PlayerInput", () => {
  const created = createEngine(og.game({
    systems: [Submit],
    start(world) {
      for (const player of [0, 1]) {
        const e = world.spawn({ name: `seat-${player}` });
        if (!e.ok) throw new Error(e.reason);
        world.insert(e.value, og.PlayerInput, { player });
      }
    },
  }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  engine.advance(3);
  const read = (name: string) => {
    const e = engine.world.find(name);
    if (e === null) throw new Error(name);
    const v = engine.world.get(e, og.PlayerInput);
    if (!v.ok) throw new Error(v.reason);
    return { x: v.value.x, buttons: v.value.buttons };
  };
  expect(read("seat-0")).toEqual({ x: -0.5, buttons: 1 });
  expect(read("seat-1")).toEqual({ x: 0.75, buttons: 2 });
  expect(refusals[0]).toContain("local seat");
  engine.destroy();
});

test("og.FirstPersonCamera looks by the locked mouse and holds its pitch", () => {
  const created = createEngine(og.game({
    systems: [...og.systems.cameras],
    start(world) {
      const e = world.spawn({ name: "eye", position: { x: 1, y: 0, z: 2 } });
      if (!e.ok) throw new Error(e.reason);
      world.insert(e.value, og.FirstPersonCamera, { sensitivity: 0.01 });
    },
  }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  engine.advance(1);
  const forward = () => {
    const c = engine.world.resource(og.Camera);
    if (!c.ok) throw new Error(c.reason);
    const q = c.value.quaternion;
    // The camera looks down its local -Z: q * (0, 0, -1).
    return {
      x: -2 * (q.x * q.z + q.w * q.y),
      y: -2 * (q.y * q.z - q.w * q.x),
      z: -(1 - 2 * (q.x * q.x + q.y * q.y)),
      at: c.value.position,
    };
  };
  let f = forward();
  expect([f.at.x, f.at.y, f.at.z].map((v) => Math.round(v * 1e5) / 1e5)).toEqual([1, 1.6, 2]);
  expect(f.z).toBeCloseTo(-1, 6);
  // Unlocked, movement turns nothing.
  engine.probe.input({ kind: "pointer", action: "move", x: 5, y: 5, button: "none", movementX: 50, movementY: 0 } as never);
  engine.advance(1);
  expect(forward().z).toBeCloseTo(-1, 6);
  engine.probe.input({ kind: "pointerLock", locked: true } as never);
  // 50 px right at 0.01 rad/px: a turn of 0.5 rad to the right (yaw -0.5).
  engine.probe.input({ kind: "pointer", action: "move", x: 5, y: 5, button: "none", movementX: 50, movementY: 0 } as never);
  engine.advance(1);
  f = forward();
  expect(f.x).toBeCloseTo(Math.sin(0.5), 5);
  expect(f.z).toBeCloseTo(-Math.cos(0.5), 5);
  // 1000 px up is far past straight up: pitch stops at maxPitch (1.48).
  engine.probe.input({ kind: "pointer", action: "move", x: 5, y: 5, button: "none", movementX: 0, movementY: -1000 } as never);
  engine.advance(1);
  expect(forward().y).toBeCloseTo(Math.sin(1.48), 5);
  engine.destroy();
});

test("a follow camera with a seat drives og.Camera only on that seat's device", () => {
  // Seat 1's follower comes first in query order; outside a room this device plays seat 0, so seat 0's must drive.
  const created = createEngine(og.game({
    systems: [...og.systems.cameras],
    start(world) {
      for (const [seat, x] of [[1, 40], [0, -40]] as const) {
        const e = world.spawn({ name: `hero-${seat}`, position: { x, y: 0, z: 0 } });
        if (!e.ok) throw new Error(e.reason);
        world.insert(e.value, og.FollowCamera, { seat, smoothTime: 0, offset: { x: 0, y: 2, z: 6 } });
      }
    },
  }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  engine.advance(2);
  const c = engine.world.resource(og.Camera);
  if (!c.ok) throw new Error(c.reason);
  expect(c.value.position.x).toBeCloseTo(-40, 4);
  engine.destroy();
});
