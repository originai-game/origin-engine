// A sample scene made ready for its look, as a game on the platform makes
// its assets (docs/art.md §2.2): each beat of `beats.json` is briefed with
// artBrief under the project's look. A pixel style's brief names the
// logical grid, which goes into the image's import settings
// (`conform { size N }`, the cook's per-asset size, og-cook README §Art
// conformance). Then `src/cells.ts` gives the scene each sprite's size in
// metres (the cooked texels over ArtPixels.unit, so every sprite shares one
// pixel size) and the view's height (ArtPixels.logicalHeight over unit).
//
//   bun tools/art-samples/prepare.ts <project> [--briefs <file>]
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { artBrief, readLook } from "../../packages/knowledge/scripts/brief.ts";
import type { ArtKind } from "../../packages/knowledge/scripts/brief.ts";

/** One asset of the scene: its brief's kind and subject, its size in metres, and `tile` for a ground that repeats (a one-tile tile set, so the cook lines no edge of it). */
export interface Beat { asset: string; kind: ArtKind; subject: string; metres?: number; tile?: boolean }

/** The width and height of an image file, read by ImageMagick. */
function dims(path: string): [number, number] {
  const r = spawnSync("magick", ["identify", "-format", "%w %h", path], { encoding: "utf8" });
  const [w, h] = r.stdout.trim().split(" ").map(Number);
  if (!w || !h) throw new Error(`${path}: not an image (${r.stderr.trim()})`);
  return [w, h];
}

/** Briefs every beat under the project's look and writes the import settings and src/cells.ts; returns the briefs' text. */
export function prepare(dir: string): string {
  const beats = JSON.parse(readFileSync(join(dir, "beats.json"), "utf8")) as Beat[];
  const look = readLook(dir);
  const px = look.art.ArtPixels ?? {};
  const unit = Number(px.unit ?? 0);
  const rows = Number(px.logicalHeight ?? 0);
  const view = unit > 0 && rows > 0 ? rows / unit : 14;
  const cells: Record<string, { w: number; h: number; grid: number }> = {};
  const briefs: string[] = [];
  for (const b of beats) {
    const brief = artBrief(look, b.kind, b.subject, b.metres ? { metres: b.metres } : {});
    briefs.push(`=== ${b.asset} (${b.kind}${b.metres ? `, ${b.metres} m` : ""})\n${brief.prompt}\n${JSON.stringify(brief.image ?? brief.model)}`);
    // A model (type model text or a glb) is briefed only: the cook holds it to ArtShape and ArtTexels.
    if (!/\.(png|jpe?g)$/.test(b.asset)) continue;
    const [sw, sh] = dims(join(dir, "assets", b.asset));
    const grid = brief.image?.grid;
    const settings = join(dir, "assets", `${b.asset}.ogt`);
    if (b.tile) {
      const name = b.asset.replace(/\.\w+$/, "");
      writeFileSync(join(dir, "assets", `${name}.ogt`), `type tileset\nversion 1\n---\n# Written by tools/art-samples/prepare.ts: one tile that repeats (the ground, the far layer), so the cook draws no line on the edges where it meets the next.\ntexture "${b.asset}"\ngrid ${sw} ${sh} 1 1\n`);
    }
    if (grid && unit > 0) {
      // The brief's cell: its long side is the size the cook draws the image to.
      const long = Math.max(grid.width, grid.height);
      const k = long / Math.max(sw, sh);
      const [cw, ch] = [Math.max(1, Math.round(sw * k)), Math.max(1, Math.round(sh * k))];
      writeFileSync(settings, `type import\nversion 1\n---\n# Written by tools/art-samples/prepare.ts from the art brief's grid (${grid.width} x ${grid.height} logical pixels).\nconform {\n  size ${long}\n}\n`);
      cells[b.asset] = { w: cw / unit, h: ch / unit, grid: long };
    } else {
      writeFileSync(settings, `type import\nversion 1\n---\n# Written by tools/art-samples/prepare.ts: the look names no pixel grid, so the image keeps its size.\n`);
      const h = b.kind === "backdrop" ? view : (b.metres ?? 2);
      cells[b.asset] = { w: (h * sw) / sh, h, grid: 0 };
    }
  }
  const round = (n: number) => Math.round(n * 1e4) / 1e4;
  const body = Object.entries(cells)
    .map(([k, c]) => `  ${JSON.stringify(k)}: { w: ${round(c.w)}, h: ${round(c.h)} },`)
    .join("\n");
  if (Object.keys(cells).length === 0) return briefs.join("\n\n");
  writeFileSync(
    join(dir, "src/cells.ts"),
    `// Written by tools/art-samples/prepare.ts for the look "${look.extends}": each sprite's size in metres\n// (its brief's cell over ArtPixels.unit) and the view's height. Do not edit.\nexport const VIEW_HEIGHT = ${round(view)};\nexport const CELLS: Record<string, { w: number; h: number }> = {\n${body}\n};\n`,
  );
  return briefs.join("\n\n");
}

if (import.meta.main) {
  const dir = process.argv[2];
  if (!dir) throw new Error("usage: bun tools/art-samples/prepare.ts <project> [--briefs <file>]");
  const text = prepare(dir);
  const i = process.argv.indexOf("--briefs");
  if (i > 0) writeFileSync(process.argv[i + 1]!, text + "\n");
  else console.log(text);
}
