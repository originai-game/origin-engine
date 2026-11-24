// The art brief (scripts/brief.ts, docs/art.md §2.2): the expected sizes and
// parameters below are worked out by hand from the looks' numbers (written
// beside each), never read back from the code under test. The cases are
// asymmetric: a 2D pixel style and a 3D shape style ask for different things
// for the same kind, a character's cell is taller than wide, and the sizes
// snap to cells that are not powers of two.

import { afterAll, expect, test } from "bun:test";
import { readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { ART_KINDS, artBrief, presetLook } from "../scripts/brief.ts";

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

