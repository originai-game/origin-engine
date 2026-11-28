// The mount writes the package into the platform's knowledge tree and owns
// only what it mounts. Each test runs the CLI against a stand-in origin-agent
// checkout (a knowledge/ tree with the platform's own skill and overlay) and
// asserts on the files it leaves and the exit status --check returns.

import { afterAll, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defaultRoots, isText, payload } from "../scripts/lib.ts";
import { installed, mount as mountPayload, summary } from "../scripts/mount.ts";

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
  mkdirSync(join(prod, "game/vfx/platform/empty"), { recursive: true });
  writeFileSync(join(prod, "game/vfx/platform/catalogue.json"), "platform-owned\n");
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

test("a mount writes the text payload byte for byte, no binaries, and leaves the platform's skills and other overlays alone", () => {
  const p = platform();
  const r = mount(p);
  expect(r.code).toBe(0);
  for (const f of text) expect(Buffer.compare(readFileSync(join(p, PROD, f.path)), Buffer.from(f.bytes))).toBe(0);
  for (const f of real.filter((f) => !isText(f.bytes))) expect(existsSync(join(p, PROD, f.path))).toBeFalse();
  expect(readFileSync(join(p, PROD, "game/skills/someone-else/SKILL.md"), "utf8")).toContain("not ours");
  expect(readFileSync(join(p, PROD, "game/vfx/platform/catalogue.json"), "utf8")).toBe("platform-owned\n");
  expect(existsSync(join(p, PROD, "game/vfx/platform/empty"))).toBeTrue();
  expect(readFileSync(join(p, PROD, "game-scaffold/three/package.json"), "utf8")).toBe("{}\n");
  expect(r.out).toContain("owned: game-scaffold/origin-engine-ts/, game/vfx/catalogue.json\n");
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
  const overlay = "game-scaffold/origin-engine-ts";
  writeFileSync(join(prod, overlay, "package.json"), "edited by hand\n");
  rmSync(join(prod, overlay, "og-project.json"));
  writeFileSync(join(prod, overlay, "retired.md"), "a file the package no longer has\n");
  writeFileSync(join(prod, "game/vfx/catalogue.json"), "edited catalogue\n");
  const check = mount(p, "--check");
  expect(check.code).toBe(1);
  expect(check.out).toContain(`changed ${overlay}/package.json`);
  expect(check.out).toContain(`added   ${overlay}/og-project.json`);
  expect(check.out).toContain(`removed ${overlay}/retired.md`);
  expect(check.out).toContain("changed game/vfx/catalogue.json");
  expect(readFileSync(join(prod, overlay, "package.json"), "utf8")).toBe("edited by hand\n");
  expect(readFileSync(join(prod, "game/vfx/catalogue.json"), "utf8")).toBe("edited catalogue\n");
  expect(mount(p).code).toBe(0);
  expect(mount(p, "--check").code).toBe(0);
  expect(existsSync(join(prod, overlay, "retired.md"))).toBeFalse();
  rmSync(join(prod, "game/vfx/catalogue.json"));
  const missing = mount(p, "--check");
  expect(missing.code).toBe(1);
  expect(missing.out).toContain("added   game/vfx/catalogue.json");
  expect(existsSync(join(prod, "game/vfx/catalogue.json"))).toBeFalse();
  expect(mount(p).code).toBe(0);
  expect(mount(p, "--check").code).toBe(0);
});

test("an invalid package mounts nothing", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-mount-invalid-"));
  temps.push(dir);
  // A copy of the engine repository's mounted sources with a broken skeleton.
  const roots = defaultRoots();
  cpSync(roots.pkg, join(dir, "packages/knowledge"), { recursive: true, filter: (s) => !s.includes("node_modules") });
  cpSync(roots.template, join(dir, "packages/create/template"), { recursive: true, filter: (s) => !/\/(cooked|dist|node_modules|og-out)(\/|$)/.test(s) });
  mkdirSync(join(dir, "packages/engine"), { recursive: true });
  cpSync(join(roots.engine, "package.json"), join(dir, "packages/engine/package.json"));
  const project = join(dir, "packages/create/template/og-project.json");
  writeFileSync(project, JSON.stringify({version: 1, engine: "0.9.0"}));
  const p = platform();
  const r = spawnSync("bun", [join(dir, "packages/knowledge/scripts/mount.ts"), p], { encoding: "utf8" });
  expect(r.status).toBe(1);
  expect(r.stderr).toContain("nothing mounted");
  expect(existsSync(join(p, PROD, "game-scaffold/origin-engine-ts"))).toBeFalse();
  expect(existsSync(join(p, PROD, "game/vfx/catalogue.json"))).toBeFalse();
});

test("a path that is not a platform checkout is refused", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-mount-empty-"));
  temps.push(dir);
  const r = mount(dir);
  expect(r.code).toBe(1);
  expect(r.err).toContain("pass the root of an origin-agent checkout");
  mkdirSync(join(dir, PROD, "game-scaffold"), { recursive: true });
  expect(mount(dir).err).toContain("no game/");
  rmSync(join(dir, PROD, "game-scaffold"), { recursive: true });
  mkdirSync(join(dir, PROD, "game"), { recursive: true });
  expect(mount(dir).err).toContain("no game-scaffold/");
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
  // The overlay's files, each from the package.
  const pin = files.find((f) => f.path === "game-scaffold/origin-engine-ts/origin-engine.json")!;
  expect(pin.source).toBe(join(dir, "knowledge/scaffold/origin-engine.json"));
  expect(Buffer.from(pin.bytes).equals(readFileSync(join(roots.pkg, "scaffold/origin-engine.json")))).toBeTrue();
  const catalogue = files.find((f) => f.path === "game/vfx/catalogue.json")!;
  expect(catalogue.source).toBe(join(dir, "knowledge/vfx/catalogue.json"));
  expect(Buffer.from(catalogue.bytes).equals(readFileSync(join(roots.pkg, "vfx/catalogue.json")))).toBeTrue();
  // No skills: the engine's knowledge reaches an agent through og api.
  expect(files.some((f) => f.path.startsWith("game/skills/"))).toBeFalse();
  // The same text as the repository's payload, path for path.
  const here = new Map(text.map((f) => [f.path, Buffer.from(f.bytes).toString("utf8")]));
  const there = new Map(files.filter((f) => isText(f.bytes)).map((f) => [f.path, Buffer.from(f.bytes).toString("utf8")]));
  expect([...there.keys()]).toEqual([...here.keys()]);

  // Exercise the installed package API used by mount-engine.ts, including
  // retiring the catalogue without claiming either of its parent directories.
  const prod = join(platform(), PROD);
  const first = mountPayload(found.roots, prod, true);
  expect(first.owned).toEqual(["game-scaffold/origin-engine-ts/", "game/vfx/catalogue.json"]);
  expect(first.files).toBe(text.length);
  expect(summary(first, found.revision, false)).toContain("added   game/vfx/catalogue.json");
  expect(mountPayload(found.roots, prod, false).changes).toEqual([]);
  const manifestPath = join(found.roots.pkg, "knowledge.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  manifest.mounts = manifest.mounts.filter((m: { to: string }) => m.to !== "game/vfx/catalogue.json");
  writeFileSync(manifestPath, JSON.stringify(manifest));
  expect(payload(found.roots).some(f => f.path === "game/vfx/catalogue.json")).toBeFalse();
  const stale = mountPayload(found.roots, prod, false);
  expect(stale.changes).toEqual([{ kind: "removed", path: "game/vfx/catalogue.json" }]);
  expect(stale.files).toBe(text.length - 1);
  expect(summary(stale, found.revision, true)).toContain("differences: 0 added, 0 changed, 1 removed");
  expect(existsSync(join(prod, "game/vfx/catalogue.json"))).toBeTrue();
  expect(mountPayload(found.roots, prod, true).changes).toEqual(stale.changes);
  expect(existsSync(join(prod, "game/vfx/catalogue.json"))).toBeFalse();
  expect(readFileSync(join(prod, "game/vfx/platform/catalogue.json"), "utf8")).toBe("platform-owned\n");
  expect(existsSync(join(prod, "game/vfx/platform/empty"))).toBeTrue();
  expect(mountPayload(found.roots, prod, false).changes).toEqual([]);
});
