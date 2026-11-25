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

test("brief vocabularies stay in lockstep with Rust cook/synth constants", async () => {
  const repo = join(real.pkg, "..", "..");
  const { MOODS, SFX_PRESETS } = await import("../scripts/brief.ts");

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
});
