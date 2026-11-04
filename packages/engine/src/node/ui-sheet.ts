// The contact sheet and filmstrip images of `og ui preview` (docs/ui.md
// §16.4): cells scaled to one height, laid out in a grid, each labelled in
// a 5x7 pixel font of its own (uppercase, digits and a little punctuation),
// so a sheet needs no font file and is the same bytes on every machine.
import type { Image } from "./png.ts";

const GLYPHS: Record<string, string> = {
  A: "01110 10001 10001 11111 10001 10001 10001", B: "11110 10001 10001 11110 10001 10001 11110",
  C: "01111 10000 10000 10000 10000 10000 01111", D: "11110 10001 10001 10001 10001 10001 11110",
  E: "11111 10000 10000 11110 10000 10000 11111", F: "11111 10000 10000 11110 10000 10000 10000",
  G: "01111 10000 10000 10011 10001 10001 01111", H: "10001 10001 10001 11111 10001 10001 10001",
  I: "01110 00100 00100 00100 00100 00100 01110", J: "00111 00010 00010 00010 00010 10010 01100",
  K: "10001 10010 10100 11000 10100 10010 10001", L: "10000 10000 10000 10000 10000 10000 11111",
  M: "10001 11011 10101 10101 10001 10001 10001", N: "10001 11001 10101 10011 10001 10001 10001",
  O: "01110 10001 10001 10001 10001 10001 01110", P: "11110 10001 10001 11110 10000 10000 10000",
  Q: "01110 10001 10001 10001 10101 10010 01101", R: "11110 10001 10001 11110 10100 10010 10001",
  S: "01111 10000 10000 01110 00001 00001 11110", T: "11111 00100 00100 00100 00100 00100 00100",
  U: "10001 10001 10001 10001 10001 10001 01110", V: "10001 10001 10001 10001 10001 01010 00100",
  W: "10001 10001 10001 10101 10101 10101 01010", X: "10001 10001 01010 00100 01010 10001 10001",
  Y: "10001 10001 01010 00100 00100 00100 00100", Z: "11111 00001 00010 00100 01000 10000 11111",
  "0": "01110 10001 10011 10101 11001 10001 01110", "1": "00100 01100 00100 00100 00100 00100 01110",
  "2": "01110 10001 00001 00010 00100 01000 11111", "3": "11110 00001 00001 01110 00001 00001 11110",
  "4": "00010 00110 01010 10010 11111 00010 00010", "5": "11111 10000 11110 00001 00001 10001 01110",
  "6": "00110 01000 10000 11110 10001 10001 01110", "7": "11111 00001 00010 00100 01000 01000 01000",
  "8": "01110 10001 10001 01110 10001 10001 01110", "9": "01110 10001 10001 01111 00001 00010 01100",
  "-": "00000 00000 00000 11111 00000 00000 00000", ".": "00000 00000 00000 00000 00000 01100 01100",
  ":": "00000 01100 01100 00000 01100 01100 00000", "%": "11001 11010 00010 00100 01000 01011 10011",
  "/": "00001 00010 00010 00100 01000 01000 10000", "×": "00000 10001 01010 00100 01010 10001 00000",
  "#": "01010 11111 01010 01010 11111 01010 00000", "!": "00100 00100 00100 00100 00100 00000 00100",
};

const BG = [18, 18, 22];
const INK = [235, 235, 240];
const BAD = [255, 96, 96];

function blank(width: number, height: number): Image {
  const rgba = new Uint8Array(width * height * 4);
  for (let i = 0; i < width * height; i++) rgba.set([BG[0]!, BG[1]!, BG[2]!, 255], i * 4);
  return { width, height, rgba };
}

/** `text` in the pixel font at `scale`, from (x, y), clipped to the image. */
export function label(img: Image, x: number, y: number, text: string, scale = 2, color = INK): void {
  let pen = x;
  for (const c of text.toUpperCase()) {
    const g = GLYPHS[c];
    if (g) {
      const rows = g.split(" ");
      for (let r = 0; r < 7; r++) for (let k = 0; k < 5; k++) if (rows[r]![k] === "1") {
        for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
          const px = pen + k * scale + dx, py = y + r * scale + dy;
          if (px >= 0 && py >= 0 && px < img.width && py < img.height) img.rgba.set([color[0]!, color[1]!, color[2]!, 255], (py * img.width + px) * 4);
        }
      }
    }
    pen += 6 * scale;
  }
}

/** The image shrunk (a box filter) so its height is `height`, its aspect kept. */
export function fit(img: Image, height: number): Image {
  if (img.height <= height) return img;
  const width = Math.max(1, Math.round((img.width * height) / img.height));
  const rgba = new Uint8Array(width * height * 4);
  const sx = img.width / width, sy = img.height / height;
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const x0 = Math.floor(x * sx), x1 = Math.max(x0 + 1, Math.floor((x + 1) * sx));
    const y0 = Math.floor(y * sy), y1 = Math.max(y0 + 1, Math.floor((y + 1) * sy));
    const acc = [0, 0, 0];
    let n = 0;
    for (let v = y0; v < y1; v++) for (let u = x0; u < x1; u++) {
      const i = (v * img.width + u) * 4;
      acc[0]! += img.rgba[i]!;
      acc[1]! += img.rgba[i + 1]!;
      acc[2]! += img.rgba[i + 2]!;
      n++;
    }
    rgba.set([acc[0]! / n, acc[1]! / n, acc[2]! / n, 255], (y * width + x) * 4);
  }
  return { width, height, rgba };
}

export interface SheetCell {
  image: Image | null;
  /** Up to two lines under the picture. */
  lines: string[];
  /** Findings in this cell: its label turns red. */
  flagged?: boolean;
}

const GAP = 8;
const LINE = 18;

/** Cells of one height in a grid of `cols` columns (a row each sheet row). */
export function sheet(cells: readonly SheetCell[], cols: number, height: number, title = ""): Image {
  const thumbs = cells.map((c) => (c.image ? fit(c.image, height) : null));
  const cw = Math.max(height / 2, ...thumbs.map((t) => t?.width ?? 0), ...cells.flatMap((c) => c.lines.map((line) => [...line.toUpperCase()].length * 12)));
  cols = Math.max(1, Math.min(cols, cells.length));
  const rows = Math.ceil(cells.length / cols);
  const lines = Math.max(1, ...cells.map((c) => c.lines.length));
  const top = title ? GAP + LINE + 4 : 0;
  const ch = height + lines * LINE + GAP;
  const out = blank(GAP + cols * (cw + GAP), top + GAP + rows * ch);
  if (title) label(out, GAP, GAP, title, 2);
  for (const [i, c] of cells.entries()) {
    const x = GAP + (i % cols) * (cw + GAP), y = top + GAP + Math.floor(i / cols) * ch;
    const t = thumbs[i];
    if (t) for (let v = 0; v < t.height; v++) out.rgba.set(t.rgba.subarray(v * t.width * 4, (v + 1) * t.width * 4), ((y + v) * out.width + x) * 4);
    else label(out, x + 4, y + height / 2, "NO FRAME", 2, BAD);
    c.lines.forEach((l, k) => label(out, x, y + height + 3 + k * LINE, l, 2, c.flagged && k === 0 ? BAD : INK));
  }
  return out;
}
