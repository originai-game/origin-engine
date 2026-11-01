// Gamepads on the web (main thread): polled once per display frame. Each
// pad's state goes to the engine whole whenever it changes (a `gamepad`
// event: og.Gamepad, pads 0 to 7 by the browser's index), with its family
// (Xbox, PlayStation, Switch, or a generic pad) read from its id, and, as a
// fallback for games that only read keys, as the keys of the mapping below
// (the same for every game; og-ui counts a pad intent and the key it maps
// to once).
//
// A pad the browser maps to the W3C standard layout (`mapping ===
// "standard"`) is read as it is. Any other (common on Firefox, Linux and
// third-party pads) is read through PAD_MAPPINGS, by its vendor and product
// ids, else through GENERIC_MAPPING (the standard indices, best effort), so
// no connected pad is dropped.
//
//   A (0) Space · B (1) Escape · X (2) KeyX · Y (3) KeyZ
//   LB (4) KeyQ · RB (5) KeyE · Back (8) Tab · Start (9) Enter
//   D-pad (12–15) and the left stick (past 0.5) ArrowUp/Down/Left/Right

import { GAMEPAD_BUTTONS, type InjectedInput, type PadDevice } from "./input.ts";

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

/** The most pads the engine reads (og-input's MAX_GAMEPADS). */
export const MAX_PADS = 8;

interface PadLike {
  readonly connected: boolean;
  readonly mapping: string;
  readonly id?: string;
  readonly buttons: ReadonlyArray<{ readonly pressed: boolean; readonly value?: number }>;
  readonly axes: ReadonlyArray<number>;
  readonly index?: number;
}

/**
 * Where a standard input comes from on a raw pad: a button (`b`), or an
 * axis (`a`) times `s` (default 1); `full` reads an axis that rests at -1
 * (a trigger on many Linux drivers) as 0 to 1.
 */
export type PadSource = { readonly b: number } | { readonly a: number; readonly s?: 1 | -1; readonly full?: boolean };

/** How a pad's raw buttons and axes map to the standard layout. */
export interface PadMapping {
  /** For each standard button (GAMEPAD_BUTTONS order), its source, or null. */
  readonly buttons: ReadonlyArray<PadSource | null>;
  /** Left x, y, right x, y (y down). */
  readonly axes: readonly [PadSource, PadSource, PadSource, PadSource];
  /** Left and right analog triggers. */
  readonly triggers: readonly [PadSource, PadSource];
}

const b = (n: number): PadSource => ({ b: n });
const a = (n: number, s: 1 | -1 = 1): PadSource => ({ a: n, s });
const full = (n: number): PadSource => ({ a: n, full: true });

/** The standard indices, for a pad nobody mapped: right for most, close for the rest. */
export const GENERIC_MAPPING: PadMapping = {
  buttons: GAMEPAD_BUTTONS.map((_, i) => b(i)),
  axes: [a(0), a(1), a(2), a(3)],
  triggers: [b(6), b(7)],
};

/**
 * XInput and DirectInput layouts the Linux and Firefox drivers report: A B X Y
 * LB RB, Back Start Home, the stick presses, triggers on axes 2 and 5 resting
 * at -1, right stick on 3 and 4, the d-pad as the hat axes 6 and 7.
 */
const XINPUT_LINUX: PadMapping = {
  buttons: [b(0), b(1), b(2), b(3), b(4), b(5), full(2), full(5), b(6), b(7), b(9), b(10), a(7, -1), a(7), a(6, -1), a(6), b(8)],
  axes: [a(0), a(1), a(3), a(4)],
  triggers: [full(2), full(5)],
};

/** A DualShock 4 or DualSense on the Linux driver: Cross Circle Triangle Square, L2 and R2 as buttons and axes. */
const DUALSHOCK_LINUX: PadMapping = {
  buttons: [b(0), b(1), b(3), b(2), b(4), b(5), b(6), b(7), b(8), b(9), b(11), b(12), a(7, -1), a(7), a(6, -1), a(6), b(10)],
  axes: [a(0), a(1), a(3), a(4)],
  triggers: [full(2), full(5)],
};

/** A DualShock 4 on Firefox for Windows and macOS (DirectInput order; its d-pad is a hat switch, which is not read). */
const DUALSHOCK_DINPUT: PadMapping = {
  buttons: [b(1), b(2), b(0), b(3), b(4), b(5), b(6), b(7), b(8), b(9), b(10), b(11), null, null, null, null, b(12)],
  axes: [a(0), a(1), a(2), a(5)],
  triggers: [full(3), full(4)],
};

/** A Switch Pro Controller or Joy-Con pair outside the standard mapping: B A Y X by position. */
const SWITCH_PRO: PadMapping = {
  buttons: [b(0), b(1), b(2), b(3), b(4), b(5), b(6), b(7), b(8), b(9), b(10), b(11), b(12), b(13), b(14), b(15), b(16)],
  axes: [a(0), a(1), a(2), a(3)],
  triggers: [b(6), b(7)],
};

/** Known pads by vendor and product id (lower-case hex, four digits), first match wins. */
export const PAD_MAPPINGS: ReadonlyArray<{ readonly vendor: string; readonly product?: RegExp; readonly linux?: boolean; readonly mapping: PadMapping }> = [
  { vendor: "045e", mapping: XINPUT_LINUX },
  { vendor: "054c", product: /^(05c4|09cc|0ba0|0ce6|0df2)$/, linux: true, mapping: DUALSHOCK_LINUX },
  { vendor: "054c", product: /^(05c4|09cc|0ba0)$/, mapping: DUALSHOCK_DINPUT },
  { vendor: "057e", mapping: SWITCH_PRO },
  // Third-party XInput pads (Logitech, PDP, PowerA, 8BitDo in X mode).
  { vendor: "046d", mapping: XINPUT_LINUX },
  { vendor: "0e6f", mapping: XINPUT_LINUX },
  { vendor: "24c6", mapping: XINPUT_LINUX },
  { vendor: "2dc8", mapping: XINPUT_LINUX },
];

/** A pad's vendor and product ids from its `id` (Chrome's `Vendor: 054c Product: 09cc`, Firefox's `54c-9cc-…`). */
export function padIds(id: string | undefined): { vendor: string; product: string } | null {
  if (!id) return null;
  const pad4 = (s: string) => s.toLowerCase().padStart(4, "0");
  const chrome = /Vendor:\s*([0-9a-f]{1,4})\s+Product:\s*([0-9a-f]{1,4})/i.exec(id);
  if (chrome) return { vendor: pad4(chrome[1]!), product: pad4(chrome[2]!) };
  const firefox = /^([0-9a-f]{1,4})-([0-9a-f]{1,4})-/i.exec(id);
  if (firefox) return { vendor: pad4(firefox[1]!), product: pad4(firefox[2]!) };
  return null;
}

/** A pad's family from its id: by vendor (Microsoft, Sony, Nintendo), else by name, else a generic pad. */
export function padDevice(id: string | undefined): PadDevice {
  const ids = padIds(id);
  if (ids?.vendor === "045e") return "xbox";
  if (ids?.vendor === "054c") return "playstation";
  if (ids?.vendor === "057e") return "switch";
  const name = (id ?? "").toLowerCase();
  if (/xbox|xinput|x-box/.test(name)) return "xbox";
  if (/playstation|dualshock|dualsense|wireless controller/.test(name)) return "playstation";
  if (/switch|pro controller|joy-con|nintendo/.test(name)) return "switch";
  return "gamepad";
}

/** The mapping a non-standard pad is read through (a standard one needs none). */
export function padMapping(pad: Pick<PadLike, "id" | "mapping">, linux = /linux/i.test(globalThis.navigator?.userAgent ?? "")): PadMapping | null {
  if (pad.mapping === "standard") return null;
  const ids = padIds(pad.id);
  const known = ids && PAD_MAPPINGS.find((m) => m.vendor === ids.vendor && (!m.product || m.product.test(ids.product)) && (m.linux === undefined || m.linux === linux));
  return known?.mapping ?? GENERIC_MAPPING;
}

const value = (pad: PadLike, s: PadSource): number => {
  if ("b" in s) {
    const btn = pad.buttons[s.b];
    return Math.max(btn?.value ?? 0, btn?.pressed ? 1 : 0);
  }
  const v = pad.axes[s.a] ?? 0;
  return s.full ? (v + 1) / 2 : v * (s.s ?? 1);
};

/** A pad as the standard layout: held buttons, sticks (y down), triggers. */
export interface PadRead {
  readonly buttons: boolean[];
  readonly axes: number[];
  readonly triggers: number[];
}

/** One pad read through its mapping (the standard layout as it is). */
export function readPad(pad: PadLike, linux?: boolean): PadRead {
  const mapping = padMapping(pad, linux);
  if (!mapping) {
    return {
      buttons: GAMEPAD_BUTTONS.map((_, i) => pad.buttons[i]?.pressed ?? false),
      axes: [0, 1, 2, 3].map((i) => pad.axes[i] ?? 0),
      triggers: [pad.buttons[6]?.value ?? 0, pad.buttons[7]?.value ?? 0],
    };
  }
  return {
    buttons: mapping.buttons.map((s) => {
      if (!s) return false;
      if ("b" in s) return pad.buttons[s.b]?.pressed ?? false;
      return value(pad, s) > 0.5;
    }),
    axes: mapping.axes.map((s) => value(pad, s)),
    triggers: mapping.triggers.map((s) => Math.max(0, Math.min(1, value(pad, s)))),
  };
}

/** The `gamepad` events that move the engine's pads from `before` to `now` (keyed by a state string). */
export function padStates(pads: ReadonlyArray<PadLike | null>, before: Map<number, string>, linux?: boolean): InjectedInput[] {
  const out: InjectedInput[] = [];
  const seen = new Set<number>();
  pads.forEach((pad, i) => {
    const index = pad?.index ?? i;
    if (!pad || !pad.connected || index >= MAX_PADS) return;
    seen.add(index);
    const round = (v: number | undefined) => Math.round((v ?? 0) * 1000) / 1000;
    const read = readPad(pad, linux);
    const event: InjectedInput = {
      kind: "gamepad",
      pad: index,
      connected: true,
      buttons: GAMEPAD_BUTTONS.filter((_, b) => read.buttons[b]),
      axes: read.axes.map(round),
      triggers: read.triggers.map(round),
      device: padDevice(pad.id),
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

/**
 * Every pad `before` holds, at rest (still connected, nothing pressed, the
 * sticks centred), with `before` forgotten so the next poll sends each pad's
 * state whole: what the shell sends when the window loses focus or the page
 * is hidden, where the browser stops reporting pads and the polling stops.
 */
export function restPads(before: Map<number, string>): InjectedInput[] {
  const out: InjectedInput[] = [...before.keys()].map((pad) => ({ kind: "gamepad", pad, connected: true, buttons: [], axes: [0, 0, 0, 0], triggers: [0, 0] }));
  before.clear();
  return out;
}

/** The keys a set of pads holds now (any pad holding a key holds it). */
export function heldKeys(pads: ReadonlyArray<PadLike | null>, linux?: boolean): Set<string> {
  const held = new Set<string>();
  for (const pad of pads) {
    if (!pad || !pad.connected) continue;
    const read = readPad(pad, linux);
    for (const [index, key] of Object.entries(GAMEPAD_KEYS)) if (read.buttons[Number(index)]) held.add(key);
    const [x = 0, y = 0] = read.axes;
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
  // Focus lost or the page hidden: the pads and the keys they hold let go
  // (nothing stays held while the page cannot see the pad); the next poll
  // sends what is still held.
  const release = () => {
    for (const e of keyEdges(held, new Set())) send(e);
    held = new Set();
    for (const e of restPads(states)) send(e);
  };
  const hidden = () => {
    if (globalThis.document?.visibilityState === "hidden") release();
  };
  addEventListener("gamepadconnected", connected);
  addEventListener("blur", release);
  globalThis.document?.addEventListener("visibilitychange", hidden);
  connected();
  return () => {
    stopped = true;
    removeEventListener("gamepadconnected", connected);
    removeEventListener("blur", release);
    globalThis.document?.removeEventListener("visibilitychange", hidden);
    if (frame) cancelAnimationFrame(frame);
    for (const e of keyEdges(held, new Set())) send(e);
    for (const e of padStates([], states)) send(e);
  };
}

/** A rumble og.input.rumble asked for: [pad (-1 every pad), low, high, ms (0 stops)]. */
export type RumbleRequest = [number, number, number, number];

interface Actuator {
  playEffect?: (kind: string, params: Record<string, number>) => Promise<unknown>;
  reset?: () => Promise<unknown>;
}

/** Plays og.input.rumble's requests on the page's pads (pad i is the browser's index i). */
export function playRumble(requests: readonly RumbleRequest[], pads?: ReadonlyArray<{ connected?: boolean; index?: number; vibrationActuator?: Actuator | null } | null>): number {
  const list = pads ?? (globalThis.navigator as { getGamepads?: () => typeof pads } | undefined)?.getGamepads?.() ?? [];
  let played = 0;
  for (const [pad, low, high, ms] of requests) {
    list.forEach((p, i) => {
      const act = p?.connected ? p.vibrationActuator : null;
      if (!act || (pad >= 0 && (p?.index ?? i) !== pad)) return;
      played++;
      if (ms <= 0) {
        void act.reset?.()?.catch(() => {});
        return;
      }
      void act
        .playEffect?.("dual-rumble", {
          startDelay: 0,
          duration: ms,
          strongMagnitude: Math.min(1, Math.max(0, low)),
          weakMagnitude: Math.min(1, Math.max(0, high)),
        })
        ?.catch(() => {});
    });
  }
  return played;
}
