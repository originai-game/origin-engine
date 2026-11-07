import { afterAll, expect, test } from "bun:test";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { artBrief, audioBrief, authoredArt, create, CREATION_CHOICES, creationSchema, inferCreationChoices, MOODS, parseCreationInput, pickStyle, presetLook, pseudoMessage, readCatalogue, resolveCreationChoices, RIG_TEMPLATES, rigParts, words } from "../bin/create.ts";

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

test("no-prompt creation uses source choices even with an outdated packed catalogue", () => {
  const checkout = join(scratch, "checkout");
  const source = join(checkout, "packages/create");
  for (const part of ["bin", "template"]) cpSync(join(pkg, part), join(source, part), { recursive: true });
  mkdirSync(join(source, "looks"), { recursive: true });
  mkdirSync(join(checkout, "packages/knowledge/looks"), { recursive: true });
  const catalogue = readFileSync(join(pkg, "../knowledge/looks/catalogue.json"), "utf8");
  writeFileSync(join(checkout, "packages/knowledge/looks/catalogue.json"), catalogue);
  writeFileSync(join(source, "looks/catalogue.json"), catalogue.replaceAll('"ui": "default"', '"ui": "fantasy"'));
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
  const expected = [".gitattributes", ".gitignore", "assets/.gitkeep", "assets/looks/same.ogt", "package.json", "project.ogt", "src/main.ts", "tests/smoke.og.ts", "tsconfig.json"];
  expect(files(defaults.dir)).toEqual(expected);
  expect(read("default/src/main.ts")).toBe('import { og } from "@origingame/engine";\n\nexport default og.game({});\n');
  for (const [i, view] of CREATION_CHOICES.view.entries()) {
    const dir = `choice-${i}`;
    const r = cli(dir, "--id", "same", "--name", "Same", "--genre", "arena", "--view", view, "--look", "16bit-pixel", "--mood", "cold", "--ui", "default", "--platform", "phone-portrait", "--input", "gamepad", "--players", "online-lockstep", "--lang", "ja,en,ar", "--music", "folk");
    expect(r.code, r.err).toBe(0);
    expect(files(join(scratch, dir))).toEqual(expected);
    for (const f of expected.filter(f => f !== "project.ogt" && f !== "assets/looks/same.ogt")) expect(read(dir, f)).toBe(read("default", f));
    expect(read(dir, "assets/looks/same.ogt")).toBe('type look\nversion 1\n---\nextends "16bit-pixel"\nmood "cold"\nui "default"\n');
    const manifest = read(dir, "project.ogt");
    expect(manifest.slice(0, manifest.indexOf("create {"))).toBe(read("default/project.ogt").split("create {")[0]);
    for (const line of ['genre "arena"', `view "${view}"`, 'look "16bit-pixel"', 'mood "cold"', 'ui "default"', 'platform "phone-portrait"', 'input "gamepad"', 'players "online-lockstep"', 'lang "ja" "en" "ar"', 'music "folk"']) expect(manifest).toContain(line);
    expect(manifest).toMatch(/^look "looks\/same.ogt"$/m);
    expect(manifest).not.toMatch(/^profile |orientation|dpr_cap/m);
  }
});

test("names are escaped, defaults are stable, and refusal leaves the destination untouched", () => {
  expect(cli("named", "--name", 'Ünïcode "Game" $&\n2!').code).toBe(0);
  expect(read("named/project.ogt")).toContain('id "unicode-game-2"');
  expect(read("named/project.ogt")).toContain('name "Ünïcode \\"Game\\" $&\\n2!"');
  expect(cli("id", "--id", "tiny-tower").code).toBe(0);
  expect(read("id/project.ogt")).toContain('name "Tiny Tower"');
  for (const [flag, value] of [["--id", "Not An Id"], ["--mood", "invalid"], ["--look", "missing"], ["--lang", "en,en"], ["--profile", "2d"], ["--archetype", "runner"]]) {
    expect(cli("refused", flag!, value!).code).toBe(1);
    expect(existsSync(join(scratch, "refused"))).toBeFalse();
  }
  mkdirSync(join(scratch, "taken"));
  writeFileSync(join(scratch, "taken/notes.txt"), "mine");
  expect(cli("taken").err).toContain("--force");
  expect(existsSync(join(scratch, "taken/project.ogt"))).toBeFalse();
  expect(cli("taken", "--force").code).toBe(0);
  expect(read("taken/notes.txt")).toBe("mine");
});

test("creation choices infer a premise, preserve overrides, and serialize every choice", () => {
  const c = resolveCreationChoices("温暖的手机横屏平台跳跃", { mood: "cold", music: "folk", look: "16bit-pixel" });
  expect(c).toEqual({ genre: "action-platformer", view: "side-2d", look: "16bit-pixel", mood: "cold", ui: "default", platform: "phone-landscape", input: "touch", players: "single", lang: ["zh"], music: "folk" });
  expect(inferCreationChoices("isometric online shooter", { platform: "desktop", input: "gamepad" })).toMatchObject({ genre: "shooter", view: "isometric-2.5d", players: "online-lockstep", input: "gamepad" });
  expect(resolveCreationChoices("", { look: "16bit-pixel", ui: "default" }).ui).toBe("default");
  expect(() => parseCreationInput({ players: "mmo" })).toThrow("players");
  expect(() => parseCreationInput({ lang: [] })).toThrow("lang");
  expect(() => parseCreationInput({ lang: ["en", "en"] })).toThrow("lang");
  expect(() => resolveCreationChoices("", { look: "missing" })).toThrow("no look preset");
});

test("creation schema labels preserve every stable choice ID", () => {
  const schema = creationSchema(readCatalogue().map(s => s.slug));
  for (const [key, values] of Object.entries(CREATION_CHOICES)) {
    const field = Object.entries(schema.properties).find(([name]) => name === key)?.[1];
    expect(field).toMatchObject({ type: "string", enum: [...values] });
    if (!field || !("x-labels" in field)) throw new Error(`missing labels for ${key}`);
    expect(Object.keys(field["x-labels"])).toEqual([...values]);
    for (const copy of Object.values(field["x-labels"])) expect(copy).toMatchObject({ label: { en: expect.any(String), "zh-CN": expect.any(String) }, description: { en: expect.any(String), "zh-CN": expect.any(String) } });
  }
  expect(schema.properties.look.enum).toHaveLength(37);
  for (const copy of Object.values(schema.properties.look["x-labels"])) expect(copy).toMatchObject({ label: { "zh-CN": expect.any(String) }, description: { "zh-CN": expect.any(String) } });
});

test("art and audio briefs remain available without creating a game", () => {
  expect(readFileSync(join(pkg, "bin/brief.ts"), "utf8")).toBe(readFileSync(join(pkg, "../knowledge/scripts/brief.ts"), "utf8"));
  expect(artBrief(presetLook("16bit-pixel"), "character", "the courier").image?.grid).toEqual({ width: 16, height: 32, scale: 32 });
  const prop = artBrief(presetLook("faceted-low-poly"), "prop", "a signpost");
  expect([prop.medium, prop.model?.faceBudget, prop.model?.textureSize]).toEqual(["model", 200, 64]);
  expect([...RIG_TEMPLATES]).toEqual(["humanoid-side", "humanoid-front", "quadruped", "creature"]);
  expect(rigParts("humanoid-front").filter(p => /^(leg|foot)/.test(p.layer)).map(p => [p.layer, p.side, p.bone, p.required])).toEqual([
    ["leg-upper-l", "l", "leg-upper-l", true], ["leg-lower-l", "l", "leg-lower-l", true], ["foot-l", "l", "foot-l", true],
    ["leg-upper-r", "r", "leg-upper-r", true], ["leg-lower-r", "r", "leg-lower-r", true], ["foot-r", "r", "foot-r", true],
  ]);
  expect(() => rigParts("biped")).toThrow("use one of");
  for (const music of CREATION_CHOICES.music) for (const mood of CREATION_CHOICES.mood) for (const genre of CREATION_CHOICES.genre) {
    const b = audioBrief({ music, mood, genre });
    expect(b.cues.length).toBeGreaterThan(2);
    expect(b.sfx.length).toBeGreaterThan(4);
  }
  expect(authoredArt(presetLook("rubber-hose-animation")).map(a => a.kind)).toEqual(["character", "sprite"]);
  expect(pseudoMessage("{n, plural, one {# coin} other {# coins}} [b]now[/b]")).toBe("[{n, plural, one {# çöíñ} other {# çöíñs}} [b]ñöw[/b] çöíñçö]");
});

test("catalogue suggestions respect profile and mood without installing them", () => {
  const styles = readCatalogue();
  expect(pickStyle("a cosy city builder with a tabletop charm", styles)?.style.slug).toBe("miniature-diorama");
  expect(pickStyle("16-bit platformer, energetic and heroic", styles, "2d")?.style.slug).toBe("16bit-pixel");
  expect(pickStyle("", styles)).toBeNull();
  expect(pickStyle("neon arcade shooter", styles.map(s => ({ ...s, look: { ...s.look, presetReady: false } })))).toBeNull();
  expect(words("Cozy Platformers in Sci-Fi colors, hand-drawn")).toEqual(["cosy", "platformer", "sci", "colour", "hand-drawn", "hand", "drawn"]);
  expect([...CREATION_CHOICES.mood]).toEqual([...MOODS]);
  expect(resolveCreationChoices("", { look: "muted-cinematic-3d" }).mood).toBe("dark");
  expect(resolveCreationChoices("", { look: "muted-cinematic-3d", mood: "warm" }).mood).toBe("warm");
});

test("the packed package creates the same skeleton and briefs from its own presets", () => {
  const installed = join(scratch, "installed");
  const manifest = JSON.parse(readFileSync(join(pkg, "package.json"), "utf8"));
  expect(manifest.files).not.toContain("archetypes");
  expect(manifest.files).not.toContain("template-2d");
  for (const part of [...manifest.files, "package.json"]) cpSync(join(pkg, part), join(installed, part), { recursive: true });
  mkdirSync(join(installed, "template/.og"), { recursive: true });
  symlinkSync("..", join(installed, "template/.og/engine"), "dir");
  symlinkSync(".", join(installed, "template/loop"), "dir");
  expect(files(join(installed, "template"))).not.toContain(".og/engine");
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
  expect(read("packed/src/main.ts")).toBe(readFileSync(join(pkg, "template/src/main.ts"), "utf8"));
  expect(readdirSync(join(scratch, "packed/assets"))).toEqual([".gitkeep", "looks"]);
  const brief = Bun.spawnSync(["bun", join(installed, "bin/brief.ts"), "sprite", "a bat", "--look", "monochrome-pixel", "--json"], { cwd: scratch });
  expect(brief.exitCode, new TextDecoder().decode(brief.stderr)).toBe(0);
  const parsed = JSON.parse(new TextDecoder().decode(brief.stdout));
  expect(parsed.palette.map((c: { hex: string }) => c.hex)).toEqual(presetLook("monochrome-pixel").palette?.colours.map(c => c.hex));
  expect(parsed.references[0].path).toBe(join(installed, "previews/monochrome-pixel.webp"));
});
