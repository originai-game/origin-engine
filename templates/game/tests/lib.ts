// Shared readers for the yard's playtests. The scaffold's choices come
// from ../src/creation.ts (the view, the seats, the player count), so every
// playtest plays the game this scaffold was made as: `get:<target>:<file>`
// writes one component or resource, `dump_entity:<name>:<file>` a row.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** The view this scaffold draws (src/creation.ts). */
export const VIEW: string = creation.view;
export const FIRST_PERSON = VIEW === "first-person-3d";
export const ISOMETRIC = VIEW === "isometric-2.5d";
/** The seats it plays: 1 alone, 2 local or online. */
export const SEATS: number = creation.seats;
const players: string = creation.players;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
/** The isometric ArtCamera's angles, degrees. */
export const ISO = { pitch: (creation.camera as { pitch?: number }).pitch ?? 35.264, yaw: (creation.camera as { yaw?: number }).yaw ?? 45 };

export type V3 = [number, number, number];

/** The value of a `get:<target>:<file>` step. */
export function value<T>(run: PlaytestRun, file: string): T {
  return run.json<{ value: T }>(file).value;
}

/** One component of a `dump_entity` file. */
export function component<T>(run: PlaytestRun, file: string, name: string): T {
  const c = run.json<{ components: Record<string, unknown> }>(file).components[name];
  if (c === undefined) throw new Error(`${file} has no ${name}`);
  return c as T;
}

export function position(run: PlaytestRun, file: string): V3 {
  return component<{ position: V3 }>(run, file, "og.Transform").position;
}

/** Where a pose looks: its -Z turned by its quaternion. */
export function forward(q: readonly number[]): V3 {
  const [x, y, z, w] = q as [number, number, number, number];
  return [-2 * (x * z + w * y), -2 * (y * z - w * x), -(1 - 2 * (x * x + y * y))];
}

/** The angle in degrees between two directions. */
export function angle(a: V3, b: V3): number {
  const la = Math.sqrt(a[0] ** 2 + a[1] ** 2 + a[2] ** 2);
  const lb = Math.sqrt(b[0] ** 2 + b[1] ** 2 + b[2] ** 2);
  const c = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb);
  return (Math.acos(Math.max(-1, Math.min(1, c))) * 180) / Math.PI;
}

/** A tap on a UI element by its id (`Start`, `Again`). */
export const tap = (t: number, id: string) => `${t}:touch_move_to:${id};${(t + 0.1).toFixed(2)}:touch_start;${(t + 0.2).toFixed(2)}:touch_end`;

/** The Mission component (the run) from a `get:dropship.Mission:<file>` step. */
export interface Mission {
  phase: number;
  ending: number;
  cells: number;
  time: number;
  paused: boolean;
  catches: number;
  jumps: number;
  runs: number;
  wins: number;
}
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const ESCAPED = 1;
export const CAUGHT = 2;
