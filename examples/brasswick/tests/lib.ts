// Shared readers and script helpers for Brasswick's playtests. `get:Game:<file>`
// writes the run's resource; `get:<entity>.<component>:<file>` one component of a
// named entity; `dump_ui:<file>` the laid-out interface; `dump_platform:<file>`
// the og.platform calls and their answers.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  level: number;
  levelsOpen: boolean;
  paused: boolean;
  settingsOpen: boolean;
  time: number;
  gears: number;
  gearMask: number;
  deaths: number;
  stomps: number;
  checkpoint: number;
  checkX: number;
  checkY: number;
  heroX: number;
  heroY: number;
  heroVY: number;
  grounded: boolean;
  onWall: number;
  pushing: boolean;
  hurt: number;
  safe: number;
  exiting: number;
  switches: number;
  jumps: number;
  wallJumps: number;
  landings: number;
  steps: number;
  shoves: number;
  presses: number;
  opens: number;
  lampsLit: number;
  score: number;
  stars: number;
  newBest: boolean;
  unlocked: number;
  best0: number;
  best1: number;
  stars0: number;
  top0: number;
  time0: number;
  saves: number;
  loads: number;
  hasSlot: boolean;
  slotLevel: number;
  revision: number;
  a11yUnnamed: number;
  a11yNodes: number;
}

/** Phase of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const RESULTS = 2;

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

/**
 * The Boiler Stair played by keys alone, from New game to the hatch (about 14 s):
 * a hop for a gear, the spike pit, the first crate shoved against the ledge as a
 * step, the lamp, the second crate onto the plate (the door opens), back for the
 * gear over the ledge, a hop onto the crate and through, then four wall jumps up
 * the shaft and the hatch. Five of the six gears (the high one over the grate is
 * left). `at` shifts it in time.
 */
export function boilerStair(at: number): string[] {
  const k = (t: number, step: string) => `${(t + at).toFixed(2)}:${step}`;
  const steps = [
    k(0.5, "key_down:ArrowRight"),
    k(1.0, "key_down:Space"),
    k(1.1, "key_up:Space"),
    k(1.5, "key_down:Space"),
    k(1.85, "key_up:Space"),
    k(4.0, "key_down:Space"),
    k(4.2, "key_up:ArrowRight"),
    k(4.35, "key_up:Space"),
    k(4.7, "key_down:Space"),
    k(4.75, "key_down:ArrowRight"),
    k(5.05, "key_up:Space"),
    k(6.95, "key_up:ArrowRight"),
    k(7.0, "key_down:ArrowLeft"),
    k(7.35, "key_down:Space"),
    k(7.45, "key_up:ArrowLeft"),
    k(7.65, "key_up:Space"),
    k(8.0, "key_down:ArrowRight"),
    k(8.35, "key_down:Space"),
    k(8.6, "key_up:Space"),
    // The shaft: Wick meets its right wall about 10.5; a jump, then off each wall in turn.
    k(10.6, "key_down:Space"),
    k(10.95, "key_up:Space"),
  ];
  let t = 11.1;
  for (const [down, up] of [["ArrowLeft", "ArrowRight"], ["ArrowRight", "ArrowLeft"], ["ArrowLeft", "ArrowRight"], ["ArrowRight", "ArrowLeft"]] as const) {
    steps.push(k(t, `key_up:${up}`), k(t, `key_down:${down}`), k(t, "key_down:Space"), k(t + 0.2, "key_up:Space"));
    t += 0.4;
  }
  return steps;
}
