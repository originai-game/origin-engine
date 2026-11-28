import { expect, test } from "bun:test";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { create, creationDocument, resolveCreationChoices } from "../../create/bin/create.ts";

test("creation offers the lookless engine default and only explicit style words select stylisation", () => {
  for (const form of ["2d", "2.5d", "3d"] as const) {
    for (const premise of ["", "Third-person action", "FPS", "Racing RPG", "第三人称射击游戏", "interactive film"]) {
      const choices = resolveCreationChoices(premise, { form });
      expect(choices.look).toBeUndefined();
      expect(choices.ui).toBe("default");
    }
    expect(resolveCreationChoices("pixel adventure", { form, look: "none" }).look).toBeUndefined();
  }
  expect(() => resolveCreationChoices("", { variant: "paper" })).toThrow("requires a look");
  expect(() => resolveCreationChoices("", { ui: "pixel" })).toThrow("requires a look");
  const doc = creationDocument();
  expect(doc.schemaVersion).toBe(2);
  expect(doc.options.look[0]).toMatchObject({ id: "none", label: { en: "Realistic (engine default)", "zh-CN": "写实（引擎默认）" }, preview: { path: "previews/realistic-default.webp" }, variants: [] });
  expect(doc.schema.required).not.toContain("look");
  for (const choices of Object.values(doc.defaults.byForm)) expect(choices.look).toBeUndefined();

  for (const [words, look] of [["low poly", "faceted-low-poly"], ["toy", "toy-3d"], ["低模赛车", "faceted-low-poly"], ["comic", "inked-comic-3d"], ["漫画射击", "inked-comic-3d"], ["卡通", "inked-comic-3d"], ["霓虹", "neon-vector"]]) {
    expect(resolveCreationChoices(words!, { form: "3d" })).toMatchObject({ form: "3d", look });
  }
  expect(resolveCreationChoices("像素冒险", { form: "2d" }).look).toBe("pixel");
  expect(resolveCreationChoices("pixel adventure")).toMatchObject({ form: "2d", look: "pixel" });
  expect(resolveCreationChoices("cartoon platformer", { form: "2d" }).look).toBe("cartoon-2d");
  expect(resolveCreationChoices("", { look: "pixel" })).toMatchObject({ form: "2d", look: "pixel" });
});

test.skipIf(!process.env.OG_BIN)("a fresh scaffold writes no look for any form or interactive film", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-lookless-"));
  try {
    for (const form of ["2d", "2.5d", "3d"] as const) {
      for (const genre of ["shooter", "interactive-film"] as const) {
        const root = join(dir, `${form}-${genre}`);
        const made = create({ dir: root, id: "lookless", choices: { form, genre } });
        expect(made.choices.look).toBeUndefined();
        expect(JSON.parse(readFileSync(join(root, "og-project.json"), "utf8"))).not.toHaveProperty("look");
        expect(existsSync(join(root, "content/looks"))).toBe(false);
        expect(readFileSync(join(root, "project.ts"), "utf8")).not.toContain('"look"');
      }
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
