// The validator holds the package to the platform's knowledge rules: the
// committed package passes, and each way it could break is named. Mutations
// run on a copy of the package, the template and the engine manifest, and
// each asserts the problem it adds over the copy's own baseline.

import { afterAll, expect, test } from "bun:test";
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defaultRoots, payload, packs, type Roots } from "../scripts/lib.ts";
import { bundle, validate } from "../scripts/validate.ts";

/** packages/engine's version: what the scaffold pins. */
const ENGINE_VERSION = (JSON.parse(readFileSync(join(defaultRoots().engine, "package.json"), "utf8")) as { version: string }).version;

const temps: string[] = [];
afterAll(() => {
  for (const t of temps) rmSync(t, { recursive: true, force: true });
});

const real = defaultRoots();

function copy(): Roots {
  const dir = mkdtempSync(join(tmpdir(), "og-knowledge-"));
  temps.push(dir);
  const roots = { pkg: join(dir, "knowledge"), template: join(dir, "template"), engine: join(dir, "engine") };
  cpSync(real.pkg, roots.pkg, { recursive: true, filter: (s) => !s.includes("node_modules") });
  cpSync(real.template, roots.template, { recursive: true, filter: (s) => !/\/(cooked|dist|node_modules)(\/|$)/.test(s) });
  cpSync(join(real.engine, "package.json"), join(roots.engine, "package.json"));
  cpSync(join(real.engine, "schema/api.schema.json"), join(roots.engine, "schema/api.schema.json"));
  return roots;
}

/** The problems `mutate` adds to the copy's baseline. */
function added(mutate: (r: Roots) => void): string[] {
  const roots = copy();
  const before = new Set(validate(roots));
  mutate(roots);
  return validate(roots).filter((p) => !before.has(p));
}

const edit = (path: string, f: (s: string) => string) => writeFileSync(path, f(readFileSync(path, "utf8")));

test("the committed package is valid against the platform rules", () => {
  expect(validate(real)).toEqual([]);
  // The skeleton is the platform's only pack from the engine: no skills.
  expect(Object.keys(packs(payload(real)))).toEqual(["scaffold:origin-engine-ts"]);
});

test("the bundle is the platform's production layout, text only", () => {
  const b = bundle(real);
  expect(b.schemaVersion).toBe(1);
  expect(Object.keys(b.files).every((p) => p.startsWith("game-scaffold/origin-engine-ts/"))).toBeTrue();
  expect(b.files["game-scaffold/origin-engine-ts/project.ogt"]).toContain("type project");
  expect(Object.keys(b.files).some((p) => p.endsWith(".glb") || p.endsWith(".png") || p.endsWith(".ttf"))).toBeFalse();
});

const guide = (r: Roots, name: string) => join(r.pkg, `wiki/guide-${name}.md`);

test("a guide that names a tool no Session is offered is refused, outside code blocks only", () => {
  const p = added((r) => edit(guide(r, "testing"), (s) => `${s}\nThen call \`engine_teleport\`.\n\n\`\`\`sh\n# not a tool: \`engine_warp\`\n\`\`\`\n`));
  expect(p).toHaveLength(1);
  expect(p[0]).toContain("names `engine_teleport`, which is no tool a Session can be offered");
});

test("a binary template file that differs from its digest, or is unlisted, is refused", () => {
  const p = added((r) => {
    writeFileSync(join(r.template, "assets/changed.bin"), new Uint8Array([0, 1, 2, 3]));
    edit(join(r.pkg, "scaffold/distribution.json"), text => {
      const dist = JSON.parse(text);
      dist.files.push({ path: "assets/changed.bin", bytes: 4, sha256: "0".repeat(64), licence: "test", source: "test" });
      return JSON.stringify(dist);
    });
    writeFileSync(join(r.template, "assets/extra.bin"), new Uint8Array([0, 1, 2, 255]));
  });
  expect(p.sort()).toEqual([
    "game-scaffold/origin-engine-ts/assets/changed.bin: differs from its distribution.json digest; run `bun scripts/scaffold.ts --write` after a deliberate asset change",
    "game-scaffold/origin-engine-ts/assets/extra.bin: a binary file distribution.json does not list; run `bun scripts/scaffold.ts --write`",
  ]);
});

test("a hand-written or orphaned wiki page is refused", () => {
  const p = added((r) => writeFileSync(join(r.pkg, "wiki/notes.md"), "# My notes\n"));
  expect(p.sort()).toEqual([
    "wiki/notes.md: not generated from api.ogt (no generated header); wiki pages are never hand-written: run `og api gen --wiki`",
    "wiki/notes.md: not linked from wiki/index.md (a page for a module the IDL no longer has?); run `og api gen --wiki`",
  ]);
});

test("a template without a required file or project key is refused", () => {
  const p = added((r) => {
    rmSync(join(r.template, "tsconfig.json"));
    edit(join(r.template, "project.ogt"), (s) => s.replace(/^seed .*$/m, ""));
  });
  expect(p.sort()).toEqual([
    "game-scaffold/origin-engine-ts/project.ogt: no `seed`",
    "game-scaffold/origin-engine-ts/tsconfig.json: the template has no tsconfig.json (../template/tsconfig.json); scaffold/README.md says every project starts with it",
  ]);
});

test("an overlay named like an existing platform overlay is refused", () => {
  const p = added((r) => edit(join(r.pkg, "knowledge.json"), (s) => s.replaceAll("origin-engine-ts", "origin-engine")));
  expect(p).toContain('knowledge.json: overlay "origin-engine" is already a platform scaffold overlay; preparing knowledge would replace it');
});

test("an engine version the pin and project.ogt do not name is refused", () => {
  const p = added((r) => edit(join(r.engine, "package.json"), (s) => s.replace(/"version": "[^"]+"/, '"version": "9.9.9"')));
  expect(p.sort()).toEqual([
    `game-scaffold/origin-engine-ts/origin-engine.json: pins engine ${ENGINE_VERSION}, packages/engine is 9.9.9`,
    `game-scaffold/origin-engine-ts/project.ogt: engine "${ENGINE_VERSION}", packages/engine is 9.9.9`,
  ]);
});

test("a guide that points at a page no reader opens, or at a module the wiki lacks, is refused", () => {
  const p = added((r) => edit(guide(r, "looks"), (s) => `${s}\nThe fog's fields are in \`reference/atmosphere.md\` and \`og api show weather\`; \`og api show og.Fog\` is fine.\n`));
  expect(p).toEqual([
    "wiki/guide-looks.md: names reference/atmosphere.md, which no reader opens: name `og api show <module>` for a module, `og api show <exact name>` for one name, or `og api find <words>`",
    "wiki/guide-looks.md: names `og api show weather`, and the generated wiki has no module weather (wiki/index.md lists them; write a type or function as its full name, og.<name>)",
  ]);
});

test("a section reference names a numbered section that exists", () => {
  const sectionsOf = (name: string) => [...readFileSync(guide(real, name), "utf8").matchAll(/^### `[a-z0-9-]+ §(\d+)` /gm)].map((m) => m[1]).join(", ");
  const p = added((r) =>
    edit(guide(r, "screens"), (s) => `${s}\nSee determinism §15, art-review §4, §15 and §9, testing §Tests.\n\n\`\`\`text\ndeterminism §99 in a code block is not a reference\n\`\`\`\n`),
  );
  expect(p.sort()).toEqual([
    "wiki/guide-screens.md: refers to art-review §15, which does not exist there (its sections: " + sectionsOf("art-review") + ")",
    "wiki/guide-screens.md: refers to art-review §9, which does not exist there (its sections: " + sectionsOf("art-review") + ")",
    "wiki/guide-screens.md: refers to determinism §15, which does not exist there (its sections: " + sectionsOf("determinism") + ")",
    "wiki/guide-screens.md: refers to testing §Tests by name; refer to a guide's sections by their numbers (## N.)",
  ].sort());
  const own = added((r) => edit(guide(r, "screens"), (s) => `${s}\nAs §42 says.\n`));
  expect(own).toEqual(["wiki/guide-screens.md: refers to its own §42, which it does not have (its sections: " + sectionsOf("screens") + ")"]);
  const other = added((r) => edit(guide(r, "screens"), (s) => `${s}\nSee og-zzz §1 and art-review §2.\n`));
  expect(other).toEqual(["wiki/guide-screens.md: refers to og-zzz §1, and there is no guide og-zzz"]);
});

test("a wiki page that calls a UI style a skin is refused; the glTF, collision and 2D skeleton skins pass", () => {
  const p = added((r) => {
    edit(guide(r, "screens"), (s) => `${s}\nCopy the pack as the game's skin and keep its\n\`fallback\` beside the skin file.\n\nThe controller keeps \`skin\` off every surface. A glTF's skins hold its joints. A skin armor { swap torso torso-armor } dresses the rig; a slot the skin leaves alone shows the default's.\n`);
    edit(join(r.pkg, "wiki/physics.md"), (s) => s.replace("The gap kept between it and every surface.", "The gap kept between it and every surface; the HUD wears the skin's panel."));
  });
  expect(p.sort()).toEqual([
    "wiki/guide-screens.md: calls a UI style a skin (\"Copy the pack as the game's skin and keep its `fallback` beside the skin file.\"); og-ui's word is style (type style, og.load.style, styles/<name>/)",
    "wiki/physics.md: calls a UI style a skin (\"the HUD wears the skin's panel.\"); og-ui's word is style (type style, og.load.style, styles/<name>/)",
  ].sort());
});

test("the style catalogue covers 37 styles, each complete", () => {
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string; checks: string[]; look: { support: string; missing: string[] } }[] };
  expect(cat.styles).toHaveLength(37);
  for (const s of cat.styles) {
    expect(s.checks).toHaveLength(3);
    if (s.look.support !== "full") expect(s.look.missing.length).toBeGreaterThan(0);
  }
});

test("an incomplete, dishonest or unknown catalogue entry is refused", () => {
  const p = added((r) =>
    edit(join(r.pkg, "looks/catalogue.json"), (s) => {
      const cat = JSON.parse(s);
      const [a, b, c] = cat.styles;
      a.checks = a.checks.slice(0, 2);
      delete a.intended;
      c.intended = `Hard steps are the style, not defects, in ${c.name}.`;
      b.look.support = "planned";
      b.look.missing = [];
      c.look.primitives = [...c.look.primitives, "Stylise.posterise"];
      c.look.profiles = ["2d", "4d"];
      c.look.layers = ["conform", "authored"];
      c.look.family = "pixel-moon";
      c.look.views = ["first-person-3d", "sideways"];
      c.look.mood = "gloomy";
      c.preview = "previews/nowhere.webp";
      cat.styles.push({ ...cat.styles[3] });
      cat.styles.push({ ...cat.styles[4], slug: "a-new-look" });
      return JSON.stringify(cat, null, 2);
    }),
  );
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string; look: { support: string } }[] };
  const [a, b, c, d] = cat.styles.map((s) => s.slug);
  expect(p.sort()).toEqual(
    [
      `looks/catalogue.json: ${a}: needs exactly three checks a critic can judge on a frame`,
      `looks/catalogue.json: ${b}: support planned must name what is missing (the primitive or the authored art)`,
      `looks/catalogue.json: ${c}: names Stylise.posterise, which the generated API does not have`,
      `looks/catalogue.json: ${c}: look.profiles must name the profiles the style draws for, one or both of 2d, 3d (create matches a premise only among the game's)`,
      `looks/catalogue.json: ${c}: look.layers must include frame and name only frame, conform, runtime, authored, each once`,
      `looks/catalogue.json: ${c}: support ${cat.styles[2]!.look.support} disagrees with look.layers (content exactly when authored art is needed)`,
      `looks/catalogue.json: ${c}: look.family is gone (schema 3): the sample scene is tools/art-samples/scenes.json's, not a reader's`,
      `looks/catalogue.json: ${c}: look.views must name the views the style suits, each once, among side-2d, top-down-2d, isometric-2.5d, third-person-3d, first-person-3d`,
      `looks/catalogue.json: ${c}: look.views names first-person-3d, which its profiles (2d, 4d) do not draw`,
      `looks/catalogue.json: ${c}: look.views names sideways, which its profiles (2d, 4d) do not draw`,
      `looks/catalogue.json: ${c}: look.mood gloomy is none of bright, dark, warm, cold, soft, high-contrast`,
      `looks/catalogue.json: ${c}: look.authored names the art exactly when look.layers needs authored art`,
      `looks/catalogue.json: ${c}: preview is previews/nowhere.webp, expected previews/${c}.webp`,
      `looks/catalogue.json: ${a}: needs intended, one sentence naming the traits that look like defects but are the style ("… are the style, not …")`,
      `looks/catalogue.json: ${c}: intended names the style; a blind critic reads it`,
      `looks/catalogue.json: ${d}: a second style with this slug`,
      "looks/catalogue.json: a-new-look: preview is previews/" + cat.styles[4]!.slug + ".webp, expected previews/a-new-look.webp",
      "looks/catalogue.json: a-new-look: look.preset is templates/looks/" + cat.styles[4]!.slug + ".ogt, expected templates/looks/a-new-look.ogt",
    ].sort(),
  );
});

test("every og.* name and Type.field a guide writes exists in the generated API", () => {
  const p = added((r) =>
    edit(
      guide(r, "start"),
      (s) =>
        `${s}\nCall \`og.render.applyLooks(world, look)\`, set \`og.Sprite.shadowy\`, read \`og.Wobble\` and \`SpriteDesc.dropShadows\`; \`og.load.look\`, \`og.Stylise.paintCanvas\`, \`GameDesc.look\` and \`away.og.ts\` are fine.\n\n\`\`\`ts\nog.ui.textAreaz(world, {});\n\`\`\`\n`,
    ),
  );
  const why = "which the generated API (api.ogt) does not have; check the name with og api find or in packages/engine/types/index.d.ts";
  expect(p.sort()).toEqual(
    ["og.render.applyLooks", "og.Sprite.shadowy", "og.Wobble", "SpriteDesc.dropShadows", "og.ui.textAreaz"].map((n) => `wiki/guide-start.md: names \`${n}\`, ${why}`).sort(),
  );
});

test("a field written as `key: value` or in a token's object literal is one the API has", () => {
  const p = added((r) =>
    edit(
      guide(r, "screens"),
      (s) =>
        `${s}\nLoop it with \`looping: true\`; \`loop: true\` and \`runs: 2\` are fine.\n\n\`\`\`ts\nworld.insert(e, og.AudioSource, { sound, loop: true, volumn: 0.5 });\n\`\`\`\n`,
    ),
  );
  const why = "which the generated API (api.ogt) does not have; check the name with og api find or in packages/engine/types/index.d.ts";
  expect(p.sort()).toEqual(["looping", "og.AudioSource.volumn"].map((n) => `wiki/guide-screens.md: names \`${n}\`, ${why}`).sort());
});

/** What the three-to-og guide maps (the threejs-migrate row): every name must appear in it. */
const THREE_TO_OG = [
  "og.Parent", "og.Transform", "og.MatrixWorld", "world.spawn", "world.find",
  "og.render.boxGeometry", "og.render.extrudeGeometry", "og.render.latheGeometry", "og.render.bufferGeometry", "og.render.mergeGeometries", "og.render.csgGeometry", "og.render.sweepGeometry", "og.Shape", "og.Path2", "type model",
  "StandardMaterialDesc", "og.render.standardMaterial", "unlit: true", "color.fromHex", "og.render.updateMaterial",
  "og.load.gltf", "og.load.texture", "og.load.environment", "world.instantiate",
  "defineSystem", "time.dt", "og.Rng", "og.Camera",
  "og.render.raycast", "screenToRay",
  "ShaderMaterial", "type shader", "InstancedMesh", "OrbitControls", "og.OrbitCamera", "og.systems.cameras",
  "og migrate three",
];

test("the three-to-og guide maps three.js's classes and idioms and names the gaps", () => {
  const text = readFileSync(guide(real, "three-to-og"), "utf8");
  expect(THREE_TO_OG.filter((n) => !text.includes(n))).toEqual([]);
});

/** A copy laid out as the repository is (packages/knowledge, packages/engine, crates/), so the guide's citers are read. */
function nested(): Roots {
  const dir = mkdtempSync(join(tmpdir(), "og-knowledge-repo-"));
  temps.push(dir);
  const roots = { pkg: join(dir, "packages/knowledge"), template: join(dir, "template"), engine: join(dir, "packages/engine") };
  cpSync(real.pkg, roots.pkg, { recursive: true, filter: (s) => !s.includes("node_modules") });
  cpSync(real.template, roots.template, { recursive: true, filter: (s) => !/\/(cooked|dist|node_modules)(\/|$)/.test(s) });
  cpSync(join(real.engine, "package.json"), join(roots.engine, "package.json"));
  cpSync(join(real.engine, "schema/api.schema.json"), join(roots.engine, "schema/api.schema.json"));
  cpSync(join(real.pkg, "../../crates/og-cli/src/migrate.rs"), join(dir, "crates/og-cli/src/migrate.rs"));
  return roots;
}

test("every section og migrate three cites is one the guide has", () => {
  const roots = nested();
  const before = new Set(validate(roots));
  edit(join(roots.pkg, "../../crates/og-cli/src/migrate.rs"), (s) => `${s}\n// see three-to-og §12\n`);
  const p = validate(roots).filter((x) => !before.has(x));
  expect(p).toEqual(["crates/og-cli/src/migrate.rs: cites three-to-og §12, which wiki/guide-three-to-og.md does not have (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)"]);
  // And the real source cites some, so the check reads something.
  expect(readFileSync(join(real.pkg, "../../crates/og-cli/src/migrate.rs"), "utf8")).toMatch(/three-to-og §\d+/);
});
