// Command-script helpers and readers for the kit's playtests.
import type { PlaytestRun } from "@origingame/engine/playtest";

/** Touch an element by id or path: move there, press, release. */
export const tap = (t: number, target: string): string =>
  [`${t.toFixed(2)}:touch_move_to:${target}`, `${(t + 0.05).toFixed(2)}:touch_start`, `${(t + 0.1).toFixed(2)}:touch_end`].join(";");

/** Touch the element showing a text (`=` for an exact match). */
export const tapText = (t: number, text: string): string =>
  [`${t.toFixed(2)}:move_to_text:${text}`, `${(t + 0.05).toFixed(2)}:touch_start`, `${(t + 0.1).toFixed(2)}:touch_end`].join(";");

/** Press and release a key (`KeyboardEvent.code`). */
export const key = (t: number, code: string): string => [`${t.toFixed(2)}:key_down:${code}`, `${(t + 0.05).toFixed(2)}:key_up:${code}`].join(";");

export interface DemoState {
  screen: number;
  balance: number;
  owned: number;
  bought: number;
  refused: number;
  saves: number;
  masterVolume: number;
  inv0: number;
  inv1: number;
  moves: number;
  line: number;
  step: number;
  taps: number;
  gen0: number;
  coins: number;
  hp: number;
  stickX: number;
  difficulty: number;
  used: number;
}

export const demo = (run: PlaytestRun, file: string): DemoState =>
  run.json<{ components: { Demo: DemoState } }>(file).components.Demo;

export interface UiElement {
  id: string;
  parent: string;
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export const ui = (run: PlaytestRun, file: string): UiElement[] => run.json<{ elements: UiElement[] }>(file).elements;

/** The screens' indices in the demo (src/main.ts SCREENS). */
export const SCREEN = {
  gallery: 0, widgets: 1, mainMenu: 2, pause: 3, settings: 4, platformer: 5, shooter: 6, puzzle: 7, idle: 8, racing: 9,
  cutscene: 10, shop: 11, inventory: 12, levels: 13, results: 14, leaderboard: 15, tutorial: 16, loading: 17, confirm: 18, alert: 19, toasts: 20,
} as const;
