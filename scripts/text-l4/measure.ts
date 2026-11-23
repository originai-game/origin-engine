// og's text sheets against Chrome's, row by row (a port of the text thread's measure.py).
// usage: bun measure.ts <og-dir> <chrome-dir> <out-dir> [packs, default default]
// <og-dir> holds og-<pack>.png (run.sh's sheets), <chrome-dir> chrome-<pack>.png (chrome-sheets.ts).
// Writes <out-dir>/cmp-<pack>.png (og over Chrome, then 4x crops of the 12–16 px Latin and Chinese
// rows), rows.json, and prints ink ratio, width, baseline and top per row.
// Exits 1 on a finding: a row that draws nothing in og, an ink ratio outside
// INK_MIN..INK_MAX (default 0.85..1.15), a width off by more than WIDTH_PCT % (default 4)
// from Chrome's, or a baseline off by more than max(2, size / 12) px. Each finding is one line on stderr starting `measure:`.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { decodePng, encodePng, type Image } from "../../packages/engine/src/node/png.ts";

const [ogDir, chDir, outArg, packArg] = process.argv.slice(2);
if (!ogDir || !chDir || !outArg) {
  console.error("usage: bun measure.ts <og-dir> <chrome-dir> <out-dir> [packs]");
  process.exit(2);
}
const out = resolve(outArg);
mkdirSync(out, { recursive: true });
const PACKS = (packArg ?? "default").split(",");
const SIZES = [12, 14, 16, 20, 28, 48];
const LINES = ["latin", "cjk", "digits"];
const BG = [0x14, 0x1a, 0x22];
const FG = [0xe8, 0xec, 0xf2];
const INK_MIN = Number(process.env.INK_MIN ?? 0.85);
const INK_MAX = Number(process.env.INK_MAX ?? 1.15);
const WIDTH_PCT = Number(process.env.WIDTH_PCT ?? 4);

function* rows(): Generator<[string, number, number, number]> {
  let y = 8;
  for (const l of LINES)
    for (const s of SIZES) {
      const h = Math.round(s * 1.3);
      yield [l, s, y, h];
      y += h;
    }
}

/** Per pixel, how far from the ground towards the ink, 0..1. */
function coverage(img: Image): Float32Array {
  const c = new Float32Array(img.width * img.height);
  for (let i = 0; i < c.length; i++) {
    let v = 0;
    for (let k = 0; k < 3; k++) v += (img.rgba[i * 4 + k]! - BG[k]!) / (FG[k]! - BG[k]!);
    c[i] = Math.min(1, Math.max(0, v / 3));
  }
  return c;
}

interface Stats { ink: number; width: number; top: number; bottom: number; base: number; peak: number }

function stats(cov: Float32Array, w: number, y: number, h: number): Stats | null {
  const x0 = 40;
  let ink = 0;
  const colMax = new Float32Array(w - x0);
  const rowMax = new Float32Array(h);
  const prof = new Float32Array(h);
  const on: number[] = [];
  for (let r = 0; r < h; r++)
    for (let x = x0; x < w; x++) {
      const v = cov[(y + r) * w + x]!;
      ink += v;
      prof[r]! += v;
      if (v > colMax[x - x0]!) colMax[x - x0] = v;
      if (v > rowMax[r]!) rowMax[r] = v;
      if (v > 0.05) on.push(v);
    }
  const cols = [...colMax.keys()].filter((i) => colMax[i]! > 0.25);
  if (!cols.length) return null;
  const rws = [...rowMax.keys()].filter((i) => rowMax[i]! > 0.5);
  const pmax = Math.max(...prof);
  const base = Math.max(...[...prof.keys()].filter((i) => prof[i]! > 0.15 * pmax));
  on.sort((a, b) => a - b);
  return { ink, width: cols.at(-1)! - cols[0]! + 1, top: rws[0] ?? 0, bottom: rws.at(-1) ?? 0, base, peak: on[Math.floor(on.length * 0.95)] ?? 0 };
}

/** `img`'s rows y..y+h, x 0..600, scaled 4x by nearest neighbour, at width 1200 (2x). */
function crop2(img: Image, y: number, h: number): Image {
  const W = 1200, H = h * 2;
  const rgba = new Uint8Array(W * H * 4);
  for (let r = 0; r < H; r++)
    for (let x = 0; x < W; x++) {
      const s = ((y + Math.floor(r / 2)) * img.width + Math.floor(x / 2)) * 4;
      rgba.set(img.rgba.subarray(s, s + 4), (r * W + x) * 4);
    }
  return { width: W, height: H, rgba };
}

function stack(parts: Image[], gap: number): Image {
  const W = 1200;
  const H = parts.reduce((a, p) => a + p.height + gap, 0);
  const rgba = new Uint8Array(W * H * 4);
  for (let i = 3; i < rgba.length; i += 4) rgba[i] = 255;
  let y = 0;
  for (const p of parts) {
    for (let r = 0; r < p.height; r++) rgba.set(p.rgba.subarray(r * p.width * 4, r * p.width * 4 + Math.min(p.width, W) * 4), ((y + r) * W) * 4);
    y += p.height + gap;
  }
  return { width: W, height: H, rgba };
}

const top540 = (img: Image): Image => ({ width: img.width, height: 540, rgba: img.rgba.subarray(0, img.width * 540 * 4) });

const report: Record<string, unknown[]> = {};
const findings: string[] = [];
console.log(`${"pack".padEnd(8)} ${"line".padEnd(6)}  px |  ink og/cr  width og-cr  base og-cr  top og-cr  peak og/cr`);
for (const p of PACKS) {
  const og = decodePng(readFileSync(join(ogDir, `og-${p}.png`)));
  const ch = decodePng(readFileSync(join(chDir, `chrome-${p}.png`)));
  const co = coverage(og), cc = coverage(ch);
  report[p] = [];
  const crops: Image[] = [];
  for (const [l, s, y, h] of rows()) {
    const a = stats(co, og.width, y, h), b = stats(cc, ch.width, y, h);
    report[p]!.push({ line: l, size: s, og: a, chrome: b });
    const where = `${p} ${l} ${s}px`;
    if (!a) findings.push(`measure: ${where}: og draws nothing in the row`);
    if (!b) findings.push(`measure: ${where}: Chrome draws nothing in the row (reference broken)`);
    if (a && b) {
      const ratio = a.ink / b.ink;
      console.log(`${p.padEnd(8)} ${l.padEnd(6)} ${String(s).padStart(3)} | ${ratio.toFixed(2).padStart(9)} ${String(a.width - b.width).padStart(11)} ${String(a.base - b.base).padStart(10)} ${String(a.top - b.top).padStart(9)}  ${a.peak.toFixed(2)}/${b.peak.toFixed(2)}`);
      if (ratio < INK_MIN || ratio > INK_MAX) findings.push(`measure: ${where}: ink ${ratio.toFixed(2)} of Chrome's (outside ${INK_MIN}..${INK_MAX})`);
      if (Math.abs(a.base - b.base) > Math.max(2, s / 12)) findings.push(`measure: ${where}: baseline ${a.base - b.base} px from Chrome's`);
      if (Math.abs(a.width - b.width) > (b.width * WIDTH_PCT) / 100) findings.push(`measure: ${where}: width ${a.width} px against Chrome's ${b.width} px`);
    }
    if (s <= 16 && l !== "digits") crops.push(crop2(og, y, h), crop2(ch, y, h));
  }
  writeFileSync(join(out, `cmp-${p}.png`), encodePng(stack([top540(og), top540(ch), ...crops], 6)));
}
writeFileSync(join(out, "rows.json"), JSON.stringify(report, null, 1));
for (const f of findings) console.error(f);
process.exit(findings.length ? 1 : 0);
