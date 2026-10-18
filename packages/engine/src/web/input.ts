// The input ring: the main thread writes page input into a SharedArrayBuffer
// the engine Worker drains once per frame (docs/design.md §2.2). One slot is
// 16 int32: kind, action, button, key length, x, y, deltaX, deltaY (as f32
// bits), then up to 32 bytes of the key code; a pointer slot carries its
// pointerId in word 8 and its type (0 mouse, 1 touch, 2 pen) in word 9
// instead. Without cross-origin isolation the shell posts the same events as
// messages instead. The engine's input layer reads one pointer and ignores
// the id and type (crates/og-input/README.md); they ride along for the IDL
// change that will take them.

/** An event as the probe's input() and the engine's input layer take it (IDL InjectedInput). */
export type InjectedInput =
  | { kind: "key"; key: string; action: "press" | "down" | "up" }
  | {
      kind: "pointer";
      action: "click" | "move" | "down" | "up";
      x: number;
      y: number;
      button: "none" | "left" | "middle" | "right";
      /** The page's PointerEvent.pointerId (multi-touch), when it came from the page. */
      pointerId?: number;
      pointerType?: "mouse" | "touch" | "pen";
    }
  | { kind: "wheel"; x: number; y: number; deltaX: number; deltaY: number };

const SLOT = 16;
const HEADER = 2; // write index, read index (monotonic)
export const RING_SLOTS = 256;

const KINDS = ["key", "pointer", "wheel"] as const;
const ACTIONS = ["press", "down", "up", "click", "move"] as const;
const BUTTONS = ["none", "left", "middle", "right"] as const;
const POINTER_TYPES = ["mouse", "touch", "pen"] as const;

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
  ints[at] = KINDS.indexOf(event.kind);
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
    ints[at + 8] = event.pointerId ?? -1;
    ints[at + 9] = POINTER_TYPES.indexOf(event.pointerType ?? "mouse");
  }
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
    const action = ACTIONS[ints[at + 1]!]!;
    const x = floats[at + 4]!;
    const y = floats[at + 5]!;
    if (kind === "key") {
      const bytes = new Uint8Array(buffer, (at + 8) * 4, ints[at + 3]!).slice();
      out.push({ kind, key: new TextDecoder().decode(bytes), action: action as "press" | "down" | "up" });
    } else if (kind === "pointer") {
      const id = ints[at + 8]!;
      const type = POINTER_TYPES[ints[at + 9]!];
      out.push({
        kind,
        action: action as "click" | "move" | "down" | "up",
        x,
        y,
        button: BUTTONS[ints[at + 2]!]!,
        ...(id >= 0 ? { pointerId: id } : {}),
        ...(id >= 0 && type ? { pointerType: type } : {}),
      });
    } else if (kind === "wheel") {
      out.push({ kind, x, y, deltaX: floats[at + 6]!, deltaY: floats[at + 7]! });
    }
  }
  Atomics.store(ints, 1, read);
  return out;
}
