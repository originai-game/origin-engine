// Shared readers for Ashen Crown's playtests. `get:Game:<file>` writes the
// run's resource; `dump_entity:hero:<file>` (seat 0) and `hero-1` (seat 1)
// a hero's row, with its hand, piles and focus; `dump_entity:foe-<i>` a
// foe's; `dump_platform` the calls to og.platform.
//
// The scripts drive seat 0, the Warden, with seat 0's own keys (src/
// creation.ts, src/seats.ts): alone it takes every key; two on one device
// (local-multiplayer) it takes WASD, Space, E and R, and seat 1 (the
// Hunter) the arrows, Enter, / and Backspace. With two, the foes act only
// once both heroes ended their turn, so `endTurn` ends both. Online
// without a room is one seat on this device, as alone.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** Two heroes on this device. */
export const TWO = (creation.players as string) === "local-multiplayer";
/** Made for a phone in portrait: the frames are the phone's. */
export const PHONE = (creation.platform as string) === "phone-portrait";
/** The war table seen from above, or the battle side-on. */
export const VIEW = creation.view as string;

export interface Game {
  phase: number;
  paused: boolean;
  party: number;
  step: number;
  path: number;
  turn: number;
  side: number;
  cleared: boolean;
  decided: number;
  won: boolean;
  fights: number;
  played: number;
  dealt: number;
  taken: number;
  turns: number;
  killedBy: number;
  runs: number;
  wins: number;
  rewards0: number[];
  rewards1: number[];
  tempers0: number[];
}

export interface Hero {
  seat: number;
  active: boolean;
  hp: number;
  hpMax: number;
  block: number;
  energy: number;
  strength: number;
  deck: number[];
  draw: number[];
  hand: number[];
  discard: number[];
  ended: boolean;
  focusCard: number;
  focusFoe: number;
}

export interface Foe {
  active: boolean;
  kind: number;
  hp: number;
  hpMax: number;
  block: number;
  poison: number;
  vulnerable: number;
  move: number;
  dying: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const MAP = 1;
export const BATTLE = 2;
export const REWARD = 3;
export const REST = 4;
export const OVER = 5;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

/** One component of a dumped entity. */
export function component<T>(run: PlaytestRun, file: string, name: string): T {
  const c = run.json<{ components: Record<string, unknown> }>(file).components[name];
  if (!c) throw new Error(`${file} has no ${name}`);
  return c as T;
}

export const hero = (run: PlaytestRun, file: string) => component<Hero>(run, file, "Hero");
export const foe = (run: PlaytestRun, file: string) => component<Foe>(run, file, "Foe");

export interface PlatformDump {
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number }>;
}

/** The UI's labels from a `dump_ui:<file>` step. */
export function texts(run: PlaytestRun, file: string): string[] {
  return run.json<{ elements: Array<{ text: string; kind: string }> }>(file).elements.filter((e) => e.text !== "").map((e) => e.text);
}

/** A script from its parts, its steps in time order. */
export function script(parts: readonly string[]): string {
  const steps = parts.flatMap((p) => p.split(";")).filter((s) => s !== "");
  return steps.map((s, i) => ({ s, i, t: parseFloat(s) })).sort((a, b) => a.t - b.t || a.i - b.i).map((x) => x.s).join(";");
}

/** A key pressed exactly at t (down, then up two ticks later). */
export const key = (k: string, t: number) => `${t}:key_down:${k};${(t + 0.034).toFixed(3)}:key_up:${k}`;

/** Seat 0's keys for what the single-player scripts press. */
const SEAT0: Record<string, string> = { ArrowUp: "KeyW", ArrowRight: "KeyD", ArrowDown: "KeyS", ArrowLeft: "KeyA", Enter: "Space" };
export const press = (k: string, t: number) => key(TWO ? SEAT0[k] ?? k : k, t);

/** Ends the turn at t: seat 0's R, and with two on one device seat 1's Backspace too. */
export const endTurn = (t: number) => key("KeyR", t) + (TWO ? `;${key("Backspace", t)}` : "");

/** A tap on a UI element by its id path (pressed, released a tick later), then the finger lifted off the surface. */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** Begin a run from the title and take the first fight, by taps. */
export const beginRun = (t: number) => `${tap("Play", t)};${tap("Path#0", t + 0.6)}`;
