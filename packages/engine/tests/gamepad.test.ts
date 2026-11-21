// web/gamepad.ts and the shell's focus release: every pad reaches the
// engine raw (og-input maps it, crates/og-input/src/padmap.rs), with its
// layout and an SDL GUID from its id, in its own slot; rumble on the right
// pad; keys and pads held when the window loses focus released.
import { expect, test } from "bun:test";
import { padGuid, padStates, playRumble } from "../src/web/gamepad.ts";
import { createRing, drainRing, writeRing } from "../src/web/input.ts";
import { releaseHeld } from "../src/web/shell.ts";

const buttons = (n: number, pressed: number[] = [], values: Record<number, number> = {}) =>
  Array.from({ length: n }, (_, i) => ({ pressed: pressed.includes(i), value: values[i] ?? (pressed.includes(i) ? 1 : 0) }));

test("a pad without the standard mapping goes raw, in the browser's order, not mapped by the shell", () => {
  // An Xbox 360 pad on Firefox for Linux: triggers rest at -1 on axes 2 and 5.
  const pad = {
    connected: true,
    mapping: "",
    index: 0,
    id: "45e-28e-Microsoft X-Box 360 pad",
    buttons: buttons(11, [0, 7], { 3: 0.25 }),
    axes: [0.25, -0.5, 1, 0.75, 0.1, -1, 0, -1],
  };
  const [e] = padStates([pad], new Map());
  expect(e).toEqual({
    kind: "gamepad",
    pad: 0,
    connected: true,
    layout: "web",
    guid: "030000005e0400008e02000000000000",
    buttons: [1, 0, 0, 0.25, 0, 0, 0, 1, 0, 0, 0],
    axes: [0.25, -0.5, 1, 0.75, 0.1, -1, 0, -1],
    hat: 0,
  });
  // A pad the browser maps is its standard layout as it is.
  const [s] = padStates([{ ...pad, mapping: "standard", index: 1 }], new Map());
  expect(s).toMatchObject({ pad: 1, layout: "standard" });
});

test("a pad's GUID comes from the ids in its id", () => {
  expect(padGuid("Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 09cc)")).toBe("030000004c050000cc09000000000000");
  expect(padGuid("54c-9cc-Wireless Controller")).toBe("030000004c050000cc09000000000000");
  expect(padGuid("57e-2009-Pro Controller")).toBe("030000007e0500000920000000000000");
  // Windows Chrome's XInput pads name no ids: Microsoft's protocol, Microsoft's vendor.
  expect(padGuid("Xbox 360 Controller (XInput STANDARD GAMEPAD)")).toBe("030000005e0400000000000000000000");
  expect(padGuid("Some Arcade Stick")).toBe("0".repeat(32));
});

test("eight pads reach the engine, a ninth does not, and a raw pad crosses the ring", () => {
  const pads = Array.from({ length: 9 }, (_, i) => ({ connected: true, mapping: "standard", index: i, buttons: buttons(17, [0]), axes: [0, 0, 0, 0] }));
  const events = padStates(pads, new Map());
  expect(events.map((e) => (e.kind === "gamepad" ? e.pad : -1))).toEqual([0, 1, 2, 3, 4, 5, 6, 7]);
  const ring = createRing();
  const raw = { kind: "gamepad", pad: 7, connected: true, layout: "web", guid: "030000004c050000cc09000000000000", buttons: [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], axes: [1, -1, 0, 0, 0, 0, 0, -1], hat: 9 } as const;
  writeRing(ring, { ...raw, buttons: [...raw.buttons], axes: [...raw.axes] });
  expect(drainRing(ring)[0]).toEqual({ ...raw, buttons: [...raw.buttons], axes: [...raw.axes] });
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

test("keys held when the window loses focus are released", () => {
  const held = new Set(["KeyW", "ShiftLeft"]);
  expect(releaseHeld(held, 5)).toEqual([
    { kind: "key", key: "KeyW", action: "up", time: 5 },
    { kind: "key", key: "ShiftLeft", action: "up", time: 5 },
  ]);
  expect(held.size).toBe(0);
  expect(releaseHeld(held)).toEqual([]);
});

test("pads the page stops seeing go, and the next poll sends them whole", () => {
  const pad = { connected: true, mapping: "standard", index: 3, id: "Xbox (Vendor: 045e Product: 0b13)", buttons: buttons(17, [0]), axes: [-1, 0, 0, 0] };
  const states = new Map<number, string>();
  expect(padStates([pad], states).length).toBe(1);
  // Unchanged: nothing more to send.
  expect(padStates([pad], states)).toEqual([]);
  // Focus lost: the shell releases with padStates([]), the pad goes.
  expect(padStates([], states)).toMatchObject([{ kind: "gamepad", pad: 3, connected: false }]);
  expect(padStates([], states)).toEqual([]);
  // Still held when focus comes back: the press is sent again.
  const [back] = padStates([pad], states);
  expect(back).toMatchObject({ pad: 3, connected: true, axes: [-1, 0, 0, 0] });
  expect(back?.kind === "gamepad" && back.buttons[0]).toBe(1);
});
