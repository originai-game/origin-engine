// Gamepads on the web (main thread): polled once per display frame. Each
// pad goes to the engine raw and whole whenever it changes (a `gamepad`
// event): its buttons' values and its axes in the browser's order, the
// browser's layout (`standard` when it maps the pad to the W3C layout,
// `web` when it does not), and an SDL GUID made from the vendor and product
// ids in its id. og-input maps it to og.Gamepad's standard layout and tells
// its family (crates/og-input/src/padmap.rs); the shell maps nothing. Pad
// slots are the browser's indices, which a pad keeps while it is connected.

import { RAW_AXES, RAW_BUTTONS, type InjectedInput } from "./input.ts";

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

const NO_GUID = "0".repeat(32);

/**
 * A pad's SDL GUID (USB bus, little-endian vendor and product) from its
 * id: Chrome's `Vendor: 054c Product: 09cc`, Firefox's `54c-9cc-…`. An
 * XInput pad on Windows Chrome names no ids: its vendor is Microsoft's,
 * whose protocol it speaks. Any other pad without ids has an empty GUID.
 */
export function padGuid(id: string | undefined): string {
  const hex = (n: number) => n.toString(16).padStart(2, "0");
  const le = (s: string) => {
    const n = parseInt(s, 16);
    return hex(n & 0xff) + hex(n >> 8);
  };
  const guid = (vendor: string, product: string) => `03000000${le(vendor)}0000${le(product)}0000` + "0".repeat(8);
  const chrome = /Vendor:\s*([0-9a-f]{1,4})\s+Product:\s*([0-9a-f]{1,4})/i.exec(id ?? "");
  if (chrome) return guid(chrome[1]!, chrome[2]!);
  const firefox = /^([0-9a-f]{1,4})-([0-9a-f]{1,4})-/i.exec(id ?? "");
  if (firefox) return guid(firefox[1]!, firefox[2]!);
  if (/xinput/i.test(id ?? "")) return guid("045e", "0");
  return NO_GUID;
}

const round = (v: number | undefined) => Math.round((v ?? 0) * 1000) / 1000;

/** The `gamepad` events that move the engine's pads from `before` to `now` (keyed by a state string). */
export function padStates(pads: ReadonlyArray<PadLike | null>, before: Map<number, string>): InjectedInput[] {
  const out: InjectedInput[] = [];
  const seen = new Set<number>();
  pads.forEach((pad, i) => {
    const index = pad?.index ?? i;
    if (!pad || !pad.connected || index >= MAX_PADS) return;
    seen.add(index);
    const event: InjectedInput = {
      kind: "gamepad",
      pad: index,
      connected: true,
      layout: pad.mapping === "standard" ? "standard" : "web",
      guid: padGuid(pad.id),
      buttons: pad.buttons.slice(0, RAW_BUTTONS).map((b) => round(Math.max(b.value ?? 0, b.pressed ? 1 : 0))),
      axes: pad.axes.slice(0, RAW_AXES).map(round),
      hat: 0,
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
    out.push({ kind: "gamepad", pad: index, connected: false, layout: "standard", guid: NO_GUID, buttons: [], axes: [], hat: 0 });
  }
  return out;
}

/** Poll the page's pads while any is connected; returns the stop function. */
export function pollGamepads(send: (event: InjectedInput) => void): () => void {
  const nav = globalThis.navigator as { getGamepads?: () => ReadonlyArray<PadLike | null> } | undefined;
  if (!nav?.getGamepads) return () => {};
  const states = new Map<number, string>();
  let frame = 0;
  let stopped = false;
  const poll = () => {
    frame = 0;
    if (stopped) return;
    const pads = nav.getGamepads!();
    for (const e of padStates(pads, states)) send(e);
    if (nav.getGamepads!().some((p) => p?.connected)) frame = requestAnimationFrame(poll);
  };
  const connected = () => {
    if (!frame) frame = requestAnimationFrame(poll);
  };
  // Focus lost or the page hidden: the browser stops reporting pads, so
  // they go (a raw pad has no rest state the shell could send: a trigger
  // axis rests at -1); the next poll sends each one still there whole.
  const release = () => {
    for (const e of padStates([], states)) send(e);
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
    release();
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
