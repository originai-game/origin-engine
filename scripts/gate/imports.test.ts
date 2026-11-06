import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ImportGraph } from "./imports";

const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) rmSync(root, { recursive: true });
});

function project(files: Record<string, string>): string {
  const root = mkdtempSync(join(tmpdir(), "og-import-graph-"));
  roots.push(root);
  for (const [name, contents] of Object.entries(files)) {
    const path = join(root, name);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, contents);
  }
  return root;
}

test("self-imports and re-export cycles select only their transitive sources", async () => {
  const root = project({
    "package.json": JSON.stringify({ name: "@gate/self", exports: { ".": "./src/index.ts" } }),
    "test/consumer.test.ts": 'import { a } from "@gate/self"; import { join } from "node:path";',
    "src/index.ts": 'export { a } from "./a";',
    "src/a.ts": 'import "./index"; export const a = 1;',
    "src/unrelated.ts": "export const b = 2;",
    "test/independent.test.ts": "export {};",
  });
  const graph = new ImportGraph(root);
  expect([...await graph.inputs("test/consumer.test.ts")].sort()).toEqual([
    "package.json", "src/a.ts", "src/index.ts", "test/consumer.test.ts",
  ]);
  expect([...await graph.inputs("test/independent.test.ts")].sort()).toEqual([
    "package.json", "test/independent.test.ts",
  ]);
});

test("unresolved imports and runtime file reads refuse selection", async () => {
  for (const source of ['import "./missing";', 'import { readFileSync as read } from "node:fs";']) {
    const root = project({ "test.ts": source });
    await expect(new ImportGraph(root).inputs("test.ts")).rejects.toThrow();
  }
});
