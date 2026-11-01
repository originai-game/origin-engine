// web/gamepad.ts and the shell's focus release: what reaches the engine
// from pads the browser does not map to the standard layout (Firefox, Linux,
// third-party pads, all dropped before), from pads past the fourth, with
// each pad's family; rumble on the right pad; keys and pads held when the
// window loses focus released.
import { expect, test } from "bun:test";
import { heldKeys, padDevice, padIds, padStates, playRumble, readPad, restPads } from "../src/web/gamepad.ts";
import { createRing, drainRing, writeRing } from "../src/web/input.ts";
import { releaseHeld } from "../src/web/shell.ts";

const buttons = (n: number, pressed: number[] = [], values: Record<number, number> = {}) =>
  Array.from({ length: n }, (_, i) => ({ pressed: pressed.includes(i), value: values[i] ?? (pressed.includes(i) ? 1 : 0) }));

test("a pad without the standard mapping is read through the table, not dropped", () => {
  // An Xbox 360 pad on Firefox for Linux: A is 0, triggers rest at -1 on
  // axes 2 and 5, the right stick is on 3 and 4, the d-pad is the hat 6/7.
  const pad = {
    connected: true,
    mapping: "",
    index: 0,
    id: "45e-28e-Microsoft X-Box 360 pad",
    buttons: buttons(11, [0, 7]),
    axes: [0.25, -0.5, 1, 0.75, 0.1, -1, 0, -1],
  };
  const read = readPad(pad, true);
  expect(read.buttons[0]).toBe(true); // A
  expect(read.buttons[9]).toBe(true); // Start is raw button 7
  expect(read.buttons[8]).toBe(false);
  expect(read.buttons[12]).toBe(true); // d-pad up from the hat
  expect(read.axes).toEqual([0.25, -0.5, 0.75, 0.1]);
  expect(read.triggers).toEqual([1, 0]); // LT fully in, RT at rest
  const events = padStates([pad], new Map(), true);
  expect(events.length).toBe(1);
  expect(events[0]).toMatchObject({ kind: "gamepad", pad: 0, device: "xbox", buttons: ["a", "leftTrigger", "start", "up"] });
  expect(heldKeys([pad], true)).toEqual(new Set(["Space", "Enter", "ArrowUp"]));
});

test("an unknown non-standard pad falls back to the standard indices", () => {
  const pad = { connected: true, mapping: "", index: 2, id: "Some Arcade Stick", buttons: buttons(12, [1]), axes: [0, 0, 0, 0] };
  const [e] = padStates([pad], new Map());
  expect(e).toMatchObject({ pad: 2, buttons: ["b"], device: "gamepad" });
});

test("eight pads reach the engine, a ninth does not", () => {
  const pads = Array.from({ length: 9 }, (_, i) => ({ connected: true, mapping: "standard", index: i, buttons: buttons(17, [0]), axes: [0, 0, 0, 0] }));
  const events = padStates(pads, new Map());
  expect(events.map((e) => (e.kind === "gamepad" ? e.pad : -1))).toEqual([0, 1, 2, 3, 4, 5, 6, 7]);
  // Pad 7 and its family cross the ring.
  const ring = createRing();
  writeRing(ring, { kind: "gamepad", pad: 7, connected: true, buttons: ["x"], axes: [0, 0, 0, 0], triggers: [0, 0], device: "playstation" });
  expect(drainRing(ring)[0]).toMatchObject({ pad: 7, buttons: ["x"], device: "playstation" });
});

test("a pad's family comes from its vendor id, else its name", () => {
  expect(padIds("Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 09cc)")).toEqual({ vendor: "054c", product: "09cc" });
  expect(padIds("54c-9cc-Wireless Controller")).toEqual({ vendor: "054c", product: "09cc" });
  expect(padDevice("Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 09cc)")).toBe("playstation");
  expect(padDevice("Xbox Wireless Controller (STANDARD GAMEPAD Vendor: 045e Product: 0b13)")).toBe("xbox");
  expect(padDevice("57e-2009-Pro Controller")).toBe("switch");
  expect(padDevice("8BitDo SN30 Pro (Vendor: 2dc8 Product: 6101)")).toBe("gamepad");
});

test("rumble plays on the asked pad, or every pad, and 0 ms stops it", () => {
  const played: Array<[number, string, Record<string, number>?]> = [];
  const pad = (index: number) => ({
    connected: true,
    index,
    vibrationActuator: {
      playEffect: (kind: string, p: Record<string, number>) => (played.push([index, kind, p]), Promise.resolve()),
      reset: () => (played.push([index, "reset"]), Promise.resolve()),
    },
  });
  const pads = [pad(0), null, pad(2)];
  expect(playRumble([[2, 1, 0.25, 300]], pads)).toBe(1);
  expect(played).toEqual([[2, "dual-rumble", { startDelay: 0, duration: 300, strongMagnitude: 1, weakMagnitude: 0.25 }]]);
  played.length = 0;
  expect(playRumble([[-1, 0.5, 0.5, 0]], pads)).toBe(2);
  expect(played).toEqual([
    [0, "reset"],
    [2, "reset"],
  ]);
});

test("keys held when the window loses focus are released", () => {
  const held = new Set(["KeyW", "ShiftLeft"]);
  expect(releaseHeld(held, 5)).toEqual([
    { kind: "key", key: "KeyW", action: "up", time: 5 },
    { kind: "key", key: "ShiftLeft", action: "up", time: 5 },
  ]);
  expect(held.size).toBe(0);
  expect(releaseHeld(held)).toEqual([]);
});

test("pads held when the window loses focus come to rest, and the next poll sends them whole", () => {
  const pad = { connected: true, mapping: "standard", index: 3, id: "Xbox (Vendor: 045e Product: 0b13)", buttons: buttons(17, [0]), axes: [-1, 0, 0, 0] };
  const states = new Map<number, string>();
  expect(padStates([pad], states).length).toBe(1);
  // Unchanged: nothing more to send.
  expect(padStates([pad], states)).toEqual([]);
  expect(restPads(states)).toEqual([{ kind: "gamepad", pad: 3, connected: true, buttons: [], axes: [0, 0, 0, 0], triggers: [0, 0] }]);
  expect(restPads(states)).toEqual([]);
  // Still held when focus comes back: the press is sent again.
  expect(padStates([pad], states)[0]).toMatchObject({ pad: 3, buttons: ["a"], axes: [-1, 0, 0, 0] });
});
