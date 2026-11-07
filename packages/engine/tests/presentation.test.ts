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
import { expect } from "bun:test";
import { cpSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject, ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
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

// og.motion steps by the tick (crates/og-runtime/src/motion/present.rs):
// its batches are taken at a fixed tick and the model is waited for when
// an agent first needs it, so the poses drawn never depend on how fast the
// machine runs. Two runs of motion-feet's course, the second beside busy
// CPU loops, draw the same frames and measure the same feet: a wall clock
// left in (a skip to catch up, a decode budget, the first batch's latency)
// moves the counts between runs, and more under load.
interface MotionAgent { entity: number; framesShown: number; framesStarved: number; framesWarmup: number; footing: unknown; plantedSlide: unknown }

function motionCourse(out: string): Pick<MotionAgent, "entity" | "framesShown" | "framesStarved" | "framesWarmup" | "footing" | "plantedSlide">[] {
  const project = join(repo, "tests", "fixtures", "motion-feet");
  const r = Bun.spawnSync([ogBin(), "run", "--headless", project, "--size", "640x360", "--commands", "14.2:dump_motion:feet.json;14.25:quit", "--out", out, "--json"], { cwd: repo });
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
  const agents = (JSON.parse(readFileSync(join(out, "feet.json"), "utf8")) as { model: { agents: MotionAgent[] } }).model.agents;
  return agents.map(({ entity, framesShown, framesStarved, framesWarmup, footing, plantedSlide }) => ({ entity, framesShown, framesStarved, framesWarmup, footing, plantedSlide }));
}

gpuTest("og.motion draws the same frames and feet on two runs of motion-feet, one under load", () => {
  cookProject(join(repo, "tests", "fixtures", "motion-feet"), repo);
  const dir = mkdtempSync(join(tmpdir(), "og-motion-runs-"));
  const busy: { kill(): void }[] = [];
  try {
    const quiet = motionCourse(join(dir, "quiet"));
    expect(quiet.length).toBe(2);
    expect(quiet.every((a) => a.framesShown > 300)).toBe(true);
    for (let i = 0; i < 8; i++) busy.push(Bun.spawn(["bun", "-e", "for (;;) {}"]));
    const loaded = motionCourse(join(dir, "loaded"));
    expect(loaded).toEqual(quiet);
  } finally {
    for (const b of busy) b.kill();
    rmSync(dir, { recursive: true, force: true });
  }
}, 180_000);
