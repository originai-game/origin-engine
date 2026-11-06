// This device's own, never in the world: the calibrated offset, the
// output's latency, and when within a tick its presses happened. Seats asks
// for them (src/seats.ts's useInput) and they cross the room in the seat's
// og.PlayerInput, so every peer judges each press by its own player's
// device, and no peer's calibration enters the world or its hash:
//   aimX  when the seat's presses this tick happened, in ticks from the
//         start of the tick the input is for (negative: a key's within its
//         tick, less the input's delay; a menu's on the tick it was clicked);
//   aimY  the device's lag in ms: the output's latency (og.audio, written
//         here by the Ui system) plus the calibrated offset.
// The fingers on the lanes are the touch controls (useTouchControls): a
// finger down on a seat's highway presses that lane; on two highways each
// takes its half of the screen.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { creation } from "./creation.ts";
import { LANES } from "./components.ts";
import { bit, LOCAL, localSeats, press, SEATS, useInput, useTouchControls } from "./seats.ts";

/**
 * This device: its calibrated offset, its output's latency (the Ui system
 * writes it), the tick each local seat's menu press was made on, and the
 * highways' layout on its screen (the Ui system's, for the fingers).
 */
export const device = { offsetMs: 0, latencyMs: 0, menuTick: [-1, -1], layout: { laneW: 1.5, spread: 0, hitY: -3.2 } };

/** A menu or HUD button's press for a local seat, as of this tick (the seat's input carries when). */
export function pressMenu(world: World, name: string, seat = 0): void {
  press(name, seat);
  device.menuTick[seat] = world.tick;
}

/** Each local seat's keys (alone, seat 0 takes both sets, as src/seats.ts binds them). */
const seatKeys = [0, 1].map((seat) => new Set(creation.controls.buttons.flatMap((b) => (LOCAL ? [...(b.keys[seat] ?? [])] : [...b.keys[0], ...b.keys[1]])) as string[]));

/** The seat and lane under a CSS pixel: a lane of the one highway, or on two highways the nearest lane of the half's. */
export function laneAt(world: World, px: number, py: number): { seat: number; lane: number } | null {
  const g = device.layout;
  const cam = world.resource(og.Camera);
  const vp = og.ui.viewport(world);
  if (!cam.ok || vp.height <= 0) return null;
  const scale = vp.height / cam.value.height;
  const x = cam.value.position.x + (px - vp.width / 2) / scale;
  const y = cam.value.position.y - (py - vp.height / 2) / scale;
  if (y > g.hitY + 3.5) return null;
  if (SEATS === 1) {
    const lane = Math.round(x / g.laneW) + 1;
    return lane >= 0 && lane < LANES && Math.abs(x - (lane - 1) * g.laneW) <= g.laneW * 0.6 ? { seat: 0, lane } : null;
  }
  const seat = px < vp.width / 2 ? 0 : 1;
  const centre = (seat === 0 ? -1 : 1) * g.spread;
  return { seat, lane: Math.max(0, Math.min(LANES - 1, Math.round((x - centre) / g.laneW) + 1)) };
}

/** The seat a device slot drives now (its own in a room). */
function seatOf(world: World, local: number): number {
  return localSeats(world)[local] ?? local;
}

/** Registers this device's readings with Seats: the fingers on the lanes, and the presses' time and the lag. */
export function useDevice(world: World): void {
  useTouchControls((local) => {
    const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
    const ptr = world.resource(og.Pointers);
    if (!ptr.ok) return out;
    const seat = seatOf(world, local);
    for (let id = 0; id < ptr.value.count(); id++) {
      if (!ptr.value.pressed(id)) continue;
      const at = ptr.value.position(id);
      const hit = laneAt(world, at.x, at.y);
      if (hit !== null && hit.seat === seat) out.buttons |= bit(`lane${hit.lane}`);
    }
    return out;
  });
  useInput((world, local, input) => {
    const lock = world.resource(og.Lockstep);
    const delay = lock.ok && lock.value.running ? lock.value.inputDelay : 1;
    const inp = world.resource(og.Input);
    const ptr = world.resource(og.Pointers);
    // The earliest press this tick: a key's or a finger's event time within it, a menu's tick.
    let at = Infinity;
    if (inp.ok) {
      for (const e of world.events(og.KeyDown)) if (!e.repeat && seatKeys[local]!.has(e.key) && inp.value.pressed(e.key)) at = Math.min(at, e.at);
    }
    if (ptr.ok) {
      const seat = seatOf(world, local);
      for (const e of world.events(og.PointerDown)) {
        const hit = ptr.value.pressed(e.id) ? laneAt(world, e.x, e.y) : null;
        if (hit !== null && hit.seat === seat) at = Math.min(at, e.at);
      }
    }
    const menu = device.menuTick[local]!;
    if (menu >= 0) {
      at = Math.min(at, menu - world.tick);
      device.menuTick[local] = -1;
    }
    input.aimX = (at === Infinity ? 0 : at) - delay;
    input.aimY = device.latencyMs + device.offsetMs;
  });
}
