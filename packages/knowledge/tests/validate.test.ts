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
  expect(names).toEqual(["scaffold:origin-engine-ts", "skill:og-ai", "skill:og-audio", "skill:og-build", "skill:og-core", "skill:og-feel", "skill:og-game", "skill:og-i18n", "skill:og-platform-sdk", "skill:og-taste", "skill:og-ui"]);
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
    'game-scaffold/origin-engine-ts/origin-engine.json: pins engine 0.0.1, packages/engine is 9.9.9',
    'game-scaffold/origin-engine-ts/project.ogt: engine "0.0.1", packages/engine is 9.9.9',
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
      cat.styles.push({ ...cat.styles[3] });
      cat.styles.push({ ...cat.styles[4], slug: "a-new-look" });
      return JSON.stringify(cat, null, 2);
    }),
  );
  const cat = JSON.parse(readFileSync(join(real.pkg, "looks/catalogue.json"), "utf8")) as { styles: { slug: string }[] };
  const [a, b, c, d] = cat.styles.map((s) => s.slug);
  expect(p.sort()).toEqual(
    [
      `looks/catalogue.json: ${a}: needs exactly three checks a critic can judge on a frame`,
      `looks/catalogue.json: ${b}: support planned must name what is missing (the primitive or the authored art)`,
      `looks/catalogue.json: ${c}: names Stylise.posterise, which the generated API does not have`,
      `looks/catalogue.json: ${d}: a second style with this slug`,
      "looks/catalogue.json: a-new-look: look.preset is templates/looks/" + cat.styles[4]!.slug + ".ogt, expected templates/looks/a-new-look.ogt",
      "skills/og-taste/SKILL.md: the style catalogue (§12) does not list `a-new-look`",
    ].sort(),
  );
});
