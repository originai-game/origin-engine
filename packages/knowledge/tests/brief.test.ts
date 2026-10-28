// The art brief (scripts/brief.ts, docs/art.md §2.2): the expected sizes and
// parameters below are worked out by hand from the looks' numbers (written
// beside each), never read back from the code under test. The cases are
// asymmetric: a 2D pixel style and a 3D shape style ask for different things
// for the same kind, a character's cell is taller than wide, and the sizes
// snap to cells that are not powers of two.

import { afterAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ART_KINDS, artBrief, colourName, parsePalette, presetLook, readLook, type BriefLook } from "../scripts/brief.ts";

const temps: string[] = [];
afterAll(() => {
  for (const t of temps) rmSync(t, { recursive: true, force: true });
});

const catalogue = JSON.parse(readFileSync(join(import.meta.dir, "..", "looks", "catalogue.json"), "utf8")) as { styles: { slug: string; look: { presetReady: boolean } }[] };

test("the same kind asks a 2D pixel style for a sprite and a 3D shape style for a model", () => {
  // 16bit-pixel: ArtPixels unit 16, spriteSizes 16 32 48; a 2D style. A prop is 1 m: 16 px, a cell of 16.
  // Drawn at the whole number nearest 1024 / 16 = 64.
  const pixel = artBrief(presetLook("16bit-pixel"), "prop", "a brass lantern");
  expect(pixel.medium).toBe("image");
  expect(pixel.model).toBeUndefined();
  expect(pixel.image).toEqual({ transparent: true, width: 1024, height: 1024, grid: { width: 16, height: 16, scale: 64 }, tiling: false });
  // faceted-low-poly: ArtShape propFaces 200, normals flat; ArtTexels maxSize 64, filter nearest, mips none; a 3D style.
  const faceted = artBrief(presetLook("faceted-low-poly"), "prop", "a brass lantern");
  expect(faceted.medium).toBe("model");
  expect(faceted.image).toBeUndefined();
  expect(faceted.model).toEqual({ faceBudget: 200, textureSize: 64, normals: "flat", voxel: 0, bevel: 0, filter: "nearest", mips: "none", metres: 1 });
  expect(faceted.prompt).toContain("at most 200 triangles");
  expect(pixel.prompt).not.toContain("triangles");
  expect(pixel.prompt).toContain("16 x 16 logical-pixel grid");
  // A character: 2 m tall is 32 px (a cell of 32), 1 m wide 16 px; 1024 / 32 = 32.
  const hero = artBrief(presetLook("16bit-pixel"), "character", "a courier");
  expect(hero.image?.grid).toEqual({ width: 16, height: 32, scale: 32 });
  expect([hero.image?.width, hero.image?.height]).toEqual([512, 1024]);
  // The voxel style's character: characterFaces 5000, voxel 0.1, normals flat.
  const voxel = artBrief(presetLook("voxel-worlds"), "character", "a courier");
  expect(voxel.model?.faceBudget).toBe(5000);
  expect(voxel.model?.voxel).toBe(0.1);
  expect(voxel.prompt).toContain("0.1 m cubes");
});

/** A look written out here, its numbers chosen so no two sizes coincide. */
const synthetic = (art: BriefLook["art"], profile: "2d" | "3d", extra: Partial<BriefLook> = {}): BriefLook => ({ extends: null, art, profile, ...extra });

test("a 2D cell snaps up to the look's sprite sizes, taller than wide for a character", () => {
  // unit 10 px per metre, cells of 12, 24 or 40. A 2 m character: 20 px tall -> 24; 10 px wide -> 12.
  // 1024 / 24 = 42.67, nearest whole 43: 12 x 43 = 516, 24 x 43 = 1032.
  const look = synthetic({ ArtPixels: { unit: 10, spriteSizes: [40, 12, 24] } }, "2d");
  const b = artBrief(look, "character", "a knight");
  expect(b.image?.grid).toEqual({ width: 12, height: 24, scale: 43 });
  expect([b.image?.width, b.image?.height]).toEqual([516, 1032]);
  // A 3.5 m sprite: 35 px -> 40. A tile set: 1 m tiles of 10 px -> 12, four by four: 48 x 48, drawn at 21 (1024 / 48 = 21.3).
  expect(artBrief(look, "sprite", "a tree", { metres: 3.5 }).image?.grid).toEqual({ width: 40, height: 40, scale: 26 });
  const tiles = artBrief(look, "tileset", "a cave floor");
  expect(tiles.image).toEqual({ transparent: false, width: 1008, height: 1008, grid: { width: 48, height: 48, scale: 21 }, tiling: true });
  // An icon takes the smallest cell; a backdrop the logical height at 16:9.
  expect(artBrief(look, "icon", "a heart").image?.grid?.width).toBe(12);
  const screen = synthetic({ ArtPixels: { logicalHeight: 180 } }, "2d");
  expect(artBrief(screen, "backdrop", "a harbour at dusk").image?.grid).toEqual({ width: 320, height: 180, scale: 3 });
});

test("a model's texture is the density over its size, to a power of two, within the largest size", () => {
  // 48 texels per metre over 3 m: 144 -> 256; maxSize 100 holds it to 64 (the largest power of two at or below 100).
  const held = synthetic({ ArtTexels: { density: 48, maxSize: 100 }, ArtShape: { propFaces: 750, characterFaces: 3000, bevel: 0.02 } }, "3d");
  const prop = artBrief(held, "prop", "a crate", { metres: 3 });
  expect(prop.model?.textureSize).toBe(64);
  expect(prop.model?.faceBudget).toBe(750);
  expect(prop.model?.bevel).toBe(0.02);
  const free = synthetic({ ArtTexels: { density: 48 } }, "3d");
  expect(artBrief(free, "prop", "a crate", { metres: 3 }).model?.textureSize).toBe(256);
  // A character defaults to 2 m: 96 -> 128, and takes the character budget.
  expect(artBrief(held, "character", "a guard").model?.faceBudget).toBe(3000);
  expect(artBrief(free, "character", "a guard").model?.textureSize).toBe(128);
  // The same prop in a 2D game is a sprite.
  expect(artBrief({ ...held, profile: "2d" }, "prop", "a crate").medium).toBe("image");
  // A surface texture's tile is 2 m: 96 -> 128 texels; linear, so drawn at the generator's 1024 and resampled.
  const tex = artBrief(free, "texture", "cobbles");
  expect(tex.image?.grid).toBeNull();
  expect(tex.prompt).toContain("128 x 128 texels covering 2 m (48 texels per metre)");
  expect([tex.image?.width, tex.image?.tiling, tex.image?.transparent]).toEqual([1024, true, false]);
});

test("the palette is named with hex, accents and strictness included", () => {
  const colours = parsePalette("p.hex", "; a test palette\n1a1c2c\n#5d275d\nb13e53\n\nffcd75\n");
  expect(colours.map((c) => c.hex)).toEqual(["#1a1c2c", "#5d275d", "#b13e53", "#ffcd75"]);
  const look = synthetic({ ArtPalette: { strict: true, accents: [2], dither: "bayer4" } }, "2d", { palette: { id: "palettes/p.hex", from: "ArtPalette.colors", colours } });
  const b = artBrief(look, "sprite", "a slime");
  for (const c of colours) expect(b.prompt).toContain(`${c.name} ${c.hex}`);
  expect(b.prompt).toContain("only these 4 colours");
  expect(b.prompt).toMatch(new RegExp(`${colours[2]!.name} #b13e53 kept for signals`));
  expect(b.prompt).toContain("bayer4 dither");
  expect(b.palette).toEqual(colours);
  // Not strict: built on them.
  const loose = artBrief(synthetic({}, "2d", { palette: { id: "x", from: "Stylise.palette", colours } }), "sprite", "a slime");
  expect(loose.prompt).toContain("built on these 4 colours");
});

test("colours get names a person says, unique within a palette; a .gpl keeps its own", () => {
  expect(colourName("#ffffff")).toBe("white");
  expect(colourName("#000000")).toBe("near-black");
  expect(colourName("#808080")).toBe("grey");
  expect(colourName("#ff0000")).toContain("red");
  expect(colourName("#0000ff")).toContain("blue");
  expect(colourName("#00a000")).toContain("green");
  expect(colourName("#ffd800")).toMatch(/yellow|amber/);
  const twins = parsePalette("t.hex", "ff0000\nfe0101\n00a000\n");
  expect(twins[0]!.name).not.toBe(twins[1]!.name);
  expect(twins[0]!.name.replace(/ \d+$/, "")).toBe(twins[1]!.name.replace(/ \d+$/, ""));
  const gpl = parsePalette("g.gpl", "GIMP Palette\nName: test\nColumns: 2\n# comment\n255 128 0 Ember\n 10  20  30\tUntitled\n");
  expect(gpl).toEqual([
    { name: "ember", hex: "#ff8000" },
    { name: colourName("#0a141e"), hex: "#0a141e" },
  ]);
});

test("references: the style's preview first, then the accepted assets in order", () => {
  const look = presetLook("monochrome-pixel");
  const b = artBrief(look, "sprite", "a bat", { references: ["assets/sprites/hero.png", "assets/sprites/door.png"] });
  expect(b.references.map((r) => r.role)).toEqual(["style", "accepted", "accepted"]);
  expect(b.references[0]!.path.endsWith("previews/monochrome-pixel.webp")).toBe(true);
  expect(b.references.slice(1).map((r) => r.path)).toEqual(["assets/sprites/hero.png", "assets/sprites/door.png"]);
  // monochrome-pixel's palette is the built-in @palettes/handheld4: four colours, strict.
  expect(look.palette?.id).toBe("@palettes/handheld4");
  expect(look.palette?.colours.length).toBe(4);
  expect(b.prompt).toContain("only these 4 colours");
});

function project(files: Record<string, string>): string {
  const dir = mkdtempSync(join(tmpdir(), "og-brief-"));
  temps.push(dir);
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(join(dir, path, ".."), { recursive: true });
    writeFileSync(join(dir, path), text);
  }
  return dir;
}

const manifest = (look: string | null, profile = "2d") => `type project\nversion 1\n---\nname "T"\nid "t"\nprofile "${profile}"\n${look ? `look "${look}"\n` : ""}`;

test("readLook follows project.ogt's look through the game's own looks to the preset, field by field", () => {
  const dir = project({
    "project.ogt": manifest("looks/game.ogt"),
    "assets/looks/game.ogt": 'type look\nversion 1\n---\nname "Game"\nextends "looks/base.ogt"\nArtPixels {\n  unit 20   # wins over base and preset\n}\nArtPalette {\n  colors "palettes/mine.hex"\n  accents 1\n}\n',
    "assets/looks/base.ogt": 'type look\nversion 1\n---\nname "Base"\nextends "16bit-pixel"\nArtPixels {\n  unit 12\n  rotation nearest\n}\n',
    "assets/palettes/mine.hex": "102030\nf0e0d0\n",
  });
  const look = readLook(dir);
  expect(look.extends).toBe("16bit-pixel");
  expect(look.style?.slug).toBe("16bit-pixel");
  expect(look.profile).toBe("2d");
  // unit from the game, rotation from the base, spriteSizes and logicalHeight from the preset (16 32 48, 224).
  expect(look.art.ArtPixels).toEqual({ unit: 20, rotation: "nearest", spriteSizes: [16, 32, 48], logicalHeight: 224, snap: true });
  expect(look.art.ArtLine).toEqual({ width: 1, place: "inside" });
  expect(look.palette?.colours.map((c) => c.hex)).toEqual(["#102030", "#f0e0d0"]);
  expect(look.preview?.endsWith("previews/16bit-pixel.webp")).toBe(true);
  // A 1 m prop at 20 px per metre snaps up to the cell of 32.
  const b = artBrief(look, "prop", "a barrel");
  expect(b.image?.grid).toEqual({ width: 32, height: 32, scale: 32 });
  expect(b.prompt).toContain("#f0e0d0 kept for signals");
});

test("readLook: a look named by its slug finds the game's own copy first; no look line is refused with the fix", () => {
  const dir = project({
    "project.ogt": manifest("assets/looks/game.ogt", "3d"),
    "assets/looks/game.ogt": 'type look\nversion 1\n---\nextends "faceted-low-poly"\n',
    "assets/looks/faceted-low-poly.ogt": 'type look\nversion 1\n---\nextends "faceted-low-poly"\nArtShape {\n  propFaces 90\n}\n',
  });
  const look = readLook(dir);
  expect(look.art.ArtShape?.propFaces).toBe(90);
  expect(look.art.ArtShape?.characterFaces).toBe(1500);
  expect(artBrief(look, "prop", "a rock").model?.faceBudget).toBe(90);
  expect(() => readLook(project({ "project.ogt": manifest(null) }))).toThrow(/look "looks\/<id>\.ogt"/);
});

test("every kind for every preset makes a complete brief", () => {
  for (const s of catalogue.styles.filter((x) => x.look.presetReady)) {
    const look = presetLook(s.slug);
    for (const kind of ART_KINDS) {
      const b = artBrief(look, kind, "a weathered signpost");
      expect(b.style).toBe(s.slug);
      expect(b.prompt.split("\n")[0]).toContain("a weathered signpost");
      expect(b.prompt).toContain("Avoid: ");
      expect(b.references[0]?.role).toBe("style");
      expect((b.medium === "model") === (b.model !== undefined) && (b.medium === "image") === (b.image !== undefined)).toBe(true);
      if (b.image?.grid) expect([b.image.width, b.image.height]).toEqual([b.image.grid.width * b.image.grid.scale, b.image.grid.height * b.image.grid.scale]);
      if (b.model) expect(Math.log2(b.model.textureSize) % 1).toBe(0);
    }
  }
  expect(() => artBrief(presetLook("16bit-pixel"), "mesh" as never, "x")).toThrow(/asset kind/);
  expect(() => artBrief(presetLook("16bit-pixel"), "sprite", "  ")).toThrow(/subject/);
});

test("a look with no contract still asks for a whole asset", () => {
  const b = artBrief({ extends: null, art: {} }, "sprite", "a fox");
  expect(b.image).toEqual({ transparent: true, width: 1024, height: 1024, grid: null, tiling: false });
  expect(b.references).toEqual([]);
  expect(artBrief({ extends: null, art: {} }, "prop", "a fox").model).toEqual({ faceBudget: null, textureSize: 1024, normals: "any", voxel: 0, bevel: 0, filter: "any", mips: "any", metres: 1 });
});
