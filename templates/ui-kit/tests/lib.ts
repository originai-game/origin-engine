// Command-script helpers and readers for the kit's playtests.
import { playtest as enginePlaytest, type PlaytestDesc, type PlaytestRun } from "@origingame/engine/playtest";
import { SCREENS, type Screen } from "../src/main.ts";

/**
 * The kit's playtest: every run draws its UI over the pack's plain ground
 * (`demo:ground:pack`), never over the key art, painted scenes or 3D
 * circuit the demo shows behind its menus and HUDs. A UI frame is judged
 * over a plain, solid-colour ground, and a scene by its own tests
 * (docs/ui.md §15.4); the scenes' textures also differ by GPU format
 * (BC7 against ASTC), which kept per-backend copies of HUD frames.
 */
export function playtest(desc: PlaytestDesc): PlaytestDesc {
  const ground = "demo:ground:pack";
  return enginePlaytest({ ...desc, setup: desc.setup ? `${ground};${desc.setup}` : ground });
}

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
  /** og.settings read back on a launch (src/main.ts followSettings). */
  loads: number;
  /** LANGUAGES index (0 en, 1 zh-CN, …). */
  language: number;
  muted: boolean;
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
  pick0: number;
  pick1: number;
  pick2: number;
  pick3: number;
  focus0: number;
  focus1: number;
  focus2: number;
  focus3: number;
  rules: number;
  a11yNodes: number;
  a11yFocus: number;
  zoom: number;
  quests: number;
}

export const demo = (run: PlaytestRun, file: string): DemoState =>
  run.json<{ components: { Demo: DemoState } }>(file).components.Demo;

export interface UiElement {
  id: string;
  /** The parent's id (not its path). */
  parent: string;
  /** The element's path from its root, ids joined by `/`. */
  path: string;
  kind: string;
  text: string;
  role: string;
  name: string;
  value: string;
  states: string[];
  x: number;
  y: number;
  width: number;
  height: number;
  /** A scroll container cuts it: its box is the part in view. */
  clipped?: boolean;
}

export const ui = (run: PlaytestRun, file: string): UiElement[] => run.json<{ elements: UiElement[] }>(file).elements;

/** The screens' indices in the demo (src/main.ts SCREENS), by name: `demo(...).screen` is one of these. */
export const SCREEN: Readonly<Record<Screen, number>> = Object.fromEntries(SCREENS.map((s, i) => [s, i])) as Record<Screen, number>;
