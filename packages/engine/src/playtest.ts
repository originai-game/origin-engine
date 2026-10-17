// Playtest scripts (`tests/*.og.ts`): a command script run headless through
// `og run --commands`, then checks over what the run wrote. `og test` runs
// every one in its own process and fails a playtest that made no check.
//
//   import { playtest } from "@origingame/engine/playtest";
//   export default playtest({
//     name: "the trickster spins after S",
//     commands: "0.5:type:s;2.0:screenshot:spin.png;2.2:dump_entity:trickster:t.json;2.5:quit",
//     check(run, t) {
//       t.ok(run.quit, "the script reached quit");
//       t.ok(run.json<{ components: Record<string, { speed: number }> }>("t.json").components.Spin!.speed > 0, "Spin started");
//     },
//   });

/** What one headless run produced. */
export interface PlaytestRun {
  /** The tick the run ended on. */
  readonly tick: number;
  /** The world hash at the end, hex. */
  readonly hash: string;
  readonly entities: number;
  /** Whether a `quit` step ran. */
  readonly quit: boolean;
  /** The renderer tier (`metal`, `vulkan-cpu`, `none`). */
  readonly tier: string;
  /** Every file the commands wrote, by its `Output::file_name()` (`smoke.png`, `dump_world-000132.json`). */
  readonly files: readonly string[];
  /** Warnings the command channel and the probe logged, each naming the fix. */
  readonly warnings: readonly string[];
  /** A written JSON output, parsed. Throws when the run wrote no such file. */
  json<T = unknown>(file: string): T;
  /** A written PNG's size. Throws when the run wrote no such file. */
  png(file: string): { width: number; height: number; bytes: number };
  /** A written text output. */
  text(file: string): string;
}

/** The checks a playtest makes; a playtest that made none fails. */
export interface PlaytestChecks {
  ok(condition: unknown, what: string): void;
  equal<T>(actual: T, expected: T, what: string): void;
  near(actual: number, expected: number, tolerance: number, what: string): void;
}

export interface PlaytestDesc {
  /** One line on what the playtest proves. */
  readonly name: string;
  /** `t:action[:args];...`, as `og run --commands`. End it with `quit`. */
  readonly commands: string;
  /** The replay seed; omitted is GameDesc.seed. */
  readonly seed?: number;
  /** The render target, `WIDTHxHEIGHT` (default 1280x720). */
  readonly size?: string;
  readonly check: (run: PlaytestRun, t: PlaytestChecks) => void | Promise<void>;
}

/** Declare a playtest: the default export of a `tests/*.og.ts` file. */
export function playtest(desc: PlaytestDesc): PlaytestDesc {
  if (!desc.name || !desc.commands || typeof desc.check !== "function") {
    throw new Error("playtest({ name, commands, check }) needs all three");
  }
  return Object.freeze({ ...desc });
}
