// Playtest scripts (`tests/*.og.ts`): a command script run headless through
// `og run --commands`, then checks over what the run wrote. `og test` runs
// every one in its own process and fails a playtest that made no check.
//
// Without a renderer (the platform's engine and probe sandboxes carry none;
// `OG_RENDERER=none` makes any machine so) the run is the same run, but a
// `screenshot` writes no PNG: the checks that read a frame go inside
// `t.frame(files, () => ...)`, which runs them with a renderer and otherwise
// reports each file "no renderer, frame unchecked" and passes. Frames are
// verified where there is a renderer: the author's machine, the
// browser-check sandbox, an executor with a GPU.
//
//   import { playtest } from "@origingame/engine/playtest";
//   export default playtest({
//     name: "the trickster spins after S",
//     commands: "0.5:type:s;2.0:screenshot:spin.png;2.2:dump_entity:trickster:t.json;2.5:quit",
//     check(run, t) {
//       t.ok(run.quit, "the script reached quit");
//       t.ok(run.json<{ components: Record<string, { speed: number }> }>("t.json").components.Spin!.speed > 0, "Spin started");
//       t.frame("spin.png", () => t.equal(run.png("spin.png").width, 1280, "the frame is 1280 wide"));
//     },
//   });

/** What one headless run produced. */
export interface PlaytestRun {
  /** The tick the run ended on. */
  readonly tick: number;
  /** The world hash at the end, hex. */
  readonly hash: string;
  /** The end hash of every run (`PlaytestDesc.runs`), the first being `hash`: equal hashes are a replay that matched. */
  readonly hashes: readonly string[];
  readonly entities: number;
  /** Whether a `quit` step ran. */
  readonly quit: boolean;
  /** The renderer tier (`metal`, `vulkan`, `webgpu`, ..., `none` without a renderer). */
  readonly tier: string;
  /** Whether the run had a renderer: without one (`tier` `none`) a `screenshot` writes no PNG. */
  readonly rendered: boolean;
  /** The screenshots the run took without a renderer, by the file each would have written: frames unchecked. */
  readonly unchecked: readonly string[];
  /** Every file the commands wrote, by its `Output::file_name()` (`smoke.png`, `dump_world-000132.json`). */
  readonly files: readonly string[];
  /** Warnings the command channel and the probe logged, each naming the fix. */
  readonly warnings: readonly string[];
  /** A written JSON output, parsed. Throws when the run wrote no such file. */
  json<T = unknown>(file: string): T;
  /** A written PNG's size. Throws when the run wrote no such file (and, for a screenshot taken without a renderer, a FrameUnchecked: read frames inside `t.frame`). */
  png(file: string): { width: number; height: number; bytes: number };
  /** A written text output. */
  text(file: string): string;
  /** A written WAV (`listen`): its rate, frames, peak and RMS (16-bit PCM read back, both channels). */
  wav(file: string): { rate: number; frames: number; peak: number; rms: number };
  /** The runs of PlaytestDesc.then, in order, each with its own files (empty without `then`). */
  readonly then: readonly PlaytestRun[];
}

/** The checks a playtest makes; a playtest that made none fails. */
export interface PlaytestChecks {
  ok(condition: unknown, what: string): void;
  equal<T>(actual: T, expected: T, what: string): void;
  near(actual: number, expected: number, tolerance: number, what: string): void;
  /**
   * The checks that read frames (`run.png`, a decoded screenshot, a comparison with
   * references/): `check` runs when the playtest had a renderer. Without one it does not run,
   * each of `files` is reported "no renderer, frame unchecked", and the playtest passes on its
   * other checks. Every check outside `t.frame` runs either way.
   */
  frame(files: string | readonly string[], check: () => void): void;
}

/**
 * What reading a screenshot the run took without a renderer throws. A check function stopped by
 * one passes with its frame unchecked (the checks after it did not run, and the report says so):
 * put frame checks inside `t.frame` so every other check runs.
 */
export class FrameUnchecked extends Error {
  readonly file: string;
  constructor(file: string) {
    super(`${file}: no renderer, frame unchecked`);
    this.name = "FrameUnchecked";
    this.file = file;
  }
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
  /** Run the script this many times in fresh processes (default 1) and report every end hash in `run.hashes`: a determinism check. The files are the first run's. */
  readonly runs?: number;
  /**
   * The host's clock at tick 0, UTC milliseconds (`og run --clock`): og.platform.now and the
   * answers' times start there and move with the ticks, so a playtest is reproducible.
   * Omitted is 1767225600000 (2026-01-01T00:00:00Z).
   */
  readonly clock?: number;
  /**
   * `keep`: every process of this playtest (the script, its `runs` and its `then` steps)
   * answers og.platform from one local store, `<out>/.store/platform.json` (a dot name, which no step's output may take), empty when the playtest
   * starts, so a later process sees what an earlier one saved (a relaunch). Omitted: each
   * process has its own store in memory.
   */
  readonly store?: "keep";
  /** More processes after the script, in order, each a fresh `og run` (a relaunch): their runs are `run.then`. */
  readonly then?: readonly PlaytestStep[];
  readonly check: (run: PlaytestRun, t: PlaytestChecks) => void | Promise<void>;
}

/** A later process of a playtest (`PlaytestDesc.then`). */
export interface PlaytestStep {
  /** Its own script, as PlaytestDesc.commands. */
  readonly commands: string;
  /** Its clock at its tick 0, UTC ms; omitted is the playtest's clock. */
  readonly clock?: number;
}

/** The clock a playtest's processes start at when it names none: 2026-01-01T00:00:00Z. */
export const PLAYTEST_CLOCK = 1767225600000;

/** Declare a playtest: the default export of a `tests/*.og.ts` file. */
export function playtest(desc: PlaytestDesc): PlaytestDesc {
  if (!desc.name || !desc.commands || typeof desc.check !== "function") {
    throw new Error("playtest({ name, commands, check }) needs all three");
  }
  return Object.freeze({ ...desc });
}
