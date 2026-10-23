// The style-pack gallery: every screen of the kit in one style, composed into
// one reference sheet per style and surface, in FRAME_SCREENS order without
// the gallery: on a desktop crates/og-ui/references/<tier>/kit-style-<name>.png
// (7 × 3 cells, each the 1280×720 frame box-filtered to 640×360), on a phone
// in portrait kit-style-<name>-phone.png (10 × 2 cells, each the 390×844@3
// frame box-filtered to its 390×844 CSS pixels). Each screen is compared with
// its cell of the sheet under og-tests' rule (frames.ts).
// `OG_UI_CAPTURE=1 og test .` records the sheets instead.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { deflateSync } from "node:zlib";
import { playtest, type PlaytestDesc } from "@origingame/engine/playtest";
import { decodePng, difference, outDir, referencesDir, type Image } from "./frames.ts";
import { framesScript } from "./script.ts";

/** How a surface's frames become a sheet: the frame size, the box filter, the columns. */
interface Surface {
  readonly size: string;
  readonly viewport: string;
  readonly factor: number;
  readonly cols: number;
  readonly suffix: string;
}

const SURFACES: Record<"desktop" | "phone", Surface> = {
  desktop: { size: "1280x720", viewport: "", factor: 2, cols: 7, suffix: "" },
  phone: { size: "1170x2532", viewport: "0.0:viewport:phone;", factor: 3, cols: 10, suffix: "-phone" },
};

/** An n× box filter (1280×720 by 2 to 640×360; 1170×2532 by 3 to 390×844). */
export function shrink(img: Image, n: number): Image {
  const w = Math.floor(img.width / n);
  const h = Math.floor(img.height / n);
  const out = new Uint8Array(w * h * 4);
  const area = n * n;
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      for (let k = 0; k < 4; k++) {
        let sum = 0;
        for (let dy = 0; dy < n; dy++) for (let dx = 0; dx < n; dx++) sum += img.rgba[((n * y + dy) * img.width + n * x + dx) * 4 + k]!;
        out[(y * w + x) * 4 + k] = Math.floor((sum + (area >> 1)) / area);
      }
    }
  }
  return { width: w, height: h, rgba: out };
}

/** Cell `i` of a sheet with `cols` columns of `cw`×`ch` cells. */
export function cell(sheet: Image, i: number, cols: number, cw: number, ch: number): Image {
  const x0 = (i % cols) * cw;
  const y0 = Math.floor(i / cols) * ch;
  const out = new Uint8Array(cw * ch * 4);
  for (let y = 0; y < ch; y++) {
    out.set(sheet.rgba.subarray(((y0 + y) * sheet.width + x0) * 4, ((y0 + y) * sheet.width + x0 + cw) * 4), y * cw * 4);
  }
  return { width: cw, height: ch, rgba: out };
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

/** The playtest of one style pack on a surface (desktop by default). */
export function stylePlaytest(style: string, on: "desktop" | "phone" = "desktop"): PlaytestDesc {
  const surface = SURFACES[on];
  const script = framesScript(0.5);
  const files = script.files.filter((f) => f !== "gallery.png");
  const name = `kit-style-${style}${surface.suffix}.png`;
  return playtest({
    name: `every screen in the ${style} style pack${on === "phone" ? " on a phone" : ""} matches its cell of ${name}`,
    size: surface.size,
    commands: `${surface.viewport}0.2:style:${style};${script.commands}`,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      // The sheet is built from frames: without a renderer each is reported unchecked.
      t.frame(files, () => {
        const cells = files.map((f) => shrink(decodePng(readFileSync(join(outDir(), f))), surface.factor));
        const cw = cells[0]!.width;
        const ch = cells[0]!.height;
        const cols = surface.cols;
        const sheet: Image = { width: cols * cw, height: Math.ceil(cells.length / cols) * ch, rgba: new Uint8Array(0) };
        sheet.rgba = new Uint8Array(sheet.width * sheet.height * 4);
        cells.forEach((c, i) => {
          const x0 = (i % cols) * cw;
          const y0 = Math.floor(i / cols) * ch;
          for (let y = 0; y < ch; y++) sheet.rgba.set(c.rgba.subarray(y * cw * 4, (y + 1) * cw * 4), ((y0 + y) * sheet.width + x0) * 4);
        });
        writeFileSync(join(outDir(), name), encodePng(sheet));
        const refs = referencesDir();
        const own = join(refs, run.tier);
        if (process.env.OG_UI_CAPTURE !== undefined) {
          mkdirSync(own, { recursive: true });
          writeFileSync(join(own, name), encodePng(sheet));
          t.ok(true, `recorded ${join(own, name)}`);
          return;
        }
        const dir = existsSync(join(own, name)) ? own : join(refs, "metal");
        if (dir !== own) process.stdout.write(`  note: no ${run.tier} reference for ${style}; comparing with metal\n`);
        const refPath = join(dir, name);
        if (!existsSync(refPath)) {
          t.ok(false, `${refPath} is missing: record it with OG_UI_CAPTURE=1 og test templates/ui-kit`);
          return;
        }
        const ref = decodePng(readFileSync(refPath));
        t.equal(`${ref.width}x${ref.height}`, `${sheet.width}x${sheet.height}`, "the sheet has every screen");
        files.forEach((f, i) => {
          const d = difference(cells[i]!, cell(ref, i, cols, cw, ch));
          t.ok(d.outliers <= 0.005 && d.rmse <= 0.01, `${style} ${f} matches its cell: ${(d.outliers * 100).toFixed(3)} % outliers (≤ 0.5 %), RMSE ${d.rmse.toFixed(5)} (≤ 0.01)`);
        });
      });
    },
  });
}
