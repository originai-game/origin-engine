// Shared readers for Saltwind Isle's playtests. `get:Game:<file>` writes the
// run's resource, the state the game's `facts` (src/main.ts) report to the
// probe; `get:<entity>.<component>:<file>` one component of a named row;
// `dump_ui:<file>` the laid-out interface; `dump_platform:<file>` the calls
// to og.platform. The scaffold's choices (src/creation.ts) say how many
// seats play and through which view, so one script plays every scaffold.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { SITES } from "../src/layout.ts";

/** The seats this scaffold plays: 1 alone, 2 with others. */
export const SEATS: number = creation.seats;
const players: string = creation.players;
export const LOCAL = players === "local-multiplayer";
export const ONLINE = players === "online-lockstep";
export const VIEW: string = creation.view;
/** The isometric view's fixed heading (src/view.ts): its stick walks along it. */
const ISO_YAW = (((creation.camera as { yaw?: number }).yaw ?? 45) * Math.PI) / 180;

export interface Game {
  phase: number;
  ending: number;
  paused: boolean;
  settingsOpen: boolean;
  time: number;
  day: number;
  hour: number;
  night: boolean;
  claimed: number;
  claimMask: number;
  seenMask: number;
  wood: number;
  stone: number;
  woodTotal: number;
  stoneTotal: number;
  felled: number;
  bridge: boolean;
  building: number;
  washes: number;
  jumps: number;
  near: number;
  gathering: boolean;
  wading: boolean;
  grounded: boolean;
  speed: number;
}

/** Phase.* and Ending.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const LIT = 1;
export const NIGHT = 2;

/** The summary's buttons answer once its entrance has played (about 1.9 s after the run ends). */
export const SUMMARY_READY = 2.0;

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

/**
 * The unit ground direction a seat's stick walks forward, from a
 * `get:<hero>.SeatLook:<file>` step: its view's heading, or in isometric
 * the fixed angle's (there the look only aims).
 */
export function forward(run: PlaytestRun, file: string): [number, number] {
  const yaw = VIEW === "isometric-2.5d" ? ISO_YAW : value<{ yaw: number }>(run, file).yaw;
  return [-Math.sin(yaw), -Math.cos(yaw)];
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
  return elements(run, file).filter((e) => e.kind === "label" && e.text !== "").map((e) => e.text);
}

/** Seat 0's own key for a key the scripts press: with two on one device the arrows and the right keys are seat 1's. */
const SEAT0: Record<string, string> = { ArrowUp: "KeyW", ArrowRight: "KeyD", ArrowDown: "KeyS", ArrowLeft: "KeyA" };
export const seat0 = (key: string) => (LOCAL ? SEAT0[key] ?? key : key);

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${seat0(key)};${(t + 0.034).toFixed(3)}:key_up:${seat0(key)}`;

/** A key held from t for `seconds`. */
export const hold = (key: string, t: number, seconds: number) => `${t}:key_down:${seat0(key)};${(t + seconds).toFixed(3)}:key_up:${seat0(key)}`;

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** A script from its parts, its steps in time order. */
export function script(parts: readonly string[]): string {
  const steps = parts.flatMap((p) => p.split(";")).filter((s) => s !== "");
  return steps.map((s, i) => ({ s, i, t: parseFloat(s) })).sort((a, b) => a.t - b.t || a.i - b.i).map((x) => x.s).join(";");
}


/**
 * A warp that leaves seat 0's explorer 13 m short of a landmark with W
 * walking it straight in: behind and facing it in third and first person
 * (`warp:<site>`), and in isometric on the line W walks along.
 */
export function approach(site: number, t: number): string {
  if (VIEW !== "isometric-2.5d") return `${t}:warp:${site}`;
  const s = SITES[site]!;
  return `${t}:warp:${(s.x + Math.sin(ISO_YAW) * 13).toFixed(2)}:${(s.z + Math.cos(ISO_YAW) * 13).toFixed(2)}`;
}

/** A touch scaffold's HUD names Gather by its button, not its key. */
export const TOUCH = (creation.input as string) === "touch";
/** How seat 0's prompts name Gather: in English and in 简体中文. */
export const GATHER = TOUCH ? "Gather" : "E";
export const GATHER_ZH = TOUCH ? "采集" : "E";
