// Mouse-look on the native host: a game's og.PointerLock.wanted reaches the
// frame answer (which the web Worker relays to the shell as `pointer-lock`),
// a host's lock report reaches og.PointerLock.locked, and injected movement
// is og.Pointers.movement for one tick.
//
// Ways it could fail, and the check that catches each:
// - wanted never read into the frame answer → `pointerLock` false/absent
// - the report written to the wrong field, or not at all → `locked`
// - movement lost on the probe's input() or kept past its tick → `seen`
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, bool } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const Wants = defineResource("PlWants", { on: bool(false) });
const seen: { x: number; y: number; locked: boolean }[] = [];
const Look = defineSystem("PlLook", { reads: [og.Pointers, Wants], writes: [og.PointerLock] }, (_q, _time, world) => {
  const lock = world.resource(og.PointerLock);
  const wants = world.resource(Wants);
  const pointers = world.resource(og.Pointers);
  if (!lock.ok || !wants.ok || !pointers.ok) throw new Error("resources");
  lock.value.wanted = wants.value.on;
  seen.push({ x: pointers.value.movement.x, y: pointers.value.movement.y, locked: lock.value.locked });
});

beforeAll(() => {
  loadNativeHost();
});

test("og.PointerLock.wanted reaches the frame answer; locked and movement come back as input", () => {
  const created = createEngine(og.game({ systems: [Look], resources: [Wants] }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const first = engine.frame(1000 / 60);
  expect(first.ok, first.reason).toBe(true);
  expect(first.pointerLock).toBe(false);
  const wants = engine.world.resource(Wants);
  if (!wants.ok) throw new Error(wants.reason);
  wants.value.on = true;
  expect(engine.frame(1000 / 60).pointerLock).toBe(true);

  expect(engine.probe.input({ kind: "pointerLock", locked: true } as never)).toBe(true);
  expect(engine.probe.input({ kind: "pointer", action: "move", x: 10, y: 10, button: "none", movementX: 6, movementY: -2 } as never)).toBe(true);
  expect(engine.probe.input({ kind: "pointer", action: "move", x: 10, y: 10, button: "none", movementX: 1, movementY: 1 } as never)).toBe(true);
  seen.length = 0;
  engine.advance(2);
  expect(seen).toEqual([{ x: 7, y: -1, locked: true }, { x: 0, y: 0, locked: true }]);
  const lock = engine.world.resource(og.PointerLock);
  expect(lock.ok && lock.value.locked).toBe(true);
  // locked is the host's: a game cannot write it.
  expect(() => {
    if (lock.ok) (lock.value as { locked: boolean }).locked = false;
  }).toThrow();

  wants.value.on = false;
  expect(engine.frame(1000 / 60).pointerLock).toBe(false);
  engine.destroy();
});
