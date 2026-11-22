// scripts/guide-recipes.ts: a guide's recipes are found with their lines,
// and a broken one fails naming its guide file and line. The runs need an
// og binary: the guide-recipes step sets OG_BIN; bun-quick, which runs
// before og is built, skips them.
import { describe, expect, test } from "bun:test";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { check, extract } from "./guide-recipes";

const GUIDE = `# A guide

Prose, then a recipe.

\`\`\`ts recipe ticks=30
import { og, defineSystem } from "@origingame/engine";
const Tick = defineSystem("Tick", { reads: [], writes: [] }, (_q, time) => {
  if (time.tick === 20) og.assert.reachable("tick twenty", {});
});
export default og.game({ systems: [Tick] });
\`\`\`

\`\`\`ts
const notARecipe = 1;
\`\`\`

\`\`\`ts recipe
import { og } from "@origingame/engine";
const n: number = "a string";
export default og.game({});
\`\`\`
`;

describe("extract", () => {
  test("finds each recipe block, its first code line and its ticks", () => {
    const recipes = extract("g.md", GUIDE);
    expect(recipes.map((r) => [r.line, r.ticks])).toEqual([[6, 30], [18, 120]]);
    expect(recipes[0]!.source.startsWith("import { og, defineSystem }")).toBe(true);
    expect(recipes[1]!.source).toContain(`const n: number = "a string";`);
  });

  test("an unclosed block names its line", () => {
    expect(() => extract("g.md", "a\n```ts recipe\nconst x = 1;\n")).toThrow("g.md:2: a recipe block that never closes");
  });
});

const og = process.env.OG_BIN;
const runs = og && existsSync(og) ? describe : describe.skip;

runs("check (og run)", () => {
  const root = resolve(import.meta.dir, "..");
  // Inside the checkout, so @origingame/engine resolves.
  const out = join(root, "target", "guide-recipes", "self-test");
  const recipe = (source: string, ticks = 30) => ({ guide: "crates/og-api/guides/x.md", line: 40, ticks, source });
  const run = (source: string, ticks = 30) => check(recipe(source, ticks), out, og!);

  test("a recipe whose assertions hold passes", () => {
    expect(run(extract("g.md", GUIDE)[0]!.source)).toEqual([]);
  }, 120_000);

  test("a type error names the guide line", () => {
    const failures = run(`import { og } from "@origingame/engine";\nconst n: number = "a string";\nexport default og.game({});\n`);
    expect(failures).toHaveLength(1);
    expect(failures[0]).toStartWith("crates/og-api/guides/x.md:41: ");
    expect(failures[0]).toContain("not assignable");
  }, 120_000);

  test("a violated always and a reachable that never runs name their guide lines", () => {
    const failures = run(`import { og, defineSystem } from "@origingame/engine";
const S = defineSystem("S", { reads: [], writes: [] }, (_q, time) => {
  og.assert.always(time.tick < 10, "under ten", {});
  if (time.tick === 1000) og.assert.reachable("never", {});
});
export default og.game({ systems: [S] });
`);
    expect(failures.some((f) => f.startsWith("crates/og-api/guides/x.md:42: og.assert.always \"under ten\" violated"))).toBe(true);
    expect(failures).toContain("crates/og-api/guides/x.md:43: og.assert.reachable \"never\" never fired in 30 ticks");
  }, 120_000);

  test("a throw names the guide line", () => {
    const failures = run(`import { og, defineSystem } from "@origingame/engine";
const S = defineSystem("S", { reads: [], writes: [] }, (_q, time) => {
  og.assert.reachable("ran", {});
  if (time.tick === 3) throw new Error("boom");
});
export default og.game({ systems: [S] });
`);
    expect(failures).toHaveLength(1);
    expect(failures[0]).toContain("boom");
    expect(failures[0]).toContain("crates/og-api/guides/x.md:43");
  }, 120_000);

  test("a recipe with no assertion fails", () => {
    expect(run(`import { og } from "@origingame/engine";\nexport default og.game({});\n`)).toEqual([
      "crates/og-api/guides/x.md:39: the recipe asserts nothing (no og.assert site)",
    ]);
  }, 120_000);

  test("the command names a broken recipe's guide file and line and exits 1", () => {
    const dir = mkdtempSync(join(tmpdir(), "guide-recipes-"));
    try {
      const guide = join(dir, "broken.md");
      writeFileSync(guide, GUIDE);
      const r = Bun.spawnSync(["bun", join(root, "scripts", "guide-recipes.ts"), "--og", og!, "--out", out, guide], { stdout: "pipe", stderr: "pipe" });
      expect(r.exitCode).toBe(1);
      expect(r.stderr.toString()).toContain(`guide-recipes: ${guide}:19: `);
      expect(r.stdout.toString()).toContain("2 recipe(s) in 1 guide(s), 1 failure(s)");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  }, 120_000);
});
