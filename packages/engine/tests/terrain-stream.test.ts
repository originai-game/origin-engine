// Terrain tile streaming leaves the simulation alone (phase 3): the
// fx-showcase terrain station (a 256 m valley the hero walks across,
// sampling the ground under it every tick) runs 600 ticks through `og run`
// twice, once with every wanted tile decoded before each frame
// (`terrain_stream:resident`) and once streamed on the decode thread within
// a quarter-MiB tile budget (`terrain_stream:stream:0.25`), and the world
// hash is the same after every one of those ticks.
//
// Ways this could fail: a tick reads resident tiles (the hero's height,
// collision or og.terrain.sample changes while tiles are in flight), so the
// hashes part; the probe command does not reach the streamer, so both runs
// stream the same way and the comparison proves nothing; dump_stream's
// terrain section misses the mode, the budget or the tiles; the streamed run
// never catches up and still has tiles waiting at the end.
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

function run(name: string, mode: string): { hashes: string[]; first: Terrain; last: Terrain; ticks: number } {
  const out = join(work, name);
  const steps = [`0:terrain_stream:${mode}`, "0:station:terrain"];
  // A step at t seconds runs at the start of tick round(60 t): one dump a
  // tick, and the streamer's tiles after the first frame at the station.
  for (let k = 1; k <= TICKS; k++) {
    steps.push(`${k / 60}:dump_world:hash-${k}.json`);
    if (k === 1) steps.push(`${k / 60}:dump_stream:first.json`);
  }
  steps.push(`${TICKS / 60}:dump_stream:last.json`, `${TICKS / 60}:screenshot:last.png`, `${TICKS / 60 + 0.05}:quit`);
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
  for (let k = 1; k <= TICKS; k++) hashes.push(read(`hash-${k}.json`).hash as string);
  return { hashes, first: read("first.json").terrain as Terrain, last: read("last.json").terrain as Terrain, ticks: summary.ticks };
}

test(
  "the fx-showcase terrain station hashes the same over 600 ticks with its tiles resident or streamed",
  () => {
    rmSync(work, { recursive: true, force: true });
    mkdirSync(work, { recursive: true });
    const resident = run("resident", "resident");
    const streamed = run("stream", "stream:0.25");
    writeFileSync(join(work, "determinism.json"), JSON.stringify({ resident, streamed }, null, 2) + "\n");

    // Both runs did what they were told: the resident one had every wanted
    // tile on the first frame at the station, the streamed one was still
    // decoding them, within its own budget.
    expect([resident.first.mode, resident.first.budget, resident.first.terrains]).toEqual(["resident", 0, 1]);
    expect([streamed.first.mode, streamed.first.budget, streamed.first.terrains]).toEqual(["stream", 0.25 * 1048576, 1]);
    expect(resident.first.resident).toBeGreaterThan(0);
    expect([resident.first.waiting, resident.first.inFlight]).toEqual([0, 0]);
    expect(streamed.first.waiting).toBeGreaterThan(0);
    expect(streamed.first.resident).toBeLessThan(resident.first.resident);
    // By the end the streamed run has caught up.
    expect(streamed.last.waiting).toBe(0);
    expect(streamed.last.resident).toBe(resident.last.resident);
    expect(streamed.last.decoded).toBeGreaterThan(0);

    expect(resident.ticks).toBeGreaterThan(TICKS - 1);
    expect(resident.hashes.length).toBe(TICKS);
    expect(resident.hashes.every((h) => /^[0-9a-f]{16}$/.test(h))).toBeTrue();
    expect(new Set(resident.hashes).size).toBe(TICKS);
    expect(streamed.hashes).toEqual(resident.hashes);
  },
  slow(240_000),
);
