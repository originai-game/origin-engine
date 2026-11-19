// The game-frame benchmark under Hermes, on the C ABI host through
// crates/og-host-c/hermes's shell (og-hermes), which supplies
// `__og_host` (every binding symbol), `__og_now`, `__og_print` and
// `__og_env` (as node.ts reads its environment, with OG_BENCH_PROJECT=@<file>
// for project.ogt's text). Built with the game by build.ts.

import type { HostAbi } from "../../src/generated/abi.ts";
import { ARRAYS, type ColumnArray, type GameToken, type HostBinding } from "../../src/runtime.ts";
import { commands, type Mode, run } from "./harness.ts";

declare const __og_host: HostAbi<Uint32Array, unknown>;
declare const __og_now: () => number;
declare const __og_print: (line: string) => void;
declare const __og_env: Record<string, string | undefined>;

const g = globalThis as unknown as { performance?: { now(): number } };
g.performance ??= { now: __og_now };

const arrays = new WeakMap<ArrayBuffer, ColumnArray>();

export function main(game: GameToken): void {
  const env = __og_env;
  const abi = __og_host;
  let shared: ArrayBuffer | null = null;
  const binding: HostBinding = {
    abi,
    name: "hermes",
    shared: () => ({ buffer: (shared ??= abi.og_host_shared() as ArrayBuffer), offset: 0 }),
    columns(raw, kinds) {
      if (!Array.isArray(raw) || raw.length === 0) return null;
      return (raw as ArrayBuffer[]).map((buffer, i) => {
        const kind = kinds[i]!.array;
        const known = arrays.get(buffer);
        if (known) return known;
        const made = new ARRAYS[kind](buffer);
        arrays.set(buffer, made);
        return made;
      });
    },
  };
  try {
    const report = run({
      binding,
      game,
      ...(env.OG_BENCH_ROOT === undefined ? {} : { root: env.OG_BENCH_ROOT }),
      ...(env.OG_BENCH_PROJECT === undefined ? {} : { project: env.OG_BENCH_PROJECT }),
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
