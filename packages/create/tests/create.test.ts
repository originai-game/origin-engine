// The scaffolder through its command line: a fresh directory gets every
// template file with the game's name and id, a directory that is not empty is
// refused unless --force, and build output is never copied.
import { afterAll, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const template = join(pkg, "..", "..", "templates", "game");
const scratch = mkdtempSync(join(tmpdir(), "og-create-"));
afterAll(() => rmSync(scratch, { recursive: true, force: true }));

function cli(...args: string[]) {
  const r = Bun.spawnSync(["bun", join(pkg, "bin", "create.ts"), ...args], { cwd: scratch });
  return { code: r.exitCode, out: new TextDecoder().decode(r.stdout), err: new TextDecoder().decode(r.stderr) };
}

function walk(root: string, at = root): string[] {
  return readdirSync(at).flatMap((e) => {
    const p = join(at, e);
    return statSync(p).isDirectory() ? walk(root, p) : [relative(root, p)];
  });
}

// The template keeps its Git attributes as `gitattributes` (this repository
// stores media as plain blobs); a scaffold gets them as `.gitattributes`.
const scaffolded = (f: string) => (f === "gitattributes" ? ".gitattributes" : f);
const sourceFiles = walk(template).filter((f) => !/^(cooked|dist|og-out|node_modules|\.og)\//.test(f)).map(scaffolded).sort();

function git(cwd: string, ...args: string[]): string {
  const r = Bun.spawnSync(["git", ...args], { cwd });
  if (r.exitCode !== 0) throw new Error(`git ${args.join(" ")}: ${new TextDecoder().decode(r.stderr)}`);
  return new TextDecoder().decode(r.stdout);
}

test("scaffolds every template file with the game's name and id", () => {
  const r = cli("space-run", "--name", "Space Run");
  expect(r.code).toBe(0);
  expect(r.out).toContain("og run .");
  const dir = join(scratch, "space-run");
  expect(walk(dir).sort()).toEqual(sourceFiles);
  expect(sourceFiles).toContain("project.ogt");
  expect(sourceFiles.some((f) => f.startsWith("assets/"))).toBeTrue();
  const manifest = readFileSync(join(dir, "project.ogt"), "utf8");
  expect(manifest.split("\n")).toContain('name "Space Run"');
  expect(manifest.split("\n")).toContain('id "space-run"');
  expect(manifest.startsWith("type project\n")).toBeTrue();
  // Everything but the name and id lines is the template's.
  const strip = (t: string) => t.split("\n").filter((l) => !/^(name|id) "/.test(l));
  expect(strip(manifest)).toEqual(strip(readFileSync(join(template, "project.ogt"), "utf8")));
  expect(JSON.parse(readFileSync(join(dir, "package.json"), "utf8")).name).toBe("space-run");
});

test("the id defaults from the name, the name from the id", () => {
  expect(cli("a", "--name", "Ünïcode Gäme 2!").code).toBe(0);
  expect(readFileSync(join(scratch, "a", "project.ogt"), "utf8")).toContain('id "unicode-game-2"');
  expect(cli("b", "--id", "tiny-tower").code).toBe(0);
  expect(readFileSync(join(scratch, "b", "project.ogt"), "utf8")).toContain('name "Tiny Tower"');
  const bad = cli("c", "--id", "Not An Id");
  expect(bad.code).not.toBe(0);
  expect(bad.err).toContain("not a game id");
});

test("a directory that is not empty is refused unless --force", () => {
  const dir = join(scratch, "taken");
  mkdirSync(dir);
  writeFileSync(join(dir, "notes.txt"), "mine");
  const refused = cli("taken", "--name", "Taken");
  expect(refused.code).not.toBe(0);
  expect(refused.err).toContain("--force");
  expect(existsSync(join(dir, "project.ogt"))).toBeFalse();
  const forced = cli("taken", "--name", "Taken", "--force");
  expect(forced.code).toBe(0);
  expect(readFileSync(join(dir, "project.ogt"), "utf8")).toContain('name "Taken"');
  expect(readFileSync(join(dir, "notes.txt"), "utf8")).toBe("mine");
  // An existing empty directory is fine without --force.
  mkdirSync(join(scratch, "empty"));
  expect(cli("empty", "--name", "Empty").code).toBe(0);
});

test("build output in the template is never copied", () => {
  // A template with cooked/, dist/ and node_modules/ next to its sources.
  const fake = join(scratch, "fake-template");
  for (const d of ["cooked", "dist", "node_modules/x", "src"]) mkdirSync(join(fake, d), { recursive: true });
  writeFileSync(join(fake, "project.ogt"), 'type project\nversion 1\n---\nname "T"\nid "t"\n');
  for (const f of ["cooked/a.ogb", "dist/game.js", "node_modules/x/i.js", "src/main.ts"]) writeFileSync(join(fake, f), "x");
  const r = Bun.spawnSync(["bun", "-e", `import { create } from ${JSON.stringify(join(pkg, "bin", "create.ts"))}; create({ dir: ${JSON.stringify(join(scratch, "clean"))}, name: "Clean", template: ${JSON.stringify(fake)} });`]);
  expect(r.exitCode).toBe(0);
  expect(walk(join(scratch, "clean")).sort()).toEqual(["project.ogt", "src/main.ts"]);
});

test("--archetype scaffolds an archetype game instead of the template", () => {
  const archetype = join(pkg, "..", "..", "templates", "archetypes", "runner");
  const r = cli("dash", "--archetype", "runner", "--name", "Dash");
  expect(r.code).toBe(0);
  const want = walk(archetype).filter((f) => !/^(cooked|dist|node_modules|og-out|\.og)\//.test(f)).map(scaffolded).sort();
  expect(walk(join(scratch, "dash")).sort()).toEqual(want);
  expect(readFileSync(join(scratch, "dash", "project.ogt"), "utf8")).toContain('name "Dash"');
  const unknown = cli("nope", "--archetype", "no-such-game");
  expect(unknown.code).not.toBe(0);
  expect(unknown.err).toContain("runner");
});

test("a scaffolded project routes media through Git LFS; the engine repository does not", () => {
  const repo = join(pkg, "..", "..");
  for (const [dir, args] of [
    ["lfs-game", ["--name", "LFS Game"]],
    ["lfs-runner", ["--archetype", "runner"]],
    ["lfs-match3", ["--archetype", "match3"]],
    ["lfs-topdown", ["--archetype", "topdown"]],
  ] as const) {
    expect(cli(dir, ...args).code).toBe(0);
    const made = join(scratch, dir);
    git(made, "init", "-q");
    for (const media of ["a.glb", "assets/b.png", "c.wav", "d.ttf"]) {
      expect(git(made, "check-attr", "filter", media).trim()).toBe(`${media}: filter: lfs`);
    }
    expect(git(made, "check-attr", "filter", "project.ogt").trim()).toBe("project.ogt: filter: unspecified");
    expect(existsSync(join(made, "gitattributes"))).toBeFalse();
  }
  for (const media of ["templates/game/assets/models/dropship.glb", "templates/archetypes/runner/assets/x.png", "templates/archetypes/topdown/a.wav"]) {
    expect(git(repo, "check-attr", "filter", media).trim()).toBe(`${media}: filter: unspecified`);
  }
});
