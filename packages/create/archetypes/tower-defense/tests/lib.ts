// Shared readers for Path Guard's playtests. `get:Game:<file>` writes the
// round's resource, the state the game's `facts` (src/main.ts) report to the
// probe; `dump_entity:<name>:<file>` one row; `dump_platform` the calls to
// og.platform.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  paused: boolean;
  settingsOpen: boolean;
  won: boolean;
  rounds: number;
  lives: number;
  gold: number;
  wave: number;
  inWave: boolean;
  walking: number;
  kills: number;
  leaks: number;
  placed: number;
  refused: number;
  refusal: number;
  shots: number;
  selected: number;
  cursorX: number;
  cursorY: number;
  score: number;
  best: number;
  newBest: boolean;
  bestRevision: number;
}

/** Phase.* and Refusal.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const OVER = 2;
export const POOR = 1;
export const BLOCKED = 2;
export const TAKEN = 3;

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

export interface PlatformDump {
  host: string;
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data: Record<string, unknown> } }>;
}

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** Play on the title: the round starts on the release. */
export const play = (t: number) => tap("play", t);

/**
 * A cell's centre in CSS pixels on the 1280×720 desktop surface: src/ui.ts
 * frames the 9 × 13 field in the room the HUD leaves (76 px above, 24
 * below, 200 each side), 46.27 px a cell.
 */
export function cellPx(x: number, y: number): string {
  const scale = Math.min(880 / 9.4, 620 / 13.4);
  const camY = 6.5 + (386 - 360) / scale;
  const sx = 640 + (x + 0.5 - 4.5) * scale;
  const sy = 360 - (12.5 - y - camY) * scale;
  return `${sx.toFixed(1)},${sy.toFixed(1)}`;
}

/** A tap on a lawn cell (desktop). */
export const tapCell = (x: number, y: number, t: number) => `${t}:touch_move:${cellPx(x, y)};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;
