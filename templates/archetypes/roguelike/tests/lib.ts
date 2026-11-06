// Shared readers for Lamp Dive's playtests. `get:Game:<file>` writes the
// dive's resource, the state the game's `facts` (src/main.ts) report to the
// probe; `get:Foes:<file>` where each foe stands; `dump_entity:<name>:<file>`
// one row; `dump_platform` the calls to og.platform.
//
// The scripts drive seat 0, the lamp-bearer, with seat 0's own keys
// (src/creation.ts, src/seats.ts): alone it takes every key, the arrows
// among them; two on one device (local-multiplayer) it takes WASD, Space,
// Q and pad 1, and seat 1's delver must have its turn too before the
// world's comes, so each turn's key is paired with seat 1's Wait (Enter).
// Online without a room is one seat on this device, as alone.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

/** Two delvers on this device: seat 1 waits each turn, and the run ends only when both are down. */
export const TWO = (creation.players as string) === "local-multiplayer";

export interface Game {
  phase: number;
  paused: boolean;
  settingsOpen: boolean;
  dives: number;
  relics: number;
  recordConflicts: number;
  seed: number;
  floor: number;
  deepest: number;
  hp: number;
  hpMax: number;
  attack: number;
  armour: number;
  lamp: number;
  potions: number;
  gold: number;
  kills: number;
  hits: number;
  hurts: number;
  finds: number;
  turns: number;
  heroX: number;
  heroY: number;
  hp2: number;
  hpMax2: number;
  hero2X: number;
  hero2Y: number;
  stairsX: number;
  stairsY: number;
  won: boolean;
  killedBy: number;
  lit: number;
  seen: number;
  language: number;
}

/** Phase.* and FoeKind.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const RAT = 0;
export const SKELETON = 1;
export const GUARDIAN = 2;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

/** Foe slot i's cell from a `get:Foes:<file>` step ([-1, -1] when it is down). */
export function foe(run: PlaytestRun, file: string, i: number): [number, number] {
  const v = run.json<{ value: Record<string, number> }>(file).value;
  return [v[`x${i}`]!, v[`y${i}`]!];
}

/** One component of a dumped entity. */
export function component<T>(run: PlaytestRun, file: string, name: string): T {
  const c = run.json<{ components: Record<string, unknown> }>(file).components[name];
  if (!c) throw new Error(`${file} has no ${name}`);
  return c as T;
}

export interface PlatformDump {
  host: string;
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data?: Record<string, unknown>; error?: { code: string } } }>;
}

/** The UI's labels from a `dump_ui:<file>` step. */
export function texts(run: PlaytestRun, file: string): string[] {
  return run.json<{ elements: Array<{ text: string; kind: string }> }>(file).elements.filter((e) => e.kind === "label" && e.text !== "").map((e) => e.text);
}

/** A script from its parts, its steps in time order (a seat 1 step paired with a seat 0 press lands between the press's down and up). */
export function script(parts: readonly string[]): string {
  const steps = parts.flatMap((p) => p.split(";")).filter((s) => s !== "");
  return steps.map((s, i) => ({ s, i, t: parseFloat(s) })).sort((a, b) => a.t - b.t || a.i - b.i).map((x) => x.s).join(";");
}

/** Seat 0's own key for a key the single-player scripts press: with two on one device the arrows are seat 1's, so seat 0 walks with WASD. */
const SEAT0: Record<string, string> = { ArrowUp: "KeyW", ArrowRight: "KeyD", ArrowDown: "KeyS", ArrowLeft: "KeyA" };
export const seat0 = (key: string) => (TWO ? SEAT0[key] ?? key : key);

/** A key pressed exactly at t (down, then up two ticks later). */
export const key = (k: string, t: number) => `${t}:key_down:${k};${(t + 0.034).toFixed(3)}:key_up:${k}`;

/** Seat 0's key pressed at t that takes no turn (a bump, a refusal, Escape, a key while paused). */
export const press = (k: string, t: number) => key(seat0(k), t);

/** Seat 1's Wait (Enter) at t, with two on one device; nothing alone. */
export const partner = (t: number) => (TWO ? `;${key("Enter", t)}` : "");

/** Seat 0's key at t taking its turn; with two on one device seat 1 waits on the same tick, so the world's turn comes. */
export const turn = (k: string, t: number) => press(k, t) + partner(t);

/** Turns one after another, a turn (0.25 s) apart from t. */
export const presses = (keys: string[], t: number) => keys.map((k, i) => turn(k, +(t + i * 0.25).toFixed(3))).join(";");

/** Seat 1's Wait every 0.1 s from t to end (while seat 0 holds a direction that repeats), with two on one device. */
export const partnerHold = (t: number, end: number) => (TWO ? Array.from({ length: Math.floor((end - t) / 0.1) + 1 }, (_, i) => partner(+(t + i * 0.1).toFixed(2)).slice(1)).join(";") : "");

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;
