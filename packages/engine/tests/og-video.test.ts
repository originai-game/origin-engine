import { gpuUnavailable } from "./support/gpu.ts";
// Footage (docs/design.md §2.9): a `type video` cooked by ffmpeg plays on
// an og.Sprite through og.VideoPlayer (tests/fixtures/video: testsrc2 with
// its frame count burned in, every frame different, 30 fps, at a 30 Hz
// tick). The frame at tick 90 is the source's frame 3 s in: of the
// source's frames 89, 90 and 91, decoded by ffmpeg straight from the .mp4,
// 90 is the one the rendered frame is nearest. A seek stops the sound for a
// tick and starts it again at the new time.
//
// The reference frame is references/<tier>/video-90.png beside the fixture,
// recorded on the GPU host (vulkan); a tier without its own is held to that
// one under the cross-platform RMSE (support/png.ts `toleranceFor`).
//
// Two players of the video (the fixture's `pair`, the second half a second ahead)
// each show their own frame: their textures differ and each half of the
// frame is nearest its own player's source frame. Broken before: they
// shared the video's texture, which showed the first player's frame.
//
// The frame is a Texture like any other (the fixture's `surfaces`): the
// player's texture as the map of an unlit og.Mesh plane on the left half
// and on an og.ui image on the right half each show the source's frame 90,
// neither black (never rendered before, the video stream's "Left out").
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
import { RECORDING, toleranceFor } from "./support/png.ts";

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

test.skipIf(!hasFfmpeg || gpuUnavailable)("a cooked video plays on a sprite: tick 90 shows the source's frame 3 s in, and its sound follows a seek", () => {
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

    if (run.tier === "none") throw new Error("missing GPU adapter: video frame comparison requires a renderer");
    const frame = rgb(["-i", join(out, "video-90.png")], "");
    const source = join(fixture, "assets/clips/count.mp4");
    const errors = [89, 90, 91].map((n) => rmse(frame, rgb(["-i", source], `select=eq(n\\,${n}),`)));
    expect(errors[1]!).toBeLessThan(errors[0]!);
    expect(errors[1]!).toBeLessThan(errors[2]!);
    expect(errors[1]!).toBeLessThan(0.12);

    const own = join(fixture, "references", run.tier, "video-90.png");
    const ref = existsSync(own) ? own : join(fixture, "references", RECORDING, "video-90.png");
    expect(rmse(frame, rgb(["-i", ref], ""))).toBeLessThan(toleranceFor(run.tier, ref).rmse);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);

test.skipIf(!hasFfmpeg || gpuUnavailable)("two players of one video at different times show different frames", () => {
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
    if (run.tier === "none") throw new Error("missing GPU adapter: video frame comparison requires a renderer");
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

test.skipIf(!hasFfmpeg || gpuUnavailable)("a video frame shows as an og.Mesh material map and on an og.ui image", () => {
  const out = mkdtempSync(join(tmpdir(), "og-video-surfaces-"));
  try {
    const script = "0.5:surfaces;3:screenshot:surfaces.png;3:dump_entity:screen:a.json;3.1:quit";
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    expect((JSON.parse(readFileSync(join(out, "a.json"), "utf8")) as Dump).components["og.VideoPlayer"]!.frame).toBe(90);
    if (run.tier === "none") throw new Error("missing GPU adapter: video frame comparison requires a renderer");
    // The plane fills the left half of the middle band, the UI image the right half.
    const shot = join(out, "surfaces.png");
    const half = (x: string) => rgb(["-i", shot], `crop=iw/2:ih/2:${x}:ih/4,`);
    const source = join(fixture, "assets/clips/count.mp4");
    const s90 = rgb(["-i", source], "select=eq(n\\,90),");
    const s30 = rgb(["-i", source], "select=eq(n\\,30),");
    const mean = (a: Uint8Array) => a.reduce((s, v) => s + v, 0) / a.length;
    for (const [name, region] of [["mesh", half("0")], ["ui image", half("iw/2")]] as const) {
      expect(mean(region), name).toBeGreaterThan(60);
      expect(rmse(region, s90), name).toBeLessThan(rmse(region, s30));
      expect(rmse(region, s90), name).toBeLessThan(0.15);
    }
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);

// A stream (count-stream.ogt: the same source cooked to AV1 with a key
// frame every second, decoded by rav1d on a thread of its own as it plays)
// shows the frame its player is at, exactly, in every capture: tick 90 the
// source's frame 90; a seek back to 1 s the frame 31 on the next tick
// (decoded from the key frame at 30, not from 90 onwards); a loop's wrap
// the frame it wrapped to. Each is nearer its own source frame than either
// neighbour, so a frame decoded a step late or from the wrong key frame
// fails. Broken before: there were no streams (the cook refused `stream`).
test.skipIf(!hasFfmpeg || gpuUnavailable)("a streamed video shows its tick's frame, after a seek and across a loop", () => {
  const out = mkdtempSync(join(tmpdir(), "og-video-stream-"));
  try {
    const script = [
      "0.5:stream",
      "3:screenshot:s90.png",
      "3:dump_entity:screen:s90.json",
      "3:set:screen.og.VideoPlayer.time=1",
      "3.034:screenshot:seek.png",
      "3.034:dump_entity:screen:seek.json",
      "3.1:set:screen.og.VideoPlayer.loop=true",
      "3.1:set:screen.og.VideoPlayer.time=3.9",
      "3.267:screenshot:loop.png",
      "3.267:dump_entity:screen:loop.json",
      "3.3:quit",
    ].join(";");
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    const dump = (f: string) => (JSON.parse(readFileSync(join(out, f), "utf8")) as Dump).components;
    const at = dump("s90.json");
    expect(at["og.VideoPlayer"]!.frame).toBe(90);
    // 90 ticks are 3 s to double precision (summed in f32 they were 2.9999998).
    expect(Math.abs((at["og.VideoPlayer"]!.time as number) - 3)).toBeLessThan(1e-9);
    expect(at["og.VideoPlayer"]!.texture).toBeGreaterThan(0);
    expect(at["og.AudioSource"]!.sound).toBeGreaterThan(0);
    expect(dump("seek.json")["og.VideoPlayer"]!.frame).toBe(31);
    const looped = dump("loop.json")["og.VideoPlayer"]!.frame as number;
    expect(looped).toBeLessThan(10);
    if (run.tier === "none") throw new Error("missing GPU adapter: video frame comparison requires a renderer");
    const source = join(fixture, "assets/clips/count.mp4");
    const src = (n: number) => rgb(["-i", source], `select=eq(n\\,${n}),`);
    for (const [shot, n] of [["s90.png", 90], ["seek.png", 31], ["loop.png", looped]] as const) {
      const frame = rgb(["-i", join(out, shot)], "");
      const [before, own, after] = [n - 1, n, n + 1].map((k) => rmse(frame, src((k + 120) % 120)));
      expect(own, shot).toBeLessThan(before!);
      expect(own, shot).toBeLessThan(after!);
      expect(own, shot).toBeLessThan(0.12);
    }
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);

// A stream's sound stays with its picture (G3-2 batch 3): the player's time
// and og.AudioPlayback's cursor (the sound's playhead at the start of the
// next tick) agree to a millisecond after a pause and a resume, and after
// the loop wraps, with a second player of the same stream half a second
// ahead showing its own frame from a decoder of its own. Paused, the frame
// holds and the sound stops.
//
// Ways this could fail: a resume starts the sound at 0 or where it paused
// minus a tick; the sound loops at its own length rather than the
// picture's; the time is summed in f32 (an hour at 60 Hz falls 2.8 s
// behind the sound); two players of one stream share one decoder (the
// second shows the first's frame, or each waits on frames the other
// skipped).
test.skipIf(!hasFfmpeg || gpuUnavailable)("a streamed video's sound stays with its picture through a pause, a loop and a second player", () => {
  const out = mkdtempSync(join(tmpdir(), "og-video-sync-"));
  try {
    const script = [
      "0.5:stream",
      "0.5:set:screen.og.VideoPlayer.loop=true",
      "1:set:screen.og.VideoPlayer.playing=false",
      "1.1:dump_entity:screen:p1.json",
      "1.9:dump_entity:screen:p2.json",
      "2:set:screen.og.VideoPlayer.playing=true",
      "2.2:dump_entity:screen:resumed.json",
      "5.5:dump_entity:screen:looped.json",
      "5.5:pair",
      "6:screenshot:pair.png",
      "6:dump_entity:screen:a.json",
      "6:dump_entity:screen2:b.json",
      "6.1:quit",
    ].join(";");
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    const dump = (f: string) => (JSON.parse(readFileSync(join(out, f), "utf8")) as Dump).components;
    // The sound's playhead, in seconds, against the picture's time.
    const apart = (c: Dump["components"]) => Math.abs((c["og.AudioPlayback"]!.cursor as number) / 48000 - (c["og.VideoPlayer"]!.time as number));

    const [p1, p2] = [dump("p1.json"), dump("p2.json")];
    expect(p2["og.VideoPlayer"]!.frame).toBe(p1["og.VideoPlayer"]!.frame);
    expect(p2["og.AudioSource"]!.playing).toBe(false);
    expect(p2["og.AudioPlayback"] === undefined).toBe(true);

    const resumed = dump("resumed.json");
    expect(resumed["og.AudioSource"]!.playing).toBe(true);
    expect(apart(resumed)).toBeLessThan(1e-3);

    // 1 s played before the pause, 3.5 s since: wrapped at 4 s to 0.5 s.
    const looped = dump("looped.json");
    expect(looped["og.VideoPlayer"]!.time as number).toBeLessThan(1);
    expect(apart(looped)).toBeLessThan(1e-3);

    const [a, b] = [dump("a.json")["og.VideoPlayer"]!, dump("b.json")["og.VideoPlayer"]!];
    expect(((b.frame as number) - (a.frame as number) + 120) % 120).toBe(15);
    expect(b.texture).not.toBe(a.texture);
    if (run.tier === "none") throw new Error("missing GPU adapter: video frame comparison requires a renderer");
    const shot = join(out, "pair.png");
    const half = (x: string) => rgb(["-i", shot], `crop=iw/2:ih/2:${x}:ih/4,`);
    const src = (n: number) => rgb(["-i", join(fixture, "assets/clips/count.mp4")], `select=eq(n\\,${n % 120}),`);
    for (const [region, n] of [[half("0"), a.frame as number], [half("iw/2"), b.frame as number]] as const) {
      const [before, own, after] = [n + 119, n, n + 1].map((k) => rmse(region, src(k)));
      expect(own).toBeLessThan(before!);
      expect(own).toBeLessThan(after!);
      expect(own).toBeLessThan(0.15);
    }
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}, 300_000);
