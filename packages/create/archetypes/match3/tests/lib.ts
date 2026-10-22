// Shared helpers for the match3 playtests: the board's Match as dump_entity
// writes it, and pointer pixels for cells at the playtests' 1280x720 surface.
import type { PlaytestRun } from "@origingame/engine/playtest";
import { cellCentre } from "../src/layout.ts";

export interface MatchDump {
  phase: number;
  score: number;
  best: number;
  movesLeft: number;
  chain: number;
  maxChain: number;
  cleared: number;
  lastCleared: number;
  rounds: number;
  selected: number;
  pxPerM: number;
  screenX0: number;
  screenY0: number;
}

export const PHASE = { title: 0, idle: 1, swap: 2, swapBack: 3, clear: 4, fall: 5, won: 6, lost: 7 } as const;

export function matchOf(run: PlaytestRun, file: string): MatchDump {
  const dump = run.json<{ components: Record<string, unknown> }>(file);
  const m = dump.components["Match"];
  if (!m) throw new Error(`${file} has no Match`);
  return m as MatchDump;
}

/**
 * The pointer mapping these scripts assume at 1280x720: the view is 12 m
 * tall (60 px per metre) and centred 0.9 m above the board's centre, so the
 * world origin is at (640, 390). Each playtest checks the dumped mapping
 * against it, so a layout change fails loudly instead of tapping elsewhere.
 */
export const MAPPING = { pxPerM: 60, screenX0: 640, screenY0: 390 } as const;

/** The CSS pixel at the centre of a cell (col 0 is the left, row 0 the bottom), offset by (dx, dy) metres. */
export function px(col: number, row: number, dx = 0, dy = 0): string {
  const c = cellCentre(col, row);
  return `${Math.round(MAPPING.screenX0 + (c.x + dx) * MAPPING.pxPerM)},${Math.round(MAPPING.screenY0 - (c.y + dy) * MAPPING.pxPerM)}`;
}

/** A tap on a cell at t seconds. */
export const tapCell = (t: number, col: number, row: number) =>
  `${t}:touch_move:${px(col, row)};${t}:touch_start;${(t + 0.05).toFixed(2)}:touch_end`;

/** A tap anywhere (starts a round from the title or an end card). */
export const tap = (t: number) => `${t}:touch_move:640,700;${t}:touch_start;${(t + 0.05).toFixed(2)}:touch_end`;

/**
 * A board with no line (top row first): swapping (2,0) with (2,1) makes a
 * line of 3s along the bottom row, and when it clears, the tiles that fall
 * make a line of 0s at (1..3, 0): a cascade. Swapping (5,0) with (6,0)
 * makes nothing.
 */
export const CASCADE_BOARD = "2340123/0123401/3401234/1234012/4022340/2031123/3300401";

export function checkMapping(m: MatchDump, t: { equal<T>(a: T, b: T, what: string): void }): void {
  t.equal(m.pxPerM, MAPPING.pxPerM, "the pointer mapping is 60 px per metre at 1280x720");
  t.equal(m.screenX0, MAPPING.screenX0, "the world origin is at x 640");
  t.equal(m.screenY0, MAPPING.screenY0, "the world origin is at y 390");
}
