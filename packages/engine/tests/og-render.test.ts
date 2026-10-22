// `og render` (docs/design.md §2.9): two runs of one script give the same
// frames byte for byte (frames.json's SHA-256 of each frame's RGBA rows)
// and the same audio, a render split over jobs (snapshots restored, warm-up
// frames drawn) gives the frames of the render in one, and an fps that does
// not divide the tick rate is refused naming the nearest that do.
//
// Ways this could fail: a wall-clock read or a renderer state not reset
// (TAA history, adapted exposure) makes a frame differ between runs; a job
// restores the wrong tick or replays a step the leader already ran; the
// frame read back is the previous one (the readback lags the draw).
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const text = (b: Uint8Array) => new TextDecoder().decode(b);

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
// The template's menu: the pointer to Start, a press, the game.
const SCRIPT = "0.2:touch_move:60,130;0.4:touch_start;0.5:touch_end";

interface Frames {
  rate: number;
  frames: { index: number; tick: number; sha256: string }[];
  audio: { sha256: string } | null;
}

function render(out: string, ...extra: string[]): Frames {
  const r = Bun.spawnSync([ogBin(), "render", "templates/game", "--commands", SCRIPT, "--fps", "20", "--to", "1.5", "--size", "160x90", "--audio", "--no-check", "--out", out, ...extra], { cwd: repo });
  if (r.exitCode !== 0) throw new Error(`og render exited ${r.exitCode}: ${text(r.stderr)}`);
  return JSON.parse(readFileSync(join(out, "frames.json"), "utf8")) as Frames;
}

test("two og render runs of one script give the same frames and audio, and --jobs gives the frames of one job", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-render-"));
  try {
    const a = render(join(dir, "a"));
    const b = render(join(dir, "b"));
    expect(a.frames.length).toBe(30);
    expect(a.frames.map((f) => f.tick)).toEqual(Array.from({ length: 30 }, (_, i) => i * (a.rate / 20)));
    // The frames move: a render of one frame repeated would pass the rest.
    expect(new Set(a.frames.map((f) => f.sha256)).size).toBeGreaterThan(10);
    expect(b.frames.map((f) => f.sha256)).toEqual(a.frames.map((f) => f.sha256));
    expect(b.audio!.sha256).toBe(a.audio!.sha256);
    const j = render(join(dir, "j"), "--jobs", "3");
    expect(j.frames.map((f) => f.sha256)).toEqual(a.frames.map((f) => f.sha256));
    expect(j.audio!.sha256).toBe(a.audio!.sha256);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 600_000);

test("og render refuses an fps that does not divide the tick rate, naming the nearest that do", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-render-"));
  try {
    const r = Bun.spawnSync([ogBin(), "render", "templates/game", "--commands", "0:pause", "--fps", "7", "--to", "1", "--size", "64x36", "--no-check", "--out", dir], { cwd: repo });
    expect(r.exitCode).not.toBe(0);
    expect(text(r.stderr)).toContain("--fps 7 does not divide the game's tick rate 60");
    expect(text(r.stderr)).toContain("6 and 10");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
