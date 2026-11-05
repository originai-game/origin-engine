// The scaffolder through its command line: a fresh directory gets every
// template file with the game's name and id, a directory that is not empty is
// refused unless --force, and build output is never copied.
import { afterAll, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { artBrief, audioBrief, CREATION_CHOICES, localeScript, pseudoMessage, creationBlock, creationSchema, inferCreationChoices, parseCreationInput, resolveCreationChoices, pickStyle, premiseProfile, presetLook, readCatalogue, readLook, RIG_TEMPLATES, rigParts, words } from "../bin/create.ts";
import { cpSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { SKIN_ELSEWHERE } from "../../knowledge/scripts/validate.ts";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const template = join(pkg, "..", "..", "templates", "game");
// realpath: macOS hands out /var/folders/..., which is /private/var/folders/... once resolved.
const scratch = realpathSync(mkdtempSync(join(tmpdir(), "og-create-")));
afterAll(() => rmSync(scratch, { recursive: true, force: true }));

function cli(...args: string[]) {
  const r = Bun.spawnSync(["bun", join(pkg, "bin", "create.ts"), ...args], { cwd: scratch });
  return { code: r.exitCode, out: new TextDecoder().decode(r.stdout), err: new TextDecoder().decode(r.stderr) };
}

test("creation choices infer a premise, preserve overrides, and serialize every choice", () => {
  const c = resolveCreationChoices("温暖的手机横屏平台跳跃", { mood: "cold", music: "folk", look: "16bit-pixel" });
  expect(c).toEqual({ genre: "action-platformer", view: "side-2d", look: "16bit-pixel", mood: "cold", ui: "pixel", platform: "phone-landscape", input: "touch", players: "single", lang: ["zh"], music: "folk" });
  expect(inferCreationChoices("isometric online shooter", { platform: "desktop", input: "gamepad" })).toMatchObject({ genre: "shooter", view: "isometric-2.5d", players: "online-lockstep", input: "gamepad" });
  expect(resolveCreationChoices("", { look: "16bit-pixel", ui: "dread" }).ui).toBe("dread");
  expect(resolveCreationChoices("", { players: "online-lockstep" })).toMatchObject({ view: "top-down-2d", look: "16bit-pixel", ui: "pixel" });
  expect(creationBlock(c)).toContain('lang "zh"');
  expect(creationSchema(readCatalogue().map(s => s.slug)).properties.look.enum).toHaveLength(37);
  expect(() => parseCreationInput({ players: "mmo" })).toThrow("players");
  expect(() => parseCreationInput({ lang: [] })).toThrow("lang");
  expect(() => parseCreationInput({ lang: ["en", "en"] })).toThrow("lang");
  expect(() => resolveCreationChoices("", { look: "missing" })).toThrow("no look preset");
});

test("choice flags write a canonical create block and reject bad choices before creating", () => {
  const r = cli("choices", "--no-prompt", "--genre", "puzzle-match", "--view", "top-down-2d", "--mood", "soft", "--ui", "pixel", "--platform", "phone-portrait", "--input", "touch", "--players", "single", "--lang", "ja,en", "--music", "chiptune");
  expect(r.code).toBe(0);
  const manifest = readFileSync(join(scratch, "choices/project.ogt"), "utf8");
  expect(manifest).toContain('create {\n');
  expect(manifest).toContain('  genre "puzzle-match"');
  expect(manifest).toContain('  lang "ja" "en"');
  expect(manifest).toContain('  music "chiptune"');
  expect(manifest).toContain('orientation portrait');
  expect(readFileSync(join(scratch, "choices/assets/looks/choices.ogt"), "utf8")).toContain('ui "styles/pixel/style.ogt"');
  expect(existsSync(join(scratch, "choices/assets/styles/pixel/atlas.png"))).toBeTrue();
  expect(cli("bad-choice", "--mood", "invalid").code).toBe(1);
  expect(existsSync(join(scratch, "bad-choice"))).toBeFalse();
  expect(cli("bad-players", "--players", "online-lockstep", "--archetype", "runner").err).toContain("arena");
  const styles = join(pkg, "../../templates/styles");
  expect([...CREATION_CHOICES.ui].sort()).toEqual(readdirSync(styles).filter(s => existsSync(join(styles, s, "style.ogt"))).sort());
});

const engine = join(pkg, "../engine");
const og = join(engine, "native", `og.${process.platform}-${process.arch}${process.platform === "win32" ? ".exe" : ""}`);
test.skipIf(!existsSync(og))("local touch arena keeps the whole field clear of HUD and controls at both aspects", () => {
  expect(cli("arena-fit", "--players", "local-multiplayer", "--platform", "phone-portrait", "--look", "16bit-pixel").code).toBe(0);
  const dir = join(scratch, "arena-fit");
  const sizes: [number, number][] = [[390, 844], [844, 390]];
  for (const [width, height] of sizes) {
    const out = join(dir, `out-${width}`);
    const r = Bun.spawnSync([og, "run", dir, "--commands", "0.3:get:og.Camera:camera.json;0.3:dump_ui:ui.json;0.4:quit", "--size", `${width}x${height}`, "--out", out], { env: { ...process.env, OG_ENGINE_PACKAGE: engine, OG_RENDERER: "none" } });
    expect(r.exitCode).toBe(0);
    const camera = JSON.parse(readFileSync(join(out, "camera.json"), "utf8")).value;
    const pixels = height / camera.height;
    // Arena bounds are 0..16 by 0..9, including both players' full movement range.
    const left = width / 2 - camera.position[0] * pixels;
    const right = left + 16 * pixels;
    const top = height / 2 - (9 - camera.position[1]) * pixels;
    const bottom = top + 9 * pixels;
    expect(left).toBeGreaterThanOrEqual(15);
    expect(right).toBeLessThanOrEqual(width - 15);
    expect(top).toBeGreaterThan(0);
    expect(bottom).toBeLessThan(height);
    const ui = JSON.parse(readFileSync(join(out, "ui.json"), "utf8"));
    for (const path of ["Status/Line", "Score/P1", "Score/P2", "CreationTouch0/Move", "CreationTouch1/Move", "CreationTouch0/Action", "CreationTouch1/Action"]) {
      const rect = ui.elements.find((e: { path: string }) => e.path === path);
      expect(rect).toBeDefined();
      expect(rect.x + rect.width <= left || rect.x >= right || rect.y + rect.height <= top || rect.y >= bottom).toBeTrue();
    }
  }
}, 30000);

test.skipIf(!existsSync(og))("local creation routes asymmetric input to independent players on the native host", () => {
  for (const [input, commands] of [
    ["keyboard-mouse", "0.1:key_down:KeyD;0.1:key_down:ArrowUp;0.5:key_up:KeyD;0.5:key_up:ArrowUp"],
    ["gamepad", "0.1:gamepad:-@0:1,0;0.1:gamepad:-@1:0,-1;0.5:gamepad:-@0:0,0;0.5:gamepad:-@1:0,0"],
  ] as const) {
    expect(cli(`local-${input}`, "--players", "local-multiplayer", "--view", "top-down-2d", "--look", "16bit-pixel", "--input", input).code).toBe(0);
    const dir = join(scratch, `local-${input}`);
    const out = join(dir, "og-out");
    const r = Bun.spawnSync([og, "run", dir, "--commands", `${commands};0.6:dump_entity:fighter-0:red.json;0.6:dump_entity:fighter-1:blue.json;0.7:quit`, "--out", out, "--json"], { env: { ...process.env, OG_ENGINE_PACKAGE: engine, OG_RENDERER: "none" } });
    expect(new TextDecoder().decode(r.stderr)).not.toContain("error");
    expect(r.exitCode).toBe(0);
    const position = (file: string): number[] => JSON.parse(readFileSync(join(out, file), "utf8")).components["og.Transform"].position;
    const red = position("red.json"), blue = position("blue.json");
    expect(red[0]).toBeCloseTo(5.4, 3);
    expect(red[1]).toBeCloseTo(4.5, 3);
    expect(blue[0]).toBeCloseTo(13, 3);
    expect(blue[1]).toBeCloseTo(6.9, 3);
  }
}, 30000);

test.skipIf(!existsSync(og))("phone touch controls move and jump together on the native host", () => {
  expect(cli("touch-input", "--view", "side-2d", "--platform", "phone-portrait", "--look", "16bit-pixel").code).toBe(0);
  const dir = join(scratch, "touch-input"), out = join(dir, "og-out");
  const commands = "0.2:touch:1:down:CreationTouch0/Move/Right;0.2:touch:2:down:CreationTouch0/Action;0.6:get:Game:game.json;0.6:get:og.RenderSettings:quality.json;0.7:quit";
  const r = Bun.spawnSync([og, "run", dir, "--commands", commands, "--size", "390x844", "--out", out], { env: { ...process.env, OG_ENGINE_PACKAGE: engine, OG_RENDERER: "none" } });
  expect(r.exitCode).toBe(0);
  const state = JSON.parse(readFileSync(join(out, "game.json"), "utf8")).value;
  expect(state.touchX).toBe(1);
  expect(state.touchJump).toBeTrue();
  expect(state.heroX).toBeGreaterThan(4);
  expect(state.heroY).toBeGreaterThan(4);
  expect(JSON.parse(readFileSync(join(out, "quality.json"), "utf8")).value.quality).toBe("low");
}, 30000);

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
  // Everything but the name and id lines and the look the game names is the template's.
  expect(manifest).toContain('\nentry "src/main.ts"\n# The game\'s look: its art contract the cook reads, GameDesc.look\'s default.\nlook "looks/space-run.ogt"\n');
  const strip = (t: string) => t.split("\n").filter((l) => !/^(name|id|look) "|^# The game's look:/.test(l));
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
  writeFileSync(join(fake, "project.ogt"), 'type project\nversion 1\n---\nname "T"\nid "t"\nlook "looks/t.ogt"\n');
  for (const f of ["cooked/a.ogb", "dist/game.js", "node_modules/x/i.js", "src/main.ts"]) writeFileSync(join(fake, f), "x");
  const r = Bun.spawnSync(["bun", "-e", `import { create } from ${JSON.stringify(join(pkg, "bin", "create.ts"))}; create({ dir: ${JSON.stringify(join(scratch, "clean"))}, name: "Clean", template: ${JSON.stringify(fake)} });`]);
  expect(r.exitCode).toBe(0);
  expect(walk(join(scratch, "clean")).sort()).toEqual(["README.md", "assets/looks/clean.ogt", "project.ogt", "src/main.ts"]);
  // A template's own project look line is replaced by the game's, not doubled.
  expect(readFileSync(join(scratch, "clean", "project.ogt"), "utf8")).toBe('type project\nversion 1\n---\nname "Clean"\nid "clean"\nlook "looks/clean.ogt"\n');
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
  // project.ogt names the same look, once: the cook reads its art contract.
  expect([...read("lantern", "project.ogt").matchAll(/^look "([^"]+)"$/gm)].map((m) => m[1])).toEqual(["looks/lantern.ogt"]);
  const readme = read("lantern", "README.md");
  expect(readme).toContain("## The look");
  expect(readme).toContain("`assets/looks/lantern.ogt`");
  expect(readme).toContain('look: "looks/lantern.ogt"');
  expect(readme).toContain('`look "looks/lantern.ogt"` in `project.ogt`');
  expect(readme).toContain("extends \"<slug>\"");
});

test("--look extends the catalogue preset it names; an unknown slug is refused with the slugs", () => {
  const r = cli("cel", "--look", "anime-cel-3d");
  expect(r.code).toBe(0);
  expect(r.out).toContain("extends anime-cel-3d (--look)");
  const look = read("cel", "assets", "looks", "cel.ogt");
  expect(topLevel(look, "extends")).toBe("anime-cel-3d");
  expect(look).toContain("built-in cyber style"); // the preset's UI style, as the catalogue gives it
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
  expect(pickStyle("16-bit platformer, energetic and heroic", styles, "2d")?.style.slug).toBe("16bit-pixel");
  expect(pick("a sandbox about building and destruction")).toBe("voxel-worlds");
  expect(pick("Cozy Puzzles")).not.toBeNull();
  expect(pick("")).toBeNull();
  expect(words("Cozy Platformers in Sci-Fi colors, hand-drawn")).toEqual(["cosy", "platformer", "sci", "colour", "hand-drawn", "hand", "drawn"]);
  const noPresets = styles.map((s) => ({ ...s, look: { ...s.look, presetReady: false } }));
  expect(pickStyle("neon arcade shooter", noPresets)).toBeNull();
  // Every slug the catalogue offers is one --look accepts, and each has its preset in templates/looks/.
  for (const s of styles) expect(existsSync(join(pkg, "..", "..", "templates", "looks", `${s.slug}.ogt`))).toBe(s.look.presetReady);
});

test("the premise's style is one drawn for the project's profile; --look still wins (rumpus-isle #1)", () => {
  // rumpus-isle #1: this 3D party premise used to pick rubber-hose-animation,
  // a style of flipbooks (2D only), for the 3D template.
  const party = "a local-multiplayer party game for two to four on one screen: round critters on a little island bump each other into the sea in short minigames; bright, bouncy, cartoon";
  const rubber = styles.find((s) => s.slug === "rubber-hose-animation")!;
  expect(rubber.look.profiles).toEqual(["2d"]);
  const three = pickStyle(party, styles, "3d")!.style;
  expect(three.slug).toBe("sculpted-clay");
  expect(three.look.profiles).toContain("3d");
  expect(pickStyle(party, styles)!.style.slug).toBe(three.slug);
  // The same premise for a 2D game may have it: the profile, not the words, rules it out.
  expect(pickStyle(party, styles, "2d")!.style.slug).toBe("rubber-hose-animation");
  // Nor does a 2D platformer land on a style of lit meshes.
  expect(pickStyle("an adventure platformer of clay figures, comedy", styles, "2d")!.style.look.profiles).toContain("2d");
  const r = cli("party", "--premise", party, "--no-prompt");
  expect(r.code).toBe(0);
  expect(r.out).toContain("Profile: 3d");
  expect(topLevel(read("party", "assets", "looks", "party.ogt"), "extends")).toBe(three.slug);
  // --look wins over the premise and the profile.
  const flag = cli("party2", "--premise", party, "--look", "rubber-hose-animation", "--no-prompt");
  expect(flag.code).toBe(0);
  expect(topLevel(read("party2", "assets", "looks", "party2.ogt"), "extends")).toBe("rubber-hose-animation");
  // Every style says which profiles it draws for.
  for (const s of styles) expect(s.look.profiles.length).toBeGreaterThan(0);
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
  // Each archetype is its own project: its cooked/, .og/ and og-out/ are
  // build output the scaffold leaves out, like the templates' own.
  const templates = join(pkg, "..", "..", "templates");
  const archetypes = join(templates, "archetypes");
  const roots = [
    join(templates, "game"),
    join(templates, "game-2d"),
    ...readdirSync(archetypes)
      .map((a) => join(archetypes, a))
      .filter((a) => existsSync(join(a, "project.ogt"))),
  ];
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

test("the art brief is re-exported: bin/brief.ts is the knowledge package's, and a scaffold's look briefs", () => {
  // A copy, byte for byte (scripts/pack-template.ts writes it): edit packages/knowledge/scripts/brief.ts and run the prepack.
  expect(readFileSync(join(pkg, "bin", "brief.ts"), "utf8")).toBe(readFileSync(join(pkg, "..", "knowledge", "scripts", "brief.ts"), "utf8"));
  // A 2D game on 16bit-pixel (16 px per metre, cells of 16, 32, 48): its 2 m courier is 32 tall, 16 wide, drawn at 1024 / 32 = 32.
  expect(cli("briefed", "--profile", "2d", "--look", "16bit-pixel", "--no-prompt").code).toBe(0);
  const two = readLook(join(scratch, "briefed"));
  expect([two.extends, two.profile]).toEqual(["16bit-pixel", "2d"]);
  expect(artBrief(two, "character", "the courier").image?.grid).toEqual({ width: 16, height: 32, scale: 32 });
  // The 3D template on faceted-low-poly: a prop is a model of at most 200 triangles with a 64-texel texture.
  expect(cli("faceted", "--profile", "3d", "--look", "faceted-low-poly", "--no-prompt").code).toBe(0);
  const three = artBrief(readLook(join(scratch, "faceted")), "prop", "a signpost");
  expect([three.medium, three.model?.faceBudget, three.model?.textureSize]).toEqual(["model", 200, 64]);
});

test("rigParts is re-exported beside the art brief and names the four templates for any other", () => {
  expect([...RIG_TEMPLATES]).toEqual(["humanoid-side", "humanoid-front", "quadruped", "creature"]);
  // The mage's lowest layers (examples/autorig/assets/chars/mage.psd): its legs, image-left first.
  const legs = rigParts("humanoid-front").filter((p) => /^(leg|foot)/.test(p.layer));
  expect(legs.map((p) => [p.layer, p.side, p.bone, p.required])).toEqual([
    ["leg-upper-l", "l", "leg-upper-l", true],
    ["leg-lower-l", "l", "leg-lower-l", true],
    ["foot-l", "l", "foot-l", true],
    ["leg-upper-r", "r", "leg-upper-r", true],
    ["leg-lower-r", "r", "leg-lower-r", true],
    ["foot-r", "r", "foot-r", true],
  ]);
  expect(() => rigParts("biped")).toThrow("use one of humanoid-side, humanoid-front, quadruped, creature");
});

test("the published package briefs from its own presets, catalogue and previews", () => {
  // The package as npm lays it out: what the prepack copies, with no engine repository beside it.
  const r = Bun.spawnSync(["bun", join(pkg, "scripts", "pack-template.ts")]);
  expect(r.exitCode).toBe(0);
  const installed = join(scratch, "node_modules", "@origingame", "create-game");
  for (const part of ["bin", "looks", "previews", "presets", "styles", "locales", "template", "template-2d", "archetypes", "package.json"]) cpSync(join(pkg, part), join(installed, part), { recursive: true });
  const r2 = Bun.spawnSync(["bun", join(installed, "bin", "brief.ts"), "sprite", "a bat", "--look", "monochrome-pixel", "--json"], { cwd: scratch });
  expect(new TextDecoder().decode(r2.stderr)).toBe("");
  const brief = JSON.parse(new TextDecoder().decode(r2.stdout)) as { palette: { hex: string }[]; references: { path: string }[] };
  // monochrome-pixel's palette is @palettes/handheld4 (templates/looks/assets/palettes/handheld4.hex), read from presets/.
  expect(brief.palette.map((c) => c.hex)).toEqual(
    readFileSync(join(pkg, "..", "..", "templates", "looks", "assets", "palettes", "handheld4.hex"), "utf8")
      .split("\n")
      .filter((l) => /^[0-9a-f]{6}$/i.test(l.trim()))
      .map((l) => `#${l.trim().toLowerCase()}`),
  );
  expect(brief.references[0]!.path).toBe(join(installed, "previews", "monochrome-pixel.webp"));
  // The repository's own copy reads the same preset from templates/looks.
  expect(presetLook("monochrome-pixel").palette?.colours.map((c) => c.hex)).toEqual(brief.palette.map((c) => c.hex));
  const created = Bun.spawnSync(["bun", join(installed, "bin/create.ts"), join(scratch, "packed-choices"), "--players", "online-lockstep", "--ui", "dread", "--no-prompt"]);
  expect(new TextDecoder().decode(created.stderr)).toBe("");
  expect(created.exitCode).toBe(0);
  expect(existsSync(join(scratch, "packed-choices/assets/styles/dread/style.ogt"))).toBeTrue();
  expect(readFileSync(join(scratch, "packed-choices/tests/duel.og.ts"), "utf8")).toContain("netHash");
});

test("--lang writes a table per language with its face, direction and licence, the pseudo-locale and a playtest each", () => {
  const r = cli(join(scratch, "langs"), "--no-prompt", "--lang", "zh,en,ar,ja,th");
  expect(r.code).toBe(0);
  const dir = join(scratch, "langs");
  const read = (f: string) => readFileSync(join(dir, f), "utf8");
  // The first chosen language with every key is the base; zh lands on zh-CN, the starter's English is kept.
  expect(read("assets/strings/zh-CN.ogt")).toMatch(/^base true$/m);
  expect(read("assets/strings/en.ogt")).not.toMatch(/^base/m);
  expect(read("assets/strings/en.ogt")).toContain('string "menu.start" "Start"');
  expect(read("assets/strings/ar.ogt")).toMatch(/^direction rtl$/m);
  expect(read("assets/strings/ar.ogt")).toContain('fonts "fonts/NotoSansArabic-Regular.ttf"');
  expect(read("assets/strings/ja.ogt")).toContain('string "menu.start" "スタート"');
  // A language with no shipped translation gets an empty table, never copied English, and a note.
  expect(read("assets/strings/th.ogt")).not.toContain("string ");
  expect(r.out).toContain("Noto Sans Thai");
  // Faces with their licences; the import cooks the table's own characters.
  for (const f of ["NotoSansArabic-Regular.ttf", "OFL-NotoSansArabic.txt", "NotoSansJP-Regular-subset.ttf", "OFL-NotoSansJP.txt"]) expect(existsSync(join(dir, "assets/fonts", f))).toBeTrue();
  expect(read("assets/fonts/LICENSES.md")).toContain("`NotoSansJP-Regular-subset.ttf`");
  expect(read("assets/fonts/NotoSansJP-Regular-subset.ttf.ogt")).toContain("ス");
  // Every table loaded, the pseudo-locale last; the smoke test reads the new base's word.
  expect(read("src/main.ts")).toContain('["zh-CN","en","ar","ja","th","qps-ploc"]');
  expect(read("tests/smoke.og.ts")).toContain("expect:ui:text:=已开始");
  expect(read("assets/strings/qps-ploc.ogt")).toContain('string "menu.start" "[开始 开]"');
  for (const t of ["zh-CN", "en", "ar", "ja", "th"]) expect(existsSync(join(dir, `tests/locale-${t}.og.ts`))).toBeTrue();
  expect(existsSync(join(dir, "tests/pseudo.og.ts")) && existsSync(join(dir, "tools/pseudo-locale.ts"))).toBeTrue();
  expect(localeScript("zh-TW")).toBe("hant");
  expect(localeScript("sr-Latn")).toBe("latin");
});

test("the pseudo-locale accents words, not message syntax", () => {
  expect(pseudoMessage("Start")).toBe("[Stárt st]");
  // Argument names, plural keywords, # and rich-text tags survive; branch text is accented.
  expect(pseudoMessage("{n, plural, one {# coin} other {# coins}} [b]now[/b]")).toBe("[{n, plural, one {# çöíñ} other {# çöíñs}} [b]ñöw[/b] çöíñçö]");
  expect(pseudoMessage("{score}")).toBe("{score}");
  expect(pseudoMessage("'{'literal'}'")).toMatch(/^\['\{'lítérál/);
});

test("audioBrief takes every music, mood and genre the creation choices offer", () => {
  for (const music of CREATION_CHOICES.music) for (const mood of CREATION_CHOICES.mood) for (const genre of CREATION_CHOICES.genre) {
    const b = audioBrief({ music, mood, genre });
    expect(b.cues.length).toBeGreaterThan(2);
    expect(b.sfx.length).toBeGreaterThan(4);
  }
});
