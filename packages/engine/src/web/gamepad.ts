// Gamepads on the web (main thread): polled once per display frame. Each
// standard-mapping pad's state goes to the engine whole whenever it changes
// (a `gamepad` event: og.Gamepad, the first four pads), and, as a fallback
// for games that only read keys, as the keys of the mapping below (the same
// for every game; og-ui counts a pad intent and the key it maps to once).
//
//   A (0) Space · B (1) Escape · X (2) KeyX · Y (3) KeyZ
//   LB (4) KeyQ · RB (5) KeyE · Back (8) Tab · Start (9) Enter
//   D-pad (12–15) and the left stick (past 0.5) ArrowUp/Down/Left/Right

import { GAMEPAD_BUTTONS, type InjectedInput } from "./input.ts";

export const GAMEPAD_KEYS: Readonly<Record<number, string>> = Object.freeze({
  0: "Space",
  1: "Escape",
  2: "KeyX",
  3: "KeyZ",
  4: "KeyQ",
  5: "KeyE",
  8: "Tab",
  9: "Enter",
  12: "ArrowUp",
  13: "ArrowDown",
  14: "ArrowLeft",
  15: "ArrowRight",
});

/** The stick deflection that counts as a direction. */
export const STICK_THRESHOLD = 0.5;

interface PadLike {
  readonly connected: boolean;
  readonly mapping: string;
  readonly buttons: ReadonlyArray<{ readonly pressed: boolean; readonly value?: number }>;
  readonly axes: ReadonlyArray<number>;
  readonly index?: number;
}

/** The `gamepad` events that move the engine's pads from `before` to `now` (keyed by a state string). */
export function padStates(pads: ReadonlyArray<PadLike | null>, before: Map<number, string>): InjectedInput[] {
  const out: InjectedInput[] = [];
  const seen = new Set<number>();
  pads.forEach((pad, i) => {
    const index = pad?.index ?? i;
    if (!pad || !pad.connected || pad.mapping !== "standard" || index > 3) return;
    seen.add(index);
    const round = (v: number | undefined) => Math.round((v ?? 0) * 1000) / 1000;
    const event: InjectedInput = {
      kind: "gamepad",
      pad: index,
      connected: true,
      buttons: GAMEPAD_BUTTONS.filter((_, b) => pad.buttons[b]?.pressed),
      axes: [0, 1, 2, 3].map((a) => round(pad.axes[a])),
      triggers: [round(pad.buttons[6]?.value), round(pad.buttons[7]?.value)],
    };
    const key = JSON.stringify(event);
    if (before.get(index) !== key) {
      before.set(index, key);
      out.push(event);
    }
  });
  for (const index of [...before.keys()]) {
    if (seen.has(index)) continue;
    before.delete(index);
    out.push({ kind: "gamepad", pad: index, connected: false, buttons: [], axes: [0, 0, 0, 0], triggers: [0, 0] });
  }
  return out;
}

/** The keys a set of pads holds now (any pad holding a key holds it). */
export function heldKeys(pads: ReadonlyArray<PadLike | null>): Set<string> {
  const held = new Set<string>();
  for (const pad of pads) {
    if (!pad || !pad.connected || pad.mapping !== "standard") continue;
    for (const [index, key] of Object.entries(GAMEPAD_KEYS)) if (pad.buttons[Number(index)]?.pressed) held.add(key);
    const [x = 0, y = 0] = pad.axes;
    if (x <= -STICK_THRESHOLD) held.add("ArrowLeft");
    if (x >= STICK_THRESHOLD) held.add("ArrowRight");
    if (y <= -STICK_THRESHOLD) held.add("ArrowUp");
    if (y >= STICK_THRESHOLD) held.add("ArrowDown");
  }
  return held;
}

/** The key events that move from `before` to `now`. */
export function keyEdges(before: ReadonlySet<string>, now: ReadonlySet<string>): InjectedInput[] {
  const out: InjectedInput[] = [];
  for (const key of now) if (!before.has(key)) out.push({ kind: "key", key, action: "down" });
  for (const key of before) if (!now.has(key)) out.push({ kind: "key", key, action: "up" });
  return out;
}

/** Poll the page's pads while any is connected; returns the stop function. */
export function pollGamepads(send: (event: InjectedInput) => void): () => void {
  const nav = globalThis.navigator as { getGamepads?: () => ReadonlyArray<PadLike | null> } | undefined;
  if (!nav?.getGamepads) return () => {};
  let held = new Set<string>();
  const states = new Map<number, string>();
  let frame = 0;
  let stopped = false;
  const poll = () => {
    frame = 0;
    if (stopped) return;
    const pads = nav.getGamepads!();
    for (const e of padStates(pads, states)) send(e);
    const now = heldKeys(pads);
    for (const e of keyEdges(held, now)) send(e);
    held = now;
    if (nav.getGamepads!().some((p) => p?.connected)) frame = requestAnimationFrame(poll);
  };
  const connected = () => {
    if (!frame) frame = requestAnimationFrame(poll);
  };
  addEventListener("gamepadconnected", connected);
  connected();
  return () => {
    stopped = true;
    removeEventListener("gamepadconnected", connected);
    if (frame) cancelAnimationFrame(frame);
    for (const e of keyEdges(held, new Set())) send(e);
    for (const e of padStates([], states)) send(e);
  };
}
