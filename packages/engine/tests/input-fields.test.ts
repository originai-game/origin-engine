// og.Input's handle carries every field the IDL declares, not only pointer,
// pointerInside and wheel: a game reading `buttons` or `buttonsPressed` after
// a pointer press sees the left button's bit (1), and the methods still work.
//
// Ways it could fail, and the check that catches each:
// - og.Input read through a hand list of fields → `buttons`/`buttonsPressed` undefined
// - the press edge kept past its tick → the second tick's `buttonsPressed`
// - the methods dropped from the handle → `buttonDown`
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

const seen: { buttons: unknown; buttonsPressed: unknown; buttonDown: boolean; pointer: unknown }[] = [];
const Probe = defineSystem("InputFieldsProbe", { reads: [og.Input] }, (_q, _time, world) => {
  const input = world.resource(og.Input);
  if (!input.ok) throw new Error(input.reason);
  const i = input.value;
  seen.push({ buttons: i.buttons, buttonsPressed: i.buttonsPressed, buttonDown: i.buttonDown("left"), pointer: { ...i.pointer } });
});

beforeAll(() => {
  loadNativeHost();
});

test("a system reads og.Input's buttons and buttonsPressed after a pointer press", () => {
  const created = createEngine(og.game({ systems: [Probe] }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  expect(engine.probe.input({ kind: "pointer", action: "down", x: 40, y: 30, button: "left" } as never)).toBe(true);
  seen.length = 0;
  engine.advance(2);
  expect(seen).toEqual([
    { buttons: 1, buttonsPressed: 1, buttonDown: true, pointer: { x: 40, y: 30 } },
    { buttons: 1, buttonsPressed: 0, buttonDown: true, pointer: { x: 40, y: 30 } },
  ]);
  // Read only: a game cannot write the host's input.
  const input = engine.world.resource(og.Input);
  expect(input.ok && input.value.buttons).toBe(1);
  expect(() => {
    if (input.ok) (input.value as { buttons: number }).buttons = 0;
  }).toThrow();
  engine.destroy();
});
