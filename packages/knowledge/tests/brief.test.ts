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
import { ART_KINDS, artBrief, audioBrief, authoredArt, readAudioChoices, SFX_PRESETS, colourName, MOODS, parsePalette, presetLook, readLook, RIG_TEMPLATES, rigParts, styleArtBriefs, UI_KINDS, uiAllows, type BriefLook } from "../scripts/brief.ts";

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

test("the kinds of before stay first and in order: the UI kinds are added after them", () => {
  expect(ART_KINDS.slice(0, 8)).toEqual(["sprite", "tileset", "portrait", "backdrop", "prop", "character", "texture", "icon"]);
  expect(ART_KINDS.slice(8)).toEqual(Object.keys(UI_KINDS));
});

test("a UI kind asks for its slot: the aspect, the focal point, the safe area and the size", () => {
  const plain: BriefLook = { extends: null, art: {} };
  // Key art: 16:9 by default, 1024 wide and 1024 x 9 / 16 = 576 tall; the title sits over its left 40 %.
  const key = artBrief(plain, "keyArt", "a lighthouse in a storm");
  expect(key.medium).toBe("image");
  expect(key.image).toEqual({ transparent: false, width: 1024, height: 576, grid: null, tiling: false });
  expect(key.ui).toMatchObject({ aspect: "16:9", focal: [0.66, 0.45], safe: [0, 0, 0.4, 1], fit: "cover", nineSlice: null, cells: null, conform: [] });
  expect(key.prompt).toContain("Safe area: from 0% to 40% across");
  expect(key.prompt).toContain("66% across and 45% down");
  // Portrait 9:16 is as tall as the generator draws: 1024 x 9 / 16 = 576 wide.
  expect([artBrief(plain, "keyArt", "x", { aspect: "9:16" }).image?.width, artBrief(plain, "keyArt", "x", { aspect: "9:16" }).image?.height]).toEqual([576, 1024]);
  // A store's vertical library capsule is its exact size, 600 x 900.
  expect(artBrief(plain, "capsule", "x", { aspect: "2:3" }).image).toMatchObject({ width: 600, height: 900 });
  // A portrait is never drawn wide; a full body anywhere from 1:2 to 9:16.
  expect(() => artBrief(plain, "bust", "x", { aspect: "16:9" })).toThrow(/drawn at 1:1, 3:4, not 16:9/);
  expect(uiAllows("fullBody", "5:9")).toBe(true);
  expect(uiAllows("fullBody", "3:4")).toBe(false);
  // A 2 x 3 inventory item: 1024 tall, 1024 x 2 / 3 = 682.7 wide, to a multiple of 8: 680.
  const sword = artBrief(plain, "itemIcon", "a rune sword", { cells: [2, 3] });
  expect([sword.image?.width, sword.image?.height, sword.image?.transparent]).toEqual([680, 1024, true]);
  expect(sword.ui?.cells).toEqual([2, 3]);
  expect(() => artBrief(plain, "itemIcon", "x", { cells: [3, 3] })).toThrow(/cells/);
  // A logo is the one kind with lettering.
  const logo = artBrief(plain, "logo", "Saltmarsh");
  expect(logo.avoid).not.toContain("text, letters or watermarks");
  expect(logo.prompt).toContain("Lettering: the game's title");
});

test("style art is a nine-slice with its insets and one cell per state", () => {
  const plain: BriefLook = { extends: null, art: {} };
  // Five states across 1024: 204.8 each, down to a multiple of 8: 200; the sheet is 1000 x 200.
  const b = artBrief(plain, "styleButton", "a brass-edged button");
  expect(b.image).toEqual({ transparent: true, width: 1000, height: 200, grid: null, tiling: false });
  expect(b.ui?.nineSlice).toEqual({ insets: [0.25, 0.25, 0.25, 0.25], states: ["normal", "hover", "pressed", "disabled", "focused"] });
  expect(b.prompt).toContain("States: one cell each, left to right: normal, hover, pressed, disabled, focused");
  expect(artBrief(plain, "styleFrame", "x", { insets: [0.2, 0.3, 0.2, 0.1] }).ui?.nineSlice).toEqual({ insets: [0.2, 0.3, 0.2, 0.1], states: ["normal"] });
  expect(() => artBrief(plain, "styleFrame", "x", { insets: [0.5, 0.25, 0.5, 0.25] })).toThrow(/leave a middle/);
  // 16bit-pixel: spriteSizes 16 32 48. A button cell is the largest, 48; five of them 240 x 48, drawn at
  // the whole number nearest 1024 / 240 = 4.27: 4, so 960 x 192. The cook then puts it on the grid.
  const pixel = artBrief(presetLook("16bit-pixel"), "styleButton", "a stone button");
  expect(pixel.image).toEqual({ transparent: true, width: 960, height: 192, grid: { width: 240, height: 48, scale: 4 }, tiling: false });
  expect(pixel.ui?.conform.join(" ")).toContain("pixel grid");
  // A 1 x 2 item there: cells of the smallest size, 16, so 16 x 32 at 1024 / 32 = 32.
  expect(artBrief(presetLook("16bit-pixel"), "itemIcon", "a torch", { cells: [1, 2] }).image?.grid).toEqual({ width: 16, height: 32, scale: 32 });
});

test("a generated style's art requests become one brief per slot, in its shape language, finish and palette", () => {
  // A style og style new wrote ("gilded"): six frames, the button and the primary, three
  // ornaments (ornament rich), the icon set.
  const file = join(import.meta.dir, "fixtures", "generated-art-requests.json");
  const requests = JSON.parse(readFileSync(file, "utf8"));
  const briefs = styleArtBriefs(presetLook("16bit-pixel"), requests);
  expect(briefs.map((b) => b.slot)).toEqual([
    "gilded/panel", "gilded/card", "gilded/field", "gilded/slot", "gilded/tooltip", "gilded/hud",
    "gilded/button", "gilded/button-primary", "gilded/corner", "gilded/divider", "gilded/flourish", "gilded/icons",
  ]);
  const button = briefs.find((b) => b.slot === "gilded/button-primary")!.brief;
  expect(button.kind).toBe("styleButton");
  expect(button.ui?.nineSlice?.states).toEqual(["normal", "hover", "pressed", "disabled", "focused"]);
  expect(button.prompt).toContain("square notched corners");
  expect(button.prompt).toContain("primary");
  expect(button.prompt).toContain(requests.palette.primary);
  const corner = briefs.find((b) => b.slot === "gilded/corner")!.brief;
  expect(corner.kind).toBe("ornament");
  expect(corner.prompt).toContain("rich ornament");
  expect(() => styleArtBriefs(presetLook("16bit-pixel"), { ...requests, slots: [{ name: "x/y", kind: "frame" }] })).toThrow("not an art kind");
});

test("rigParts lists a template's layers back to front, each with its side, its bone and its joint's rule", () => {
  // humanoid-front as the mage draws it (autorig/art/mage.py): the legs behind the robe, the
  // image-left side before the right, the staff under the hand that holds it. og-cook's
  // rig_parts_match_the_templates holds the fields equal to its templates; this holds the shape.
  const front = rigParts("humanoid-front");
  const layers = front.map((p) => p.layer);
  expect(layers.length).toBe(25);
  expect(new Set(layers).size).toBe(25);
  const before = (a: string, b: string) => expect(layers.indexOf(a)).toBeLessThan(layers.indexOf(b));
  before("cape", "leg-upper-l");
  before("foot-r", "pelvis");
  before("pelvis", "torso");
  before("torso", "head");
  before("hand-l", "arm-upper-r");
  before("weapon", "hand-r");
  const required = front.filter((p) => p.required).map((p) => p.layer).sort();
  expect(required).toEqual(
    ["arm-lower-l", "arm-lower-r", "arm-upper-l", "arm-upper-r", "foot-l", "foot-r", "hand-l", "hand-r", "head", "leg-lower-l", "leg-lower-r", "leg-upper-l", "leg-upper-r", "pelvis", "torso"],
  );
  const part = (name: string) => front.find((p) => p.layer === name)!;
  expect(part("foot-l")).toMatchObject({ side: "l", bone: "foot-l", bones: 1, bends: false, hangsFrom: ["leg-lower-l"], joint: "top", what: "the image-left foot" });
  expect(part("cape")).toMatchObject({ side: null, required: false, bones: 3, bends: true, hangsFrom: ["torso"] });
  expect(part("head").hangsFrom).toEqual(["neck", "torso"]);
  expect(part("pelvis")).toMatchObject({ hangsFrom: [], joint: "centre" });
  expect(part("pelvis").note).toContain("root part");
  expect(part("leg-lower-r").note).toContain("hangs from `leg-upper-r`: its joint is the centre of where it overlaps that part at its top");
  expect(part("leg-lower-r").note).toContain("`pivot leg-lower-r <x> <y>`");
  expect(part("weapon").note).toContain("hangs from `hand-r`, else `hand-l`");
  // The same layers seen from the side, where -l is the far side.
  expect(rigParts("humanoid-side").find((p) => p.layer === "foot-l")?.what).toBe("the far foot");
  // A dog's 22 parts (its body, its head and four legs of three required), a creature's 17 with only its body required.
  expect([rigParts("quadruped").length, rigParts("quadruped").filter((p) => p.required).length]).toEqual([22, 14]);
  expect(rigParts("creature").filter((p) => p.required).map((p) => p.layer)).toEqual(["body"]);
  expect([...RIG_TEMPLATES]).toEqual(["humanoid-side", "humanoid-front", "quadruped", "creature"]);
  // Another name is refused, naming the four.
  expect(() => rigParts("humanoid")).toThrow("'humanoid' is not an auto-rig template: use one of humanoid-side, humanoid-front, quadruped, creature");
});

// The audio brief (docs/art.md §2.9). Tempos worked by hand: a cue's bpm is
// the style's range at the mood's place in it plus the cue's own offset
// (chiptune 120-160; warm 0.45; an action cue +0.2, a title +0), its length
// bars x 4 beats x 60 / bpm.
test("the audio brief times each cue from the style's tempo range and the mood, and a rhythm song holds its bpm", () => {
  const b = audioBrief({ music: "chiptune", mood: "warm", genre: "rhythm", premise: "a bakery that keeps time" });
  expect(b.cues.map((c) => c.id)).toEqual(["title", "song", "calibrate", "victory", "defeat"]);
  const song = b.cues.find((c) => c.id === "song")!;
  expect([song.bpm, song.seconds, song.loop, song.file]).toEqual([146, 105.21, true, "music/song.ogg"]); // 120 + 40 x 0.65; 64 x 240 / 146
  expect(song.stems).toEqual(["lead", "harmony", "bass", "drums"]);
  expect(song.prompt).toContain("Hold the tempo exactly at the bpm");
  expect(song.prompt).toContain("a bakery that keeps time");
  const title = b.cues[0]!;
  expect([title.bpm, title.seconds, title.stems.length]).toEqual([138, 55.65, 0]); // 120 + 40 x 0.45; 32 x 240 / 138
  const victory = b.cues.find((c) => c.id === "victory")!;
  expect([victory.loop, victory.prompt.includes("plays once")]).toEqual([false, true]);
});

test("the mood and style change the requests: a dark orchestral RPG is slower, minor, acoustic, its victory still major", () => {
  const b = audioBrief({ music: "orchestral", mood: "dark", genre: "rpg" });
  const battle = b.cues.find((c) => c.id === "battle")!;
  expect([battle.bpm, battle.mode]).toEqual([103, "minor"]); // 70 + 60 x (0.35 + 0.2)
  expect(b.cues.find((c) => c.id === "victory")!.mode).toBe("major");
  expect(b.cues.find((c) => c.id === "defeat")!.mode).toBe("minor");
  expect(b.avoid).toContain("synthesisers");
  expect(b.sfx.find((e) => e.id === "swing")!.prompt).toContain("recorded foley");
  expect(audioBrief({ music: "electronic", mood: "dark", genre: "rpg" }).sfx.find((e) => e.id === "swing")!.prompt).not.toContain("foley");
});

test("every effect names the file and bus it goes to, and its stand-in is a type sfx document og cook reads", () => {
  const b = audioBrief({ music: "chiptune", mood: "bright", genre: "racing" });
  expect(b.sfx.slice(0, 4).map((e) => [e.id, e.bus])).toEqual([["ui-hover", "ui"], ["ui-press", "ui"], ["ui-back", "ui"], ["ui-error", "ui"]]);
  // An engine has no preset: it is requested with no stand-in.
  expect(b.sfx.find((e) => e.id === "engine-loop")!.preset).toBeNull();
  expect(b.presets.some((p) => p.id === "engine-loop")).toBeFalse();
  const skid = b.presets.find((p) => p.id === "skid")!;
  expect([skid.preset, skid.pitch, skid.length, skid.file]).toEqual(["whoosh", 1.08, 0.9, "audio/skid.ogt"]); // chiptune 1 x bright 1.08; chiptune's length 0.9
  expect(skid.ogt).toMatch(/^type sfx\nversion 1\n---\n/);
  expect(skid.ogt).toContain('preset "whoosh"\nseed ');
  expect(b.presets.find((p) => p.id === "ui-hover")!.volume).toBe(0.3);
  // Seeds differ between effects sharing a preset.
  const whooshes = b.presets.filter((p) => p.preset === "whoosh").map((p) => p.seed);
  expect(new Set(whooshes).size).toBe(whooshes.length);
});

test("the stand-in presets are og cook's: SFX_PRESETS is og-cook's sfx::PRESETS", () => {
  const rs = readFileSync(join(import.meta.dir, "..", "..", "..", "crates", "og-cook", "src", "cook", "sfx.rs"), "utf8");
  const block = /pub const PRESETS[^=]*= &\[([\s\S]*?)\];/.exec(rs)![1]!;
  expect([...block.matchAll(/\("([a-z]+)", "/g)].map((m) => m[1])).toEqual([...SFX_PRESETS]);
  for (const genre of ["action-platformer", "shooter", "rpg", "puzzle-match", "strategy-tower-defense", "racing", "runner", "roguelike-card", "idle", "rhythm", "arena", "open-world"]) {
    for (const p of audioBrief({ music: "rock", mood: "soft", genre }).presets) expect(SFX_PRESETS).toContain(p.preset);
  }
});

test("an unknown style, mood or genre is refused with the values; a project's create block is read", () => {
  expect(() => audioBrief({ music: "jazz", mood: "warm", genre: "rpg" })).toThrow("music 'jazz' is not one of chiptune, orchestral");
  expect(() => audioBrief({ music: "folk", mood: "warm", genre: "mmo" })).toThrow("genre 'mmo'");
  const dir = mkdtempSync(join(tmpdir(), "og-audio-"));
  writeFileSync(join(dir, "project.ogt"), 'type project\nversion 1\n---\nname "x"\ncreate {\n  genre "idle"\n  mood "cold"\n  music "folk"\n}\n');
  expect(readAudioChoices(dir)).toEqual({ music: "folk", mood: "cold", genre: "idle" });
  writeFileSync(join(dir, "project.ogt"), 'type project\nversion 1\n---\nname "x"\n');
  expect(() => readAudioChoices(dir)).toThrow("no create block");
  rmSync(dir, { recursive: true, force: true });
});

test("the mood: the preset's own, or the one the game's look asks, reaches every request", () => {
  // faceted-low-poly is drawn soft; nothing asked, the brief says soft.
  expect(presetLook("faceted-low-poly").mood).toBe("soft");
  expect(artBrief(presetLook("faceted-low-poly"), "prop", "a rock").prompt).toContain("Mood: soft: gentle low-contrast light");
  expect(artBrief(presetLook("faceted-low-poly", { mood: "dark" }), "prop", "a rock").prompt).toContain("Mood: dark: low-key light");
  expect(() => presetLook("faceted-low-poly", { mood: "gloomy" as never })).toThrow(/one of bright, dark/);
  // The nearest look naming a mood wins: a flashback in dark over a game drawn warm.
  const dir = project({
    "project.ogt": manifest("looks/flashback.ogt"),
    "assets/looks/game.ogt": 'type look\nversion 1\n---\nextends "monochrome-pixel"\nmood "warm"\n',
    "assets/looks/flashback.ogt": 'type look\nversion 1\n---\nextends "looks/game.ogt"\nmood "dark"\n',
  });
  const look = readLook(dir);
  expect(look.mood).toBe("dark");
  // monochrome-pixel's own palette is handheld4; its dark layer swaps in handheld4-dark, darkest first.
  expect(look.palette?.id).toBe("@palettes/handheld4-dark");
  expect(look.palette?.colours.map((c) => c.hex)).toEqual(["#0a120c", "#1c3320", "#3d6234", "#78935a"]);
  expect(artBrief(look, "sprite", "a lamp").prompt).toContain("#78935a");
  // In its own mood the preset keeps its palette.
  expect(presetLook("monochrome-pixel").palette?.colours.map((c) => c.hex)).toEqual(["#14261b", "#3d6234", "#84a440", "#c9da8f"]);
  expect(presetLook("monochrome-pixel", { mood: "soft" }).palette?.id).toBe("@palettes/handheld4");
  // Every mood reads on every preset.
  for (const s of catalogue.styles.filter((x) => x.look.presetReady)) for (const m of MOODS) expect(artBrief(presetLook(s.slug, { mood: m }), "backdrop", "a quay").prompt).toContain(`Mood: ${m === "high-contrast" ? "high contrast" : m}:`);
});

test("a content style's authored art is what its brief asks for, by kind", () => {
  const clay = presetLook("sculpted-clay");
  expect(authoredArt(clay).map((a) => a.kind)).toEqual(["character", "prop"]);
  const hero = artBrief(clay, "character", "a baker");
  expect(hero.prompt).toContain("Authored: this style's character art is made for it");
  expect(hero.prompt).toContain("hand-modelled in clay");
  // A kind the style does not author carries no such line, and a full style authors nothing.
  expect(artBrief(clay, "icon", "a loaf").prompt).not.toContain("Authored:");
  expect(authoredArt(presetLook("16bit-pixel"))).toEqual([]);
  // Every content style names its art, each kind a brief kind whose prompt asks for it.
  const content = (JSON.parse(readFileSync(join(import.meta.dir, "..", "looks", "catalogue.json"), "utf8")) as { styles: { slug: string; look: { support: string } }[] }).styles.filter((s) => s.look.support === "content");
  expect(content.length).toBe(14);
  for (const s of content) {
    const look = presetLook(s.slug);
    const art = authoredArt(look);
    expect(art.length).toBeGreaterThan(0);
    for (const a of art) expect(artBrief(look, a.kind, "the hero").prompt).toContain(a.what);
  }
});
