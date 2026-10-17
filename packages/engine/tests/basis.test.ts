// Web KTX2 transcoding (src/web/basis.ts) on the cook's real output: the
// template's stone textures are cooked by og-cook (ETC1S colour, UASTC normal
// map), the KTX2 is taken out of each .ogb, transcoded with the vendored
// Binomial transcoder, and compared with the source PNG. Block formats are
// checked for their exact size per level.
import { expect, test } from "bun:test";
import { createRequire } from "node:module";
import { existsSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  compressionsFromWebGl2,
  compressionsFromWebGpu,
  loadBasisTranscoder,
  pickFormat,
  transcodeKtx2,
  VENDORED,
  wgpuFormatName,
  type BasisTranscoder,
} from "../src/web/basis.ts";

const require = createRequire(import.meta.url);
const { cpSync, mkdirSync, mkdtempSync } = require("node:fs") as {
  cpSync(from: string, to: string): void;
  mkdirSync(path: string, options?: { recursive?: boolean }): void;
  mkdtempSync(prefix: string): string;
};
const { tmpdir } = require("node:os") as { tmpdir(): string };
const { inflateSync } = require("node:zlib") as { inflateSync(data: Uint8Array): Uint8Array };

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const textures = join(repo, "templates", "game", "assets", "textures");

// ── a .ogb section reader (crates/og-blob/README.md §Layout) ──────────────

function section(blob: Uint8Array, name: string): Uint8Array {
  const v = new DataView(blob.buffer, blob.byteOffset, blob.byteLength);
  const magic = [0x89, 0x4f, 0x47, 0x42, 0x0d, 0x0a, 0x1a, 0x0a];
  if (!magic.every((b, i) => blob[i] === b)) throw new Error("not an .ogb blob");
  const count = v.getUint32(12, true);
  const pool = 64 + 48 * count;
  for (let i = 0; i < count; i++) {
    const at = 64 + 48 * i;
    const nameOff = v.getUint32(at, true);
    const nameLen = v.getUint32(at + 4, true);
    const n = new TextDecoder().decode(blob.subarray(pool + nameOff, pool + nameOff + nameLen));
    if (n !== name) continue;
    const off = Number(v.getBigUint64(at + 8, true));
    const size = Number(v.getBigUint64(at + 16, true));
    return blob.subarray(off, off + size);
  }
  throw new Error(`no section '${name}'`);
}

// ── a PNG decoder for 8-bit RGB/RGBA, non-interlaced (the template's) ─────

function decodePng(file: Uint8Array): { width: number; height: number; rgba: Uint8Array } {
  const v = new DataView(file.buffer, file.byteOffset, file.byteLength);
  let at = 8;
  let width = 0, height = 0, channels = 0;
  const idat: Uint8Array[] = [];
  while (at < file.length) {
    const len = v.getUint32(at);
    const type = new TextDecoder().decode(file.subarray(at + 4, at + 8));
    const data = file.subarray(at + 8, at + 8 + len);
    if (type === "IHDR") {
      width = v.getUint32(at + 8);
      height = v.getUint32(at + 12);
      const depth = data[8]!, colour = data[9]!, interlace = data[12]!;
      if (depth !== 8 || interlace !== 0 || (colour !== 2 && colour !== 6)) throw new Error("the test decoder reads 8-bit RGB/RGBA non-interlaced PNGs only");
      channels = colour === 2 ? 3 : 4;
    } else if (type === "IDAT") idat.push(data);
    at += 12 + len;
  }
  const joined = new Uint8Array(idat.reduce((n, d) => n + d.length, 0));
  idat.reduce((at, d) => (joined.set(d, at), at + d.length), 0);
  const raw = inflateSync(joined);
  const stride = width * channels;
  const out = new Uint8Array(width * height * 4);
  let prev = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]!;
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    const cur = new Uint8Array(stride);
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? cur[x - channels]! : 0, b = prev[x]!, c = x >= channels ? prev[x - channels]! : 0;
      const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      const pred = [0, a, b, (a + b) >> 1, pa <= pb && pa <= pc ? a : pb <= pc ? b : c][filter]!;
      cur[x] = (line[x]! + pred) & 255;
    }
    for (let x = 0; x < width; x++) {
      for (let k = 0; k < 3; k++) out[(y * width + x) * 4 + k] = cur[x * channels + k]!;
      out[(y * width + x) * 4 + 3] = channels === 4 ? cur[x * 4 + 3]! : 255;
    }
    prev = cur;
  }
  return { width, height, rgba: out };
}

function psnrRgb(a: Uint8Array, b: Uint8Array): number {
  let sum = 0;
  for (let i = 0; i < a.length; i += 4) for (let k = 0; k < 3; k++) sum += (a[i + k]! - b[i + k]!) ** 2;
  const mse = sum / ((a.length / 4) * 3);
  return 10 * Math.log10((255 * 255) / mse);
}

function flipRows(rgba: Uint8Array, width: number, height: number): Uint8Array {
  const out = new Uint8Array(rgba.length);
  for (let y = 0; y < height; y++) out.set(rgba.subarray(y * width * 4, (y + 1) * width * 4), (height - 1 - y) * width * 4);
  return out;
}

// ── fixtures: cook the two textures once (top-level, so no hook timeout) ──

const ktx2: Record<string, Uint8Array> = {};

const basis: BasisTranscoder = await (async () => {
  for (const url of [VENDORED.js, VENDORED.wasm]) {
    if (!existsSync(fileURLToPath(url))) throw new Error(`the vendored Basis transcoder is missing: ${fileURLToPath(url)} (packages/engine/vendor/basis/README.md)`);
  }
  const transcoder = await loadBasisTranscoder();
  const project = mkdtempSync(join(tmpdir(), "og-basis-"));
  try {
    mkdirSync(join(project, "assets", "textures"), { recursive: true });
    for (const f of ["stone.png", "stone_n.png", "stone_n.png.ogt"]) cpSync(join(textures, f), join(project, "assets", "textures", f));
    const cmd = process.env.OG_COOK ? [process.env.OG_COOK, project] : ["cargo", "run", "-q", "-p", "og-cook", "--bin", "og-cook", "--", project];
    const r = Bun.spawnSync(cmd, { cwd: repo, env: process.env });
    if (r.exitCode !== 0) throw new Error(`og-cook failed: ${new TextDecoder().decode(r.stderr)}${new TextDecoder().decode(r.stdout)}`);
    for (const f of ["stone.png", "stone_n.png"]) ktx2[f] = section(new Uint8Array(readFileSync(join(project, "cooked", "textures", `${f}.ogb`))), "ktx2").slice();
  } finally {
    rmSync(project, { recursive: true, force: true });
  }
  return transcoder;
})();

test("an ETC1S colour texture transcodes to RGBA8 close to its source", () => {
  const src = decodePng(new Uint8Array(readFileSync(join(textures, "stone.png"))));
  const t = transcodeKtx2(basis, ktx2["stone.png"]!, "rgba8");
  expect([t.width, t.height, t.format, t.kind, t.srgb]).toEqual([src.width, src.height, "rgba8", "etc1s", true]);
  expect(t.levels[0]!.length).toBe(src.width * src.height * 4);
  const psnr = psnrRgb(t.levels[0]!, src.rgba);
  expect(psnr).toBeGreaterThan(30);
  // Orientation: rows come back top to bottom, as the PNG has them.
  expect(psnrRgb(flipRows(t.levels[0]!, t.width, t.height), src.rgba)).toBeLessThan(psnr - 3);
  // The full mip chain, each level half the last.
  expect(t.sizes.length).toBe(Math.log2(src.width) + 1);
  t.sizes.forEach((s, i) => expect(s).toEqual({ width: src.width >> i, height: src.height >> i }));
});

test("a UASTC normal map transcodes linear and close to its source", () => {
  const src = decodePng(new Uint8Array(readFileSync(join(textures, "stone_n.png"))));
  const t = transcodeKtx2(basis, ktx2["stone_n.png"]!, "rgba8");
  expect([t.kind, t.srgb]).toEqual(["uastc", false]);
  expect(psnrRgb(t.levels[0]!, src.rgba)).toBeGreaterThan(35);
});

test("block formats have exactly one 16-byte block per 4x4 pixels per level", () => {
  for (const file of ["stone.png", "stone_n.png"]) {
    for (const format of ["bc7", "astc4x4", "etc2"] as const) {
      const t = transcodeKtx2(basis, ktx2[file]!, format);
      expect(t.format).toBe(format);
      t.levels.forEach((level, i) => {
        const { width, height } = t.sizes[i]!;
        expect(level.length).toBe(Math.ceil(width / 4) * Math.ceil(height / 4) * 16);
      });
    }
  }
});

test("a file that is not Basis KTX2 is refused with the reason", () => {
  expect(() => transcodeKtx2(basis, new Uint8Array(64), "rgba8")).toThrow(/not a KTX2 file/);
});

test("the format follows the device's compressions; WebGPU names follow the format", () => {
  expect(pickFormat(new Set(["etc2", "astc", "bc"]))).toBe("bc7");
  expect(pickFormat(["astc", "etc2"], "uastc")).toBe("astc4x4");
  expect(pickFormat(["etc2"])).toBe("etc2");
  expect(pickFormat([])).toBe("rgba8");
  expect(compressionsFromWebGpu(new Set(["texture-compression-astc", "float32-filterable"]))).toEqual(new Set(["astc"]));
  expect(compressionsFromWebGl2(["EXT_texture_compression_bptc", "WEBGL_compressed_texture_etc"])).toEqual(new Set(["bc", "etc2"]));
  expect(wgpuFormatName("bc7", true)).toBe("bc7-rgba-unorm-srgb");
  expect(wgpuFormatName("astc4x4", false)).toBe("astc-4x4-unorm");
  expect(wgpuFormatName("etc2", true)).toBe("etc2-rgba8unorm-srgb");
  expect(wgpuFormatName("rgba8", false)).toBe("rgba8unorm");
});
