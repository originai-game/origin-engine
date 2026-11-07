// The vendored bun:test declarations move with bun (docs/ci.md §Toolchain):
// the version on packages/engine/vendor/bun-types/test.d.ts's first line,
// docs/ci.md's bun-types row and BUN_VERSION in scripts/install-tools.sh
// are one version.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const repo = join(import.meta.dir, "..");
const read = (path: string) => readFileSync(join(repo, path), "utf8");

test("the vendored bun-types, its docs/ci.md pin and BUN_VERSION are one version", () => {
  const bun = /^BUN_VERSION="\$\{BUN_VERSION:-([^}]+)\}"$/m.exec(read("scripts/install-tools.sh"))?.[1];
  const vendored = /^\/\/ bun-types (\S+):/.exec(read("packages/engine/vendor/bun-types/test.d.ts"))?.[1];
  const ci = read("docs/ci.md");
  const bunRow = /^\| bun \| ([^ |]+) \|/m.exec(ci)?.[1];
  const typesRow = /^\| bun-types [^|]*\| ([^ |]+) \|/m.exec(ci)?.[1];
  expect(bun, "BUN_VERSION in scripts/install-tools.sh").toBeDefined();
  expect({ vendored, bunRow, typesRow }, "move packages/engine/vendor/bun-types (README.md) and docs/ci.md with BUN_VERSION").toEqual({ vendored: bun, bunRow: bun, typesRow: bun });
});
