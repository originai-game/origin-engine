// Shared readers for the runner's playtests: the runner row's components as
// `dump_entity:runner:<file>` writes them.
import type { PlaytestRun } from "@origingame/engine/playtest";

export interface RunDump {
  phase: number;
  phaseTicks: number;
  speed: number;
  distance: number;
  coins: number;
  score: number;
  best: number;
  rounds: number;
  grounded: boolean | number;
}

interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

/** The runner's Run component from a dump_entity file. */
export function runOf(run: PlaytestRun, file: string): RunDump {
  const dump = run.json<EntityDump>(file);
  const r = dump.components["Run"];
  if (!r) throw new Error(`${file} has no Run component: ${Object.keys(dump.components).join(", ")}`);
  return r as unknown as RunDump;
}

/** The runner's position from a dump_entity file. */
export function positionOf(run: PlaytestRun, file: string): { x: number; y: number } {
  const dump = run.json<EntityDump>(file);
  const t = (dump.components["og.Transform"] ?? dump.components["Transform"]) as { position: [number, number, number] } | undefined;
  if (!t) throw new Error(`${file} has no Transform`);
  return { x: t.position[0], y: t.position[1] };
}

export const TITLE = 0;
export const RUNNING = 1;
export const OVER = 2;

/** A press of Space at t seconds (down, then up two ticks later): a tap, which is a hop. */
export const tap = (t: number) => `${t}:key_down:Space;${(t + 0.034).toFixed(3)}:key_up:Space`;
