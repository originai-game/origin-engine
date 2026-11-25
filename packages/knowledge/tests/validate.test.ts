// The validator holds the package to the platform's knowledge rules: the
// committed package passes, and each way it could break is named. Mutations
// run on a copy of the package, the template and the engine manifest, and
// each asserts the problem it adds over the copy's own baseline.

import { afterAll, expect, test } from "bun:test";
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defaultRoots, payload, packs, type Roots } from "../scripts/lib.ts";
import { validate } from "../scripts/validate.ts";

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

const guide = (r: Roots, name: string) => join(r.pkg, `wiki/guide-${name}.md`);

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

test("brief and creation vocabularies stay in lockstep with Rust cook/synth constants and catalogue.json", async () => {
  const repo = join(real.pkg, "..", "..");
  const { MOODS, SFX_PRESETS } = await import("../scripts/brief.ts");
  const { CREATION_CHOICES } = await import("../scripts/creation.ts");
  const { resolveCreationChoices } = await import("../../create/bin/create.ts");

  // SFX_PRESETS === crates/og-audio/src/synth.rs PRESETS
  const synth = readFileSync(join(repo, "crates/og-audio/src/synth.rs"), "utf8");
  const synthBlock = synth.slice(synth.indexOf("pub const PRESETS"), synth.indexOf("];", synth.indexOf("pub const PRESETS")));
  const rustPresets = [...synthBlock.matchAll(/\("([a-z0-9_-]+)",\s*"/g)].map((m) => m[1]!);
  expect([...SFX_PRESETS]).toEqual(rustPresets);

  // MOODS === crates/og-bundle/src/look.rs MOODS
  const lookRs = readFileSync(join(repo, "crates/og-bundle/src/look.rs"), "utf8");
  const moodsMatch = /pub const MOODS:\s*\[&str;\s*\d+\]\s*=\s*\[([^\]]+)\]/m.exec(lookRs);
  const rustMoods = [...(moodsMatch?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]!);
  expect([...MOODS]).toEqual(rustMoods);

  // CREATION_CHOICES.ui === catalogue.json ui (including tactical-arctic)
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { ui: string[]; styles: unknown[] };
  expect([...CREATION_CHOICES.ui]).toEqual(cat.ui);
  expect(CREATION_CHOICES.ui).toContain("tactical-arctic");

  // resolveCreationChoices honors explicit custom catalogue path (3-arg signature)
  const dir = mkdtempSync(join(tmpdir(), "og-cat-"));
  temps.push(dir);
  const customCat = join(dir, "catalogue.json");
  const pixel = (cat.styles as Array<{ slug: string }>).find((s) => s.slug === "pixel")!;
  writeFileSync(customCat, JSON.stringify({ ...cat, styles: [pixel] }));
  expect(resolveCreationChoices("", { form: "2d", ui: "tactical-arctic" }, customCat)).toMatchObject({ look: "pixel", ui: "tactical-arctic" });
  expect(() => resolveCreationChoices("", { form: "3d", look: "toy-3d" }, customCat)).toThrow(/no look preset 'toy-3d'/);
});

test("IDL splits match wasm-modules.json, SW never precaches unused split modules, and referencedModules resolves destruction by profile", async () => {
  const { mkdirSync } = await import("node:fs");
  const bindings = JSON.parse(readFileSync(join(real.engine, "types/bindings.json"), "utf8")) as {
    splits: Record<string, string[]>;
    components: Array<{ name: string; split?: string[] }>;
    resources: Array<{ name: string; split?: string[] }>;
    events: Array<{ name: string; split?: string[] }>;
  };
  const wasmModules = JSON.parse(readFileSync(join(real.engine, "scripts/wasm-modules.json"), "utf8")) as {
    modules: Record<string, string[]>;
  };
  const defined = new Set(Object.keys(wasmModules.modules));
  const declared = new Set<string>();
  for (const ms of Object.values(bindings.splits)) for (const m of ms) declared.add(m);
  for (const r of [...bindings.components, ...bindings.resources, ...bindings.events]) {
    for (const m of r.split ?? []) declared.add(m);
  }
  for (const m of declared) {
    if (m === "destruction") {
      expect(defined.has("destruction2d")).toBe(true);
      expect(defined.has("destruction3d")).toBe(true);
    } else {
      expect(defined.has(m), `IDL split module '${m}' must exist in wasm-modules.json`).toBe(true);
    }
  }

  const { serviceWorkerLists } = await import("../../engine/src/node/web-build.ts");
  const sw = serviceWorkerLists(
    [
      "index.html",
      "game.abc123456789.js",
      "origin-engine.abc123456789.js",
      "origin-engine_bg.abc123456789.wasm",
      "origin-engine-lanes.abc123456789.js",
      "origin-engine-lanes_bg.abc123456789.wasm",
      "modules/physics.111111111111.wasm",
      "modules/physics2d.222222222222.wasm",
      "modules/world3d.333333333333.wasm",
      "modules-lanes/physics.444444444444.wasm",
      "modules-lanes/world3d.555555555555.wasm",
    ],
    new Set(),
    new Set(),
    {
      single: new Set(["modules/physics.111111111111.wasm", "modules/physics2d.222222222222.wasm"]),
      lanes: new Set(["modules-lanes/physics.444444444444.wasm"]),
    },
  );
  expect(sw.files.some((f) => f.startsWith("modules"))).toBe(false);
  expect(sw.single).toContain("modules/physics.111111111111.wasm");
  expect(sw.single).toContain("modules/physics2d.222222222222.wasm");
  expect(sw.single).not.toContain("modules/world3d.333333333333.wasm");
  expect(sw.lanes).toContain("modules-lanes/physics.444444444444.wasm");
  expect(sw.lazy).toContain("modules/world3d.333333333333.wasm");
  expect(sw.lazy).toContain("modules-lanes/world3d.555555555555.wasm");

  const { referencedModules } = await import("../../engine/src/node/modules.ts");
  const proj = mkdtempSync(join(tmpdir(), "og-mods-"));
  temps.push(proj);
  mkdirSync(join(proj, "cooked"), { recursive: true });
  mkdirSync(join(proj, "src"), { recursive: true });
  writeFileSync(join(proj, "cooked/.cook.ogt"), 'type cook\nversion 1\n---\nprofile "2d"\n');
  const mainTs = join(proj, "src/main.ts");
  const unusedTs = join(proj, "src/unused.ts");
  writeFileSync(
    mainTs,
    '// og.terrain.load(world, "t")\n/* og.video.play(world, e) */\nog.destruction.explode2d(world, {});\nog.render.extrudeGeometry(world, {});\n',
  );
  writeFileSync(unusedTs, "og.story.start(world, {});\n");
  const mods = referencedModules(proj, bindings, [mainTs]);
  expect(mods).toEqual(["destruction2d", "physics", "physics2d", "shape"]);
  expect(mods).not.toContain("destruction");
  expect(mods).not.toContain("world3d");
  expect(mods).not.toContain("video");
  expect(mods).not.toContain("story");
});


