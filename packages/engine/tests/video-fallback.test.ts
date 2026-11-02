// The web's AV1 fallback (G3-2, src/web/video.ts): rav1d built for
// wasm32-wasip1 (dist/av1/og-av1.wasm, scripts/build-av1.sh) decodes what
// the cook's ffmpeg encodes, here in bun through the same WASI shim the
// decoder Worker uses; and the decode plan is og-video's `stream::plan`.
//
// Ways this could fail: a WASI import missing from the shim (the module
// refuses to instantiate); planes copied at the wrong length (U and V of a
// size not a multiple of two rounded down); the plan seeking to the wanted
// frame instead of its key frame, or never wrapping a loop.
import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { av1Module, planStep } from "../src/web/video.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const module = join(pkg, "dist/av1/og-av1.wasm");
const fixture = join(pkg, "tests/fixtures/video/assets/clips/count.mp4");
const ffmpeg = process.env.OG_FFMPEG ?? "ffmpeg";
/** Whether two byte arrays hold the same bytes. */
const same = (a: Uint8Array, b: Uint8Array) => a.length === b.length && a.every((v, i) => v === b[i]);
const encoder = (() => {
  try {
    const r = Bun.spawnSync([ffmpeg, "-hide_banner", "-encoders"]);
    const list = new TextDecoder().decode(r.stdout);
    return ["libsvtav1", "libaom-av1"].find((e) => list.includes(e)) ?? null;
  } catch {
    return null;
  }
})();

test("the plan decodes on, seeks to key frames and wraps a loop", () => {
  const keys = [0, 30, 60, 90];
  const has = (set: number[]) => (f: number) => set.includes(f);
  expect(planStep(100, keys, 0, has([]), 0, 0, false, 4)).toBe("decode");
  expect(planStep(100, keys, 50, has([]), 0, 40, false, 4)).toBe(30);
  expect(planStep(100, keys, 12, has([]), 0, 75, false, 4)).toBe(60);
  expect(planStep(100, keys, 12, has([]), 0, 20, false, 4)).toBe("decode");
  expect(planStep(100, keys, 7, has([3, 4, 5, 6]), 4, 3, false, 4)).toBe("idle");
  expect(planStep(100, keys, 100, has([99]), 1, 99, false, 4)).toBe("idle");
  expect(planStep(100, keys, 100, has([99]), 1, 99, true, 4)).toBe(0);
  expect(planStep(100, keys, 100, has([99]), 1, 500, false, 4)).toBe("idle");
});

test.skipIf(!encoder || !existsSync(module))("rav1d's wasm module decodes the cook's AV1 as ffmpeg does", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-av1-"));
  try {
    const ivf = join(dir, "c.ivf");
    // An odd chroma size: 130x74 has 65x37 chroma planes.
    const enc = Bun.spawnSync([ffmpeg, "-v", "error", "-y", "-i", fixture, "-t", "1", "-an", "-vf", "fps=30,scale=130:74", "-pix_fmt", "yuv420p", "-c:v", encoder!, "-g", "15", "-f", "ivf", ivf]);
    expect(enc.exitCode).toBe(0);
    const bytes = new Uint8Array(readFileSync(ivf));
    const dv = new DataView(bytes.buffer, bytes.byteOffset);
    const units: Uint8Array[] = [];
    for (let at = dv.getUint16(6, true); at + 12 <= bytes.length; ) {
      const size = dv.getUint32(at, true);
      units.push(bytes.subarray(at + 12, at + 12 + size));
      at += 12 + size;
    }
    expect(units.length).toBe(30);
    const theirs = Bun.spawnSync([ffmpeg, "-v", "error", "-i", ivf, "-f", "rawvideo", "-pix_fmt", "yuv420p", "-"]).stdout;
    const per = 130 * 74 + 2 * 65 * 37;
    expect(theirs.length).toBe(per * 30);
    const m = await av1Module(new Uint8Array(readFileSync(module)).buffer);
    const h = m.open();
    expect(h).toBeGreaterThan(0);
    for (let n = 0; n < 30; n++) {
      const p = m.decode(h, units[n]!)!;
      expect([p.width, p.height, p.i420.length]).toEqual([130, 74, per]);
      // The same decoder bit for bit: AV1 decoding is exact.
      expect(same(p.i420, theirs.subarray(n * per, (n + 1) * per))).toBe(true);
    }
    // After a flush, decoding from the key frame at 15 gives frame 17 again.
    m.flush(h);
    let last = null;
    for (let n = 15; n <= 17; n++) last = m.decode(h, units[n]!);
    expect(same(last!.i420, theirs.subarray(17 * per, 18 * per))).toBe(true);
    m.close(h);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
