// Shared readers for The Kestrel Stair's playtests. `get:Game:<file>` writes
// the run's resource (what `facts` in src/main.ts report to the probe);
// `get:<entity>.<Component>:<file>` one component of a named row;
// `dump_ui:<file>` the laid-out interface; `dump_ai` / `dump_nav` an agent;
// `dump_platform:<file>` the og.platform calls and their answers.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  ending: number;
  chapter: number;
  paused: boolean;
  settingsOpen: boolean;
  bagOpen: boolean;
  chaptersOpen: boolean;
  recordsOpen: boolean;
  held: boolean;
  scene: number;
  line: number;
  tutorial: boolean;
  tutorialStep: number;
  hp: number;
  sigils: number;
  herbs: number;
  herbsUsed: number;
  foesDown: number;
  kills: number;
  hits: number;
  hurts: number;
  swings: number;
  jumps: number;
  landings: number;
  steps: number;
  time: number;
  hour: number;
  reached: number;
  nearKind: number;
  invulnerable: number;
  knock: number;
  attack: number;
  facing: number;
  yaw: number;
  grounded: boolean;
  speed: number;
  heroX: number;
  heroY: number;
  heroZ: number;
  gateOpen: boolean;
  gateLift: number;
  inCrypt: boolean;
  capeHang: number;
  capeBehind: number;
  bannerSway: number;
  lampDrop: number;
  reverb: number;
  look: number;
  musicChunks: number;
  saves: number;
  loads: number;
  hasSave: boolean;
  savePlace: number;
  saveParts: number;
  saveBytes: number;
  chapterStars: number;
  chaptersReached: number;
  tutorialDone: boolean;
  best: number;
  bestScore: number;
  newBest: boolean;
  score: number;
  submitted: number;
  rank: number;
  recordsState: number;
  records: string;
  bag: string;
  bagMoves: number;
  refusedAt: number;
  a11yUnnamed: number;
  a11yNodes: number;
}

export interface Foe {
  kind: number;
  hp: number;
  stun: number;
  slam: number;
  dying: number;
  hitsTaken: number;
  bites: number;
  alerts: number;
}

/** Phase.*, Ending.*, Scene.* and Near.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const ENDING = 2;
export const OVER = 3;
export const RUNG = 1;
export const NIGHT = 2;
export const FALLEN = 3;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

/** The value of a `get:<target>:<file>` step. */
export function value<T>(run: PlaytestRun, file: string): T {
  return run.json<{ value: T }>(file).value;
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

/** The distance between two positions. */
export function dist(a: readonly number[], b: readonly number[]): number {
  return Math.hypot(a[0]! - b[0]!, (a[1] ?? 0) - (b[1] ?? 0), (a[2] ?? 0) - (b[2] ?? 0));
}
