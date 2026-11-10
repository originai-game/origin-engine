import { afterAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { affected } from "../src/node/affected.ts";

declare const Bun: {
  spawnSync(argv: string[], options: { cwd: string }): { exitCode: number; stderr: Uint8Array };
};

const roots: string[] = [];
afterAll(() => { for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true }); });

function git(root: string, ...args: string[]): void {
  const result = Bun.spawnSync(["git", ...args], { cwd: root });
  if (result.exitCode !== 0) throw new Error(new TextDecoder().decode(result.stderr));
}

test("affected selection includes transitive game inputs and conservatively retains runtime readers", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-affected-"));
  roots.push(root);
  // A nested project also checks Git-root-relative paths.
  const project = join(root, "game");
  mkdirSync(project);
  const put = (file: string, text: string) => writeFileSync(join(project, file), text);
  put("entry.ts", 'import "./shared";');
  put("shared.ts", "export const value = 1;");
  put("unit.test.ts", 'import "./leaf";');
  put("leaf.ts", "export const value = 1;");
  put("scene.og.ts", "export default {};");
  put("reader.test.ts", 'import { readFileSync } from "node:fs";');
  put("unresolved.test.ts", 'import "./absent";');
  git(root, "init", "-q");
  git(root, "add", ".");
  git(root, "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "-c", "commit.gpgsign=false", "commit", "-qm", "baseline");
  const suites = ["unit.test.ts", "scene.og.ts", "reader.test.ts", "unresolved.test.ts"];
  const select = async () => (await affected(project, "entry.ts", suites, "HEAD")).selected;
  expect(await select()).toEqual([]);
  put("shared.ts", "export const value = 2;");
  expect(await select()).toEqual(["scene.og.ts", "reader.test.ts", "unresolved.test.ts"]);
  git(root, "add", ".");
  put("leaf.ts", "export const value = 2;");
  expect(await select()).toEqual(suites);
  git(root, "restore", "--source=HEAD", "--staged", "--worktree", ".");
  // Untracked source matters when a previously unresolved import appears.
  put("absent.ts", "export {};");
  expect(await select()).toEqual(["reader.test.ts", "unresolved.test.ts"]);
  put("asset.ogt", "mesh {}");
  expect(await select()).toEqual(suites);
  rmSync(join(project, "asset.ogt"));
  rmSync(join(project, "leaf.ts"));
  expect(await select()).toEqual(["unit.test.ts", "reader.test.ts", "unresolved.test.ts"]);
  expect(await affected(project, "entry.ts", suites, "no-such-revision").then(() => false, () => true)).toBe(true);
});

test("computed imports and implicit runtime reads cannot be skipped as independent", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-runtime-inputs-"));
  roots.push(root);
  writeFileSync(join(root, "entry.ts"), "export default {};");
  const sources = [
    'export const load = (path: string) => import(path);',
    'export const read = () => Bun.file("data.ts").text();',
    'export const read = () => B\\u0075n.file("data.ts").text();',
    'export const read = () => globalThis["Bun"].file("data.ts").text();',
    'export const load = () => require("./data");',
  ];
  const suites = sources.map((source, i) => {
    const file = `runtime-${i}.test.ts`;
    writeFileSync(join(root, file), source);
    return file;
  });
  writeFileSync(join(root, "independent.test.ts"), "export const answer = 42;");
  git(root, "init", "-q");
  git(root, "add", ".");
  git(root, "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "-c", "commit.gpgsign=false", "commit", "-qm", "baseline");
  writeFileSync(join(root, "data.ts"), "export default 5;");
  expect((await affected(root, "entry.ts", [...suites, "independent.test.ts"], "HEAD")).selected).toEqual(suites);
});
