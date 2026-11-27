// A store package's game on the desktop shell (crates/og-host-c/hermes):
// the game's engine, drawn in the shell's window. The shell evaluates the
// package's bytecode, which runs `main` with the game, then owns the loop:
// each display frame it pumps the window and calls `__og_frame(nowMs)` with
// its clock, which hands the frame to the engine (`engine.present`: the frame
// cap, the display's interval, the frame-time record) and answers when the
// next drawn frame is due, which the shell waits until; on exit it prints
// `__og_report()` (the engine's performance record) and calls `__og_close()`.
//
// The shell supplies `__og_env`: OG_ROOT (the package's directory: cooked/
// and project.ogt), OG_PROJECT (project.ogt's text), OG_TITLE, OG_SIZE
// (`1280x720`, CSS pixels), OG_LOCALE (the OS's locale: og.i18n's), OG_SHELL_CAPTURE (keep frames for a capture),
// OG_BIN (the shell's directory) and a provider's OG_PROVIDER_LIB. og.platform
// is platform.ts's.

import { bindHost, createEngine, type GameToken } from "../runtime.ts";
import { hermesBinding } from "./index.ts";
import type { ShellProviderFactory } from "../provider.ts";
import { shellPlatform } from "./platform.ts";

declare const __og_env: Record<string, string | undefined>;
declare const __og_audio: () => string;
declare const __og_data: (game: string, document?: string) => string | boolean | null;
declare const __og_provider: (path: string) => string;
declare const __og_provider_call: (method: string, json: string) => string | null;
declare const __og_print: (text: string) => void;
declare const __og_window_open: (engine: number, width: number, height: number, title: string, keep: boolean) => string;
declare const __og_window_fullscreen: (on: boolean) => void;

/** Drawn frames before the report's record starts: pipelines compile and assets land in the first ones. */
const SETTLE = 60;

/** Runs `game`; `provider` is the store's, when the package bundled one (package.ts --provider). */
export function main(game: GameToken, provider?: ShellProviderFactory): void {
  const env = __og_env;
  const binding = hermesBinding();
  bindHost(binding);

  const created = createEngine(game, {
    ...(env.OG_ROOT === undefined ? {} : { root: env.OG_ROOT }),
    ...(env.OG_PROJECT === undefined ? {} : { project: JSON.parse(env.OG_PROJECT) }),
    ...(env.OG_LOCALE ? { locale: env.OG_LOCALE } : {}),
    deferStart: true,
  });
  if (!created.ok) throw new Error(`the game: ${created.reason}`);
  const engine = created.value;
  engine.attachPlatform(
    shellPlatform(
      env,
      {
        load: (name) => __og_data(name) as string | null,
        save: (name, document) => __og_data(name, document) === true,
        open: (path) => JSON.parse(__og_provider(path)) as { ok: boolean; reason?: string },
        call: __og_provider_call,
        warn: __og_print,
      },
      provider,
    ),
  );
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

  let drawn = 0;
  let fullscreen = false;
  const g = globalThis as unknown as Record<string, unknown>;
  g.__og_frame = (nowMs: number): number => {
    const answer = engine.present(nowMs);
    if (!answer.ok) throw new Error(`frame ${drawn}: ${answer.reason}`);
    if (answer.drawn) {
      // The player's settings ask for fullscreen: the window carries it out and the next pump reports it.
      if (answer.fullscreen !== undefined && answer.fullscreen !== fullscreen) __og_window_fullscreen((fullscreen = answer.fullscreen));
      if (++drawn === SETTLE) binding.abi.og_engine_perf(engine.id, true);
    }
    return answer.nextMs;
  };
  g.__og_close = () => engine.destroy();
  g.__og_report = () => {
    const report = drawn >= SETTLE ? (JSON.parse(binding.abi.og_engine_perf_report(engine.id, "{}")) as { json: unknown }).json : null;
    return JSON.stringify({
      engine: "hermes",
      renderer: opened.value,
      frames: drawn,
      // The engine's performance record after the first SETTLE drawn frames (guide perf): `frame interval` is the
      // display frame's (vsync paces it), `frame work` the engine's tick and draw, the game's code by stage.
      perf: report,
      // What the output device played (null on a machine without one).
      audio: JSON.parse(__og_audio()) as unknown,
    });
  };
}
