// The validator holds the package to the platform's knowledge rules: the
// committed package passes, and each way it could break is named. Mutations
// run on a copy of the package, the template and the engine manifest, and
// each asserts the problem it adds over the copy's own baseline.

import { afterAll, expect, test } from "bun:test";
import { cpSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
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
  const names = Object.keys(packs(payload(real)));
  expect(names).toEqual(["scaffold:origin-engine-ts", "skill:og-ai", "skill:og-audio", "skill:og-build", "skill:og-core", "skill:og-feel", "skill:og-game", "skill:og-i18n", "skill:og-model", "skill:og-platform-sdk", "skill:og-taste", "skill:og-ui"]);
});

test("the bundle is the platform's production layout, text only", () => {
  const b = bundle(real);
  expect(b.schemaVersion).toBe(1);
  for (const s of ["og-core", "og-build", "og-game"]) expect(b.files[`game/skills/${s}/SKILL.md`]).toStartWith(`---\nname: ${s}\n`);
  expect(b.files["game/skills/og-game/reference/index.md"]).toStartWith("<!-- Generated from crates/og-api/api.ogt");
  expect(b.files["game-scaffold/origin-engine-ts/project.ogt"]).toContain("type project");
  expect(Object.keys(b.files).some((p) => p.endsWith(".glb") || p.endsWith(".png") || p.endsWith(".ttf"))).toBeFalse();
});

test("a skill whose front matter name is not its directory is refused", () => {
  const p = added((r) => edit(join(r.pkg, "skills/og-core/SKILL.md"), (s) => s.replace("name: og-core", "name: core")));
  expect(p).toEqual([`game/skills/og-core/SKILL.md: front matter name is "core", its directory is "og-core"; they must be equal`]);
});

test("a skill that names a tool no Session is offered is refused, outside code blocks only", () => {
  const p = added((r) =>
    edit(join(r.pkg, "skills/og-build/SKILL.md"), (s) => `${s}\nThen call \`engine_teleport\`.\n\n\`\`\`sh\n# not a tool: \`engine_warp\`\n\`\`\`\n`),
  );
  expect(p).toHaveLength(1);
  expect(p[0]).toContain("names `engine_teleport`, which is no tool a Session can be offered");
});

test("a binary template file that differs from its digest, or is unlisted, is refused", () => {
  const p = added((r) => {
    const glb = join(r.template, "assets/models/dropship.glb");
    const bytes = readFileSync(glb);
    bytes[100] = bytes[100]! ^ 0xff;
    writeFileSync(glb, bytes);
    writeFileSync(join(r.template, "assets/extra.bin"), new Uint8Array([0, 1, 2, 255]));
  });
  expect(p.sort()).toEqual([
    "game-scaffold/origin-engine-ts/assets/extra.bin: a binary file distribution.json does not list; run `bun scripts/scaffold.ts --write`",
    "game-scaffold/origin-engine-ts/assets/models/dropship.glb: differs from its distribution.json digest; run `bun scripts/scaffold.ts --write` after a deliberate asset change",
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

test("non-UTF-8 and symbolic-link skill files are refused", () => {
  const p = added((r) => {
    writeFileSync(join(r.pkg, "skills/og-game/notes.md"), new Uint8Array([0x66, 0xff, 0xfe, 0x0a]));
    symlinkSync(join(r.pkg, "skills/og-core/SKILL.md"), join(r.pkg, "skills/og-game/link.md"));
  });
  expect(p).toHaveLength(2);
  expect(p.find((x) => x.startsWith("game/skills/og-game/link.md:"))).toContain("a symbolic link");
  expect(p).toContain("game/skills/og-game/notes.md: not UTF-8 text; the platform payload is text only (packages/game-knowledge/scripts/production-bundle.mjs)");
});

test("a skill that names a reference page the generated wiki does not have is refused", () => {
  const p = added((r) => edit(join(r.pkg, "skills/og-taste/SKILL.md"), (s) => `${s}\nThe fog's fields are in \`reference/weather.md\`.\n`));
  expect(p).toEqual([
    "game/skills/og-taste/SKILL.md: names reference/weather.md, which the generated wiki does not have (the wiki's pages: reference/index.md lists them; a module's page is reference/<module>.md)",
  ]);
});

test("each module is taught by the one skill the wiki's index groups it under", () => {
  // og-taste names og-ui's page, og-core stops naming one of its own.
  const p = added((r) => {
    edit(join(r.pkg, "skills/og-taste/SKILL.md"), (s) => `${s}\nThe calls are in \`reference/ui.md\`.\n`);
    edit(join(r.pkg, "skills/og-core/SKILL.md"), (s) => s.replace("`reference/rng.md` (seeded streams)", "the Rng resource (seeded streams)"));
  });
  expect(p).toEqual([
    "skills/og-core/SKILL.md: teaches the rng module (wiki/index.md) but never names reference/rng.md",
    "skills/og-taste/SKILL.md: names reference/ui.md, which og-ui teaches; point to og-ui instead, so each module is taught in one place",
  ]);
  const grouped = added((r) => edit(join(r.pkg, "wiki/index.md"), (s) => s.replace("### `og-ai`", "### `og-enemies`")));
  expect(grouped).toContain("wiki/index.md: groups modules under og-enemies, which is no skill this package ships; fix the modules' `skill` in api.ogt");
});

test("a section reference names a numbered section that exists", () => {
  const p = added((r) =>
    edit(join(r.pkg, "skills/og-audio/SKILL.md"), (s) => `${s}\nSee og-feel §15, og-taste §4, §13 and §9, og-build §Tests.\n\n\`\`\`text\nog-feel §99 in a code block is not a reference\n\`\`\`\n`),
  );
  expect(p).toEqual([
    "skills/og-audio/SKILL.md: refers to og-build §Tests by name; refer to a skill's sections by their numbers (## N.)",
    "skills/og-audio/SKILL.md: refers to og-feel §15, which does not exist there (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14)",
    "skills/og-audio/SKILL.md: refers to og-taste §13, which does not exist there (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)",
  ]);
  const own = added((r) => edit(join(r.pkg, "skills/og-audio/SKILL.md"), (s) => `${s}\nAs §8 says.\n`));
  expect(own).toEqual(["skills/og-audio/SKILL.md: refers to its own §8, which it does not have (its sections: 1, 2, 3, 4, 5, 6, 7)"]);
});

test("a skill or wiki page that calls a UI style a skin is refused; the glTF, collision and 2D skeleton skins pass", () => {
  const p = added((r) => {
    edit(join(r.pkg, "skills/og-ui/SKILL.md"), (s) => `${s}\nCopy the pack as the game's skin and keep its\n\`fallback\` beside the skin file.\n\nThe controller keeps \`skin\` off every surface. A glTF's skins hold its joints. A skin armor { swap torso torso-armor } dresses the rig; a slot the skin leaves alone shows the default's.\n`);
    edit(join(r.pkg, "wiki/physics.md"), (s) => s.replace("The gap kept between it and every surface.", "The gap kept between it and every surface; the HUD wears the skin's panel."));
  });
  expect(p).toEqual([
    "wiki/physics.md: calls a UI style a skin (\"the HUD wears the skin's panel.\"); og-ui's word is style (type style, og.load.style, styles/<name>/)",
    "skills/og-ui/SKILL.md: calls a UI style a skin (\"Copy the pack as the game's skin and keep its `fallback` beside the skin file.\"); og-ui's word is style (type style, og.load.style, styles/<name>/)",
  ]);
});

test("the style catalogue covers 37 styles, each complete and listed in og-taste", () => {
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string; checks: string[]; look: { support: string; missing: string[] } }[] };
  expect(cat.styles).toHaveLength(37);
  for (const s of cat.styles) {
    expect(s.checks).toHaveLength(3);
    if (s.look.support !== "full") expect(s.look.missing.length).toBeGreaterThan(0);
  }
  const b = bundle(real);
  expect(b.files["game/skills/og-taste/looks/catalogue.json"]).toBe(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8"));
});

test("an incomplete, dishonest or unknown catalogue entry is refused", () => {
  const p = added((r) =>
    edit(join(r.pkg, "looks/catalogue.json"), (s) => {
      const cat = JSON.parse(s);
      const [a, b, c] = cat.styles;
      a.checks = a.checks.slice(0, 2);
      b.look.support = "planned";
      b.look.missing = [];
      c.look.primitives = [...c.look.primitives, "Stylise.posterise"];
      c.look.profiles = ["2d", "4d"];
      cat.styles.push({ ...cat.styles[3] });
      cat.styles.push({ ...cat.styles[4], slug: "a-new-look" });
      return JSON.stringify(cat, null, 2);
    }),
  );
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string }[] };
  const [a, b, c, d] = cat.styles.map((s) => s.slug);
  const taste = readFileSync(join(real.pkg, "skills/og-taste/SKILL.md"), "utf8");
  const row = (slug: string) => new RegExp(`^\\| \`${slug}\` \\| (.*) \\|$`, "m").exec(taste)![1]!.split(" | ").map((x) => x.replaceAll("`", "")).join(", ");
  expect(p.sort()).toEqual(
    [
      `looks/catalogue.json: ${a}: needs exactly three checks a critic can judge on a frame`,
      `looks/catalogue.json: ${b}: support planned must name what is missing (the primitive or the authored art)`,
      `looks/catalogue.json: ${c}: names Stylise.posterise, which the generated API does not have`,
      `looks/catalogue.json: ${c}: look.profiles must name the profiles the style draws for, one or both of 2d, 3d (create matches a premise only among the game's)`,
      `looks/catalogue.json: ${d}: a second style with this slug`,
      "looks/catalogue.json: a-new-look: look.preset is templates/looks/" + cat.styles[4]!.slug + ".ogt, expected templates/looks/a-new-look.ogt",
      "skills/og-taste/SKILL.md: the style catalogue (§12) does not list `a-new-look`",
      `skills/og-taste/SKILL.md: §12's row for \`${b}\` reads ${row(b!)}; the catalogue says ${row(b!).replace(/, full, /, ", planned, ")}`,
    ].sort(),
  );
});

test("every og.* name and Type.field a skill writes exists in the generated API", () => {
  const p = added((r) =>
    edit(
      join(r.pkg, "skills/og-game/SKILL.md"),
      (s) =>
        `${s}\nCall \`og.render.applyLooks(world, look)\`, set \`og.Sprite.shadowy\`, read \`og.Wobble\` and \`SpriteDesc.dropShadows\`; \`og.load.look\`, \`og.Stylise.paintCanvas\`, \`GameDesc.look\` and \`away.og.ts\` are fine.\n\n\`\`\`ts\nog.ui.textAreaz(world, {});\n\`\`\`\n`,
    ),
  );
  const why = "which the generated API (api.ogt) does not have; check the name in reference/ or packages/engine/types/index.d.ts";
  expect(p.sort()).toEqual(
    ["og.render.applyLooks", "og.Sprite.shadowy", "og.Wobble", "SpriteDesc.dropShadows", "og.ui.textAreaz"].map((n) => `skills/og-game/SKILL.md: names \`${n}\`, ${why}`).sort(),
  );
});

test("a field written as `key: value` or in a token's object literal is one the API has", () => {
  const p = added((r) =>
    edit(
      join(r.pkg, "skills/og-audio/SKILL.md"),
      (s) =>
        `${s}\nLoop it with \`looping: true\`; \`loop: true\` and \`runs: 2\` are fine.\n\n\`\`\`ts\nworld.insert(e, og.AudioSource, { sound, loop: true, volumn: 0.5 });\n\`\`\`\n`,
    ),
  );
  const why = "which the generated API (api.ogt) does not have; check the name in reference/ or packages/engine/types/index.d.ts";
  expect(p.sort()).toEqual(["looping", "og.AudioSource.volumn"].map((n) => `skills/og-audio/SKILL.md: names \`${n}\`, ${why}`).sort());
});

test("og-taste §12's table is held to the catalogue, row by row", () => {
  const p = added((r) =>
    edit(join(r.pkg, "skills/og-taste/SKILL.md"), (s) =>
      s.replace("| `voxel-worlds` | 3D rendering | Voxel worlds | full | `pixel` |", "| `voxel-worlds` | 3D rendering | Voxel worlds | partial | `cartoon` |").replace(/^\| `neon-vector` \|.*\n/m, "Also `neon-vector`.\n"),
    ),
  );
  expect(p.sort()).toEqual([
    "skills/og-taste/SKILL.md: §12's row for `voxel-worlds` reads 3D rendering, Voxel worlds, partial, cartoon; the catalogue says 3D rendering, Voxel worlds, full, pixel",
    "skills/og-taste/SKILL.md: §12's table has no row for `neon-vector` (| slug | family | style | support | ui |)",
  ]);
});

/**
 * Phase 4's APIs, each with the skill that teaches it: every name must
 * appear in that skill (and, by the validator, exist in the generated API).
 */
const TAUGHT: Record<string, string[]> = {
  "og-game": [
    "GameDesc.look", "og.load.look", "og.load.lookInfo", "og.load.palette", "og.render.applyLook", "og.render.look", "og.Stylise", "og.MaterialStyle",
    "@canvas/paper", "@palettes/handheld4", "RenderCapabilities.stylise",
    "og.Environment", "og.load.environment", "type environment", "toneMapping", "agx", "autoExposure", "adaptSpeedUp", "adaptSpeedDown",
    "shadowSoftness", "cascadeBlend", "pointShadows", "taaHistory", "taaSharpen", "fxaa",
    "clearcoat", "sheen", "transmission", "iridescence", "anisotropy",
    "og.Sprite.dropShadow", "SpriteDesc.dropShadow", "stepFps", "og.load.textureArray", "SpriteDesc.arrayLayer", "og.Sprite.arrayLayer", "FlipbookInfo.array", "array true",
    "StreamBudget", "StreamStatus", "og.stream.release", "og.physics.raycast", "og.physics.shapeCast", "og.physics.overlap", "lanes", ".vox", "compress true",
  ],
  "og-taste": ["extends", "GameDesc.look", "--premise", "--look", "og.render.look", "looks/catalogue.json", "hatchingSpace object", "hatchingDepth"],
  "og-ui": ["og.ui.textArea", "og.ui.textField", "compose:", "commit:", "ellipsis", "maxLines", "fallbackFonts", "og.ui.a11y", "og.A11yAction", "og.ui.zoomView", "og.Pointers", "og.ui.hasIcon", "kerning"],
  "og-i18n": ["runtime true", "fallbackFonts", "shaped", "Kerning"],
  "og-build": ["cooked/stream/", "environment maps", "texture arrays", "palettes", "looks and LUTs", ".vox", "compress true", "stream true", "isolation", "OG_LANES", "dump_lanes", "compose:"],
  "og-audio": ["stream true", "chunk 4", "chunks", "og.load.soundInfo"],
  "og-feel": ["stepFps"],
  "og-core": ["lanes"],
};

test("every phase 4 API is taught by its skill", () => {
  const b = bundle(real);
  const missing: string[] = [];
  for (const [skill, names] of Object.entries(TAUGHT)) {
    const text = b.files[`game/skills/${skill}/SKILL.md`]!;
    for (const n of names) if (!text.includes(n)) missing.push(`${skill}: ${n}`);
  }
  expect(missing).toEqual([]);
});

/** The model review's commands, tool and generators: og-model teaches each (og-game §18 owns the generators' fields). */
const MODEL_REVIEW = [
  "view:<entity>:front", "view:<entity>:back", "view:<entity>:side", "view:<entity>:top", "view:<entity>:orbit:<degrees>", "view:off", "turntable:<entity>:<n>",
  "og compare", "--mask", "--align", "--key", "--sheet", "silhouette IoU", "ΔE00",
  "og.render.shapeGeometry", "og.render.extrudeGeometry", "og.render.latheGeometry", "og.render.sweepGeometry", "og.render.csgGeometry", "og.render.mergeGeometries", "og.render.icosahedronGeometry", "og.Shape", "og.Path2",
  "Blockout", "Structure", "Form", "Material", "Surface detail", "Lighting", "Interaction", "Optimisation",
];

test("og-model teaches the model review: the views, og compare, the generators and the eight passes", () => {
  const text = bundle(real).files["game/skills/og-model/SKILL.md"]!;
  expect(MODEL_REVIEW.filter((n) => !text.includes(n))).toEqual([]);
  // A generator name it writes must exist: a typo is refused like any skill's.
  const p = added((r) => edit(join(r.pkg, "skills/og-model/SKILL.md"), (s) => s.replace("og.render.sweepGeometry`", "og.render.sweptGeometry`")));
  expect(p).toEqual(["skills/og-model/SKILL.md: names `og.render.sweptGeometry`, which the generated API (api.ogt) does not have; check the name in reference/ or packages/engine/types/index.d.ts"]);
});

/** The two doc rows of threejs-migrate: og-build points to the model review, og-model to the asset route. */
const ROUTES: Record<string, string[]> = {
  "og-build": ["og compare <reference.png> <render.png>", "turntable:<entity>:<n>", "og-model §3", "og-model §4"],
  "og-model": ["type model", "og-game §18", "og.load.gltf(world, \"models/lantern.ogt\")", "og cook --report", "`pivot`"],
};

test("og-build points to og compare and og-model names the type model route", () => {
  const b = bundle(real);
  const missing: string[] = [];
  for (const [skill, names] of Object.entries(ROUTES)) {
    const text = b.files[`game/skills/${skill}/SKILL.md`]!;
    for (const n of names) if (!text.includes(n)) missing.push(`${skill}: ${n}`);
  }
  expect(missing).toEqual([]);
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
  const text = bundle(real).files["game/skills/og-game/guides/three-to-og.md"];
  expect(text).toBeDefined();
  expect(THREE_TO_OG.filter((n) => !text!.includes(n))).toEqual([]);
});

test("a guide is held to the skills' rules: its API names, sections, pages and who names it", () => {
  const why = "which the generated API (api.ogt) does not have; check the name in reference/ or packages/engine/types/index.d.ts";
  const p = added((r) => {
    edit(join(r.pkg, "guides/three-to-og.md"), (s) => `${s}\nPick with \`og.render.raycasts(world, ray)\`; see §42, og-feel §99 and \`reference/raycast.md\`.\n`);
    edit(join(r.pkg, "skills/og-ui/SKILL.md"), (s) => `${s}\nThree.js's CSS2DRenderer is in \`guides/three-to-ui.md\`.\n`);
    writeFileSync(join(r.pkg, "guides/notes.md"), "# Notes\n");
  });
  expect(p.sort()).toEqual(
    [
      `guides/three-to-og.md: names \`og.render.raycasts\`, ${why}`,
      "guides/three-to-og.md: names reference/raycast.md, which the generated wiki does not have (a module's page is reference/<module>.md)",
      "guides/three-to-og.md: refers to its own §42, which it does not have (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)",
      "guides/three-to-og.md: refers to og-feel §99, which does not exist there (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14)",
      "guides/notes.md: not in GUIDES (scripts/validate.ts); list it there and name it from og-game, or remove it",
      "skills/og-ui/SKILL.md: names guides/three-to-ui.md, which this package does not ship (its guides: three-to-og)",
    ].sort(),
  );
  const unnamed = added((r) => edit(join(r.pkg, "skills/og-game/SKILL.md"), (s) => s.replaceAll("guides/three-to-og.md", "the guide")));
  expect(unnamed).toEqual(["skills/og-game/SKILL.md: never names guides/three-to-og.md, so no agent finds the guide; name it where its subject is taught"]);
  const unmounted = added((r) => edit(join(r.pkg, "knowledge.json"), (s) => s.replace('"to": "game/skills/og-game/guides/"', '"to": "game/skills/og-game/notes/"')));
  expect(unmounted).toContain("guides/three-to-og.md: missing, or not mounted at game/skills/og-game/guides/ (knowledge.json mounts guides/ there)");
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
  expect(p).toEqual(["crates/og-cli/src/migrate.rs: cites three-to-og §12, which guides/three-to-og.md does not have (its sections: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)"]);
  // And the real source cites some, so the check reads something.
  expect(readFileSync(join(real.pkg, "../../crates/og-cli/src/migrate.rs"), "utf8")).toMatch(/three-to-og §\d+/);
});
