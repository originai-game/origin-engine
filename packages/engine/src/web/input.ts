// The input ring: the main thread writes page input into a SharedArrayBuffer
// the engine Worker drains once per frame (docs/design.md §2.2). One slot is
// 16 int32: kind, action, button, key length, x, y, deltaX, deltaY (as f32
// bits), then up to 32 bytes of the key code; a pointer slot carries its
// engine pointer id (0 the primary, 1 to 4 further touches) in word 8
// instead. A gamepad slot (kind 3) is a pad's whole state: word 2 the pad,
// word 3 connected, words 4-7 the axes, word 8 the button mask, words 9-10
// the triggers. A text slot (kind 4) carries typed or composed text: word 1
// 0 committed or 1 composing, word 2 1 when it continues the slot before it
// (text past 48 bytes is split between characters over several slots,
// joined again by drainRing), word 3 the UTF-8 length, the text in words
// 4-15. An a11y slot (kind 5) is laid out as a text slot with word 1 the
// action (A11Y_ACTIONS) and the control's path as its text. A pointer slot
// carries the mouse's relative movement (movementX, movementY) as f32 in
// words 9-10; a pointer lock slot (kind 6) has word 1 1 when the pointer is
// locked now, 0 when it was released. A key or pointer slot carries the event's time in words
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
      /** The mouse's relative movement since its last event (PointerEvent.movementX/Y), CSS px; omitted is 0. */
      movementX?: number;
      movementY?: number;
      time?: number;
      at?: number;
    }
  | {
      kind: "pointerLock";
      /** The pointer is locked to the canvas now (pointerlockchange): og.PointerLock.locked. */
      locked: boolean;
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
    }
  | {
      kind: "text";
      /** Typed text (a key's character, a paste, an IME commit), or with `composing` an input method's composition so far (empty ends it). */
      text: string;
      composing?: boolean;
    }
  | {
      kind: "a11y";
      /** What a screen reader asked of the UI control at `path` (a11y.ts): focus it, activate it, or step an adjustable one. */
      action: A11yActionName;
      /** The control's path, as og.ui.a11y() names it. */
      path: string;
    };

export type A11yActionName = (typeof A11Y_ACTIONS)[number];

/** og-input's A11yAct, by index. */
export const A11Y_ACTIONS = ["focus", "activate", "increment", "decrement"] as const;

export type GamepadButtonName = (typeof GAMEPAD_BUTTONS)[number];

/** The W3C standard mapping's buttons, by index (og-input's GAMEPAD_BUTTONS). */
export const GAMEPAD_BUTTONS = [
  "a", "b", "x", "y", "leftBumper", "rightBumper", "leftTrigger", "rightTrigger",
  "back", "start", "leftStick", "rightStick", "up", "down", "left", "right", "home",
] as const;

const SLOT = 16;
const HEADER = 2; // write index, read index (monotonic)
export const RING_SLOTS = 256;

const KINDS = ["key", "pointer", "wheel", "gamepad", "text", "a11y", "pointerLock"] as const;
/** Bytes of text one slot holds (words 4-15). */
const TEXT_BYTES = 48;

/** UTF-8 pieces of at most TEXT_BYTES, split between characters. */
function textPieces(text: string): Uint8Array[] {
  const encoder = new TextEncoder();
  const out: Uint8Array[] = [];
  let piece = "";
  for (const ch of text) {
    if (encoder.encode(piece + ch).length > TEXT_BYTES) {
      out.push(encoder.encode(piece));
      piece = "";
    }
    piece += ch;
  }
  out.push(encoder.encode(piece));
  return out;
}
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
  if (event.kind === "text" || event.kind === "a11y") {
    // An a11y slot is a text slot whose text is the control's path.
    const pieces = textPieces(event.kind === "text" ? event.text : event.path);
    if (write - read + pieces.length > RING_SLOTS) return false;
    pieces.forEach((bytes, k) => {
      const at = HEADER + ((write + k) % RING_SLOTS) * SLOT;
      new Int32Array(buffer, at * 4, SLOT).fill(0);
      ints[at] = KINDS.indexOf(event.kind);
      ints[at + 1] = event.kind === "text" ? (event.composing ? 1 : 0) : A11Y_ACTIONS.indexOf(event.action);
      ints[at + 2] = k > 0 ? 1 : 0;
      ints[at + 3] = bytes.length;
      new Uint8Array(buffer, (at + 4) * 4, TEXT_BYTES).set(bytes);
    });
    Atomics.store(ints, 0, write + pieces.length);
    return true;
  }
  const at = HEADER + (write % RING_SLOTS) * SLOT;
  new Int32Array(buffer, at * 4, SLOT).fill(0);
  ints[at] = KINDS.indexOf(event.kind);
  if (event.kind === "pointerLock") {
    ints[at + 1] = event.locked ? 1 : 0;
    Atomics.store(ints, 0, write + 1);
    return true;
  }
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
  if (event.kind === "pointer") {
    ints[at + 8] = event.id ?? 0;
    floats[at + 9] = event.movementX ?? 0;
    floats[at + 10] = event.movementY ?? 0;
  }
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
    if (kind === "text") {
      const text = new TextDecoder().decode(new Uint8Array(buffer, (at + 4) * 4, ints[at + 3]!).slice());
      const composing = ints[at + 1] === 1;
      const last = out[out.length - 1];
      if (ints[at + 2] === 1 && last?.kind === "text" && (last.composing ?? false) === composing) last.text += text;
      else out.push(composing ? { kind, text, composing } : { kind, text });
      continue;
    }
    if (kind === "a11y") {
      const path = new TextDecoder().decode(new Uint8Array(buffer, (at + 4) * 4, ints[at + 3]!).slice());
      const action = A11Y_ACTIONS[ints[at + 1]!] ?? "focus";
      const last = out[out.length - 1];
      if (ints[at + 2] === 1 && last?.kind === "a11y" && last.action === action) last.path += path;
      else out.push({ kind, action, path });
      continue;
    }
    if (kind === "pointerLock") {
      out.push({ kind, locked: ints[at + 1] === 1 });
      continue;
    }
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
      const movementX = floats[at + 9]!;
      const movementY = floats[at + 10]!;
      out.push({
        kind,
        action: action as "click" | "move" | "down" | "up",
        x,
        y,
        button: BUTTONS[ints[at + 2]!]!,
        ...(id > 0 ? { id } : {}),
        ...(movementX !== 0 || movementY !== 0 ? { movementX, movementY } : {}),
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
