// Shared readers for Greywatch Keep's playtests. `get:Game:<file>` writes
// the run's resource; `get:<entity>.<component>:<file>` one component of a
// named row (`hero.Hero`, `foe-0.Foe`); `dump_ui:<file>` the laid-out
// interface; `dump_platform:<file>` the calls to og.platform. The seats,
// the view and the input come from ../src/creation.ts, so every scaffold's
// playtests check what it plays.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** The seats this scaffold plays (src/creation.ts): 1 alone, 2 as a party. */
export const SEATS: number = creation.seats;
const players: string = creation.players;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
export const VIEW: string = creation.view;
export const ISOMETRIC = VIEW === "isometric-2.5d";
export const FIRST_PERSON = VIEW === "first-person-3d";
const input: string = creation.input;
export const TOUCH = input === "touch";
/** What the frames of this scaffold are named after: its view, and local or phone. */
export const FLAVOUR = `${VIEW}${LOCAL ? "-local" : ""}${TOUCH ? "-phone" : ""}`;

export interface Game {
  phase: number;
  ending: number;
  paused: boolean;
  settingsOpen: boolean;
  bagOpen: boolean;
  runs: number;
  wins: number;
  best: number;
  newBest: boolean;
  recordConflicts: number;
  time: number;
  quest: number;
  slain: number;
  gold: number;
  potions: number;
  seal: boolean;
  chestOpen: boolean;
  conversation: number;
  talker: number;
  hits: number;
  hurts: number;
  drunk: number;
  levelUps: number;
  target: number;
}

export interface Hero {
  seat: number;
  hp: number;
  maxHp: number;
  level: number;
  xp: number;
  facing: number;
  attack: number;
  dead: boolean;
  respawn: number;
  near: number;
  speed: number;
  swings: number;
  kills: number;
}

export interface Foe {
  index: number;
  kind: number;
  hp: number;
  maxHp: number;
  attack: number;
  dying: number;
  alert: boolean;
  strikes: number;
}

/** Phase.*, Ending.* and Quest.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const WON = 1;
export const FALLEN = 2;
export const Q_NONE = 0;
export const Q_CLEAR = 1;
export const Q_FETCH = 2;
export const Q_RETURN = 3;
export const Q_DONE = 4;

/** The summary's buttons answer once its entrance has played (about 1.85 s after the run ends). */
export const SUMMARY_READY = 1.9;

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

export interface PlatformDump {
  host: string;
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data?: Record<string, unknown>; error?: { code: string } } }>;
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

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/**
 * Talks Maren into the quest from t with seat 0's keys (the hero by her):
 * E starts the conversation, 1 asks what happened, E moves on, 1 agrees,
 * E closes. Answers the commands and the time it ends.
 */
export function takeQuest(t: number): { commands: string[]; end: number } {
  const steps: [string, number][] = [["KeyE", 0], ["Digit1", 0.6], ["KeyE", 1.2], ["Digit1", 1.8], ["KeyE", 2.4]];
  return { commands: steps.map(([k, dt]) => press(k, +(t + dt).toFixed(3))), end: t + 2.8 };
}

/** The steps (each `t:...`, or several joined by `;`) in time order, joined: a script's times must ascend. */
export function script(...steps: string[]): string {
  const all = steps.flatMap((s) => s.split(";")).filter((s) => s !== "");
  return all.map((s, i) => ({ s, i, t: Number(s.slice(0, s.indexOf(":"))) })).sort((a, b) => a.t - b.t || a.i - b.i).map((x) => x.s).join(";");
}
