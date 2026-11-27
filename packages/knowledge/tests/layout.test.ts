// The paths a platform reads from the engine package's knowledge/ (AGENTS.md
// §Ownership public contracts, docs/release.md §2): moving one is a break.
// prepack (scripts/pack.ts) fills scaffold/template/ (og api gen writes presets/), and the
// package's `files` is what pack.sh copies into the engine as knowledge/.
import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const pkg = join(import.meta.dir, "..");

test("the package carries looks/catalogue.json, a preset per ready style, the scaffold and the wiki", () => {
  const packed = spawnSync("bun", ["scripts/pack.ts"], { cwd: pkg, encoding: "utf8" });
  expect(packed.status, packed.stderr).toBe(0);
  const files = (JSON.parse(readFileSync(join(pkg, "package.json"), "utf8")) as { files: string[] }).files;
  for (const dir of ["looks", "presets", "scaffold", "wiki", "scripts"]) expect(files).toContain(dir);
  for (const path of [
    "looks/catalogue.json",
    "scaffold/origin-engine.json",
    "scaffold/distribution.json",
    "scaffold/template/og-project.json",
    "scaffold/template/project.ts",
    "scaffold/template/src/main.ts",
    "wiki/index.md",
    "scripts/lib.ts",
  ])
    expect(existsSync(join(pkg, path)), path).toBe(true);
  const styles = (JSON.parse(readFileSync(join(pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string; look: { presetReady: boolean } }[] }).styles;
  const ready = styles.filter((s) => s.look.presetReady).map((s) => s.slug);
  expect(ready.length).toBeGreaterThan(0);
  for (const slug of ready) expect(existsSync(join(pkg, "presets", `${slug}.json`)), `presets/${slug}.json`).toBe(true);
  const scaffold = JSON.parse(readFileSync(join(pkg, "scaffold/origin-engine.json"), "utf8"));
  expect(Object.keys(scaffold)).toEqual(expect.arrayContaining(["schema", "engine", "template", "probe"]));
});
