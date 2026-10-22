// Shared readers for Tomb Room's playtests: `dump_entity:<name>:<file>`
// writes an entity's components; the hero's Report carries the run's facts
// (src/components.ts says why).
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Report {
  room: number;
  hp: number;
  coins: number;
  hasKey: boolean;
  doorOpen: boolean;
  enemyHp: number;
  hits: number;
  swings: number;
  steps: number;
  hurts: number;
  facing: number;
  doorWall: number;
}

interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

/** One component of a dumped entity. */
export function component<T>(run: PlaytestRun, file: string, name: string): T {
  const dump = run.json<EntityDump>(file);
  const c = dump.components[name];
  if (!c) throw new Error(`${file} has no ${name}: ${Object.keys(dump.components).join(", ")}`);
  return c as unknown as T;
}

/** The hero's Report from a `dump_entity:hero` file. */
export function report(run: PlaytestRun, file: string): Report {
  return component<Report>(run, file, "Report");
}

/** A dumped entity's position. */
export function position(run: PlaytestRun, file: string): { x: number; y: number } {
  const p = component<{ position: [number, number, number] }>(run, file, "og.Transform").position;
  return { x: p[0], y: p[1] };
}

/** A dumped entity's scale. */
export function scale(run: PlaytestRun, file: string): { x: number; y: number } {
  const s = component<{ scale: [number, number, number] }>(run, file, "og.Transform").scale;
  return { x: s[0], y: s[1] };
}

/** Facing.* of src/components.ts. */
export const DOWN = 0;
export const UP = 1;
export const LEFT = 2;
export const RIGHT = 3;

/** A key held from t0 to t1 seconds. */
export const hold = (key: string, t0: number, t1: number) => `${t0}:key_down:${key};${t1}:key_up:${key}`;

/** A press (down, up two ticks later): Space swings, ShiftLeft dashes. */
export const press = (key: string, t: number) => hold(key, t, Number((t + 0.034).toFixed(3)));

/** Where the map puts things (world units: the map's bottom-left corner is the origin, y up). */
export const MAP = {
  spawn: { x: 4.5, y: 5.5 },
  slime: { x: 13.5, y: 6.5 },
  key: { x: 16.5, y: 2.5 },
  /** The doorway: cells 19–20, rows 5–6, so x 19..21, y 5..7. */
  door: { x: 20, y: 6 },
  /** The east room starts here. */
  split: 20,
} as const;
