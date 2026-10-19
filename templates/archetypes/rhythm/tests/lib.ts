// Shared readers and the chart for Three-Lane Beat's playtests.
// `get:Game:<file>` writes the play's resource, the state the game's
// `facts` (src/main.ts) report to the probe; `dump_ui` the screen;
// `dump_entity` a row; `dump_platform` the calls to og.platform. The chart
// is read from assets/charts/neon-pulse.ogt (data), so a script can press
// each lane at its notes' times.
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface Game {
  phase: number;
  paused: boolean;
  held: boolean;
  songStart: number;
  pausedAt: number;
  countTo: number;
  score: number;
  combo: number;
  maxCombo: number;
  perfect: number;
  great: number;
  good: number;
  miss: number;
  hits: number;
  judged: number;
  notes: number;
  taps: number;
  groove: number;
  failed: boolean;
  cleared: boolean;
  lastJudgement: number;
  lastOffsetMs: number;
  calTaps: number;
  calSum: number;
  best: number;
  newBest: boolean;
  language: number;
}

/** Phase.* of src/components.ts. */
export const TITLE = 0;
export const PLAY = 1;
export const RESULTS = 2;
export const CALIBRATE = 3;

/** The Game resource from a `get:Game:<file>` step. */
export function game(run: PlaytestRun, file: string): Game {
  return run.json<{ value: Game }>(file).value;
}

export interface UiElement { id: string; parent: string; kind: string; text: string; x: number; y: number; width: number; height: number }

export function ui(run: PlaytestRun, file: string): { surface: { width: number; height: number }; elements: UiElement[] } {
  return run.json(file);
}

/** The text of the first element with this id (under `parent` when given). */
export function text(run: PlaytestRun, file: string, id: string, parent?: string): string | undefined {
  return ui(run, file).elements.find((e) => e.id === id && (parent === undefined || e.parent === parent))?.text;
}

export interface PlatformDump {
  calls: Array<{ method: string; args: Record<string, unknown>; tick: number; result?: { ok: boolean; data?: Record<string, unknown> } }>;
}

/** The chart's notes in time order: lane and beat. */
export interface ChartNote { lane: number; beat: number }

/** The project's assets/charts/neon-pulse.ogt: from the bundled playtest's directory up (og test runs it from og-out/test/<name>/, og polish from elsewhere), else from the working directory. */
function chartPath(): string {
  const rel = join("assets", "charts", "neon-pulse.ogt");
  for (let dir = dirname(fileURLToPath(import.meta.url)), n = 0; n < 8; dir = dirname(dir), n++) {
    if (existsSync(join(dir, rel)) && existsSync(join(dir, "project.ogt"))) return join(dir, rel);
  }
  return resolve(process.cwd(), rel);
}

/** The chart's notes, read as data. */
export function chart(): ChartNote[] {
  const textOf = readFileSync(chartPath(), "utf8");
  const notes: ChartNote[] = [];
  const re = /Note \{\s*index (\d+)\s*lane (\d+)\s*beat ([\d.]+)\s*\}/g;
  for (let m = re.exec(textOf); m !== null; m = re.exec(textOf)) notes.push({ lane: Number(m[2]), beat: Number(m[3]) });
  return notes;
}

/** The song's beat (120 BPM). */
export const BEAT = 0.5;

/**
 * Where bar 1 starts for a `play` (or a Play tap's start) at time t: the
 * next whole tick a quarter second on, then the four beats of the count-in
 * (src/ui.ts `begin`), less the bars skipped by `play:<bar>`.
 */
export function songStart(t: number, fromBar = 1): number {
  return Math.ceil((Math.round(t * 60) / 60 + 0.25) * 60) / 60 + 4 * BEAT - (fromBar - 1) * 4 * BEAT;
}

/** The lanes' keys. */
export const KEYS = ["KeyA", "KeyS", "KeyD"] as const;

/** A key pressed at t (down, then up two ticks later). */
export const press = (key: string, t: number) => `${t.toFixed(4)}:key_down:${key};${(t + 0.034).toFixed(4)}:key_up:${key}`;

/** Presses for the notes in [fromBeat, toBeat), each `late` seconds after its time (a chord's lanes together). */
export function play(notes: readonly ChartNote[], start: number, fromBeat: number, toBeat: number, late = 0): string[] {
  return notes.filter((n) => n.beat >= fromBeat && n.beat < toBeat).map((n) => press(KEYS[n.lane]!, start + n.beat * BEAT + late));
}

/** Commands in time order (the probe wants ascending times). */
export function sorted(cmds: readonly string[]): string {
  return cmds
    .flatMap((c) => c.split(";"))
    .map((c) => ({ t: Number(c.slice(0, c.indexOf(":"))), c }))
    .sort((a, b) => a.t - b.t)
    .map((x) => x.c)
    .join(";");
}

/** A tap on a UI element by its id path (pressed, released a tick later). */
export const tap = (id: string, t: number) => `${t}:touch_move_to:${id};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.05).toFixed(3)}:touch_end`;
