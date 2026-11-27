import { expect, test } from "bun:test";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

// The test fixtures (tests/fixtures/*) are standalone project skeletons: each
// carries its own tsconfig.json and licences its own assets. Both are kept as
// written files, so this check pins them instead of a generator: every
// fixture's tsconfig.json is byte-identical to the project template's, and
// every asset a fixture's LICENSES.md records exists in that fixture.

const template = join(import.meta.dir, "..", "packages", "create", "template");
const templateTsconfig = readFileSync(join(template, "tsconfig.json"));
const fixtures = join(import.meta.dir, "..", "tests", "fixtures");

for (const name of readdirSync(fixtures).sort()) {
  const dir = join(fixtures, name);
  const tsconfig = join(dir, "tsconfig.json");
  if (existsSync(tsconfig)) {
    test(`${relative(fixtures, tsconfig)} matches packages/create/template/tsconfig.json`, () => {
      expect(readFileSync(tsconfig)).toEqual(templateTsconfig);
    });
  }
  const licences = join(dir, "LICENSES.md");
  if (existsSync(licences)) {
    test(`${name}/LICENSES.md records assets the fixture has`, () => {
      const missing: string[] = [];
      for (const line of readFileSync(licences, "utf8").split("\n")) {
        if (!line.startsWith("| ")) continue;
        const cell = line.slice(2, line.indexOf("|", 2));
        for (const token of cell.replace(/\([^)]*\)/g, "").match(/`([^`]+)`/g) ?? []) {
          const path = token.slice(1, -1);
          if (!path.startsWith("assets/") && !path.startsWith("content/")) continue;
          if (path.includes("*")) {
            const glob = new Bun.Glob(path);
            const hits = [...glob.scanSync({ cwd: dir })].length;
            if (hits === 0) missing.push(path);
          } else if (!existsSync(join(dir, path))) {
            missing.push(path);
          }
        }
      }
      expect(missing).toEqual([]);
    });
  }
}
