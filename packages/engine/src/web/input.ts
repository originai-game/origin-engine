// The input ring: the main thread writes page input into a SharedArrayBuffer
// the engine Worker drains once per frame (docs/design.md §2.2). One slot is
// 16 int32: kind, action, button, key length, x, y, deltaX, deltaY (as f32
// bits), then up to 32 bytes of the key code; a pointer slot carries its
// engine pointer id (0 the primary, 1 to 4 further touches) in word 8
// instead. A gamepad slot (kind 3) is a pad's whole state: word 2 the pad,
// word 3 connected, words 4-7 the axes, word 8 the button mask, words 9-10
// the triggers. A key or pointer slot carries the event's time in words
// 6-7 (an f64: performance.timeOrigin + the event's timeStamp, ms; 0 when
// unknown), which the Worker turns into the event's `at` within the tick.
// Without cross-origin isolation the shell posts the same events as
// messages instead. crates/og-input/src/ring.rs decodes the same layout.

/** An event as the probe's input() and the engine's input layer take it (IDL InjectedInput). */
export type InjectedInput =
  | {
      kind: "key";
      key: string;
      action: "press" | "down" | "up";
      /** The page's clock when it happened (performance.timeOrigin + timeStamp, ms): the Worker turns it into `at`. */
      time?: number;
      /** When it happened, in ticks from the start of the tick that reads it (-1 to 1; IDL KeyInput.at). */
      at?: number;
    }
  | {
      kind: "pointer";
      action: "click" | "move" | "down" | "up";
      x: number;
      y: number;
      button: "none" | "left" | "middle" | "right";
      /** Which pointer: 0 the primary (omitted), 1 to 4 further touches (multi-touch). */
      id?: number;
      time?: number;
      at?: number;
    }
  | { kind: "wheel"; x: number; y: number; deltaX: number; deltaY: number }
  | {
      kind: "gamepad";
      pad: number;
      connected: boolean;
      /** Held standard-mapping buttons by name (GAMEPAD_BUTTONS). */
      buttons: GamepadButtonName[];
      /** Left x, y, right x, y; -1..1, y down. */
      axes: number[];
      /** Left and right analog triggers, 0..1. */
      triggers: number[];
    };

export type GamepadButtonName = (typeof GAMEPAD_BUTTONS)[number];

/** The W3C standard mapping's buttons, by index (og-input's GAMEPAD_BUTTONS). */
export const GAMEPAD_BUTTONS = [
  "a", "b", "x", "y", "leftBumper", "rightBumper", "leftTrigger", "rightTrigger",
  "back", "start", "leftStick", "rightStick", "up", "down", "left", "right", "home",
] as const;

const SLOT = 16;
const HEADER = 2; // write index, read index (monotonic)
export const RING_SLOTS = 256;

const KINDS = ["key", "pointer", "wheel", "gamepad"] as const;
const ACTIONS = ["press", "down", "up", "click", "move"] as const;
const BUTTONS = ["none", "left", "middle", "right"] as const;

export function createRing(): SharedArrayBuffer {
  return new SharedArrayBuffer((HEADER + RING_SLOTS * SLOT) * 4);
}

/** Main thread: append one event; false when the ring is full (the event is dropped). */
export function writeRing(buffer: SharedArrayBuffer, event: InjectedInput): boolean {
  const ints = new Int32Array(buffer);
  const floats = new Float32Array(buffer);
  const write = Atomics.load(ints, 0);
  const read = Atomics.load(ints, 1);
  if (write - read >= RING_SLOTS) return false;
  const at = HEADER + (write % RING_SLOTS) * SLOT;
  new Int32Array(buffer, at * 4, SLOT).fill(0);
  ints[at] = KINDS.indexOf(event.kind);
  if (event.kind === "gamepad") {
    ints[at + 2] = event.pad;
    ints[at + 3] = event.connected ? 1 : 0;
    for (let i = 0; i < 4; i++) floats[at + 4 + i] = event.axes[i] ?? 0;
    ints[at + 8] = event.buttons.reduce((m, b) => m | (1 << GAMEPAD_BUTTONS.indexOf(b)), 0);
    floats[at + 9] = event.triggers[0] ?? 0;
    floats[at + 10] = event.triggers[1] ?? 0;
    Atomics.store(ints, 0, write + 1);
    return true;
  }
  ints[at + 1] = "action" in event ? ACTIONS.indexOf(event.action) : 0;
  ints[at + 2] = event.kind === "pointer" ? BUTTONS.indexOf(event.button) : 0;
  const key = event.kind === "key" ? new TextEncoder().encode(event.key).subarray(0, 32) : new Uint8Array(0);
  ints[at + 3] = key.length;
  floats[at + 4] = "x" in event ? event.x : 0;
  floats[at + 5] = "y" in event ? event.y : 0;
  floats[at + 6] = event.kind === "wheel" ? event.deltaX : 0;
  floats[at + 7] = event.kind === "wheel" ? event.deltaY : 0;
  new Uint8Array(buffer, (at + 8) * 4, 32).fill(0).set(key);
  if (event.kind === "pointer") ints[at + 8] = event.id ?? 0;
  if ((event.kind === "key" || event.kind === "pointer") && event.time !== undefined) new Float64Array(buffer, (at + 6) * 4, 1)[0] = event.time;
  Atomics.store(ints, 0, write + 1);
  return true;
}

/** Worker: every event written since the last drain, oldest first. */
export function drainRing(buffer: SharedArrayBuffer): InjectedInput[] {
  const ints = new Int32Array(buffer);
  const floats = new Float32Array(buffer);
  const write = Atomics.load(ints, 0);
  let read = Atomics.load(ints, 1);
  const out: InjectedInput[] = [];
  for (; read < write; read++) {
    const at = HEADER + (read % RING_SLOTS) * SLOT;
    const kind = KINDS[ints[at]!];
    if (kind === "gamepad") {
      const mask = ints[at + 8]!;
      out.push({
        kind,
        pad: ints[at + 2]!,
        connected: ints[at + 3]! !== 0,
        buttons: GAMEPAD_BUTTONS.filter((_, i) => mask & (1 << i)),
        axes: [floats[at + 4]!, floats[at + 5]!, floats[at + 6]!, floats[at + 7]!],
        triggers: [floats[at + 9]!, floats[at + 10]!],
      });
      continue;
    }
    const action = ACTIONS[ints[at + 1]!]!;
    const x = floats[at + 4]!;
    const y = floats[at + 5]!;
    const time = kind === "key" || kind === "pointer" ? new Float64Array(buffer, (at + 6) * 4, 1)[0]! : 0;
    const stamped = time > 0 ? { time } : {};
    if (kind === "key") {
      const bytes = new Uint8Array(buffer, (at + 8) * 4, ints[at + 3]!).slice();
      out.push({ kind, key: new TextDecoder().decode(bytes), action: action as "press" | "down" | "up", ...stamped });
    } else if (kind === "pointer") {
      const id = ints[at + 8]!;
      out.push({
        kind,
        action: action as "click" | "move" | "down" | "up",
        x,
        y,
        button: BUTTONS[ints[at + 2]!]!,
        ...(id > 0 ? { id } : {}),
        ...stamped,
      });
    } else if (kind === "wheel") {
      out.push({ kind, x, y, deltaX: floats[at + 6]!, deltaY: floats[at + 7]! });
    }
  }
  Atomics.store(ints, 1, read);
  return out;
}

/**
 * `at` of an event that happened at `time` (the page's clock, ms) and is
 * injected now, before the tick that reads it: the sim clock is `behind` ms
 * (the accumulator the last frame left, plus what elapsed since) ahead of the
 * last tick's start, so the event falls at `(behind - age) / tickMs` ticks
 * into the next tick, clamped to -1..1. 0 without a time.
 */
export function stampAt(time: number | undefined, now: number, behind: number, tickMs: number): number {
  if (time === undefined || !(time > 0) || !(tickMs > 0)) return 0;
  const at = (behind - (now - time)) / tickMs;
  return Number.isFinite(at) ? Math.max(-1, Math.min(1, at)) : 0;
}
