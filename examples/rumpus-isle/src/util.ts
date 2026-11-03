// Small helpers every file uses.

import { og } from "@origingame/engine";
import type { Color, World } from "@origingame/engine";

/** A Result's value, or a throw naming its reason (setup code only: systems handle refusals). */
export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** An entity by name, or a throw. */
export function named(world: World, name: string) {
  const e = world.find(name);
  if (e === null) throw new Error(`no entity named ${name}`);
  return e;
}

/** A string from the active table (og-i18n §2). */
export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

/** The game's screen documents (assets/ui/), loaded once at the start. */
export const SCREENS = ["main-menu", "pause", "settings"] as const;

/**
 * A screen document shown while `on`: mounted (pushed when it is not on the layer stack) and built here with its
 * viewmodel, answering the events it emitted this tick (Back's too); taken off the stack when it is no longer wanted.
 */
export function screen(world: World, name: (typeof SCREENS)[number], on: boolean, vm: Record<string, unknown> = {}): string[] {
  if (!on) {
    while (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** "#rrggbb" (sRGB) to a linear colour. */
export function linear(hex: string): Color {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return { r: c[0]!, g: c[1]!, b: c[2]! };
}

/** A rotation about +y by `yaw` radians (0 faces +z). */
export function yawQuat(yaw: number) {
  return { x: 0, y: Math.sin(yaw / 2), z: 0, w: Math.cos(yaw / 2) };
}

/** m:ss, or s.s under a minute. */
export function clock(seconds: number): string {
  const s = Math.max(0, Math.ceil(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}
