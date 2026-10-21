// The seats live in Game as numbered fields (joined0..joined3, ...): a
// component has no array field (og-core §4), and every system and the UI read
// the four seats together. These read and write one seat's field by number.

import type { Value } from "@origingame/engine";
import { Game, MAX_PLAYERS } from "./components.ts";

type GameV = Value<typeof Game>;

export interface Seat {
  joined: boolean;
  source: number;
  colour: number;
  ready: boolean;
  alive: boolean;
  place: number;
  gained: number;
  points: number;
  berries: number;
  lasted: number;
  x: number;
  z: number;
  y: number;
}

type Field = keyof Seat;
const FIELDS: readonly Field[] = ["joined", "source", "colour", "ready", "alive", "place", "gained", "points", "berries", "lasted", "x", "z", "y"];

/** One seat's fields, read now (a plain copy: write through `put`). */
export function seat(g: GameV, k: number): Seat {
  const out = {} as Record<Field, number | boolean>;
  const r = g as unknown as Record<string, number | boolean>;
  for (const f of FIELDS) out[f] = r[`${f}${k}`]!;
  return out as unknown as Seat;
}

/** Writes some of a seat's fields. */
export function put(g: GameV, k: number, s: Partial<Seat>): void {
  const r = g as unknown as Record<string, number | boolean>;
  for (const [f, v] of Object.entries(s)) r[`${f}${k}`] = v as number | boolean;
}

/** The seats, 0 to 3. */
export function seats(g: GameV): Seat[] {
  return Array.from({ length: MAX_PLAYERS }, (_, k) => seat(g, k));
}

/** The seats someone sits in. */
export function joined(g: GameV): number[] {
  return seats(g).flatMap((s, k) => (s.joined ? [k] : []));
}
