import { afterAll, expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { artBrief, audioBrief, authoredArt, create, creationDocument, creationLooks, CREATION_CHOICES, FORM_VIEWS, formOf, previewSlots, fontPackFor, fontPacksDir, creationSchema, inferCreationChoices, MOODS, parseCreationInput, pickStyle, presetLook, pseudoMessage, readCatalogue, resolveCreationChoices, RIG_TEMPLATES, rigParts, words } from "../bin/create.ts";

import * as knowledgeBrief from "../../knowledge/scripts/brief.ts";

const pkg = join(import.meta.dir, "..");
const scratch = realpathSync(mkdtempSync(join(tmpdir(), "og-create-")));
afterAll(() => rmSync(scratch, { recursive: true, force: true }));
const read = (...parts: string[]) => readFileSync(join(scratch, ...parts), "utf8");

function cli(...args: string[]) {
  const r = Bun.spawnSync(["bun", join(pkg, "bin/create.ts"), ...args, "--no-prompt"], { cwd: scratch });
  return { code: r.exitCode, out: new TextDecoder().decode(r.stdout), err: new TextDecoder().decode(r.stderr) };
}

function files(root: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap(e => e.isDirectory() ? files(join(root, e.name)).map(f => `${e.name}/${f}`) : e.isFile() ? [e.name] : []).sort();
}

test("the installed npm og routes create and new without a native platform package", () => {
  const install = join(scratch, "npm-install");
  const engine = join(install, "node_modules/@origingame/engine");
  mkdirSync(join(engine, "bin"), { recursive: true });
  cpSync(join(pkg, "../engine/bin/og.mjs"), join(engine, "bin/og.mjs"));
  symlinkSync(pkg, join(install, "node_modules/@origingame/create-game"), "dir");
  for (const command of ["create", "new"]) {
    const r = Bun.spawnSync(["node", join(engine, "bin/og.mjs"), command, command, "--name", "Installed Game", "--view", "side-2d", "--look", "pixel", "--no-prompt"], { cwd: install });
    expect(r.exitCode, new TextDecoder().decode(r.stderr)).toBe(0);
    expect(read("npm-install", command, "project.ogt")).toContain('name "Installed Game"');
    expect(read("npm-install", command, "project.ogt")).toContain('view "side-2d"');
    expect(existsSync(join(install, command, "src/main.ts"))).toBeTrue();
  }
});

test("the fixtures carry the skeleton's tsconfig, whose include names only what a project has", () => {
  const skeleton = readFileSync(join(pkg, "template/tsconfig.json"), "utf8");
  expect(JSON.parse(skeleton).include).toEqual(["src/**/*.ts", "src/**/*.tsx", "assets/ui/**/*.tsx", "tests/**/*.ts", ".og/types/**/*.d.ts"]);
  const fixtures = join(pkg, "../../tests/fixtures");
  const copies = readdirSync(fixtures).map(name => join(fixtures, name))
    .map(dir => join(dir, "tsconfig.json")).filter(file => existsSync(file));
  expect(copies.length).toBeGreaterThan(10);
  for (const file of copies) expect(readFileSync(file, "utf8"), file).toBe(skeleton);
});

test("no-prompt creation uses source choices even with an outdated packed catalogue", () => {
  const checkout = join(scratch, "checkout");
  const source = join(checkout, "packages/create");
  for (const part of ["bin", "template", "package.json"]) cpSync(join(pkg, part), join(source, part), { recursive: true });
  cpSync(fontPacksDir(), join(checkout, "templates/fonts"), { recursive: true });
  mkdirSync(join(source, "looks"), { recursive: true });
  mkdirSync(join(checkout, "packages/knowledge/looks"), { recursive: true });
  cpSync(join(pkg, "../knowledge/scripts"), join(checkout, "packages/knowledge/scripts"), { recursive: true });
  const catalogue = readFileSync(join(pkg, "../knowledge/looks/catalogue.json"), "utf8");
  writeFileSync(join(checkout, "packages/knowledge/looks/catalogue.json"), catalogue);
  writeFileSync(join(source, "looks/catalogue.json"), catalogue.replaceAll('"ui": "default"', '"ui": "elsewhere"'));
  const r = Bun.spawnSync(["bun", join(source, "bin/create.ts"), "game", "--no-prompt"], { cwd: checkout });
  expect(r.exitCode, new TextDecoder().decode(r.stderr)).toBe(0);
  expect(read("checkout/game/project.ogt")).toContain('ui "default"');
  expect(read("checkout/game/assets/looks/game.ogt")).toContain('ui "default"');
});

test("packing and creating leave the source skeleton unchanged", () => {
  const template = join(pkg, "template");
  const paths = ["assets/.gitkeep", "gitattributes", "gitignore", "package.json", "project.ogt", "src/main.ts", "tests/smoke.og.ts", "tsconfig.json"];
  // Also detects ignored output left by the old prepack, before the run.
  expect(files(template)).toEqual(paths);
  const before = paths.map(path => readFileSync(join(template, path), "utf8"));
  const packed = Bun.spawnSync(["bun", join(pkg, "scripts/pack-template.ts")], { cwd: scratch });
  expect(packed.exitCode, new TextDecoder().decode(packed.stderr)).toBe(0);
  create({ dir: join(scratch, "source-isolation"), id: "origin-game" });
  expect(files(template)).toEqual(paths);
  expect(paths.map(path => readFileSync(join(template, path), "utf8"))).toEqual(before);
});

test("creation choices write manifest and look configuration, never gameplay", () => {
  const defaults = create({ dir: join(scratch, "default"), id: "same", name: "Same" });
  expect(defaults.choices).toEqual(resolveCreationChoices(""));
  // Each language installs its font pack whole, as `og font add` does (docs/ui.md §3.4).
  const pack = (name: string) => files(join(fontPacksDir(), name)).filter((f) => !f.endsWith(".py")).map((f) => `assets/fonts/${name}/${f}`);
  const skeleton = [".gitattributes", ".gitignore", "assets/.gitkeep", "assets/looks/same.ogt", "package.json", "project.ogt", "src/main.ts", "tests/smoke.og.ts", "tsconfig.json"];
  expect(files(defaults.dir)).toEqual([...skeleton, ...pack("latin")].sort());
  const expected = [...skeleton, ...pack("japanese"), ...pack("latin"), ...pack("arabic")].sort();
  expect(read("default/src/main.ts")).toBe('import { og } from "@origingame/engine";\n\nexport default og.game({});\n');
  for (const [i, view] of CREATION_CHOICES.view.entries()) {
    const dir = `choice-${i}`;
    const look = formOf(view) === "2d" ? "pixel" : "faceted-low-poly";
    const r = cli(dir, "--id", "same", "--name", "Same", "--genre", "arena", "--view", view, "--look", look, "--ui", "default", "--platform", "phone-portrait", "--input", "gamepad", "--players", "online-lockstep", "--lang", "ja,en,ar", "--music", "folk");
    expect(r.code, r.err).toBe(0);
    expect(files(join(scratch, dir))).toEqual(expected);
    for (const f of skeleton.filter(f => f !== "project.ogt" && f !== "assets/looks/same.ogt")) expect(read(dir, f)).toBe(read("default", f));
    expect(read(dir, "assets/looks/same.ogt")).toBe(`type look\nversion 1\n---\nextends "${look}"\nui "default"\n`);
    const manifest = read(dir, "project.ogt");
    expect(manifest.slice(0, manifest.indexOf("create {"))).toBe(read("default/project.ogt").split("create {")[0]);
    for (const line of [`form "${formOf(view)}"`, 'genre "arena"', `view "${view}"`, `look "${look}"`, 'ui "default"', 'platform "phone-portrait"', 'input "gamepad"', 'players "online-lockstep"', 'lang "ja" "en" "ar"', 'music "folk"']) expect(manifest).toContain(line);
    expect(manifest).toMatch(/^look "looks\/same.ogt"$/m);
    expect(manifest).not.toMatch(/^profile |orientation|dpr_cap|^\s*mood /m);
  }
});

test("a language takes the font pack written in it; Chinese by script", () => {
  const packs = [{ name: "latin", languages: ["en", "vi"] }, { name: "chinese-simplified", languages: ["zh", "zh-Hans", "zh-CN"] }, { name: "chinese-traditional", languages: ["zh-Hant", "zh-TW", "zh-HK"] }, { name: "arabic", languages: ["ar", "fa"] }];
  expect(fontPackFor("en-GB", packs)).toBe("latin");
  expect(fontPackFor("fa_IR", packs)).toBe("arabic");
  expect(fontPackFor("zh", packs)).toBe("chinese-simplified");
  expect(fontPackFor("zh-TW", packs)).toBe("chinese-traditional");
  expect(fontPackFor("zh-Hant-HK", packs)).toBe("chinese-traditional");
  expect(fontPackFor("zh-SG", packs)).toBe("chinese-simplified");
  expect(fontPackFor("th", packs)).toBeUndefined();
});

test("names are escaped, defaults are stable, and refusal leaves the destination untouched", () => {
  expect(cli("named", "--name", 'Ünïcode "Game" $&\n2!').code).toBe(0);
  expect(read("named/project.ogt")).toContain('id "unicode-game-2"');
  expect(read("named/project.ogt")).toContain('name "Ünïcode \\"Game\\" $&\\n2!"');
  expect(cli("id", "--id", "tiny-tower").code).toBe(0);
  expect(read("id/project.ogt")).toContain('name "Tiny Tower"');
  for (const [flag, value] of [["--id", "Not An Id"], ["--mood", "cold"], ["--look", "missing"], ["--lang", "en,en"], ["--profile", "2d"], ["--archetype", "runner"]]) {
    expect(cli("refused", flag!, value!).code).toBe(1);
    expect(existsSync(join(scratch, "refused"))).toBeFalse();
  }
  expect(cli("refused", "--mood", "cold").err).toContain("--mood is no longer a creation choice");
  mkdirSync(join(scratch, "taken"));
  writeFileSync(join(scratch, "taken/notes.txt"), "mine");
  expect(cli("taken").err).toContain("--force");
  expect(existsSync(join(scratch, "taken/project.ogt"))).toBeFalse();
  expect(cli("taken", "--force").code).toBe(0);
  expect(read("taken/notes.txt")).toBe("mine");
});

test("creation choices infer a premise, preserve overrides, and serialize every choice", () => {
  const c = resolveCreationChoices("温暖的手机横屏平台跳跃", { music: "folk", look: "pixel" });
  expect(c).toEqual({ form: "2d", genre: "action-platformer", view: "side-2d", look: "pixel", ui: "default", platform: "phone-landscape", input: "touch", players: "single", lang: ["zh"], music: "folk" });
  expect(inferCreationChoices("isometric online shooter", { platform: "desktop", input: "gamepad" })).toMatchObject({ genre: "shooter", view: "isometric-2.5d", players: "online-lockstep", input: "gamepad" });
  expect(resolveCreationChoices("", { look: "pixel", ui: "default" }).ui).toBe("default");
  expect(() => parseCreationInput({ players: "mmo" })).toThrow("players");
  expect(() => parseCreationInput({ mood: "cold" } as never)).toThrow("mood is no longer a creation choice");
  expect(() => parseCreationInput({ lang: [] })).toThrow("lang");
  expect(() => parseCreationInput({ lang: ["en", "en"] })).toThrow("lang");
  expect(() => resolveCreationChoices("", { look: "missing" })).toThrow("no look preset");
  // A style folded into a look's variant, or deleted, is no look: it fails as any unknown name.
  for (const gone of ["watercolor-wash", "16bit-pixel", "illuminated-manuscript"]) expect(() => resolveCreationChoices("", { look: gone })).toThrow(`no look preset '${gone}'; pick one of cinematic-realism,`);
  // A look's own UI pack is the default UI choice; an explicit one wins.
  expect(resolveCreationChoices("", { look: "cartoon-2d" }).ui).toBe("default");
  expect(resolveCreationChoices("", { look: "painterly-3d", ui: "default" }).ui).toBe("default");
});

test("creation schema labels preserve every stable choice ID", () => {
  const schema = creationSchema(creationLooks());
  for (const [key, values] of Object.entries(CREATION_CHOICES)) {
    const field = Object.entries(schema.properties).find(([name]) => name === key)?.[1];
    expect(field).toMatchObject({ type: "string", enum: [...values] });
    if (!field || !("x-labels" in field)) throw new Error(`missing labels for ${key}`);
    expect(Object.keys(field["x-labels"])).toEqual([...values]);
    for (const copy of Object.values(field["x-labels"])) expect(copy).toMatchObject({ label: { en: expect.any(String), "zh-CN": expect.any(String) }, description: { en: expect.any(String), "zh-CN": expect.any(String) } });
  }
  expect(schema.properties.look.enum).toEqual(["cinematic-realism", "anime-cel-3d", "inked-comic-3d", "painterly-3d", "faceted-low-poly", "low-resolution-3d", "pixel", "minimal-geometric", "neon-vector", "toy-3d", "cartoon-2d", "pixel-3d-hybrid"]);
  for (const copy of Object.values(schema.properties.look["x-labels"])) expect(copy).toMatchObject({ label: { "zh-CN": expect.any(String) }, description: { "zh-CN": expect.any(String) } });
});

test("art and audio briefs remain available without creating a game", () => {
  // The knowledge package's brief itself, not a copy.
  expect(artBrief).toBe(knowledgeBrief.artBrief);
  expect(artBrief(presetLook("pixel"), "character", "the courier").image?.grid).toEqual({ width: 16, height: 32, scale: 32 });
  const prop = artBrief(presetLook("faceted-low-poly"), "prop", "a signpost");
  expect([prop.medium, prop.model?.faceBudget, prop.model?.normals]).toEqual(["model", 200, "flat"]);
  expect([...RIG_TEMPLATES]).toEqual(["humanoid-side", "humanoid-front", "quadruped", "creature"]);
  expect(rigParts("humanoid-front").filter(p => /^(leg|foot)/.test(p.layer)).map(p => [p.layer, p.side, p.bone, p.required])).toEqual([
    ["leg-upper-l", "l", "leg-upper-l", true], ["leg-lower-l", "l", "leg-lower-l", true], ["foot-l", "l", "foot-l", true],
    ["leg-upper-r", "r", "leg-upper-r", true], ["leg-lower-r", "r", "leg-lower-r", true], ["foot-r", "r", "foot-r", true],
  ]);
  expect(() => rigParts("biped")).toThrow("use one of");
  for (const music of CREATION_CHOICES.music) for (const mood of MOODS) for (const genre of CREATION_CHOICES.genre) {
    const b = audioBrief({ music, mood, genre });
    expect(b.cues.length).toBeGreaterThan(2);
    expect(b.sfx.length).toBeGreaterThan(4);
  }
  expect(authoredArt(presetLook("cartoon-2d", { variant: "rubber-hose" })).map(a => a.kind)).toEqual(["character", "sprite"]);
  expect(pseudoMessage("{n, plural, one {# coin} other {# coins}} [b]now[/b]")).toBe("[{n, plural, one {# çöíñ} other {# çöíñs}} [b]ñöw[/b] çöíñçö]");
});

test("catalogue suggestions respect profile without installing them", () => {
  const styles = readCatalogue();
  expect(pickStyle("a cheerful toy party builder for families", styles)?.style.slug).toBe("toy-3d");
  expect(pickStyle("a whimsical cartoon platformer for children", styles, "2d")?.style.slug).toBe("cartoon-2d");
  expect(pickStyle("16-bit platformer, energetic and heroic", styles, "2d")?.style.slug).toBe("pixel");
  expect(pickStyle("", styles)).toBeNull();
  expect(pickStyle("neon arcade shooter", styles.map(s => ({ ...s, look: { ...s.look, presetReady: false } })))).toBeNull();
  expect(words("Cozy Platformers in Sci-Fi colors, hand-drawn")).toEqual(["cosy", "platformer", "sci", "colour", "hand-drawn", "hand", "drawn"]);
  expect(CREATION_CHOICES).not.toHaveProperty("mood");
  expect(resolveCreationChoices("", { look: "low-resolution-3d" })).not.toHaveProperty("mood");
});

test("the packed package creates the same skeleton, and briefs with the engine's knowledge beside it", () => {
  // The release archive's layout: create/ beside engine/, which carries the knowledge package as knowledge/.
  const installed = join(scratch, "dist", "create");
  const knowledge = join(scratch, "dist", "engine", "knowledge");
  const source = join(pkg, "..", "knowledge");
  for (const part of [...JSON.parse(readFileSync(join(source, "package.json"), "utf8")).files, "package.json"]) cpSync(join(source, part), join(knowledge, part), { recursive: true });
  const manifest = JSON.parse(readFileSync(join(pkg, "package.json"), "utf8"));
  expect(manifest.files).not.toContain("archetypes");
  expect(manifest.files).not.toContain("template-2d");
  for (const part of [...manifest.files, "package.json"]) cpSync(join(pkg, part), join(installed, part), { recursive: true });
  mkdirSync(join(installed, "template/node_modules/@origingame"), { recursive: true });
  symlinkSync("../../..", join(installed, "template/node_modules/@origingame/engine"), "dir");
  symlinkSync(".", join(installed, "template/loop"), "dir");
  // A file left in template/ that the manifest does not list (a checkout's untracked file) is not part of the skeleton.
  writeFileSync(join(installed, "template/stray.png"), "left behind");
  mkdirSync(join(installed, "template/tests/references"), { recursive: true });
  writeFileSync(join(installed, "template/tests/references/frame.png"), "left behind");
  expect(files(join(installed, "template"))).not.toContain("node_modules/@origingame/engine");
  const packed = Bun.spawnSync(["bun", "pm", "pack", "--dry-run", "--ignore-scripts"], { cwd: installed });
  expect(packed.exitCode, new TextDecoder().decode(packed.stderr)).toBe(0);
  const listing = new TextDecoder().decode(packed.stdout);
  expect(listing).toContain("template/src/main.ts");
  expect(listing).not.toContain("template/.og");
  expect(listing).not.toContain("template/loop");
  const r = Bun.spawnSync(["bun", join(installed, "bin/create.ts"), join(scratch, "packed"), "--players", "online-lockstep", "--ui", "default", "--no-prompt"]);
  expect(r.exitCode, new TextDecoder().decode(r.stderr)).toBe(0);
  expect(existsSync(join(scratch, "packed/.og"))).toBeFalse();
  expect(existsSync(join(scratch, "packed/loop"))).toBeFalse();
  expect(existsSync(join(scratch, "packed/stray.png"))).toBeFalse();
  expect(readdirSync(join(scratch, "packed/tests"))).toEqual(["smoke.og.ts"]);
  expect(read("packed/src/main.ts")).toBe(readFileSync(join(pkg, "template/src/main.ts"), "utf8"));
  expect(readdirSync(join(scratch, "packed/assets")).sort()).toEqual([".gitkeep", "fonts", "looks"]);
  expect(readdirSync(join(scratch, "packed/assets/fonts"))).toEqual(["latin"]);
  // The packed package's briefs are the engine's knowledge's: its presets, variants and previews.
  const brief = (variant?: string) => {
    const code = `const c = await import(${JSON.stringify(join(installed, "bin/create.ts"))}); console.log(JSON.stringify(c.artBrief(c.presetLook("pixel", ${JSON.stringify(variant ? { variant } : {})}), "sprite", "a bat")))`;
    const r = Bun.spawnSync(["bun", "-e", code], { cwd: scratch });
    expect(r.exitCode, new TextDecoder().decode(r.stderr)).toBe(0);
    return JSON.parse(new TextDecoder().decode(r.stdout));
  };
  const mono = brief("monochrome");
  expect(mono.palette.map((c: { hex: string }) => c.hex)).toEqual(presetLook("pixel", { variant: "monochrome" }).palette?.colours.map(c => c.hex));
  expect(mono.palette).toHaveLength(4);
  expect(brief().references[0].path).toBe(join(knowledge, "previews/pixel.webp"));
});

test("in a checkout missing a pinned font, create refuses with the one fix and writes nothing", () => {
  // The checkout's layout: packages/create, packages/knowledge's looks and scripts, templates/ with og-assets.json.
  const root = join(scratch, "unfetched-checkout");
  const repo = join(pkg, "..", "..");
  for (const part of ["bin", "template", "package.json"]) cpSync(join(pkg, part), join(root, "packages/create", part), { recursive: true });
  for (const part of ["looks", "scripts"]) cpSync(join(repo, "packages/knowledge", part), join(root, "packages/knowledge", part), { recursive: true });
  cpSync(join(repo, "templates/og-assets.json"), join(root, "templates/og-assets.json"));
  cpSync(join(repo, "templates/fonts/latin"), join(root, "templates/fonts/latin"), { recursive: true, filter: (s) => !s.endsWith("NotoSans-Latin.ttf") });
  const r = Bun.spawnSync(["bun", join(root, "packages/create/bin/create.ts"), "game", "--lang", "en", "--no-prompt"], { cwd: scratch });
  const err = new TextDecoder().decode(r.stderr);
  expect(r.exitCode).toBe(1);
  expect(err.trim().split("\n")).toHaveLength(1);
  expect(err).toContain("fonts/latin/NotoSans-Latin.ttf");
  expect(err).toContain("run bun scripts/assets.ts ensure templates");
  expect(existsSync(join(scratch, "game"))).toBe(false);
});

test("a look that does not draw the form or view is refused, naming the looks that do, and writes nothing", () => {
  expect(() => resolveCreationChoices("", { look: "pixel", form: "3d" })).toThrow("look pixel does not draw 3d (third-person-3d)");
  expect(() => resolveCreationChoices("", { look: "cinematic-realism", view: "side-2d" })).toThrow(/For side-2d pick one of pixel, minimal-geometric, neon-vector, cartoon-2d$/);
  expect(() => resolveCreationChoices("", { form: "2d", view: "third-person-3d" })).toThrow("view third-person-3d is 3d, not form 2d");
  expect(() => resolveCreationChoices("", { look: "pixel", variant: "voxel" })).toThrow("look pixel has no variant 'voxel'");
  for (const args of [["--form", "3d", "--look", "pixel"], ["--form", "film"], ["--genre", "interactive-film", "--form", "2d", "--view", "third-person-3d"], ["--look", "toy-3d", "--variant", "8bit"]]) {
    const r = cli("refused-pair", ...args);
    expect(r.code, r.err).toBe(1);
    expect(existsSync(join(scratch, "refused-pair"))).toBeFalse();
  }
  // Each form's default view is one its default look draws.
  for (const form of CREATION_CHOICES.form) {
    const c = resolveCreationChoices("", { form });
    expect(FORM_VIEWS[form]).toContain(c.view!);
  }
  expect(resolveCreationChoices("", { form: "2d", genre: "rpg" }).view).toBe("top-down-2d");
  // An interactive film is form 3d unless named, with no view unless named; any form takes it.
  expect(resolveCreationChoices("an interactive film about a heist")).toMatchObject({ genre: "interactive-film", form: "3d", players: "single" });
  expect(resolveCreationChoices("an interactive film about a heist").view).toBeUndefined();
  expect(resolveCreationChoices("", { genre: "interactive-film", form: "2d", view: "side-2d" })).toMatchObject({ form: "2d", view: "side-2d" });
  expect(resolveCreationChoices("", { look: "toy-3d", variant: "clay" })).toMatchObject({ variant: "clay" });
});

test("genre interactive-film writes the flow skeleton over the one skeleton, run by project.ogt's flow line", () => {
  // 底牌's playtest: --genre interactive-film was refused and the block read action-platformer.
  const g = cli("film-genre", "--genre", "interactive-film", "--lang", "zh,en", "--no-prompt");
  expect(g.code, g.err).toBe(0);
  const genreBlock = read("film-genre/project.ogt").split("create {")[1]!;
  for (const line of ['form "3d"', 'genre "interactive-film"', 'look "cinematic-realism"']) expect(genreBlock).toContain(line);
  expect(genreBlock).not.toMatch(/action-platformer|^\s*view /m);
  const r = cli("film", "--genre", "interactive-film", "--look", "painterly-3d", "--variant", "lush");
  expect(r.code, r.err).toBe(0);
  const made = files(join(scratch, "film"));
  for (const f of ["assets/story/film.yarn", "assets/segments/opening.ogt", "assets/segments/ending.ogt", "assets/segments/live.ogt", "assets/scenes/ending.ogt", "assets/models/set.ogt", "assets/clips/opening.ogt", "assets/clips/opening.mp4", "tests/smoke.og.ts", "tsconfig.json"]) expect(made).toContain(f);
  // No glue: the engine starts the flow and draws it.
  expect(read("film/src/main.ts")).toBe(read("film-genre/src/main.ts"));
  expect(read("film/src/main.ts")).not.toContain("og.flow");
  expect(read("film/project.ogt")).toMatch(/^flow "story\/film.yarn"$/m);
  expect(read("film-genre/project.ogt")).toMatch(/^flow "story\/film.yarn"$/m);
  expect(read("film/assets/looks/film.ogt")).toBe('type look\nversion 1\n---\nextends "painterly-3d"\nvariant "lush"\nui "default"\n');
  // A game of another genre has neither.
  const plain = cli("plain-game", "--genre", "rpg", "--lang", "en", "--no-prompt");
  expect(plain.code, plain.err).toBe(0);
  expect(read("plain-game/project.ogt")).not.toMatch(/^flow /m);
});

test("the creation document matches the options, and a release cannot ship an unfilled preview slot", () => {
  const doc = creationDocument();
  const looks = creationLooks();
  expect(doc.options.look.map(l => l.id)).toEqual(looks);
  expect(doc.schema.properties.look.enum).toEqual(looks);
  for (const key of Object.keys(CREATION_CHOICES) as (keyof typeof CREATION_CHOICES)[]) {
    const ids = doc.options[key].map((o: { id: string }) => o.id);
    expect(ids, key).toEqual([...CREATION_CHOICES[key]]);
    expect(doc.schema.properties[key].enum, key).toEqual(ids);
  }
  expect([...doc.order, ...doc.more].sort()).toEqual(Object.keys(doc.schema.properties).sort());
  // Every look and variant has its slot and its compat entry; every compat look draws its views' forms.
  const slots = previewSlots();
  const paths = new Set(slots.map(s => s.path));
  const referenced = [...doc.options.look.flatMap(l => [l.preview, ...l.variants.map(v => v.preview)]), ...doc.options.ui.map(o => o.preview!)];
  expect(referenced.map(p => p.path).sort()).toEqual([...paths].sort());
  expect(paths.has("previews/pixel.8bit.webp") && paths.has("previews/ui/default.webp")).toBeTrue();
  expect(slots.map(s => s.kind).filter(k => k !== "look" && k !== "variant" && k !== "ui")).toEqual([]);
  expect(slots.some(s => s.path.startsWith("previews/mood/"))).toBeFalse();
  expect(doc.options).not.toHaveProperty("mood");
  expect([...doc.order, ...doc.more]).not.toContain("mood");
  for (const l of doc.options.look) {
    expect(doc.compat.looks[l.id]!.variants).toEqual(l.variants.map(v => v.id));
    for (const v of l.variants) expect(v.label["zh-CN"], `${l.id}.${v.id}`).toEqual(expect.any(String));
  }
  expect(doc.compat.looks.pixel!.forms).toEqual(["2d"]);
  expect(doc.compat.looks["minimal-geometric"]!.forms).toEqual(["2d", "2.5d", "3d"]);
  expect(doc.compat.genreDefaultView.rpg).toEqual({ "2d": "top-down-2d", "2.5d": "isometric-2.5d", "3d": "third-person-3d" });
  expect(doc.defaults.byForm).not.toHaveProperty("film");
  // Filled: each slot carries its file; a missing or unknown slot throws.
  const filled = Object.fromEntries(slots.map((s, i) => [s.path, { url: `https://cdn/${s.path}`, sha256: String(i).padStart(64, "0"), size: i + 1, engine: "0.9.9" }]));
  const full = creationDocument(filled);
  expect(full.options.look.find(l => l.id === "pixel")!.variants[0]!.preview).toEqual({ path: "previews/pixel.8bit.webp", ...filled["previews/pixel.8bit.webp"]! });
  expect(JSON.stringify(creationDocument(filled))).toBe(JSON.stringify(full));
  const { ["previews/ui/default.webp"]: _, ...short } = filled;
  expect(() => creationDocument(short)).toThrow("previews/ui/default.webp");
  expect(() => creationDocument({ ...filled, "previews/mood/dark.webp": filled["previews/pixel.webp"]! })).toThrow("previews/mood/dark.webp");
  expect(() => creationDocument({ ...filled, "previews/extra.webp": filled["previews/pixel.webp"]! })).toThrow("previews/extra.webp");
  const printed = Bun.spawnSync(["bun", join(pkg, "bin/create.ts"), "--creation"]);
  expect(JSON.parse(new TextDecoder().decode(printed.stdout))).toEqual(JSON.parse(JSON.stringify(doc)));
});

test("in a checkout missing the film's pinned clip, genre interactive-film refuses with the one fix and writes nothing", () => {
  const root = join(scratch, "unfetched-film");
  const repo = join(pkg, "..", "..");
  for (const part of ["bin", "template", "package.json"]) cpSync(join(pkg, part), join(root, "packages/create", part), { recursive: true });
  for (const part of ["looks", "scripts"]) cpSync(join(repo, "packages/knowledge", part), join(root, "packages/knowledge", part), { recursive: true });
  cpSync(join(repo, "templates/og-assets.json"), join(root, "templates/og-assets.json"));
  cpSync(join(repo, "templates/fonts/latin"), join(root, "templates/fonts/latin"), { recursive: true });
  cpSync(join(repo, "templates/film"), join(root, "templates/film"), { recursive: true, filter: (s) => !s.endsWith(".mp4") });
  const r = Bun.spawnSync(["bun", join(root, "packages/create/bin/create.ts"), "film-unfetched", "--genre", "interactive-film", "--lang", "en", "--no-prompt"], { cwd: scratch });
  const err = new TextDecoder().decode(r.stderr);
  expect(r.exitCode).toBe(1);
  expect(err).toContain("film/assets/clips/opening.mp4");
  expect(err).toContain("run bun scripts/assets.ts ensure templates");
  expect(existsSync(join(scratch, "film-unfetched"))).toBe(false);
});
