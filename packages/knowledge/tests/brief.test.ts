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
import { ART_KINDS, artBrief, presetLook, readAudioChoices, readLook } from "../scripts/brief.ts";

const temps: string[] = [];
afterAll(() => {
  for (const t of temps) rmSync(t, { recursive: true, force: true });
});

const catalogue = JSON.parse(readFileSync(join(import.meta.dir, "..", "looks", "catalogue.json"), "utf8")) as { styles: { slug: string; look: { presetReady: boolean } }[] };

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
  expect(() => artBrief(presetLook("pixel"), "mesh" as never, "x")).toThrow(/asset kind/);
  expect(() => artBrief(presetLook("pixel"), "sprite", "  ")).toThrow(/subject/);
});

test.skipIf(!process.env.OG_BIN)("project briefs use the restricted JSON/TS bootstrap and refresh imported choices", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-brief-ts-"));
  temps.push(dir);
  mkdirSync(join(dir, "src"));
  mkdirSync(join(dir, "lib"));
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  writeFileSync(join(dir, "og-project.json"), JSON.stringify({ version: 1, engine: "0.9.0", name: "Brief", look: { $ogRef: "base/looks/brief" } }));
  writeFileSync(join(dir, "project.ts"), `import { defineProject } from "@origingame/engine/author";
import { choices } from "./lib/choices.ts";
export default defineProject({ id: "brief", entry: "src/main.ts", create: choices });`);
  writeFileSync(join(dir, "src/main.ts"), "throw new Error('GAME_MUST_NOT_EXECUTE');");
  writeFileSync(join(dir, "assets/looks/brief.ogt"), 'type look\nversion 1\n---\nextends "pixel"\nmood "cold"\n');
  const choices = join(dir, "lib/choices.ts");
  writeFileSync(choices, 'export const choices = { form: "2d", music: "ambient", genre: "adventure" };');
  const look = readLook(dir);
  expect([look.profile, look.style?.slug, look.mood]).toEqual(["2d", "pixel", "cold"]);
  expect(readAudioChoices(dir)).toEqual({ music: "ambient", genre: "adventure", mood: "cold" });
  writeFileSync(choices, 'export const choices = { form: "3d", music: "orchestral", genre: "strategy" };');
  expect(readLook(dir).profile).toBe("3d");
  expect(readAudioChoices(dir)).toEqual({ music: "orchestral", genre: "strategy", mood: "cold" });
  writeFileSync(choices, 'export const choices = { form: "3d", music: String(Date.now()), genre: "strategy" };');
  expect(() => readLook(dir)).toThrow();
});
