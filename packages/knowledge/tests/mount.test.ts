// The mount writes the package into the platform's knowledge tree and owns
// only what it mounts. Each test runs the CLI against a stand-in origin-agent
// checkout (a knowledge/ tree with someone else's skill and overlay) and
// asserts on the files it leaves and the exit status --check returns.

import { afterAll, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defaultRoots, isText, payload } from "../scripts/lib.ts";
import { installed } from "../scripts/mount.ts";

const temps: string[] = [];
afterAll(() => {
  for (const t of temps) rmSync(t, { recursive: true, force: true });
});

const SCRIPT = join(import.meta.dir, "../scripts/mount.ts");
const PROD = "knowledge";

function platform(): string {
  const dir = mkdtempSync(join(tmpdir(), "og-mount-platform-"));
  temps.push(dir);
  const prod = join(dir, PROD);
  mkdirSync(join(prod, "game/skills/someone-else"), { recursive: true });
  writeFileSync(join(prod, "game/skills/someone-else/SKILL.md"), "---\nname: someone-else\ndescription: not ours\n---\n");
  mkdirSync(join(prod, "game-scaffold/three"), { recursive: true });
  writeFileSync(join(prod, "game-scaffold/three/package.json"), "{}\n");
  return dir;
}

function mount(checkout: string, ...args: string[]) {
  const r = spawnSync("bun", [SCRIPT, checkout, ...args], { encoding: "utf8" });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

const real = payload(defaultRoots());
const text = real.filter((f) => isText(f.bytes));

test("a mount writes the text payload byte for byte, no binaries, and leaves other skills and overlays alone", () => {
  const p = platform();
  const r = mount(p);
  expect(r.code).toBe(0);
  for (const f of text) expect(Buffer.compare(readFileSync(join(p, PROD, f.path)), Buffer.from(f.bytes))).toBe(0);
  for (const f of real.filter((f) => !isText(f.bytes))) expect(existsSync(join(p, PROD, f.path))).toBeFalse();
  expect(readFileSync(join(p, PROD, "game/skills/someone-else/SKILL.md"), "utf8")).toContain("not ours");
  expect(readFileSync(join(p, PROD, "game-scaffold/three/package.json"), "utf8")).toBe("{}\n");
  expect(r.out).toContain(`${text.length} added, 0 changed, 0 removed`);
});

test("the overlay carries the template's Git attributes as .gitattributes", () => {
  const p = platform();
  expect(mount(p).code).toBe(0);
  const overlay = join(p, PROD, "game-scaffold/origin-engine-ts");
  expect(readFileSync(join(overlay, ".gitattributes"), "utf8").split("\n")).toContain("*.glb filter=lfs diff=lfs merge=lfs -text");
  expect(existsSync(join(overlay, "gitattributes"))).toBeFalse();
});

test("a second mount changes nothing and --check passes", () => {
  const p = platform();
  mount(p);
  const again = mount(p);
  expect(again.code).toBe(0);
  expect(again.out).toContain("0 added, 0 changed, 0 removed");
  expect(mount(p, "--check").code).toBe(0);
});

test("--check names an edited, a missing and a stray file and writes nothing; a mount repairs all three", () => {
  const p = platform();
  mount(p);
  const prod = join(p, PROD);
  writeFileSync(join(prod, "game/skills/og-core/SKILL.md"), "edited by hand\n");
  rmSync(join(prod, "game-scaffold/origin-engine-ts/project.ogt"));
  writeFileSync(join(prod, "game/skills/og-game/reference/retired.md"), "a page the wiki no longer has\n");
  const check = mount(p, "--check");
  expect(check.code).toBe(1);
  expect(check.out).toContain("changed game/skills/og-core/SKILL.md");
  expect(check.out).toContain("added   game-scaffold/origin-engine-ts/project.ogt");
  expect(check.out).toContain("removed game/skills/og-game/reference/retired.md");
  expect(readFileSync(join(prod, "game/skills/og-core/SKILL.md"), "utf8")).toBe("edited by hand\n");
  expect(mount(p).code).toBe(0);
  expect(mount(p, "--check").code).toBe(0);
  expect(existsSync(join(prod, "game/skills/og-game/reference/retired.md"))).toBeFalse();
});

test("an invalid package mounts nothing", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-mount-invalid-"));
  temps.push(dir);
  // A copy of the engine repository's mounted sources with a broken skill.
  const roots = defaultRoots();
  cpSync(roots.pkg, join(dir, "packages/knowledge"), { recursive: true, filter: (s) => !s.includes("node_modules") });
  cpSync(roots.template, join(dir, "templates/game"), { recursive: true, filter: (s) => !/\/(cooked|dist|node_modules|og-out)(\/|$)/.test(s) });
  mkdirSync(join(dir, "packages/engine"), { recursive: true });
  cpSync(join(roots.engine, "package.json"), join(dir, "packages/engine/package.json"));
  const skill = join(dir, "packages/knowledge/skills/og-core/SKILL.md");
  writeFileSync(skill, readFileSync(skill, "utf8").replace("name: og-core", "name: core"));
  const p = platform();
  const r = spawnSync("bun", [join(dir, "packages/knowledge/scripts/mount.ts"), p], { encoding: "utf8" });
  expect(r.status).toBe(1);
  expect(r.stderr).toContain("nothing mounted");
  expect(existsSync(join(p, PROD, "game/skills/og-core"))).toBeFalse();
  expect(existsSync(join(p, PROD, "game-scaffold/origin-engine-ts"))).toBeFalse();
});

test("a path that is not a platform checkout is refused", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-mount-empty-"));
  temps.push(dir);
  const r = mount(dir);
  expect(r.code).toBe(1);
  expect(r.err).toContain("pass the root of an origin-agent checkout");
});

test("an installed engine's knowledge is read in place: installed(dir).roots through payload", () => {
  // The published layout: @origingame/engine with the knowledge as knowledge/ and the template's text in its scaffold/template/.
  const dir = mkdtempSync(join(tmpdir(), "og-mount-installed-"));
  temps.push(dir);
  const roots = defaultRoots();
  const version = (JSON.parse(readFileSync(join(roots.engine, "package.json"), "utf8")) as { version: string }).version;
  writeFileSync(join(dir, "package.json"), JSON.stringify({ name: "@origingame/engine", version }));
  cpSync(roots.pkg, join(dir, "knowledge"), { recursive: true, filter: (s) => !s.includes("node_modules") });
  cpSync(roots.template, join(dir, "knowledge/scaffold/template"), { recursive: true, filter: (s) => !/\/(cooked|dist|node_modules|og-out)(\/|$)/.test(s) });
  const found = installed(dir);
  expect(found.roots).toEqual({ pkg: join(dir, "knowledge"), template: join(dir, "knowledge/scaffold/template"), engine: dir });
  expect(found.revision).toBe(`@origingame/engine@${version} (knowledge)`);
  const files = payload(found.roots);
  for (const f of files) expect(Object.keys(f).sort()).toEqual(["bytes", "path", "source", "symlink"]);
  expect(files.map((f) => f.path)).toEqual([...files.map((f) => f.path)].sort());
  // Every skill at game/skills/<name>/, with the files beside its SKILL.md.
  const script = files.find((f) => f.path === "game/skills/level-texturing/scripts/texture-level.ts")!;
  expect(script.source).toBe(join(dir, "knowledge/skills/level-texturing/scripts/texture-level.ts"));
  expect(Buffer.from(script.bytes).equals(readFileSync(join(roots.pkg, "skills/level-texturing/scripts/texture-level.ts")))).toBeTrue();
  const skills = new Set(files.map((f) => /^game\/skills\/([^/]+)\/SKILL\.md$/.exec(f.path)?.[1]).filter(Boolean));
  for (const s of ["architecture-discipline", "component-forge", "level-texturing", "localization", "recipe-craft", "visual-acceptance", "og-game"]) expect(skills.has(s)).toBeTrue();
  // The same text as the repository's payload, path for path.
  const here = new Map(text.map((f) => [f.path, Buffer.from(f.bytes).toString("utf8")]));
  const there = new Map(files.filter((f) => isText(f.bytes)).map((f) => [f.path, Buffer.from(f.bytes).toString("utf8")]));
  expect([...there.keys()]).toEqual([...here.keys()]);
});
