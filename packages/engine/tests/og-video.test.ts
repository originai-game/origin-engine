// Footage (docs/design.md §2.9): a `type video` cooked by ffmpeg plays on
// an og.Sprite through og.VideoPlayer (tests/fixtures/video: testsrc2 with
// its frame count burned in, every frame different, 30 fps, at a 30 Hz
// tick). The frame at tick 90 is the source's frame 3 s in: of the
// source's frames 89, 90 and 91, decoded by ffmpeg straight from the .mp4,
// 90 is the one the rendered frame is nearest. A seek stops the sound for a
// tick and starts it again at the new time.
//
// The reference frame is references/<tier>/video-90.png beside the fixture
// (recorded by the coordinator on Metal); without one for this tier the
// frame is checked against the source only.
//
// Two players of the video (the fixture's `pair`, the second half a second ahead)
// each show their own frame: their textures differ and each half of the
// frame is nearest its own player's source frame. Broken before: they
// shared the video's texture, which showed the first player's frame.
//
// Ways this could fail: the player's time accumulates in f32 to 2.9999998 s
// and shows frame 89; the texture is not replaced when the frame changes
// (frame 0 forever) or lags a tick behind; the sound starts at 0 after a
// seek, or keeps playing from the old place.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const text = (b: Uint8Array) => new TextDecoder().decode(b);

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const fixture = join(repo, "packages/engine/tests/fixtures/video");
const ffmpeg = process.env.OG_FFMPEG ?? "ffmpeg";
const hasFfmpeg = (() => {
  try {
    return Bun.spawnSync([ffmpeg, "-version"]).exitCode === 0;
  } catch {
    return false;
  }
})();

/** An image as 192x108 RGB, through ffmpeg (area-averaged when shrunk). */
function rgb(input: string[], vf: string): Uint8Array {
  const r = Bun.spawnSync([ffmpeg, "-v", "error", ...input, "-vf", `${vf}scale=192:108:flags=area`, "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
  if (r.exitCode !== 0) throw new Error(text(r.stderr));
  return r.stdout;
}

function rmse(a: Uint8Array, b: Uint8Array): number {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += (a[i]! - b[i]!) ** 2;
  return Math.sqrt(s / a.length) / 255;
}

interface Dump {
  components: Record<string, Record<string, number | boolean>>;
}

test.skipIf(!hasFfmpeg)("a cooked video plays on a sprite: tick 90 shows the source's frame 3 s in, and its sound follows a seek", () => {
  const out = mkdtempSync(join(tmpdir(), "og-video-"));
  try {
    const script = "3:screenshot:video-90.png;3:dump_entity:screen:t90.json;3:set:screen.og.VideoPlayer.time=1;3.034:dump_entity:screen:seek.json;3.067:dump_entity:screen:after.json;3.1:quit";
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    const dump = (f: string) => JSON.parse(readFileSync(join(out, f), "utf8")) as Dump;

    const t90 = dump("t90.json").components;
    expect(t90["og.VideoPlayer"]!.frame).toBe(90);
    expect(t90["og.AudioSource"]!.playing).toBe(true);
    expect(t90["og.AudioSource"]!.sound).toBeGreaterThan(0);

    // The seek (time 1 s at tick 90): the sound stops for that tick, then starts at the video's time.
    const seek = dump("seek.json").components;
    expect(seek["og.VideoPlayer"]!.frame).toBe(31);
    expect(seek["og.AudioSource"]!.playing).toBe(false);
    const after = dump("after.json").components;
    expect(after["og.AudioSource"]!.playing).toBe(true);
    expect(after["og.AudioSource"]!.startOffset as number).toBeCloseTo(31 / 30, 3);

    if (run.tier === "none") return; // no renderer: the frame is unchecked
    const frame = rgb(["-i", join(out, "video-90.png")], "");
    const source = join(fixture, "assets/clips/count.mp4");
    const errors = [89, 90, 91].map((n) => rmse(frame, rgb(["-i", source], `select=eq(n\\,${n}),`)));
    expect(errors[1]!).toBeLessThan(errors[0]!);
    expect(errors[1]!).toBeLessThan(errors[2]!);
    expect(errors[1]!).toBeLessThan(0.12);

    const ref = join(fixture, "references", run.tier, "video-90.png");
    if (existsSync(ref)) expect(rmse(frame, rgb(["-i", ref], ""))).toBeLessThan(0.01);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);

test.skipIf(!hasFfmpeg)("two players of one video at different times show different frames", () => {
  const out = mkdtempSync(join(tmpdir(), "og-video-pair-"));
  try {
    const script = "0.5:pair;3:screenshot:pair.png;3:dump_entity:screen:a.json;3:dump_entity:screen2:b.json;3.1:quit";
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    const player = (f: string) => (JSON.parse(readFileSync(join(out, f), "utf8")) as Dump).components["og.VideoPlayer"]!;
    const a = player("a.json");
    const b = player("b.json");
    expect(a.frame).toBe(90);
    expect(b.frame).toBe(105);
    expect(a.texture).toBeGreaterThan(0);
    expect(b.texture).toBeGreaterThan(0);
    expect(b.texture).not.toBe(a.texture);
    if (run.tier === "none") return;
    // Each sprite fills a quarter of the view: the left and right halves of
    // its middle band.
    const shot = join(out, "pair.png");
    const half = (x: string) => rgb(["-i", shot], `crop=iw/2:ih/2:${x}:ih/4,`);
    const source = join(fixture, "assets/clips/count.mp4");
    const src = (n: number) => rgb(["-i", source], `select=eq(n\\,${n}),`);
    const [left, right] = [half("0"), half("iw/2")];
    const [s90, s105] = [src(90), src(105)];
    // Each half is its own player's frame, nearer it than the other's.
    expect(rmse(left, s90)).toBeLessThan(rmse(left, s105));
    expect(rmse(right, s105)).toBeLessThan(rmse(right, s90));
    expect(rmse(left, s90)).toBeLessThan(0.15);
    expect(rmse(right, s105)).toBeLessThan(0.15);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);
