// The reference-frame check: decode the PNGs a run wrote and compare them
// with references/<tier>/ under og-tests' tolerance (a channel off by more
// than 32/255 is an outlier; fail above 0.5 % outliers or an RGB RMSE above
// 0.01). A tier without its own references compares with `metal` and says
// so. `OG_FX_CAPTURE=1 og test .` records this tier's references instead.
// (Copied from examples/lantern-vale/tests/frames.ts, as a game author would.)

import { copyFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { inflateSync } from "node:zlib";
import type { PlaytestChecks, PlaytestRun } from "@origingame/engine/playtest";

export interface Image {
  width: number;
  height: number;
  /** RGBA8 rows. */
  rgba: Uint8Array;
}

/** An 8-bit RGB or RGBA, non-interlaced PNG, unfiltered. */
export function decodePng(bytes: Uint8Array): Image {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 8;
  let width = 0;
  let height = 0;
  let channels = 4;
  const idat: Uint8Array[] = [];
  while (at < bytes.length) {
    const len = view.getUint32(at);
    const type = String.fromCharCode(...bytes.subarray(at + 4, at + 8));
    const data = bytes.subarray(at + 8, at + 8 + len);
    if (type === "IHDR") {
      width = view.getUint32(at + 8);
      height = view.getUint32(at + 12);
      const depth = data[8]!;
      const color = data[9]!;
      if (depth !== 8 || (color !== 6 && color !== 2) || data[12] !== 0) throw new Error(`unsupported PNG (depth ${depth}, colour ${color}, interlace ${data[12]})`);
      channels = color === 6 ? 4 : 3;
    } else if (type === "IDAT") idat.push(data);
    else if (type === "IEND") break;
    at += 12 + len;
  }
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const out = new Uint8Array(width * height * 4);
  const prev = new Uint8Array(stride);
  const cur = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]!;
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let i = 0; i < stride; i++) {
      const a = i >= channels ? cur[i - channels]! : 0;
      const b = prev[i]!;
      const c = i >= channels ? prev[i - channels]! : 0;
      let v = line[i]!;
      if (filter === 1) v += a;
      else if (filter === 2) v += b;
      else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      cur[i] = v & 255;
    }
    for (let x = 0; x < width; x++) {
      for (let k = 0; k < 4; k++) out[(y * width + x) * 4 + k] = k < channels ? cur[x * channels + k]! : 255;
    }
    prev.set(cur);
  }
  return { width, height, rgba: out };
}

/** og-tests' rule: outlier fraction and RGB RMSE between two equal-size frames. */
export function difference(a: Image, b: Image): { outliers: number; rmse: number } {
  if (a.width !== b.width || a.height !== b.height) return { outliers: 1, rmse: 1 };
  let outliers = 0;
  let sum = 0;
  const n = a.width * a.height;
  for (let p = 0; p < n; p++) {
    let out = false;
    for (let k = 0; k < 3; k++) {
      const d = Math.abs(a.rgba[p * 4 + k]! - b.rgba[p * 4 + k]!);
      if (d > 32) out = true;
      const f = d / 255;
      sum += f * f;
    }
    if (out) outliers++;
  }
  return { outliers: outliers / n, rmse: Math.sqrt(sum / (n * 3)) };
}

/** The playtest's output directory (the bundled playtest runs from it). */
export function outDir(): string {
  return dirname(fileURLToPath(import.meta.url));
}

/** The project's references/, from its og-out/test/<name>/. */
export function referencesDir(): string {
  return resolve(outDir(), "..", "..", "..", "references");
}

/** Compares each screenshot with its reference (`<prefix><file>`), or records it. */
export function checkFrames(run: PlaytestRun, t: PlaytestChecks, files: readonly string[], prefix: string): void {
  const refs = referencesDir();
  t.ok(existsSync(refs) || process.env.OG_FX_CAPTURE !== undefined, `the references directory exists at ${refs}`);
  const own = join(refs, run.tier);
  const capture = process.env.OG_FX_CAPTURE !== undefined;
  for (const f of files) {
    const actual = join(outDir(), f);
    // Each file on its own: this tier's reference where it has one, metal's otherwise.
    const mine = join(own, prefix + f);
    const ref = capture || existsSync(mine) ? mine : join(refs, "metal", prefix + f);
    if (ref !== mine) process.stdout.write(`  note: no ${run.tier} reference for ${prefix}${f}; comparing with metal\n`);
    if (capture) {
      mkdirSync(own, { recursive: true });
      copyFileSync(actual, ref);
      t.ok(true, `recorded ${ref}`);
      continue;
    }
    if (!existsSync(ref)) {
      t.ok(false, `${ref} is missing: record it with OG_FX_CAPTURE=1 og test .`);
      continue;
    }
    const d = difference(decodePng(readFileSync(actual)), decodePng(readFileSync(ref)));
    process.stdout.write(`  ${f}: ${(d.outliers * 100).toFixed(3)} % outliers, RMSE ${d.rmse.toFixed(5)} against ${ref}\n`);
    t.ok(d.outliers <= 0.005 && d.rmse <= 0.01, `${f} matches ${prefix}${f}: ${(d.outliers * 100).toFixed(3)} % outliers (≤ 0.5 %), RMSE ${d.rmse.toFixed(5)} (≤ 0.01)`);
  }
}
