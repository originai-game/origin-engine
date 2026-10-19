// Shared readers for Spark Orb's playtests. `get:Game:<file>` writes the
// run's resource, the state the game's `facts` (src/main.ts) report to the
// probe; `dump_ui` the screen's elements and their text; `dump_platform`
// the calls to og.platform.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  paused: boolean;
  held: boolean;
  confirmNova: boolean;
  welcome: boolean;
  sparks: number;
  runEarned: number;
  lifetime: number;
  owned0: number;
  owned1: number;
  owned2: number;
  owned3: number;
  dust: number;
  novas: number;
  lastGain: number;
  rate: number;
  tapValue: number;
  taps: number;
  buys: number;
  refused: number;
  selected: number;
  away: number;
  offline: number;
  saves: number;
  saveConflicts: number;
  loaded: boolean;
  language: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const NOVA = 2;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

export interface UiElement { id: string; parent: string; kind: string; text: string; x: number; y: number; width: number; height: number }

/** The UI's elements from a `dump_ui:<file>` step. */
export function ui(run: PlaytestRun, file: string): { surface: { width: number; height: number }; elements: UiElement[] } {
  return run.json(file);
}

/** The text of the first element with this id whose parent is `parent` (any parent when omitted). */
export function text(run: PlaytestRun, file: string, id: string, parent?: string): string | undefined {
  return ui(run, file).elements.find((e) => e.id === id && (parent === undefined || e.parent === parent))?.text;
}

export interface PlatformDump {
  host: string;
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; state: string; result?: { ok: boolean; data?: Record<string, unknown>; error?: { code: string } } }>;
}

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t}:key_down:${key};${(t + 0.034).toFixed(3)}:key_up:${key}`;

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** A tap at a CSS pixel. */
export const tapAt = (x: number, y: number, t: number) => `${t}:touch_move:${x},${y};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;

/** The orb's centre on the 1280×720 desktop surface (src/ui.ts: 10 m tall, the orb at y 0.35 m). */
export const ORB = { x: 640, y: 360 - 0.35 * 72 } as const;
