// The fused flow fixture's game code (tests/fixtures/fused/src/main.ts),
// which holds only gameplay: the engine starts and draws the flow from
// project.ogt's `flow`. Holding D rolls the level's ball off the set, which
// wins it; the cutscene plays, the arcade's puck lands in 2D physics, and
// the live-action ending plays to its end. Q gives the level up instead,
// which reaches the lost ending. crates/og-runtime/tests/flow_acceptance.rs
// checks every cut of the same routes for readiness and a restore's hash.
//
// Ways this could fail: the level's outcome never reached (the ball's
// impulse lost, or 3D physics not running in the level's scene); the
// arcade never advancing (the 2D profile not switched on, so the puck never
// rests in the xy plane); the flow not declared, so nothing plays.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const text = (b: Uint8Array) => new TextDecoder().decode(b);
const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const fixture = join(repo, "packages/engine/tests/fixtures/fused");

interface Flow {
  node: string;
  ended: boolean;
}

function play(keys: string): Flow {
  const out = mkdtempSync(join(tmpdir(), "og-fused-"));
  try {
    // The routes are the flow's and the simulation's, never a frame's: no
    // renderer, so the run is its 726 ticks alone (drawn on a CPU adapter
    // they took 80 s to 120 s, past the test's budget).
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", `${keys};12:dump_flow:flow.json;12.1:quit`, "--out", out], {
      cwd: repo,
      env: { ...process.env, OG_RENDERER: "none" },
    });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    return (JSON.parse(readFileSync(join(out, "flow.json"), "utf8")) as { flows: Flow[] }).flows[0]!;
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

test("the fused fixture's game wins the level, lands the arcade's puck and reaches the ending", () => {
  const f = play("3:key_down:KeyD");
  expect(f.node).toBe("ending");
  expect(f.ended).toBe(true);
}, 120_000);

test("giving the level up reaches the lost ending", () => {
  const f = play("3:key_down:KeyQ;3.1:key_up:KeyQ");
  expect(f.node).toBe("lost");
  expect(f.ended).toBe(true);
}, 120_000);
