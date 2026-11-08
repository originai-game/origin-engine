// Playtests (`tests/*.og.ts`): bun:test files whose tests run the game.
// `og test` builds the game, then runs each file under `bun test`; inside a
// test, `await runPlaytest({ commands })` plays the command script headless
// through `og run --commands` in fresh processes and resolves to what the
// run wrote, read with `expect` as any other value.
//
// Write properties first: `invariants` are checked after every tick (the
// first failing tick and its value are the failure), the engine's default
// invariants (NaN, ranges, dangling entities, og.WorldBounds) and the game's
// og.assert calls run beside them, and `require` names the og.assert
// sometimes/reachable/each sites the run must see fire. They, the script's
// `expect:` steps and every violation are one assertion of the test:
// `runPlaytest` rejects with each failure, its tick and its value. Keep a
// frame only for what only a frame shows (a look, a layout), inside
// `run.frame`.
//
// Without a renderer (the platform's engine and probe sandboxes carry none;
// `OG_RENDERER=none` makes any machine so) the run is the same run, but a
// `screenshot` writes no PNG: the checks that read a frame go inside
// `run.frame(files, () => ...)`, which runs them with a renderer and
// otherwise reports each file "no renderer, frame unchecked" (never passed;
// `og test` counts it). Reading a frame outside it without a renderer throws
// FrameUnchecked, which fails the test.
//
// Every test of a playtest makes an assertion (`og test` runs them with
// `expect.hasAssertions()`): one that made none fails.
//
//   import { describe, expect, test } from "bun:test";
//   import { runPlaytest } from "@origingame/engine/playtest";
//
//   describe("the trickster", () => {
//     test("spins after S", async () => {
//       const run = await runPlaytest({ commands: "0.5:type:s;2.0:screenshot:spin.png;2.2:dump_entity:trickster:t.json;2.5:quit" });
//       expect(run.quit, "the script reached quit").toBe(true);
//       expect(run.json<{ components: Record<string, { speed: number }> }>("t.json").components.Spin!.speed).toBeGreaterThan(0);
//       run.frame("spin.png", () => expect(run.png("spin.png").width, "the frame is 1280 wide").toBe(1280));
//     });
//   });
//
// A tolerance is absolute: `expect(Math.abs(actual - expected)).toBeLessThanOrEqual(tolerance)`
// (`toBeCloseTo` takes decimal places, not a tolerance).

import { decodePng, encodePngBest } from "./node/png.ts";
import type { LocalFake } from "./platform-local.ts";

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
  /** A written PNG's size. Throws when the run wrote no such file (and, for a screenshot taken without a renderer, a FrameUnchecked: read frames inside `frame`). */
  png(file: string): { width: number; height: number; bytes: number };
  /** A written PNG decoded: its RGBA8 rows, for checks that read pixels (contrast, a colour at a point). Throws as `png` does. */
  pixels(file: string): { width: number; height: number; rgba: Uint8Array };
  /** A written text output. */
  text(file: string): string;
  /** A written WAV (`listen`): its rate, frames, peak and RMS (16-bit PCM read back, both channels). */
  wav(file: string): { rate: number; frames: number; peak: number; rms: number };
  /** The runs of PlaytestDesc.then, in order, each with its own files (empty without `then`). */
  readonly then: readonly PlaytestRun[];
  /** The run's assertions (the probe's `assertions`): violations, sites, buckets and what never fired. */
  readonly assertions: PlaytestAssertions;
  /** og.net's shared hash at the end (og.net.hash): the world without this process's session, what lockstep peers compare. */
  readonly netHash: string;
  /** With PlaytestDesc.room: og.net.status at the end (running, the start tick, the seats, the first divergent tick); null outside a room. */
  readonly lockstep: PlaytestLockstep | null;
  /** With PlaytestDesc.room: the room's other processes (peer scripts 1, 2, ...), each with its own files; empty outside a room. */
  readonly peers: readonly PlaytestRun[];
  /**
   * The checks that read frames (`png`, `pixels`, a comparison with references/): `check`
   * runs when the run had a renderer. Without one it does not run, each of `files` is
   * reported "no renderer, frame unchecked" (never passed), and the test goes on to the
   * checks after it. Every check outside `frame` runs either way.
   */
  frame(files: string | readonly string[], check: () => void): void;
}

/** One violated assertion: its first failure. */
export interface PlaytestViolation {
  readonly kind: string;
  /** `game` (og.assert), `default` (from reflection) or `playtest` (an invariant). */
  readonly origin: string;
  readonly name: string;
  readonly location: string;
  readonly tick: number;
  readonly entity?: string | null;
  readonly component?: string | null;
  readonly field?: string | null;
  readonly value: unknown;
  readonly expected: unknown;
  readonly message: string;
  readonly details: unknown;
  readonly failures: number;
}

/** A run's assertion report (IDL `assert.AssertReport`). */
export interface PlaytestAssertions {
  readonly violations: readonly PlaytestViolation[];
  readonly uncovered: ReadonlyArray<{ kind: string; name: string; location: string; key?: string; ran?: boolean }>;
  readonly sites: ReadonlyArray<{ kind: string; origin: string; name: string; location: string; ran: boolean; fired: boolean; firstTick: number | null; failures: number; buckets?: ReadonlyArray<{ key: string; fired: boolean; firstTick: number | null }> }>;
  readonly defaults: { fields: number; ticks: number; values: number; totalMs: number; meanUs: number; maxUs: number };
}

/** og.net.status of a playtest process in a room, at its end. */
export interface PlaytestLockstep {
  readonly running: boolean;
  readonly peer: number | null;
  readonly startTick: number | null;
  readonly players: number | null;
  readonly waits: number;
  /** The first divergent tick, once og.Desync was sent. */
  readonly desync: number | null;
}

/** PlaytestDesc.room: the script's processes play together in one room. */
export interface PlaytestRoom {
  /** How many processes join the room (2 to 16). */
  readonly players: number;
  /** Each process's script by its place (0 is the playtest's own `commands`); a process without one runs `commands`. */
  readonly commands?: readonly string[];
  /** The room's name in the relay (default `playtest`). */
  readonly name?: string;
  /**
   * The tick every process stops at (`og run --ticks`), when the scripts do not end first: their
   * `netHash` there is what lockstep makes equal. Leave it out to end each script by its `quit`
   * (or its last step). A script's steps are on the room's shared ticks once lockstep runs (a
   * peer that takes the authority's snapshot at the start moves to its tick and its steps stay
   * on theirs, a step whose tick it jumped past running at once), so a `quit` or an `expect` at
   * tick t runs at tick t on every peer.
   */
  readonly ticks?: number;
}

/**
 * What reading a screenshot the run took without a renderer throws: the test fails, naming the
 * frame. Put frame checks inside `run.frame`, which reports them unchecked instead.
 */
export class FrameUnchecked extends Error {
  readonly file: string;
  constructor(file: string) {
    super(`${file}: no renderer, frame unchecked`);
    this.name = "FrameUnchecked";
    this.file = file;
  }
}

/** What `runPlaytest` plays: the script and the processes it runs in. */
export interface PlaytestDesc {
  /** `t:action[:args];...`, as `og run --commands`. End it with `quit`. */
  readonly commands: string;
  /** The replay seed; omitted is GameDesc.seed. */
  readonly seed?: number;
  /** The render target, `WIDTHxHEIGHT` (default 1280x720). */
  readonly size?: string;
  /**
   * `action[:args];...` run once through the command channel after `start`, before the first
   * tick (`og run --setup`), in every process of the playtest: the stand-ins for what a host does
   * around the game, such as `cover:12,12,40,40`, the published page's Back over the play surface
   * (`Viewport.covered`), so the frames lay out as the page's. Not part of `commands`: the
   * published page plays a script without it and covers itself.
   */
  readonly setup?: string;
  /** Run the script this many times with fresh game state (default 1) and report every end hash in `run.hashes`: a determinism check. A resident host may reuse its process, never its world. The files are the first run's. */
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
  /**
   * How the platform services are faked in every process of the playtest (`og run --platform-fake`):
   * the products, each method's outcomes in order (`{ "purchase.buy": ["cancelled", "ok"] }`), the
   * player, friends, invites, notices sent at a tick, and the methods the host lacks. Omitted: every
   * purchase, ad, achievement, identity and share call succeeds.
   */
  readonly platform?: LocalFake;
  /**
   * Play the script in a room (docs/design.md §2.8, §2.10): the playtest starts a relay
   * (`og relay`, in its own process) and runs `players` processes at once, each `og run
   * --room ws://127.0.0.1:P/<name>` with its own script (`room.commands[i]`, else `commands`)
   * and its own files. The first is the run; the others are `run.peers`. Every process stops
   * at `room.ticks`, or at its script's `quit`, which lands on one tick of the room for all.
   * `runs` is refused with it; each `then` step relaunches every process at once into a room of
   * its own (`<name>.then-<n>`), each running the step's script, its runs in `run.then[n].peers`;
   * `store: "keep"` keeps one store per place across them.
   */
  readonly room?: PlaytestRoom;
  /** More fresh runs after the script, in order (a relaunch): their runs are `run.then`. With `room`, every place of the room relaunches at once. */
  readonly then?: readonly PlaytestStep[];
  /**
   * Properties checked after every tick of every process of the playtest, written first: each is
   * an `expect:` form but `region` and `frame` (`state:lives>=0`, `entity:hero:og.Transform.position.y>-5`),
   * run as an `invariant:` step from tick 0. The first tick one fails is a failure naming the tick
   * and the observed value. The engine's default invariants (no NaN, every field inside its
   * min..max, no dangling Entity, og.WorldBounds) and the game's og.assert.always run beside them.
   */
  readonly invariants?: readonly string[];
  /**
   * The og.assert.sometimes, reachable and each sites (by name) the playtest must see fire, in any
   * of its processes (the script, its `then` steps, a room's peers): one that never fired in any
   * fails the playtest. Every other site that never fired is listed as uncovered,
   * never a failure. `each` names a site, all of whose buckets the run saw must hold; `name[key]`
   * names one bucket.
   */
  readonly require?: readonly string[];
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

/**
 * Plays `desc` headless, in fresh processes of the game `og test` built, and resolves to what the
 * first run wrote (its `runs`, `then` steps and room peers beside it). The script's `expect:`
 * steps, `invariants`, `require` and every og.assert violation make one assertion: it rejects
 * with each failure. A run that does not finish rejects too. Call it inside a test of a
 * `tests/*.og.ts` file that `og test` runs; each call is new processes, never a reused world.
 */
export function runPlaytest(desc: PlaytestDesc): Promise<PlaytestRun> {
  // The runner is node code, reached at run time so a game type-checks its playtests without node's declarations.
  const runner = (import.meta as unknown as { url: string }).url.replace(/[^/]*$/, "node/run-playtest.ts");
  return (import(runner) as Promise<{ runPlaytest(desc: PlaytestDesc): Promise<PlaytestRun> }>).then((m) => m.runPlaytest(desc));
}

/**
 * Records a reference frame: the screenshot a run wrote (the probe's fast PNG, large) decoded
 * and written to `reference` at maximum compression (`encodePngBest`: the same pixels in a file
 * 3 to 8 times smaller, about a second for a 1280×720 frame), its directory made. Every recorder
 * of a `references/` frame uses it instead of copying the screenshot.
 */
export function recordReference(screenshot: string, reference: string): void {
  // node:fs reached at run time, not imported: a game type-checks its playtests without node's declarations.
  const fs = (globalThis as unknown as { process: { getBuiltinModule(id: string): NodeFs } }).process.getBuiltinModule("node:fs");
  const png = encodePngBest(decodePng(new Uint8Array(fs.readFileSync(screenshot))));
  fs.mkdirSync(reference.replace(/[\\/][^\\/]*$/, "") || ".", { recursive: true });
  fs.writeFileSync(reference, png);
}

interface NodeFs {
  readFileSync(path: string): Uint8Array;
  mkdirSync(path: string, options: { recursive: boolean }): void;
  writeFileSync(path: string, data: Uint8Array): void;
}
