// Shared readers for Redline Ring's playtests. `get:Game:<file>` writes the
// race's resource; `get:car-<i>.Car:<file>` one car's state (car-0 and
// car-1 are the seats', rival-2 to rival-5 the field's); `dump_ui:<file>`
// the laid-out interface; `dump_platform:<file>` the calls to og.platform.
// The seat count comes from ../src/creation.ts, so every scaffold (alone,
// two on one device, two over a room) runs the same scripts.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** The seats this scaffold plays: 1 alone, 2 on one device or over a room. */
export const SEATS: number = creation.seats;
const players: string = creation.players;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
const view: string = creation.view;
export const ISOMETRIC = view === "isometric-2.5d";
export const FIRST_PERSON = view === "first-person-3d";
/** The player chose touch: the touch controls show on every screen. */
export const TOUCH = (creation.input as string) === "touch";

export interface Game {
  phase: number;
  paused: boolean;
  held: boolean;
  clock: number;
  firstFinish: number;
  lastSeatFinish: number;
  closed: boolean;
  bumps: number;
}

export interface Car {
  index: number;
  seat: number;
  steer: number;
  throttle: number;
  brake: number;
  heading: number;
  speed: number;
  slip: number;
  drifting: boolean;
  boosting: boolean;
  boost: number;
  hits: number;
  along: number;
  across: number;
  laps: number;
  checkpoint: number;
  lastLap: number;
  bestLap: number;
  finish: number;
  out: boolean;
  place: number;
  wrongWay: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const RACE = 1;
export const RESULTS = 2;
/** The countdown, in seconds (COUNTDOWN ticks). */
export const COUNTDOWN_S = 3;
/** From the race's close until the results answer Throttle and Menu (RESULTS_DELAY + RESULTS_READY ticks), seconds. */
export const RESULTS_READY_S = (150 + 40) / 60;

/** The value of a `get:<target>:<file>` step. */
export function value<T>(run: PlaytestRun, file: string): T {
  return run.json<{ value: T }>(file).value;
}
export const game = (run: PlaytestRun, file: string) => value<Game>(run, file);
export const car = (run: PlaytestRun, file: string) => value<Car>(run, file);

/** A seat's car entity name, or a rival's by its grid slot. */
export const carName = (slot: number) => (slot < SEATS ? `car-${slot}` : `rival-${slot}`);

export interface PlatformDump {
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data?: Record<string, unknown>; error?: { code: string } } }>;
}

export interface Element { id: string; parent: string; path: string; kind: string; text: string; x: number; y: number; width: number; height: number }

/** The laid-out elements from a `dump_ui:<file>` step. */
export function elements(run: PlaytestRun, file: string): Element[] {
  return run.json<{ elements: Element[] }>(file).elements;
}

/** The UI's labels from a `dump_ui:<file>` step. */
export function texts(run: PlaytestRun, file: string): string[] {
  return elements(run, file).filter((e) => e.kind === "label" && e.text !== "").map((e) => e.text);
}

const at = (t: number) => t.toFixed(3);
/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${at(t)}:key_down:${key};${at(t + 0.034)}:key_up:${key}`;
/** A key held from t for `seconds`. */
export const hold = (key: string, t: number, seconds: number) => `${at(t)}:key_down:${key};${at(t + seconds)}:key_up:${key}`;
/** A tap on a UI element by its id path (pressed, released a few ticks later). */
export const tap = (id: string, t: number) => `${at(t)}:touch_move_to:${id};${at(t + 0.017)}:touch_start;${at(t + 0.05)}:touch_end`;
/** A second finger holding a UI element from t for `seconds`. */
export const finger = (id: string, t: number, seconds: number, n = 1) => `${at(t)}:touch:${n}:down:${id};${at(t + seconds)}:touch:${n}:up:${id}`;

/** The script's steps in time order (a step's time is its first field). */
export function sorted(steps: string[]): string {
  return steps.flatMap((s) => s.split(";")).map((s, i) => ({ s, i, t: Number(s.split(":")[0]) })).sort((a, b) => a.t - b.t || a.i - b.i).map((x) => x.s).join(";");
}
