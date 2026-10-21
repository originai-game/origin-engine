// Shared readers for Meridian Loop's playtests (the shape of
// examples/kestrel-stair/tests/lib.ts). `get:Race:<file>` writes the run's
// resource (what `facts` in src/main.ts report to the probe);
// `get:<entity>.<Component>:<file>` one component of a named entity;
// `dump_ui:<file>` the laid-out interface; `dump_platform:<file>` the
// og.platform calls and their answers.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Race {
  phase: number;
  paused: boolean;
  settingsOpen: boolean;
  boardOpen: boolean;
  countdown: number;
  clock: number;
  lap: number;
  lapStart: number;
  split1: number;
  split2: number;
  lap1: number;
  lap2: number;
  lap3: number;
  runBest: number;
  bestS1: number;
  bestS2: number;
  bestS3: number;
  record: number;
  recordS1: number;
  recordS2: number;
  recordS3: number;
  recordPoints: number;
  delta: number;
  deltaAt: number;
  valid: boolean;
  newRecord: boolean;
  total: number;
  finishedFor: number;
  at: number;
  wrongWay: number;
  offTicks: number;
  laps: number;
  resets: number;
  bumps: number;
  jumps: number;
  landings: number;
  airTicks: number;
  longestAir: number;
  skidTicks: number;
  splits: number;
  curLen: number;
  bestLen: number;
  runLen: number;
  ghostActive: boolean;
  replaying: boolean;
  replays: number;
  replayMatched: number;
  replayTotal: number;
  snapBytes: number;
  ghostTick: number;
  autopilot: boolean;
  kmh: number;
  inSteer: number;
  inThrottle: number;
  gear: number;
  rpm: number;
  slip: number;
  skidding: boolean;
  airborne: boolean;
  rank: number;
  boardState: number;
  board: string;
  saves: number;
  loaded: boolean;
  conflicts: number;
  a11yNodes: number;
  a11yUnnamed: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const COUNTDOWN = 1;
export const RACING = 2;
export const FINISHED = 3;

/** The Race resource from a `get:Race:<file>` step. */
export function race(run: PlaytestRun, file: string): Race {
  return run.json<{ value: Race }>(file).value;
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

export function dist(a: readonly number[], b: readonly number[]): number {
  return Math.hypot(a[0]! - b[0]!, (a[1] ?? 0) - (b[1] ?? 0), (a[2] ?? 0) - (b[2] ?? 0));
}

/** og.Transform from a `get:<entity>.og.Transform:<file>` step: [x, y, z]. */
export function position(run: PlaytestRun, file: string): number[] {
  return value<{ position: number[] }>(run, file).position;
}
