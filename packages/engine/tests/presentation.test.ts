import { gpuTest } from "./support/gpu.ts";
// The presentation gate (docs/design.md §2.4): a capture is a function of
// the world and the presentation checkpoint, never of the frames drawn
// before it. On the project skeleton and on vehicle-terrain (a moving car,
// TAA, tyre smoke switched on and off by a slide, decals, a HUD), three
// ways of producing the same tick's capture give the same PNG bytes, world
// hash and presentation hash; and GGPO's SyncTest, rolling back four ticks
// and playing them again on every tick of the slide, reaches the same
// hashes. Each run is its own process (tests/support/presentation-capture.ts).
//
// Ways it could fail, and the check that catches each:
// - TAA's history, the jitter's phase or the shadow spiral carried from
//   the frames before → `capture` (no frames) against `every`
// - particles stepped by the frame's system instead of each tick's (the
//   smoke's on and off lost when frames are skipped) → `capture`
// - the checkpoint not restoring the particles' ticks, so a fork's other
//   branch leaks into the restored one → `restore` (a fork)
// - a restore that changes the simulation (terrain edits replayed into
//   physics) → the world hashes
// - presentation state not stepped deterministically under rollback → the SyncTest
// - a restore that brings back the world but not the display's frame history
//   (TAA's, the previous camera and objects for motion vectors, the jitter's
//   phase, interpolation, the streamed terrain tiles and their fades) → the
//   display forks, which compare every display frame after a restore with
//   the uninterrupted run's
import { expect } from "bun:test";
import { createHash } from "node:crypto";
import { cpSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject, ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
  spawn(cmd: string[]): { kill(): void };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");

type Run = { mode: string; tick: number; hash: string; presentation: string; png: string; first?: string; checked?: number; mismatches?: string[] };

function run(project: string, mode: string, args: string[]): Run {
  const r = Bun.spawnSync(["bun", "tests/support/presentation-capture.ts", "--project", project, "--mode", mode, ...args], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${mode}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Run;
}

/** The three ways at one tick: all equal, the fork's first pass too. */
function threeWays(project: string, args: string[]): void {
  const every = run(project, "every", args);
  const capture = run(project, "capture", args);
  const fork = run(project, "restore", args);
  const same = ({ tick, hash, presentation, png }: Run) => ({ tick, hash, presentation, png });
  expect(same(capture)).toEqual(same(every));
  expect(same(fork)).toEqual(same(every));
  expect(fork.first).toBe(every.png);
}

// Throttle, then a handbrake turn from tick 120: the smoke starts and stops.
const DRIVE = ["--ticks", "156", "--size", "960x540", "--inputs", "12:key_down:KeyW;120:key_down:KeyD;120:key_down:Space;138:key_up:Space"];

gpuTest("the skeleton's capture is the same drawn every frame, without frames, and after a fork", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-presentation-"));
  try {
    cpSync(join(repo, "packages", "create", "template"), dir, { recursive: true });
    cookProject(dir, repo);
    threeWays(dir, ["--ticks", "32", "--size", "640x360"]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);

gpuTest("vehicle-terrain's capture mid-slide is the same drawn every frame, without frames, and after a fork", () => {
  const project = join(repo, "tests", "fixtures", "vehicle-terrain");
  cookProject(project, repo);
  // The fork at 60, before the slide; the snapshot at 140, in it.
  threeWays(project, [...DRIVE, "--fork-at", "60", "--restore-at", "140"]);
}, 120_000);

gpuTest("SyncTest: four ticks rolled back and played again on every tick of the slide reach the same hashes", () => {
  const project = join(repo, "tests", "fixtures", "vehicle-terrain");
  cookProject(project, repo);
  const sync = run(project, "synctest", [...DRIVE, "--restore-at", "110", "--rollback", "4"]);
  expect(sync.mismatches).toEqual([]);
  expect(sync.checked).toBe(47);
}, 120_000);

/** Display frames after a fork: every one the uninterrupted run's, byte for byte. */
function displayFork(project: string, args: string[], checked: number): void {
  const r = run(project, "display", args);
  expect(r.mismatches).toEqual([]);
  expect(r.checked).toBe(checked);
}

gpuTest("vehicle-terrain's display frames after a fork mid-slide are the uninterrupted run's (TAA, motion vectors)", () => {
  const project = join(repo, "tests", "fixtures", "vehicle-terrain");
  cookProject(project, repo);
  displayFork(project, [...DRIVE, "--fork-at", "110"], 46);
}, 120_000);

gpuTest("terrain-stream's display frames after a fork before its tiles stream in are the uninterrupted run's (tile arrival, fades)", () => {
  const project = join(repo, "tests", "fixtures", "terrain-stream");
  cookProject(project, repo);
  displayFork(project, ["--ticks", "110", "--fork-at", "20", "--size", "960x540"], 90);
}, 120_000);

gpuTest("motion-feet's display frames after a fork mid-course are the uninterrupted run's (og.motion's Presenter)", () => {
  const project = join(repo, "tests", "fixtures", "motion-feet");
  cookProject(project, repo);
  displayFork(project, ["--ticks", "360", "--fork-at", "300", "--size", "640x360", "--settle"], 60);
}, 120_000);

// og.motion's results are applied as of the tick they were due, however
// late they come back (crates/og-runtime/src/bind/motion/present.rs): no tick
// waits for one, the Presenter falls behind and catches up, and a capture
// settles first. Three runs of motion-feet's course, one with every GPU
// result asked for 600 ms after its batch started
// (OG_MOTION_READBACK_DELAY_MS) and one beside busy CPU loops, capture the
// same screenshots and measure the same feet; the late run's world ran on
// while results were late. A wall clock left in a step (a skip to catch
// up, a result applied at the tick it arrived) moves the frames.
interface MotionAgent { entity: number; framesShown: number; framesStarved: number; framesWarmup: number; footing: unknown; plantedSlide: unknown }
interface MotionRun { agents: Pick<MotionAgent, "entity" | "framesShown" | "framesStarved" | "framesWarmup" | "footing" | "plantedSlide">[]; shots: string[]; late: { ticks: number; most: number } }

function motionCourse(out: string, env: Record<string, string> = {}): MotionRun {
  const project = join(repo, "tests", "fixtures", "motion-feet");
  const shots = ["3", "6.5", "10", "14"];
  const commands = `${shots.map((t, i) => `${t}:screenshot:s${i}.png`).join(";")};14.2:dump_motion:feet.json;14.25:quit`;
  const r = Bun.spawnSync([ogBin(), "run", "--headless", project, "--size", "640x360", "--commands", commands, "--out", out, "--json"], { cwd: repo, env: { ...process.env, ...env } });
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
  const model = (JSON.parse(readFileSync(join(out, "feet.json"), "utf8")) as { model: { agents: MotionAgent[]; late: { ticks: number; most: number } } }).model;
  return {
    agents: model.agents.map(({ entity, framesShown, framesStarved, framesWarmup, footing, plantedSlide }) => ({ entity, framesShown, framesStarved, framesWarmup, footing, plantedSlide })),
    shots: shots.map((_, i) => createHash("sha256").update(readFileSync(join(out, `s${i}.png`))).digest("hex")),
    late: model.late,
  };
}

gpuTest("og.motion captures the same frames and feet with late GPU results and under load, and no tick waits", () => {
  cookProject(join(repo, "tests", "fixtures", "motion-feet"), repo);
  const dir = mkdtempSync(join(tmpdir(), "og-motion-runs-"));
  const busy: { kill(): void }[] = [];
  try {
    const quiet = motionCourse(join(dir, "quiet"));
    expect(quiet.agents.length).toBe(2);
    expect(quiet.agents.every((a) => a.framesShown > 300)).toBe(true);
    const late = motionCourse(join(dir, "late"), { OG_MOTION_READBACK_DELAY_MS: "600" });
    // The world ran on past results due and not back: no tick waited.
    expect(late.late.ticks).toBeGreaterThan(30);
    expect(late.shots).toEqual(quiet.shots);
    expect(late.agents).toEqual(quiet.agents);
    for (let i = 0; i < 8; i++) busy.push(Bun.spawn(["bun", "-e", "for (;;) {}"]));
    const loaded = motionCourse(join(dir, "loaded"));
    expect(loaded.shots).toEqual(quiet.shots);
    expect(loaded.agents).toEqual(quiet.agents);
  } finally {
    for (const b of busy) b.kill();
    rmSync(dir, { recursive: true, force: true });
  }
}, 240_000);

// og run --capture-only draws no display frame, only the captures its
// commands ask for; each is a function of the tick-stepped presentation
// state, so its PNGs are the bytes of a run that draws every frame.
gpuTest("og run --capture-only screenshots vehicle-terrain with the bytes of a run that draws every frame", () => {
  const project = join(repo, "tests", "fixtures", "vehicle-terrain");
  cookProject(project, repo);
  const dir = mkdtempSync(join(tmpdir(), "og-capture-only-"));
  const shots = ["0.5", "1.3", "2.1"];
  const sha = (b: Uint8Array): string => createHash("sha256").update(b).digest("hex");
  const commands = `${shots.map((t, i) => `${t}:screenshot:s${i}.png`).join(";")};2.2:quit`;
  const runOnce = (out: string, extra: string[]): Uint8Array[] => {
    const r = Bun.spawnSync([ogBin(), "run", "--headless", project, "--size", "640x360", "--commands", commands, "--out", out, ...extra], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
    return shots.map((_, i) => readFileSync(join(out, `s${i}.png`)));
  };
  try {
    const drawn = runOnce(join(dir, "drawn"), []);
    const only = runOnce(join(dir, "only"), ["--capture-only"]);
    for (let i = 0; i < shots.length; i++) expect(sha(only[i]!)).toBe(sha(drawn[i]!));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);

// og run --forks: each snapshot of the run (the world, the presentation
// checkpoint, og.platform's state), restored and played on with the steps
// still queued at it, gives the run's world hash on every tick, its
// screenshots' bytes and its presentation hash at the end. A fork that
// stops at its own tick (a `quit` left with the probe, the queue lost) or
// draws other bytes fails the run.
gpuTest("og run --forks plays three forks of a vehicle-terrain run to its frames and hashes", () => {
  const project = join(repo, "tests", "fixtures", "vehicle-terrain");
  cookProject(project, repo);
  const dir = mkdtempSync(join(tmpdir(), "og-forks-"));
  const commands = "0.2:key_down:KeyW;1:screenshot:a.png;2:key_down:KeyD;2:key_down:Space;2.3:key_up:Space;2.4:screenshot:b.png;2.6:quit";
  try {
    for (const extra of [[], ["--capture-only"]]) {
      const r = Bun.spawnSync([ogBin(), "run", "--headless", project, "--size", "640x360", "--commands", commands, "--out", dir, "--forks", "3", "--json", ...extra], { cwd: repo });
      const err = new TextDecoder().decode(r.stderr);
      expect(r.exitCode, err).toBe(0);
      const lines = new TextDecoder().decode(r.stdout).trim().split("\n");
      const summary = JSON.parse(lines[lines.length - 1]!) as { tick: number; hash: string; forks: Array<{ tick: number; hash: string; mismatches: string[] }> };
      expect(summary.forks.map((f) => f.tick)).toEqual([39, 78, 117]);
      for (const f of summary.forks) {
        expect(f.mismatches).toEqual([]);
        expect(f.hash).toBe(summary.hash);
      }
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
