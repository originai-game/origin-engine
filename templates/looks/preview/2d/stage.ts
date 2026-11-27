// Stages the 2D preview scene for one look (render.ts calls it with the
// scene copied to `dir`): the look's art into assets/art/ with the import
// settings its contract gives each beat, the level's tile set and map, the
// HUD's font, and src/art.ts, which tells the scene each beat's size.
//
// A beat's size is preview.json's `heights` or `widths` (metres; the other
// side from the image). On a pixel grid (`unit` logical pixels a metre) that is a whole
// number of logical pixels each way, which the import settings ask of the
// cook (`conform { size N }`), so a texel is a logical pixel. The level's
// tiles are a metre, `unit` pixels, each.
import { copyFileSync, cpSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export interface Scene2d {
  plan?: "side" | "top";
  art?: string;
  /** Logical pixels a metre; unset for a look without a pixel grid. */
  unit?: number;
  /** The view's height in metres. */
  view?: number;
  /** day, night or none (the header of preview.json). */
  light?: "day" | "night" | "none";
  /** The tree line's opacity. */
  haze?: number;
  /** The HUD count's colour and its rim's, as #rrggbb: white on near black unless the look's contract names its text's colours. */
  text?: string;
  rim?: string;
  heights?: Record<string, number>;
  /** Beats sized by their width in metres instead. */
  widths?: Record<string, number>;
}

/** A beat's size in metres from preview.json (its height, or its width) and its image; on a pixel grid a whole number of logical pixels each way. */
export function beatSize(scene: Scene2d, beat: string, w: number, h: number): { width: number; height: number; pw: number; ph: number } | undefined {
  const unit = scene.unit ?? 0;
  const wide = scene.widths?.[beat];
  const tall = scene.heights?.[beat];
  if (wide === undefined && tall === undefined) return undefined;
  const height = wide !== undefined ? (wide * h) / w : tall!;
  const width = wide ?? (tall! * w) / h;
  if (!unit) return { width, height, pw: 0, ph: 0 };
  const pw = Math.max(1, Math.round(width * unit));
  const ph = Math.max(1, Math.round(height * unit));
  return { width: pw / unit, height: ph / unit, pw, ph };
}

/** A #rrggbb colour as the linear colour the engine takes. */
function linear(hex: string): { r: number; g: number; b: number } {
  const c = (i: number) => {
    const v = parseInt(hex.slice(i, i + 2), 16) / 255;
    return Number((v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4).toFixed(4));
  };
  return { r: c(1), g: c(3), b: c(5) };
}

/** An image's width and height, from its PNG or JPEG header. */
function size2(file: string): [number, number] {
  const b = readFileSync(file);
  if (b.readUInt32BE(0) === 0x89504e47) return [b.readUInt32BE(16), b.readUInt32BE(20)];
  for (let i = 2; i < b.length; ) {
    const marker = b[i + 1]!;
    if (marker >= 0xc0 && marker <= 0xc3) return [b.readUInt16BE(i + 7), b.readUInt16BE(i + 5)];
    i += 2 + b.readUInt16BE(i + 2);
  }
  throw new Error(`${file}: no size in its header`);
}

/** The tile sets of each plan: 4 x 4 tiles an image. */
const TILESETS = { side: ["tiles"], top: ["grass", "path", "wall"] } as const;
/** Layers the cook must not line as sprites: one tile the image's size. */
const LAYERS = ["backdrop", "mid"];

/**
 * The side level, 40 x 8 cells with its top-left corner at (-20, 2): the
 * ground's top at y 0, a ledge two tiles up from x 4 on. A column's tile is
 * its depth under its own surface (the tile set's rows: grass, then three
 * of earth, the last repeating) and its x (the set's four columns).
 */
function sideMap(unit: number): string {
  const cells: number[] = [];
  for (let r = 0; r < 8; r++) {
    for (let c = 0; c < 40; c++) {
      const surface = c - 20 >= 4 ? 0 : 2;
      const depth = r - surface;
      cells.push(depth < 0 ? 0 : Math.min(depth, 3) * 4 + (c % 4) + 1);
    }
  }
  return `import {file,ref} from '@origingame/engine/author';\n// The level's ground and its ledge.\nexport default file.tilemap({width:40,height:8,tileWidth:${unit},tileHeight:${unit},unit:1,tilesets:[ref('base/art/tiles-set')],layers:[{name:'ground',tileset:0,cells:${JSON.stringify(cells)}}]});\n`;
}

/**
 * The top-down level, 28 x 16 cells with its top-left corner at (-14, 8): a
 * meadow, a paved path two tiles wide along y 0 that turns south at x 2,
 * and a plateau from x 4 on above y 2 whose south face is two tiles of
 * cliff, with a rim of the same rock up its west side. Each layer draws its own tile set by the cell's x and y.
 */
function topMap(unit: number): string {
  const layer = (name: string, tileset: number, z: number, at: (x: number, y: number) => boolean, tile: (c: number, r: number) => number) => {
    const cells: number[] = [];
    for (let r = 0; r < 16; r++) {
      for (let c = 0; c < 28; c++) cells.push(at(c - 14, 7 - r) ? tile(c, r) + 1 : 0);
    }
    return { name, tileset, z, cells };
  };
  const any = (c: number, r: number) => (r % 4) * 4 + (c % 4);
  const cliff = (x: number, y: number) => (x >= 4 && (y === 2 || y === 3)) || (x === 4 && y > 3);
  const path = (x: number, y: number) => ((y === 0 || y === -1) && x < 4) || ((x === 2 || x === 3) && y < 0);
  // The cliff's rows: its top, then its lowest (the darkest) at its foot; the rim up the plateau's west side takes the two between.
  const face = (c: number, r: number) => (r === 4 ? 0 : r === 5 ? 12 : 4 + (r % 2) * 4) + (c % 4);
  const layers = [layer("grass", 0, 0, () => true, any), layer("path", 1, 1, path, any), layer("cliff", 2, 2, cliff, face)];
  return `import {file,ref} from '@origingame/engine/author';\n// The meadow, its path and the plateau's face.\nexport default file.tilemap({width:28,height:16,tileWidth:${unit},tileHeight:${unit},unit:1,tilesets:[ref('base/art/grass-set'),ref('base/art/path-set'),ref('base/art/wall-set')],layers:${JSON.stringify(layers)}});\n`;
}

/** Stages the scene in `dir` for the look `name`, whose art preview.json's `scene` names under `looks`. */
export function stage2d(dir: string, name: string, scene: Scene2d, looks: string, repo: string): void {
  const plan = scene.plan ?? "side";
  const unit = scene.unit ?? 0;
  const art = join(looks, scene.art!);
  const out = join(dir, "assets/art");
  const content = join(dir, "content/art");
  mkdirSync(content, { recursive: true });
  cpSync(art, out, { recursive: true });
  const beats: Record<string, { file: string; width: number; height: number }> = {};
  // A tile's side in the map: the pixel unit, or a quarter of a tile set's image.
  let tile = unit;
  for (const file of readdirSync(art).filter((n) => /\.(png|jpe?g)$/.test(n))) {
    const beat = file.replace(/\.[a-z]+$/, "");
    const [w, h] = size2(join(art, file));
    const at = beatSize(scene, beat, w, h);
    const settings = { kind: "texture", source: `assets/art/${file}`, ...(unit && at ? { conform: {kind: "auto", size: Math.max(at.pw, at.ph)} } : {}) };
    writeFileSync(join(content, `${beat}.ts`), `import {file} from '@origingame/engine/author';\nexport default file.import(${JSON.stringify(settings)});\n`);
    if ((TILESETS[plan] as readonly string[]).includes(beat)) {
      tile ||= w / 4;
      // The cook brings a tile set's tiles to the look's pixel unit by itself.
      writeFileSync(join(content, `${beat}-set.ts`), `import {file,ref} from '@origingame/engine/author';\n// Written by templates/looks/preview/2d/stage.ts: 4 x 4 tiles.\nexport default file.tileset({texture:ref(${JSON.stringify(`base/art/${beat}`)}),tileWidth:${w / 4},tileHeight:${h / 4},columns:4,rows:4,filter:'nearest'});\n`);
      continue;
    }
    if (!at) continue;
    beats[beat] = { file: `base/art/${beat}`, width: at.width, height: at.height };
    if (!unit) continue;
    if (LAYERS.includes(beat)) writeFileSync(join(content, `${beat}-set.ts`), `import {file,ref} from '@origingame/engine/author';\n// Written by templates/looks/preview/2d/stage.ts: one tile, so the cook lines no edge of the layer.\nexport default file.tileset({texture:ref(${JSON.stringify(`base/art/${beat}`)}),tileWidth:${w},tileHeight:${h},columns:1,rows:1,filter:'nearest'});\n`);
  }
  mkdirSync(join(dir, "content/level"), { recursive: true });
  writeFileSync(join(dir, "content/level/level.ts"), plan === "top" ? topMap(tile) : sideMap(tile));
  mkdirSync(join(dir, "assets/fonts"), { recursive: true });
  copyFileSync(join(repo, "templates/fonts/latin/assets/fonts/latin/NotoSans-Latin.ttf"), join(dir, "assets/fonts/NotoSans-Latin.ttf"));
  mkdirSync(join(dir, "content/fonts"), { recursive: true });
  writeFileSync(join(dir, "content/fonts/NotoSans-Latin.ts"), "import {file} from '@origingame/engine/author';export default file.import({kind:'font',source:'assets/fonts/NotoSans-Latin.ttf'});\n");
  writeFileSync(
    join(dir, "src/art.ts"),
    `// Written by templates/looks/preview/2d/stage.ts from preview.json.\nexport const PLAN: "side" | "top" = ${JSON.stringify(plan)};\nexport const VIEW_HEIGHT = ${scene.view};\n/** Logical pixels a metre; 0 without a pixel grid. */\nexport const UNIT = ${unit};\nexport const LIGHT: "day" | "night" | "none" = ${JSON.stringify(scene.light ?? "day")};\nexport const HAZE = ${scene.haze ?? 1};\n/** The HUD count's linear colour and its rim's. */\nexport const TEXT = ${JSON.stringify({ color: linear(scene.text ?? "#ffffff"), rim: linear(scene.rim ?? "#0b0b12") })};\nexport const ART: Record<string, { file: string; width: number; height: number }> = ${JSON.stringify(beats)};\n`,
  );
}
