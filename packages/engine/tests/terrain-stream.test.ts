// Terrain tile streaming leaves the simulation alone (phase 3): the
// fx-showcase terrain station (a 256 m valley the hero walks across,
// sampling the ground under it every tick) runs through `og run` twice,
// once with every wanted tile decoded before each frame
// (`terrain_stream:resident`) and once streamed on the decode thread within
// a quarter-MiB tile budget (`terrain_stream:stream:0.25`), and the world
// hash is the same after every tick. Since 441f4e6e the first frame that
// draws a streamed terrain settles its tiles as a capture does, so at the
// station's own camera both runs hold every wanted tile (all five) at its
// first frame: the first test proves the hashes agree with nothing
// streaming after the settle. The second starts the camera far off the
// valley, where its first frame wants only the root tile, and moves it to
// the station's eye a second later: the four finer tiles are then wanted
// after the first frame, the streamed run has them waiting and catches up
// over the next frames, and its hashes still equal the resident run's at
// every tick, which proves the stream itself leaves the simulation alone.
//
// Ways this could fail: a tick reads resident tiles (the hero's height,
// collision or og.terrain.sample changes while tiles are in flight), so the
// hashes part; the probe command does not reach the streamer, so both runs
// stream the same way and the comparison proves nothing; dump_stream's
// terrain section misses the mode, the budget or the tiles; the streamed run
// never catches up and still has tiles waiting at the end; the camera move
// wants no tile the first frame did not (nothing streams after the settle,
// and the second test proves no more than the first).
import { expect, test } from "bun:test";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const project = join(repo, "templates", "fx-showcase");
const work = join(repo, "target", "evidence", "terrain-stream");

type Terrain = { mode: string; budget: number; terrains: number; resident: number; waiting: number; inFlight: number; cached: number; cachedBytes: number; decoded: number };

const TICKS = 600;

type Run = { hashes: string[]; first: Terrain; last: Terrain; streams: Terrain[]; ticks: number };

/**
 * One run of `ticks` ticks at the terrain station: `setup` after the
 * station is shown, then `moves` (each `t:action`), a world dump each tick
 * and, with `everyTick`, a stream dump each tick (`streams[k - 1]` is the
 * streamer's state after the frames before tick k).
 */
function run(name: string, mode: string, ticks: number, o: { setup?: string[]; moves?: string[]; everyTick?: boolean } = {}): Run {
  const out = join(work, name);
  const steps = [`0:terrain_stream:${mode}`, "0:station:terrain", ...(o.setup ?? []).map((s) => `0:${s}`)];
  // A step at t seconds runs at the start of tick round(60 t): one dump a
  // tick, and the streamer's tiles after the first frame at the station.
  const moves = [...(o.moves ?? [])];
  for (let k = 1; k <= ticks; k++) {
    while (moves.length > 0 && Math.round(60 * Number(moves[0]!.split(":")[0])) <= k) steps.push(moves.shift()!);
    steps.push(`${k / 60}:dump_world:hash-${k}.json`);
    if (k === 1 || o.everyTick) steps.push(`${k / 60}:dump_stream:stream-${k}.json`);
  }
  steps.push(`${ticks / 60}:dump_stream:last.json`, `${ticks / 60}:screenshot:last.png`, `${ticks / 60 + 0.05}:quit`);
  const r = Bun.spawnSync([ogBin(), "run", project, "--headless", "--size", "320x180", "--commands", steps.join(";"), "--out", out, "--json"], {
    cwd: repo,
    env: { ...process.env, OG_ENGINE_PACKAGE: pkg },
  });
  const text = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(text + new TextDecoder().decode(r.stderr));
  const summary = JSON.parse(text.trim().split("\n").pop()!) as { ticks: number; warnings: string[] };
  expect(summary.warnings).toEqual([]);
  const read = (f: string) => JSON.parse(readFileSync(join(out, f), "utf8"));
  const hashes: string[] = [];
  for (let k = 1; k <= ticks; k++) hashes.push(read(`hash-${k}.json`).hash as string);
  const streams: Terrain[] = [];
  if (o.everyTick) for (let k = 1; k <= ticks; k++) streams.push(read(`stream-${k}.json`).terrain as Terrain);
  return { hashes, first: read("stream-1.json").terrain as Terrain, last: read("last.json").terrain as Terrain, streams, ticks: summary.ticks };
}

test(
  "the fx-showcase terrain station hashes the same over 600 ticks with its tiles resident or streamed",
  () => {
    rmSync(work, { recursive: true, force: true });
    mkdirSync(work, { recursive: true });
    const resident = run("resident", "resident", TICKS);
    const streamed = run("stream", "stream:0.25", TICKS);
    writeFileSync(join(work, "determinism.json"), JSON.stringify({ resident, streamed }, null, 2) + "\n");

    // Both runs did what they were told: the resident one had every wanted
    // tile on the first frame at the station, and so had the streamed one,
    // because the first frame that draws a streamed terrain settles its
    // tiles as a capture does (web findings #5: no frame draws a terrain
    // before any of its tiles arrived); its budget is its own, and the
    // tiles the hero's walk wants after that frame stream within it.
    expect([resident.first.mode, resident.first.budget, resident.first.terrains]).toEqual(["resident", 0, 1]);
    expect([streamed.first.mode, streamed.first.budget, streamed.first.terrains]).toEqual(["stream", 0.25 * 1048576, 1]);
    expect(resident.first.resident).toBeGreaterThan(0);
    expect([resident.first.waiting, resident.first.inFlight]).toEqual([0, 0]);
    expect([streamed.first.waiting, streamed.first.inFlight]).toEqual([0, 0]);
    expect(streamed.first.resident).toBe(resident.first.resident);
    // The settle decoded them on the stream's own thread; by the end the
    // streamed run has nothing waiting (the station's walk stays within
    // the tiles the first frame wanted, so nothing more streams here).
    expect(streamed.first.decoded).toBeGreaterThan(0);
    expect(streamed.last.waiting).toBe(0);
    expect(streamed.last.resident).toBe(resident.last.resident);
    expect(streamed.last.decoded).toBeGreaterThanOrEqual(streamed.first.decoded);

    expect(resident.ticks).toBeGreaterThan(TICKS - 1);
    expect(resident.hashes.length).toBe(TICKS);
    expect(resident.hashes.every((h) => /^[0-9a-f]{16}$/.test(h))).toBeTrue();
    expect(new Set(resident.hashes).size).toBe(TICKS);
    expect(streamed.hashes).toEqual(resident.hashes);
  },
  slow(240_000),
);

/** Ticks of the second test, the move at its second second. */
const MOVE_TICKS = 240;
const MOVE_AT = 60;

test(
  "tiles the camera wants after the first frame stream in later, and the hashes still agree at every tick",
  () => {
    mkdirSync(work, { recursive: true });
    // The station's camera moved far east of the valley (beyond its far
    // plane, so the first frame wants only the root tile), then at 1 s back
    // to the station's own eye (showP2: 38 m west, 21 m up, 64 m south of
    // the valley's centre), where all five tiles are wanted.
    const o = { setup: ["set:og.Camera.position=13000,20,0"], moves: [`${MOVE_AT / 60}:set:og.Camera.position=2962,21,64`], everyTick: true };
    const resident = run("move-resident", "resident", MOVE_TICKS, o);
    const streamed = run("move-stream", "stream:0.25", MOVE_TICKS, o);
    const pick = (r: Run) => r.streams.map((s) => [s.resident, s.waiting, s.inFlight, s.decoded]);
    writeFileSync(join(work, "move-determinism.json"), JSON.stringify({ resident: { hashes: resident.hashes, streams: pick(resident) }, streamed: { hashes: streamed.hashes, streams: pick(streamed) } }, null, 2) + "\n");

    // The first frame wanted the root tile alone, in both runs (settled).
    expect([streamed.first.mode, streamed.first.terrains]).toEqual(["stream", 1]);
    expect([resident.first.resident, streamed.first.resident]).toEqual([1, 1]);
    expect([streamed.first.waiting, streamed.first.inFlight]).toEqual([0, 0]);
    // Until the move nothing more is wanted.
    const before = streamed.streams.slice(0, MOVE_AT);
    expect(before.every((s) => s.resident === 1 && s.waiting === 0)).toBeTrue();
    // The resident run holds every tile the moved camera wants from the
    // frame of the move on; the streamed run asked for the four finer ones
    // that frame (a tile requested is never handed over the frame it was
    // asked for), so they wait, then arrive over the next frames.
    const after = (r: Run) => r.streams.slice(MOVE_AT);
    expect(after(resident).every((s) => s.resident === 5 && s.waiting === 0)).toBeTrue();
    const moved = streamed.streams[MOVE_AT]!;
    expect([moved.resident, moved.waiting, moved.inFlight]).toEqual([1, 4, 4]);
    const caughtUp = after(streamed).findIndex((s) => s.resident === 5 && s.waiting === 0);
    expect(caughtUp).toBeGreaterThan(0);
    expect(after(streamed).slice(caughtUp).every((s) => s.resident === 5 && s.waiting === 0)).toBeTrue();
    expect([streamed.last.resident, streamed.last.waiting, streamed.last.inFlight]).toEqual([5, 0, 0]);
    expect(streamed.last.decoded).toBe(5);

    // And the stream changed nothing the ticks computed.
    expect(resident.hashes.length).toBe(MOVE_TICKS);
    expect(new Set(resident.hashes).size).toBe(MOVE_TICKS);
    expect(streamed.hashes).toEqual(resident.hashes);
  },
  slow(180_000),
);
