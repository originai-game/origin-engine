// The style-pack gallery: every screen of the kit in one skin, composed into
// one reference sheet per skin (crates/og-ui/references/<tier>/kit-skin-<name>.png:
// 7 × 3 screens, each the 1280×720 frame box-filtered to 640×360, in
// FRAME_SCREENS order without the gallery). Each screen is compared with its
// cell of the sheet under og-tests' rule (frames.ts). `OG_UI_CAPTURE=1 og test .`
// records the sheet instead.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { deflateSync } from "node:zlib";
import { playtest, type PlaytestDesc } from "@origingame/engine/playtest";
import { decodePng, difference, outDir, referencesDir, type Image } from "./frames.ts";
import { framesScript } from "./script.ts";

const COLS = 7;
const CELL_W = 640;
const CELL_H = 360;

/** A 2× box filter (1280×720 to 640×360). */
export function half(img: Image): Image {
  const w = img.width >> 1;
  const h = img.height >> 1;
  const out = new Uint8Array(w * h * 4);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      for (let k = 0; k < 4; k++) {
        const at = (yy: number, xx: number) => img.rgba[(yy * img.width + xx) * 4 + k]!;
        out[(y * w + x) * 4 + k] = (at(2 * y, 2 * x) + at(2 * y, 2 * x + 1) + at(2 * y + 1, 2 * x) + at(2 * y + 1, 2 * x + 1) + 2) >> 2;
      }
    }
  }
  return { width: w, height: h, rgba: out };
}

/** Cell `i` of a sheet. */
export function cell(sheet: Image, i: number): Image {
  const x0 = (i % COLS) * CELL_W;
  const y0 = Math.floor(i / COLS) * CELL_H;
  const out = new Uint8Array(CELL_W * CELL_H * 4);
  for (let y = 0; y < CELL_H; y++) {
    out.set(sheet.rgba.subarray(((y0 + y) * sheet.width + x0) * 4, ((y0 + y) * sheet.width + x0 + CELL_W) * 4), y * CELL_W * 4);
  }
  return { width: CELL_W, height: CELL_H, rgba: out };
}

function crc32(bytes: Uint8Array): number {
  let c = ~0;
  for (const b of bytes) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}

/** An 8-bit RGB PNG (filter 0). */
export function encodePng(img: Image): Uint8Array {
  const raw = new Uint8Array((img.width * 3 + 1) * img.height);
  for (let y = 0; y < img.height; y++) {
    const row = y * (img.width * 3 + 1);
    for (let x = 0; x < img.width; x++) {
      for (let k = 0; k < 3; k++) raw[row + 1 + x * 3 + k] = img.rgba[(y * img.width + x) * 4 + k]!;
    }
  }
  const chunk = (type: string, data: Uint8Array): Uint8Array => {
    const out = new Uint8Array(12 + data.length);
    const view = new DataView(out.buffer);
    view.setUint32(0, data.length);
    out.set(new TextEncoder().encode(type), 4);
    out.set(data, 8);
    view.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
    return out;
  };
  const ihdr = new Uint8Array(13);
  const v = new DataView(ihdr.buffer);
  v.setUint32(0, img.width);
  v.setUint32(4, img.height);
  ihdr.set([8, 2, 0, 0, 0], 8);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", new Uint8Array())];
  const total = parts.reduce((n, p) => n + p.length, 0);
  const png = new Uint8Array(total);
  let at = 0;
  for (const p of parts) {
    png.set(p, at);
    at += p.length;
  }
  return png;
}

/** The playtest of one style pack. */
export function skinPlaytest(skin: string): PlaytestDesc {
  const script = framesScript(0.5);
  const files = script.files.filter((f) => f !== "gallery.png");
  return playtest({
    name: `every screen in the ${skin} style pack matches its cell of kit-skin-${skin}.png`,
    size: "1280x720",
    commands: `0.2:skin:${skin};${script.commands}`,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      const cells = files.map((f) => half(decodePng(readFileSync(join(outDir(), f)))));
      const sheet: Image = { width: COLS * CELL_W, height: Math.ceil(cells.length / COLS) * CELL_H, rgba: new Uint8Array(0) };
      sheet.rgba = new Uint8Array(sheet.width * sheet.height * 4);
      cells.forEach((c, i) => {
        const x0 = (i % COLS) * CELL_W;
        const y0 = Math.floor(i / COLS) * CELL_H;
        for (let y = 0; y < CELL_H; y++) sheet.rgba.set(c.rgba.subarray(y * CELL_W * 4, (y + 1) * CELL_W * 4), ((y0 + y) * sheet.width + x0) * 4);
      });
      writeFileSync(join(outDir(), `kit-skin-${skin}.png`), encodePng(sheet));
      const refs = referencesDir();
      const own = join(refs, run.tier);
      const name = `kit-skin-${skin}.png`;
      if (process.env.OG_UI_CAPTURE !== undefined) {
        mkdirSync(own, { recursive: true });
        writeFileSync(join(own, name), encodePng(sheet));
        t.ok(true, `recorded ${join(own, name)}`);
        return;
      }
      const dir = existsSync(join(own, name)) ? own : join(refs, "metal");
      if (dir !== own) process.stdout.write(`  note: no ${run.tier} reference for ${skin}; comparing with metal\n`);
      const refPath = join(dir, name);
      if (!existsSync(refPath)) {
        t.ok(false, `${refPath} is missing: record it with OG_UI_CAPTURE=1 og test templates/ui-kit`);
        return;
      }
      const ref = decodePng(readFileSync(refPath));
      t.equal(`${ref.width}x${ref.height}`, `${sheet.width}x${sheet.height}`, "the sheet has every screen");
      files.forEach((f, i) => {
        const d = difference(cells[i]!, cell(ref, i));
        t.ok(d.outliers <= 0.005 && d.rmse <= 0.01, `${skin} ${f} matches its cell: ${(d.outliers * 100).toFixed(3)} % outliers (≤ 0.5 %), RMSE ${d.rmse.toFixed(5)} (≤ 0.01)`);
      });
    },
  });
}
