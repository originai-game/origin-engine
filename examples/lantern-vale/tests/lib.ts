// Shared readers for Lantern Vale's playtests. `get:Game:<file>` writes the
// run's resource (what the game's `facts` in src/main.ts report to the
// probe); `get:<entity>.<component>:<file>` one component of a named row;
// `dump_ui:<file>` the laid-out interface; `dump_stream:<file>` the
// streaming groups.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  ending: number;
  paused: boolean;
  settingsOpen: boolean;
  hearts: number;
  hurts: number;
  best: number;
  newBest: boolean;
  lit: number;
  litMask: number;
  time: number;
  hour: number;
  near: number;
  kindle: number;
  kindling: boolean;
  grounded: boolean;
  speed: number;
  steps: number;
  invulnerable: number;
  look: number;
  far: number;
  farLoads: number;
  farUsed: number;
  musicChunks: number;
  rockMoved: number;
  a11yUnnamed: number;
  a11yNodes: number;
  styled: boolean;
}

/** Phase.* and Ending.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const LIT = 1;
export const NIGHT = 2;
export const CAUGHT = 3;
/** StreamState's order (StreamEvent.state). */
export const RESIDENT = 3;
export const UNLOADED = 0;

/** The results' buttons answer once their entrance has played (OVER_DELAY, 75 ticks, after the end, then about 1.4 s). */
export const RESULTS_READY = 2.8;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

/** The value of a `get:<target>:<file>` step. */
export function value<T>(run: PlaytestRun, file: string): T {
  return run.json<{ value: T }>(file).value;
}

/** A position from a `get:<entity>.og.Transform:<file>` step. */
export function position(run: PlaytestRun, file: string): [number, number, number] {
  return value<{ position: [number, number, number] }>(run, file).position;
}

export interface Element { id: string; parent: string; kind: string; text: string; role: string; name: string; states: string[]; x: number; y: number; width: number; height: number }

/** The laid-out elements from a `dump_ui:<file>` step. */
export function elements(run: PlaytestRun, file: string): Element[] {
  return run.json<{ elements: Element[] }>(file).elements;
}

/** The UI's texts from a `dump_ui:<file>` step. */
export function texts(run: PlaytestRun, file: string): string[] {
  return elements(run, file).filter((e) => e.text !== "").map((e) => e.text);
}

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** A key held from t for `seconds`. */
export const hold = (key: string, t: number, seconds: number) => `${t}:key_down:${key};${(t + seconds).toFixed(3)}:key_up:${key}`;

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;
