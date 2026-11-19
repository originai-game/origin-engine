// A store package's game on the desktop shell (crates/og-host-c/hermes):
// the game's engine, drawn in the shell's window. The shell evaluates the
// package's bytecode, which runs `main` with the game, then owns the loop:
// each display frame it pumps the window and calls `__og_frame(elapsedMs)`;
// on exit it prints `__og_report()` and calls `__og_close()`.
//
// The shell supplies `__og_env`: OG_ROOT (the package's directory: cooked/
// and project.ogt), OG_PROJECT (project.ogt's text), OG_TITLE, OG_SIZE
// (`1280x720`, CSS pixels) and OG_SHELL_CAPTURE (keep frames for a capture).

import { bindHost, createEngine, type GameToken } from "../runtime.ts";
import type { Dispatcher, HostAbi } from "../generated/abi.ts";
import { hermesBinding } from "./index.ts";

declare const __og_env: Record<string, string | undefined>;
declare const __og_now: () => number;
declare const __og_window_open: (engine: number, width: number, height: number, title: string, keep: boolean) => string;

/** Frames before the report's figures start: pipelines compile and assets land in the first ones. */
const SETTLE = 60;

export function main(game: GameToken): void {
  const env = __og_env;
  const now = __og_now;
  const binding = hermesBinding();
  const raw = binding.abi as unknown as Record<string, unknown>;
  // The game's time is the dispatcher's: its systems, hooks and callbacks.
  let gameMs = 0;
  const abi = {
    ...raw,
    og_host_set_dispatcher: (d: Dispatcher) =>
      binding.abi.og_host_set_dispatcher((...a: Parameters<Dispatcher>) => {
        const s = now();
        try {
          return d(...a);
        } finally {
          gameMs += now() - s;
        }
      }),
  } as unknown as HostAbi<Uint32Array, unknown>;
  bindHost({ ...binding, abi });

  const created = createEngine(game, {
    ...(env.OG_ROOT === undefined ? {} : { root: env.OG_ROOT }),
    ...(env.OG_PROJECT === undefined ? {} : { project: env.OG_PROJECT }),
    deferStart: true,
  });
  if (!created.ok) throw new Error(`the game: ${created.reason}`);
  const engine = created.value;
  const [w, h] = (env.OG_SIZE ?? "1280x720").split("x").map(Number);
  const opened = JSON.parse(__og_window_open(engine.id, w || 1280, h || 720, env.OG_TITLE ?? "Origin", env.OG_SHELL_CAPTURE !== undefined)) as {
    ok: boolean;
    value?: string;
    reason?: string;
  };
  if (!opened.ok) throw new Error(`the window: ${opened.reason}`);
  // The first tick boundary, with the surface in place (GameDesc.start).
  const started = engine.advance(0);
  if (!started.ok) throw new Error(`the game's start: ${started.reason}`);

  const intervals: number[] = [];
  let frames = 0;
  let calls = 0;
  let gameTotal = 0;
  let ticks = 0;
  const g = globalThis as unknown as Record<string, unknown>;
  g.__og_frame = (elapsedMs: number) => {
    const g0 = gameMs;
    const s = now();
    const answer = engine.frame(elapsedMs);
    const call = now() - s;
    if (!answer.ok) throw new Error(`frame ${frames}: ${answer.reason}`);
    if (++frames <= SETTLE) return;
    intervals.push(elapsedMs);
    calls += call;
    gameTotal += gameMs - g0;
    ticks += answer.ticks ?? 0;
  };
  g.__og_close = () => engine.destroy();
  g.__og_report = () => {
    const n = intervals.length;
    const sorted = [...intervals].sort((a, b) => a - b);
    const at = (q: number) => +(sorted[Math.min(n - 1, Math.floor(n * q))] ?? 0).toFixed(3);
    const per = (x: number) => +(n === 0 ? 0 : x / n).toFixed(4);
    return JSON.stringify({
      engine: "hermes",
      renderer: opened.value,
      frames,
      measured: n,
      // The display frame: its interval (vsync paces it), the frame call (the
      // engine's tick and draw, the present's wait included) and the game's code in it.
      intervalMs: per(sorted.reduce((a, b) => a + b, 0)),
      p50: at(0.5),
      p95: at(0.95),
      frameCallMs: per(calls),
      gameMs: per(gameTotal),
      // A slow display frame runs several fixed ticks: the game's time per tick.
      ticksPerFrame: per(ticks),
      gameMsPerTick: +(ticks === 0 ? 0 : gameTotal / ticks).toFixed(4),
    });
  };
}
