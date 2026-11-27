// The game-frame benchmark under node (V8) or bun (JavaScriptCore) on the
// napi addon. Built with the game by build.ts; the bundle reads:
//
//   OG_BENCH_ROOT    the project directory (its cooked/ and project.ts), if any
//   OG_BENCH_COMMANDS  commands to run first (`spawn_wave 10000`; `;` between)
//   OG_BENCH_MODE    totals | calls (default totals)
//   OG_BENCH_WARM, OG_BENCH_FRAMES  frame counts (default 120, 600)
//
// and prints the report as one JSON line.

import type { GameToken } from "../../src/runtime.ts";
import { loadNativeHost } from "../../src/node/index.ts";
import { readProject } from "../../src/node/project.ts";
import { commands, type Mode, run } from "./harness.ts";

export function main(game: GameToken): void {
  const env = process.env;
  const root = env.OG_BENCH_ROOT;
  const binding = loadNativeHost();
  const report = run({
    binding,
    game,
    ...(root === undefined ? {} : { root, project: readProject(root)!.startup }),
    commands: commands(env.OG_BENCH_COMMANDS),
    now: () => performance.now(),
    mode: (env.OG_BENCH_MODE ?? "totals") as Mode,
    warm: Number(env.OG_BENCH_WARM ?? 120),
    frames: Number(env.OG_BENCH_FRAMES ?? 600),
    tickMs: 1000 / 60,
  });
  process.stdout.write(`${JSON.stringify({ engine: typeof Bun === "undefined" ? `node ${process.version}` : `bun ${Bun.version}`, ...report })}\n`);
}
