// The scaffolder through its command line: a fresh directory gets every
// template file with the game's name and id, a directory that is not empty is
// refused unless --force, and build output is never copied.
import { afterAll, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { pickStyle, premiseProfile, readCatalogue, words } from "../bin/create.ts";
import { fileURLToPath } from "node:url";
import { SKIN_ELSEWHERE } from "../../knowledge/scripts/validate.ts";

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
const copied = (root: string) => walk(root).filter((f) => !/^(cooked|dist|og-out|node_modules|\.og)\//.test(f)).map(scaffolded);
/** What a scaffold of `root` holds: its files, its look renamed to the game's id (or added), and a README. */
function expected(root: string, id: string): string[] {
  const files = copied(root).filter((f) => !f.startsWith("assets/looks/"));
  return [...new Set([...files, `assets/looks/${id}.ogt`, "README.md"])].sort();
}
const sourceFiles = copied(template).sort();

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
  expect(walk(dir).sort()).toEqual(expected(template, "space-run"));
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
  expect(walk(join(scratch, "clean")).sort()).toEqual(["README.md", "assets/looks/clean.ogt", "project.ogt", "src/main.ts"]);
});

test("--archetype scaffolds an archetype game instead of the template", () => {
  const archetype = join(pkg, "..", "..", "templates", "archetypes", "runner");
  const r = cli("dash", "--archetype", "runner", "--name", "Dash");
  expect(r.code).toBe(0);
  expect(walk(join(scratch, "dash")).sort()).toEqual(expected(archetype, "dash"));
  expect(readFileSync(join(scratch, "dash", "project.ogt"), "utf8")).toContain('name "Dash"');
  const unknown = cli("nope", "--archetype", "no-such-game");
  expect(unknown.code).not.toBe(0);
  expect(unknown.err).toContain("runner");
});

test("a scaffolded project routes media through Git LFS; the engine repository does not", () => {
  const repo = join(pkg, "..", "..");
  for (const [dir, args] of [
    ["lfs-game", ["--name", "LFS Game"]],
    ["lfs-game-2d", ["--profile", "2d"]],
    ["lfs-runner", ["--archetype", "runner"]],
    ["lfs-match3", ["--archetype", "match3"]],
    ["lfs-topdown", ["--archetype", "topdown"]],
    ["lfs-tower-defense", ["--archetype", "tower-defense"]],
    ["lfs-roguelike", ["--archetype", "roguelike"]],
    ["lfs-idle", ["--archetype", "idle"]],
    ["lfs-rhythm", ["--archetype", "rhythm"]],
    ["lfs-third-person", ["--archetype", "third-person"]],
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
  for (const media of ["templates/game/assets/models/dropship.glb", "templates/game-2d/assets/sprites/hero.png", "templates/archetypes/runner/assets/x.png", "templates/archetypes/topdown/a.wav"]) {
    expect(git(repo, "check-attr", "filter", media).trim()).toBe(`${media}: filter: unspecified`);
  }
  // Nine scaffolds and some eighty git spawns: 1.2 s alone, past bun's 5 s
  // default beside the gate's cargo step.
}, 60_000);

// The profile: --profile 2d writes the 2D starter (templates/game-2d), a
// premise that says 2D picks it, and the 3D template is the default.

test("--profile 2d scaffolds the 2D starter with its look renamed; the output names the profile", () => {
  const template2d = join(pkg, "..", "..", "templates", "game-2d");
  const r = cli("hop", "--profile", "2d", "--name", "Hop", "--no-prompt");
  expect(r.code).toBe(0);
  expect(r.out).toContain("Profile: 2d (--profile)");
  expect(walk(join(scratch, "hop")).sort()).toEqual(expected(template2d, "hop"));
  const manifest = readFileSync(join(scratch, "hop", "project.ogt"), "utf8");
  expect(manifest.split("\n")).toContain('profile "2d"');
  expect(manifest.split("\n")).toContain('name "Hop"');
  expect(manifest.split("\n")).toContain('id "hop"');
  expect(JSON.parse(readFileSync(join(scratch, "hop", "package.json"), "utf8")).name).toBe("hop");
  const look = readFileSync(join(scratch, "hop", "assets", "looks", "hop.ogt"), "utf8");
  expect(new RegExp('^extends "([^"]*)"$', "m").exec(look)?.[1]).toBe("16bit-pixel");
  expect(existsSync(join(scratch, "hop", "assets", "looks", "origin-game-2d.ogt"))).toBeFalse();
  expect([...readFileSync(join(scratch, "hop", "src", "main.ts"), "utf8").matchAll(/^\s*look: "([^"]+)",$/gm)].map((m) => m[1])).toEqual(["looks/hop.ogt"]);
  // --profile 3d and no flag both write the 3D template.
  const three = cli("flat", "--profile", "3d", "--no-prompt");
  expect(three.code).toBe(0);
  expect(three.out).toContain("Profile: 3d (--profile)");
  expect(walk(join(scratch, "flat")).sort()).toEqual(expected(template, "flat"));
  const plain = cli("plain", "--no-prompt");
  expect(plain.code).toBe(0);
  expect(plain.out).toContain("Profile: 3d.\n");
  expect(readFileSync(join(scratch, "plain", "project.ogt"), "utf8")).toContain('profile "3d"');
  // An archetype has its own profile; --profile beside it is refused before anything is written.
  expect(cli("dash2", "--archetype", "runner", "--no-prompt").out).toContain("Profile: 2d (the runner archetype's)");
  const both = cli("both", "--profile", "2d", "--archetype", "runner");
  expect(both.code).not.toBe(0);
  expect(both.err).toContain("--profile and --archetype together");
  expect(existsSync(join(scratch, "both"))).toBeFalse();
  const bad = cli("bad", "--profile", "4d");
  expect(bad.code).not.toBe(0);
  expect(bad.err).toContain("use 2d or 3d");
});

test("a premise that says 2D picks the 2D starter unless it also says 3D", () => {
  const cave = cli("cave", "--premise", "A 16-bit metroidvania platformer, energetic and heroic");
  expect(cave.code).toBe(0);
  expect(cave.out).toContain("Profile: 2d (from the premise)");
  expect(cave.out).toContain("extends 16bit-pixel (from the premise)");
  expect(readFileSync(join(scratch, "cave", "project.ogt"), "utf8")).toContain('profile "2d"');
  const voxel = cli("cave3", "--premise", "A 3D platformer with pixel textures");
  expect(voxel.code).toBe(0);
  expect(voxel.out).toContain("Profile: 3d.\n");
  expect(readFileSync(join(scratch, "cave3", "project.ogt"), "utf8")).toContain('profile "3d"');
  expect(premiseProfile("Cozy Side-Scroller about a lost kite")).toBe("2d");
  expect(premiseProfile("a 2D puzzle game")).toBe("2d");
  expect(premiseProfile("a sprite-based roguelike in first-person")).toBeNull();
  expect(premiseProfile("a survival horror in lonely corridors")).toBeNull();
  expect(premiseProfile("")).toBeNull();
});

// The look: every game gets assets/looks/<id>.ogt, named by GameDesc.look.

const read = (...p: string[]) => readFileSync(join(scratch, ...p), "utf8");
const topLevel = (text: string, key: string) => new RegExp(`^${key} "([^"]*)"$`, "m").exec(text)?.[1] ?? null;
const lookKeys = (main: string) => [...main.matchAll(/^\s*look: "([^"]+)",$/gm)].map((m) => m[1]);
const styles = readCatalogue();

test("the template's look becomes the game's: renamed to its id, named by GameDesc.look, explained in the README", () => {
  expect(cli("lantern", "--name", "Lantern", "--no-prompt").code).toBe(0);
  const look = read("lantern", "assets", "looks", "lantern.ogt");
  expect(look.startsWith("type look\nversion 1\n---\n")).toBeTrue();
  expect(topLevel(look, "name")).toBe("Lantern");
  // templates/game/assets/looks/origin-game.ogt's preset, kept when no premise or --look is given.
  expect(topLevel(look, "extends")).toBe(topLevel(readFileSync(join(template, "assets", "looks", "origin-game.ogt"), "utf8"), "extends"));
  expect(existsSync(join(scratch, "lantern", "assets", "looks", "origin-game.ogt"))).toBeFalse();
  expect(lookKeys(read("lantern", "src", "main.ts"))).toEqual(["looks/lantern.ogt"]);
  const readme = read("lantern", "README.md");
  expect(readme).toContain("## The look");
  expect(readme).toContain("`assets/looks/lantern.ogt`");
  expect(readme).toContain('look: "looks/lantern.ogt"');
  expect(readme).toContain("extends \"<slug>\"");
});

test("--look extends the catalogue preset it names; an unknown slug is refused with the slugs", () => {
  const r = cli("cel", "--look", "anime-cel-3d");
  expect(r.code).toBe(0);
  expect(r.out).toContain("extends anime-cel-3d (--look)");
  const look = read("cel", "assets", "looks", "cel.ogt");
  expect(topLevel(look, "extends")).toBe("anime-cel-3d");
  expect(look).toContain("built-in neon style"); // the preset's UI style, as the catalogue gives it
  expect(read("cel", "README.md")).toContain("It extends `anime-cel-3d`, chosen with `--look`");
  const bad = cli("nope-look", "--look", "photoreal-ultra");
  expect(bad.code).not.toBe(0);
  expect(bad.err).toContain("no look preset 'photoreal-ultra'");
  expect(bad.err).toContain("16bit-pixel");
  expect(existsSync(join(scratch, "nope-look"))).toBeFalse(); // refused before anything is written
});

test("--premise picks the catalogue style whose premise fits", () => {
  const r = cli("synth", "--premise", "A neon arcade shooter: electric, hypnotic synthwave racing");
  expect(r.code).toBe(0);
  expect(r.out).toContain("extends neon-vector (from the premise)");
  expect(topLevel(read("synth", "assets", "looks", "synth.ogt"), "extends")).toBe("neon-vector");
  expect(read("synth", "README.md")).toContain("chosen from the game's premise");
  // No word of the premise matches a style: the template's look stays, and the output says why.
  const none = cli("vague", "--premise", "zzz qqq");
  expect(none.code).toBe(0);
  expect(none.out).toContain("no catalogue style matched the premise");
  expect(topLevel(read("vague", "assets", "looks", "vague.ogt"), "extends")).toBe("faceted-low-poly");
});

test("pickStyle weighs genre over mood over the rest, prefers full support on a tie, and never picks a style without a preset", () => {
  const pick = (p: string) => pickStyle(p, styles)?.style.slug ?? null;
  expect(pick("a cosy city builder with a tabletop charm")).toBe("miniature-diorama");
  expect(pick("survival horror with lonely, uncanny corridors")).toBe("low-resolution-3d");
  expect(pick("16-bit platformer, energetic and heroic")).toBe("16bit-pixel");
  expect(pick("a sandbox about building and destruction")).toBe("voxel-worlds");
  expect(pick("Cozy Puzzles")).not.toBeNull();
  expect(pick("")).toBeNull();
  expect(words("Cozy Platformers in Sci-Fi colors, hand-drawn")).toEqual(["cosy", "platformer", "sci", "colour", "hand-drawn", "hand", "drawn"]);
  const noPresets = styles.map((s) => ({ ...s, look: { ...s.look, presetReady: false } }));
  expect(pickStyle("neon arcade shooter", noPresets)).toBeNull();
  // Every slug the catalogue offers is one --look accepts, and each has its preset in templates/looks/.
  for (const s of styles) expect(existsSync(join(pkg, "..", "..", "templates", "looks", `${s.slug}.ogt`))).toBe(s.look.presetReady);
});

test("an archetype without a look gets one: GameDesc.look inserted into og.game({...}), its frames untouched until a preset is chosen", () => {
  expect(cli("run2", "--archetype", "runner", "--no-prompt").code).toBe(0);
  const main = read("run2", "src", "main.ts");
  expect(lookKeys(main)).toEqual(["looks/run2.ogt"]);
  expect(main).toMatch(/export default og\.game\(\{\n  \/\/ The art direction as data[^\n]*\n  look: "looks\/run2\.ogt",\n/);
  const look = read("run2", "assets", "looks", "run2.ogt");
  expect(topLevel(look, "extends")).toBeNull();
  expect(look.split("\n").filter((l) => l !== "" && !l.startsWith("#"))).toEqual(["type look", "version 1", "---", 'name "Run2"']);
  // With a preset, the reference frames will move: the output and the README say so.
  const r = cli("run3", "--archetype", "runner", "--look", "16bit-pixel");
  expect(r.code).toBe(0);
  expect(r.out).toContain("the frames in references/ show the archetype's own look; with 16bit-pixel they will differ");
  expect(read("run3", "README.md")).toContain("OG_FX_CAPTURE=1 og test .");
});

test("the idle archetype's look carries over: its UI style and render settings stay, a preset goes under them", () => {
  expect(cli("orb", "--archetype", "idle").code).toBe(0);
  const plain = read("orb", "assets", "looks", "orb.ogt");
  expect(topLevel(plain, "ui")).toBe("styles/sparkorb/style.ogt");
  expect(plain).toContain("RenderSettings {\n  toneMapped false\n  environmentIntensity 0\n}");
  expect(topLevel(plain, "extends")).toBeNull();
  expect(existsSync(join(scratch, "orb", "assets", "looks", "idle.ogt"))).toBeFalse();
  expect(lookKeys(read("orb", "src", "main.ts"))).toEqual(["looks/orb.ogt"]);
  expect(cli("orb2", "--archetype", "idle", "--look", "luminous-pixel").code).toBe(0);
  const lit = read("orb2", "assets", "looks", "orb2.ogt");
  expect(topLevel(lit, "extends")).toBe("luminous-pixel");
  expect(topLevel(lit, "ui")).toBe("styles/sparkorb/style.ogt");
  expect(lit.indexOf('extends "luminous-pixel"')).toBeLessThan(lit.indexOf("RenderSettings {"));
});

test("what a scaffold ships never calls a UI style a skin", () => {
  // og-ui's word is style (type style, og.load.style, UiDesc.style,
  // styles/<name>/). A sentence of code or prose that says skin beside the
  // UI's words (og.ui, ui, style, entrance, theme) is that use, unless it
  // names one of the word's other meanings (knowledge's SKIN_ELSEWHERE: a
  // glTF rig, the controllers' collision gap, a surface).
  const roots = ["game", "game-2d", "archetypes"].map((d) => join(pkg, "..", "..", "templates", d));
  const found: string[] = [];
  for (const root of roots) {
    for (const f of copied(root).filter((f) => /\.(ts|md|ogt|py)$/.test(f))) {
      for (const paragraph of readFileSync(join(root, f), "utf8").split(/\n\s*\n/)) {
        for (const s of paragraph.replace(/\s*\n\s*/g, " ").split(/(?<=[.;!?])\s+/)) {
          if (/\bskins?\b/i.test(s) && /\b(og\.ui|ui|styles?|entrance|theme)\b/i.test(s) && !SKIN_ELSEWHERE.test(s)) found.push(`${relative(join(pkg, "..", ".."), join(root, f))}: ${s.trim().slice(0, 100)}`);
        }
      }
    }
  }
  expect(found).toEqual([]);
});
