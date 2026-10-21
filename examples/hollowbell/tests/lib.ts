// Shared readers for Hollowbell's playtests. `get:Game:<file>` writes the
// run's resource (what the game's `facts` in src/main.ts report to the
// probe); `get:<entity>.<component>:<file>` one component of a named row;
// `dump_ui:<file>` the laid-out interface; `dump_platform:<file>` the
// og.platform calls and their answers.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  naming: boolean;
  paused: boolean;
  settingsOpen: boolean;
  held: boolean;
  hearts: number;
  glimmers: number;
  chimes: number;
  chimeMask: number;
  killMask: number;
  bossDown: boolean;
  bossAwake: boolean;
  bossHp: number;
  wick: boolean;
  cloak: boolean;
  checkpoint: number;
  litMask: number;
  time: number;
  hurts: number;
  kills: number;
  hits: number;
  swings: number;
  falls: number;
  steps: number;
  jumps: number;
  landings: number;
  heroX: number;
  heroY: number;
  facing: number;
  grounded: boolean;
  invulnerable: number;
  knock: number;
  respawn: number;
  nearMole: boolean;
  nearBell: boolean;
  talking: boolean;
  talkIndex: number;
  shopping: boolean;
  talks: number;
  bought: number;
  arena: boolean;
  slowTicks: number;
  ringing: number;
  score: number;
  best: number;
  newBest: boolean;
  area: number;
  ambient: number;
  light: number;
  look: number;
  saves: number;
  loads: number;
  hasSave: boolean;
  conflicts: number;
  musicChunks: number;
  a11yUnnamed: number;
  a11yNodes: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;

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

/** Script steps in time order: the command channel refuses a step earlier than the one before it. */
export function script(...steps: string[]): string {
  const all = steps.flatMap((s) => s.split(";")).filter((s) => s !== "");
  return all
    .map((s, i) => ({ s, i, t: Number(s.slice(0, s.indexOf(":"))) }))
    .sort((a, b) => a.t - b.t || a.i - b.i)
    .map((x) => x.s)
    .join(";");
}
