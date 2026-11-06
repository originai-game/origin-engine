// Shared readers for Holdout's playtests. `get:Game:<file>` writes the
// run's resource; `get:<entity>.<component>:<file>` one component of a
// named row (seat 0's trooper is `hero`, seat 1's `hero-1`, the foes
// `walker-<i>` and `drone-<i>`); `dump_ui:<file>` the laid-out interface.
// The seats, the view and the players come from ../src/creation.ts, so the
// same tests pass in every scaffold the starter makes.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** The seats this scaffold plays: 1 alone, 2 local or online. */
export const SEATS: number = creation.seats;
const players: string = creation.players;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
const view: string = creation.view;
export const FIRST_PERSON = view === "first-person-3d";
export const THIRD_PERSON = view === "third-person-3d";
export const ISOMETRIC = view === "isometric-2.5d";
/** The frames' prefix for this scaffold: its view, and `-local` for split screen. */
export const VARIANT = `${FIRST_PERSON ? "fp" : THIRD_PERSON ? "tp" : "iso"}${LOCAL ? "-local" : ""}`;
/**
 * How far right of the trooper's line the crosshair's ray starts: over the
 * shoulder (third person) the camera, and so the aim, sits SHOULDER right
 * (src/hero.ts); a foe straight down the line needs it.
 */
export const AIM_RIGHT = THIRD_PERSON ? 0.62 : 0;

export interface Game {
  phase: number;
  mode: number;
  ending: number;
  winner: number;
  paused: boolean;
  time: number;
  wave: number;
  waveState: number;
  toSpawn: number;
  alive: number;
  score: number;
  kills: number;
  headshots: number;
  shots: number;
  hits: number;
  downs: number;
  runs: number;
  wins: number;
  best: number;
}

export interface Hero {
  seat: number;
  health: number;
  down: boolean;
  mag: number;
  reserve: number;
  reload: number;
  kills: number;
  frags: number;
  score: number;
  facing: number;
  speed: number;
  grounded: boolean;
  jumps: number;
}

export interface Foe {
  kind: number;
  state: number;
  health: number;
}

/** Phase.*, Mode.*, Ending.* and WaveState.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const HOLDOUT = 0;
export const DUEL = 1;
export const HELD = 1;
export const OVERRUN = 2;
export const DUEL_WON = 3;
export const FIGHTING = 1;
/** FoeState.Pooled and .Dying. */
export const POOLED = 0;
export const DYING = 4;

/** The summary's buttons answer once its entrance has played (src/ui.ts SUMMARY_TICKS, 110 ticks after the end). */
export const SUMMARY_READY = 2.1;

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

export interface Element { id: string; parent: string; path: string; kind: string; text: string; x: number; y: number; width: number; height: number }

/** The laid-out elements from a `dump_ui:<file>` step. */
export function elements(run: PlaytestRun, file: string): Element[] {
  return run.json<{ elements: Element[] }>(file).elements;
}

/** The UI's labels from a `dump_ui:<file>` step. */
export function texts(run: PlaytestRun, file: string): string[] {
  return elements(run, file).filter((e) => e.text !== "").map((e) => e.text);
}

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** A key held from t for `seconds`. */
export const hold = (key: string, t: number, seconds: number) => `${t}:key_down:${key};${(t + seconds).toFixed(3)}:key_up:${key}`;

/** A tap on a UI element by its id or id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** Every seat's trooper downed at t: a holdout overrun. */
export function downAll(t: number): string {
  return Array.from({ length: SEATS }, (_, s) => `${t}:hurt:500:${s}`).join(";");
}
