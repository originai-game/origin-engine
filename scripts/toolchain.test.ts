// The vendored bun:test declarations move with bun (docs/ci.md §Toolchain):
// the version on packages/engine/vendor/bun-types/test.d.ts's first line,
// docs/ci.md's bun-types row and BUN_VERSION in scripts/install-tools.sh
// are one version.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
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

// The engine package's own typecheck reads the same vendored file through
// tsconfig.runtime.json `paths`; a local `declare module "bun:test"` stub once
// shadowed it, and engine tests used matchers the stub lacked. The pinned
// tsrs (what `og check` runs) checks the three configs of `bun run typecheck`
// without the native og, which quick does not build, and a probe that uses an
// asymmetric matcher through the runtime config.
const tsrs = (config: string) => {
  const r = Bun.spawnSync(["tsrs", "-p", config, "--noEmit", "--pretty", "false"], { cwd: repo });
  return `${r.stdout}${r.stderr}`.trim() || (r.exitCode === 0 ? "ok" : `exit ${r.exitCode}`);
};

test.each(["tsconfig.json", "tsconfig.runtime.json", "tsconfig.templates.json"])("packages/engine/%s type-checks", (config) => {
  expect(tsrs(`packages/engine/${config}`)).toBe("ok");
}, 60_000);

test("bun:test in the engine package resolves to the vendored bun-types", () => {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "og-bun-types-")));
  try {
    writeFileSync(join(dir, "probe.test.ts"), `import { expect } from "bun:test";
expect({ reason: "a stable id" }).toMatchObject({ reason: expect.stringContaining("stable") });
expect("abc").toEqual(expect.stringMatching(/b/));
`);
    writeFileSync(join(dir, "tsconfig.json"), JSON.stringify({ extends: join(repo, "packages/engine/tsconfig.runtime.json"), include: ["probe.test.ts"] }));
    expect(tsrs(join(dir, "tsconfig.json"))).toBe("ok");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
