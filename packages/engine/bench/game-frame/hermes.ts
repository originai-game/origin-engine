// The game-frame benchmark under Hermes, on the C ABI host through
// crates/og-host-c/hermes's shell (og-hermes), which supplies
// `__og_host` (every binding symbol), `__og_now`, `__og_print` and
// `__og_env` (as node.ts reads its environment, with OG_BENCH_PROJECT=@<file>
// for checked ProjectStartup JSON). Built with the game by build.ts.

import type { GameToken } from "../../src/runtime.ts";
import { hermesBinding } from "../../src/hermes/index.ts";
import { commands, type Mode, run } from "./harness.ts";

declare const __og_now: () => number;
declare const __og_print: (line: string) => void;
declare const __og_env: Record<string, string | undefined>;

export function main(game: GameToken): void {
  const env = __og_env;
  const binding = hermesBinding();
  try {
    const report = run({
      binding,
      game,
      ...(env.OG_BENCH_ROOT === undefined ? {} : { root: env.OG_BENCH_ROOT }),
      ...(env.OG_BENCH_PROJECT === undefined ? {} : { project: JSON.parse(env.OG_BENCH_PROJECT) }),
      commands: commands(env.OG_BENCH_COMMANDS),
      now: __og_now,
      mode: (env.OG_BENCH_MODE ?? "totals") as Mode,
      warm: Number(env.OG_BENCH_WARM ?? 120),
      frames: Number(env.OG_BENCH_FRAMES ?? 600),
      tickMs: 1000 / 60,
    });
    __og_print(JSON.stringify({ engine: "hermes", ...report }));
  } catch (e) {
    __og_print(`error: ${e instanceof Error ? `${e.message}\n${e.stack}` : String(e)}`);
  }
}
