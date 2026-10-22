// Shared readers and script helpers for Rumpus Isle's playtests. `get:Game:<file>`
// writes the run's resource; `get:<entity>.<component>:<file>` one component of a
// named entity; `dump_ui:<file>` the laid-out interface; `dump_platform:<file>`
// the og.platform calls and their answers. (The form of examples/brasswick/tests/lib.ts.)
import type { PlaytestRun } from "@origingame/engine/playtest";

/** The fields of src/components.ts' Game the playtests read. */
export interface Game {
  phase: number;
  round: number;
  paused: boolean;
  settingsOpen: boolean;
  ticks: number;
  left: number;
  played: number;
  standing: number;
  outs: number;
  leftTitle: boolean;
  launch: number;
  matches: number;
  bestBerries: number;
  bestCrumble: number;
  newRecord: boolean;
  recordsRead: boolean;
  recordsRevision: number;
  saves: number;
  joined0: boolean;
  joined1: boolean;
  joined2: boolean;
  joined3: boolean;
  source0: number;
  source1: number;
  source2: number;
  source3: number;
  colour0: number;
  colour1: number;
  colour2: number;
  colour3: number;
  ready0: boolean;
  ready1: boolean;
  ready2: boolean;
  alive0: boolean;
  alive1: boolean;
  alive2: boolean;
  place0: number;
  place1: number;
  place2: number;
  gained0: number;
  gained1: number;
  gained2: number;
  points0: number;
  points1: number;
  points2: number;
  berries0: number;
  berries1: number;
  lasted0: number;
  lasted1: number;
  x0: number;
  x1: number;
  x2: number;
  y0: number;
  y1: number;
  y2: number;
  z0: number;
  z1: number;
  z2: number;
  bumps: number;
  dashes: number;
  splashes: number;
  berriesTaken: number;
  cracks: number;
  joins: number;
  leaves: number;
  a11yUnnamed: number;
  a11yNodes: number;
}

/** Phase of src/components.ts. */
export const TITLE = 0;
export const LOBBY = 1;
export const INTRO = 2;
export const PLAY = 3;
export const OVER = 4;
export const SCORES = 5;
export const PODIUM = 6;

/** Source of src/components.ts. */
export const PAD0 = 0;
export const PAD1 = 1;
export const KEYS_LEFT = 4;
export const KEYS_RIGHT = 5;

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

/** One og.platform call from a `dump_platform:<file>` step. */
export interface Call {
  method: string;
  args: { key?: string; value?: Record<string, unknown>; revision?: number };
  result?: { ok: boolean; data?: { revision?: number }; error?: { code: string } };
}

export function calls(run: PlaytestRun, file: string): Call[] {
  return run.json<{ calls: Call[] }>(file).calls;
}

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** A key held from t for `seconds`. */
export const hold = (key: string, t: number, seconds: number) => `${t}:key_down:${key};${(t + seconds).toFixed(3)}:key_up:${key}`;

/** A pad's button pressed at t (the whole pad state set, released two ticks later). */
export const pad = (buttons: string, who: number, t: number) => `${t}:gamepad:${buttons}@${who};${(t + 0.034).toFixed(3)}:gamepad:-@${who}`;

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
 * Two players seated by the commands (Pip on WASD, Juno on the arrows) and round
 * `round` (1 to 3) begun without its count-in: play starts a tick after `at + 0.3`.
 * `match` instead of a round starts the whole match at its first round.
 */
export function twoPlayers(at: number, round: number | "match" = "match"): string[] {
  const k = (dt: number, step: string) => `${(at + dt).toFixed(2)}:${step}`;
  return [k(0, "seat:wasd"), k(0.1, "seat:arrows"), k(0.2, round === "match" ? "match" : `round:${round}`), k(0.3, "go")];
}

/**
 * Pip's sweep of the island from his start (2.4, 2.4): left through the middle
 * (three quarters of a second: a critter takes a third of a second to reach its
 * 5.2 m/s and keeps its way after the key is let go), then up, right and down a
 * second each, back about where he began after 3.75 s.
 */
export function sweep(at: number): string[] {
  return [hold("KeyA", at, 0.75), hold("KeyW", at + 0.75, 1), hold("KeyD", at + 1.75, 1), hold("KeyS", at + 2.75, 1)];
}
