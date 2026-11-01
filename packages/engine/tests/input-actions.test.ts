// og.input's gameplay actions on the native host, through the same binding
// symbols the web host exposes: a game defines actions at start and reads
// them in a system, a key and a pad drive one action, a controls screen's
// capture rebinds it and reports the conflict, the bindings survive as
// text, a glyph follows the device, and a rumble reaches the frame answer.
//
// Ways it could fail, and the check that catches each:
// - the action map not sampled before the game's systems → `seen` lags a tick
// - og.input missing from the runtime object → `og.input.defineAction` throws
// - the capture's key also firing the action it was bound to → `seen` on that tick
// - a rumble lost between the map and the frame answer → `rumble` absent
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const seen: { jump: boolean; move: { x: number; y: number } }[] = [];
const Read = defineSystem("ActRead", { reads: [] }, (_q, _time, world) => {
  seen.push({ jump: og.input.actionPressed(world, "jump"), move: og.input.actionVector(world, "move") });
});

beforeAll(() => {
  loadNativeHost();
});

test("actions over keys and a pad, rebound by a capture, saved, and glyphed", () => {
  const game = og.game({
    systems: [Read],
    start: (world) => {
      const jump = og.input.defineAction(world, { name: "jump", bindings: [{ control: "Space" }, { control: "pad.a" }] });
      if (!jump.ok) throw new Error(jump.reason);
      og.input.defineAction(world, { name: "crouch", bindings: [{ control: "KeyC" }] });
      og.input.defineAction(world, {
        name: "move",
        kind: "axis2",
        bindings: [{ up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD" }, { control: "pad.leftXY" }],
      });
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const world = engine.world;

  // A refusal names the fix.
  const bad = og.input.defineAction(world, { name: "bad", bindings: [{ control: "pad.leftXY" }] });
  expect(bad.ok).toBe(false);

  engine.probe.input({ kind: "key", key: "KeyD", action: "down" } as never);
  engine.probe.input({ kind: "key", key: "Space", action: "down" } as never);
  seen.length = 0;
  engine.advance(1);
  expect(seen[0]).toEqual({ jump: true, move: { x: 1, y: 0 } });
  expect(og.input.device(world)).toBe("keyboardMouse");
  expect(og.input.glyph(world, "move")).toMatchObject({ icon: "key.wasd", text: "W A S D" });
  engine.probe.input({ kind: "key", key: "KeyD", action: "up" } as never);
  engine.probe.input({ kind: "key", key: "Space", action: "up" } as never);
  engine.advance(1);

  // A PlayStation pad's A (cross) drives the same action; the prompt follows it.
  engine.probe.input({ kind: "gamepad", pad: 5, buttons: ["a"], device: "playstation" } as never);
  seen.length = 0;
  engine.advance(1);
  expect(seen[0]!.jump).toBe(true);
  expect(og.input.device(world)).toBe("playstation");
  expect(og.input.glyph(world, "jump")).toMatchObject({ icon: "ps.cross", device: "playstation", control: "pad.a" });
  expect(og.input.padDevice(world, 5)).toBe("playstation");
  engine.probe.input({ kind: "gamepad", pad: 5, buttons: [] } as never);

  // A controls screen: capture jump's key, press C, see crouch's conflict, swap.
  expect(og.input.startCapture(world, { action: "jump", slot: 0 }).ok).toBe(true);
  engine.advance(1);
  engine.probe.input({ kind: "key", key: "KeyC", action: "down" } as never);
  seen.length = 0;
  engine.advance(1);
  const c = og.input.capture(world);
  expect(c).toMatchObject({ done: true, control: "KeyC", conflicts: [{ action: "crouch", slot: 0, control: "KeyC" }] });
  expect(seen[0]!.jump).toBe(false);
  const applied = og.input.applyCapture(world, "swap");
  expect(applied.ok && applied.value.applied).toBe(true);
  const listed = og.input.actions(world);
  expect(listed.find((a) => a.name === "crouch")!.bindings[0]!.control).toBe("Space");
  expect(listed.find((a) => a.name === "jump")!.rebound).toBe(true);

  // The player's bindings as text, back after a reset.
  const text = og.input.saveBindings(world);
  expect(og.input.resetBindings(world).ok).toBe(true);
  expect(og.input.actions(world).find((a) => a.name === "jump")!.bindings[0]!.control).toBe("Space");
  const loaded = og.input.loadBindings(world, text);
  expect(loaded.ok && loaded.value).toEqual([]);
  expect(og.input.actions(world).find((a) => a.name === "jump")!.bindings[0]!.control).toBe("KeyC");

  // Rumble is output: it reaches the frame answer, once.
  expect(og.input.rumble(world, { pad: 5, low: 1, high: 0.5, ms: 120 }).ok).toBe(true);
  expect(og.input.rumble(world, { pad: 9 }).ok).toBe(false);
  expect(engine.frame(0).rumble).toEqual([[5, 1, 0.5, 120]]);
  expect(engine.frame(0).rumble).toBe(undefined);

  expect(og.input.controlGlyph("pad.leftXY", "switch")).toMatchObject({ icon: "switch.stickL", text: "L Stick" });
  engine.destroy();
});
