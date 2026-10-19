// Shared readers for Lamp Dive's playtests. `get:Game:<file>` writes the
// dive's resource, the state the game's `facts` (src/main.ts) report to the
// probe; `get:Foes:<file>` where each foe stands; `dump_entity:<name>:<file>`
// one row; `dump_platform` the calls to og.platform.
import type { PlaytestRun } from "@origingame/engine/playtest";

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

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** Keys pressed one after another, a turn (0.25 s) apart from t. */
export const presses = (keys: string[], t: number) => keys.map((k, i) => press(k, +(t + i * 0.25).toFixed(3))).join(";");

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;
